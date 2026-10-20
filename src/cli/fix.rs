use std::collections::BTreeMap;
use std::path::Path;

use crate::build::{BuildOptions, BuildReport};
use crate::diagnostics::fixes::{apply, merge};
use crate::diagnostics::{Diagnostic, Edit, Fix};
use crate::error::{Result, WebFluentError};
use crate::vfs::FsVfs;

/// How many times `wf fix` looks again: a fix left out because it overlapped
/// another is taken on the next look, and a fix may make way for another.
const ROUNDS: usize = 8;

/// `wf fix [-d dir] [--code C] [--dry-run]`: every finding that has a fix
/// — or every one of a code — fixed, each by its first fix, as `wf check`
/// and the editor offer it; then what is left is said. `--dry-run` writes
/// nothing and says what would change.
pub fn run_fix(dir: &Path, code: Option<&str>, dry_run: bool) -> Result<()> {
    let mut fixed: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_code: BTreeMap<&'static str, usize> = BTreeMap::new();
    // In a dry run nothing is written, so a second look sees the same.
    let rounds = if dry_run { 1 } else { ROUNDS };
    for _ in 0..rounds {
        let report = check(dir)?;
        let wanted: Vec<&Diagnostic> = report
            .diagnostics
            .iter()
            .filter(|d| !d.fixes.is_empty() && code.is_none_or(|c| d.code == c))
            .collect();
        if wanted.is_empty() {
            break;
        }
        // Every fix that does not overlap one taken before it; the rest
        // wait for the next look.
        let (edits, taken) = merge(wanted.iter().copied());
        for d in &taken {
            *fixed.entry(d.file.clone()).or_default() += 1;
            *by_code.entry(d.code).or_default() += 1;
        }
        let mut per_file: BTreeMap<String, Vec<Edit>> = BTreeMap::new();
        for e in edits {
            per_file.entry(e.file.clone()).or_default().push(e);
        }
        for (file, edits) in per_file {
            let path = dir.join(&file);
            let text = std::fs::read_to_string(&path)
                .map_err(|e| WebFluentError::IoError(format!("{}: {e}", path.display())))?;
            let out = apply(
                &text,
                &Fix {
                    title: String::new(),
                    edits,
                },
            );
            if dry_run {
                println!("would change {file}");
            } else if out != text {
                std::fs::write(&path, out)
                    .map_err(|e| WebFluentError::IoError(format!("{}: {e}", path.display())))?;
            }
        }
        if dry_run {
            break;
        }
    }
    let total: usize = fixed.values().sum();
    let verb = if dry_run { "Would fix" } else { "Fixed" };
    if total == 0 {
        println!("Nothing to fix.");
    } else {
        let codes: Vec<String> = by_code.iter().map(|(c, n)| format!("{n} {c}")).collect();
        println!(
            "{verb} {total} finding{} in {} file{}: {}",
            if total == 1 { "" } else { "s" },
            fixed.len(),
            if fixed.len() == 1 { "" } else { "s" },
            codes.join(", ")
        );
    }
    if !dry_run {
        let left = check(dir)?.diagnostics;
        let left: Vec<&Diagnostic> = left
            .iter()
            .filter(|d| code.is_none_or(|c| d.code == c))
            .collect();
        if !left.is_empty() {
            let with_fix = left.iter().filter(|d| !d.fixes.is_empty()).count();
            println!(
                "{} left, which a person decides{}; `wf check` lists them",
                left.len(),
                if with_fix > 0 {
                    format!(" ({with_fix} with a fix that did not fit)")
                } else {
                    String::new()
                }
            );
        }
    }
    Ok(())
}

/// Every finding, as `wf check` would report it, without printing.
fn check(dir: &Path) -> Result<BuildReport> {
    let options = BuildOptions::new(dir, &FsVfs);
    match crate::build::check(&options) {
        Ok(report) => Ok(report),
        Err(failure) if !failure.report.diagnostics.is_empty() => Ok(*failure.report),
        Err(failure) => Err(failure.error),
    }
}
