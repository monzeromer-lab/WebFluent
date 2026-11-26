//! `wf migrate`: a project carried forward to the current WebFluent.
//!
//! The steps, in the order they run ([`project::plan`]):
//!
//! - **4 → 4.2** ([`externals`]): `external` is gone; each declaration
//!   becomes a `meta.scripts` entry, a plain script under `src/`, or an
//!   `Element(…)` call.
//! - **Each file**: one already in the current grammar is left as it is,
//!   or written by indentation (`--wfx`). One in the grammar of WebFluent 2
//!   is not migrated: WebFluent 8 no longer reads that grammar, and the
//!   file is reported with the release that does ([`WF2_MIGRATION`]).
//! - **3 → 4** ([`project::four`]): what 4 changed in what the compiler
//!   allows.
//! - **6 → 7** ([`seven`]): the fixes `wf check` knows, applied until none
//!   is left.

pub mod externals;
pub mod project;
pub mod seven;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::diagnostics::error::{Result, WebFluentError};
use crate::syntax::{Dialect, detect_dialect};

pub use crate::syntax::WF2_MIGRATION;

/// Something the migration could not rewrite mechanically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

/// The outcome for one file.
#[derive(Debug)]
pub struct Migrated {
    /// The file's text, as the migration leaves it.
    pub text: String,
    pub notes: Vec<Note>,
    /// Whether `text` differs from the input.
    pub changed: bool,
}

/// Every file of a project, each as [`migrate_file`] leaves it.
pub fn migrate_project(files: &[(PathBuf, String)]) -> Vec<(PathBuf, Result<Migrated>)> {
    files
        .iter()
        .map(|(path, source)| (path.clone(), migrate_file(source, &path.to_string_lossy())))
        .collect()
}

/// One file: one in the current grammar is returned as it is.
///
/// # Errors
///
/// [`WebFluentError::ParseError`] for a file in the grammar of WebFluent 2,
/// which this release no longer migrates: its `E004` says to run `wf
/// migrate` from WebFluent 7.x first ([`WF2_MIGRATION`]).
pub fn migrate_file(source: &str, file: &str) -> Result<Migrated> {
    if detect_dialect(source) == Dialect::V1 {
        return Err(crate::syntax::parse_source(source, file)
            .err()
            .unwrap_or_else(|| WebFluentError::edit(format!("{file}: {WF2_MIGRATION}"))));
    }
    Ok(Migrated {
        text: source.to_string(),
        notes: Vec::new(),
        changed: false,
    })
}

/// The `.wf` files under `dir`, `App.wf` first, as `wf build` reads them.
///
/// What is listed is what a formatter or a migration may write back, so a
/// link that leads out of `dir` is left out — writing through it would
/// change a file somewhere else — and no directory is entered twice, so a
/// link that loops ends the walk rather than running it forever.
///
/// # Errors
///
/// The [`std::io::Error`] of a directory that cannot be listed or a file
/// that cannot be read.
pub fn source_files(dir: &Path) -> std::io::Result<Vec<(PathBuf, String)>> {
    fn walk(
        root: &Path,
        dir: &Path,
        seen: &mut std::collections::HashSet<PathBuf>,
        out: &mut Vec<PathBuf>,
    ) -> std::io::Result<()> {
        let real = std::fs::canonicalize(dir)?;
        if !seen.insert(real) {
            return Ok(());
        }
        let mut entries: Vec<(PathBuf, bool)> = std::fs::read_dir(dir)?
            .flatten()
            .map(|e| (e.path(), e.file_type().is_ok_and(|t| t.is_symlink())))
            .collect();
        entries.sort();
        for (path, linked) in entries {
            if linked && std::fs::canonicalize(&path).map_or(true, |real| !real.starts_with(root)) {
                eprintln!(
                    "  {}: left out, a link that leads outside {}",
                    path.display(),
                    root.display()
                );
                continue;
            }
            if path.is_dir() {
                walk(root, &path, seen, out)?;
            } else if crate::syntax::is_source_file(&path) {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut paths = Vec::new();
    let root = std::fs::canonicalize(dir)?;
    walk(
        &root,
        dir,
        &mut std::collections::HashSet::new(),
        &mut paths,
    )?;
    let mut files = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)?;
        files.push((path, text));
    }
    Ok(files)
}

/// A one-line report of a file's notes, for the console.
pub fn report(path: &Path, migrated: &Migrated) -> String {
    let mut out = String::new();
    for note in &migrated.notes {
        out.push_str(&format!(
            "  {}:{}:{}: {}\n",
            path.display(),
            note.line,
            note.column,
            note.message
        ));
    }
    out
}

/// Migrations keyed by path, in order.
pub type Outcome = BTreeMap<PathBuf, Result<Migrated>>;

#[cfg(test)]
mod tests {
    use super::*;

    /// A WebFluent 2 file is not migrated here: it is refused, naming the
    /// release that migrates it.
    #[test]
    fn a_webfluent_2_file_is_sent_to_the_7_x_migration() {
        let err = migrate_file("Page Home (path: \"/\") {\n    Text(\"Hi\")\n}\n", "a.wf")
            .err()
            .map(|e| format!("{e} {:?}", e.diagnostics()))
            .unwrap_or_default();
        assert!(err.contains("E004"), "{err}");
        assert!(err.contains("--version ^7"), "{err}");
    }

    #[test]
    fn a_current_file_is_left_as_it_is() {
        let src = "page Home(path: \"/\") {\n    Text(\"Hi\")\n}\n";
        let out = migrate_file(src, "a.wf").unwrap();
        assert!(!out.changed);
        assert_eq!(out.text, src);
    }
}
