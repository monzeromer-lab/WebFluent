//! Lexical analysis — tokenizes `.wf` source code.
//!
//! The lexer ([`LexerV2`]) converts raw source text into a stream of
//! [`Token`]s: identifiers, string literals (with `{var}` interpolation),
//! numbers, `$tokens`, operators, punctuation, and — inside a `style` or
//! `theme` block — raw CSS values.

pub mod token;
pub mod v2;

pub use token::{StringKind, StringLit, Token, TokenType};
pub use v2::LexerV2;
