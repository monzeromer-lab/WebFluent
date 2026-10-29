//! Running a project's tests: every `test "…" { … }` under `tests/` (and in
//! `src/`), rendered through the template engine over its data, held to what
//! it expects and to its snapshot; and a test that acts — or one that looks
//! at a `Portal`, whose block only a running page draws — built as a site of
//! one page and played in a browser.
//!
//! A test's body renders with the project's components, stores, types,
//! enums and constants at hand, as a page's would; a `state` in the body
//! is seeded with its initial value, and `data: { … }` supplies the rest
//! by name. The rendered fragment must contain every `expect "text"` and
//! none of the `expect not "text"`; it is then compared to
//! `tests/__snapshots__/<file>/<test>.html`, written when missing and
//! rewritten with `update`. A `t("key")` reads the project's messages in
//! its default locale, as the site does.
//!
//! `wf test` prints what this finds; the mobile editor shows it. The
//! browser a test that acts needs is theirs: Chrome for `wf test`, the
//! editor's WebView ([the `Browser` trait](crate::testing::Browser)).

use std::collections::HashMap;
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
    /// The project's own scripts, linked on every page a test that acts
    /// builds as a site links them, so a test that clicks runs the code
    /// they define.
    pub scripts: Vec<crate::project_js::Script>,
    /// The project's translations, so `t("key")` in a test reads the same
    /// messages the site does.
    pub i18n: Option<I18n>,
    /// The project's own styles, which a test that acts is drawn with as
    /// the built page is.
    pub styles: Styles,
}

/// What a project styles its pages with beyond the theme and its `style { }`
/// blocks, for a test's page to be drawn as the built one is.
#[derive(Debug, Clone, Default)]
pub struct Styles {
    /// Every `.css` file under `src/`, bundled as the build bundles them
    /// into `styles.css`.
    pub css: String,
    /// The stylesheets `meta.stylesheets` links, by URL or site-relative
    /// path, with the `meta.integrity` of any that has one.
    pub linked: Vec<String>,
    pub integrity: HashMap<String, String>,
    /// The project's `public/` folder, when it has one: what a site-relative
    /// stylesheet, or a font a stylesheet names, is served from.
    pub public: Option<PathBuf>,
}

/// A project's `i18n` settings and every locale's messages.
pub type I18n = (
    crate::config::project::I18nConfig,
    HashMap<String, HashMap<String, String>>,
);

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
    // A file's project is the nearest folder above it with a
    // `webfluent.app.json` — `wf test tests/cart.wf` runs that file's tests
    // in the project it belongs to.
    let project_dir = if path.is_file() {
        let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        absolute
            .ancestors()
            .skip(1)
            .find(|dir| dir.join("webfluent.app.json").is_file())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.parent().unwrap_or(Path::new(".")).to_path_buf())
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
    let mut shared: Vec<Declaration> = declarations
        .iter()
        .filter(|d| {
            !matches!(
                d,
                Declaration::Page(_) | Declaration::App(_) | Declaration::Test(_)
            )
        })
        .cloned()
        .collect();
    // A page a test can place by its name, as it places a component: `Tasks`
    // draws the Tasks page's body — its state, its stores, its handlers — so
    // a test can click through a page as a reader does. A page that takes a
    // route parameter, or whose name a component has, is left out.
    let components: Vec<String> = shared
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();
    for d in &declarations {
        if let Declaration::Page(page) = d
            && page.params.is_empty()
            && !components.contains(&page.name)
        {
            shared.push(Declaration::Component(crate::parser::ast::ComponentDecl {
                name: page.name.clone(),
                props: Vec::new(),
                events: Vec::new(),
                slots: Vec::new(),
                parts: Vec::new(),
                doc: None,
                body: page.body.clone(),
                span: page.span,
                header_span: page.header_span,
                body_span: page.body_span,
            }));
        }
    }
    let config = crate::config::ProjectConfig::load(&project_dir).ok();
    // The project's translations: a test draws `t("key")` as the site does,
    // in its default locale.
    let i18n = config.as_ref().and_then(|c| c.i18n.clone()).and_then(|i| {
        crate::i18n::load(&project_dir, &i)
            .ok()
            .map(|tables| (i, tables))
    });
    let scripts =
        crate::project_js::load(&project_dir, &project_dir.join("src")).unwrap_or_default();
    // Absolute: the test's site links to it from somewhere else.
    let public = project_dir.join("public");
    let public = public.canonicalize().unwrap_or(public);
    let styles = Styles {
        css: crate::codegen::project_css::bundle(&project_dir, &project_dir.join("src"))?,
        linked: config
            .as_ref()
            .map(|c| c.meta.stylesheets.clone())
            .unwrap_or_default(),
        integrity: config
            .as_ref()
            .map(|c| c.meta.integrity.clone())
            .unwrap_or_default(),
        public: public.is_dir().then_some(public),
    };
    Ok(Tests {
        shared,
        tests,
        snapshots: project_dir.join("tests").join("__snapshots__"),
        theme: config.map(|c| c.theme).unwrap_or_default(),
        scripts,
        i18n,
        styles,
    })
}

impl Tests {
    /// How many of the tests are run in a browser ([`Tests::in_browser`]).
    pub fn acting(&self) -> usize {
        self.tests
            .iter()
            .filter(|(_, t)| self.in_browser(t))
            .count()
    }

    /// Whether `test` is run in a browser rather than rendered: it acts —
    /// a click is not something a renderer can do — or what it draws holds
    /// a `Portal`, whose block only a running page draws.
    pub fn in_browser(&self, test: &TestDecl) -> bool {
        runs_in_browser(test, &self.shared)
    }

    /// Run a test that only looks: render it, hold it to its expects and its
    /// snapshot (written when missing, or with `update`).
    pub fn run_rendered(&self, file: &str, test: &TestDecl, update: bool) -> Outcome {
        let messages = self.messages();
        let locale = self.locale();
        match run_one(
            &self.shared,
            test,
            &self.snapshots,
            file,
            update,
            &messages,
            &locale,
        ) {
            Ok(written) if written => Outcome::Written,
            Ok(_) => Outcome::Passed,
            Err(reasons) => Outcome::Failed(reasons),
        }
    }

    /// Run a test that acts in `browser`, its site built under `work`.
    pub fn run_acting(&self, browser: &mut dyn Browser, test: &TestDecl, work: &Path) -> Outcome {
        let site = Site {
            theme: &self.theme,
            scripts: &self.scripts,
            i18n: self.i18n.as_ref(),
            styles: &self.styles,
        };
        match act(browser, &self.shared, test, work, &site) {
            Ok(Ok(())) => Outcome::Passed,
            Ok(Err(reasons)) => Outcome::Failed(reasons),
            Err(e) => Outcome::Failed(vec![e.to_string()]),
        }
    }

    /// The project's default locale, whose plural rules a `t()` with a
    /// `count` follows; English without translations.
    pub fn locale(&self) -> String {
        self.i18n
            .as_ref()
            .map(|(i, _)| i.default_locale.clone())
            .unwrap_or_else(|| "en".to_string())
    }

    /// The messages of the default locale, which a test that looks reads.
    pub fn messages(&self) -> HashMap<String, String> {
        self.i18n
            .as_ref()
            .and_then(|(i, tables)| tables.get(&i.default_locale).cloned())
            .unwrap_or_default()
    }
}

/// Whether `test` is run in a browser, with `declarations` the components
/// it may place: it acts, or what it draws holds a `Portal`.
pub fn runs_in_browser(test: &TestDecl, declarations: &[Declaration]) -> bool {
    test.acts() || draws_portal(&test.body, declarations, &mut Vec::new())
}

/// Whether `body`, or a component it places, draws a `Portal`. A render
/// paints only a portal's place: its block is drawn into a node the running
/// page has, so a test that looks at one is run in a browser.
fn draws_portal<'a>(
    body: &'a [Statement],
    shared: &'a [Declaration],
    seen: &mut Vec<&'a str>,
) -> bool {
    body.iter().any(|stmt| match &stmt.kind {
        StatementKind::UIElement(el) => {
            let own = match &el.component {
                ComponentRef::BuiltIn(name) => name == "Portal",
                ComponentRef::UserDefined(name) if !seen.contains(&name.as_str()) => {
                    seen.push(name);
                    shared.iter().any(|d| {
                        matches!(d, Declaration::Component(c)
                            if c.name == *name && draws_portal(&c.body, shared, seen))
                    })
                }
                _ => false,
            };
            own || draws_portal(&el.children, shared, seen)
                || el
                    .slot_fills
                    .iter()
                    .any(|f| draws_portal(&f.body, shared, seen))
        }
        StatementKind::If(i) => {
            draws_portal(&i.then_body, shared, seen)
                || i.else_if_branches
                    .iter()
                    .any(|(_, b)| draws_portal(b, shared, seen))
                || i.else_body
                    .as_deref()
                    .is_some_and(|b| draws_portal(b, shared, seen))
        }
        StatementKind::For(f) => draws_portal(&f.body, shared, seen),
        StatementKind::Show(s) => draws_portal(&s.body, shared, seen),
        StatementKind::Match(m) => m.arms.iter().any(|a| draws_portal(&a.body, shared, seen)),
        _ => false,
    })
}

/// The rendered fragment of one test: its body as a page over the shared
/// declarations, seeded with its `state` and its `data`.
pub fn render_test(
    shared: &[Declaration],
    test: &TestDecl,
    messages: &HashMap<String, String>,
    locale: &str,
) -> Result<String> {
    let mut declarations = shared.to_vec();
    declarations.push(Declaration::Page(PageDecl {
        name: "__Test".to_string(),
        path: "/__test".to_string(),
        title: None,
        title_expr: None,
        description_expr: None,
        guard: None,
        redirect: None,
        description: None,
        image: None,
        image_alt: None,
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
    crate::template::render_program_fragment_in(&program, &data, messages.clone(), locale)
}

/// The text a render shows, as a browser lays it out and a reader reads
/// it: tags gone, the common entities decoded. An element laid out as a
/// block — a block by its tag, or any element a layout box (`Row`, `Stack`,
/// `Grid`) lays out, as a browser blockifies a flex or grid item — has a
/// space at each edge; an inline one has none, so `<code>live</code><code>:
/// </code>` reads `live:`.
///
/// The render is written for people to read: each element on a line of its
/// own, indented. That formatting is not text — the line break and the
/// indentation the renderer puts before a tag, after one or around an
/// element's own words are dropped — so two inline elements that touch read
/// as touching, as they do on the live page.
fn rendered_text(html: &str) -> String {
    const INLINE: &[&str] = &[
        "a", "abbr", "b", "bdi", "bdo", "code", "em", "i", "kbd", "mark", "q", "s", "small",
        "span", "strong", "sub", "sup", "time", "u",
    ];
    // Elements with no end tag, which open nothing.
    const VOID: &[&str] = &[
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ];
    // The engine's own layout boxes: their children are flex or grid items.
    const LAYOUT: &[&str] = &["wf-row", "wf-stack", "wf-grid"];
    // Each open element: its name, whether it is laid out as a block, and
    // whether it lays its children out.
    let mut open: Vec<(String, bool, bool)> = Vec::new();
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        let at = rest.find('<').unwrap_or(rest.len());
        out.push_str(unformatted(&rest[..at]));
        if at == rest.len() {
            break;
        }
        rest = &rest[at..];
        // A comment says nothing.
        if let Some(after) = rest.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |end| &after[end + 3..]);
            continue;
        }
        let Some(close) = rest.find('>') else {
            break;
        };
        let tag = &rest[1..close];
        rest = &rest[close + 1..];
        let closing = tag.starts_with('/');
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if name.is_empty() {
            continue; // `<!doctype …>`, or a `<` that opens no tag
        }
        if closing {
            if let Some(at) = open.iter().rposition(|(n, ..)| *n == name) {
                if open[at].1 {
                    out.push(' ');
                }
                open.truncate(at);
            }
            continue;
        }
        let laid_out = open.last().is_some_and(|(.., lays_out)| *lays_out);
        let block = laid_out || !INLINE.contains(&name.as_str());
        if block {
            out.push(' ');
        }
        if !VOID.contains(&name.as_str()) && !tag.ends_with('/') {
            let lays_out = class_of(tag)
                .is_some_and(|class| class.split_whitespace().any(|c| LAYOUT.contains(&c)));
            open.push((name, block, lays_out));
        }
    }
    decode_entities(&out)
}

/// A run of text between two tags without the renderer's formatting: the
/// line break and indentation it starts with, and the one it ends with.
/// A run that is only formatting is nothing; a reader's own spaces stay.
fn unformatted(run: &str) -> &str {
    let mut run = run;
    if let Some(after) = run.strip_prefix('\n') {
        run = after.trim_start_matches(' ');
    }
    if let Some(at) = run.rfind('\n')
        && run[at + 1..].bytes().all(|b| b == b' ')
    {
        run = &run[..at];
    }
    run
}

/// The value of a tag's `class` attribute, as the renderers write it.
fn class_of(tag: &str) -> Option<&str> {
    let at = tag.find(" class=\"")? + " class=\"".len();
    let end = tag[at..].find('"')?;
    Some(&tag[at..at + end])
}

/// The text an HTML escape stands for: the named entities the renderers
/// write, and any numeric one, decimal (`&#39;`) or hex (`&#x27;`, how an
/// apostrophe is escaped). One the reader would see as written stays so.
fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let end = tail.find(';').filter(|&e| e <= 10);
        let decoded = end.and_then(|e| {
            let name = &tail[1..e];
            let c = match name {
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "amp" => Some('&'),
                "nbsp" => Some(' '),
                _ => name
                    .strip_prefix("#x")
                    .or_else(|| name.strip_prefix("#X"))
                    .and_then(|h| u32::from_str_radix(h, 16).ok())
                    .or_else(|| name.strip_prefix('#').and_then(|d| d.parse().ok()))
                    .and_then(char::from_u32),
            }?;
            Some((c, e + 1))
        });
        match decoded {
            Some((c, len)) => {
                out.push(c);
                rest = &tail[len..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod entity_tests {
    use super::{decode_entities, rendered_text, squeeze};

    /// What an `expect` reads in a render, white space collapsed.
    fn read(html: &str) -> String {
        squeeze(&rendered_text(html))
    }

    #[test]
    fn inline_elements_that_touch_read_as_touching_whatever_the_formatting() {
        let head = "    <p class=\"wf-text\">\n        <code class=\"wf-code\">derived </code>\n        <code class=\"wf-code\">live</code>\n        <code class=\"wf-code\">: </code>\n        <code class=\"wf-code\">[Deploy]</code>\n    </p>\n";
        assert_eq!(read(head), "derived live: [Deploy]");
        // A word of its own before the children keeps the space it was
        // given, and only that.
        let words = "<p class=\"wf-text\">\n    Hello \n    <strong>world</strong>\n</p>\n<p class=\"wf-text\">\n    Hello\n    <strong>world</strong>\n</p>\n";
        assert_eq!(read(words), "Hello world Helloworld");
        // Blocks are apart, and so is what a layout box lays out: a flex or
        // grid item is a block to the browser, inline by its tag or not.
        let row = "<div class=\"wf-row p-line\">\n    <code class=\"wf-code\">a</code>\n    <code class=\"wf-code\">b</code>\n</div>\n<p>c</p><p>d<br>e</p>";
        assert_eq!(read(row), "a b c d e");
        // Inside a laid-out item, inline is inline again.
        let nested =
            "<div class=\"wf-stack\"><p class=\"x\"><code>b</code>\n<code>ack</code></p></div>";
        assert_eq!(read(nested), "back");
        // A comment and an empty element say nothing; entities are decoded.
        assert_eq!(
            read("<!--wf-portal--><span></span><b>a &amp;</b><i> b</i>"),
            "a & b"
        );
    }

    #[test]
    fn what_an_escape_stands_for_is_what_a_reader_sees() {
        assert_eq!(
            decode_entities("isn&#x27;t &#39;so&#39; &amp; &lt;b&gt; &quot;q&quot; &#X41;&#66;"),
            "isn't 'so' & <b> \"q\" AB"
        );
        assert_eq!(
            decode_entities("a & b &unknown; &#xZZ; &amp;lt;"),
            "a & b &unknown; &#xZZ; &lt;",
            "one it does not know, or not an escape, stays"
        );
        assert_eq!(decode_entities("&#x1F600; ok"), "😀 ok");
    }
}

/// Runs of whitespace as one space, for comparing what text says.
fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One test that looks: `Ok(true)` when it passed and its snapshot was
/// written, `Ok(false)` when it passed against the one there.
fn run_one(
    shared: &[Declaration],
    test: &TestDecl,
    snapshots: &Path,
    file: &str,
    update: bool,
    messages: &HashMap<String, String>,
    locale: &str,
) -> std::result::Result<bool, Vec<String>> {
    let html = render_test(shared, test, messages, locale).map_err(|e| vec![e.to_string()])?;
    let mut reasons = Vec::new();
    let empty = crate::codegen::static_eval::Scope::of([]);
    // What a reader sees: the text, not the markup — `expect "wf-text"` is
    // not satisfied by a class name.
    let visible = rendered_text(&html);
    for step in &test.steps {
        let Step::Expect { text, negated, .. } = step else {
            continue; // a test that acts does not come this way
        };
        let text = crate::codegen::static_eval::eval(text, &empty)
            .map(|v| v.to_text())
            .unwrap_or_default();
        let found = squeeze(&visible).contains(&squeeze(&text));
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

/// What a test's site is built with, beside its program: the project's
/// theme settings, its own scripts, its translations and its styles.
#[derive(Clone, Copy)]
pub struct Site<'a> {
    pub theme: &'a ThemeConfig,
    pub scripts: &'a [crate::project_js::Script],
    pub i18n: Option<&'a I18n>,
    pub styles: &'a Styles,
}

/// Build a test that acts as a site of one page under `work`, open it in
/// `browser`, and play its steps: what went wrong, if anything.
pub fn act(
    browser: &mut dyn Browser,
    shared: &[Declaration],
    test: &TestDecl,
    work: &Path,
    site: &Site,
) -> Result<std::result::Result<(), Vec<String>>> {
    let dir = work.join(slug_acting(&test.name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    // The program as written, with the test's body as its one page. It is
    // compiled here rather than written back out as source: there is no
    // printer from the tree to the text, and a test does not need one — the
    // compiler takes the tree.
    let program = crate::sema::lower(as_program(shared, test));
    write_site(&program, &dir, site)?;
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
            // Read as a rendered test reads it: runs of white space are one
            // space, so `expect "Total tasks 3"` finds a card's title and its
            // number on two lines, in a browser as in a render.
            let shown = visible_text(page)?;
            let found = squeeze(&shown).contains(&squeeze(&want));
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
            let done = page.eval(&step_script("click", &name, ""))?;
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
            let done = page.eval(&step_script("type", &into, &what))?;
            match done.as_str() {
                Some("ok") => {}
                Some("no-option") => {
                    return Ok(Err(format!("{into:?} has no option {what:?}")));
                }
                _ => return Ok(Err(format!("no control called {into:?} to type into"))),
            }
            page.settle(120)?;
        }
        Step::Press { key, target, .. } => {
            let key = text(key);
            let where_ = target.as_ref().map(text).unwrap_or_default();
            let done = page.eval(&step_script("press", &where_, &key))?;
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
pub fn write_site(program: &Program, dir: &Path, site: &Site) -> Result<()> {
    let mut config = crate::config::ProjectConfig::default_config("test");
    config.theme = site.theme.clone();
    config.i18n = site.i18n.map(|(c, _)| c.clone());
    for script in site.scripts {
        let to = dir.join(&script.href);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(to, &script.source)?;
        config.build.scripts.push(script.href.clone());
    }
    // One page, one file: a split build links a chunk that this does not
    // write, and the shell would be served in its place.
    config.build.split = false;
    let mut js = crate::codegen::JsCodegen::new();
    js.set_split_pages(false);
    if let Some((c, tables)) = site.i18n {
        js.set_i18n(c.default_locale.clone(), tables.clone());
    }
    std::fs::write(dir.join("app.js"), js.generate(program))?;

    // The sheet in the build's order: the engine's rules and the theme's
    // tokens, the dark theme's, the project's own stylesheets, then what
    // its `style { }` blocks compile to.
    let tokens = crate::themes::resolve_tokens(program, &config.theme)?;
    let mut css = crate::codegen::generate_css_for(&tokens, config.theme.builtin, program);
    if let Some(dark) = crate::themes::resolve_dark_tokens(program, &config.theme)? {
        css.push_str(&crate::codegen::dark_css(&dark));
    }
    css.push_str(&site.styles.css);
    css.push_str(&crate::codegen::scoped_css::scoped_rules(program));
    std::fs::write(dir.join("styles.css"), css)?;
    // The stylesheets the project links, linked as the built page links
    // them.
    config.meta.stylesheets = site.styles.linked.clone();
    config.meta.integrity = site.styles.integrity.clone();

    std::fs::write(
        dir.join("index.html"),
        crate::codegen::generate_html(&config, program),
    )?;
    // `public/` beside the page, as the build copies it: a site-relative
    // sheet, or a font a sheet names, is found where it is on the site.
    // Last, so nothing written above goes through a link into the project.
    if let Some(public) = &site.styles.public {
        serve_public(public, dir)?;
    }
    Ok(())
}

/// What is in `public/` (an absolute path), reachable from the test's site
/// `dir` as from the built one: each entry linked in, or copied where a
/// link cannot be made.
/// What the test's site wrote itself is its own, as the build's own files
/// are; a folder both have is gone into.
fn serve_public(public: &Path, dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(public)? {
        let from = entry?.path();
        let Some(name) = from.file_name() else {
            continue;
        };
        let to = dir.join(name);
        if to.symlink_metadata().is_ok() {
            if from.is_dir() && to.symlink_metadata()?.is_dir() {
                serve_public(&from, &to)?;
            }
            continue;
        }
        #[cfg(unix)]
        let linked = std::os::unix::fs::symlink(&from, &to).is_ok();
        #[cfg(not(unix))]
        let linked = false;
        if !linked {
            copy_into(&from, &to)?;
        }
    }
    Ok(())
}

fn copy_into(from: &Path, to: &Path) -> Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let path = entry?.path();
            if let Some(name) = path.file_name() {
                copy_into(&path, &to.join(name))?;
            }
        }
    } else {
        std::fs::copy(from, to)?;
    }
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
        .map(|d| match d {
            // `data: { Cart: { items: […] } }` starts the store there, as a
            // test that only looks does.
            Declaration::Store(mut store) => {
                if let Some(Expr::MapLiteral(pairs)) = &test.data
                    && let Some((_, given)) = pairs
                        .iter()
                        .find(|(k, _)| k.trim_matches('"') == store.name)
                {
                    store.body = seeded(&store.body, Some(given));
                }
                Declaration::Store(store)
            }
            other => other,
        })
        .collect();
    declarations.push(Declaration::Page(PageDecl {
        name: "Test".to_string(),
        path: "/".to_string(),
        title: Some(test.name.clone()),
        title_expr: None,
        description_expr: None,
        description: Some("A test.".to_string()),
        guard: None,
        redirect: None,
        image: None,
        image_alt: None,
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
/// The script a step plays in its page: `what` is `"click"`, `"type"` or
/// `"press"`, `name` what a reader calls the thing it acts on (empty for a
/// `press` into what has focus), `value` what is typed or the key. It
/// evaluates to `"ok"`, `"none"` (nothing by that name) or `"no-option"` (a
/// select without that option).
///
/// A [`Page`] that is not Chrome — the editor's own window, playing its
/// scenarios — runs the same steps by evaluating this, so a scenario finds
/// a control exactly as a test does.
pub fn step_script(what: &str, name: &str, value: &str) -> String {
    format!(
        "(() => {{\n{FIND}\n  const what = {}, name = {}, value = {};\n{ACT}\n}})()",
        serde_json::to_string(what).unwrap_or_default(),
        serde_json::to_string(name).unwrap_or_default(),
        serde_json::to_string(value).unwrap_or_default(),
    )
}

const FIND: &str = r#"
  const words = (s) => (s || "").replace(/\s+/g, " ").trim();
  // Its text as a reader hears it: what is hidden from readers (a menu
  // item's shortcut, a glyph) is not part of its name.
  const heard = (el) => {
    let t = "";
    for (const c of el.childNodes) {
      if (c.nodeType === 3) t += c.textContent;
      else if (c.nodeType === 1 && c.getAttribute("aria-hidden") !== "true") t += heard(c);
    }
    return t;
  };
  const named = (el, n) => {
    const label = el.labels && el.labels[0] ? el.labels[0].textContent : "";
    return [
      el.getAttribute("aria-label"),
      label,
      el.getAttribute("placeholder"),
      el.getAttribute("title"),
      el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.tagName === "SELECT"
        ? null
        : heard(el),
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
    // Whatever a reader can press, by its name: a button or a link, a
    // control, or a widget's role — a tab, a menu's item, an option, a row
    // of a tree.
    const el = find(name, "button, a, [role=button], [role=link], [role=tab], [role=menuitem], [role=menuitemcheckbox], [role=menuitemradio], [role=option], [role=treeitem], [role=switch], [role=checkbox], [role=radio], input, select, summary, label, [onclick]");
    if (!el) return "none";
    el.click();
    return "ok";
  }
  if (what === "type") {
    const el = find(name, "input, textarea, select");
    if (!el) return "none";
    el.focus();
    if (el.tagName === "SELECT") {
      // A select is chosen by what the reader sees: an option's text, or
      // failing that its value.
      const opt = [...el.options].find((o) => o.text.trim() === value) ||
        [...el.options].find((o) => o.value === value);
      if (!opt) return "no-option";
      el.value = opt.value;
    } else {
      const setter = Object.getOwnPropertyDescriptor(
        el.tagName === "TEXTAREA" ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype,
        "value",
      );
      if (setter && setter.set) setter.set.call(el, value);
      else el.value = value;
    }
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
    return "ok";
  }
  if (what === "press") {
    const el = name ? find(name, "input, textarea, select, button, a, [tabindex]") : (document.activeElement || document.body);
    if (!el) return "none";
    const key = value;
    const init = { key, code: key.length === 1 ? "Key" + key.toUpperCase() : key, bubbles: true, cancelable: true };
    const down = el.dispatchEvent(new KeyboardEvent("keydown", init));
    el.dispatchEvent(new KeyboardEvent("keyup", init));
    // What the browser does with a real Escape that nothing cancelled: the
    // open modal dialog is asked (`cancel`) and, unless it refuses, closed.
    // A synthetic key event does not reach that default.
    if (down && (key === "Escape" || key === "Esc")) {
      const open = [...document.querySelectorAll("dialog[open]")];
      const dialog = (el.closest && el.closest("dialog[open]")) || open[open.length - 1];
      if (dialog && dialog.dispatchEvent(new Event("cancel", { cancelable: true }))) dialog.close();
    }
    // A real Enter in a form's text field submits the form (its implicit
    // submission); a synthetic one does not, so the step does it.
    if (down && key === "Enter" && el.form && el.tagName === "INPUT"
        && !/^(button|submit|reset|checkbox|radio|file)$/.test(el.type) && el.form.requestSubmit) {
      el.form.requestSubmit();
    }
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
