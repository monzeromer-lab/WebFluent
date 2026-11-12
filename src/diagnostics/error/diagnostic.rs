use std::fmt;

pub use crate::diagnostics::Diagnostic;

/// Defines an error type of one cause: a message, and the error beneath it
/// when there is one, which [`std::error::Error::source`] hands on.
macro_rules! message_error {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug)]
        pub struct $name {
            message: String,
            source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
        }

        impl $name {
            /// The error, saying `message`.
            pub fn new(message: impl Into<String>) -> Self {
                Self {
                    message: message.into(),
                    source: None,
                }
            }

            /// The error, saying `message`, with the error beneath it.
            pub fn with_source(
                message: impl Into<String>,
                source: impl std::error::Error + Send + Sync + 'static,
            ) -> Self {
                Self {
                    message: message.into(),
                    source: Some(Box::new(source)),
                }
            }

            /// What it says.
            pub fn message(&self) -> &str {
                &self.message
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.message)
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.source
                    .as_deref()
                    .map(|e| e as &(dyn std::error::Error + 'static))
            }
        }

        impl From<String> for $name {
            fn from(message: String) -> Self {
                Self::new(message)
            }
        }

        impl From<&str> for $name {
            fn from(message: &str) -> Self {
                Self::new(message)
            }
        }
    };
}

message_error!(
    /// A file that could not be read or written, or a process that could
    /// not be run. The message names the file; an [`std::io::Error`] is its
    /// source.
    IoError
);
message_error!(
    /// A configuration that could not be loaded: `webfluent.app.json` that
    /// is not a config, or a setting that cannot work.
    ConfigError
);
message_error!(
    /// Output that could not be made: a page, a bundle, a PDF.
    CodegenError
);
message_error!(
    /// A structured edit that could not be applied: an unknown node, a
    /// snippet that does not parse, edits that overlap.
    EditError
);
message_error!(
    /// What was asked for is not there: a file, a page, the text an edit
    /// names. Unlike a refusal, it is the asker's to correct.
    NotFound
);

impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        Self::with_source(err.to_string(), err)
    }
}

/// The error type for all WebFluent operations.
///
/// Covers lexing, parsing, code generation, configuration, and I/O errors.
/// A variant of one cause holds that cause's own error type, whose
/// [`source`](std::error::Error::source) is the error beneath it, so a
/// chain of causes can be walked.
#[derive(Debug)]
#[non_exhaustive]
// Every variant names the stage it came from — `LexerError`, `ParseError`.
// The shared suffix is the point, and these are public API.
pub enum WebFluentError {
    /// Tokenization error (invalid characters, unterminated strings, etc.).
    LexerError(Box<Diagnostic>),
    /// Syntax error (unexpected token, missing brace, etc.).
    ParseError(Box<Diagnostic>),
    /// Code generation error.
    CodegenError(CodegenError),
    /// Configuration error (invalid `webfluent.app.json`).
    ConfigError(ConfigError),
    /// File I/O error.
    IoError(IoError),
    /// Structured-edit error (unknown node id, invalid snippet, overlapping edits).
    EditError(EditError),
    /// What was asked for is not there: a file, or the text an edit names.
    /// Unlike a refusal, it is the asker's to correct.
    NotFound(NotFound),
    /// The checks found errors: every finding of the project, warnings
    /// included, already printed by the command that ran them.
    Diagnostics(Vec<Diagnostic>),
}

impl WebFluentError {
    /// An [`IoError`] saying `message`.
    pub fn io(message: impl Into<String>) -> Self {
        WebFluentError::IoError(IoError::new(message))
    }

    /// An [`IoError`] saying `message`, with the error that caused it as
    /// its [`source`](std::error::Error::source).
    pub fn io_with(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        WebFluentError::IoError(IoError::with_source(message, source))
    }

    /// A [`ConfigError`] saying `message`.
    pub fn config(message: impl Into<String>) -> Self {
        WebFluentError::ConfigError(ConfigError::new(message))
    }

    /// A [`ConfigError`] saying `message`, with the error that caused it
    /// as its [`source`](std::error::Error::source).
    pub fn config_with(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        WebFluentError::ConfigError(ConfigError::with_source(message, source))
    }

    /// A [`CodegenError`] saying `message`.
    pub fn codegen(message: impl Into<String>) -> Self {
        WebFluentError::CodegenError(CodegenError::new(message))
    }

    /// An [`EditError`] saying `message`.
    pub fn edit(message: impl Into<String>) -> Self {
        WebFluentError::EditError(EditError::new(message))
    }

    /// A [`NotFound`] saying `message`.
    pub fn not_found(message: impl Into<String>) -> Self {
        WebFluentError::NotFound(NotFound::new(message))
    }

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
            WebFluentError::CodegenError(e) => write!(f, "Codegen Error: {e}"),
            WebFluentError::ConfigError(e) => write!(f, "Config Error: {e}"),
            WebFluentError::IoError(e) => write!(f, "IO Error: {e}"),
            WebFluentError::EditError(e) => write!(f, "Edit Error: {e}"),
            WebFluentError::NotFound(e) => write!(f, "{e}"),
            WebFluentError::Diagnostics(list) => {
                for d in list.iter().filter(|d| d.is_error()) {
                    writeln!(f, "{d}")?;
                }
                write!(f, "{}", crate::diagnostics::summary(list))
            }
        }
    }
}

impl std::error::Error for WebFluentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WebFluentError::CodegenError(e) => Some(e),
            WebFluentError::ConfigError(e) => Some(e),
            WebFluentError::IoError(e) => Some(e),
            WebFluentError::EditError(e) => Some(e),
            WebFluentError::NotFound(e) => Some(e),
            WebFluentError::LexerError(_)
            | WebFluentError::ParseError(_)
            | WebFluentError::Diagnostics(_) => None,
        }
    }
}

impl From<std::io::Error> for WebFluentError {
    fn from(err: std::io::Error) -> Self {
        WebFluentError::IoError(err.into())
    }
}

// A server holds and sends these across threads and keeps them in a
// `Box<dyn Error + Send + Sync>`: held to that here, so a change that
// breaks it fails to compile rather than surprising a user.
const _: () = {
    const fn holds<T: Send + Sync + 'static>() {}
    holds::<WebFluentError>();
};

/// A specialized `Result` type for WebFluent operations.
pub type Result<T> = std::result::Result<T, WebFluentError>;
