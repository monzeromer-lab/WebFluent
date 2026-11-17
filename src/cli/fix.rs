use std::path::Path;

use crate::diagnostics::error::Result;
use crate::migrate::seven;

/// `wf fix [-d dir] [--code C] [--dry-run]`: every finding that has a fix
/// — or every one of a code — fixed, each by its first fix, as `wf check`
/// and the editor offer it; then what is left is said. `--dry-run` writes
/// nothing and says what would change.
pub(crate) fn run_fix(dir: &Path, code: Option<&str>, dry_run: bool) -> Result<()> {
    let done = seven::fix(dir, &|d| code.is_none_or(|c| d.code == c), dry_run)?;
    if dry_run {
        for file in done.by_file.keys() {
            println!("would change {file}");
        }
    }
    let total = done.total();
    let verb = if dry_run { "Would fix" } else { "Fixed" };
    if total == 0 {
        println!("Nothing to fix.");
    } else {
        let codes: Vec<String> = done
            .by_code
            .iter()
            .map(|(c, n)| format!("{n} {c}"))
            .collect();
        println!(
            "{verb} {total} finding{} in {} file{}: {}",
            if total == 1 { "" } else { "s" },
            done.by_file.len(),
            if done.by_file.len() == 1 { "" } else { "s" },
            codes.join(", ")
        );
    }
    if !dry_run && !done.left.is_empty() {
        let with_fix = done.left.iter().filter(|d| !d.fixes.is_empty()).count();
        println!(
            "{} left, which a person decides{}; `wf check` lists them",
            done.left.len(),
            if with_fix > 0 {
                format!(" ({with_fix} with a fix that did not fit)")
            } else {
                String::new()
            }
        );
    }
    Ok(())
}
