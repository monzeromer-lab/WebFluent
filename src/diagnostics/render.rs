//! Findings, written for a person.
//!
//! ```text
//! error[T05]: `User` has no field `nmae`
//!  --> src/pages/Profile.wf:5:18
//!    |
//!  5 |     Heading(user.nmae).h1
//!    |                  ^^^^
//!    = help: its fields are `id`, `name`, `email`
//!    = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t05-a-field-or-method-that-does-not-exist
//! ```
//!
//! The place comes first on its own line in the `file:line:col` form every
//! editor's and CI's problem matcher reads. Colour when the stream is a
//! terminal and `NO_COLOR` is not set.

use super::{Diagnostic, Severity};
use std::io::IsTerminal;

/// Whether standard error should be coloured.
pub fn stderr_wants_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal()
}

struct Paint {
    on: bool,
}

impl Paint {
    fn wrap(&self, code: &str, text: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
    fn severity(&self, s: Severity, text: &str) -> String {
        match s {
            Severity::Error => self.wrap("1;31", text),
            Severity::Warning => self.wrap("1;33", text),
            Severity::Info => self.wrap("1;36", text),
        }
    }
    fn blue(&self, text: &str) -> String {
        self.wrap("1;34", text)
    }
    fn bold(&self, text: &str) -> String {
        self.wrap("1", text)
    }
}

/// One finding, with the line of source it points at when `source` has the
/// file's text.
pub fn diagnostic(d: &Diagnostic, source: Option<&str>, color: bool) -> String {
    let paint = Paint { on: color };
    let head = if d.code.is_empty() {
        d.severity.word().to_string()
    } else {
        format!("{}[{}]", d.severity.word(), d.code)
    };
    let mut out = format!(
        "{}{}",
        paint.severity(d.severity, &head),
        paint.bold(&format!(": {}", d.message))
    );
    let line_text = source.and_then(|s| s.lines().nth(d.line.saturating_sub(1)));
    let gutter = d.line.to_string().len().max(1);
    let pad = " ".repeat(gutter);
    out.push_str(&format!(
        "\n{pad}{} {}:{}:{}",
        paint.blue("-->"),
        d.file,
        d.line,
        d.column
    ));
    if let Some(text) = line_text {
        let text = text.replace('\t', "    ");
        let bar = paint.blue("|");
        let (from, width) = underline(d, &text);
        out.push_str(&format!("\n{pad} {bar}"));
        out.push_str(&format!(
            "\n{} {bar} {}",
            paint.blue(&d.line.to_string()),
            text.trim_end()
        ));
        out.push_str(&format!(
            "\n{pad} {bar} {}{}",
            " ".repeat(from),
            paint.severity(d.severity, &"^".repeat(width))
        ));
    }
    if let Some(hint) = &d.hint {
        let mut lines = hint.lines();
        if let Some(first) = lines.next() {
            out.push_str(&format!("\n{pad} {} {first}", paint.blue("= help:")));
            for more in lines {
                out.push_str(&format!("\n{pad}         {more}"));
            }
        }
    }
    for r in &d.related {
        out.push_str(&format!(
            "\n{pad} {} {} at {}:{}:{}",
            paint.blue("= note:"),
            r.message,
            r.file,
            r.line,
            r.column
        ));
    }
    if let Some(url) = d.docs_url() {
        out.push_str(&format!("\n{pad} {} {url}", paint.blue("= docs:")));
    }
    out
}

/// Where the underline starts on the line (0-based, in characters) and how
/// wide it is: the whole span when it ends on this line, the rest of the line
/// when it runs on, and the word at the start when the end is unknown.
fn underline(d: &Diagnostic, text: &str) -> (usize, usize) {
    let chars: Vec<char> = text.chars().collect();
    let from = d.column.saturating_sub(1).min(chars.len());
    let width = if d.end_line == d.line && d.end_column > d.column {
        d.end_column - d.column
    } else if d.end_line > d.line {
        chars.len().saturating_sub(from)
    } else {
        chars[from..]
            .iter()
            .take_while(|c| c.is_alphanumeric() || **c == '_' || **c == '.' || **c == '$')
            .count()
    };
    (
        from,
        width.max(1).min(chars.len().saturating_sub(from).max(1)),
    )
}

/// Every finding, each followed by a blank line, then the summary. `source`
/// reads a file by the name a finding carries.
pub fn human(
    diagnostics: &[Diagnostic],
    source: &dyn Fn(&str) -> Option<String>,
    color: bool,
) -> String {
    let mut out = String::new();
    let mut cache: std::collections::HashMap<String, Option<String>> = Default::default();
    for d in diagnostics {
        let text = cache
            .entry(d.file.clone())
            .or_insert_with(|| source(&d.file))
            .as_deref();
        out.push_str(&diagnostic(d, text, color));
        out.push_str("\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finding_shows_its_line_and_underlines_its_span() {
        let src = "page P(path: \"/\") {\n    Heading(user.nmae).h1\n}\n";
        let d = Diagnostic::coded("T05", "`User` has no field `nmae`", "src/P.wf", 2, 13)
            .with_end(2, 22)
            .with_hint("its fields are `id`, `name`");
        let out = diagnostic(&d, Some(src), false);
        assert_eq!(
            out,
            "error[T05]: `User` has no field `nmae`\n --> src/P.wf:2:13\n  |\n2 |     Heading(user.nmae).h1\n  |             ^^^^^^^^^\n  = help: its fields are `id`, `name`\n  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t05-a-field-or-method-that-does-not-exist"
        );
    }

    #[test]
    fn with_no_end_the_word_is_underlined_and_without_the_file_none_is() {
        let src = "    Buton(\"Save\")";
        let d = Diagnostic::coded("E101", "unknown component `Buton`", "a.wf", 1, 5);
        assert!(diagnostic(&d, Some(src), false).contains("    ^^^^^\n"));
        let bare = diagnostic(&d, None, false);
        assert!(!bare.contains('|'), "{bare}");
        assert!(bare.starts_with("error[E101]: unknown component `Buton`\n --> a.wf:1:5"));
    }

    #[test]
    fn colour_is_only_on_request() {
        let d = Diagnostic::coded("A01", "no alt", "a.wf", 1, 1);
        assert!(!diagnostic(&d, None, false).contains('\x1b'));
        assert!(diagnostic(&d, None, true).contains("\x1b[1;33mwarning[A01]"));
    }
}
