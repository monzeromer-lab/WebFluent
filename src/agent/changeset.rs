//! A change set: what an agent writes, held apart from the working tree.
//!
//! It is one more layer over whatever the editor already lays over the
//! disk — its unsaved buffers — so the compiler builds the project as it
//! would be with the set accepted, and nothing is written until someone
//! accepts it: whole, a file at a time, or a hunk at a time.
//!
//! What the agent may touch is the set's to refuse, not the agent's to
//! remember: nothing outside the project, nothing in `.git/` or the build's
//! output, no `.env` file; and `webfluent.app.json` is read with each
//! private `env` value withheld, and written back with them kept.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;

use super::diff::{self, Hunk};
use crate::diagnostics::{Diagnostic, Severity};
use crate::error::{Result, WebFluentError};
use crate::vfs::{OverlayVfs, Vfs};

/// What a withheld value reads as.
pub const WITHHELD: &str = "‹withheld›";

const CONFIG: &str = "webfluent.app.json";

fn refused(why: impl Into<String>) -> WebFluentError {
    WebFluentError::EditError(why.into())
}

/// `rel` as a path inside the project, or why it is not one the set may
/// touch.
fn inside(rel: &str) -> Result<String> {
    let path = Path::new(rel);
    if path.is_absolute() {
        return Err(refused(format!(
            "`{rel}` is outside the project: a change set takes paths inside it"
        )));
    }
    let mut parts = Vec::new();
    for c in path.components() {
        match c {
            Component::Normal(p) => parts.push(p.to_string_lossy().to_string()),
            Component::CurDir => {}
            _ => return Err(refused(format!("`{rel}` leaves the project"))),
        }
    }
    if parts.is_empty() {
        return Err(refused("an empty path"));
    }
    let first = parts[0].as_str();
    if matches!(
        first,
        ".git" | "build" | ".wf-cache" | "node_modules" | "dist" | "target"
    ) {
        return Err(refused(format!(
            "`{rel}` is in `{first}/`, which a change set does not touch"
        )));
    }
    if parts
        .last()
        .is_some_and(|n| n == ".env" || n.starts_with(".env."))
    {
        return Err(refused(format!(
            "`{rel}` holds private values; it is never read or written"
        )));
    }
    Ok(parts.join("/"))
}

/// What happened to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileKind {
    Added,
    Modified,
    Deleted,
}

/// One file of the set, as its tab shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileChange {
    pub path: String,
    pub kind: FileKind,
    pub added: usize,
    pub removed: usize,
    pub hunks: Vec<Hunk>,
    /// Outside `src/`: a write the plan needs a yes for.
    pub outside_src: bool,
}

/// What building the set says, against building without it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Checked {
    /// No errors with the set laid over the project.
    pub builds: bool,
    /// The problems without the set, and with it.
    pub before: usize,
    pub after: usize,
    /// The diagnostics the set brings: ones the project without it does
    /// not have. Shown as the set's, not the project's.
    pub owned: Vec<Diagnostic>,
    /// How many of the project's the set takes away.
    pub fixed: usize,
}

/// The change set.
pub struct ChangeSet {
    base: Arc<dyn Vfs>,
    dir: PathBuf,
    /// Each changed file, by path inside the project: its new text, or none
    /// for a deleted one.
    changes: BTreeMap<String, Option<String>>,
    /// The files read, whether or not they were changed.
    read: BTreeSet<String>,
}

impl ChangeSet {
    /// An empty set over `base` — the disk, or the editor's buffers over it
    /// — for the project at `dir`.
    pub fn new(base: Arc<dyn Vfs>, dir: impl Into<PathBuf>) -> Self {
        ChangeSet {
            base,
            dir: dir.into(),
            changes: BTreeMap::new(),
            read: BTreeSet::new(),
        }
    }

    fn on_disk(&self, rel: &str) -> Option<String> {
        self.base.read_to_string(&self.dir.join(rel)).ok()
    }

    /// The private `env` names of the project's config, with their values.
    fn private_env(&self) -> Vec<(String, serde_json::Value)> {
        let Some(text) = self.on_disk(CONFIG) else {
            return Vec::new();
        };
        let Ok(config) = serde_json::from_str::<crate::config::ProjectConfig>(&text) else {
            return Vec::new();
        };
        let Ok(Some(serde_json::Value::Object(env))) =
            crate::config::json_edit::get(&text, &["env"])
        else {
            return Vec::new();
        };
        env.into_iter()
            .filter(|(name, _)| !config.env_is_public(name))
            .collect()
    }

    /// The file at `rel` as the agent may see it: through the set, with
    /// each private `env` value of the config withheld. It is recorded as
    /// read.
    pub fn read(&mut self, rel: &str) -> Result<String> {
        let rel = inside(rel)?;
        let text = match self.changes.get(&rel) {
            Some(Some(t)) => t.clone(),
            Some(None) => return Err(refused(format!("`{rel}` is deleted in this change set"))),
            None => self
                .on_disk(&rel)
                .ok_or_else(|| refused(format!("there is no `{rel}`")))?,
        };
        self.read.insert(rel.clone());
        if rel == CONFIG {
            return Ok(withhold(&text, &self.private_env()));
        }
        Ok(text)
    }

    /// Write `text` to `rel` in the set. A private `env` value the agent
    /// was shown as withheld is kept as it is.
    pub fn write(&mut self, rel: &str, text: &str) -> Result<()> {
        let rel = inside(rel)?;
        let mut text = text.to_string();
        if rel == CONFIG {
            text = restore(&text, &self.private_env())?;
        }
        if self.on_disk(&rel).as_deref() == Some(text.as_str()) {
            self.changes.remove(&rel);
        } else {
            self.changes.insert(rel, Some(text));
        }
        Ok(())
    }

    /// Delete `rel` in the set.
    pub fn delete(&mut self, rel: &str) -> Result<()> {
        let rel = inside(rel)?;
        if rel == CONFIG {
            return Err(refused(
                "the project's config is not deleted by a change set",
            ));
        }
        if self.on_disk(&rel).is_none() {
            self.changes.remove(&rel);
        } else {
            self.changes.insert(rel, None);
        }
        Ok(())
    }

    /// Take `rel`'s change out of the set: what rejecting one file does.
    pub fn revert(&mut self, rel: &str) -> bool {
        inside(rel)
            .map(|r| self.changes.remove(&r).is_some())
            .unwrap_or(false)
    }

    /// Whether it changes anything.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// The project as it would be with the set accepted: the base, with the
    /// set over it.
    pub fn vfs(&self) -> OverlayVfs {
        let mut v = OverlayVfs::new(self.base.clone());
        for (rel, change) in &self.changes {
            let path = self.dir.join(rel);
            match change {
                Some(text) => v.set(path, text.clone()),
                None => v.delete(path),
            }
        }
        v
    }

    /// Every file the set changes, each with its hunks.
    pub fn files(&self) -> Vec<FileChange> {
        self.changes
            .iter()
            .map(|(rel, change)| {
                let old = self.on_disk(rel);
                let new = change.clone().unwrap_or_default();
                let hunks = diff::hunks(old.as_deref().unwrap_or(""), &new);
                FileChange {
                    kind: match (&old, change) {
                        (None, _) => FileKind::Added,
                        (Some(_), None) => FileKind::Deleted,
                        _ => FileKind::Modified,
                    },
                    added: hunks.iter().map(Hunk::added).sum(),
                    removed: hunks.iter().map(Hunk::removed).sum(),
                    outside_src: !rel.starts_with("src/"),
                    path: rel.clone(),
                    hunks,
                }
            })
            .collect()
    }

    /// The files it read and did not change.
    pub fn left_alone(&self) -> Vec<String> {
        self.read
            .iter()
            .filter(|r| !self.changes.contains_key(*r))
            .cloned()
            .collect()
    }

    /// Build the project with the set and without it, and say what the set
    /// did.
    pub fn check(&self) -> Checked {
        let run = |vfs: &dyn Vfs| -> Vec<Diagnostic> {
            let options = crate::build::BuildOptions::new(&self.dir, vfs);
            match crate::build::check(&options) {
                Ok(r) => r.diagnostics,
                Err(f) => f.report.diagnostics,
            }
        };
        let before = run(self.base.as_ref());
        let with = self.vfs();
        let after = run(&with);
        // A diagnostic is the same one when it says the same thing about
        // the same file; its line moves with the edits around it.
        let same = |d: &Diagnostic| (d.code, d.file.clone(), d.message.clone());
        let mut left: Vec<_> = before.iter().map(same).collect();
        let mut owned = Vec::new();
        for d in &after {
            match left.iter().position(|x| *x == same(d)) {
                Some(i) => {
                    left.remove(i);
                }
                None => owned.push(d.clone()),
            }
        }
        Checked {
            builds: !after.iter().any(|d| d.severity == Severity::Error),
            before: before.len(),
            after: after.len(),
            fixed: left.len(),
            owned,
        }
    }

    /// The set's heading: `4 files · +86 −12 · builds · 6 problems → 2`.
    pub fn heading(&self, checked: &Checked) -> String {
        let files = self.files();
        let (added, removed): (usize, usize) = files
            .iter()
            .fold((0, 0), |(a, r), f| (a + f.added, r + f.removed));
        let n = files.len();
        format!(
            "{n} file{} · +{added} −{removed} · {} · {} problem{} → {}",
            if n == 1 { "" } else { "s" },
            if checked.builds {
                "builds"
            } else {
                "does not build"
            },
            checked.before,
            if checked.before == 1 { "" } else { "s" },
            checked.after
        )
    }

    /// What accepting writes for `rel`: the whole file, or only the hunks
    /// named. None for a file to delete, or one the set does not change.
    pub fn accepted(&self, rel: &str, hunks: Option<&[usize]>) -> Option<Option<String>> {
        let rel = inside(rel).ok()?;
        let change = self.changes.get(&rel)?;
        let Some(new) = change else {
            return Some(None);
        };
        let Some(ids) = hunks else {
            return Some(Some(new.clone()));
        };
        let old = self.on_disk(&rel).unwrap_or_default();
        Some(Some(diff::apply(&old, &diff::hunks(&old, new), ids)))
    }

    /// Every file accepting the whole set writes, as a path on the disk
    /// and its text, none for a file to delete.
    pub fn accept_all(&self) -> Vec<(PathBuf, Option<String>)> {
        self.changes
            .iter()
            .map(|(rel, c)| (self.dir.join(rel), c.clone()))
            .collect()
    }
}

/// `text` with each private `env` value withheld.
fn withhold(text: &str, private: &[(String, serde_json::Value)]) -> String {
    let mut out = text.to_string();
    for (name, _) in private {
        if let Ok(t) = crate::config::json_edit::set(
            &out,
            &["env", name],
            &serde_json::Value::String(WITHHELD.into()),
        ) {
            out = t;
        }
    }
    out
}

/// `text`, an edited config, with each private value it shows as withheld
/// put back; a private name it left out is kept too, since the agent
/// cannot have meant to remove what it was never shown.
fn restore(text: &str, private: &[(String, serde_json::Value)]) -> Result<String> {
    let mut out = text.to_string();
    for (name, value) in private {
        let now = crate::config::json_edit::get(&out, &["env", name])
            .map_err(|e| refused(e.to_string()))?;
        let keep = match now {
            Some(serde_json::Value::String(s)) => s == WITHHELD,
            None => true,
            // A value of its own: the agent set it, and it is the agent's.
            Some(_) => false,
        };
        if keep {
            out = crate::config::json_edit::set(&out, &["env", name], value)
                .map_err(|e| refused(e.to_string()))?;
        }
    }
    Ok(out)
}
