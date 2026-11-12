//! Error types and diagnostics for the WebFluent compiler.

pub mod diagnostic;

pub use diagnostic::{A11yWarning, Diagnostic, Result, VocabWarning, WebFluentError};
