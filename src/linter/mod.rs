//! Compile-time linting for accessibility, PDF validation, and slides validation.
//!
//! - [`lint_accessibility`] — WCAG-based checks (missing alt text, form labels, heading hierarchy, etc.)
//! - [`lint_contrast`] — WCAG contrast ratios over a project's own design
//!   tokens, which is only checkable because a `Theme` is parsed source
//! - [`lint_vocabulary`] — dead bare-word arguments (a misspelled modifier parses
//!   as an expression and silently does nothing), with did-you-mean hints.
//!   Warnings, not gate errors — see the module for why.
//! - [`validate_for_pdf`] — ensures interactive components aren't used in PDF output
//! - [`validate_for_slides`] — enforces slide deck structure and rejects interactive components
//! - [`validate_semantics`] — the studio compile-gate: undefined component refs,
//!   unknown route targets, duplicate declarations

pub mod accessibility;
pub mod contrast;
pub mod pdf_validation;
pub mod semantic;
pub mod slides_validation;
pub mod vocabulary;

pub use accessibility::lint_accessibility;
pub use contrast::lint_contrast;
pub use pdf_validation::validate_for_pdf;
pub use slides_validation::validate_for_slides;
// These are public API — a consumer reaches them as `webfluent::…`, and the LSP,
// the studio and the docs all do. Nothing inside this crate calls them, which is
// why `cargo clippy --fix` removed them; `#[allow]` records that the emptiness is
// expected rather than leaving the next person to delete them again.
#[allow(unused_imports)]
pub use semantic::validate_semantics;
#[allow(unused_imports)]
pub use vocabulary::lint_vocabulary;
