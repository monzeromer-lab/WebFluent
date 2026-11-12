//! Parsing — builds an abstract syntax tree from tokens.
//!
//! Consumes a token stream from the [`crate::lexer`] and produces a [`Program`] AST
//! containing page declarations, components, stores, and an optional app declaration.

pub mod ast;
// `parser::parser` mirrors the crate layout the rest of the tree uses;
// flattening it would move every public path for no gain.
#[allow(clippy::module_inception)]
pub mod parser;
pub mod v2;
pub mod vocabulary;

pub use ast::*;

pub use parser::Parser;
// These are public API — a consumer reaches them as `webfluent::…`, and the LSP,
// the studio and the docs all do. Nothing inside this crate calls them, which is
// why `cargo clippy --fix` removed them; `#[allow]` records that the emptiness is
// expected rather than leaving the next person to delete them again.
#[allow(unused_imports)]
pub use vocabulary::{MODIFIER_KEYWORDS, is_modifier_keyword};

/// How deeply a program may nest: expressions inside expressions, blocks
/// inside blocks, and the links of one chain (`a.b.c`, `1 + 2 + 3`), which
/// nest as deeply as they are long. Past it the parser, the checker and the
/// template engine report `E120` instead of running out of stack. The
/// deepest program of the guide, the examples and the site nests 15 levels;
/// at 100, every pass that walks the tree fits in a 2 MB thread in a
/// release build (a debug build's frames are several times larger).
pub const MAX_NESTING: usize = 100;

/// The message `E120` carries, so every pass says it in one way.
pub const NESTING_MESSAGE: &str = "This nests too deeply";
