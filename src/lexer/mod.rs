//! Lexical analysis — tokenizes `.wf` source code.
//!
//! The lexer converts raw source text into a stream of [`Token`]s:
//! identifiers, string literals (with `{var}` interpolation), numbers,
//! `$tokens`, operators, punctuation, and — inside a `style` or `theme`
//! block — raw CSS values.

// `lexer::lexer` mirrors the crate layout the rest of the tree uses;
// flattening it would move every public path for no gain.
#[allow(clippy::module_inception)]
pub mod lexer;
pub mod token;
pub mod v2;

pub use lexer::Lexer;
pub use token::{StringKind, StringLit, Token, TokenType};
pub use v2::LexerV2;
