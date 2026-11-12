//! The crate's modules sit in layers, and a module may use only its own
//! layer and the ones below it: base, diagnostics, syntax, registry,
//! project, check, web, paged, template, build, tools — in that order. This
//! test reads every `crate::…` path outside the tests and fails on one that
//! reaches up, naming the file and line. A cycle between two modules is
//! always one reach upward, so the layers also keep the graph acyclic.
//!
//! Today's reaches upward are listed in `EXCEPTIONS`, each with what it is
//! waiting on; a change that removes one removes its line, and a line that
//! matches nothing any more fails too, so the list only ever shrinks.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The layers, bottom first.
const ORDER: &[&str] = &[
    "base",
    "diagnostics",
    "syntax",
    "registry",
    "project",
    "check",
    "web",
    "paged",
    "template",
    "build",
    "tools",
];

/// Each module's layer, by its path in the crate; the longest prefix wins,
/// so `diagnostics::check` sits apart from the rest of `diagnostics`.
const GROUPS: &[(&str, &str)] = &[
    ("base", "base"),
    ("project", "project"),
    ("check", "check"),
    ("web", "web"),
    ("tools", "tools"),
    ("vfs", "base"),
    ("outfs", "base"),
    ("platform", "base"),
    ("span", "base"),
    ("text", "base"),
    ("error", "diagnostics"),
    ("diagnostics", "diagnostics"),
    ("diagnostics::allow", "check"),
    ("diagnostics::check", "template"),
    ("lexer", "syntax"),
    ("parser", "syntax"),
    ("syntax", "syntax"),
    ("layout", "syntax"),
    ("print", "syntax"),
    ("reprint", "syntax"),
    ("fmt", "syntax"),
    ("project_js::scan", "syntax"),
    ("codegen::node_id", "syntax"),
    ("registry", "registry"),
    ("registry::json", "tools"),
    ("registry::schema", "tools"),
    ("codegen::style_tokens", "registry"),
    ("config", "project"),
    ("themes", "project"),
    ("themes::view", "tools"),
    ("i18n", "project"),
    ("project_js", "project"),
    ("project_js::jsdoc", "check"),
    ("data", "project"),
    ("media", "project"),
    ("openapi", "project"),
    ("sources", "project"),
    ("codegen::url", "project"),
    ("codegen::format", "project"),
    ("codegen::sanitize", "project"),
    ("codegen::static_eval", "project"),
    ("codegen::builtin", "project"),
    ("codegen::scoped_css", "project"),
    ("sema", "check"),
    ("scope", "check"),
    ("linter", "check"),
    ("validation", "check"),
    ("edit", "check"),
    ("codegen", "web"),
    ("runtime", "web"),
    ("motion", "web"),
    ("codegen::offline", "build"),
    ("paged", "paged"),
    ("style_value", "paged"),
    ("template", "template"),
    ("build", "build"),
    ("assets", "build"),
    ("migrate", "tools"),
    ("testing", "tools"),
    ("verify", "tools"),
    ("browser", "tools"),
    ("audit", "tools"),
    ("studio", "tools"),
    ("agent", "tools"),
    ("patch", "tools"),
    ("init", "tools"),
    ("generate", "tools"),
    ("gallery", "tools"),
    ("memory", "tools"),
    ("commands", "tools"),
];

/// Today's reaches upward: the file, the path it reaches (a prefix), and
/// what it is waiting on.
const EXCEPTIONS: &[(&str, &str, &str)] = &[];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn group_of(path: &str) -> Option<&'static str> {
    let parts: Vec<&str> = path.split("::").collect();
    (1..=parts.len()).rev().find_map(|n| {
        let key = parts[..n].join("::");
        GROUPS.iter().find(|(m, _)| *m == key).map(|(_, g)| *g)
    })
}

fn rank(group: &str) -> usize {
    ORDER
        .iter()
        .position(|g| *g == group)
        .unwrap_or_else(|| panic!("{group} is not a layer"))
}

/// `src/codegen/js.rs` → `codegen::js`; `src/build/mod.rs` → `build`.
fn module_of(file: &Path) -> String {
    let rel = file
        .strip_prefix(root().join("src"))
        .expect("under src/")
        .with_extension("");
    let mut parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.last().is_some_and(|p| p == "mod") {
        parts.pop();
    }
    parts.join("::")
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // The CLI is the top of the stack and may use anything.
            if path.file_name().is_some_and(|n| n == "cli") {
                continue;
            }
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The text of a file up to its test module, which may reach anywhere.
fn without_tests(text: &str) -> &str {
    let mut offset = 0;
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_end() == "#[cfg(test)]"
            && lines
                .get(i + 1)
                .is_some_and(|next| next.trim_start().starts_with("mod "))
        {
            return &text[..offset];
        }
        offset += line.len();
    }
    text
}

/// Every `crate::a::b` path in a line, as `a::b`.
fn crate_paths(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("crate::") {
        let after = &rest[at + "crate::".len()..];
        let path: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ':')
            .collect();
        let consumed = path.len().max(1).min(after.len());
        let path = path.trim_end_matches(':').to_string();
        if path.starts_with(|c: char| c.is_ascii_lowercase()) {
            out.push(path);
        }
        rest = &after[consumed..];
    }
    out
}

#[test]
fn no_module_reaches_a_layer_above_its_own() {
    let mut files = Vec::new();
    sources(&root().join("src"), &mut files);
    files.sort();

    let mut reaches = Vec::new();
    let mut used = vec![false; EXCEPTIONS.len()];
    for file in &files {
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == "lib.rs" || name == "main.rs" {
            continue;
        }
        let module = module_of(file);
        let Some(from) = group_of(&module) else {
            panic!("{module} has no layer; add it to GROUPS");
        };
        let text = std::fs::read_to_string(file).expect("a source file");
        let rel = file
            .strip_prefix(root())
            .expect("in the repository")
            .to_string_lossy()
            .replace('\\', "/");
        for (n, line) in without_tests(&text).lines().enumerate() {
            let trimmed = line.trim_start();
            // A comment, or a re-export that keeps an old public path where
            // users find it: neither is a dependency.
            if trimmed.starts_with("//") || trimmed.starts_with("pub use ") {
                continue;
            }
            for path in crate_paths(line) {
                let Some(to) = group_of(&path) else { continue };
                if to == from || rank(to) < rank(from) {
                    continue;
                }
                let excepted = EXCEPTIONS
                    .iter()
                    .position(|(f, p, _)| *f == rel && path.starts_with(p));
                match excepted {
                    Some(i) => used[i] = true,
                    None => reaches.push(format!("{rel}:{}  {from} → {to}  crate::{path}", n + 1)),
                }
            }
        }
    }

    let stale: Vec<String> = EXCEPTIONS
        .iter()
        .zip(&used)
        .filter(|(_, used)| !**used)
        .map(|((f, p, _), _)| format!("{f}  crate::{p}"))
        .collect();

    let mut by_edge: BTreeMap<String, usize> = BTreeMap::new();
    for r in &reaches {
        let edge = r.split("  ").nth(1).unwrap_or_default().to_string();
        *by_edge.entry(edge).or_default() += 1;
    }
    assert!(
        reaches.is_empty() && stale.is_empty(),
        "{}{}",
        if reaches.is_empty() {
            String::new()
        } else {
            format!(
                "a module reaches a layer above its own ({}):\n  {}\n\n",
                by_edge
                    .iter()
                    .map(|(e, n)| format!("{e} ×{n}"))
                    .collect::<Vec<_>>()
                    .join(", "),
                reaches.join("\n  ")
            )
        },
        if stale.is_empty() {
            String::new()
        } else {
            format!(
                "these exceptions match nothing any more; remove them:\n  {}",
                stale.join("\n  ")
            )
        }
    );
}
