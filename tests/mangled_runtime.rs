//! The runtime a minified build ships has its names shortened: the
//! top-level ones, and each function's own. The JavaScript tests run
//! against it too, so a renaming that changed what the runtime does fails
//! here rather than on a page.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn the_javascript_tests_pass_against_the_shortened_runtime() {
    if !Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
    {
        eprintln!("\n  SKIPPED: `node` is not on PATH\n");
        return;
    }
    let short = webfluent::runtime::full_mangled();
    let plain = webfluent::runtime::full();
    assert!(
        short.len() < plain.len(),
        "the names were not shortened: {} of {} bytes",
        short.len(),
        plain.len()
    );
    let file = std::env::temp_dir().join(format!("wf-runtime-short-{}.js", std::process::id()));
    std::fs::write(&file, &short).unwrap();
    let tests: Vec<String> = std::fs::read_dir(repo_root().join("tests/js"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(".test.mjs"))
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let out = Command::new("node")
        .arg("--test")
        .args(&tests)
        .env("WF_RUNTIME", &file)
        .current_dir(repo_root())
        .output()
        .expect("run node --test");
    let _ = std::fs::remove_file(&file);
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "the JavaScript tests fail against the shortened runtime:\n{}",
        said.lines()
            .filter(|l| l.starts_with("not ok") || l.contains("Error") || l.starts_with("# "))
            .take(60)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
