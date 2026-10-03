use std::fmt;

pub use crate::diagnostics::Diagnostic;

/// The error type for all WebFluent operations.
///
/// Covers lexing, parsing, code generation, configuration, and I/O errors.
#[derive(Debug)]
// Every variant names the stage it came from — `LexerError`, `ParseError`.
// The shared suffix is the point, and these are public API.
#[allow(clippy::enum_variant_names)]
pub enum WebFluentError {
    /// Tokenization error (invalid characters, unterminated strings, etc.).
    LexerError(Box<Diagnostic>),
    /// Syntax error (unexpected token, missing brace, etc.).
    ParseError(Box<Diagnostic>),
    /// Code generation error.
    CodegenError(String),
    /// Configuration error (invalid `webfluent.app.json`).
    ConfigError(String),
    /// File I/O error.
    IoError(String),
    /// Structured-edit error (unknown node id, invalid snippet, overlapping edits).
    EditError(String),
    /// The checks found errors: every finding of the project, warnings
    /// included, already printed by the command that ran them.
    Diagnostics(Vec<Diagnostic>),
}

impl WebFluentError {
    /// What a command exits with: `1` when the program has problems, `2`
    /// when the build could not run at all — a file it could not read or
    /// write, a configuration it could not load.
    pub fn exit_code(&self) -> i32 {
        match self {
            WebFluentError::ConfigError(_) | WebFluentError::IoError(_) => 2,
            _ => 1,
        }
    }

    /// The same error with its finding given another code.
    pub fn with_code(self, code: &'static str) -> Self {
        match self {
            WebFluentError::LexerError(d) => {
                WebFluentError::LexerError(Box::new(d.with_code(code)))
            }
            WebFluentError::ParseError(d) => {
                WebFluentError::ParseError(Box::new(d.with_code(code)))
            }
            other => other,
        }
    }

    /// Every finding this error carries.
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        match self {
            WebFluentError::LexerError(d) | WebFluentError::ParseError(d) => vec![(**d).clone()],
            WebFluentError::Diagnostics(list) => list.clone(),
            _ => Vec::new(),
        }
    }
}

impl fmt::Display for WebFluentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebFluentError::LexerError(d) => write!(f, "{}", d),
            WebFluentError::ParseError(d) => write!(f, "{}", d),
            WebFluentError::CodegenError(msg) => write!(f, "Codegen Error: {}", msg),
            WebFluentError::ConfigError(msg) => write!(f, "Config Error: {}", msg),
            WebFluentError::IoError(msg) => write!(f, "IO Error: {}", msg),
            WebFluentError::EditError(msg) => write!(f, "Edit Error: {}", msg),
            WebFluentError::Diagnostics(list) => {
                for d in list.iter().filter(|d| d.is_error()) {
                    writeln!(f, "{d}")?;
                }
                write!(f, "{}", crate::diagnostics::summary(list))
            }
        }
    }
}

impl std::error::Error for WebFluentError {}

impl From<std::io::Error> for WebFluentError {
    fn from(err: std::io::Error) -> Self {
        WebFluentError::IoError(err.to_string())
    }
}

/// A specialized `Result` type for WebFluent operations.
pub type Result<T> = std::result::Result<T, WebFluentError>;

/// A non-fatal accessibility warning emitted during compilation.
///
/// These warnings are displayed during `wf build` but never block compilation.
/// Rules cover WCAG basics: image alt text, form labels, heading hierarchy, etc.
#[derive(Debug)]
pub struct A11yWarning {
    /// Rule identifier (e.g., `"A01"` for missing alt text).
    pub rule_id: String,
    /// Human-readable warning message.
    pub message: String,
    /// Source file path.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
    /// Suggested fix.
    pub hint: String,
}

impl A11yWarning {
    /// Create a new accessibility warning.
    pub fn new(
        rule_id: impl Into<String>,
        message: impl Into<String>,
        file: impl Into<String>,
        line: usize,
        column: usize,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            message: message.into(),
            file: file.into(),
            line,
            column,
            hint: hint.into(),
        }
    }
}

impl fmt::Display for A11yWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  Warning [{}]: {} at {}:{}:{}\n    {}",
            self.rule_id, self.message, self.file, self.line, self.column, self.hint
        )
    }
}

/// A vocabulary warning (studio ask A-1): a bare word in argument position that
/// resolves to nothing — not a modifier, not anything in scope — and therefore
/// does nothing, silently. Warning severity by construction: it has its own
/// type and entry point (`lint_vocabulary`) precisely so it cannot fail a
/// compile that succeeds today; promotion to a gate error is a separate,
/// later decision once existing projects are clean.
#[derive(Debug, Clone, PartialEq)]
pub struct VocabWarning {
    /// Rule identifier (`"V01"`: dead bare-word argument).
    pub rule_id: String,
    /// Human-readable warning message.
    pub message: String,
    /// Source file path.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
    /// Suggested fix ("did you mean `outlined`?"), when something is close
    /// enough to suggest.
    pub hint: Option<String>,
}

impl fmt::Display for VocabWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  Warning [{}]: {} at {}:{}:{}",
            self.rule_id, self.message, self.file, self.line, self.column
        )?;
        if let Some(hint) = &self.hint {
            write!(f, "\n    {}", hint)?;
        }
        Ok(())
    }
}

impl From<A11yWarning> for Diagnostic {
    fn from(w: A11yWarning) -> Self {
        let code = crate::diagnostics::codes::info(&w.rule_id)
            .map(|c| c.code)
            .unwrap_or("");
        Diagnostic::coded(code, w.message, w.file, w.line, w.column).with_hint(w.hint)
    }
}

impl From<VocabWarning> for Diagnostic {
    fn from(w: VocabWarning) -> Self {
        let code = crate::diagnostics::codes::info(&w.rule_id)
            .map(|c| c.code)
            .unwrap_or("");
        let d = Diagnostic::coded(code, w.message, w.file, w.line, w.column);
        match w.hint {
            Some(hint) => d.with_hint(hint),
            None => d,
        }
    }
}
