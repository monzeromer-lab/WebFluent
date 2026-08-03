//! Parser — builds an abstract syntax tree from tokens.
//!
//! Consumes a token stream from the [`crate::lexer`] and produces a [`Program`] AST
//! containing page declarations, components, stores, and an optional app declaration.

pub mod ast;
pub mod parser;
pub mod vocabulary;

pub use ast::*;
pub use parser::Parser;
pub use vocabulary::{MODIFIER_KEYWORDS, is_modifier_keyword};
