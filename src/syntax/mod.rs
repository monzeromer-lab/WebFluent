//! The one door into the front end.
//!
//! Every consumer of a `.wf` file — the build, the template engine, the
//! structured editor, the studio, the language server — goes through here.
//!
//! WebFluent 3 has one grammar (see `spec/SYNTAX_V2.md`), written with
//! braces in a `.wf` file and by indentation in a `.wfx` file
//! (`crate::syntax::lexer::v2::Layout`). The grammar
//! WebFluent 2 had is read by `wf migrate` alone, through
//! `wf migrate`; a file written in it is refused here with a pointer
//! to the migration. The dialect is read from a file's first word: the
//! declarations of WebFluent 3 are lowercase (`page`, `component`,
//! `store`, `theme`, `app`, `type`, `enum`); the old grammar's were
//! capitalised.

pub mod fmt;
pub mod js_scan;
pub mod layout;
internal! {
    pub mod lexer;
}
pub mod node_id;
pub mod parser;
internal! {
    pub mod print;
}
internal! {
    pub mod reprint;
}
use crate::diagnostics::error::{Diagnostic, Result, WebFluentError};
use crate::syntax::lexer::Token;
use crate::syntax::parser::Program;

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
///
/// # Errors
///
/// [`WebFluentError::LexerError`] when the source does not lex,
/// [`WebFluentError::ParseError`] when it does not parse (or is in the
/// WebFluent 2 grammar, `E004`), and [`WebFluentError::Diagnostics`] when
/// several declarations do not parse.
pub fn parse_source(source: &str, file: &str) -> Result<Program> {
    match detect_dialect(source) {
        Dialect::V1 => {
            let (line, col) = position_of_first_word(source);
            Err(WebFluentError::ParseError(Box::new(
                Diagnostic::coded(
                    "E004",
                    format!(
                        "`{}` is a WebFluent 2 declaration; this is WebFluent 3",
                        first_word(source).unwrap_or("Page")
                    ),
                    file,
                    line,
                    col,
                )
                .with_hint("Run `wf migrate` to convert the project to the current grammar"),
            )))
        }
        Dialect::V2 => {
            let mut program = crate::syntax::parser::v2::parse_v2(source, file)?;
            prefer_own_components(&mut program);
            Ok(program)
        }
    }
}

/// Parse one file, keeping going after an error: every declaration that
/// parses, and a diagnostic for each mistake — so one typo does not hide the
/// rest of the file. A file the lexer cannot read, or one in the grammar of
/// WebFluent 2, is one error and no declarations.
pub fn parse_source_recovering(source: &str, file: &str) -> (Program, Vec<Diagnostic>) {
    let empty = || Program {
        declarations: Vec::new(),
    };
    if detect_dialect(source) == Dialect::V1 {
        let err = parse_source(source, file)
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default();
        return (empty(), err);
    }
    match crate::syntax::lexer::LexerV2::for_file(source, file).tokenize() {
        Ok(tokens) => {
            let (mut program, errors) =
                crate::syntax::parser::v2::ParserV2::new(tokens, file).parse_recovering();
            prefer_own_components(&mut program);
            (program, errors)
        }
        Err(e) => (empty(), e.diagnostics()),
    }
}

/// A component the program declares is what a call of its name draws, even
/// where a built-in has that name: the project's `Callout`, not the one a
/// later WebFluent brought. Without this, every built-in added to the
/// language would quietly take the place of a component projects already
/// had. The parser cannot know what the program declares, so a reader of
/// a whole program — one file, or every file merged — calls this.
pub fn prefer_own_components(program: &mut Program) {
    use crate::syntax::parser::ast::{ComponentRef, Declaration, Statement, StatementKind};
    use std::collections::HashSet;

    fn redirect(stmts: &mut [Statement], own: &HashSet<String>) {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &mut stmt.kind {
                if let ComponentRef::BuiltIn(name) = &el.component
                    && own.contains(name)
                {
                    el.component = ComponentRef::UserDefined(name.clone());
                }
                redirect(&mut el.children, own);
                for handler in &mut el.events {
                    redirect(&mut handler.body, own);
                }
                for fill in &mut el.slot_fills {
                    redirect(&mut fill.body, own);
                }
            }
            for body in stmt.kind.bodies_mut() {
                redirect(body, own);
            }
        }
    }

    let own: HashSet<String> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();
    if own.is_empty() {
        return;
    }
    for decl in &mut program.declarations {
        let body = match decl {
            Declaration::Page(p) => &mut p.body,
            Declaration::Component(c) => &mut c.body,
            Declaration::Store(s) => &mut s.body,
            Declaration::App(a) => &mut a.body,
            Declaration::Test(t) => &mut t.body,
            _ => continue,
        };
        redirect(body, &own);
    }
}

/// The line and column of the first word, for the diagnostic.
fn position_of_first_word(source: &str) -> (usize, usize) {
    let Some((_, at)) = first_word_at(source) else {
        return (1, 1);
    };
    let line = source[..at].matches('\n').count() + 1;
    let col = at - source[..at].rfind('\n').map_or(0, |i| i + 1) + 1;
    (line, col)
}

internal! {
    /// The token stream of `source`, for tools that work at the token level
    /// (the language server's cursor context).
    ///
    /// # Errors
    ///
    /// [`WebFluentError::LexerError`] when the source does not lex.
    pub fn tokens(source: &str, file: &str) -> Result<Vec<Token>> {
        crate::syntax::lexer::LexerV2::for_file(source, file).tokenize()
    }
}

/// The `//` comments of `source` — line, column and the text after the
/// slashes — as the lexer finds them, so text in a string, a raw string or
/// a splice is never one. A file that does not lex has none.
pub fn comments(source: &str, file: &str) -> Vec<(usize, usize, String)> {
    let mut lexer = crate::syntax::lexer::LexerV2::for_file(source, file);
    match lexer.tokenize() {
        Ok(_) => lexer.comments().to_vec(),
        Err(_) => Vec::new(),
    }
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

    #[test]
    fn a_file_reports_each_broken_declaration_and_keeps_the_rest() {
        let src = "page A(path: \"/a\") { Text(\"a\") }\n\
                   component B( { Text(\"b\") }\n\
                   page C(path: \"/c\") { Text(\"c\") }\n\
                   store D { state = 1 }\n\
                   page E(path: \"/e\") { Text(\"e\") }\n";
        let (program, errors) = parse_source_recovering(src, "t.wf");
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert_eq!((errors[0].line, errors[1].line), (2, 4));
        assert!(errors.iter().all(|e| e.code == "E002"));
        assert_eq!(program.declarations.len(), 3, "A, C and E still parse");
    }
}
