//! `on menu(key, x, y)`: a handler names each value a component's event
//! sends, and a name past the last one is refused. Run as a reader runs it,
//! the `wf` binary over a scratch project.

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str, app: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-handler-values-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build", "split": false } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    dir
}

fn wf(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("wf runs");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

const ENTRY: &str = "component Entry(_ label: String) {\n    event menu(key: String, x: Number, y: Number)\n    Button(label) { on click { emit menu(label, 12, 34) } }\n}\n";

fn app(handler: &str) -> String {
    format!(
        "{ENTRY}page P(path: \"/\", title: \"T\", description: \"d\") {{\n    state said = \"nothing\"\n    Heading(\"x\").h1\n    Entry(\"row-1\") {{ {handler} }}\n    Text(said)\n}}\n"
    )
}

#[test]
fn the_caller_receives_every_value_the_event_sends() {
    let dir = scratch(
        "build",
        &app("on menu(key, x, y) { said = \"{key} at {x}, {y}\" }"),
    );
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let js: String = std::fs::read_to_string(dir.join("build/app.js"))
        .unwrap()
        .split_whitespace()
        .collect();
    assert!(js.contains("menu:(key,x,y)=>"), "{js}");
}

#[test]
fn a_three_value_event_reaches_its_handler_in_a_browser() {
    let dir = scratch(
        "run",
        &app("on menu(key, x, y) { said = \"{key} at {x}, {y}\" }"),
    );
    std::fs::write(
        dir.join("tests/menu.wf"),
        "test \"a three-value event reaches its handler\" {\n    state said = \"nothing\"\n    Entry(\"row-1\") { on menu(key, x, y) { said = \"{key} at {x + 1}, {y}\" } }\n    Text(said)\n    click \"row-1\"\n    expect \"row-1 at 13, 34\"\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["test"]);
    if out.contains("no Chrome or Chromium") {
        eprintln!("\n  SKIPPED: no Chrome or Chromium for `wf test`.\n");
        return;
    }
    assert!(ok, "{out}");
    assert!(out.contains("1 passed, 0 failed"), "{out}");
}

#[test]
fn a_name_past_the_last_value_is_refused_where_it_is_written() {
    let dir = scratch(
        "extra",
        &app("on menu(key, x, y, z) { said = \"{key} {z}\" }"),
    );
    let (ok, out) = wf(&dir, &["check"]);
    assert!(!ok, "{out}");
    assert!(
        out.contains("error[T22]: `on menu` names 4 values, but `event menu(key: String, x: Number, y: Number)` sends 3 values"),
        "{out}"
    );
    assert!(out.contains("--> src/App.wf:8:41"), "{out}");
    assert!(
        out.contains("Name at most 3, in order: `on menu(key, x, y)`"),
        "{out}"
    );
}

#[test]
fn a_dom_event_sends_the_event_alone() {
    let dir = scratch(
        "dom",
        "page P(path: \"/\", title: \"T\", description: \"d\") {\n    Heading(\"x\").h1\n    Button(\"Go\") { on click(e, where) { log(where) } }\n}\n",
    );
    let (ok, out) = wf(&dir, &["check"]);
    assert!(!ok, "{out}");
    assert!(
        out.contains(
            "error[T22]: `on click` names 2 values, but a DOM event sends one value, the event"
        ),
        "{out}"
    );
    assert!(out.contains("--> src/App.wf:3:32"), "{out}");
}
