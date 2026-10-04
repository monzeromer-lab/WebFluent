//! The guide's tutorials are real projects: each under `examples/tutorials/`
//! builds with nothing to report, and every block of the chapter that names
//! a project file — a first line `// src/…` or `// tests/…` — is that file,
//! or a run of it, word for word. The chapter cannot teach code the
//! projects no longer have.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn projects() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(root().join("examples/tutorials"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("webfluent.app.json").is_file())
        .collect();
    out.sort();
    out
}

/// A copy of a project without what a build wrote, so the test never
/// touches the repository.
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        let name = e.file_name();
        if name == "build" || name == ".wf-cache" || name == ".wf-sizes.json" {
            continue;
        }
        if p.is_dir() {
            copy(&p, &to.join(&name));
        } else {
            std::fs::copy(&p, to.join(&name)).unwrap();
        }
    }
}

#[test]
fn there_are_three_tutorials() {
    let names: Vec<String> = projects()
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(names.len(), 3, "{names:?}");
    assert!(names.iter().any(|n| n.starts_with("easy-")), "{names:?}");
    assert!(names.iter().any(|n| n.starts_with("medium-")), "{names:?}");
    assert!(names.iter().any(|n| n.starts_with("hard-")), "{names:?}");
}

#[test]
fn every_tutorial_builds_and_checks_clean() {
    for project in projects() {
        let name = project.file_name().unwrap().to_string_lossy().to_string();
        let tmp = std::env::temp_dir().join(format!("wf-tutorial-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        copy(&project, &tmp);
        for args in [
            &["check", "--deny-warnings", "-d"][..],
            &["build", "-d"][..],
        ] {
            let out = Command::new(env!("CARGO_BIN_EXE_wf"))
                .args(args)
                .arg(&tmp)
                .output()
                .unwrap();
            let text = String::from_utf8_lossy(&out.stdout).to_string()
                + &String::from_utf8_lossy(&out.stderr);
            assert!(out.status.success(), "{name}: wf {}: {text}", args[0]);
            assert!(
                !text.contains("warning["),
                "{name}: wf {} warns: {text}",
                args[0]
            );
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

/// The chapter's blocks that name a project file: `(line, path, body)`.
fn file_blocks(markdown: &str) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    let mut current: Option<(usize, Vec<&str>)> = None;
    for (ix, line) in markdown.lines().enumerate() {
        match &mut current {
            Some((_, lines)) => {
                if line.trim_end() == "```" {
                    let (at, lines) = current.take().unwrap();
                    if let Some(first) = lines.first()
                        && let Some(path) = first.strip_prefix("// ")
                        && (path.starts_with("src/") || path.starts_with("tests/"))
                        && !path.contains(' ')
                    {
                        out.push((at, path.to_string(), lines[1..].join("\n")));
                    }
                } else {
                    lines.push(line);
                }
            }
            None if matches!(line.trim_end(), "```wf" | "```wfx" | "```css" | "```json") => {
                current = Some((ix + 1, Vec::new()))
            }
            None => {}
        }
    }
    out
}

#[test]
fn the_chapter_shows_the_projects_as_they_are() {
    let chapter = std::fs::read_to_string(root().join("md-docs/35-tutorials.md")).unwrap();
    let blocks = file_blocks(&chapter);
    assert!(
        blocks.len() >= 12,
        "the tutorials show their files: {}",
        blocks.len()
    );
    let mut stale = Vec::new();
    for (line, path, body) in blocks {
        let found = projects()
            .iter()
            .any(|p| std::fs::read_to_string(p.join(&path)).is_ok_and(|file| file.contains(&body)));
        if !found {
            stale.push(format!("35-tutorials.md:{line} — {path}"));
        }
    }
    assert!(
        stale.is_empty(),
        "these blocks differ from the project files they name: {stale:#?}"
    );
}
