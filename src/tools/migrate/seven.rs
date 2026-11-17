//! WebFluent 6 → 7: every finding 7.0 adds that has a known fix, fixed —
//! the same fixes `wf fix` and the editor offer — until none is left to
//! take; then what still needs a person, by place.
//!
//! 7.0 has no loose mode: an old form is an error with its fix, and this
//! step is those fixes applied at once. What it cannot be sure of it leaves
//! for a person and lists, so nothing it writes is a guess.

use std::collections::BTreeMap;
use std::path::Path;

use crate::base::vfs::FsVfs;
use crate::build::{BuildOptions, BuildReport};
use crate::diagnostics::error::{Result, WebFluentError};
use crate::diagnostics::fixes::{apply, merge};
use crate::diagnostics::{Diagnostic, Edit, Fix, Severity};

/// How many times a fix run looks again: a fix left out because it
/// overlapped another is taken on the next look, and one fix may make way
/// for another (an `N08` rewritten, then the `T26` its action now has).
pub const ROUNDS: usize = 8;

/// The codes 7.0 adds or makes errors, whose fixes the step applies: the
/// request forms 7.0 removed (`N08`), open types at a service (`N11`), an
/// error enum's `http` case (`N12`), text joined to a number with `+`
/// (`T18`), a read's missing `.failed` arm (`T21`), and the typing rules
/// (`T25`–`T35`).
pub const SEVEN: &[&str] = &[
    "N08", "N11", "N12", "T18", "T21", "T23", "T24", "T25", "T26", "T27", "T28", "T29", "T30",
    "T31", "T32", "T33", "T34", "T35",
];

/// Whether a finding is one 7.0 brought: a code of [`SEVEN`], or another
/// of the `N` family.
pub fn is_seven(code: &str) -> bool {
    SEVEN.contains(&code) || code.starts_with('N')
}

/// What a fix run did.
#[derive(Debug, Default)]
pub struct Fixed {
    /// Findings fixed, by code.
    pub by_code: BTreeMap<&'static str, usize>,
    /// Findings fixed, by file.
    pub by_file: BTreeMap<String, usize>,
    /// What is left that a person decides: the findings of the codes asked
    /// for, after the last look.
    pub left: Vec<Diagnostic>,
}

impl Fixed {
    pub fn total(&self) -> usize {
        self.by_code.values().sum()
    }
}

/// Every finding `wants` picks that has a fix, fixed by its first, until
/// none is left to take (one look with `dry_run`, which writes nothing).
///
/// # Errors
///
/// When the project cannot be read, or a file cannot be written.
pub fn fix(dir: &Path, wants: &dyn Fn(&Diagnostic) -> bool, dry_run: bool) -> Result<Fixed> {
    let mut out = Fixed::default();
    // In a dry run nothing is written, so a second look sees the same.
    let rounds = if dry_run { 1 } else { ROUNDS };
    let mut last = None;
    for _ in 0..rounds {
        let report = check(dir)?;
        let wanted: Vec<&Diagnostic> = report
            .diagnostics
            .iter()
            .filter(|d| !d.fixes.is_empty() && wants(d))
            .collect();
        if wanted.is_empty() {
            last = Some(report);
            break;
        }
        // Every fix that does not overlap one taken before it; the rest
        // wait for the next look.
        let (edits, taken) = merge(wanted.iter().copied());
        for d in &taken {
            *out.by_file.entry(d.file.clone()).or_default() += 1;
            *out.by_code.entry(d.code).or_default() += 1;
        }
        if dry_run {
            last = Some(report);
            break;
        }
        let mut per_file: BTreeMap<String, Vec<Edit>> = BTreeMap::new();
        for e in edits {
            per_file.entry(e.file.clone()).or_default().push(e);
        }
        for (file, edits) in per_file {
            let path = dir.join(&file);
            // A fix is written into the project, never through a link out
            // of it.
            if crate::base::vfs::escapes(dir, &path) {
                continue;
            }
            let text = std::fs::read_to_string(&path)
                .map_err(|e| WebFluentError::io_with(format!("{}: {e}", path.display()), e))?;
            let fixed = apply(
                &text,
                &Fix {
                    title: String::new(),
                    edits,
                },
            );
            if fixed != text {
                std::fs::write(&path, fixed)
                    .map_err(|e| WebFluentError::io_with(format!("{}: {e}", path.display()), e))?;
            }
        }
    }
    let report = match last {
        Some(r) if dry_run => r,
        _ => check(dir)?,
    };
    out.left = report
        .diagnostics
        .into_iter()
        .filter(|d| wants(d))
        .collect();
    Ok(out)
}

/// The 6 → 7 step: every 7.0 finding with a fix, fixed; the 7.0 errors
/// left, for a person.
///
/// # Errors
///
/// What [`fix`] returns.
pub fn migrate(dir: &Path, dry_run: bool) -> Result<Fixed> {
    // `Any` and `Map` become `Json` only where that leaves fewer errors:
    // a value read as a record is a `T28` for every field once it is
    // `Json`, so there the type is a person's to write.
    let mut fixed = fix(
        dir,
        &|d| (is_seven(d.code) || takes_a_result(d)) && !open_type(d.code),
        dry_run,
    )?;
    if !dry_run {
        for (code, n) in json_where_it_helps(dir)? {
            *fixed.by_code.entry(code).or_default() += n;
        }
    }
    // What is left for a person is every error: a warning is `wf check`'s.
    let report = check(dir)?;
    fixed.left = report
        .diagnostics
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    Ok(fixed)
}

fn open_type(code: &str) -> bool {
    matches!(code, "T23" | "T24")
}

/// `T23` and `T24` fixed to `Json`, a file at a time, keeping a file's only
/// where the project then has fewer errors than before. What it fixed, by
/// code.
fn json_where_it_helps(dir: &Path) -> Result<Vec<(&'static str, usize)>> {
    let errors = |r: &BuildReport| {
        r.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    };
    let report = check(dir)?;
    let mut before = errors(&report);
    let mut by_file: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
    for d in report.diagnostics {
        if open_type(d.code) && !d.fixes.is_empty() {
            by_file.entry(d.file.clone()).or_default().push(d);
        }
    }
    let mut kept: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (file, found) in by_file {
        let path = dir.join(&file);
        if crate::base::vfs::escapes(dir, &path) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (edits, taken) = merge(found.iter());
        let changed = apply(
            &text,
            &Fix {
                title: String::new(),
                edits,
            },
        );
        if changed == text {
            continue;
        }
        std::fs::write(&path, &changed)
            .map_err(|e| WebFluentError::io_with(format!("{}: {e}", path.display()), e))?;
        let after = errors(&check(dir)?);
        if after < before {
            before = after;
            for d in taken {
                *kept.entry(d.code).or_default() += 1;
            }
        } else {
            std::fs::write(&path, &text)
                .map_err(|e| WebFluentError::io_with(format!("{}: {e}", path.display()), e))?;
        }
    }
    Ok(kept.into_iter().collect())
}

/// A request's result where its value is wanted — `let rows: [Row] =
/// await http.get(url)`, which 6.x read as the value — fixed with `try`.
fn takes_a_result(d: &Diagnostic) -> bool {
    d.code == "T01" && d.fixes.iter().any(|f| f.title == "Write `try await`")
}

/// What 7.0 changes with no edit to make, which the step states.
pub const WITHOUT_AN_EDIT: &[&str] = &[
    "A `POST` or `PATCH` is retried, or replayed offline, only when its endpoint is marked `idempotent`.",
    "Nothing is cached unless the server's `Cache-Control` or the contract says so.",
    "Every answer is held to its declared type, deployed too: a server that does not keep its contract now fails as `.decode(at, message)` where it used to pass.",
];

/// Every finding, as `wf check` would report it, without printing.
fn check(dir: &Path) -> Result<BuildReport> {
    // A check writes nothing, so where the build would write is no reason
    // to stop: a project that builds to `../docs` is migrated and fixed too.
    let options = BuildOptions {
        allow_output_outside: true,
        ..BuildOptions::new(dir, &FsVfs)
    };
    match crate::build::check(&options) {
        Ok(report) => Ok(report),
        Err(failure) if !failure.report.diagnostics.is_empty() => Ok(*failure.report),
        Err(failure) => Err(failure.error),
    }
}
