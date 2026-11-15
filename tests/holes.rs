//! `wf types --holes` over the docs site and every example: how many
//! places of each kind leave a value's type unknown. The count is the
//! measure of how much of the language is not yet typed; a change that
//! moves it shows here first, project by project.
//!
//!     UPDATE_GOLDEN=1 cargo test --test it -- holes::

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
fn the_places_a_type_is_unknown_are_counted() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut counts: BTreeMap<String, BTreeMap<&str, usize>> = BTreeMap::new();
    let mut total: BTreeMap<&str, usize> = BTreeMap::new();
    for project in projects(root) {
        let holes = holes_of(&project).unwrap_or_else(|e| panic!("{}: {e}", project.display()));
        let name = project
            .strip_prefix(root)
            .expect("under the repository")
            .to_string_lossy()
            .replace('\\', "/");
        let mut kinds = BTreeMap::new();
        for kind in HoleKind::ALL {
            let n = holes.iter().filter(|h| h.kind == kind).count();
            if n > 0 {
                kinds.insert(kind.name(), n);
                *total.entry(kind.name()).or_default() += n;
            }
        }
        counts.insert(name, kinds);
    }
    let mut actual = serde_json::to_string_pretty(&serde_json::json!({
        "total": total,
        "projects": counts,
    }))
    .expect("serialises");
    actual.push('\n');
    let path = root.join("tests/holes.json");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &actual).expect("writes the snapshot");
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        expected == actual,
        "the count of places a type is unknown changed:\n{actual}\nIf the change is meant, run \
         `UPDATE_GOLDEN=1 cargo test --test it -- holes::` and commit tests/holes.json"
    );
}
