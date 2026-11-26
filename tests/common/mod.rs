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

#![allow(
    dead_code,
    reason = "shared by many test binaries, each using a part; an expect would be unfulfilled in some"
)]

use std::collections::HashMap;

use webfluent::Template;
use webfluent::codegen::ssg::SiteContext;
use webfluent::codegen::{JsCodegen, render_page_html};
use webfluent::config::ProjectConfig;
use webfluent::syntax::parser::ast::*;

/// Which renderer produced an element. Reported in assertion messages so a
/// failure names the backend that drifted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Backend {
    /// `wf build` — the JS bundle that paints the DOM at runtime.
    Spa,
    /// `wf build` with SSG enabled — pre-painted static HTML.
    Ssg,
    /// `Template::render_html_fragment` — the library/template-engine path.
    Template,
}

impl Backend {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Backend::Spa => "SPA (codegen/js.rs)",
            Backend::Ssg => "SSG (codegen/ssg.rs)",
            Backend::Template => "Template (template/mod.rs)",
        }
    }

    pub(crate) const ALL: [Backend; 3] = [Backend::Spa, Backend::Ssg, Backend::Template];
}

/// One rendered element, normalised across backends.
#[derive(Debug, Clone)]
pub(crate) struct Elem {
    pub tag: String,
    pub classes: Vec<String>,
    /// Attribute name -> raw value, excluding `class`/`className`.
    pub attrs: HashMap<String, String>,
    /// The source text this was extracted from, for failure messages.
    pub raw: String,
}

impl Elem {
    pub(crate) fn has_class(&self, c: &str) -> bool {
        self.classes.iter().any(|x| x == c)
    }
    pub(crate) fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.get(k).map(std::string::String::as_str)
    }
}

// ─── Source construction ────────────────────────────────────────────────

/// Wrap a body snippet in a minimal page.
pub(crate) fn page(body: &str) -> String {
    format!("page P(path: \"/\", title: \"T\") {{\n{body}\n}}\n")
}

pub(crate) fn parse_program(src: &str) -> Result<Program, String> {
    parse_program_as(src, "<test>")
}

/// Parsed under a file name, which picks the layout: `t.wfx` is indented.
pub(crate) fn parse_program_as(src: &str, file: &str) -> Result<Program, String> {
    webfluent::parse_source(src, file)
        .map(webfluent::sema::lower)
        .map_err(|e| format!("parse error: {e:?}"))
}

fn test_config() -> ProjectConfig {
    serde_json::from_str(r#"{"name":"t"}"#).expect("test config")
}

// ─── Backend drivers ────────────────────────────────────────────────────

/// The JS bundle `wf build` writes to `app.js`.
pub(crate) fn spa_js(src: &str) -> String {
    spa_js_as(src, "<test>")
}

pub(crate) fn spa_js_as(src: &str, file: &str) -> String {
    let program = parse_program_as(src, file).expect("source must parse");
    JsCodegen::new().generate(&program)
}

/// The static HTML `wf build` writes when SSG is on (full document).
pub(crate) fn ssg_html(src: &str) -> String {
    ssg_html_as(src, "<test>")
}

pub(crate) fn ssg_html_as(src: &str, file: &str) -> String {
    let program = parse_program_as(src, file).expect("source must parse");
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
        .find_map(|d| {
            if let Declaration::Page(p) = d {
                Some(p)
            } else {
                None
            }
        })
        .expect("source must declare a page");
    render_page_html(
        page,
        &SiteContext {
            config: &test_config(),
            app_body: app_body.as_deref(),
            translations: &HashMap::new(),
            components: &components,
            program: &program,
            program_scope: None,
            page_sheets: None,
            node_map: None,
        },
    )
}

/// The template engine's fragment output.
pub(crate) fn template_html(src: &str) -> String {
    Template::from_str(src)
        .expect("source must parse")
        .render_html_fragment(&serde_json::json!({}))
        .expect("fragment must render")
}

/// The JS the codegen wrote for *this* source, with the embedded runtime removed.
///
/// The runtime is hand-written; scanning it for codegen defects only produces
/// false positives. It is everything up to the close of its closure.
pub(crate) fn spa_generated(src: &str) -> String {
    const END: &str = "\n})();\n";

    let js = spa_js(src);
    match js.find(END) {
        Some(at) => js[at + END.len()..].to_string(),
        None => js,
    }
}

/// Raw output of one backend, for hygiene checks that care about text, not structure.
pub(crate) fn raw_output(backend: Backend, src: &str) -> String {
    match backend {
        Backend::Spa => spa_js(src),
        Backend::Ssg => ssg_html(src),
        Backend::Template => template_html(src),
    }
}

/// Raw output of one backend for a source under a file name; a `.wfx`
/// source reaches the template engine as its braced spelling, as
/// `Template::from_file` reads it.
pub(crate) fn raw_output_as(backend: Backend, src: &str, file: &str) -> String {
    match backend {
        Backend::Spa => spa_js_as(src, file),
        Backend::Ssg => ssg_html_as(src, file),
        Backend::Template => {
            let braced = if file.ends_with(".wfx") {
                webfluent::syntax::layout::to_braces(src, file).expect("layout")
            } else {
                src.to_string()
            };
            template_html(&braced)
        }
    }
}

/// Every element one backend emits for `src`, in document order.
pub(crate) fn elems(backend: Backend, src: &str) -> Vec<Elem> {
    match backend {
        Backend::Spa => parse_spa(&spa_js(src)),
        Backend::Ssg => parse_html(&strip_document(&ssg_html(src))),
        Backend::Template => parse_html(&template_html(src)),
    }
}

/// The first element a backend emits — the root of the component under test.
/// The raw output of one backend for a source, for substring checks.
pub(crate) fn render(backend: Backend, src: &str) -> String {
    match backend {
        Backend::Spa => spa_js(src),
        Backend::Ssg => ssg_html(src),
        Backend::Template => template_html(src),
    }
}

pub(crate) fn root(backend: Backend, src: &str) -> Option<Elem> {
    elems(backend, src).into_iter().next()
}

/// The first element carrying `class`, skipping structural wrappers.
pub(crate) fn first_with_class(backend: Backend, src: &str, class: &str) -> Option<Elem> {
    elems(backend, src).into_iter().find(|e| e.has_class(class))
}

/// Drop the SSG document shell so only the page body remains.
///
/// The shell is `<div id="app">` wrapping a `<main id="wf-main">` landmark; both
/// are structure the renderer supplies, not the component under test.
fn strip_document(html: &str) -> String {
    let body = match html.split_once("<div id=\"app\">") {
        Some((_, rest)) => rest
            .rsplit_once("</div>")
            .map_or_else(|| rest.to_string(), |(body, _)| body.to_string()),
        None => html.to_string(),
    };
    match body.split_once("<main id=\"wf-main\">") {
        Some((_, rest)) => rest
            .rsplit_once("</main>")
            .map(|(inner, _)| inner.to_string())
            .unwrap_or(rest.to_string()),
        None => body,
    }
}

// ─── Output parsing ─────────────────────────────────────────────────────

/// Pull `WF.el("tag", { … })` calls out of generated JS.
///
/// The codegen emits one call per element, so scanning for the constructor is
/// enough to recover the element list without running the bundle.
fn parse_spa(js: &str) -> Vec<Elem> {
    let mut out = Vec::new();
    // Only look past the embedded runtime, which contains its own `WF.el` uses.
    let body = js.split("function Page_").nth(1).unwrap_or(js);
    let body = match body.find("function Component_") {
        Some(_) => body,
        None => body,
    };

    let mut rest = body;
    while let Some(i) = rest.find("WF.el(\"") {
        let after = &rest[i + "WF.el(\"".len()..];
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
    let Some(prop) = split_top_level_commas(inner).into_iter().find(|p| {
        p.trim_start()
            .trim_start_matches('"')
            .starts_with("className")
    }) else {
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
        let Some((k, v)) = part.split_once(':') else {
            continue;
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
            raw: format!("<{inner}>"),
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
        while i < bytes.len()
            && bytes[i] != b'='
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'>'
        {
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
    s.split_whitespace()
        .map(std::string::ToString::to_string)
        .collect()
}

// ─── Reference data ─────────────────────────────────────────────────────

/// HTML void elements — must never be given children or a closing tag.
pub(crate) const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Every modifier the parser accepts, from the single-source vocabulary table.
/// Every class word a flag or case of the registry can produce: what the
/// stylesheet's `.wf-x--word` rules must be keyed on to be reachable.
pub(crate) fn all_modifiers() -> Vec<&'static str> {
    use webfluent::registry::{self, Legacy, PropType};
    let mut words = Vec::new();
    for sig in registry::COMPONENTS {
        for prop in sig.all_props() {
            match prop.ty {
                PropType::Enum(cases) => {
                    words.extend(
                        cases
                            .iter()
                            .filter(|c| !c.legacy.is_empty())
                            .map(|c| c.legacy),
                    );
                }
                _ => {
                    if let Legacy::Modifier(w) = prop.legacy {
                        words.push(w);
                    }
                }
            }
        }
    }
    // Words the original grammar had that the registry retired, whose
    // rules the stylesheet still carries for a project's own `class:`.
    words.extend(registry::RETIRED_MODIFIERS.iter().copied());
    words.sort_unstable();
    words.dedup();
    words
}

/// The full stylesheet a default build ships.
pub(crate) fn built_in_css() -> String {
    webfluent::codegen::generate_css(&webfluent::themes::tokens::default_tokens())
}

/// Whether the built-in stylesheet defines a rule for `class`.
pub(crate) fn css_defines_class(css: &str, class: &str) -> bool {
    let needle = format!(".{class}");
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

// ─── The documentation, read from the site's pages ──────────────────────
//
// The guide is the documentation site's own pages: `site/src/pages/` holds a
// `page` per chapter, recipe and cheat sheet, its words in a few elements —
// `Heading`, `Markdown(…)`, `DocsCodeBlock`, `DocsCallout`, `Table`,
// `Terminal`, `Quiz` — inside a shell (`DocsShell`, `RecipeShell`, …). The
// compiler's parser reads each page, and [`guide_pages`] writes it back as the
// Markdown it shows, so a check reads a page as a reader does: its headings,
// its prose, its links, and each code block as a fence carrying what the
// block says of itself (`DocsCodeBlock(code, expect: "T04")` is
// ```` ```wf expect=T04 ````, `check: false` is `no-check`). Each code's page
// is the compiler's explanation of it (`src/diagnostics/explain/CODE.md`).

/// A page of the documentation: its name (its address without the leading
/// slash: `learn/07-state-and-reactivity`, `errors/T05`, `recipes`), the
/// file it is written in, and its text — a front matter of what the page
/// says of itself (`route`, `title`, `description`, `kind`, `group`,
/// `blurb`, `learned`, `lang`, and `chapter` for a numbered chapter), then
/// `# Title`, then its Markdown.
#[derive(Debug, Clone)]
pub(crate) struct DocPage {
    pub name: String,
    pub file: String,
    pub text: String,
    /// Every `#anchor` the page has: each element's `id:` (a `Heading`'s,
    /// its shell's own sections') and each heading of its Markdown.
    pub anchors: Vec<String>,
}

/// Every page of the documentation, by name: the site's pages of words and
/// each code's page.
pub(crate) fn doc_pages() -> Vec<DocPage> {
    static PAGES: std::sync::OnceLock<Vec<DocPage>> = std::sync::OnceLock::new();
    PAGES.get_or_init(read_doc_pages).clone()
}

/// Every page of the guide, `(name, text)`, in name order — as
/// [`doc_pages`].
pub(crate) fn guide_pages() -> Vec<(String, String)> {
    doc_pages().into_iter().map(|p| (p.name, p.text)).collect()
}

/// A value of a guide page's front matter (`chapter: 12`, `route: "/learn/…"`),
/// a quoted one read as the JSON string it is.
pub(crate) fn front_matter(text: &str, key: &str) -> Option<String> {
    let front = text.strip_prefix("---\n")?.split_once("\n---\n")?.0;
    front.lines().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        (k.trim() == key).then(|| {
            let v = v.trim();
            serde_json::from_str::<String>(v).unwrap_or_else(|_| v.to_string())
        })
    })
}

/// The guide's chapters — the pages whose front matter numbers them — as
/// `(number, name, text)`, in chapter order.
pub(crate) fn guide_chapters() -> Vec<(u32, String, String)> {
    let mut out: Vec<(u32, String, String)> = guide_pages()
        .into_iter()
        .filter_map(|(name, text)| {
            let n = front_matter(&text, "chapter")?.parse().ok()?;
            Some((n, name, text))
        })
        .collect();
    out.sort();
    out
}

/// The site's translations in `lang`, by key.
fn translations(lang: &str) -> HashMap<String, String> {
    let path = repo().join(format!("site/src/translations/{lang}.json"));
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn repo() -> &'static std::path::Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.wf` file under `dir`, in path order.
pub(crate) fn wf_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "wf") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// What the page writer needs beside a page: the translations it reads
/// `t("key")` in, and the components of words a page places
/// (a shell's own parts).
struct Writer {
    lang: HashMap<String, String>,
    en: HashMap<String, String>,
    components: HashMap<String, Vec<Statement>>,
}

impl Writer {
    fn text(&self, e: &Expr) -> String {
        match e {
            Expr::StringLiteral(s) => s.clone(),
            Expr::InterpolatedString(parts) => parts
                .iter()
                .map(|p| match p {
                    StringPart::Literal(s) => s.clone(),
                    _ => "{…}".to_string(),
                })
                .collect(),
            Expr::FunctionCall(f, args) if f == "t" => {
                let key = args.first().map(|a| self.text(a)).unwrap_or_default();
                self.lang
                    .get(&key)
                    .or_else(|| self.en.get(&key))
                    .cloned()
                    .unwrap_or(key)
            }
            Expr::NumberLiteral(n) => n.to_string(),
            _ => String::new(),
        }
    }

    fn pos(&self, el: &UIElement, i: usize) -> Option<String> {
        el.args
            .iter()
            .filter_map(|a| match a {
                Arg::Positional(e) => Some(e),
                Arg::Named(..) => None,
            })
            .nth(i)
            .map(|e| self.text(e))
    }

    fn markdown(&self, body: &[Statement], out: &mut Vec<String>) {
        for st in body {
            match &st.kind {
                StatementKind::UIElement(el) => self.element(el, out),
                StatementKind::If(i) => {
                    self.markdown(&i.then_body, out);
                    for (_, b) in &i.else_if_branches {
                        self.markdown(b, out);
                    }
                    if let Some(b) = &i.else_body {
                        self.markdown(b, out);
                    }
                }
                StatementKind::For(f) => self.markdown(&f.body, out),
                StatementKind::Show(s) => self.markdown(&s.body, out),
                _ => {}
            }
        }
    }

    fn element(&self, el: &UIElement, out: &mut Vec<String>) {
        let name = match &el.component {
            ComponentRef::BuiltIn(n) | ComponentRef::UserDefined(n) => n.clone(),
            ComponentRef::SubComponent(a, b) => format!("{a}.{b}"),
        };
        match name.as_str() {
            "Heading" => {
                let level = el
                    .modifiers
                    .iter()
                    .find_map(|m| m.strip_prefix('h').and_then(|n| n.parse::<usize>().ok()))
                    .unwrap_or(2);
                out.push(format!(
                    "{} {}",
                    "#".repeat(level),
                    self.pos(el, 0).unwrap_or_default()
                ));
            }
            "Markdown" => out.push(self.pos(el, 0).unwrap_or_default()),
            "DocsCodeBlock" => {
                let title =
                    arg(&el.args, "title").map_or_else(|| "wf".to_string(), |e| self.text(e));
                let lang = match title.as_str() {
                    "markdown" => "md".to_string(),
                    t => t.rsplit('.').next().unwrap_or(t).to_string(),
                };
                let mut info = lang;
                if let Some(code) = arg(&el.args, "expect").map(|e| self.text(e)) {
                    info.push_str(&format!(" expect={code}"));
                }
                if el
                    .args
                    .iter()
                    .any(|a| matches!(a, Arg::Named(k, Expr::BoolLiteral(false)) if k == "check"))
                {
                    info.push_str(" no-check");
                }
                out.push(fenced(&info, &self.pos(el, 0).unwrap_or_default()));
            }
            "Terminal" => {
                let mut lines = Vec::new();
                for a in &el.args {
                    if let Arg::Named(k, Expr::ListLiteral(items)) = a
                        && k == "lines"
                    {
                        for item in items {
                            if let Expr::Record(_, fields) = item
                                && let Some((_, c)) = fields.iter().find(|(k, _)| k == "command")
                            {
                                lines.push(self.text(c));
                            }
                        }
                    }
                }
                out.push(fenced("bash", &lines.join("\n")));
            }
            "DocsCallout" => {
                let kind = el
                    .args
                    .iter()
                    .find_map(|a| match a {
                        Arg::Named(k, Expr::EnumCase(c)) if k == "kind" => Some(c.to_uppercase()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "NOTE".to_string());
                let mut text = format!("> [!{kind}]");
                for l in self.pos(el, 0).unwrap_or_default().lines() {
                    text.push_str(&format!("\n> {l}"));
                }
                out.push(text);
            }
            "Quiz" => {
                let mut lines = vec![self.pos(el, 0).unwrap_or_default()];
                for a in &el.args {
                    if let Arg::Named(k, Expr::ListLiteral(items)) = a
                        && k == "choices"
                    {
                        for item in items {
                            let Expr::Record(_, f) = item else { continue };
                            let get = |n: &str| f.iter().find(|(k, _)| k == n).map(|(_, v)| v);
                            let right = matches!(get("right"), Some(Expr::BoolLiteral(true)));
                            let why = get("why").map(|v| self.text(v)).unwrap_or_default();
                            let text = get("text").map(|v| self.text(v)).unwrap_or_default();
                            let why = if why.is_empty() {
                                String::new()
                            } else {
                                format!(" -- {why}")
                            };
                            lines.push(format!("{} {text}{why}", if right { '+' } else { '-' }));
                        }
                    }
                }
                out.push(fenced("quiz", &lines.join("\n")));
            }
            "Table" => {
                let mut rows: Vec<Vec<String>> = Vec::new();
                self.rows(&el.children, &mut rows);
                if let Some(head) = rows.first() {
                    let mut t = format!("| {} |\n|{}", head.join(" | "), "---|".repeat(head.len()));
                    for r in &rows[1..] {
                        t.push_str(&format!("\n| {} |", r.join(" | ")));
                    }
                    out.push(t);
                }
            }
            "Text" | "Paragraph" if self.pos(el, 0).is_some() => {
                out.push(self.pos(el, 0).unwrap_or_default());
            }
            other => {
                if let Some(body) = self.components.get(other) {
                    self.markdown(body, out);
                }
                for fill in &el.slot_fills {
                    self.markdown(&fill.body, out);
                }
                self.markdown(&el.children, out);
            }
        }
    }

    /// Every `id:` an element in `body` is given, and in the components it
    /// places (a shell's own sections), as far as `depth` goes.
    fn ids(&self, body: &[Statement], out: &mut Vec<String>, depth: usize) {
        for st in body {
            match &st.kind {
                StatementKind::UIElement(el) => {
                    if let Some(id) = arg(&el.args, "id").map(|e| self.text(e))
                        && !id.is_empty()
                    {
                        out.push(id);
                    }
                    if let ComponentRef::UserDefined(n) = &el.component
                        && depth > 0
                        && let Some(b) = self.components.get(n)
                    {
                        self.ids(b, out, depth - 1);
                    }
                    for f in &el.slot_fills {
                        self.ids(&f.body, out, depth);
                    }
                    self.ids(&el.children, out, depth);
                }
                StatementKind::If(i) => {
                    self.ids(&i.then_body, out, depth);
                    if let Some(b) = &i.else_body {
                        self.ids(b, out, depth);
                    }
                }
                StatementKind::For(f) => self.ids(&f.body, out, depth),
                _ => {}
            }
        }
    }

    fn rows(&self, body: &[Statement], rows: &mut Vec<Vec<String>>) {
        for st in body {
            let StatementKind::UIElement(el) = &st.kind else {
                continue;
            };
            if matches!(&el.component, ComponentRef::SubComponent(a, b) if a == "Table" && b == "Row")
            {
                let cells = el
                    .children
                    .iter()
                    .filter_map(|c| match &c.kind {
                        StatementKind::UIElement(cell) => {
                            Some(self.pos(cell, 0).unwrap_or_else(|| {
                                let mut inner = Vec::new();
                                self.markdown(&cell.children, &mut inner);
                                inner.join(" ")
                            }))
                        }
                        _ => None,
                    })
                    .map(|t| t.replace('\n', " ").replace('|', "\\|"))
                    .collect();
                rows.push(cells);
            } else {
                self.rows(&el.children, rows);
            }
        }
    }
}

/// The anchor a heading gets on the site: its letters and digits in any
/// script, lower-cased, a combining mark kept with its letter, the rest a
/// dash — as `scripts/docsite/sitepages.py` and the compiler make it.
pub(crate) fn slug(text: &str) -> String {
    let text = text.replace('`', "").to_lowercase();
    let mut out = String::new();
    let mut dash = false;
    for c in text.chars() {
        if matches!(c, '\u{0300}'..='\u{036F}' | '\u{0610}'..='\u{061A}' | '\u{064B}'..='\u{065F}' | '\u{0670}')
        {
            continue;
        }
        if c.is_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// The anchors of a text's `##` to `####` headings, outside its code.
fn markdown_anchors(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
        } else if !fenced
            && let Some(h) = ["## ", "### ", "#### "]
                .iter()
                .find_map(|p| line.strip_prefix(p))
        {
            out.push(slug(h.trim()));
        }
    }
    out
}

fn fenced(info: &str, code: &str) -> String {
    let ticks = if code.contains("```") { "````" } else { "```" };
    format!("{ticks}{info}\n{code}\n{ticks}")
}

/// A front matter line: `key: "value"`.
fn meta(key: &str, value: &str) -> String {
    format!("{key}: {}\n", serde_json::to_string(value).unwrap())
}

fn arg<'a>(args: &'a [Arg], key: &str) -> Option<&'a Expr> {
    args.iter().find_map(|a| match a {
        Arg::Named(k, e) if k == key => Some(e),
        _ => None,
    })
}

fn read_doc_pages() -> Vec<DocPage> {
    let pages_dir = repo().join("site/src/pages");
    let mut components: HashMap<String, Vec<Statement>> = HashMap::new();
    let mut programs = Vec::new();
    for path in wf_files(&repo().join("site/src")) {
        let rel = path.strip_prefix(repo()).unwrap().display().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        let program = webfluent::parse_source(&text, &rel)
            .unwrap_or_else(|e| panic!("{rel} does not parse: {e}"));
        for d in &program.declarations {
            if let Declaration::Component(c) = d {
                components.insert(c.name.clone(), c.body.clone());
            }
        }
        // The pages of words: the guide's chapters, the recipes, the cheat
        // sheets, the reference's pages of words, and each built-in's page
        // (its example is `scripts/docsite/examples.py`'s). A command's
        // page is the CLI's words, read below, and the error pages are
        // each code's explanation, read below too.
        let words = ["guide", "recipes", "comingfrom"]
            .iter()
            .any(|d| path.starts_with(pages_dir.join(d)))
            || ["Builtins.wf", "Cli.wf", "Config.wf", "Tokens.wf"]
                .iter()
                .any(|f| path == pages_dir.join("reference").join(f));
        if words {
            programs.push((path, rel, program));
        }
    }
    let en = translations("en");
    let mut out = Vec::new();
    for (path, rel, program) in &programs {
        for d in &program.declarations {
            let Declaration::Page(page) = d else { continue };
            if page.redirect.is_some() || page.path.contains(':') {
                continue;
            }
            let lang = page.lang.clone().unwrap_or_else(|| "en".to_string());
            let w = Writer {
                lang: translations(&lang),
                en: en.clone(),
                components: components.clone(),
            };
            // The shell: the page's layout, or the first element of its body.
            let mut front = meta("route", &page.path);
            let mut shell_args: Vec<Arg> = Vec::new();
            let mut shell_name = String::new();
            let mut lines = Vec::new();
            let mut title = page.title.clone().unwrap_or_default();
            if let Some(layout) = &page.layout {
                shell_name = layout.name.clone();
                shell_args = layout.args.clone();
                // A `DocsShell` page draws its words in one `Stack`.
                w.markdown(&page.body, &mut lines);
            } else if let Some(el) = page.body.iter().find_map(|s| match &s.kind {
                StatementKind::UIElement(el)
                    if matches!(&el.component, ComponentRef::UserDefined(n) if n.ends_with("Shell")) =>
                {
                    Some(el)
                }
                _ => None,
            }) {
                if let ComponentRef::UserDefined(n) = &el.component {
                    shell_name = n.clone();
                }
                shell_args = el.args.clone();
                for fill in ["code", "differences", "port", "notes"] {
                    if let Some(f) = el.slot_fills.iter().find(|f| f.name == fill) {
                        w.markdown(&f.body, &mut lines);
                    }
                }
                // A recipe's variations come after its words, as the shell draws them.
                let how = !el
                    .args
                    .iter()
                    .any(|a| matches!(a, Arg::Named(k, Expr::BoolLiteral(false)) if k == "how"));
                if shell_name == "RecipeShell" && how {
                    lines.push("## How it works".to_string());
                }
                w.markdown(&el.children, &mut lines);
                if let Some(f) = el.slot_fills.iter().find(|f| f.name == "variations") {
                    lines.push("## Variations".to_string());
                    w.markdown(&f.body, &mut lines);
                }
            } else {
                w.markdown(&page.body, &mut lines);
            }
            if let Some(Arg::Positional(e)) = shell_args.first() {
                let t = w.text(e);
                if !t.is_empty() {
                    title = t;
                }
            }
            let kind = match shell_name.as_str() {
                "DocsShell" => arg(&shell_args, "kind")
                    .and_then(|e| match e {
                        Expr::EnumCase(c) => Some(c.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "tutorial".to_string()),
                "RecipeShell" => "howto".to_string(),
                "ComingFromShell" => "comingfrom".to_string(),
                "ReferenceItemShell" => "reference".to_string(),
                _ => "page".to_string(),
            };
            front.push_str(&meta("kind", &kind));
            front.push_str(&meta("title", &title));
            front.push_str(&meta(
                "description",
                page.description.as_deref().unwrap_or(""),
            ));
            front.push_str(&meta("lang", &lang));
            if let Some(Expr::FunctionCall(f, a)) = arg(&shell_args, "chapter")
                && f == "t"
                && let Some(key) = a.first().map(|k| w.text(k))
            {
                front.push_str(&meta("group", key.trim_start_matches("nav.group.")));
            }
            if let Some(blurb) = arg(&shell_args, "blurb") {
                front.push_str(&meta("blurb", &w.text(blurb)));
            }
            if let Some(Expr::ListLiteral(items)) = arg(&shell_args, "learned") {
                let items: Vec<String> = items.iter().map(|i| w.text(i)).collect();
                front.push_str(&format!(
                    "learned: {}\n",
                    serde_json::to_string(&items).unwrap()
                ));
            }
            // A chapter is a page under `guide/` numbered by its file
            // (`07-state-and-reactivity.wf`), but a translation.
            let stem = path.file_name().unwrap().to_string_lossy().to_string();
            if rel.starts_with("site/src/pages/guide/")
                && page.lang.is_none()
                && let Some(n) = stem.get(..2).and_then(|n| n.parse::<u32>().ok())
            {
                front.push_str(&format!("chapter: {n}\n"));
            }
            let lines: Vec<String> = lines.into_iter().filter(|l| !l.trim().is_empty()).collect();
            let text = format!("---\n{front}---\n# {title}\n\n{}\n", lines.join("\n\n"));
            let mut anchors = Vec::new();
            w.ids(&page.body, &mut anchors, 2);
            if let Some(layout) = &page.layout
                && let Some(b) = components.get(&layout.name)
            {
                w.ids(b, &mut anchors, 1);
            }
            anchors.extend(markdown_anchors(&text));
            out.push(DocPage {
                name: page.path.trim_start_matches('/').to_string(),
                file: rel.clone(),
                text,
                anchors,
            });
        }
    }
    // The command line reference is one page of words: the index's, then
    // each command's page, under its name.
    let (commands, rest): (Vec<DocPage>, Vec<DocPage>) = out
        .into_iter()
        .partition(|p| p.name.starts_with("reference/cli/"));
    out = rest;
    if let Some(cli) = out.iter_mut().find(|p| p.name == "reference/cli") {
        for c in commands {
            let body = c
                .text
                .split_once("\n---\n")
                .map_or(c.text.as_str(), |(_, b)| b);
            let body = body.split_once('\n').map_or("", |(_, b)| b);
            let title = front_matter(&c.text, "title").unwrap_or_default();
            cli.text.push_str(&format!("\n### `{title}`\n{body}"));
        }
    }
    // Each code's page: the compiler's explanation of it.
    for c in webfluent::diagnostics::codes::CODES {
        if let Some(source) = webfluent::diagnostics::codes::explain_source(c.code) {
            let (front, body) = source
                .strip_prefix("---\n")
                .and_then(|r| r.split_once("\n---\n"))
                .unwrap_or(("", source));
            let text = format!(
                "---\n{front}\n{}{}---\n{body}",
                meta("route", &format!("/errors/{}", c.code)),
                meta("kind", "error")
            );
            // The error page's sections, as `ErrorShell` names them.
            let mut anchors: Vec<String> = ["printed", "fix", "why"].map(String::from).to_vec();
            anchors.extend(markdown_anchors(&text));
            out.push(DocPage {
                name: format!("errors/{}", c.code),
                file: format!(
                    "{}/{}.md",
                    webfluent::diagnostics::codes::EXPLAIN_DIR,
                    c.code
                ),
                text,
                anchors,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// What a code fence in the guide says about its block: ```` ```wf expect=T04 ````.
///
/// The first word is the language; each word after it is one of
/// `page` (a whole page: built, previewed and run), `live` (the reader may
/// edit it), `no-check` (not compiled; the prose must say why),
/// `expect=CODE` (it must draw that code), `fixed` (the corrected version of
/// the `expect=` block before it: it must draw none of that code, and
/// nothing else), `break="from:to"` (what *Break it* swaps) and `files=a,b`
/// (the files of a multi-file example this block belongs to). A site
/// page's `DocsCodeBlock` is written as a fence with `expect=` and
/// `no-check` ([`doc_pages`]); a code's explanation
/// (`src/diagnostics/explain/CODE.md`) writes its fences by hand, `fixed`
/// among them.
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is a word a fence may carry"
)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Fence {
    pub lang: String,
    pub page: bool,
    pub live: bool,
    pub no_check: bool,
    pub expect: Option<String>,
    pub fixed: bool,
    pub swap: Option<(String, String)>,
    pub files: Vec<String>,
}

impl Fence {
    /// A block the compiler is held to as an ordinary sample: neither one
    /// that must fail nor one marked not to be checked.
    pub(crate) fn is_sample(&self) -> bool {
        self.expect.is_none() && !self.no_check
    }
}

/// The fence `line` opens, if it opens one: `None` for a line that is not a
/// fence or that closes one, `Some(Err(…))` for a word no fence takes.
pub(crate) fn fence(line: &str) -> Option<Result<Fence, String>> {
    let info = line.trim_end().strip_prefix("```")?.trim_start_matches('`');
    if info.is_empty() {
        return None;
    }
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    for c in info.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            c => word.push(c),
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    let mut out = Fence {
        lang: words.remove(0),
        ..Fence::default()
    };
    for w in words {
        match w.split_once('=') {
            None if w == "page" => out.page = true,
            None if w == "live" => out.live = true,
            None if w == "no-check" => out.no_check = true,
            None if w == "fixed" => out.fixed = true,
            Some(("expect", code)) if !code.is_empty() => out.expect = Some(code.to_string()),
            Some(("break", swap)) => match swap.split_once(':') {
                Some((from, to)) if !from.is_empty() => {
                    out.swap = Some((from.to_string(), to.to_string()));
                }
                _ => return Some(Err(format!("`break=` takes \"from:to\", not `{swap}`"))),
            },
            Some(("files", list)) => {
                out.files = list.split(',').map(str::to_string).collect();
            }
            _ => return Some(Err(format!("`{w}` is no word a fence takes"))),
        }
    }
    Some(Ok(out))
}
