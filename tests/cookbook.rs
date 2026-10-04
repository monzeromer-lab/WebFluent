//! Three complete applications — todos, a static blog, a guarded dashboard —
//! built with the `wf` binary and run by `tests/js/cookbook.test.mjs` against
//! the fake DOM.
//!
//! `docs_parse.rs` holds every block in the documentation to the grammar and
//! the semantic checks. That stops short of the back end: a block can parse,
//! type-check and still compile to JavaScript that throws on the first click.
//! These applications were the guide's cookbook; when the tutorials replaced
//! it they moved to `tests/fixtures/cookbook/`, where they keep exercising
//! the runtime end to end.

use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// An application: what to call it, its source under
/// `tests/fixtures/cookbook/`, the config it needs, and any file it reads.
struct App {
    name: &'static str,
    heading: &'static str,
    config: &'static str,
    files: &'static [(&'static str, &'static str)],
}

const POSTS: &str = r#"[
  { "slug": "hello", "title": "Hello", "summary": "The first post",
    "date": "2026-01-01", "tags": ["intro"], "body": "Hi there." },
  { "slug": "second", "title": "Second", "summary": "Another one",
    "date": "2026-02-01", "tags": ["notes"], "body": "More of it." }
]"#;

const APPS: &[App] = &[
    App {
        name: "cookbook-todos",
        heading: "todos.wf",
        config: r#"{ "name": "todos", "build": { "output": "build" } }"#,
        files: &[],
    },
    App {
        name: "cookbook-blog",
        // Two themes, so the build is told which one is the light default.
        heading: "blog.wf",
        config: r#"{ "name": "blog", "build": { "output": "build", "ssg": true },
                     "theme": { "name": "Paper" } }"#,
        files: &[("src/posts.json", POSTS)],
    },
    App {
        name: "cookbook-dashboard",
        heading: "dashboard.wf",
        config: r#"{ "name": "dashboard", "build": { "output": "build" } }"#,
        files: &[],
    },
];

/// Write `app` into `target/e2e/<name>` and build it there.
fn build(app: &App) -> (bool, String) {
    let dir = repo_root().join("target/e2e").join(app.name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).expect("create the project");
    std::fs::write(dir.join("webfluent.app.json"), app.config).expect("write the config");
    let source = std::fs::read_to_string(
        repo_root()
            .join("tests/fixtures/cookbook")
            .join(app.heading),
    )
    .expect("read the application");
    std::fs::write(dir.join("src/App.wf"), source).expect("write the source");
    for (rel, body) in app.files {
        std::fs::write(dir.join(rel), body).expect("write a data file");
    }

    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("build")
        .current_dir(&dir)
        .output()
        .expect("running `wf build`");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// Every application built once, however many tests ask for them: each build
/// clears and rewrites its own directory, so letting the tests race would have
/// one deleting the tree another is reading.
static BUILT: LazyLock<Vec<(bool, String)>> = LazyLock::new(|| APPS.iter().map(build).collect());

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Every application builds, and cleanly: a warning is a failure here too.
#[test]
fn the_cookbook_applications_build_as_printed() {
    let mut failures = Vec::new();
    for (app, (ok, out)) in APPS.iter().zip(BUILT.iter()) {
        if !ok {
            failures.push(format!("{} did not build:\n{out}", app.heading));
        } else if out.contains("warning[") {
            failures.push(format!("{} builds with warnings:\n{out}", app.heading));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// …and the bundles they produce behave when a reader clicks through them.
#[test]
fn the_cookbook_applications_behave_when_executed() {
    if !node_available() {
        eprintln!(
            "\n  SKIPPED: `node` is not on PATH, so the cookbook applications \
             (tests/js/cookbook.test.mjs) were not run.\n"
        );
        return;
    }

    for (app, (ok, out)) in APPS.iter().zip(BUILT.iter()) {
        assert!(ok, "{} did not build:\n{out}", app.heading);
    }

    let out = Command::new("node")
        .args(["--test", "tests/js/cookbook.test.mjs"])
        .current_dir(repo_root())
        .output()
        .expect("running node --test");

    assert!(
        out.status.success(),
        "the cookbook applications did not behave when executed:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The Todos app is the one a store's `remove` action broke: it must
/// keep calling the action, never a list's `splice`.
#[test]
fn a_store_action_survives_into_the_cookbooks_bundle() {
    let (ok, out) = &BUILT[0];
    assert!(ok, "the todo app did not build:\n{out}");

    let dir = repo_root().join("target/e2e/cookbook-todos/build");
    let mut js = std::fs::read_to_string(dir.join("app.js")).expect("read app.js");
    let pages = dir.join("pages");
    if pages.is_dir() {
        for entry in std::fs::read_dir(&pages).unwrap().flatten() {
            if entry.path().extension().is_some_and(|e| e == "js") {
                js.push_str(&std::fs::read_to_string(entry.path()).unwrap());
            }
        }
    }
    assert!(
        js.contains("Todos.remove("),
        "the store's `remove` action is not called in the bundle"
    );
    assert!(
        !js.contains("Todos.splice("),
        "the store is being handed a list's `splice`"
    );
}
