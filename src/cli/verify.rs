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

pub fn run_verify(
    project_dir: &Path,
    json: bool,
    budget_ms: Option<u64>,
    returning: bool,
) -> Result<()> {
    let config = ProjectConfig::load(project_dir)?;
    let output_dir = project_dir.join(&config.build.output);
    if !output_dir.exists() {
        return Err(WebFluentError::IoError(format!(
            "nothing to verify: {} does not exist. Run `wf build` first",
            output_dir.display()
        )));
    }
    let base_path = config.build.base_path.trim_end_matches('/').to_string();
    let routes = routes(&output_dir, project_dir, &config)?;
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
    // A first visit, then — with `--returning-visitor` — a second, by a
    // reader whose storage holds what the previous build's pages kept.
    let mut passes = vec![None];
    if returning {
        passes.push(Some(returning_storage(project_dir)));
    }
    for pass in passes {
        let label = if pass.is_some() { " (returning)" } else { "" };
        browser.before_each_page(pass);
        for route in &routes {
            let url = format!("{}{}{}", server.origin, base_path, route);
            let mut visit = browser.visit(&url, 900)?;
            visit.url = format!("{route}{label}");
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
                    visit.url,
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

/// Every route the build serves: the files it wrote, the pages the project
/// declares that a single-page build serves from one shell, and each
/// `:param` route — at the values its `paths:` names, or, without them, at
/// a placeholder (`1`), which shows how the page meets a value it may not
/// know.
fn routes(output_dir: &Path, project_dir: &Path, config: &ProjectConfig) -> Result<Vec<String>> {
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
            let crate::parser::ast::Declaration::Page(page) = decl else {
                continue;
            };
            if page.path == "*" || page.path.is_empty() {
                continue;
            }
            if !page.path.contains(':') {
                found.insert(page.path.clone());
                continue;
            }
            match crate::codegen::ssg::static_routes(page, &program, &config.env) {
                Ok(listed) if !listed.is_empty() => {
                    found.extend(listed.into_iter().map(|(route, _)| route));
                }
                _ => {
                    let placeholder: Vec<String> = page
                        .path
                        .split('/')
                        .map(|seg| {
                            if seg.starts_with(':') {
                                "1".to_string()
                            } else {
                                seg.to_string()
                            }
                        })
                        .collect();
                    found.insert(placeholder.join("/"));
                }
            }
        }
    }
    Ok(found.into_iter().collect())
}

/// The script a returning visitor's pages run first: storage holding what
/// the previous build's pages kept (`.wf-cache/persist-values.previous.json`,
/// written when a build changed them), else what this build's keep.
fn returning_storage(project_dir: &Path) -> String {
    let read = |name: &str| {
        std::fs::read_to_string(project_dir.join(name))
            .ok()
            .and_then(|t| {
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&t).ok()
            })
    };
    let values = read(crate::linter::project::PERSIST_VALUES_BEFORE)
        .or_else(|| read(crate::linter::project::PERSIST_VALUES))
        .unwrap_or_default();
    let mut script = String::from("try {");
    for (key, value) in &values {
        script.push_str(&format!(
            "localStorage.setItem({}, {});",
            serde_json::to_string(key).unwrap_or_default(),
            serde_json::to_string(&value.to_string()).unwrap_or_default()
        ));
    }
    script.push_str("} catch (e) {}");
    script
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn param_routes_are_visited_at_their_paths_or_a_placeholder() {
        let dir = std::env::temp_dir().join(format!("wf-verify-routes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("build")).unwrap();
        std::fs::write(dir.join("build/index.html"), "<!doctype html>").unwrap();
        std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "v" }"#).unwrap();
        std::fs::write(
            dir.join("src/App.wf"),
            "app { Router }\nconst SLUGS = [\"a\", \"b\"]\npage Home(path: \"/\", title: \"H\", description: \"D\") { Heading(\"H\").h1 }\npage Post(path: \"/p/:slug\", title: \"P\", description: \"D\", slug: String, paths: SLUGS) { Heading(slug).h1 }\npage User(path: \"/user/:id\", title: \"U\", description: \"D\", id: String) { Heading(id).h1 }\npage Missing(path: \"*\", title: \"M\", description: \"D\") { Heading(\"404\").h1 }\n",
        )
        .unwrap();
        let config = ProjectConfig::load(&dir).unwrap();
        let found = routes(&dir.join("build"), &dir, &config).unwrap();
        assert_eq!(found, vec!["/", "/p/a", "/p/b", "/user/1"]);
    }

    #[test]
    fn a_returning_visitor_has_the_previous_build_s_values() {
        let dir = std::env::temp_dir().join(format!("wf-verify-return-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".wf-cache")).unwrap();
        std::fs::write(
            dir.join(crate::linter::project::PERSIST_VALUES),
            r#"{ "wf:Home.items": [] }"#,
        )
        .unwrap();
        assert!(returning_storage(&dir).contains(r#"localStorage.setItem("wf:Home.items", "[]")"#));
        std::fs::write(
            dir.join(crate::linter::project::PERSIST_VALUES_BEFORE),
            r#"{ "wf:Home.items": { "a": 1 } }"#,
        )
        .unwrap();
        let script = returning_storage(&dir);
        assert!(
            script.contains(r#""wf:Home.items", "{\"a\":1}""#),
            "{script}"
        );
    }
}
