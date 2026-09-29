//! Running a project's tests: every `test "…" { … }` under `tests/` (and in
//! `src/`), rendered through the template engine over its data, held to what
//! it expects and to its snapshot; and a test that acts, built as a site of
//! one page and played in a browser.
//!
//! A test's body renders with the project's components, stores, types,
//! enums and constants at hand, as a page's would; a `state` in the body
//! is seeded with its initial value, and `data: { … }` supplies the rest
//! by name. The rendered fragment must contain every `expect "text"` and
//! none of the `expect not "text"`; it is then compared to
//! `tests/__snapshots__/<file>/<test>.html`, written when missing and
//! rewritten with `update`.
//!
//! `wf test` prints what this finds; the mobile editor shows it. The
//! browser a test that acts needs is theirs: Chrome for `wf test`, the
//! editor's WebView (the [`Browser`] trait).

use std::path::{Path, PathBuf};

use crate::config::project::ThemeConfig;
use crate::error::Result;
use crate::parser::ast::*;

/// A project's tests, read and ready to run.
pub struct Tests {
    /// What every test renders with: everything but pages, the app and the
    /// tests.
    pub shared: Vec<Declaration>,
    /// Each test, with the file it is in (project-relative).
    pub tests: Vec<(String, TestDecl)>,
    /// `tests/__snapshots__`.
    pub snapshots: PathBuf,
    /// The project's theme settings, for a test that acts: a project that
    /// declares a dark theme declares two, and a page built without knowing
    /// which to use refuses to build.
    pub theme: ThemeConfig,
}

/// How one test went.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Passed,
    /// It passed, and its snapshot was written: there was none, or
    /// `update` asked for the render to be kept.
    Written,
    /// Why it failed, one reason a line; the last may be what it rendered.
    Failed(Vec<String>),
}

/// The tests of the project at `path`, or of the one test file `path` names.
pub fn collect(path: &Path) -> Result<Tests> {
    let project_dir = if path.is_file() {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        path.to_path_buf()
    };
    let (program, files) = crate::build::read_project(&project_dir)?;
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
    let theme = crate::config::ProjectConfig::load(&project_dir)
        .map(|config| config.theme)
        .unwrap_or_default();
    Ok(Tests {
        shared,
        tests,
        snapshots: project_dir.join("tests").join("__snapshots__"),
        theme,
    })
}

impl Tests {
    /// How many of the tests act, and so need a browser.
    pub fn acting(&self) -> usize {
        self.tests.iter().filter(|(_, t)| t.acts()).count()
    }

    /// Run a test that only looks: render it, hold it to its expects and its
    /// snapshot (written when missing, or with `update`).
    pub fn run_rendered(&self, file: &str, test: &TestDecl, update: bool) -> Outcome {
        match run_one(&self.shared, test, &self.snapshots, file, update) {
            Ok(written) if written => Outcome::Written,
            Ok(_) => Outcome::Passed,
            Err(reasons) => Outcome::Failed(reasons),
        }
    }

    /// Run a test that acts in `browser`, its site built under `work`.
    pub fn run_acting(&self, browser: &mut dyn Browser, test: &TestDecl, work: &Path) -> Outcome {
        match act(browser, &self.shared, test, work, &self.theme) {
            Ok(Ok(())) => Outcome::Passed,
            Ok(Err(reasons)) => Outcome::Failed(reasons),
            Err(e) => Outcome::Failed(vec![e.to_string()]),
        }
    }
}

/// The rendered fragment of one test: its body as a page over the shared
/// declarations, seeded with its `state` and its `data`.
pub fn render_test(shared: &[Declaration], test: &TestDecl) -> Result<String> {
    let mut declarations = shared.to_vec();
    declarations.push(Declaration::Page(PageDecl {
        name: "__Test".to_string(),
        path: "/__test".to_string(),
        title: None,
        title_expr: None,
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

/// One test that looks: `Ok(true)` when it passed and its snapshot was
/// written, `Ok(false)` when it passed against the one there.
fn run_one(
    shared: &[Declaration],
    test: &TestDecl,
    snapshots: &Path,
    file: &str,
    update: bool,
) -> std::result::Result<bool, Vec<String>> {
    let html = render_test(shared, test).map_err(|e| vec![e.to_string()])?;
    let mut reasons = Vec::new();
    let empty = crate::codegen::static_eval::Scope::of([]);
    for step in &test.steps {
        let Step::Expect { text, negated, .. } = step else {
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
    let snapshot = snapshot_path(snapshots, file, test);
    if snapshot.exists() && !update {
        let stored = std::fs::read_to_string(&snapshot).map_err(|e| vec![e.to_string()])?;
        if stored != html {
            return Err(vec![
                format!("the render differs from {}", snapshot.display()),
                "run `wf test --update` to accept the render".to_string(),
                format!("rendered:\n{}", indent(&html)),
            ]);
        }
        return Ok(false);
    }
    if let Some(dir) = snapshot.parent() {
        std::fs::create_dir_all(dir).map_err(|e| vec![e.to_string()])?;
    }
    std::fs::write(&snapshot, &html).map_err(|e| vec![e.to_string()])?;
    Ok(true)
}

/// The file a test's snapshot is kept in.
pub fn snapshot_path(snapshots: &Path, file: &str, test: &TestDecl) -> PathBuf {
    let stem = Path::new(file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "tests".to_string());
    snapshots
        .join(stem)
        .join(format!("{}.html", slug(&test.name)))
}

pub(crate) fn slug(name: &str) -> String {
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

// ── Tests that act ─────────────────────────────────────────────────────────
//
// A test that only looks is rendered: the template engine paints its body
// and the expects read the result. A test that **clicks** cannot be — a
// click runs a handler, which runs an action, which changes a store, which
// repaints. There is one thing that does all of that, and it is a browser.
//
// So a test with an interaction in it is compiled into a real project of
// one page, built, served, and driven. It is the same compiler, the same
// runtime and the same output a reader would get; nothing about the test
// is special except that it is thrown away afterwards.

/// A page of a test's site, open in a browser.
pub trait Page {
    /// Run `script` in the page and hand back what it gave.
    fn eval(&mut self, script: &str) -> Result<serde_json::Value>;
    /// Give the page `ms` to react, listening to what it says meanwhile.
    fn settle(&mut self, ms: u64) -> Result<()>;
    /// Anything the page said went wrong, in the order it said it: an
    /// exception, a `console.error`.
    fn errors(&self) -> Vec<String>;
}

/// A browser a test that acts runs in: Chrome for `wf test`, the mobile
/// editor's WebView.
pub trait Browser {
    /// Serve the site in `dir` (`index.html`, `app.js`, `styles.css`), open
    /// its root and let it settle for `settle_ms`. The page closes when it is
    /// dropped.
    fn open(&mut self, dir: &Path, settle_ms: u64) -> Result<Box<dyn Page + '_>>;
}

/// Build a test that acts as a site of one page under `work`, open it in
/// `browser`, and play its steps: what went wrong, if anything.
pub fn act(
    browser: &mut dyn Browser,
    shared: &[Declaration],
    test: &TestDecl,
    work: &Path,
    theme: &ThemeConfig,
) -> Result<std::result::Result<(), Vec<String>>> {
    let dir = work.join(slug_acting(&test.name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    // The program as written, with the test's body as its one page. It is
    // compiled here rather than written back out as source: there is no
    // printer from the tree to the text, and a test does not need one — the
    // compiler takes the tree.
    let program = crate::sema::lower(as_program(shared, test));
    write_site(&program, &dir, theme)?;
    let outcome = match browser.open(&dir, 600) {
        Ok(mut page) => play(page.as_mut(), test),
        Err(e) => Err(e),
    };
    let _ = std::fs::remove_dir_all(&dir);
    outcome
}

/// A test's steps against its open page.
fn play(page: &mut dyn Page, test: &TestDecl) -> Result<std::result::Result<(), Vec<String>>> {
    let mut reasons = Vec::new();
    for step in &test.steps {
        if let Err(why) = step_one(page, step)? {
            reasons.push(why);
            break; // the steps after a failed one mean nothing
        }
    }
    // One last listen, for anything the page said after the last step and
    // has not been heard yet.
    page.settle(80)?;
    // Anything the page itself complained about is the test's problem too:
    // a handler that threw is a failure, not a passing click.
    for error in page.errors() {
        reasons.push(format!("the page reported: {error}"));
    }
    let shown = visible_text(page).unwrap_or_default();
    if reasons.is_empty() {
        return Ok(Ok(()));
    }
    reasons.push(format!("the page showed:\n{}", indent_acting(&shown)));
    Ok(Err(reasons))
}

/// The text a reader would see, for an `expect`.
pub fn visible_text(page: &mut dyn Page) -> Result<String> {
    let value = page.eval("((document.querySelector('main') || document.body).innerText || '')")?;
    Ok(value.as_str().unwrap_or_default().to_string())
}

/// One step, against the open page.
fn step_one(page: &mut dyn Page, step: &Step) -> Result<std::result::Result<(), String>> {
    let text = |e: &Expr| {
        crate::codegen::static_eval::eval(e, &crate::codegen::static_eval::Scope::of([]))
            .map(|v| v.to_text())
            .unwrap_or_default()
    };
    match step {
        Step::Expect {
            text: want,
            negated,
            ..
        } => {
            let want = text(want);
            let shown = visible_text(page)?;
            let found = shown.contains(&want);
            if found == *negated {
                return Ok(Err(if *negated {
                    format!("expected not to find {want:?}, and the page shows it")
                } else {
                    format!("expected {want:?}, which the page does not show")
                }));
            }
        }
        Step::Click { target, .. } => {
            let name = text(target);
            let done = page.eval(&script("click", &name, ""))?;
            if done.as_str() != Some("ok") {
                return Ok(Err(format!("nothing to click called {name:?}")));
            }
            page.settle(120)?;
        }
        Step::Type {
            text: what, into, ..
        } => {
            let what = text(what);
            let into = text(into);
            let done = page.eval(&script("type", &into, &what))?;
            if done.as_str() != Some("ok") {
                return Ok(Err(format!("no control called {into:?} to type into")));
            }
            page.settle(120)?;
        }
        Step::Press { key, target, .. } => {
            let key = text(key);
            let where_ = target.as_ref().map(text).unwrap_or_default();
            let done = page.eval(&script("press", &where_, &key))?;
            if done.as_str() != Some("ok") {
                return Ok(Err(format!(
                    "nothing called {where_:?} to press {key:?} in"
                )));
            }
            page.settle(120)?;
        }
    }
    Ok(Ok(()))
}

/// Compile `program` into `dir`: the three files a page needs.
///
/// Not `wf build`, which reads a project from disk and writes a site with
/// everything a site has. This is the same code generators over a program
/// already in hand, for a page that exists for one test and is then thrown
/// away.
pub fn write_site(program: &Program, dir: &Path, theme: &ThemeConfig) -> Result<()> {
    let mut config = crate::config::ProjectConfig::default_config("test");
    config.theme = theme.clone();
    // One page, one file: a split build links a chunk that this does not
    // write, and the shell would be served in its place.
    config.build.split = false;
    let mut js = crate::codegen::JsCodegen::new();
    js.set_split_pages(false);
    std::fs::write(dir.join("app.js"), js.generate(program))?;

    let tokens = crate::themes::resolve_tokens(program, &config.theme)?;
    let mut css = crate::codegen::generate_css_for(&tokens, config.theme.builtin, program);
    css.push_str(&crate::codegen::scoped_css::scoped_rules(program));
    std::fs::write(dir.join("styles.css"), css)?;

    std::fs::write(
        dir.join("index.html"),
        crate::codegen::generate_html(&config, program),
    )?;
    Ok(())
}

/// The program a test becomes: everything the project declares, and the
/// test's body as the one page.
fn as_program(shared: &[Declaration], test: &TestDecl) -> Program {
    let mut declarations: Vec<Declaration> = shared
        .iter()
        .filter(|d| {
            !matches!(
                d,
                Declaration::Page(_) | Declaration::App(_) | Declaration::Test(_)
            )
        })
        .cloned()
        .collect();
    declarations.push(Declaration::Page(PageDecl {
        name: "Test".to_string(),
        path: "/".to_string(),
        title: Some(test.name.clone()),
        title_expr: None,
        description: Some("A test.".to_string()),
        guard: None,
        redirect: None,
        image: None,
        page_type: None,
        noindex: true,
        layout: None,
        params: Vec::new(),
        head: Vec::new(),
        paths: None,
        body: seeded(&test.body, test.data.as_ref()),
        span: test.span,
        header_span: test.span,
        body_span: test.span,
    }));
    Program { declarations }
}

/// The body, with any `state` the test's `data` names starting at that
/// value — the same seeding a rendered test gets, for the one thing a built
/// page can take it from.
fn seeded(body: &[Statement], data: Option<&Expr>) -> Vec<Statement> {
    let Some(Expr::MapLiteral(pairs)) = data else {
        return body.to_vec();
    };
    body.iter()
        .map(|stmt| {
            let mut stmt = stmt.clone();
            if let StatementKind::State(state) = &mut stmt.kind
                && let Some((_, value)) = pairs
                    .iter()
                    .find(|(k, _)| k.trim_matches('"') == state.name)
            {
                state.value = value.clone();
            }
            stmt
        })
        .collect()
}

/// The page-side half of a step.
///
/// Everything is found the way a reader finds it: by the name on it. A
/// button is its text, a control is its label or its placeholder — which is
/// the same name the accessibility checks hold a page to, so a test that
/// passes here is a page somebody can use.
fn script(what: &str, name: &str, value: &str) -> String {
    format!(
        "(() => {{\n{FIND}\n  const what = {}, name = {}, value = {};\n{ACT}\n}})()",
        serde_json::to_string(what).unwrap_or_default(),
        serde_json::to_string(name).unwrap_or_default(),
        serde_json::to_string(value).unwrap_or_default(),
    )
}

const FIND: &str = r#"
  const words = (s) => (s || "").replace(/\s+/g, " ").trim();
  const named = (el, n) => {
    const label = el.labels && el.labels[0] ? el.labels[0].textContent : "";
    return [
      el.getAttribute("aria-label"),
      label,
      el.getAttribute("placeholder"),
      el.getAttribute("title"),
      el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.tagName === "SELECT"
        ? null
        : el.textContent,
    ].some((t) => words(t) === n);
  };
  const find = (n, kinds) => {
    const all = [...document.querySelectorAll(kinds)];
    // The one whose own name it is, not an ancestor that contains it.
    return all.filter((el) => named(el, n)).pop() || null;
  };
"#;

const ACT: &str = r#"
  if (what === "click") {
    const el = find(name, "button, a, [role=button], input, select, summary, label, [onclick]");
    if (!el) return "none";
    el.click();
    return "ok";
  }
  if (what === "type") {
    const el = find(name, "input, textarea, select");
    if (!el) return "none";
    el.focus();
    const setter = Object.getOwnPropertyDescriptor(
      el.tagName === "TEXTAREA" ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype,
      "value",
    );
    if (setter && setter.set) setter.set.call(el, value);
    else el.value = value;
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
    return "ok";
  }
  if (what === "press") {
    const el = name ? find(name, "input, textarea, select, button, a, [tabindex]") : (document.activeElement || document.body);
    if (!el) return "none";
    const key = value;
    const init = { key, code: key.length === 1 ? "Key" + key.toUpperCase() : key, bubbles: true, cancelable: true };
    el.dispatchEvent(new KeyboardEvent("keydown", init));
    el.dispatchEvent(new KeyboardEvent("keyup", init));
    return "ok";
  }
  return "none";
"#;

fn indent_acting(text: &str) -> String {
    text.lines()
        .take(24)
        .map(|l| format!("        {l}\n"))
        .collect()
}

/// A test's folder name: act's own, which keeps a leading dash's absence.
fn slug_acting(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}
