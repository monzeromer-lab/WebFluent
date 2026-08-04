//! Lexical analysis for WebFluent source code.
//!
//! Converts `.wf` source text into a stream of [`Token`]s, handling keywords,
//! identifiers, string literals (with `{var}` interpolation), numbers, and operators.

// `lexer::lexer` mirrors the crate layout the rest of the tree uses;
// flattening it would move every public path for no gain.
#[allow(clippy::module_inception)]
pub mod lexer;
pub mod token;

pub use lexer::Lexer;
pub use token::{Token, TokenType};
