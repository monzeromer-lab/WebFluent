//! Coordinate conversion between the three systems in play.
//!
//! - The compiler's [`Span`]s are UTF-8 **byte** offsets.
//! - The compiler's diagnostics carry a 1-based line and a 1-based **character**
//!   column (the lexer advances its column once per `char`).
//! - LSP positions are 0-based lines and 0-based **UTF-16 code unit** columns.
//!
//! Mixing the three is the classic way an editor's squiggles drift on lines
//! with Arabic, an em dash or an emoji, so every conversion goes through here.

use tower_lsp::lsp_types::{Position, Range};
use webfluent::parser::ast::Span;

#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of the start of each line.
    line_starts: Vec<usize>,
    /// Total byte length of the source.
    total_len: usize,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self {
            line_starts,
            total_len: source.len(),
        }
    }

    /// Number of lines (a trailing newline starts an empty last line).
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Byte range of a 0-based line, newline excluded.
    fn line_bounds(&self, line: usize) -> Option<(usize, usize)> {
        let start = *self.line_starts.get(line)?;
        let end = self
            .line_starts
            .get(line + 1)
            .map(|&next| next.saturating_sub(1))
            .unwrap_or(self.total_len);
        Some((start, end.max(start)))
    }

    /// Convert a byte offset in `source` to an LSP `Position`.
    pub fn offset_to_position(&self, source: &str, offset: usize) -> Position {
        let offset = offset.min(self.total_len);
        let line = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_starts[line];
        let slice = source.get(line_start..offset).unwrap_or("");
        Position::new(line as u32, slice.encode_utf16().count() as u32)
    }

    /// Convert an LSP `Position` to a byte offset in `source`.
    ///
    /// A column past the end of the line clamps to the end of the line, as the
    /// protocol asks; a line past the end of the file is `None`.
    pub fn position_to_offset(&self, source: &str, position: Position) -> Option<usize> {
        let (start, end) = self.line_bounds(position.line as usize)?;
        let line = source.get(start..end)?;
        let mut units = 0u32;
        for (byte, ch) in line.char_indices() {
            if units >= position.character {
                return Some(start + byte);
            }
            units += ch.len_utf16() as u32;
        }
        Some(end)
    }

    /// Byte offset of a compiler coordinate: 1-based line, 1-based character
    /// column. Clamps to the line, so a column past its end lands on the end.
    pub fn line_col_to_offset(&self, source: &str, line: usize, col: usize) -> Option<usize> {
        let (start, end) = self.line_bounds(line.saturating_sub(1))?;
        let text = source.get(start..end)?;
        let target = col.saturating_sub(1);
        let byte = text
            .char_indices()
            .nth(target)
            .map(|(b, _)| b)
            .unwrap_or(text.len());
        Some(start + byte)
    }

    /// Convert a compiler `Span` to an LSP `Range`.
    pub fn span_to_range(&self, source: &str, span: Span) -> Range {
        Range::new(
            self.offset_to_position(source, span.start as usize),
            self.offset_to_position(source, span.end as usize),
        )
    }

    /// The range of the word at a compiler coordinate — the identifier a
    /// diagnostic is about — or a single character when nothing word-like
    /// starts there.
    pub fn word_range_at_line_col(&self, source: &str, line: usize, col: usize) -> Range {
        let Some(start) = self.line_col_to_offset(source, line, col) else {
            let line = line.saturating_sub(1) as u32;
            let col = col.saturating_sub(1) as u32;
            return Range::new(Position::new(line, col), Position::new(line, col + 1));
        };
        let end = word_end(source, start);
        let end = if end == start {
            source[start..]
                .chars()
                .next()
                .map(|c| start + c.len_utf8())
                .unwrap_or(start)
        } else {
            end
        };
        Range::new(
            self.offset_to_position(source, start),
            self.offset_to_position(source, end),
        )
    }
}

/// Whether `c` can be part of a WebFluent name: an identifier character, or
/// the hyphen that joins `aria-pressed` and `border-radius`.
pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

/// Byte offset one past the end of the word that starts at `start`.
pub fn word_end(source: &str, start: usize) -> usize {
    let tail = &source[start..];
    let len: usize = tail
        .chars()
        .take_while(|&c| is_word_char(c))
        .map(char::len_utf8)
        .sum();
    start + len
}

/// The word touching byte `offset` (an identifier, keyword or hyphenated
/// name) and its byte range, if any.
pub fn word_at(source: &str, offset: usize) -> Option<(&str, std::ops::Range<usize>)> {
    let offset = offset.min(source.len());
    // Step back to a char boundary, then over any word characters.
    let mut start = offset;
    while !source.is_char_boundary(start) {
        start -= 1;
    }
    while let Some(prev) = source[..start].chars().next_back() {
        if is_word_char(prev) {
            start -= prev.len_utf8();
        } else {
            break;
        }
    }
    let end = word_end(source, start);
    if start == end {
        return None;
    }
    Some((&source[start..end], start..end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_coordinates() {
        let src = "Page Home {\n  Button(\"Save\")\n}\n";
        let index = LineIndex::new(src);
        let pos = index.offset_to_position(src, 14); // 'B' of Button
        assert_eq!((pos.line, pos.character), (1, 2));
        assert_eq!(index.position_to_offset(src, pos), Some(14));
    }

    #[test]
    fn multibyte_coordinates_round_trip() {
        // '🎉' is 4 bytes in UTF-8, 2 code units in UTF-16, 1 char.
        let src = "Text(\"🎉 Hello\")\n";
        let index = LineIndex::new(src);
        let offset = "Text(\"🎉".len();
        let pos = index.offset_to_position(src, offset);
        assert_eq!((pos.line, pos.character), (0, 8));
        assert_eq!(index.position_to_offset(src, pos), Some(offset));
    }

    #[test]
    fn compiler_columns_count_chars_not_bytes() {
        // The word after the Arabic string starts at char column 22 (1-based)
        // but at a much later byte, and at UTF-16 unit 21.
        let src = "Text(\"أهلاً بالعالم\", muted)\n";
        let index = LineIndex::new(src);
        let col = src.chars().position(|c| c == 'm').unwrap() + 1;
        let range = index.word_range_at_line_col(src, 1, col);
        let expected_start = index.offset_to_position(src, src.find("muted").unwrap());
        assert_eq!(range.start, expected_start);
        assert_eq!(range.end.character, range.start.character + 5);
    }

    #[test]
    fn word_at_handles_hyphens_and_multibyte_neighbours() {
        let src = "Button(\"ö\", aria-pressed: on)";
        let at = src.find("pressed").unwrap();
        let (word, range) = word_at(src, at).unwrap();
        assert_eq!(word, "aria-pressed");
        assert_eq!(&src[range], "aria-pressed");
        // Just before the `(` the word is `Button`; on the `"` there is none.
        assert_eq!(
            word_at(src, src.find('(').unwrap()).map(|w| w.0),
            Some("Button")
        );
        assert_eq!(
            word_at(src, src.find("\"ö").unwrap() + 1).map(|w| w.0),
            Some("ö")
        );
    }

    #[test]
    fn positions_past_the_line_end_clamp() {
        let src = "Text(\"x\")\n";
        let index = LineIndex::new(src);
        assert_eq!(index.position_to_offset(src, Position::new(0, 99)), Some(9));
        assert_eq!(index.position_to_offset(src, Position::new(5, 0)), None);
    }
}
