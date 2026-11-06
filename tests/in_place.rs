//! An in-place patch agrees with the next full build, for an edit to every
//! element of every file of a real project: a style set and a literal text
//! changed, each read back, compared and generated both ways.

use std::path::{Path, PathBuf};

use webfluent::edit::{EditOp, apply_edits_to};
use webfluent::parse_source;
use webfluent::parser::ast::{Declaration, Program};
use webfluent::patch::{InPlace, agrees, in_place};

fn wf_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            wf_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "wf") {
            out.push(path);
        }
    }
}

struct Tally {
    patched: usize,
    rebuilt: usize,
    text: usize,
    style: usize,
    /// What did not agree: the component, the edit, and how.
    wrong: Vec<String>,
}

/// Every file of a project, parsed; a program of all of them but `edited`,
/// with that one as given.
fn merged(files: &[(PathBuf, String)], edited: Option<(usize, &str)>) -> Option<Program> {
    let mut declarations: Vec<Declaration> = Vec::new();
    for (i, (path, text)) in files.iter().enumerate() {
        let source = match edited {
            Some((at, new)) if at == i => new,
            _ => text.as_str(),
        };
        let parsed = parse_source(source, &path.to_string_lossy()).ok()?;
        declarations.extend(parsed.declarations);
    }
    Some(Program { declarations })
}

fn exercise(project: &Path, tally: &mut Tally) {
    let mut paths = Vec::new();
    wf_files(&project.join("src"), &mut paths);
    let files: Vec<(PathBuf, String)> = paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).unwrap();
            (p, text)
        })
        .collect();
    let Some(before) = merged(&files, None) else {
        return; // A fixture of a file that is meant not to read.
    };

    for (index, (path, text)) in files.iter().enumerate() {
        let file = path.to_string_lossy().to_string();
        let own = parse_source(text, &file).unwrap();
        let mut nodes = Vec::new();
        webfluent::codegen::node_id::visit_nodes(&own, &mut |ui, id, _| {
            nodes.push((id.to_string(), webfluent::print::component(&ui.component)));
        });
        for (id, component) in nodes {
            if component == "Children" || component == "_StyleBlock" {
                continue;
            }
            let edits = [
                EditOp::SetStyle {
                    node: id.clone(),
                    prop: "outline-offset".to_string(),
                    value: "3px".to_string(),
                },
                EditOp::SetText {
                    node: id.clone(),
                    value: "A new <text> & \"quotes\"".to_string(),
                },
            ];
            for op in edits {
                let is_text = matches!(op, EditOp::SetText { .. });
                let Ok(new) = apply_edits_to(text, &file, std::slice::from_ref(&op)) else {
                    continue;
                };
                let Some(after) = merged(&files, Some((index, &new))) else {
                    continue;
                };
                match in_place(&before, &after) {
                    InPlace::Patch(patch) => {
                        if let Err(why) = agrees(&before, &after, &patch) {
                            let kind = if is_text { "text" } else { "style" };
                            tally
                                .wrong
                                .push(format!("{component} {kind} ({file} {id}): {why}"));
                        }
                        tally.patched += 1;
                        if is_text {
                            tally.text += 1;
                        } else {
                            tally.style += 1;
                        }
                    }
                    InPlace::Rebuild { .. } | InPlace::Same => tally.rebuilt += 1,
                }
            }
        }
    }
}

#[test]
fn a_patch_agrees_with_the_build_on_every_element_of_every_fixture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut tally = Tally {
        patched: 0,
        rebuilt: 0,
        text: 0,
        style: 0,
        wrong: Vec::new(),
    };
    let mut projects: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    projects.sort();
    let mut read = 0;
    for project in projects {
        if project.join("src").is_dir() {
            read += 1;
            exercise(&project, &mut tally);
        }
    }
    assert!(read > 5, "only {read} fixtures");
    assert!(
        tally.wrong.is_empty(),
        "{} did not agree:\n{}",
        tally.wrong.len(),
        tally.wrong.join("\n")
    );
    eprintln!(
        "{read} fixtures: {} patched ({} style, {} text), {} rebuilt",
        tally.patched, tally.style, tally.text, tally.rebuilt
    );
}

#[test]
fn a_patch_agrees_with_the_build_on_every_element_of_the_tutorials() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/tutorials");
    let mut tally = Tally {
        patched: 0,
        rebuilt: 0,
        text: 0,
        style: 0,
        wrong: Vec::new(),
    };
    for name in ["easy-todos", "medium-field-notes", "hard-pocket"] {
        exercise(&root.join(name), &mut tally);
    }
    assert!(
        tally.wrong.is_empty(),
        "{} did not agree:\n{}",
        tally.wrong.len(),
        tally.wrong.join("\n")
    );
    assert!(tally.style > 100, "only {} styles patched", tally.style);
    assert!(tally.text > 30, "only {} texts patched", tally.text);
    eprintln!(
        "{} patched ({} style, {} text), {} rebuilt",
        tally.patched, tally.style, tally.text, tally.rebuilt
    );
}
