//! What an agent works through: a change set (`changeset::ChangeSet`) it
//! writes to instead of the working tree, which the compiler builds before
//! anyone reads it, with its `diff` by file and by hunk; and the context
//! (`context::Context`) it is given — the project, the registry and the
//! guide — with what was withheld named.

pub mod changeset;
pub mod context;
pub mod diff;

use std::path::Path;

/// Whether `path`, inside the project at `dir` by its words, leads out of
/// it on the disk: through a link, anywhere on the way, to somewhere else
/// or to nothing. What an agent reads, writes or is shown must not.
pub fn escapes(dir: &Path, path: &Path) -> bool {
    let Ok(root) = std::fs::canonicalize(dir) else {
        return false;
    };
    // The deepest part of the path that is there, resolved.
    let mut at = path.to_path_buf();
    loop {
        if std::fs::symlink_metadata(&at).is_ok() {
            return match std::fs::canonicalize(&at) {
                Ok(real) => !real.starts_with(&root),
                // A link to nothing: writing through it would make its
                // target, wherever that is.
                Err(_) => true,
            };
        }
        if !at.pop() {
            return false;
        }
    }
}

/// Whether `path` is a link that leads out of the project at `dir`.
pub(crate) fn leaves(dir: &Path, path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) && escapes(dir, path)
}
