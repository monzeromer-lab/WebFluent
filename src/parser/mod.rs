//! Parser — builds an abstract syntax tree from tokens.
//!
//! Consumes a token stream from the [`crate::lexer`] and produces a [`Program`] AST
//! containing page declarations, components, stores, and an optional app declaration.

pub mod ast;
// `parser::parser` mirrors the crate layout the rest of the tree uses;
// flattening it would move every public path for no gain.
#[allow(clippy::module_inception)]
pub mod parser;
pub mod vocabulary;

pub use ast::*;
pub use parser::Parser;
// These are public API — a consumer reaches them as `webfluent::…`, and the LSP,
// the studio and the docs all do. Nothing inside this crate calls them, which is
// why `cargo clippy --fix` removed them; `#[allow]` records that the emptiness is
// expected rather than leaving the next person to delete them again.
#[allow(unused_imports)]
pub use vocabulary::{MODIFIER_KEYWORDS, is_modifier_keyword};
