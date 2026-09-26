//! `wf test [path] [--update]`: every `test "…" { … }` under `tests/`
//! (and in `src/`), rendered through the template engine over its data,
//! held to what it expects and to its snapshot.
//!
//! A test's body renders with the project's components, stores, types,
//! enums and constants at hand, as a page's would; a `state` in the body
//! is seeded with its initial value, and `data: { … }` supplies the rest
//! by name. The rendered fragment must contain every `expect "text"` and
//! none of the `expect not "text"`; it is then compared to
//! `tests/__snapshots__/<file>/<test>.html`, written when missing and
//! rewritten with `--update`.

use std::path::{Path, PathBuf};

use crate::error::{Result, WebFluentError};
use crate::parser::ast::*;

pub fn run_test(path: &Path, update: bool) -> Result<()> {
    let project_dir = if path.is_file() {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        path.to_path_buf()
    };
    let (program, files) = crate::cli::build::read_project(&project_dir)?;
    let mut declarations = program.declarations;
    let mut owners: Vec<String> = files;
    // The test files: `tests/*.wf` and `tests/*.wfx`, or the one named.
    let test_files: Vec<PathBuf> = if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        let dir = project_dir.join("tests");
        let mut found = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(&dir)? {
                let p = entry?.path();
                if p.extension().is_some_and(|e| e == "wf" || e == "wfx") {
                    found.push(p);
                }
            }
        }
        found.sort();
        found
    };
    for file in &test_files {
        let source = std::fs::read_to_string(file)?;
        let name = file
            .strip_prefix(&project_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();
        let parsed = crate::syntax::parse_source(&source, &name)?;
        owners.extend(parsed.declarations.iter().map(|_| name.clone()));
        declarations.extend(parsed.declarations);
    }
    let tests: Vec<(String, TestDecl)> = declarations
        .iter()
        .zip(&owners)
        .filter_map(|(d, file)| match d {
            Declaration::Test(t) => Some((file.clone(), t.clone())),
            _ => None,
        })
        .collect();
    if tests.is_empty() {
        println!("no tests: declare one as `test \"name\" {{ … expect \"text\" }}` under tests/");
        return Ok(());
    }
    // What every test renders with: everything but pages, the app and the tests.
    let shared: Vec<Declaration> = declarations
        .iter()
        .filter(|d| {
            !matches!(
                d,
                Declaration::Page(_) | Declaration::App(_) | Declaration::Test(_)
            )
        })
        .cloned()
        .collect();
    let snapshots = project_dir.join("tests").join("__snapshots__");
    let mut passed = 0;
    let mut failed = 0;
    let mut written = 0;
    // A test that acts needs a browser, and a browser is expensive to
    // start: one is opened for all of them, and only if any test asks.
    let mut stage = if tests.iter().any(|(_, t)| t.acts()) {
        match crate::cli::act::Stage::open() {
            Ok(stage) => Some(match crate::config::ProjectConfig::load(&project_dir) {
                Ok(config) => stage.with_theme(config.theme),
                Err(_) => stage,
            }),
            Err(e) => {
                println!(
                    "  {} test(s) act, and there is no browser to run them in",
                    tests.iter().filter(|(_, t)| t.acts()).count()
                );
                println!("  {e}");
                return Err(crate::error::WebFluentError::IoError(
                    "a test that clicks needs a browser".to_string(),
                ));
            }
        }
    } else {
        None
    };
    for (file, test) in &tests {
        // A test that clicks is run; one that only looks is rendered.
        if test.acts() {
            let stage = stage.as_mut().expect("opened above");
            match stage.run(&shared, test) {
                Ok(()) => {
                    passed += 1;
                    println!("  ok    {} — {} (in a browser)", file, test.name);
                }
                Err(reasons) => {
                    failed += 1;
                    println!("  FAIL  {} — {}", file, test.name);
                    for r in reasons {
                        println!("        {r}");
                    }
                }
            }
            continue;
        }
        match run_one(&shared, test, &snapshots, file, update) {
            Ok(Outcome::Passed) => {
                passed += 1;
                println!("  ok    {} — {}", file, test.name);
            }
            Ok(Outcome::Written) => {
                passed += 1;
                written += 1;
                println!("  new   {} — {} (snapshot written)", file, test.name);
            }
            Err(reasons) => {
                failed += 1;
                println!("  FAIL  {} — {}", file, test.name);
                for r in reasons {
                    println!("        {r}");
                }
            }
        }
    }
    println!(
        "{passed} passed, {failed} failed{}",
        if written > 0 {
            format!(", {written} snapshot(s) written")
        } else {
            String::new()
        }
    );
    if failed > 0 {
        return Err(WebFluentError::IoError(format!("{failed} test(s) failed")));
    }
    Ok(())
}

enum Outcome {
    Passed,
    Written,
}

/// The rendered fragment of one test: its body as a page over the shared
/// declarations, seeded with its `state` and its `data`.
pub fn render_test(shared: &[Declaration], test: &TestDecl) -> Result<String> {
    let mut declarations = shared.to_vec();
    declarations.push(Declaration::Page(PageDecl {
        name: "__Test".to_string(),
        path: "/__test".to_string(),
        title: None,
        guard: None,
        redirect: None,
        description: None,
        image: None,
        page_type: None,
        noindex: false,
        layout: None,
        params: Vec::new(),
        head: Vec::new(),
        paths: None,
        body: test.body.clone(),
        span: test.span,
        header_span: test.span,
        body_span: test.span,
    }));
    let program = crate::sema::lower(Program { declarations });
    // The data: the test's own, and the body's initial state on top.
    let mut data = match &test.data {
        Some(expr) => {
            crate::codegen::static_eval::eval(expr, &crate::codegen::static_eval::Scope::of([]))
                .map(|v| v.to_json())
                .unwrap_or(serde_json::Value::Object(Default::default()))
        }
        None => serde_json::Value::Object(Default::default()),
    };
    // Every store the test can read, as a value: its initial state with
    // whatever the test's `data` named on top, and its derived values
    // computed from the result. The template engine reads data, not
    // declarations, so without this a `Cart.count` in a test rendered as
    // nothing.
    if let serde_json::Value::Object(map) = &mut data {
        let seeds: Vec<(String, serde_json::Value)> = program
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Store(store) => Some((
                    store.name.clone(),
                    crate::codegen::static_eval::store_as_value(store, map.get(&store.name)),
                )),
                _ => None,
            })
            .collect();
        for (name, value) in seeds {
            map.insert(name, value);
        }
    }
    let scope = crate::codegen::static_eval::Scope::from_program(&program, &test.body);
    if let serde_json::Value::Object(map) = &mut data {
        for stmt in &test.body {
            if let StatementKind::State(s) = &stmt.kind
                && let Some(v) = crate::codegen::static_eval::eval(&s.value, &scope)
            {
                map.entry(s.name.clone()).or_insert(v.to_json());
            }
        }
    }
    crate::template::render_program_fragment(&program, &data)
}

fn run_one(
    shared: &[Declaration],
    test: &TestDecl,
    snapshots: &Path,
    file: &str,
    update: bool,
) -> std::result::Result<Outcome, Vec<String>> {
    let html = render_test(shared, test).map_err(|e| vec![e.to_string()])?;
    let mut reasons = Vec::new();
    let empty = crate::codegen::static_eval::Scope::of([]);
    for step in &test.steps {
        let crate::parser::ast::Step::Expect { text, negated, .. } = step else {
            continue; // a test that acts does not come this way
        };
        let text = crate::codegen::static_eval::eval(text, &empty)
            .map(|v| v.to_text())
            .unwrap_or_default();
        let found = html.contains(&text);
        if found == *negated {
            reasons.push(if *negated {
                format!("expected not to find {text:?}, but the render holds it")
            } else {
                format!("expected {text:?}, which the render does not hold")
            });
        }
    }
    if !reasons.is_empty() {
        reasons.push(format!("rendered:\n{}", indent(&html)));
        return Err(reasons);
    }
    let stem = Path::new(file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "tests".to_string());
    let snapshot = snapshots
        .join(&stem)
        .join(format!("{}.html", slug(&test.name)));
    if snapshot.exists() && !update {
        let stored = std::fs::read_to_string(&snapshot).map_err(|e| vec![e.to_string()])?;
        if stored != html {
            return Err(vec![
                format!("the render differs from {}", snapshot.display()),
                "run `wf test --update` to accept the render".to_string(),
                format!("rendered:\n{}", indent(&html)),
            ]);
        }
        return Ok(Outcome::Passed);
    }
    if let Some(dir) = snapshot.parent() {
        std::fs::create_dir_all(dir).map_err(|e| vec![e.to_string()])?;
    }
    std::fs::write(&snapshot, &html).map_err(|e| vec![e.to_string()])?;
    Ok(Outcome::Written)
}

fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|l| format!("          {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
