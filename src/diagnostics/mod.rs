//! What the compiler says about a program: one [`Diagnostic`] type for every
//! finding, a registry of the codes they carry ([`codes`]), the pipeline that
//! runs every check over a project ([`check`]), and the renderers that show
//! them to a person or a tool ([`render`]).
//!
//! Every finding has a code, a severity, a place — a start and, where the
//! compiler knows it, an end — a message and, where there is one, a hint, the
//! other places it concerns, and the edits that fix it. The build, the
//! language server, `wf test`, the docs tests and the studio all read the
//! same list, so the editor shows exactly what the build refuses.

pub mod check;
pub mod codes;
pub mod render;

use serde::Serialize;
use std::fmt;

/// How much a finding matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The program cannot mean what it says; the build stops.
    Error,
    /// The build goes on, and something is probably wrong.
    Warning,
    /// Worth knowing; nothing is wrong.
    Info,
}

impl Severity {
    /// The word a renderer writes: `error`, `warning`, `info`.
    pub fn word(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// Another place a finding concerns: the first of two declarations, the
/// route table a link was looked up in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Related {
    pub message: String,
    pub file: String,
    /// 1-based.
    pub line: usize,
    /// 1-based.
    pub column: usize,
}

/// One text edit: replace what lies between the start and the end (both
/// 1-based, the end exclusive) with `text`. An empty range inserts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edit {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub text: String,
}

/// A fix: what it does, in a few words, and the edits that do it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<Edit>,
}

/// What an editor may show about a finding besides its severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tag {
    /// Code nothing needs — an editor greys it out.
    Unnecessary,
    /// A form that still works and is on its way out.
    Deprecated,
}

/// A finding, where it is, and what to do about it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    /// The code, from the [`codes`] registry: `T05`, `E102`, `A01`.
    pub code: &'static str,
    pub severity: Severity,
    /// What is wrong, in one sentence.
    pub message: String,
    /// The file it is in, as the project names it (`src/pages/Home.wf`).
    pub file: String,
    /// 1-based line of the start.
    pub line: usize,
    /// 1-based column of the start.
    pub column: usize,
    /// 1-based line of the end, when the compiler knows it; `0` otherwise,
    /// and a renderer underlines the word at the start.
    pub end_line: usize,
    /// 1-based column one past the end; `0` when unknown.
    pub end_column: usize,
    /// What to do about it.
    pub hint: Option<String>,
    pub related: Vec<Related>,
    pub fixes: Vec<Fix>,
    pub tags: Vec<Tag>,
}

impl Diagnostic {
    /// A finding with a code, at a place. Its severity is the code's.
    pub fn coded(
        code: &'static str,
        message: impl Into<String>,
        file: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        let mut d = Self {
            code: "",
            severity: Severity::Error,
            message: message.into(),
            file: file.into(),
            line,
            column,
            end_line: 0,
            end_column: 0,
            hint: None,
            related: Vec::new(),
            fixes: Vec::new(),
            tags: Vec::new(),
        };
        d.set_code(code);
        d
    }

    /// A finding at a place, with no code yet: an error until
    /// [`with_code`](Self::with_code) says what it is. The compiler itself
    /// always gives one.
    pub fn new(
        message: impl Into<String>,
        file: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self::coded("", message, file, line, column)
    }

    /// The code, and with it the severity and tags the registry gives it.
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.set_code(code);
        self
    }

    fn set_code(&mut self, code: &'static str) {
        self.code = code;
        if let Some(info) = codes::info(code) {
            self.severity = info.severity;
            if info.unnecessary && !self.tags.contains(&Tag::Unnecessary) {
                self.tags.push(Tag::Unnecessary);
            }
        }
    }

    /// Add a hint (fix suggestion) to the diagnostic.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        let hint = hint.into();
        self.hint = if hint.is_empty() { None } else { Some(hint) };
        self
    }

    /// The severity, over the code's.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Where the finding ends (1-based, the column one past the end).
    pub fn with_end(mut self, end_line: usize, end_column: usize) -> Self {
        if end_line > self.line || (end_line == self.line && end_column > self.column) {
            self.end_line = end_line;
            self.end_column = end_column;
        }
        self
    }

    /// The whole of a node's span — its start and end — read against the
    /// text of its file, when the file is at hand.
    pub fn with_span(mut self, span: crate::parser::ast::Span, source: Option<&str>) -> Self {
        if span.line > 0 {
            self.line = span.line as usize;
            self.column = span.col as usize;
        }
        if let Some(source) = source
            && span.end > span.start
            && (span.end as usize) <= source.len()
            && let Some((line, column)) = line_col(source, span.end as usize)
        {
            self = self.with_end(line, column);
        }
        self
    }

    /// Another place the finding concerns.
    pub fn with_related(
        mut self,
        message: impl Into<String>,
        file: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        self.related.push(Related {
            message: message.into(),
            file: file.into(),
            line,
            column,
        });
        self
    }

    /// A fix.
    pub fn with_fix(mut self, title: impl Into<String>, edits: Vec<Edit>) -> Self {
        self.fixes.push(Fix {
            title: title.into(),
            edits,
        });
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// The finding written as a warning, whatever its severity: the same
    /// place and hint.
    pub fn as_warning(&self) -> String {
        let mut shown = self.clone();
        shown.severity = Severity::Warning;
        shown.to_string()
    }

    /// The link to the code's entry in the guide.
    pub fn docs_url(&self) -> Option<String> {
        codes::info(self.code).map(|_| codes::docs_url(self.code))
    }
}

/// 1-based line and column of a byte offset.
pub fn line_col(source: &str, offset: usize) -> Option<(usize, usize)> {
    if offset > source.len() || !source.is_char_boundary(offset) {
        return None;
    }
    let before = &source[..offset];
    let line = before.matches('\n').count() + 1;
    let column = match before.rfind('\n') {
        Some(nl) => before[nl + 1..].chars().count() + 1,
        None => before.chars().count() + 1,
    };
    Some((line, column))
}

/// `error[T05]: message`, then where, then the hint — the head of what the
/// human renderer writes, without the source line it shows when it has the
/// file.
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.code.is_empty() {
            write!(f, "{}: {}", self.severity.word(), self.message)?;
        } else {
            write!(
                f,
                "{}[{}]: {}",
                self.severity.word(),
                self.code,
                self.message
            )?;
        }
        write!(f, "\n  --> {}:{}:{}", self.file, self.line, self.column)?;
        if let Some(hint) = &self.hint {
            write!(f, "\n  = help: {hint}")?;
        }
        for r in &self.related {
            write!(
                f,
                "\n  = note: {} at {}:{}:{}",
                r.message, r.file, r.line, r.column
            )?;
        }
        Ok(())
    }
}

/// Drop the second and later of any finding with one code at one place
/// saying one thing, and order the rest by file and place. Two checks that
/// both see one mistake used to print it two or three times.
pub fn dedupe(diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = std::collections::HashSet::new();
    diagnostics
        .retain(|d| seen.insert((d.code, d.file.clone(), d.line, d.column, d.message.clone())));
    diagnostics.sort_by(|a, b| {
        (&a.file, a.line, a.column, a.severity).cmp(&(&b.file, b.line, b.column, b.severity))
    });
}

/// How many errors and warnings a list holds.
pub fn counts(diagnostics: &[Diagnostic]) -> (usize, usize) {
    let errors = diagnostics.iter().filter(|d| d.is_error()).count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    (errors, warnings)
}

/// `2 errors, 5 warnings` — the one line a build ends on.
pub fn summary(diagnostics: &[Diagnostic]) -> String {
    let (errors, warnings) = counts(diagnostics);
    let plural = |n: usize, word: &str| {
        if n == 1 {
            format!("1 {word}")
        } else {
            format!("{n} {word}s")
        }
    };
    match (errors, warnings) {
        (0, 0) => "no problems".to_string(),
        (0, w) => plural(w, "warning"),
        (e, 0) => plural(e, "error"),
        (e, w) => format!("{}, {}", plural(e, "error"), plural(w, "warning")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_brings_its_severity_and_tags() {
        let d = Diagnostic::coded("U01", "`x` is never read", "a.wf", 3, 5);
        assert_eq!(d.severity, Severity::Warning);
        assert_eq!(d.tags, vec![Tag::Unnecessary]);
        let e = Diagnostic::coded("T05", "no field", "a.wf", 1, 1);
        assert!(e.is_error());
        assert!(
            e.to_string()
                .starts_with("error[T05]: no field\n  --> a.wf:1:1")
        );
    }

    #[test]
    fn a_finding_seen_twice_is_said_once() {
        let mut list = vec![
            Diagnostic::coded("T05", "no field `nmae`", "b.wf", 5, 18),
            Diagnostic::coded("A01", "no alt", "a.wf", 2, 1),
            Diagnostic::coded("T05", "no field `nmae`", "b.wf", 5, 18),
        ];
        dedupe(&mut list);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].file, "a.wf", "ordered by file");
        assert_eq!(summary(&list), "1 error, 1 warning");
    }

    #[test]
    fn a_span_gives_the_end() {
        let src = "page P(path: \"/\") {\n    Heading(user.nmae).h1\n}";
        let start = src.find("user.nmae").unwrap() as u32;
        let span = crate::parser::ast::Span::new(start, start + 9, 2, 13);
        let d = Diagnostic::coded("T05", "x", "p.wf", 1, 1).with_span(span, Some(src));
        assert_eq!((d.line, d.column, d.end_line, d.end_column), (2, 13, 2, 22));
    }
}
