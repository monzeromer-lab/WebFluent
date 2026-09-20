//! The one door into the front end.
//!
//! Every consumer of a `.wf` file — the build, the template engine, the
//! structured editor, the studio, the language server — used to lex and
//! parse on its own, four lines each. They all go through here now, so the
//! front end can decide *which* grammar a file is written in without every
//! caller knowing that there is more than one.
//!
//! WebFluent 3 introduces a second grammar (see `spec/SYNTAX_V2.md`). During
//! the transition a file's dialect is read from its first word: the new
//! grammar's declarations are lowercase (`page`, `component`, `store`,
//! `theme`, `app`, `type`, `enum`); the old grammar's are capitalised. Once
//! the transition is over the old grammar is only read by `wf migrate`.

use crate::error::Result;
use crate::lexer::{Lexer, Token};
use crate::parser::{Parser, Program};

/// Which grammar a source text is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    /// The original grammar: `Page Home (path: "/") { … }`.
    V1,
    /// WebFluent 3: `page Home(path: "/") { … }`.
    V2,
}

/// The dialect of `source`, read from its first word after comments.
///
/// A file that declares nothing — empty, or comments only — is the new
/// grammar, which is what a new file should be.
pub fn detect_dialect(source: &str) -> Dialect {
    match first_word(source) {
        Some("Page" | "Component" | "Store" | "App" | "Theme") => Dialect::V1,
        _ => Dialect::V2,
    }
}

/// The first word of `source` that is not inside a comment.
fn first_word(source: &str) -> Option<&str> {
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
    (end > 0).then(|| &rest[..end])
}

/// Parse one file into a program, in whichever dialect it is written.
///
/// `file` names the source in diagnostics.
pub fn parse_source(source: &str, file: &str) -> Result<Program> {
    match detect_dialect(source) {
        Dialect::V1 | Dialect::V2 => {
            let tokens = Lexer::new(source, file).tokenize()?;
            Parser::new(tokens, file).parse()
        }
    }
}

/// The token stream of `source`, for tools that work at the token level
/// (the language server's cursor context).
pub fn tokens(source: &str, file: &str) -> Result<Vec<Token>> {
    Lexer::new(source, file).tokenize()
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
    fn parse_source_reads_the_old_grammar() {
        let program = parse_source("Page Home (path: \"/\") { Text(\"hi\") }", "t.wf").unwrap();
        assert_eq!(program.declarations.len(), 1);
    }
}
