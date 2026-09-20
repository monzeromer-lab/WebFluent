use std::path::Path;

use crate::error::{Result, WebFluentError};
use crate::migrate::{migrate_project, report, source_files};

/// `wf migrate [path] [--check] [--stdout] [--wfx]`.
pub fn run_migrate(path: &Path, check: bool, stdout: bool, wfx: bool) -> Result<()> {
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
                // `--wfx`: the migrated text in the indented layout, under
                // the `.wfx` name; the `.wf` file goes.
                let (text, written_to) = if wfx {
                    let name = file.to_string_lossy();
                    match crate::layout::to_offside(&migrated.text, &name) {
                        Ok(text) => (text, super::fmt::offside_path(file)),
                        Err(e) => {
                            failed += 1;
                            eprintln!("  {}: {e}", file.display());
                            continue;
                        }
                    }
                } else {
                    (migrated.text.clone(), file.clone())
                };
                if stdout {
                    print!("{text}");
                    continue;
                }
                if migrated.changed || wfx {
                    changed += 1;
                    println!("  {}", written_to.display());
                }
                notes += migrated.notes.len();
                print!("{}", report(file, migrated));
                if (migrated.changed || wfx) && !check {
                    std::fs::write(&written_to, &text)?;
                    if written_to != *file {
                        std::fs::remove_file(file)?;
                    }
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("  {}: {e}", file.display());
            }
        }
    }
    if stdout {
        if failed > 0 {
            return Err(WebFluentError::IoError(format!(
                "{failed} file(s) could not be migrated"
            )));
        }
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
