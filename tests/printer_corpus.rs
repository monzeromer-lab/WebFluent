//! The printer over every project in the repository — the examples, the
//! site, the fixtures, the grammars' tests — and, when `WF_PRINT_CORPUS`
//! names more folders (Halyard's `src/`, say), those too: every statement
//! of every file, printed and put back in its own place, parses to the
//! same tree.

use std::path::{Path, PathBuf};

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if p.is_dir() {
            if !matches!(
                name.as_str(),
                "build" | "node_modules" | "target" | ".wf-cache" | ".git"
            ) {
                sources(&p, out);
            }
        } else if name.ends_with(".wf") || name.ends_with(".wfx") {
            out.push(p);
        }
    }
}

#[test]
fn every_statement_prints_back_as_itself() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<PathBuf> = Vec::new();
    // `editors/zed/` holds copies of this repository; the grammars' own
    // tests are what is new there.
    for dir in [
        "examples",
        "site",
        "tests",
        "editors/tree-sitter-webfluent",
        "editors/tree-sitter-webfluentx",
    ] {
        sources(&root.join(dir), &mut files);
    }
    if let Ok(more) = std::env::var("WF_PRINT_CORPUS") {
        for dir in more.split(':').filter(|d| !d.is_empty()) {
            sources(Path::new(dir), &mut files);
        }
    }
    // A debug build reads every fourth file; CI runs the whole of it in
    // release.
    if cfg!(debug_assertions) {
        files = files.into_iter().step_by(4).collect();
    }
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let results: Vec<(usize, Vec<String>, Vec<PathBuf>)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|k| {
                let files = &files;
                s.spawn(move || {
                    let (mut proven, mut failures, mut unparsed) = (0, Vec::new(), Vec::new());
                    for f in files.iter().skip(k).step_by(threads) {
                        let source = std::fs::read_to_string(f).unwrap();
                        match webfluent::print::prove(&source, &f.to_string_lossy()) {
                            Ok(n) => proven += n,
                            // A file written not to parse — a fixture of an
                            // error, the old grammar — has nothing to print.
                            Err(wrong) if wrong.len() == 1 && wrong[0].line == 0 => {
                                unparsed.push(f.clone())
                            }
                            Err(wrong) => {
                                for w in wrong {
                                    failures.push(format!(
                                        "{}:{} — {}\n  written: {}\n  printed: {}",
                                        f.strip_prefix(root).unwrap_or(f).display(),
                                        w.line,
                                        w.why,
                                        w.written.lines().next().unwrap_or(""),
                                        w.printed.lines().next().unwrap_or("")
                                    ));
                                }
                            }
                        }
                    }
                    (proven, failures, unparsed)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let proven: usize = results.iter().map(|r| r.0).sum();
    let failures: Vec<String> = results.iter().flat_map(|r| r.1.clone()).collect();
    let unparsed: Vec<PathBuf> = results.iter().flat_map(|r| r.2.clone()).collect();
    eprintln!(
        "{} files ({} that do not parse), {proven} statements proven, {} not",
        files.len(),
        unparsed.len(),
        failures.len()
    );
    for f in &unparsed {
        eprintln!(
            "  does not parse: {}",
            f.strip_prefix(root).unwrap_or(f).display()
        );
    }
    assert!(
        failures.is_empty(),
        "{} statements did not print back as themselves:\n{}",
        failures.len(),
        failures
            .iter()
            .take(60)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        proven
            > if cfg!(debug_assertions) {
                3_000
            } else {
                15_000
            },
        "{proven} statements"
    );
}
