//! Error types and diagnostics.
//!
//! Provides [`WebFluentError`] (with variants for lexer, parser, codegen, config, and I/O errors),
//! [`Diagnostic`] (with file, line, column, and optional hint),
//! and [`A11yWarning`] for accessibility lint results.
//!
//! Error types and diagnostics for the WebFluent compiler.

pub mod diagnostic;

pub use diagnostic::{A11yWarning, Diagnostic, Result, VocabWarning, WebFluentError};
