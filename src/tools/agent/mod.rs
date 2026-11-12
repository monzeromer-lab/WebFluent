//! What an agent works through: a change set held apart from the working
//! tree, built before it is read, and the context it is given.
//!
//! What an agent works through: a change set (`changeset::ChangeSet`) it
//! writes to instead of the working tree, which the compiler builds before
//! anyone reads it, with its `diff` by file and by hunk; and the context
//! (`context::Context`) it is given — the project, the registry and the
//! guide — with what was withheld named.

pub mod changeset;
pub mod context;
pub mod diff;
pub mod edit;

pub use crate::base::vfs::escapes;
pub(crate) use crate::base::vfs::leaves;
