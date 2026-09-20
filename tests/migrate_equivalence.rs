//! `wf migrate` is a change of spelling and nothing else. The corpus under
//! `tests/migrate/corpus` holds every project of the repository — the
//! fixtures, the documentation site, each `wf init` template — as it was
//! written in the grammar of WebFluent 2; `tests/migrate/expected` holds
//! what the migration writes for each, reviewed, and proven while both
//! grammars still built to have built byte for byte what the original
//! did. The migration must still write exactly that, every result must
//! build, and a second migration must have nothing to say.

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

fn assert_migrates_as_expected(name: &str, project: &Path) {
    let root = repo_root().join("target/migrate").join(name);
    let _ = std::fs::remove_dir_all(&root);
    copy_tree(project, &root);
    let notes = migrate(&root);
    let expected = repo_root()
        .join("tests/migrate/expected")
        .join(name)
        .join("src");
    let mut diffs = Vec::new();
    for path in walkdir(&root.join("src")) {
        if !path.extension().is_some_and(|e| e == "wf") {
            continue;
        }
        first_word_is_lowercase(&path);
        let rel = path.strip_prefix(root.join("src")).unwrap();
        let got = std::fs::read(&path).unwrap();
        match std::fs::read(expected.join(rel)) {
            Ok(want) if want == got => {}
            Ok(want) => diffs.push(format!(
                "{} differs from the expected migration\n--- expected ---\n{}\n--- got ---\n{}",
                rel.display(),
                excerpt(&want, &got),
                excerpt(&got, &want)
            )),
            Err(_) => diffs.push(format!("{} has no expected migration", rel.display())),
        }
    }
    assert!(
        diffs.is_empty(),
        "{name}: the migration changed\n{}\nmigration output:\n{notes}",
        diffs.join("\n")
    );
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
    // And it builds.
    build(&root);
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
        assert_migrates_as_expected(&name, &corpus.join(&name));
    }
}
