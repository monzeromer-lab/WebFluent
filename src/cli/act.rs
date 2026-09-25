//! Running a test that acts.
//!
//! A test that only looks is rendered: the template engine paints its
//! body and the expects read the result. A test that **clicks** cannot be
//! — a click runs a handler, which runs an action, which changes a store,
//! which repaints. There is one thing that does all of that, and it is a
//! browser.
//!
//! So a test with an interaction in it is compiled into a real project of
//! one page, built, served, and driven. It is the same compiler, the same
//! runtime and the same output a reader would get; nothing about the test
//! is special except that it is thrown away afterwards.

use std::path::Path;

use crate::browser::Browser;
use crate::browser::chrome::Page;
use crate::error::Result;
use crate::parser::ast::{Declaration, Expr, PageDecl, Program, Statement, StatementKind, Step};

/// A browser, kept for as long as there are tests that need one.
pub struct Stage {
    browser: Browser,
    work: std::path::PathBuf,
}

impl Stage {
    pub fn open() -> Result<Self> {
        Ok(Stage {
            browser: Browser::start()?,
            work: std::env::temp_dir().join(format!("wf-act-{}", std::process::id())),
        })
    }

    /// Run one test, and say what went wrong — nothing, when it passed.
    pub fn run(
        &mut self,
        shared: &[Declaration],
        test: &crate::parser::ast::TestDecl,
    ) -> std::result::Result<(), Vec<String>> {
        self.attempt(shared, test)
            .unwrap_or_else(|e| Err(vec![e.to_string()]))
    }

    fn attempt(
        &mut self,
        shared: &[Declaration],
        test: &crate::parser::ast::TestDecl,
    ) -> Result<std::result::Result<(), Vec<String>>> {
        let dir = self.work.join(slug(&test.name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        // The program as written, with the test's body as its one page.
        // It is compiled here rather than written back out as source:
        // there is no printer from the tree to the text, and a test does
        // not need one — the compiler takes the tree.
        let program = crate::sema::lower(as_program(shared, test));
        write_site(&program, &dir)?;

        let server = super::preview::serve_directory(dir.clone(), "")?;
        let origin = server.origin.clone();
        let outcome = self.act(&origin, test);
        server.close();
        let _ = std::fs::remove_dir_all(&dir);
        outcome
    }

    fn act(
        &mut self,
        origin: &str,
        test: &crate::parser::ast::TestDecl,
    ) -> Result<std::result::Result<(), Vec<String>>> {
        let mut page = self.browser.open(&format!("{origin}/"), 600)?;
        let mut reasons = Vec::new();
        for step in &test.steps {
            if let Err(why) = self.step(&mut page, step)? {
                reasons.push(why);
                break; // the steps after a failed one mean nothing
            }
        }
        // One last listen, for anything the page said after the last
        // step and has not been heard yet.
        self.browser.settle(&mut page, 80)?;
        // Anything the page itself complained about is the test's problem
        // too: a handler that threw is a failure, not a passing click.
        for error in page.errors() {
            reasons.push(format!("the page reported: {error}"));
        }
        let shown = self.browser.visible_text(&page).unwrap_or_default();
        let _ = self.browser.close(page);
        if reasons.is_empty() {
            return Ok(Ok(()));
        }
        reasons.push(format!("the page showed:\n{}", indent(&shown)));
        Ok(Err(reasons))
    }

    /// One step, against the open page.
    fn step(&mut self, page: &mut Page, step: &Step) -> Result<std::result::Result<(), String>> {
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
                let shown = self.browser.visible_text(page)?;
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
                let done = self.browser.eval(page, &script("click", &name, ""))?;
                if done.as_str() != Some("ok") {
                    return Ok(Err(format!("nothing to click called {name:?}")));
                }
                self.browser.settle(page, 120)?;
            }
            Step::Type {
                text: what, into, ..
            } => {
                let what = text(what);
                let into = text(into);
                let done = self.browser.eval(page, &script("type", &into, &what))?;
                if done.as_str() != Some("ok") {
                    return Ok(Err(format!("no control called {into:?} to type into")));
                }
                self.browser.settle(page, 120)?;
            }
            Step::Press { key, target, .. } => {
                let key = text(key);
                let where_ = target.as_ref().map(text).unwrap_or_default();
                let done = self.browser.eval(page, &script("press", &where_, &key))?;
                if done.as_str() != Some("ok") {
                    return Ok(Err(format!(
                        "nothing called {where_:?} to press {key:?} in"
                    )));
                }
                self.browser.settle(page, 120)?;
            }
        }
        Ok(Ok(()))
    }
}

/// Compile `program` into `dir`: the three files a page needs.
///
/// Not `wf build`, which reads a project from disk and writes a site with
/// everything a site has. This is the same code generators over a program
/// already in hand, for a page that exists for one test and is then
/// thrown away.
fn write_site(program: &Program, dir: &Path) -> Result<()> {
    let mut config = crate::config::ProjectConfig::default_config("test");
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
fn as_program(shared: &[Declaration], test: &crate::parser::ast::TestDecl) -> Program {
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
/// value — the same seeding a rendered test gets, for the one thing a
/// built page can take it from.
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
/// button is its text, a control is its label or its placeholder — which
/// is the same name the accessibility checks hold a page to, so a test
/// that passes here is a page somebody can use.
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

fn indent(text: &str) -> String {
    text.lines()
        .take(24)
        .map(|l| format!("        {l}\n"))
        .collect()
}

fn slug(name: &str) -> String {
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
