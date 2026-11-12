//! Error types and diagnostics.
//!
//! Provides [`WebFluentError`] (with variants for lexer, parser, codegen, config, and I/O errors),
//! and [`Diagnostic`] (with file, line, column, and optional hint), the one
//! finding type the checks report.
//!
//! Error types and diagnostics for the WebFluent compiler.

pub mod diagnostic;

pub use diagnostic::{
    CodegenError, ConfigError, Diagnostic, EditError, IoError, NotFound, Result, WebFluentError,
};
