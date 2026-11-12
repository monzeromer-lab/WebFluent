//! Format a document: the same formatter `wf fmt` runs, so the editor and
//! the command line write a file the same way.

use lsp_types::*;

use crate::project::Project;

/// The edit that formats the file, or none when it is formatted already, is
/// not WebFluent (a script, a Markdown page), or does not parse — the
/// formatter never changes what a file says, so it refuses one it cannot
/// read.
pub fn format_document(project: &Project, file_ix: usize) -> Option<Vec<TextEdit>> {
    let file = &project.files[file_ix];
    if file.script || file.markdown {
        return None;
    }
    // The name picks the layout: a `.wfx` file is formatted as indented.
    let name = file.name();
    let source: &str = &file.source;
    let formatted = webfluent::syntax::fmt::format_source(source, &name).ok()?;
    if formatted == source {
        return Some(Vec::new());
    }
    Some(vec![TextEdit {
        range: Range::new(
            Position::new(0, 0),
            file.index.offset_to_position(source, source.len()),
        ),
        new_text: formatted,
    }])
}

/// The edits that format the lines `range` covers, and nothing outside
/// them: the whole file is formatted as `wf fmt` would, and each line's
/// change is kept when the line lies in the range.
///
/// The formatter changes white space only — indentation, a brace's spacing,
/// trailing blanks, runs of blank lines — so every line it writes is a line
/// of the file, or a blank one it folded away. A file whose lines cannot be
/// paired that way (an indented `.wfx` file, normalised through its braced
/// spelling) is not range-formatted: `None`, and the editor formats the
/// whole document instead.
pub fn format_range(project: &Project, file_ix: usize, range: Range) -> Option<Vec<TextEdit>> {
    let file = &project.files[file_ix];
    let whole = format_document(project, file_ix)?;
    let Some(edit) = whole.into_iter().next() else {
        return Some(Vec::new());
    };
    let source: &str = &file.source;
    let original: Vec<&str> = source.split('\n').collect();
    let formatted: Vec<&str> = edit.new_text.split('\n').collect();
    let bare = |line: &str| -> String { line.chars().filter(|c| !c.is_whitespace()).collect() };
    let line_start = |line: usize| -> Position { Position::new(line as u32, 0) };
    let line_end = |line: usize| -> Position {
        let text = original[line].strip_suffix('\r').unwrap_or(original[line]);
        file.index
            .offset_to_position(source, line_offset(&original, line) + text.len())
    };
    let wanted =
        |line: usize| (range.start.line as usize..=range.end.line as usize).contains(&line);
    let mut edits = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < original.len() || j < formatted.len() {
        match (original.get(i), formatted.get(j)) {
            (Some(a), Some(b)) if bare(a) == bare(b) => {
                let a = a.strip_suffix('\r').unwrap_or(a);
                if a != *b && wanted(i) {
                    edits.push(TextEdit {
                        range: Range::new(line_start(i), line_end(i)),
                        new_text: b.to_string(),
                    });
                }
                i += 1;
                j += 1;
            }
            // A blank line the formatter folded away.
            (Some(a), _) if a.trim().is_empty() => {
                if wanted(i) && i + 1 < original.len() {
                    edits.push(TextEdit {
                        range: Range::new(line_start(i), line_start(i + 1)),
                        new_text: String::new(),
                    });
                }
                i += 1;
            }
            // A blank line the formatter wrote.
            (_, Some(b)) if b.trim().is_empty() => {
                if wanted(i.min(original.len() - 1)) {
                    // Past the last line, the new line goes at its end.
                    let at = if i < original.len() {
                        line_start(i)
                    } else {
                        line_end(original.len() - 1)
                    };
                    edits.push(TextEdit {
                        range: Range::new(at, at),
                        new_text: "\n".to_string(),
                    });
                }
                j += 1;
            }
            _ => return None,
        }
    }
    Some(edits)
}

/// The byte offset at which line `line` (from 0) of the split lines begins.
fn line_offset(lines: &[&str], line: usize) -> usize {
    lines[..line].iter().map(|l| l.len() + 1).sum()
}
