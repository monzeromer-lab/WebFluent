//! Format a document: the same formatter `wf fmt` runs, so the editor and
//! the command line write a file the same way.

use tower_lsp::lsp_types::*;

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
    let name = if file.path.as_os_str().is_empty() {
        file.uri.path().to_string()
    } else {
        file.path.to_string_lossy().to_string()
    };
    let source: &str = &file.source;
    let formatted = webfluent::fmt::format_source(source, &name).ok()?;
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
