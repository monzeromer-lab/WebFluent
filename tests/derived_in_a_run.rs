//! A derived read while a change is still running is the new value. An
//! effect that answers a change by calling an action (the effect saw a
//! request; the action writes a state and reads a derived of it) runs
//! inside the change's run, where the derived's own update waits its turn:
//! it must not be read as it was before the write. The effect ran again
//! once the derived caught up, which hid it, unless the action refused the
//! second call. Run on the runtime the compiler ships, under Node.

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn a_derived_read_in_a_running_change_is_the_new_value() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("no node: skipped");
        return;
    }
    let script = format!(
        r#"globalThis.window = globalThis; globalThis.document = undefined;
{runtime}
const reads = [];
const Box = WF.store("Box", () => ({{
  state: {{ typed: "dep", taken: 0 }},
  derived: {{ mode: (s) => (s.typed.startsWith(">") ? "commands" : "all") }},
  actions: {{
    answer: (s, n, p) => {{ if (n === s.taken) return; s.taken = n; s.typed = p; reads.push(s.mode); }},
  }},
}}), {{ scope: "app" }});
const asks = WF.signal(0);
const before = Box.mode;
WF.effect(() => {{ if (asks() > 0) Box.answer(asks(), ">"); }});
asks.set(1);
console.log(JSON.stringify({{ before, reads, now: Box.mode }}));
"#,
        runtime = webfluent::runtime::full()
    );
    // The runtime is too long for a command line: it goes in on stdin.
    let mut node = Command::new("node")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    node.stdin
        .take()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    let out = node.wait_with_output().unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{}{said}",
        String::from_utf8_lossy(&out.stderr)
    );
    // One read, the new value: an action that refuses a request it took
    // already (as a palette's open does) is not run again to correct it.
    assert_eq!(
        said.trim(),
        r#"{"before":"all","reads":["commands"],"now":"commands"}"#
    );
}
