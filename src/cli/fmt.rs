use std::path::{Path, PathBuf};

use crate::error::{Result, WebFluentError};
use crate::layout::{to_braces, to_offside};
use crate::migrate::source_files;

/// `wf fmt [path] [--check] [--stdout]`: every source file of a project (or
/// the one file named) in its canonical spelling, written back; `--check`
/// writes nothing and fails when a file would change. With `--to wfx|wf`
/// the files are rewritten in the other layout instead, and renamed.
pub fn run_fmt(path: &Path, to: Option<&str>, check: bool, stdout: bool) -> Result<()> {
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
    match to {
        Some(layout) => change_layout(&files, layout, stdout),
        None => format_files(&files, check, stdout),
    }
}

/// Every file in its canonical spelling.
fn format_files(files: &[(PathBuf, String)], check: bool, stdout: bool) -> Result<()> {
    let mut changed = 0;
    let mut failed = 0;
    let mut would_change: Vec<String> = Vec::new();
    for (file, text) in files {
        let name = file.to_string_lossy();
        match crate::fmt::format_source(text, &name) {
            Ok(out) => {
                if stdout {
                    print!("{out}");
                    continue;
                }
                if out == *text {
                    continue;
                }
                if check {
                    would_change.push(file.display().to_string());
                    continue;
                }
                std::fs::write(file, out)?;
                changed += 1;
                println!("  {}", file.display());
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
    if check {
        if !would_change.is_empty() {
            return Err(WebFluentError::IoError(format!(
                "{} file(s) are not formatted: {}",
                would_change.len(),
                would_change.join(", ")
            )));
        }
        println!("{} file(s) formatted", files.len() - failed);
    } else {
        println!(
            "{changed} file(s) formatted, {} already were, {failed} failed",
            files.len() - changed - failed
        );
    }
    if failed > 0 {
        return Err(WebFluentError::IoError(format!(
            "{failed} file(s) could not be formatted"
        )));
    }
    Ok(())
}

/// Every file rewritten in the other layout, and renamed to its extension.
/// A file already in the layout asked for is left alone.
fn change_layout(files: &[(PathBuf, String)], to: &str, stdout: bool) -> Result<()> {
    let target = match to {
        "wfx" | "offside" | "indent" => "wfx",
        "wf" | "braces" => "wf",
        other => {
            return Err(WebFluentError::IoError(format!(
                "`--to {other}` is not a layout; write `--to wfx` or `--to wf`"
            )));
        }
    };
    let mut changed = 0;
    let mut failed = 0;
    for (file, text) in files {
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
