//! Every source file the repository ships is in the formatter's canonical
//! spelling, and formatting is a fixed point: `wf fmt --check` is clean
//! on the documentation site and every fixture.

use std::path::Path;

use webfluent::fmt::format_source;

fn wf_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            wf_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "wf" || e == "wfx") {
            out.push(path);
        }
    }
}

#[test]
fn the_site_and_every_fixture_are_formatted() {
    let mut files = Vec::new();
    wf_files(Path::new("site/src"), &mut files);
    wf_files(Path::new("tests/fixtures"), &mut files);
    assert!(files.len() > 20, "found {} files", files.len());
    let mut unformatted = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let name = file.to_string_lossy();
        let out = format_source(&text, &name).unwrap_or_else(|e| panic!("{name}: {e}"));
        if out != text {
            unformatted.push(name.to_string());
        }
        assert_eq!(
            format_source(&out, &name).unwrap(),
            out,
            "{name}: formatting is not a fixed point"
        );
    }
    assert!(unformatted.is_empty(), "not formatted: {unformatted:?}");
}
