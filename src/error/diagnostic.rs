use std::fmt;

/// A source location diagnostic with file, line, column, and optional hint.
///
/// Used by [`WebFluentError::LexerError`] and [`WebFluentError::ParseError`]
/// to provide precise error locations.
#[derive(Debug)]
pub struct Diagnostic {
    /// The error message.
    pub message: String,
    /// Source file path.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
    /// Optional fix suggestion.
    pub hint: Option<String>,
}

impl Diagnostic {
    /// Create a new diagnostic at the given source location.
    pub fn new(message: impl Into<String>, file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            message: message.into(),
            file: file.into(),
            line,
            column,
            hint: None,
        }
    }

    /// Add a hint (fix suggestion) to the diagnostic.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {} at {}:{}:{}", self.message, self.file, self.line, self.column)?;
        if let Some(hint) = &self.hint {
            write!(f, "\n  {}", hint)?;
        }
        Ok(())
    }
}

/// The error type for all WebFluent operations.
///
/// Covers lexing, parsing, code generation, configuration, and I/O errors.
#[derive(Debug)]
pub enum WebFluentError {
    /// Tokenization error (invalid characters, unterminated strings, etc.).
    LexerError(Diagnostic),
    /// Syntax error (unexpected token, missing brace, etc.).
    ParseError(Diagnostic),
    /// Code generation error.
    CodegenError(String),
    /// Configuration error (invalid `webfluent.app.json`).
    ConfigError(String),
    /// File I/O error.
    IoError(String),
}

impl fmt::Display for WebFluentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebFluentError::LexerError(d) => write!(f, "{}", d),
            WebFluentError::ParseError(d) => write!(f, "{}", d),
            WebFluentError::CodegenError(msg) => write!(f, "Codegen Error: {}", msg),
            WebFluentError::ConfigError(msg) => write!(f, "Config Error: {}", msg),
            WebFluentError::IoError(msg) => write!(f, "IO Error: {}", msg),
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
