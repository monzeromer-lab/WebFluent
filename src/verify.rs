//! `wf verify` — every page of a project, in a real browser.
//!
//! A build says what the compiler wrote. This says what a browser does
//! with it: whether each route loads, whether anything threw, whether
//! every file it asked for arrived, how long it took to paint, and which
//! of the project's components were actually drawn.
//!
//! It is the half of a build that cannot be checked by reading the
//! output, and it is the reason a component can be in the registry, in
//! the tests and in the documentation and still be broken in a page.
//! `wf verify` visits the pages in Chrome; the mobile editor in its
//! WebView. Both read a page's report with [`PAGE_REPORT`] and judge it
//! here.

use std::collections::BTreeSet;
use std::path::Path;

use crate::error::Result;

// For the mobile editor's visits; `wf verify` reads them in the driver.
#[allow(unused_imports)]
pub use crate::browser::chrome::{PAGE_REPORT, Visit, read_report};

/// Hold a visit to what a page must do: show some text (a 404 page may
/// not), and paint within `budget_ms` when there is one.
pub fn judge(visit: &mut Visit, route: &str, budget_ms: Option<u64>) {
    if visit.text == 0 && !route.contains("404") {
        visit.errors.push("the page rendered no text".to_string());
    }
    if let Some(budget) = budget_ms
        && visit.first_contentful_paint > budget as i64
    {
        visit.errors.push(format!(
            "first paint at {}ms, over the {budget}ms this build allows",
            visit.first_contentful_paint
        ));
    }
}

/// Every route the build serves: the files it wrote, and the pages the
/// project declares that a single-page build serves from one shell.
pub fn routes(output_dir: &Path, project_dir: &Path, base_path: &str) -> Result<Vec<String>> {
    let mut found: BTreeSet<String> = BTreeSet::new();
    let mut walk = vec![output_dir.to_path_buf()];
    while let Some(dir) = walk.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk.push(path);
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) != Some("index.html") {
                continue;
            }
            let route = path
                .parent()
                .and_then(|p| p.strip_prefix(output_dir).ok())
                .map(|p| format!("/{}", p.display()))
                .unwrap_or_else(|| "/".to_string());
            found.insert(route);
        }
    }
    // A single-page build writes one shell, so its routes are what the
    // program says they are.
    if let Ok((program, _)) = crate::build::read_project(project_dir) {
        for decl in &program.declarations {
            if let crate::parser::ast::Declaration::Page(page) = decl
                && !page.path.contains(':')
                && page.path != "*"
                && !page.path.is_empty()
            {
                found.insert(page.path.clone());
            }
        }
    }
    let _ = base_path;
    Ok(found.into_iter().collect())
}

/// The built-ins no page drew.
///
/// Every built-in renders a root with a class of its own, so the classes
/// the pages carried are the list of what actually ran. The registry is
/// the list of what exists, and the difference is what nothing in this
/// project has exercised — a component can be in the registry, in the
/// tests and in the documentation and still be broken in a page.
///
/// A project is not expected to draw all of them. The number is a fact
/// about this project's coverage, not a failure.
pub fn undrawn(drew: &BTreeSet<String>) -> Vec<String> {
    crate::registry::components()
        .filter(|sig| {
            let crate::registry::Ir::BuiltIn(name) = sig.ir else {
                return false;
            };
            // What never reaches a browser: a PDF or slide component, and
            // the few whose root is somebody else's element.
            if matches!(
                name,
                "Children"
                    | "Router"
                    | "Unsafe"
                    | "UnsafeHtml"
                    | "Host"
                    | "Document"
                    | "Header"
                    | "Footer"
                    | "PageBreak"
                    | "Paragraph"
                    | "Presentation"
                    | "Slide"
                    | "TitleSlide"
                    | "SectionSlide"
                    | "TwoColumn"
                    | "ImageSlide"
            ) {
                return false;
            }
            let (_, class) = crate::codegen::builtin::builtin_to_html(name);
            // `wf-input wf-textarea`: the last class is the one that names it.
            let Some(base) = class.split_whitespace().last() else {
                return false;
            };
            !drew
                .iter()
                .any(|c| c == base || c.starts_with(&format!("{base}--")))
        })
        .map(|sig| sig.name.to_string())
        .collect()
}
