//! A change runs what reads it in the order it was made: a branch before
//! what it drew, so a branch that leaves takes what it drew with it before
//! any of that reads the value that made it leave. And one reader that
//! throws does not keep the others from running. Run as a reader runs it,
//! the `wf` binary over a scratch project.

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str, app: &str, test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-effect-order-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build", "split": false } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    std::fs::write(dir.join("tests/order.wf"), test).unwrap();
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
    Some(text)
}

// A chip drawn while a store holds a value, reading it, and reading a
// derived made before the branch: the derived changes first, and what
// the chip reads must not run before the branch takes it away.
const APP: &str = r#"store Picked {
    state item: Any = null
}

component Chips {
    use Picked
    derived path: String = Picked.item?.file ?? ""
    derived name = path.split("/").last() ?? ""
    Row {
        if Picked.item != null {
            Button("{Picked.item.element} · {name}", aria-label: "{Picked.item.element} at {name}:{Picked.item.line}")
        }
        Text("chips")
    }
}

page P(path: "/", title: "T", description: "d") {
    Heading("x").h1
    Chips
}
"#;

#[test]
fn a_branch_that_leaves_takes_what_it_drew_before_that_reads_again() {
    let dir = scratch(
        "branch",
        APP,
        r#"test "the chip goes when what it shows does" {
    Button("Pick") { on click { Picked.item = { file: "src/pages/Deploys.wf", element: "Heading", line: 11 } } }
    Button("Clear") { on click { Picked.item = null } }
    Chips
    click "Pick"
    expect "Heading · Deploys.wf"
    click "Clear"
    expect not "Heading · Deploys.wf"
    expect "chips"
    click "Pick"
    expect "Heading · Deploys.wf"
}
"#,
    );
    let Some(out) = wf_test(&dir) else { return };
    assert!(out.contains("1 passed, 0 failed"), "{out}");
}

// Two readers of one value: the first throws, the second still runs.
const THROWS: &str = r#"store Count {
    state n = 0
}

component Both {
    use Count
    Stack {
        Text(if Count.n == 2 { "{[1][5].x}" } else { "first {Count.n}" })
        Text("second {Count.n}")
    }
}

page P(path: "/", title: "T", description: "d") {
    Heading("x").h1
    Both
}
"#;

#[test]
fn a_reader_that_throws_does_not_keep_the_next_from_running() {
    let dir = scratch(
        "throws",
        THROWS,
        r#"test "the second reader still hears the change" {
    Button("More") { on click { Count.n = Count.n + 1 } }
    Both
    click "More"
    expect "second 1"
    click "More"
    expect "second 2"
}
"#,
    );
    let Some(out) = wf_test(&dir) else { return };
    // The page reports what threw; the second reader ran all the same.
    assert!(
        out.contains("uncaught: TypeError"),
        "the first reader threw: {out}"
    );
    assert!(
        out.contains("second 2") && !out.contains("expected to find \"second 2\""),
        "{out}"
    );
}
