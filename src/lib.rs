#![allow(dead_code)]

pub mod lexer;
pub mod parser;
pub mod codegen;
pub mod runtime;
pub mod themes;
pub mod config;
pub mod error;
pub mod linter;
pub mod template;

pub use template::Template;
pub use error::{WebFluentError, Result};
