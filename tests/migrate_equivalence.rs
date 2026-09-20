//! `wf migrate` is a change of spelling and nothing else: every project of
//! the corpus under `tests/migrate/corpus` — the fixtures, the documentation
//! site and each `wf init` template as they were written in the original
//! grammar — must build to byte-identical output before and after it.
//!
//! This is the gate for the new parser, the resolver and the migrator at
//! once: a difference in any of them shows up here as a diff in a built
//! file. The corpus is frozen: the live projects have moved to the new
//! grammar, and these copies are what the migrator is still measured on.

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
    let mut out = BTreeMap::new();
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
fn migrate(dir: &Path) -> String {
    let output = wf().arg("migrate").arg(dir).output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "migrate failed in {}:\n{text}",
        dir.display()
    );
    text
}

fn first_word_is_lowercase(path: &Path) {
    let text = std::fs::read_to_string(path).unwrap();
    let dialect = webfluent::detect_dialect(&text);
    assert_eq!(
        dialect,
        webfluent::Dialect::V2,
        "{} was not migrated:\n{text}",
        path.display()
    );
}

fn assert_same_build(name: &str, project: &Path) {
    let root = repo_root().join("target/migrate").join(name);
    let _ = std::fs::remove_dir_all(&root);
    copy_tree(project, &root);
    let before = build(&root);
    let notes = migrate(&root);
    for entry in walkdir(&root.join("src")) {
        if entry.extension().is_some_and(|e| e == "wf") {
            first_word_is_lowercase(&entry);
        }
    }
    // A migrated project is done: a second pass has nothing to say.
    let again = wf()
        .arg("migrate")
        .arg("--check")
        .arg(&root)
        .output()
        .unwrap();
    let again = String::from_utf8_lossy(&again.stdout).to_string();
    assert!(
        again.contains("0 file(s) would change"),
        "{name}: a second migration would change the project again:\n{again}"
    );
    let after = build(&root);
    let mut diffs = Vec::new();
    for (file, bytes) in &before {
        // A compressed copy differs exactly when its source does.
        if file.ends_with(".gz") {
            continue;
        }
        match after.get(file) {
            Some(b) if b == bytes => {}
            Some(b) => diffs.push(format!(
                "{file} differs\n--- before ---\n{}\n--- after ---\n{}",
                excerpt(bytes, b),
                excerpt(b, bytes)
            )),
            None => diffs.push(format!("{file} is missing after migration")),
        }
    }
    for file in after.keys() {
        if !before.contains_key(file) && !file.ends_with(".gz") {
            diffs.push(format!("{file} appeared after migration"));
        }
    }
    assert!(
        diffs.is_empty(),
        "{name}: the migrated project builds differently\n{}\nmigration output:\n{notes}",
        diffs.join("\n")
    );
}

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

#[test]
fn every_project_of_the_corpus_builds_the_same_after_migration() {
    let corpus = repo_root().join("tests/migrate/corpus");
    let mut names: Vec<String> = std::fs::read_dir(&corpus)
        .unwrap()
        .flatten()
        .filter(|e| e.path().join("webfluent.app.json").exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert!(names.len() >= 12, "{names:?}");
    for name in names {
        assert_same_build(&name, &corpus.join(&name));
    }
}
