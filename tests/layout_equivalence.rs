//! `wf fmt --to wfx` is a change of layout and nothing else. The projects
//! under `tests/layout` — the fixtures, the documentation site and each
//! `wf init` template as they stood when WebFluent 3 arrived — written by
//! indentation must build byte for byte what their braced spelling builds,
//! and come back to that spelling.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn wf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wf"))
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "build" || entry.file_name() == "node_modules" {
                continue;
            }
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

/// Every file under `dir`, keyed by relative path.
fn outputs(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                out.insert(rel, std::fs::read(&path).unwrap());
            }
        }
    }

    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// Build the project at `dir` and hand back its outputs.
fn build(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let output = wf().arg("build").current_dir(dir).output().unwrap();
    assert!(
        output.status.success(),
        "build failed in {}:\n{}{}",
        dir.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("webfluent.app.json")).unwrap())
            .unwrap();
    let out = config["build"]["output"].as_str().unwrap_or("./build");
    outputs(&dir.join(out))
}

/// Migrate the project at `dir` in place; every `.wf` file must come out in
/// the new grammar.
fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walkdir(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

/// `a` around the first byte where it differs from `b`.
fn excerpt(a: &[u8], b: &[u8]) -> String {
    let at = a
        .iter()
        .zip(b.iter())
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()));
    let line = a[..at].iter().filter(|&&c| c == b'\n').count() + 1;
    let start = at.saturating_sub(160);
    let end = (at + 160).min(a.len());
    format!(
        "line {line}, byte {at}: …{}…",
        String::from_utf8_lossy(&a[start..end])
    )
}

/// Every source file under `dir` with the extension `ext`.
fn sources(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = walkdir(dir)
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == ext))
        .collect();
    files.sort();
    files
}

fn fmt_to(dir: &Path, layout: &str) -> String {
    let output = wf()
        .arg("fmt")
        .arg("--to")
        .arg(layout)
        .arg(dir)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "fmt --to {layout} failed in {}:\n{text}",
        dir.display()
    );
    text
}

#[test]
fn every_project_builds_the_same_by_indentation_and_comes_back() {
    let corpus = repo_root().join("tests/layout");
    let mut names: Vec<String> = std::fs::read_dir(&corpus)
        .unwrap()
        .flatten()
        .filter(|e| e.path().join("webfluent.app.json").exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert!(names.len() >= 12, "{names:?}");
    for name in names {
        let root = repo_root().join("target/layout").join(&name);
        let _ = std::fs::remove_dir_all(&root);
        copy_tree(&corpus.join(&name), &root);
        let braced: BTreeMap<PathBuf, String> = sources(&root.join("src"), "wf")
            .into_iter()
            .map(|p| {
                let text = std::fs::read_to_string(&p).unwrap();
                (p, text)
            })
            .collect();
        let before = build(&root);

        // To the indented layout: every `.wf` becomes a `.wfx`, and the
        // project builds the same.
        fmt_to(&root, "wfx");
        assert!(
            sources(&root.join("src"), "wf").is_empty(),
            "{name}: a .wf remained"
        );
        assert_eq!(
            sources(&root.join("src"), "wfx").len(),
            braced.len(),
            "{name}: a file was lost"
        );
        let after = build(&root);
        assert_eq!(
            before.keys().collect::<Vec<_>>(),
            after.keys().collect::<Vec<_>>(),
            "{name}: the indented project wrote different files"
        );
        for (file, bytes) in &before {
            assert!(
                after[file] == *bytes,
                "{name}: {file} differs between the braced and the indented project\n{}",
                excerpt(bytes, &after[file])
            );
        }

        // And back: the braced spelling, line for line — a blank line
        // before a closing brace, and an empty block's brace on a line of
        // its own, are what the indented layout cannot keep.
        fmt_to(&root, "wf");
        let lines = |text: &str| {
            let mut out: Vec<String> = Vec::new();
            for line in text.lines().map(str::trim_end).filter(|l| !l.is_empty()) {
                if line.trim() == "}" && out.last().is_some_and(|p| p.ends_with('{')) {
                    let last = out.last_mut().unwrap();
                    last.push_str(" }");
                } else {
                    out.push(line.to_string());
                }
            }
            out.join("\n")
        };
        for (path, text) in &braced {
            let again = std::fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("{name}: {} did not come back", path.display()));
            let (was, back) = (lines(text), lines(&again));
            assert!(
                was == back,
                "{name}: {} did not come back as it was\n--- was ---\n{}\n--- came back ---\n{}",
                path.display(),
                excerpt(was.as_bytes(), back.as_bytes()),
                excerpt(back.as_bytes(), was.as_bytes())
            );
        }
    }
}
