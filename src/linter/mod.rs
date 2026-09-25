//! Compile-time linting for accessibility, PDF validation, and slides validation.
//!
//! - [`lint_accessibility`] — WCAG-based checks (missing alt text, form labels, heading hierarchy, etc.)
//! - [`lint_contrast`] — WCAG contrast ratios over a project's own design
//!   tokens, which is only checkable because a `Theme` is parsed source
//! - [`lint_vocabulary`] — dead bare-word arguments (a misspelled modifier parses
//!   as an expression and silently does nothing), with did-you-mean hints.
//!   Warnings, not gate errors — see the module for why.
//! - [`lint_unused`] — state, derived values, actions, components and store
//!   members the program declares and never reads (`U01`–`U05`)
//! - [`validate_for_pdf`] — ensures interactive components aren't used in PDF output
//! - [`validate_for_slides`] — enforces slide deck structure and rejects interactive components
//! - [`validate_semantics`] — the studio compile-gate: undefined component refs,
//!   unknown route targets, duplicate declarations

pub mod secrets;
pub use secrets::lint_env;
pub mod accessibility;
pub mod contrast;
pub mod pdf_validation;
pub mod semantic;
pub mod slides_validation;
pub mod unused;
pub mod vocabulary;

pub use accessibility::lint_accessibility_in;
pub use contrast::lint_contrast_in;
pub use pdf_validation::validate_for_pdf;
pub use semantic::validate_semantics_in;
pub use slides_validation::validate_for_slides;
#[allow(unused_imports)]
pub use unused::lint_unused;
pub use unused::lint_unused_in;
pub use vocabulary::lint_vocabulary_with;
// The single-file forms are public API — the studio and the docs reach them
// as `webfluent::…` — while the build and the LSP use the per-file forms.
#[allow(unused_imports)]
pub use accessibility::lint_accessibility;
#[allow(unused_imports)]
pub use contrast::lint_contrast;
#[allow(unused_imports)]
pub use semantic::validate_semantics;
#[allow(unused_imports)]
pub use vocabulary::lint_vocabulary;
// The LSP lints a project it has no stylesheets for.
#[allow(unused_imports)]
pub use vocabulary::lint_vocabulary_in;
