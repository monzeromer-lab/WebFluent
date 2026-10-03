//! The project's own JavaScript: every `.js` file under `src/`.
//!
//! A `.css` file under `src/` is part of the build with nothing declaring
//! it, and so is a `.js` file. It is a plain browser script — what a
//! `<script src>` runs, with no `export`, no `import` and no build step —
//! copied to the output as written and linked on every page before the
//! compiled code. Every classic script on a page shares one global scope,
//! which the compiled pages run in too, so what a script declares at its top
//! level is simply there for them: the compiler reads those names
//! ([`scan`](crate::project_js::scan)), puts them in scope, and the editor offers them.

pub mod jsdoc;
pub mod scan;

use std::fs;
use std::path::{Path, PathBuf};

pub use scan::Scan;

/// How a name a script declares reads to the person calling it:
/// `money(n: Number, currency?: String) -> String`, its types as the
/// checker reads its JSDoc.
pub fn signature(n: &scan::Name) -> String {
    use crate::sema::types::jsdoc_type;
    let doc = n.doc.as_deref().map(jsdoc::parse).unwrap_or_default();
    let params = |params: &[scan::Param]| {
        params
            .iter()
            .map(|p| {
                let documented = doc.param(&p.name);
                let ty = documented
                    .and_then(|d| d.ty.as_deref())
                    .map(|t| jsdoc_type(t).to_string())
                    .unwrap_or_else(|| "Any".to_string());
                let optional = p.optional || documented.is_some_and(|d| d.optional);
                format!(
                    "{}{}{}: {ty}",
                    if p.rest { "..." } else { "" },
                    p.name,
                    if optional { "?" } else { "" }
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let returns = doc
        .returns
        .as_deref()
        .map(|t| format!(" -> {}", jsdoc_type(t)))
        .unwrap_or_default();
    match &n.kind {
        scan::NameKind::Function {
            params: p,
            is_async,
        } => format!(
            "{}{}({}){returns}",
            if *is_async { "async " } else { "" },
            n.name,
            params(p)
        ),
        scan::NameKind::Class { params: p, .. } => format!("class {}({})", n.name, params(p)),
        scan::NameKind::Value => n.name.clone(),
    }
}

/// One script under `src/`, and what reading it found.
#[derive(Debug, Clone)]
pub struct Script {
    /// The file, relative to the project directory, `/`-separated.
    pub path: String,
    /// Where it is copied to and linked from, relative to the output root:
    /// `js/<path under src>`.
    pub href: String,
    pub source: String,
    pub scan: Scan,
}

/// Every `.js` file under `dir`, sorted by path so the order scripts are
/// linked in is stable. `.mjs` is a module by name and is left out; a
/// `node_modules` directory is not the project's.
pub fn find_scripts(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n != "node_modules") {
                    walk(&path, files);
                }
            } else if path.extension().is_some_and(|ext| ext == "js") {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(dir, &mut files);
    files.sort();
    files
}

/// Read every script under `src_dir`.
pub fn load(project_dir: &Path, src_dir: &Path) -> std::io::Result<Vec<Script>> {
    let mut out = Vec::new();
    for file in find_scripts(src_dir) {
        let source = fs::read_to_string(&file)?;
        out.push(from_source(project_dir, src_dir, &file, source));
    }
    Ok(out)
}

/// A script read from `source`, as if it were the file at `file`.
pub fn from_source(project_dir: &Path, src_dir: &Path, file: &Path, source: String) -> Script {
    let rel = |base: &Path| {
        file.strip_prefix(base)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/")
    };
    Script {
        path: rel(project_dir),
        href: format!("js/{}", rel(src_dir)),
        scan: scan::scan(&source),
        source,
    }
}
