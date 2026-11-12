//! Everything that checks a program: names and types, the linters, the
//! validate rules, scopes and edits.

pub mod allow;
pub mod edit;
pub mod jsdoc;
pub mod linter;
/// What a name at a place can refer to — the editor's completion scope —
/// and, typed, what a path picker lists there.
pub mod scope;
pub mod sema;
/// Validation as data: every rule a `validate` block may hold, the named
/// patterns, when a form shows its messages, the rules one state is held
/// to, and the block written back one rule at a time.
pub mod validation;
