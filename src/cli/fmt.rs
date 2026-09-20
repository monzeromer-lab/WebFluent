use std::path::{Path, PathBuf};

use crate::error::{Result, WebFluentError};
use crate::layout::{to_braces, to_offside};
use crate::migrate::source_files;

/// `wf fmt --to wfx|wf [path] [--stdout]`: every source file of a project
/// (or the one file named) rewritten in the other layout, and renamed to
/// its extension. A file already in the layout asked for is left alone.
pub fn run_fmt(path: &Path, to: &str, stdout: bool) -> Result<()> {
    let target = match to {
        "wfx" | "offside" | "indent" => "wfx",
        "wf" | "braces" => "wf",
        other => {
            return Err(WebFluentError::IoError(format!(
                "`--to {other}` is not a layout; write `--to wfx` or `--to wf`"
            )));
        }
    };
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
            "no .wf or .wfx files under {}",
            path.display()
        )));
    }
    let mut changed = 0;
    let mut failed = 0;
    for (file, text) in &files {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == target {
            continue;
        }
        let name = file.to_string_lossy();
        let converted = if target == "wfx" {
            to_offside(text, &name)
        } else {
            to_braces(text, &name)
        };
        match converted {
            Ok(out) => {
                if stdout {
                    print!("{out}");
                    continue;
                }
                let renamed = file.with_extension(target);
                std::fs::write(&renamed, out)?;
                std::fs::remove_file(file)?;
                changed += 1;
                println!("  {} → {}", file.display(), renamed.display());
            }
            Err(e) => {
                failed += 1;
                eprintln!("  {}: {e}", file.display());
            }
        }
    }
    if !stdout {
        println!(
            "{changed} file(s) written as .{target}, {} already were, {failed} failed",
            files.len() - changed - failed
        );
    }
    if failed > 0 {
        return Err(WebFluentError::IoError(format!(
            "{failed} file(s) could not change layout"
        )));
    }
    Ok(())
}

/// The path a migrated file is written to when `--wfx` asks for the
/// indented layout: the same name with the `.wfx` extension.
pub fn offside_path(file: &Path) -> PathBuf {
    file.with_extension("wfx")
}
