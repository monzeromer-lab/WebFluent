//! `wf types --holes` over the docs site and every example: in 7.0 a
//! program that checks has no place a value's type is unknown, and these
//! are held to it — a hole that comes back fails here, by place.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use webfluent::sema::types::{HoleKind, holes_of};

fn projects(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() && !matches!(name, "build" | "node_modules" | ".wf-cache") {
                if path.join("webfluent.app.json").is_file() && path.join("src").is_dir() {
                    out.push(path.clone());
                }
                walk(&path, out);
            }
        }
    }
    let mut out = vec![root.join("site")];
    walk(&root.join("examples"), &mut out);
    out
}

#[test]
fn the_site_and_every_example_have_no_place_a_type_is_unknown() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for project in projects(root) {
        let holes = holes_of(&project).unwrap_or_else(|e| panic!("{}: {e}", project.display()));
        let name = project
            .strip_prefix(root)
            .expect("under the repository")
            .to_string_lossy()
            .replace('\\', "/");
        if !holes.is_empty() {
            found.insert(
                name,
                holes
                    .iter()
                    .map(|h| {
                        format!(
                            "{}:{}:{} {} {}",
                            h.file,
                            h.line,
                            h.column,
                            h.kind.name(),
                            h.what
                        )
                    })
                    .collect(),
            );
        }
    }
    assert!(
        found.is_empty(),
        "7.0 has no place a type is unknown, and these have some — give each the type it holds:\n{found:#?}"
    );
    // Every kind is still one the checker can name, so a hole that comes
    // back is said in words.
    assert!(HoleKind::ALL.iter().all(|k| !k.name().is_empty()));
}
