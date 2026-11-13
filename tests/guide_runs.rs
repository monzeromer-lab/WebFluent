//! Every page the guide shows, built and run.
//!
//! `tests/docs_parse.rs` holds each example to the parser, the checker and
//! the linters. That said nothing about what the page does when it runs, and
//! the documentation audit found programs that passed every check and threw
//! in the browser: an action whose parameters were read as signals, a head
//! tag that read a `derived` before it existed, `navigator` compiled as a
//! signal. So each whole-page example is built here, and
//! `tests/js/guide.test.mjs` mounts it against the fake DOM, clicks every
//! button, types into every field, and fails on anything that throws.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The `wf` blocks of a Markdown file that are whole programs with a page:
/// `(line, source)`.
fn page_examples(markdown: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (ix, line) in markdown.lines().enumerate() {
        match &mut current {
            Some((_, body)) => {
                if line.trim_end() == "```" {
                    let (at, body) = current.take().unwrap();
                    out.push((at, body));
                } else {
                    body.push_str(line);
                    body.push('\n');
                }
            }
            None if crate::common::fence(line)
                .and_then(Result::ok)
                .is_some_and(|f| f.lang == "wf" && f.is_sample()) =>
            {
                current = Some((ix + 1, String::new()));
            }
            None => {}
        }
    }
    out.into_iter()
        .filter(|(_, src)| {
            let whole = src
                .lines()
                .filter(|l| {
                    !l.trim().is_empty()
                        && !l.starts_with(char::is_whitespace)
                        && !l.starts_with('}')
                })
                .all(|l| {
                    let word = l.split(|c: char| !c.is_alphanumeric()).next().unwrap_or("");
                    matches!(
                        word,
                        "page"
                            | "component"
                            | "store"
                            | "theme"
                            | "app"
                            | "type"
                            | "enum"
                            | "api"
                            | "const"
                            | "animation"
                            | "test"
                            | "data"
                            | "image"
                    ) || l.starts_with("//")
                });
            whole
                && src.lines().any(|l| l.starts_with("page "))
                && !src.contains('…')
                // Paper, a canvas or an editor a library from a CDN draws
                // into, a file on disk, or a deliberate failure: nothing a
                // fake DOM can stand for.
                && !src.contains("Document(")
                && !src.contains("Host(tag: \"canvas\"")
                && !src.contains("startEditor(")
                && !src.contains("Presentation")
                && !src.contains("data ")
                && !src.contains("image ")
                && !src.contains(" from \"")
                // A tutorial project's file, named on its first line: built
                // with its project by `tests/tutorials.rs`.
                && !src.starts_with("// src/")
        })
        .collect()
}

/// Every example, built into `target/e2e/guide/<chapter>-<line>/`. Returns
/// the directory names that built, and the failures.
fn build_all() -> (Vec<String>, Vec<String>) {
    let out_root = repo_root().join("target/e2e/guide");
    let _ = std::fs::remove_dir_all(&out_root);
    // Every chapter and every recipe: a code's page under `errors/` shows a
    // program that fails on purpose.
    let mut names: Vec<String> = crate::common::guide_pages()
        .into_iter()
        .map(|(path, _)| path)
        .filter(|n| n.contains('/') && !n.starts_with("errors/") && !n.ends_with("/index.md"))
        // A template reads the JSON it is rendered with, not a browser.
        .filter(|n| !n.contains("render-a-template") && !n.contains("render-from-"))
        .collect();
    names.sort();
    let mut built = Vec::new();
    let mut failures = Vec::new();
    for name in names {
        let markdown = std::fs::read_to_string(repo_root().join("md-docs").join(&name)).unwrap();
        let scripts = chapter_scripts(&markdown);
        for (line, src) in page_examples(&markdown) {
            let dir_name = format!("{}-{line}", name[..name.len() - 3].replace('/', "--"));
            let dir = out_root.join(&dir_name);
            std::fs::create_dir_all(dir.join("src")).unwrap();
            let i18n = if src.contains("t(\"") || src.contains("setLocale") {
                std::fs::create_dir_all(dir.join("src/translations")).unwrap();
                std::fs::write(dir.join("src/translations/en.json"), "{}").unwrap();
                std::fs::write(dir.join("src/translations/ar.json"), "{}").unwrap();
                r#", "i18n": { "default_locale": "en", "locales": ["en", "ar"] }"#
            } else {
                ""
            };
            // A project with two themes names one, as the guide's configs do.
            let themes: Vec<&str> = src
                .lines()
                .filter_map(|l| l.strip_prefix("theme "))
                .filter_map(|l| l.split(|c: char| !c.is_alphanumeric()).next())
                .collect();
            let theme = match themes.as_slice() {
                [first, second, ..] => {
                    format!(r#", "theme": {{ "name": "{first}", "dark": "{second}" }}"#)
                }
                _ => String::new(),
            };
            // An example is a page of a bigger site: a link it shows to a
            // page it does not, and the translations it reads, are the
            // site's, so `R01` and the `I` family are off here.
            std::fs::write(
                dir.join("webfluent.app.json"),
                format!(
                    r#"{{ "name": "guide"{i18n}{theme}, "lints": {{ "R01": "off", "I": "off" }} }}"#
                ),
            )
            .unwrap();
            std::fs::write(dir.join("src/App.wf"), &src).unwrap();
            // The scripts the chapter shows beside it, as its project has them.
            for (path, text) in &scripts {
                let at = dir.join(path);
                std::fs::create_dir_all(at.parent().unwrap()).unwrap();
                std::fs::write(at, text).unwrap();
            }
            let out = Command::new(env!("CARGO_BIN_EXE_wf"))
                .args(["build", "-d"])
                .arg(&dir)
                .output()
                .expect("run wf build");
            if out.status.success() {
                built.push(dir_name);
            } else {
                failures.push(format!(
                    "md-docs/{name}:{line} does not build:\n{}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }
    }
    (built, failures)
}

/// The scripts a chapter shows: each ```` ```js ```` block whose first line
/// is `// src/<file>.js`, as `(src/<file>.js, its text)`.
fn chapter_scripts(markdown: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut lines = markdown.lines();
    while let Some(line) = lines.next() {
        if line.trim_end() != "```js" {
            continue;
        }
        let body: Vec<&str> = lines
            .by_ref()
            .take_while(|l| l.trim_end() != "```")
            .collect();
        if let Some(path) = body
            .first()
            .and_then(|l| l.strip_prefix("// "))
            .filter(|p| p.starts_with("src/"))
        {
            out.push((path.trim().to_string(), format!("{}\n", body.join("\n"))));
        }
    }
    out
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn every_page_the_guide_shows_builds_and_runs() {
    let (built, failures) = build_all();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    assert!(
        built.len() > 60,
        "found only {} runnable examples",
        built.len()
    );
    std::fs::write(
        repo_root().join("target/e2e/guide/examples.json"),
        serde_json::to_string(&built).unwrap(),
    )
    .unwrap();
    if !node_available() {
        eprintln!("\n  SKIPPED running the guide's pages: `node` is not on PATH\n");
        return;
    }
    let out = Command::new("node")
        .args(["--test", "tests/js/guide.test.mjs"])
        .current_dir(repo_root())
        .output()
        .expect("run node --test");
    assert!(
        out.status.success(),
        "a page the guide shows throws when it runs:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = Path::new("");
}
