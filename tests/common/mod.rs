//! Shared harness for the built-in component conformance suite.
//!
//! The engine has four independent renderers for the same `.wf` source — the SPA
//! JS codegen (`wf build`), the static site generator (`build.ssg`), the template
//! engine (`Template::render_html*`) and the PDF/slides writers. Each one carries
//! its own copy of the built-in component table, so a component can render one way
//! in `wf build` and another way in `wf build --ssg` without anything failing.
//!
//! These helpers reduce all three HTML-producing backends to the same shape — a
//! flat list of [`Elem`] — so a test can state one expectation and hold every
//! backend to it.

#![allow(dead_code)]

use std::collections::HashMap;

use webfluent::codegen::{render_page_html, JsCodegen};
use webfluent::config::ProjectConfig;
use webfluent::lexer::Lexer;
use webfluent::parser::ast::*;
use webfluent::parser::Parser;
use webfluent::Template;

/// Which renderer produced an element. Reported in assertion messages so a
/// failure names the backend that drifted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// `wf build` — the JS bundle that paints the DOM at runtime.
    Spa,
    /// `wf build` with SSG enabled — pre-painted static HTML.
    Ssg,
    /// `Template::render_html_fragment` — the library/template-engine path.
    Template,
}

impl Backend {
    pub fn name(self) -> &'static str {
        match self {
            Backend::Spa => "SPA (codegen/js.rs)",
            Backend::Ssg => "SSG (codegen/ssg.rs)",
            Backend::Template => "Template (template/mod.rs)",
        }
    }

    pub const ALL: [Backend; 3] = [Backend::Spa, Backend::Ssg, Backend::Template];
}

/// One rendered element, normalised across backends.
#[derive(Debug, Clone)]
pub struct Elem {
    pub tag: String,
    pub classes: Vec<String>,
    /// Attribute name -> raw value, excluding `class`/`className`.
    pub attrs: HashMap<String, String>,
    /// The source text this was extracted from, for failure messages.
    pub raw: String,
}

impl Elem {
    pub fn has_class(&self, c: &str) -> bool {
        self.classes.iter().any(|x| x == c)
    }
    pub fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.get(k).map(|s| s.as_str())
    }
}

// ─── Source construction ────────────────────────────────────────────────

/// Wrap a body snippet in a minimal page.
pub fn page(body: &str) -> String {
    format!("Page P (path: \"/\", title: \"T\") {{\n{}\n}}\n", body)
}

pub fn parse_program(src: &str) -> Result<Program, String> {
    let tokens = Lexer::new(src, "<test>")
        .tokenize()
        .map_err(|e| format!("lex error: {e:?}"))?;
    Parser::new(tokens, "<test>")
        .parse()
        .map_err(|e| format!("parse error: {e:?}"))
}

fn test_config() -> ProjectConfig {
    serde_json::from_str(r#"{"name":"t"}"#).expect("test config")
}

// ─── Backend drivers ────────────────────────────────────────────────────

/// The JS bundle `wf build` writes to `app.js`.
pub fn spa_js(src: &str) -> String {
    let program = parse_program(src).expect("source must parse");
    JsCodegen::new().generate(&program)
}

/// The static HTML `wf build` writes when SSG is on (full document).
pub fn ssg_html(src: &str) -> String {
    let program = parse_program(src).expect("source must parse");
    let components: HashMap<String, ComponentDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some((c.name.clone(), c.clone())),
            _ => None,
        })
        .collect();
    let app_body: Option<Vec<Statement>> = program.declarations.iter().find_map(|d| {
        if let Declaration::App(a) = d {
            Some(a.body.clone())
        } else {
            None
        }
    });
    let page = program
        .declarations
        .iter()
        .find_map(|d| if let Declaration::Page(p) = d { Some(p) } else { None })
        .expect("source must declare a page");
    render_page_html(
        page,
        &test_config(),
        app_body.as_deref(),
        &HashMap::new(),
        &components,
    )
}

/// The template engine's fragment output.
pub fn template_html(src: &str) -> String {
    Template::from_str(src)
        .expect("source must parse")
        .render_html_fragment(&serde_json::json!({}))
        .expect("fragment must render")
}

/// The JS the codegen wrote for *this* source, with the embedded runtime removed.
///
/// The runtime is a fixed hand-written blob; scanning it for codegen defects only
/// produces false positives.
pub fn spa_generated(src: &str) -> String {
    let js = spa_js(src);
    js.strip_prefix(webfluent::runtime::RUNTIME_JS)
        .map(|s| s.to_string())
        .unwrap_or(js)
}

/// Raw output of one backend, for hygiene checks that care about text, not structure.
pub fn raw_output(backend: Backend, src: &str) -> String {
    match backend {
        Backend::Spa => spa_js(src),
        Backend::Ssg => ssg_html(src),
        Backend::Template => template_html(src),
    }
}

/// Every element one backend emits for `src`, in document order.
pub fn elems(backend: Backend, src: &str) -> Vec<Elem> {
    match backend {
        Backend::Spa => parse_spa(&spa_js(src)),
        Backend::Ssg => parse_html(&strip_document(&ssg_html(src))),
        Backend::Template => parse_html(&template_html(src)),
    }
}

/// The first element a backend emits — the root of the component under test.
pub fn root(backend: Backend, src: &str) -> Option<Elem> {
    elems(backend, src).into_iter().next()
}

/// The first element carrying `class`, skipping structural wrappers.
pub fn first_with_class(backend: Backend, src: &str, class: &str) -> Option<Elem> {
    elems(backend, src).into_iter().find(|e| e.has_class(class))
}

/// Drop the SSG document shell so only the page body remains.
fn strip_document(html: &str) -> String {
    match html.split_once("<div id=\"app\">") {
        Some((_, rest)) => rest
            .rsplit_once("</div>")
            .map(|(body, _)| body.to_string())
            .unwrap_or_else(|| rest.to_string()),
        None => html.to_string(),
    }
}

// ─── Output parsing ─────────────────────────────────────────────────────

/// Pull `WF.h("tag", { … })` calls out of generated JS.
///
/// The codegen emits one call per element, so scanning for the constructor is
/// enough to recover the element list without running the bundle.
fn parse_spa(js: &str) -> Vec<Elem> {
    let mut out = Vec::new();
    // Only look past the embedded runtime, which contains its own `WF.h` uses.
    let body = js.split("function Page_").nth(1).unwrap_or(js);
    let body = match body.find("function Component_") {
        Some(_) => body,
        None => body,
    };

    let mut rest = body;
    while let Some(i) = rest.find("WF.h(\"") {
        let after = &rest[i + 6..];
        let Some(q) = after.find('"') else { break };
        let tag = after[..q].to_string();
        let args = &after[q + 1..];
        // Take the balanced `{ … }` attribute object that follows.
        let obj = match args.find('{') {
            Some(b) => balanced(&args[b..]),
            None => String::new(),
        };
        let class_src = extract_js_class(&obj);
        // Keep the whole call, including any child arguments after the attribute
        // object — a void element handed a child is only visible there.
        let call_end = rest[i..].find('\n').unwrap_or(rest.len() - i);
        out.push(Elem {
            tag,
            classes: split_classes(&class_src),
            attrs: parse_js_attrs(&obj),
            raw: rest[i..i + call_end].to_string(),
        });
        rest = &rest[i + 6..];
    }
    out
}

/// The substring from `{` to its matching `}`, respecting string literals.
fn balanced(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut prev_escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        match in_str {
            Some(q) => {
                if prev_escape {
                    prev_escape = false;
                } else if b == b'\\' {
                    prev_escape = true;
                } else if b == q {
                    in_str = None;
                }
            }
            None => match b {
                b'"' | b'\'' => in_str = Some(b),
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return s[..=i].to_string();
                    }
                }
                _ => {}
            },
        }
    }
    s.to_string()
}

/// The class list from a `className:` property.
///
/// Handles both the static form (`className: "wf-card wf-card--elevated"`) and
/// the reactive ternary the open/close components emit
/// (`className: () => _s() ? "wf-modal open" : "wf-modal"`). For the ternary we
/// take the union of both branches minus the state word, since a test cares that
/// the component's own classes survive, not which branch is live.
fn extract_js_class(obj: &str) -> String {
    let inner = obj.trim().trim_start_matches('{').trim_end_matches('}');
    // Take only the `className` property, not every string in the object —
    // `src:`/`alt:` values are not classes.
    let Some(prop) = split_top_level_commas(inner)
        .into_iter()
        .find(|p| p.trim_start().trim_start_matches('"').starts_with("className"))
    else {
        return String::new();
    };
    let Some((_, value)) = prop.split_once(':') else {
        return String::new();
    };

    let mut lits = Vec::new();
    let mut rest = value;
    while let Some(a) = rest.find('"') {
        let after_q = &rest[a + 1..];
        let Some(b) = after_q.find('"') else { break };
        lits.push(after_q[..b].to_string());
        rest = &after_q[b + 1..];
    }
    // Both branches of the open/closed ternary name the same base classes; keep
    // each once so a reactive className compares equal to a static one.
    let mut seen: Vec<String> = Vec::new();
    for c in lits.join(" ").split_whitespace() {
        if !seen.iter().any(|s| s == c) {
            seen.push(c.to_string());
        }
    }
    seen.join(" ")
}

fn parse_js_attrs(obj: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    // `key: value` pairs at depth 1; keys may be quoted ("on:click", "data-icon").
    let inner = obj.trim().trim_start_matches('{').trim_end_matches('}');
    for part in split_top_level_commas(inner) {
        let part = part.trim();
        let (k, v) = match part.split_once(':') {
            Some(kv) => kv,
            None => continue,
        };
        let key = k.trim().trim_matches('"').trim_matches('\'').to_string();
        if key == "className" || key.is_empty() {
            continue;
        }
        map.insert(key, v.trim().trim_matches('"').to_string());
    }
    map
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut prev_escape = false;
    let mut start = 0usize;
    for (i, &b) in s.as_bytes().iter().enumerate() {
        match in_str {
            Some(q) => {
                if prev_escape {
                    prev_escape = false;
                } else if b == b'\\' {
                    prev_escape = true;
                } else if b == q {
                    in_str = None;
                }
            }
            None => match b {
                b'"' | b'\'' => in_str = Some(b),
                b'{' | b'(' | b'[' => depth += 1,
                b'}' | b')' | b']' => depth -= 1,
                b',' if depth == 0 => {
                    out.push(s[start..i].to_string());
                    start = i + 1;
                }
                _ => {}
            },
        }
    }
    out.push(s[start..].to_string());
    out
}

/// Pull opening tags out of an HTML fragment.
fn parse_html(html: &str) -> Vec<Elem> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find('<') {
        let after = &rest[i + 1..];
        if after.starts_with('/') || after.starts_with('!') {
            rest = after;
            continue;
        }
        let Some(end) = after.find('>') else { break };
        let inner = &after[..end];
        let mut parts = inner.splitn(2, char::is_whitespace);
        let tag = parts.next().unwrap_or("").trim_end_matches('/').to_string();
        if tag.is_empty() || !tag.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            rest = after;
            continue;
        }
        let attr_src = parts.next().unwrap_or("");
        let attrs = parse_html_attrs(attr_src);
        let classes = attrs
            .get("class")
            .map(|c| split_classes(c))
            .unwrap_or_default();
        let mut attrs = attrs;
        attrs.remove("class");
        out.push(Elem {
            tag,
            classes,
            attrs,
            raw: format!("<{}>", inner),
        });
        rest = after;
    }
    out
}

fn parse_html_attrs(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let name_start = i;
        while i < bytes.len() && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
            i += 1;
        }
        if name_start == i {
            break;
        }
        let name = s[name_start..i].trim_end_matches('/').to_string();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            let value = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                let v = s[vs..i.min(s.len())].to_string();
                i += 1;
                v
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                s[vs..i].to_string()
            };
            if !name.is_empty() {
                map.insert(name, value);
            }
        } else if !name.is_empty() {
            // Boolean attribute.
            map.insert(name, String::new());
        }
    }
    map
}

fn split_classes(s: &str) -> Vec<String> {
    s.split_whitespace().map(|c| c.to_string()).collect()
}

// ─── Reference data ─────────────────────────────────────────────────────

/// HTML void elements — must never be given children or a closing tag.
pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

/// Every modifier the parser accepts, from the single-source vocabulary table.
pub fn all_modifiers() -> Vec<&'static str> {
    webfluent::parser::MODIFIER_KEYWORDS.to_vec()
}

/// The full stylesheet a default build ships.
pub fn built_in_css() -> String {
    webfluent::codegen::generate_css("default", &HashMap::new())
}

/// Whether the built-in stylesheet defines a rule for `class`.
pub fn css_defines_class(css: &str, class: &str) -> bool {
    let needle = format!(".{}", class);
    let mut rest = css;
    while let Some(i) = rest.find(&needle) {
        let after = &rest[i + needle.len()..];
        // A real selector ends here; `.wf-btn` must not match `.wf-btn-group`.
        let boundary = after
            .chars()
            .next()
            .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_');
        if boundary {
            return true;
        }
        rest = after;
    }
    false
}
