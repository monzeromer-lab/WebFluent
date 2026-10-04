//! What the binary says when it is asked, rather than run by an editor.

use std::process::Command;

fn run(arg: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_wf-lsp"))
        .arg(arg)
        .output()
        .expect("wf-lsp runs");
    assert!(out.status.success(), "{arg} exits 0");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn version_names_the_crate_version() {
    let expected = format!("wf-lsp {}\n", env!("CARGO_PKG_VERSION"));
    assert_eq!(run("--version"), expected);
    assert_eq!(run("-V"), expected);
}

#[test]
fn help_says_what_it_is_and_how_it_is_run() {
    let help = run("--help");
    assert!(help.starts_with(&format!("wf-lsp {}", env!("CARGO_PKG_VERSION"))));
    assert!(help.contains("--stdio") && help.contains("--version"));
    assert_eq!(run("-h"), help);
}
