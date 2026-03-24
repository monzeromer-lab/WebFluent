use std::fmt;

#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            message: message.into(),
            file: file.into(),
            line,
            column,
            hint: None,
        }
    }

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

#[derive(Debug)]
pub enum WebFluentError {
    LexerError(Diagnostic),
    ParseError(Diagnostic),
    CodegenError(String),
    ConfigError(String),
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

pub type Result<T> = std::result::Result<T, WebFluentError>;

/// A non-fatal accessibility warning emitted during compilation.
#[derive(Debug)]
pub struct A11yWarning {
    pub rule_id: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub hint: String,
}

impl A11yWarning {
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
