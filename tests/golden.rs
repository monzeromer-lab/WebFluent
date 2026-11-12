//! Every project the repository holds — the examples, the test fixtures and
//! the documentation site — built, and what each wrote held to a manifest:
//! every output file's path and SHA-256, and every finding the build
//! reported (code, severity, message, place, hint, fixes), in
//! `tests/golden/<project>.json`.
//!
//! A change that alters what a project builds to, or what the compiler
//! says about it, fails here, naming each file added, removed or changed
//! and each finding that came or went. When the change is the point,
//! write the manifests again and commit them with it:
//!
//!     UPDATE_GOLDEN=1 cargo test --release --test it -- golden::
//!
//! The trees that hold projects are copied whole to a scratch folder first,
//! keeping their layout, so a build never writes into the repository and a
//! project that reads a folder beside it (`"fonts": ["../fonts"]`) finds it;
//! what a build writes does not depend on the folder it is built in. A build
//! that borrowed a font from the machine, or found none for a character,
//! writes a PDF that differs from one machine to the next: its manifest says
//! `"machine_fonts": true` and records its PDFs as `machine-dependent`, and
//! every other file it writes is held as usual. The
//! WebFluent 2 projects under `tests/migrate/corpus/` are left out: a build
//! refuses them by design.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};
use webfluent::build::{BuildOptions, build};
use webfluent::vfs::FsVfs;

/// Folders a project walk never enters: what a build writes and what a
/// package manager installs.
const SKIP: &[&str] = &["build", "node_modules", ".wf-cache", ".git"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The trees that hold the projects, relative to the repository.
const TREES: &[&str] = &["examples", "tests/fixtures", "site"];

/// Every folder holding a `webfluent.app.json`, relative to the repository.
fn projects() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        if dir.join("webfluent.app.json").is_file() {
            out.push(dir.to_path_buf());
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                !SKIP.contains(&name)
            })
            .collect();
        dirs.sort();
        for d in dirs {
            walk(&d, out);
        }
    }
    let root = root();
    let mut out = Vec::new();
    for top in TREES {
        walk(&root.join(top), &mut out);
    }
    out.into_iter()
        .map(|p| {
            p.strip_prefix(&root)
                .expect("under the repository")
                .to_path_buf()
        })
        .collect()
}

/// `tests/fixtures/behaviour` → `tests--fixtures--behaviour`.
fn golden_name(project: &Path) -> String {
    project
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("--")
}

fn golden_dir() -> PathBuf {
    root().join("tests/golden")
}

fn copy_project(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("scratch folder");
    for entry in std::fs::read_dir(from).expect("project folder").flatten() {
        let name = entry.file_name();
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            if SKIP.contains(&name.to_string_lossy().as_ref()) {
                continue;
            }
            copy_project(&path, &to.join(&name));
        } else if kind.is_file() {
            std::fs::copy(&path, to.join(&name)).expect("copy a project file");
        }
    }
}

/// Every file under `dir`, by its path from `dir` (with `/`), and its hash.
fn manifest(dir: &Path) -> BTreeMap<String, String> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else if let Ok(bytes) = std::fs::read(&path) {
                let rel = path
                    .strip_prefix(base)
                    .expect("under the output")
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let digest = Sha256::digest(&bytes);
                let hex = digest.iter().fold(String::new(), |mut acc, b| {
                    let _ = write!(acc, "{b:02x}");
                    acc
                });
                out.insert(rel, hex);
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// What a project builds to: its manifest, or the error that stopped it
/// (with the scratch folder taken out, so it reads the same anywhere).
fn build_one(project: &Path, scratch: &Path) -> serde_json::Value {
    let work = scratch.join(project);
    let out = scratch.join("__golden_out").join(golden_name(project));
    let options = BuildOptions {
        output_dir: Some(out.clone()),
        ..BuildOptions::new(&work, &FsVfs)
    };
    let here = work.to_string_lossy().into_owned();
    let findings = |report: &webfluent::build::BuildReport| -> Vec<String> {
        report
            .diagnostics
            .iter()
            .map(|d| {
                serde_json::to_string(d)
                    .expect("a finding serialises")
                    .replace(&here, "<project>")
            })
            .collect()
    };
    // A PDF drawn with this machine's fonts, or without a font for some of
    // its text, is not the same PDF elsewhere.
    let unreproducible = |report: &webfluent::build::BuildReport| -> Vec<String> {
        report
            .lines
            .iter()
            .map(|l| l.text.trim().to_string())
            .filter(|t| t.contains("came from this machine") || t.contains("no font has"))
            .collect()
    };
    let value = match build(&options) {
        Ok(report) if !unreproducible(&report).is_empty() => {
            let files: BTreeMap<String, String> = manifest(&out)
                .into_iter()
                .map(|(path, hash)| {
                    let shown = if path.ends_with(".pdf") {
                        "machine-dependent".to_string()
                    } else {
                        hash
                    };
                    (path, shown)
                })
                .collect();
            serde_json::json!({
                "diagnostics": findings(&report),
                "files": files,
                "machine_fonts": true,
            })
        }
        Ok(report) => serde_json::json!({
            "diagnostics": findings(&report),
            "files": manifest(&out),
        }),
        Err(failure) => serde_json::json!({
            "diagnostics": findings(&failure.report),
            "error": failure.error.to_string().replace(&here, "<project>"),
        }),
    };
    let _ = std::fs::remove_dir_all(&out);
    value
}

/// A change between two manifests, one line per file.
fn differences(was: &serde_json::Value, now: &serde_json::Value) -> Vec<String> {
    let files = |v: &serde_json::Value| -> BTreeMap<String, String> {
        v.get("files")
            .and_then(|f| f.as_object())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    if was.get("machine_fonts") != now.get("machine_fonts") {
        out.push(format!(
            "machine fonts: {} → {} (a PDF now draws, or no longer draws, with this machine's fonts)",
            was.get("machine_fonts").is_some(),
            now.get("machine_fonts").is_some()
        ));
    }
    if was.get("error") != now.get("error") {
        out.push(format!(
            "build result: {} → {}",
            was.get("error").map_or("built".into(), ToString::to_string),
            now.get("error").map_or("built".into(), ToString::to_string)
        ));
    }
    let findings = |v: &serde_json::Value| -> Vec<String> {
        v.get("diagnostics")
            .and_then(|d| d.as_array())
            .map(|d| {
                d.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let (fa, fb) = (findings(was), findings(now));
    for f in fb.iter().filter(|f| !fa.contains(f)) {
        out.push(format!("finding  + {f}"));
    }
    for f in fa.iter().filter(|f| !fb.contains(f)) {
        out.push(format!("finding  - {f}"));
    }
    let (a, b) = (files(was), files(now));
    for (path, hash) in &b {
        match a.get(path) {
            None => out.push(format!("added    {path}")),
            Some(old) if old != hash => out.push(format!("changed  {path}")),
            Some(_) => {}
        }
    }
    for path in a.keys().filter(|p| !b.contains_key(*p)) {
        out.push(format!("removed  {path}"));
    }
    out
}

#[test]
fn every_project_builds_to_its_golden_manifest() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let projects = projects();
    assert!(
        projects.len() >= 40,
        "found only {} projects; the walk has gone wrong",
        projects.len()
    );
    let scratch = std::env::temp_dir().join(format!("wf-golden-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    for tree in TREES {
        copy_project(&root().join(tree), &scratch.join(tree));
    }

    // A few at a time: each build is heavy, and the machines this runs on
    // are not all large.
    let results: Mutex<Vec<(PathBuf, serde_json::Value)>> = Mutex::new(Vec::new());
    let queue: Mutex<Vec<PathBuf>> = Mutex::new(projects.iter().rev().cloned().collect());
    let workers = std::thread::available_parallelism().map_or(2, |n| n.get().clamp(1, 4));
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                loop {
                    let next = queue.lock().expect("queue lock").pop();
                    let Some(project) = next else { break };
                    let value = build_one(&project, &scratch);
                    results.lock().expect("results lock").push((project, value));
                }
            });
        }
    });
    let _ = std::fs::remove_dir_all(&scratch);
    let mut results = results.into_inner().expect("results lock");
    results.sort_by(|a, b| a.0.cmp(&b.0));

    let dir = golden_dir();
    let mut failures = Vec::new();
    let mut expected_names = Vec::new();
    for (project, now) in &results {
        let name = format!("{}.json", golden_name(project));
        expected_names.push(name.clone());
        let file = dir.join(&name);
        if update {
            std::fs::create_dir_all(&dir).expect("golden folder");
            let text = format!(
                "{}\n",
                serde_json::to_string_pretty(now).expect("a manifest serialises")
            );
            std::fs::write(&file, text).expect("write a golden manifest");
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            failures.push(format!(
                "{}: no golden manifest ({name})",
                project.display()
            ));
            continue;
        };
        let was: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name} is not JSON: {e}"));
        let changes = differences(&was, now);
        if !changes.is_empty() {
            failures.push(format!(
                "{}:\n    {}",
                project.display(),
                changes.join("\n    ")
            ));
        }
    }
    // A manifest whose project is gone.
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".json") && !expected_names.contains(&name) {
                if update {
                    std::fs::remove_file(entry.path()).expect("remove a stale manifest");
                } else {
                    failures.push(format!("{name}: no project builds to it any more"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "what a project builds to changed. If that is the point, run \
         `UPDATE_GOLDEN=1 cargo test --release --test it -- golden::` and commit the \
         manifests with the change:\n\n{}",
        failures.join("\n\n")
    );
}
