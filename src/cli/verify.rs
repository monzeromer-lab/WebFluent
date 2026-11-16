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
use crate::diagnostics::error::{Result, WebFluentError};

pub(crate) fn run_verify(
    project_dir: &Path,
    report: super::Report,
    budget_ms: Option<u64>,
    returning: bool,
    allow_output_outside: bool,
    fakes: Option<&Path>,
) -> Result<()> {
    let json = report == super::Report::Json;
    let config = ProjectConfig::load(project_dir)?;
    // What is served to the browser is the project's own output: a config
    // that names a directory elsewhere would serve that to the page. With
    // `--fakes`, it is a build of its own, with the fakes answering every
    // endpoint, written aside and thrown away after.
    let faked = match fakes {
        Some(file) => Some(build_with_fakes(project_dir, file, json)?),
        None => None,
    };
    let output_dir = match &faked {
        Some(dir) => dir.clone(),
        None => {
            crate::build::project_output(project_dir, &config.build.output, allow_output_outside)?
        }
    };
    let outcome = visit_all(
        project_dir,
        &config,
        &output_dir,
        json,
        budget_ms,
        returning,
    );
    if let Some(dir) = faked {
        let _ = std::fs::remove_dir_all(dir);
    }
    outcome
}

/// The project built with the fakes `file` declares answering its
/// endpoints, into a folder of its own: where it was written.
fn build_with_fakes(project_dir: &Path, file: &Path, json: bool) -> Result<std::path::PathBuf> {
    // The file as the project names it: relative to the project, whether it
    // was given that way or from where the command was run.
    let absolute = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let project = absolute(project_dir);
    let given = absolute(file);
    let relative = given
        .strip_prefix(&project)
        .map(Path::to_path_buf)
        .or_else(|_| {
            let under = project.join(file);
            under.is_file().then(|| file.to_path_buf()).ok_or(())
        })
        .map_err(|()| {
            WebFluentError::io(format!(
                "{}: a file of fakes is one of the project's, under {}",
                file.display(),
                project_dir.display()
            ))
        })?;
    let relative = relative.to_string_lossy().replace('\\', "/");
    let out = std::env::temp_dir().join(format!("wf-verify-fakes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let mut options = crate::build::BuildOptions::new(project_dir, &crate::base::vfs::FsVfs);
    options.output_dir = Some(out.clone());
    options.fakes = Some(&relative);
    // What the build said is said only when it failed: what it found is
    // the reason there is nothing to load.
    crate::build::build(&options).map_err(|failure| {
        let _ = std::fs::remove_dir_all(&out);
        if !json {
            for line in &failure.report.lines {
                match line.stream {
                    crate::build::Stream::Stdout => println!("{}", line.text),
                    crate::build::Stream::Stderr => eprintln!("{}", line.text),
                }
            }
        }
        failure.error
    })?;
    Ok(out)
}

fn visit_all(
    project_dir: &Path,
    config: &ProjectConfig,
    output_dir: &Path,
    json: bool,
    budget_ms: Option<u64>,
    returning: bool,
) -> Result<()> {
    let output_dir = output_dir.to_path_buf();
    if !output_dir.exists() {
        return Err(WebFluentError::io(format!(
            "nothing to verify: {} does not exist. Run `wf build` first",
            output_dir.display()
        )));
    }
    let base_path = config.build.base_path.trim_end_matches('/').to_string();
    let routes = crate::verify::routes(&output_dir, project_dir, config)?;
    if routes.is_empty() {
        return Err(WebFluentError::io("the build wrote no pages to load"));
    }

    let server = super::preview::serve_directory(output_dir, &base_path)?;
    let mut visits = Vec::new();
    let mut drew: BTreeSet<String> = BTreeSet::new();
    // The controls drawn under 24×24: said, never failed on.
    let mut small = 0usize;
    let mut problems = 0usize;

    if !json {
        println!("  {} route(s) in {}", routes.len(), server.origin);
    }
    // A first visit, then — with `--returning-visitor` — a second, by a
    // reader whose storage holds what the previous build's pages kept.
    let mut passes = vec![None];
    if returning {
        passes.push(Some(crate::verify::returning_storage(project_dir)));
    }
    // A site of many routes is visited by a few browsers at once, each
    // taking every n-th route; a small one by one. Each browser is a
    // Chrome of its own, so they are few.
    let browsers = routes
        .len()
        .div_ceil(40)
        .min(std::thread::available_parallelism().map_or(1, |n| n.get() / 2))
        .clamp(1, 3);
    for pass in passes {
        let label = if pass.is_some() { " (returning)" } else { "" };
        let origin = format!("{}{}", server.origin, base_path);
        let visited: Vec<Result<Vec<(usize, crate::verify::Visit)>>> =
            std::thread::scope(|scope| {
                let workers: Vec<_> = (0..browsers)
                    .map(|k| {
                        let (routes, origin, pass) = (&routes, &origin, pass.clone());
                        scope.spawn(move || -> Result<Vec<(usize, crate::verify::Visit)>> {
                            let mut browser = Browser::start()?;
                            browser.before_each_page(pass);
                            // A page that has loaded, fetched what it asked
                            // for and said nothing for a tenth of a second is
                            // done; 900 ms is the most any is given.
                            browser.settle_when_quiet(Some(100));
                            let mut out = Vec::new();
                            for (i, route) in routes.iter().enumerate().skip(k).step_by(browsers) {
                                out.push((i, browser.visit(&format!("{origin}{route}"), 900)?));
                            }
                            Ok(out)
                        })
                    })
                    .collect();
                workers
                    .into_iter()
                    .map(|w| {
                        w.join()
                            .unwrap_or_else(|_| Err(WebFluentError::io("a browser stopped")))
                    })
                    .collect()
            });
        let mut in_order = Vec::with_capacity(routes.len());
        for part in visited {
            in_order.extend(part?);
        }
        in_order.sort_by_key(|(i, _)| *i);
        for (i, mut visit) in in_order {
            let route = &routes[i];
            visit.url = format!("{route}{label}");
            crate::verify::judge(&mut visit, route, budget_ms);
            drew.extend(visit.drew.iter().cloned());
            problems += visit.errors.len();
            small += visit.small.len();
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
                for small in &visit.small {
                    println!("         small: {small}, under 24×24");
                }
            }
            visits.push(visit);
        }
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
                "small": small,
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
        if small > 0 {
            println!(
                "\n  {small} control(s) drawn under 24×24, the least a pointer target should be (WCAG 2.5.8)"
            );
        }
        println!("\n  {} page(s), {problems} problem(s)", visits.len());
    }
    if problems > 0 {
        return Err(WebFluentError::io(format!(
            "{problems} problem(s) in the browser"
        )));
    }
    Ok(())
}
