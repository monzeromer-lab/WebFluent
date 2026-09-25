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

use std::collections::BTreeSet;
use std::path::Path;

use crate::browser::Browser;
use crate::config::ProjectConfig;
use crate::error::{Result, WebFluentError};

pub fn run_verify(project_dir: &Path, json: bool, budget_ms: Option<u64>) -> Result<()> {
    let config = ProjectConfig::load(project_dir)?;
    let output_dir = project_dir.join(&config.build.output);
    if !output_dir.exists() {
        return Err(WebFluentError::IoError(format!(
            "nothing to verify: {} does not exist. Run `wf build` first",
            output_dir.display()
        )));
    }
    let base_path = config.build.base_path.trim_end_matches('/').to_string();
    let routes = routes(&output_dir, project_dir, &base_path)?;
    if routes.is_empty() {
        return Err(WebFluentError::IoError(
            "the build wrote no pages to load".to_string(),
        ));
    }

    let server = super::preview::serve_directory(output_dir.clone(), &base_path)?;
    let mut browser = Browser::start()?;
    let mut visits = Vec::new();
    let mut drew: BTreeSet<String> = BTreeSet::new();
    let mut problems = 0usize;

    if !json {
        println!("  {} route(s) in {}", routes.len(), server.origin);
    }
    for route in &routes {
        let url = format!("{}{}{}", server.origin, base_path, route);
        let mut visit = browser.visit(&url, 900)?;
        visit.url = route.clone();
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
        drew.extend(visit.drew.iter().cloned());
        problems += visit.errors.len();
        if !json {
            println!(
                "    {} {:<34} {:>5}ms  {:>6} nodes  {:>8.1} kB  {:>3} req",
                if visit.errors.is_empty() {
                    "ok  "
                } else {
                    "FAIL"
                },
                route,
                visit.first_contentful_paint,
                visit.elements,
                visit.transferred as f64 / 1024.0,
                visit.requests,
            );
            for problem in &visit.errors {
                println!("         {problem}");
            }
        }
        visits.push(visit);
    }
    server.close();

    // What the project declares and no page drew. A component nothing
    // builds is a component nothing has run.
    let undrawn = undrawn(&drew);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "routes": visits,
                "undrawn": undrawn,
                "problems": problems,
            }))
            .unwrap_or_default()
        );
    } else {
        if undrawn.is_empty() {
            println!("\n  every built-in this project can draw, it drew");
        } else {
            println!(
                "\n  {} built-in(s) no page drew: {}",
                undrawn.len(),
                undrawn.join(", ")
            );
        }
        println!("\n  {} page(s), {problems} problem(s)", visits.len());
    }
    if problems > 0 {
        return Err(WebFluentError::IoError(format!(
            "{problems} problem(s) in the browser"
        )));
    }
    Ok(())
}

/// Every route the build serves: the files it wrote, and the pages the
/// project declares that a single-page build serves from one shell.
fn routes(output_dir: &Path, project_dir: &Path, base_path: &str) -> Result<Vec<String>> {
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
    if let Ok((program, _)) = super::build::read_project(project_dir) {
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
fn undrawn(drew: &BTreeSet<String>) -> Vec<String> {
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
