//! Lexical analysis for WebFluent source code.
//!
//! Converts `.wf` source text into a stream of [`Token`]s, handling keywords,
//! identifiers, string literals (with `{var}` interpolation), numbers, and operators.

pub mod token;
pub mod lexer;

pub use token::{Token, TokenType};
pub use lexer::Lexer;
