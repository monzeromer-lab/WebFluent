//! The one door into the front end.
//!
//! Every consumer of a `.wf` file — the build, the template engine, the
//! structured editor, the studio, the language server — goes through here.
//!
//! WebFluent 3 has one grammar (see `spec/SYNTAX_V2.md`), written with
//! braces in a `.wf` file and by indentation in a `.wfx` file
//! ([`crate::lexer::v2::Layout`]). The grammar
//! WebFluent 2 had is read by `wf migrate` alone, through
//! [`crate::migrate`]; a file written in it is refused here with a pointer
//! to the migration. The dialect is read from a file's first word: the
//! declarations of WebFluent 3 are lowercase (`page`, `component`,
//! `store`, `theme`, `app`, `type`, `enum`); the old grammar's were
//! capitalised.

use crate::error::{Diagnostic, Result, WebFluentError};
use crate::lexer::Token;
use crate::parser::Program;

/// Which grammar a source text is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    /// The grammar of WebFluent 2: `Page Home (path: "/") { … }`. Read by
    /// `wf migrate` only.
    V1,
    /// WebFluent 3: `page Home(path: "/") { … }`.
    V2,
}

/// Whether `path` is a WebFluent source file: `.wf`, or `.wfx` for the
/// indented layout.
pub fn is_source_file(path: &std::path::Path) -> bool {
    path.extension().is_some_and(|e| e == "wf" || e == "wfx")
}

/// The dialect of `source`, read from its first word after comments.
///
/// A file that declares nothing — empty, or comments only — is the current
/// grammar, which is what a new file should be.
pub fn detect_dialect(source: &str) -> Dialect {
    match first_word(source) {
        Some("Page" | "Component" | "Store" | "App" | "Theme") => Dialect::V1,
        _ => Dialect::V2,
    }
}

/// The first word of `source` that is not inside a comment.
fn first_word(source: &str) -> Option<&str> {
    first_word_at(source).map(|(word, _)| word)
}

/// The first word outside a comment, with its byte offset.
fn first_word_at(source: &str) -> Option<(&str, usize)> {
    let mut rest = source;
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix("//") {
            rest = after.split_once('\n').map_or("", |(_, r)| r);
        } else if let Some(after) = rest.strip_prefix("/*") {
            rest = after.split_once("*/").map_or("", |(_, r)| r);
        } else {
            break;
        }
    }
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    (end > 0).then(|| (&rest[..end], source.len() - rest.len()))
}

/// Parse one file into a program.
///
/// `file` names the source in diagnostics. A file in the grammar of
/// WebFluent 2 is an error that names the migration.
pub fn parse_source(source: &str, file: &str) -> Result<Program> {
    match detect_dialect(source) {
        Dialect::V1 => {
            let (line, col) = position_of_first_word(source);
            Err(WebFluentError::ParseError(
                Diagnostic::new(
                    format!(
                        "`{}` is a WebFluent 2 declaration; this is WebFluent 3",
                        first_word(source).unwrap_or("Page")
                    ),
                    file,
                    line,
                    col,
                )
                .with_hint("Run `wf migrate` to convert the project to the current grammar"),
            ))
        }
        Dialect::V2 => crate::parser::v2::parse_v2(source, file),
    }
}

/// The line and column of the first word, for the diagnostic.
fn position_of_first_word(source: &str) -> (usize, usize) {
    let Some((_, at)) = first_word_at(source) else {
        return (1, 1);
    };
    let line = source[..at].matches('\n').count() + 1;
    let col = at - source[..at].rfind('\n').map(|i| i + 1).unwrap_or(0) + 1;
    (line, col)
}

/// The token stream of `source`, for tools that work at the token level
/// (the language server's cursor context).
pub fn tokens(source: &str, file: &str) -> Result<Vec<Token>> {
    crate::lexer::LexerV2::for_file(source, file).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_declaration_word_decides_the_dialect() {
        assert_eq!(detect_dialect("Page Home (path: \"/\") { }"), Dialect::V1);
        assert_eq!(
            detect_dialect("// a comment\n/* another */\nComponent Card () { }"),
            Dialect::V1
        );
        assert_eq!(detect_dialect("page Home(path: \"/\") { }"), Dialect::V2);
        assert_eq!(detect_dialect("enum Tone { neutral }"), Dialect::V2);
        assert_eq!(detect_dialect(""), Dialect::V2);
        assert_eq!(detect_dialect("// only a comment\n"), Dialect::V2);
    }

    #[test]
    fn the_old_grammar_is_refused_with_the_way_forward() {
        // The comment names the word too; the position is the declaration's.
        let err = parse_source(
            "// The Page\nPage Home (path: \"/\") { Text(\"hi\") }",
            "t.wf",
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("`Page` is a WebFluent 2 declaration"), "{err}");
        assert!(err.contains("wf migrate"), "{err}");
        assert!(err.contains("t.wf:2:1"), "{err}");
    }
}
