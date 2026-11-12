use std::fmt;

pub use crate::diagnostics::Diagnostic;

/// The error type for all WebFluent operations.
///
/// Covers lexing, parsing, code generation, configuration, and I/O errors.
#[derive(Debug)]
// Every variant names the stage it came from — `LexerError`, `ParseError`.
// The shared suffix is the point, and these are public API.
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
    /// What was asked for is not there: a file, or the text an edit names.
    /// Unlike a refusal, it is the asker's to correct.
    NotFound(String),
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
    #[must_use]
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
            WebFluentError::LexerError(d) => write!(f, "{d}"),
            WebFluentError::ParseError(d) => write!(f, "{d}"),
            WebFluentError::CodegenError(msg) => write!(f, "Codegen Error: {msg}"),
            WebFluentError::ConfigError(msg) => write!(f, "Config Error: {msg}"),
            WebFluentError::IoError(msg) => write!(f, "IO Error: {msg}"),
            WebFluentError::EditError(msg) => write!(f, "Edit Error: {msg}"),
            WebFluentError::NotFound(msg) => write!(f, "{msg}"),
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
