//! `wf migrate` over a project or one file: each file carried to the
//! current grammar (2 → 3, a change of spelling), then what 4 changed
//! (3 → 4, a change in what the compiler allows): the mechanical part done,
//! what needs a person named with its file and line, and what changed
//! without an edit stated.
//!
//! [`plan`] reads and migrates in memory and writes nothing; `wf migrate`
//! prints the plan and writes it, `--check` only prints it, and the mobile
//! editor shows it before [`apply`].

use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;
use crate::error::{Result, WebFluentError};

use super::{Migrated, Note, migrate_project, source_files};

/// One file's migration.
#[derive(Debug)]
pub struct FileChange {
    /// The file as it is.
    pub path: PathBuf,
    /// Where the migrated text goes: the file itself, or its `.wfx` name.
    pub written_to: PathBuf,
    /// The migrated text, in the layout asked for.
    pub text: String,
    /// Whether the file changes (or is rewritten as `.wfx`).
    pub changed: bool,
    /// What the migration could not rewrite mechanically.
    pub notes: Vec<Note>,
    /// Why the file could not be migrated.
    pub error: Option<String>,
}

impl FileChange {
    /// The notes as `wf migrate` prints them, a `path:line:col: message`
    /// line each.
    pub fn report(&self) -> String {
        super::report(
            &self.path,
            &Migrated {
                text: self.text.clone(),
                notes: self.notes.clone(),
                changed: self.changed,
            },
        )
    }
}

/// What 4 changed in a project already in 3's grammar.
#[derive(Debug, Default)]
pub struct Four {
    /// The `env` names the pages read. In 3 every name was inlined into the
    /// bundle; in 4 only a public one may be, so they go in `public_env`.
    pub public_env: Vec<String>,
    /// What 4 refuses that 3 allowed, each a message with its file and line.
    pub manual: Vec<String>,
}

/// A migration, before anything is written.
#[derive(Debug)]
pub struct Plan {
    /// The project, when `path` was one (not a lone file).
    pub project_dir: Option<PathBuf>,
    /// Every file looked at, in order.
    pub files: Vec<FileChange>,
    /// The 3 → 4 step: for a project whose every file migrated.
    pub four: Option<Four>,
}

impl Plan {
    /// The files that change.
    pub fn changed(&self) -> impl Iterator<Item = &FileChange> {
        self.files.iter().filter(|f| f.changed && f.error.is_none())
    }

    /// The files that could not be migrated.
    pub fn failed(&self) -> impl Iterator<Item = &FileChange> {
        self.files.iter().filter(|f| f.error.is_some())
    }
}

/// What 4 changed under a program that still compiles, and is worth
/// knowing: no edit makes these, so a migration states them.
pub const WITHOUT_AN_EDIT: &[&str] = &[
    "A store is built on first read, not at boot. A `derived` with a side effect now runs when something reads it; `store X(eager: true)` restores the old timing.",
    "A `persist` value follows the site's other tabs. `sync: false` keeps one to its own tab.",
    "`wf init` turns the Content-Security-Policy on; an existing project opts in with `\"build\": { \"csp\": true }`, and the build then holds its own output to it.",
    "A hand-written script calling `WF.store(def)` wants `WF.store(name, define, options)`, and `WF.host(…)` is now `WF.attach(…)`.",
];

/// Migrate `path` (a project, or one `.wf` file) in memory; `wfx` writes
/// the result in the indented layout, as `.wfx`.
pub fn plan(path: &Path, wfx: bool) -> Result<Plan> {
    let files = if path.is_file() {
        vec![(path.to_path_buf(), std::fs::read_to_string(path)?)]
    } else {
        let src = if path.join("src").is_dir() {
            path.join("src")
        } else {
            path.to_path_buf()
        };
        source_files(&src)?
    };
    if files.is_empty() {
        return Err(WebFluentError::IoError(format!(
            "no .wf files under {}",
            path.display()
        )));
    }
    let changes: Vec<FileChange> = migrate_project(&files)
        .into_iter()
        .map(|(file, outcome)| match outcome {
            Ok(migrated) => {
                // `wfx`: the migrated text in the indented layout, under the
                // `.wfx` name; the `.wf` file goes.
                let (text, written_to, error) = if wfx {
                    match crate::layout::to_offside(&migrated.text, &file.to_string_lossy()) {
                        Ok(text) => (text, offside_path(&file), None),
                        Err(e) => (migrated.text.clone(), file.clone(), Some(e.to_string())),
                    }
                } else {
                    (migrated.text.clone(), file.clone(), None)
                };
                FileChange {
                    path: file,
                    written_to,
                    text,
                    changed: migrated.changed || wfx,
                    notes: migrated.notes,
                    error,
                }
            }
            Err(e) => FileChange {
                written_to: file.clone(),
                path: file,
                text: String::new(),
                changed: false,
                notes: Vec::new(),
                error: Some(e.to_string()),
            },
        })
        .collect();
    let failed = changes.iter().any(|c| c.error.is_some());
    let project_dir = path.is_dir().then(|| path.to_path_buf());
    // The spelling is only half of it: 4 also changed what the compiler
    // allows, and a project already written in 3's grammar needs that half
    // on its own.
    let four = match &project_dir {
        Some(dir) if !failed => four(dir, &files),
        _ => None,
    };
    Ok(Plan {
        project_dir,
        files: changes,
        four,
    })
}

/// The WebFluent 3 → 4 step over a project's files (as they were read):
/// `None` when the folder is not a project.
///
/// Most of what 4 changed is a decision only the author can make: whether a
/// value in `env` may be read by anyone who opens the site, what an
/// `onclick:` attribute was for. So this finds the one mechanical part, and
/// names every place that needs a person with its file and line.
pub fn four(project_dir: &Path, files: &[(PathBuf, String)]) -> Option<Four> {
    let config = ProjectConfig::load(project_dir).ok()?;
    let mut public_env: Vec<String> = Vec::new();
    for (file, _) in files {
        let relative = file
            .strip_prefix(project_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();
        for diagnostic in crate::linter::lint_env(project_dir, &config, &[relative]) {
            if let Some(name) = between(&diagnostic.message, "`env.", "`")
                && !public_env.contains(&name)
            {
                public_env.push(name);
            }
        }
    }
    let mut manual: Vec<String> = Vec::new();
    for (file, text) in files {
        let name = file
            .strip_prefix(project_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();
        let Ok(program) = crate::syntax::parse_source(text, &name) else {
            continue;
        };
        for error in crate::sema::check(&program, &|_| name.clone()).errors {
            let message = error.to_string();
            if message.contains("would put script in an attribute")
                || message.contains("which a browser runs")
            {
                manual.push(message);
            }
        }
    }
    Some(Four { public_env, manual })
}

/// Write one file's migration: its text where it goes, and the old file
/// gone when it was renamed (`.wf` → `.wfx`).
pub fn write(change: &FileChange) -> Result<()> {
    std::fs::write(&change.written_to, &change.text)?;
    if change.written_to != change.path {
        std::fs::remove_file(&change.path)?;
    }
    Ok(())
}

/// Add `names` to the project's `public_env`, leaving everything else in
/// its config — including the order it was written in — alone. Returns the
/// config's path.
pub fn write_public_env(project_dir: &Path, names: &[String]) -> Result<PathBuf> {
    let mut config = ProjectConfig::load(project_dir)?;
    config.public_env.extend(names.iter().cloned());
    config.public_env.sort();
    config.public_env.dedup();
    let path = project_dir.join("webfluent.app.json");
    let text = std::fs::read_to_string(&path)?;
    let mut value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| WebFluentError::ConfigError(format!("{}: {e}", path.display())))?;
    if let Some(map) = value.as_object_mut() {
        map.insert(
            "public_env".to_string(),
            serde_json::Value::Array(
                config
                    .public_env
                    .iter()
                    .map(|n| serde_json::Value::String(n.clone()))
                    .collect(),
            ),
        );
    }
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&value).unwrap_or(text) + "\n",
    )?;
    Ok(path)
}

/// Write a whole plan: every file that changes, and the names 4 needs in
/// `public_env`. What it wrote, in order.
pub fn apply(plan: &Plan) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for change in plan.changed() {
        write(change)?;
        written.push(change.written_to.clone());
    }
    if let (Some(dir), Some(four)) = (&plan.project_dir, &plan.four)
        && !four.public_env.is_empty()
    {
        written.push(write_public_env(dir, &four.public_env)?);
    }
    Ok(written)
}

/// The path a migrated file is written to when the indented layout is
/// asked for: the same name with the `.wfx` extension.
pub fn offside_path(file: &Path) -> PathBuf {
    file.with_extension("wfx")
}

/// The text between two markers, when both are there.
fn between(text: &str, open: &str, close: &str) -> Option<String> {
    let from = text.find(open)? + open.len();
    let to = text[from..].find(close)? + from;
    Some(text[from..to].to_string())
}
