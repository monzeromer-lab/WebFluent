//! Build the fixture sites, then run their bundles in a DOM.
//!
//! `e2e_sites.rs` asserts on the files a build writes. This one goes further and
//! executes what it wrote: it drives `tests/js/rendered_site.test.mjs`, which
//! loads each `app.js` against the fake DOM in `tests/js/dom.mjs`, mounts it and
//! asserts on the live page — clicks, state, stores, routing.
//!
//! Keeping the entry point here means `cargo test` covers the JavaScript half
//! too, rather than leaving it to a step somebody has to remember.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Sites the rendered-behaviour suite mounts.
const NEEDED: &[&str] = &["gallery", "marketing", "dashboard", "bespoke"];

fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("create dir");
    for entry in std::fs::read_dir(from).expect("read dir") {
        let entry = entry.expect("entry");
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).expect("copy");
        }
    }
}

fn build_fixture(name: &str) {
    let root = repo_root();
    let dir = root.join("target/e2e").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_tree(&root.join("tests/fixtures").join(name), &dir);

    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("build")
        .current_dir(&dir)
        .output()
        .expect("running `wf build`");
    assert!(
        out.status.success(),
        "{name} failed to build:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// The compiled bundles must run, and the pages they paint must behave.
#[test]
fn built_bundles_behave_when_executed() {
    if !node_available() {
        eprintln!(
            "\n  SKIPPED: `node` is not on PATH, so the rendered-behaviour suite \
             (tests/js/rendered_site.test.mjs) did not run.\n  \
             Install Node to cover the JavaScript half of the output.\n"
        );
        return;
    }

    for name in NEEDED {
        build_fixture(name);
    }

    let out = Command::new("node")
        .args(["--test", "tests/js/rendered_site.test.mjs"])
        .current_dir(repo_root())
        .output()
        .expect("running node --test");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "the built sites did not behave when executed:\n{stdout}\n{stderr}"
    );
}

/// The runtime's own unit tests, so one `cargo test` covers every layer.
#[test]
fn runtime_unit_tests_pass() {
    if !node_available() {
        eprintln!("\n  SKIPPED: `node` is not on PATH; tests/js/hydrate.test.mjs did not run.\n");
        return;
    }

    let out = Command::new("node")
        .args(["--test", "tests/js/hydrate.test.mjs"])
        .current_dir(repo_root())
        .output()
        .expect("running node --test");

    assert!(
        out.status.success(),
        "runtime tests failed:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
