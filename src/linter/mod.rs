//! Compile-time linting for accessibility, PDF validation, and slides validation.
//!
//! - [`lint_accessibility`] — 12 WCAG-based checks (missing alt text, form labels, heading hierarchy, etc.)
//! - [`validate_for_pdf`] — ensures interactive components aren't used in PDF output
//! - [`validate_for_slides`] — enforces slide deck structure and rejects interactive components
//! - [`validate_semantics`] — the studio compile-gate: undefined component refs,
//!   unknown route targets, duplicate declarations

pub mod accessibility;
pub mod pdf_validation;
pub mod semantic;
pub mod slides_validation;

pub use accessibility::lint_accessibility;
pub use pdf_validation::validate_for_pdf;
pub use semantic::validate_semantics;
pub use slides_validation::validate_for_slides;
