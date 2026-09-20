use std::path::Path;

use crate::error::{Result, WebFluentError};
use crate::migrate::{migrate_project, report, source_files};

/// `wf migrate [path] [--check] [--stdout]`.
pub fn run_migrate(path: &Path, check: bool, stdout: bool) -> Result<()> {
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

    let outcomes = migrate_project(&files);
    let mut changed = 0;
    let mut failed = 0;
    let mut notes = 0;
    for (file, outcome) in &outcomes {
        match outcome {
            Ok(migrated) => {
                if stdout {
                    print!("{}", migrated.text);
                    continue;
                }
                if migrated.changed {
                    changed += 1;
                    println!("  {}", file.display());
                }
                notes += migrated.notes.len();
                print!("{}", report(file, migrated));
                if migrated.changed && !check {
                    std::fs::write(file, &migrated.text)?;
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("  {}: {e}", file.display());
            }
        }
    }
    if stdout {
        return Ok(());
    }
    let verb = if check { "would change" } else { "migrated" };
    println!(
        "{changed} file(s) {verb}, {} unchanged, {notes} note(s), {failed} failed",
        outcomes.len() - changed - failed
    );
    if failed > 0 {
        return Err(WebFluentError::IoError(format!(
            "{failed} file(s) could not be migrated"
        )));
    }
    Ok(())
}
