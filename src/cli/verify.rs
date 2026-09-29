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
    let routes = crate::verify::routes(&output_dir, project_dir, &base_path)?;
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
        crate::verify::judge(&mut visit, route, budget_ms);
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
    let undrawn = crate::verify::undrawn(&drew);
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

