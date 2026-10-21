//! The node map both ways, over every project in the repository (and,
//! with `WF_NODES_PROJECTS`, more — Halyard): the program as the editor
//! parses it and as the code generators read it give every element the
//! same id at the same place; the caret at an element's first and last
//! byte finds that element; every element sits inside its parent; and the
//! Layers tree holds every element exactly once, in source order.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use webfluent::codegen::node_id::{Layer, NodeMap, build_node_map_with, layers};

fn projects() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = vec![root.join("site")];
    for base in [
        "tests/fixtures",
        "examples/tutorials",
        "examples/decks",
        "examples/documents",
    ] {
        if let Ok(entries) = std::fs::read_dir(root.join(base)) {
            let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            dirs.sort();
            out.extend(
                dirs.into_iter()
                    .filter(|d| d.join("webfluent.app.json").is_file()),
            );
        }
    }
    if let Ok(more) = std::env::var("WF_NODES_PROJECTS") {
        out.extend(more.split(':').filter(|d| !d.is_empty()).map(PathBuf::from));
    }
    out
}

fn walk_layers<'a>(layers: &'a [Layer], out: &mut Vec<&'a Layer>) {
    for l in layers {
        out.push(l);
        walk_layers(&l.children, out);
    }
}

fn siblings_in_order(layers: &[Layer], wrong: &mut Vec<String>) {
    for pair in layers.windows(2) {
        if pair[0].file == pair[1].file && pair[1].start < pair[0].start && pair[1].start > 0 {
            wrong.push(format!(
                "{} before {} in Layers",
                pair[1].label, pair[0].label
            ));
        }
    }
    for l in layers {
        siblings_in_order(&l.children, wrong);
    }
}

#[test]
fn every_element_is_found_both_ways() {
    let mut elements = 0;
    let mut wrong: Vec<String> = Vec::new();
    for dir in projects() {
        let Ok((program, files)) = webfluent::build::read_project(&dir) else {
            continue;
        };
        let file_of = |i: usize| files.get(i).cloned();
        let parsed = build_node_map_with(&program, &file_of);
        let lowered = build_node_map_with(&webfluent::sema::lower(program.clone()), &file_of);
        let at = |m: &NodeMap| -> HashMap<String, (Option<String>, u32, u32)> {
            m.iter()
                .map(|(id, n)| (id.clone(), (n.file.clone(), n.start, n.end)))
                .collect()
        };
        if at(&parsed) != at(&lowered) {
            wrong.push(format!(
                "{}: the lowered program numbers its elements differently",
                dir.display()
            ));
        }
        for (id, n) in parsed.iter() {
            if n.start >= n.end {
                continue;
            }
            elements += 1;
            let file = n.file.clone().unwrap_or_default();
            for offset in [n.start as usize, n.end as usize - 1] {
                match parsed.at(&file, offset) {
                    Some(found) if &found.path == id => {}
                    other => wrong.push(format!(
                        "{}: {file}:{} the caret at {offset} finds {:?}, not {id}",
                        dir.display(),
                        n.line,
                        other.map(|o| &o.path)
                    )),
                }
            }
            if let Some(parent) = n.parent.as_ref().and_then(|p| parsed.info(p))
                && parent.start < parent.end
                && !(parent.start <= n.start && n.end <= parent.end && parent.file == n.file)
            {
                wrong.push(format!(
                    "{}: {id} is outside its parent {}",
                    dir.display(),
                    parent.path
                ));
            }
        }
        // Layers: every element once, siblings in order.
        let tree = layers(&program, &file_of);
        let mut all = Vec::new();
        walk_layers(&tree, &mut all);
        let mut seen = HashSet::new();
        for l in &all {
            if let Some(id) = &l.id
                && !seen.insert(id.clone())
            {
                wrong.push(format!("{}: {id} is in Layers twice", dir.display()));
            }
        }
        for (id, n) in parsed.iter() {
            if n.start < n.end
                && n.element != "Children"
                && !n.element.starts_with("__")
                && !seen.contains(id)
            {
                wrong.push(format!(
                    "{}: {id} ({}) is not in Layers",
                    dir.display(),
                    n.element
                ));
            }
        }
        siblings_in_order(&tree, &mut wrong);
    }
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(elements > 8_000, "{elements} elements");
}

/// A studio build stamps every element with the id the map gives it, in the
/// pre-rendered pages and the bundle, and says where each is written.
#[test]
fn a_studio_build_stamps_what_the_map_says() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!("wf-nodes-site-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let copy = |from: &Path, to: &Path| {
        fn walk(from: &Path, to: &Path) {
            std::fs::create_dir_all(to).unwrap();
            for e in std::fs::read_dir(from).unwrap().flatten() {
                let p = e.path();
                let name = e.file_name();
                if p.is_dir() {
                    if name != "build" {
                        walk(&p, &to.join(&name));
                    }
                } else {
                    std::fs::copy(&p, to.join(&name)).unwrap();
                }
            }
        }
        walk(from, to);
    };
    copy(&root.join("tests/fixtures/dashboard"), &dir);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "--studio", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let output = dir.join("build");
    let nodes: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(output.join("nodes.json")).unwrap()).unwrap();
    let known: HashSet<String> = nodes["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["path"].as_str().unwrap().to_string())
        .collect();
    assert!(known.len() > 20, "{} nodes", known.len());
    for n in nodes["nodes"].as_array().unwrap() {
        assert!(
            n["file"].as_str().is_some_and(|f| f.starts_with("src/")),
            "{n}"
        );
    }
    // Every id stamped anywhere is one the map has.
    let stamp = regex::Regex::new(r#"data-wf-node"?\s*[:=]\s*"([^"']+)""#).unwrap();
    let mut stamped = HashSet::new();
    let mut stack = vec![output.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "html" || x == "js") {
                let text = std::fs::read_to_string(&p).unwrap();
                for c in stamp.captures_iter(&text) {
                    let id = c[1].to_string();
                    if !id.contains('+') {
                        stamped.insert(id);
                    }
                }
            }
        }
    }
    let unknown: Vec<&String> = stamped.iter().filter(|id| !known.contains(*id)).collect();
    assert!(
        unknown.is_empty(),
        "stamped ids the map does not have: {unknown:?}"
    );
    assert!(stamped.len() > 10, "{} ids stamped", stamped.len());
    // A site build after it stamps nothing, and leaves no map behind.
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!output.join("nodes.json").exists());
    let html = std::fs::read_to_string(output.join("index.html")).unwrap();
    assert!(!html.contains("data-wf-node=\""));
    let _ = std::fs::remove_dir_all(&dir);
}
