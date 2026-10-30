//! What a component's call writes on it — an `on click`, a `class:`, an
//! attribute — lands on the root element the component draws, and follows
//! it when the root is an `if` or a `match` that draws another one. Run as
//! a reader runs it, the `wf` binary over a scratch project.

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str, app: &str, test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-component-root-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build", "split": false } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    std::fs::write(dir.join("tests/root.wf"), test).unwrap();
    dir
}

fn wf_test(dir: &Path) -> Option<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("test")
        .current_dir(dir)
        .output()
        .expect("wf runs");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if text.contains("no Chrome or Chromium") {
        eprintln!("\n  SKIPPED: no Chrome or Chromium for `wf test`.\n");
        return None;
    }
    assert!(out.status.success(), "{text}");
    Some(text)
}

// A button that explains itself while disabled: its root is a `Tooltip`
// in one branch and a bare `Button` in the other.
const BTN: &str = r#"component Btn(_ label: String, chip: String = "", disabled: Bool = false, why: String = "") {
    if chip == "" && disabled && why != "" {
        Tooltip(why) { Button(label, disabled: disabled) }
    } else {
        Button(label, disabled: disabled) { if chip != "" { Text(chip) } }
    }
}

page P(path: "/", title: "T", description: "d") {
    Heading("x").h1
}
"#;

#[test]
fn a_handler_on_the_call_follows_the_root_an_if_draws_again() {
    let dir = scratch(
        "if",
        BTN,
        r#"test "the push button pushes once it is enabled" {
    state n = 0
    state clicked = 0
    Btn("Push", chip: if n > 0 { "{n}" } else { "" }, disabled: n == 0, why: "Nothing to push.", data-kind: "push") {
        on click { clicked = clicked + 1 }
    }
    Button("More") { on click { n = n + 1 } }
    Text("clicked {clicked}")
    click "More"
    click "Push1"
    expect "clicked 1"
}
"#,
    );
    let Some(out) = wf_test(&dir) else { return };
    assert!(out.contains("1 passed, 0 failed"), "{out}");
}

#[test]
fn a_handler_on_the_call_follows_the_root_a_match_draws_again() {
    let dir = scratch(
        "match",
        r#"enum Mode { idle, busy }

component Toggle(_ label: String, mode: Mode = .idle) {
    match mode {
        .idle { Button(label) }
        .busy { Button("{label}…") }
    }
}

page P(path: "/", title: "T", description: "d") {
    Heading("x").h1
}
"#,
        r#"test "a match's new arm keeps the call's handler and attributes" {
    state mode: Mode = .idle
    state clicked = 0
    Toggle("Go", mode: mode, aria-label: "Go now") { on click { clicked = clicked + 1 } }
    Button("Busy") { on click { mode = .busy } }
    Text("clicked {clicked}")
    click "Busy"
    click "Go now"
    expect "clicked 1"
}
"#,
    );
    let Some(out) = wf_test(&dir) else { return };
    assert!(out.contains("1 passed, 0 failed"), "{out}");
}
