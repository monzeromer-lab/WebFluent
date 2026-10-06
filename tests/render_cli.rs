//! `wf render` with no data renders with none.
//!
//! It read standard input whenever `--data` was missing, so a template that
//! needs no data waited on the keyboard, and an empty pipe was "Invalid JSON
//! data". The launch videos had to start every render with `echo '{}' |`.

use std::io::Write;
use std::process::{Command, Stdio};

fn render(template: &str, stdin: &str) -> (bool, String) {
    let dir = std::env::temp_dir().join(format!(
        "wf-render-cli-{}-{}",
        std::process::id(),
        stdin.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("t.wf"), template).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["render", "t.wf", "--format", "html-fragment"])
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("wf runs");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

const TEMPLATE: &str =
    r#"page P(path: "/", title: "T") { Heading("Hello, {name ?? "world"}").h1 }"#;

#[test]
fn an_empty_pipe_is_no_data() {
    let (ok, out) = render(TEMPLATE, "");
    assert!(ok, "{out}");
    assert!(out.contains("Hello, world"), "{out}");
    let (ok, out) = render(TEMPLATE, "  \n");
    assert!(ok, "{out}");
}

#[test]
fn piped_data_is_still_read() {
    let (ok, out) = render(TEMPLATE, r#"{"name":"Ada"}"#);
    assert!(ok, "{out}");
    assert!(out.contains("Hello, Ada"), "{out}");
}
