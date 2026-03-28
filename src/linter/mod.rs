//! Compile-time linting for accessibility and PDF validation.
//!
//! - [`lint_accessibility`] — 12 WCAG-based checks (missing alt text, form labels, heading hierarchy, etc.)
//! - [`validate_for_pdf`] — ensures interactive components aren't used in PDF output

pub mod accessibility;
pub mod pdf_validation;

pub use accessibility::lint_accessibility;
pub use pdf_validation::validate_for_pdf;
