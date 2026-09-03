use tower_lsp::lsp_types::{Position, Range};
use webfluent::parser::ast::Span;

/// An index mapping UTF-8 byte offsets to LSP 0-based UTF-16 line and character coordinates.
///
/// LSP coordinates are 0-based and count UTF-16 code units on each line.
/// Rust strings and WebFluent `Span`s use 0-based UTF-8 byte offsets.
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

    /// Convert a byte offset in `source` to an LSP `Position`.
    pub fn offset_to_position(&self, source: &str, offset: usize) -> Position {
        let offset = offset.min(self.total_len);

        // Binary search for the line index
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };

        let line_start = self.line_starts[line_idx];
        let line_slice = &source[line_start..offset];

        // Count UTF-16 code units in the slice
        let utf16_col = line_slice.encode_utf16().count() as u32;

        Position::new(line_idx as u32, utf16_col)
    }

    /// Convert an LSP `Position` to a UTF-8 byte offset in `source`.
    #[allow(dead_code)]
    pub fn position_to_offset(&self, source: &str, position: Position) -> Option<usize> {
        let line_idx = position.line as usize;
        if line_idx >= self.line_starts.len() {
            return None;
        }

        let line_start = self.line_starts[line_idx];
        let line_end = self
            .line_starts
            .get(line_idx + 1)
            .map(|&s| s.saturating_sub(1)) // exclude \n
            .unwrap_or(self.total_len);

        let line_str = source.get(line_start..line_end)?;
        let mut utf16_count = 0u32;
        let target_utf16 = position.character;

        for (byte_idx, ch) in line_str.char_indices() {
            if utf16_count >= target_utf16 {
                return Some(line_start + byte_idx);
            }
            utf16_count += ch.len_utf16() as u32;
        }

        if utf16_count >= target_utf16 {
            Some(line_start + line_str.len())
        } else {
            Some(line_end)
        }
    }

    /// Convert a WebFluent AST `Span` to an LSP `Range`.
    pub fn span_to_range(&self, source: &str, span: Span) -> Range {
        let start = self.offset_to_position(source, span.start as usize);
        let end = self.offset_to_position(source, span.end as usize);
        Range::new(start, end)
    }

    /// Convert a 1-based (line, column) coordinate (e.g. from a linter) to an LSP `Range`.
    pub fn line_col_to_range(&self, source: &str, line: usize, col: usize, len: usize) -> Range {
        let lsp_line = if line > 0 { line - 1 } else { 0 };
        let lsp_col = if col > 0 { col - 1 } else { 0 };

        if lsp_line < self.line_starts.len() {
            let line_start = self.line_starts[lsp_line];
            let line_end = self
                .line_starts
                .get(lsp_line + 1)
                .map(|&s| s.saturating_sub(1))
                .unwrap_or(self.total_len);

            let line_str = source.get(line_start..line_end).unwrap_or("");
            // Convert byte col to UTF-16
            let col_byte = lsp_col.min(line_str.len());
            let prefix = &line_str[..col_byte];
            let start_utf16 = prefix.encode_utf16().count() as u32;

            let end_byte = (col_byte + len).min(line_str.len());
            let span_str = &line_str[col_byte..end_byte];
            let span_utf16 = span_str.encode_utf16().count() as u32;

            Range::new(
                Position::new(lsp_line as u32, start_utf16),
                Position::new(
                    lsp_line as u32,
                    start_utf16 + if span_utf16 > 0 { span_utf16 } else { 1 },
                ),
            )
        } else {
            Range::new(
                Position::new(lsp_line as u32, lsp_col as u32),
                Position::new(lsp_line as u32, (lsp_col + len.max(1)) as u32),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_coordinates() {
        let src = "Page Home {\n  Button(\"Save\")\n}\n";
        let index = LineIndex::new(src);

        let pos = index.offset_to_position(src, 14); // 'B' of Button
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 2);

        let offset = index.position_to_offset(src, pos).unwrap();
        assert_eq!(offset, 14);
    }

    #[test]
    fn multibyte_coordinates() {
        // '🎉' is 4 bytes in UTF-8, but 2 code units in UTF-16
        let src = "Text(\"🎉 Hello\")\n";
        let index = LineIndex::new(src);

        // After the emoji
        let offset = "Text(\"🎉".len();
        let pos = index.offset_to_position(src, offset);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 8); // 'Text("' = 6 + 2 (UTF-16 emoji) = 8

        let back_offset = index.position_to_offset(src, pos).unwrap();
        assert_eq!(back_offset, offset);
    }
}
