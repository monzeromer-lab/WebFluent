use crate::codegen::builtin::{
    builtin_to_html, class_list, element_tag, implicit_role, input_type, landmark_label,
    layout_arg_classes,
};
use crate::codegen::css::generate_css;
use crate::config::project::{PdfConfig, SlidesConfig};
use crate::error::{Result, WebFluentError};
use crate::parser::ast::{ArmPattern, ForStmt, IfStmt, PropDecl};
use crate::parser::{
    Arg, ComponentRef, Declaration, Expr, Program, Statement, StatementKind, StringPart, UIElement,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// What a PDF render drew.
#[derive(Debug, Clone)]
pub struct PdfReport {
    /// The PDF file.
    pub bytes: Vec<u8>,
    pub pages: usize,
    /// The text of each page, a line at a time, in reading order.
    pub text: Vec<Vec<String>>,
    /// A character no font had; a font taken from this machine.
    pub notes: Vec<String>,
}

/// A compiled WebFluent template ready for rendering with JSON data.
///
/// `Template` is the primary public API for using WebFluent as a library.
/// It parses `.wf` source code and renders it to HTML or PDF with data substitution.
///
/// # Examples
///
/// ```rust
/// use webfluent::Template;
/// use serde_json::json;
///
/// let tpl = Template::from_str(r##"
///     page Home(path: "/", title: "Hello") {
///         Container { Heading("Hello, {name}!").h1 }
///     }
/// "##).unwrap();
///
/// let html = tpl.render_html(&json!({"name": "World"})).unwrap();
/// assert!(html.contains("Hello, World!"));
/// ```
///
/// # Theming
///
/// Declare a `Theme` in the template. [`with_theme`](Template::with_theme)
/// picks one when the source declares several; [`with_tokens`](Template::with_tokens)
/// layers machine-supplied values on top.
///
/// ```rust
/// # use webfluent::Template;
/// # use serde_json::json;
/// let html = Template::from_str(r##"
///     theme Brand { color-primary: #0F766E }
///     page P(path: "/") { Text("Hi") }
/// "##)
///     .unwrap()
///     .with_tokens(&[("color-secondary", "#8B5CF6")])
///     .render_html(&json!({}))
///     .unwrap();
/// assert!(html.contains("--color-primary: #0F766E"));
/// ```
#[derive(Clone)]
pub struct Template {
    /// The program, parsed, its `data` files read and lowered — once, when
    /// the template is made, and shared by every clone, so a server builds a
    /// template at start-up and renders it on every request.
    program: Arc<Program>,
    /// The page a render draws, when [`page`](Template::page) chose one;
    /// every page of the source otherwise.
    page: Option<String>,
    theme: Option<String>,
    custom_tokens: HashMap<String, String>,
    /// The document's language, for `<html lang>`.
    lang: String,
    pdf: PdfConfig,
    slides: SlidesConfig,
    /// Where the template's files are: what an image's address and the
    /// `fonts/` directory are read from.
    root: Option<std::path::PathBuf>,
}

// `Template::from_str` is documented public API used throughout the README and
// the docs site; it predates any `FromStr` impl and renaming it would break every
// consumer to satisfy a naming lint.
#[allow(clippy::should_implement_trait)]
impl Template {
    /// Create a template from a `.wf` source string.
    ///
    /// The source is parsed and checked immediately — a component nothing
    /// declares, a flag a component does not take, a value of the wrong type
    /// is an error here, not at render time.
    ///
    /// # Errors
    ///
    /// Returns [`WebFluentError::LexerError`] or [`WebFluentError::ParseError`]
    /// if the source is invalid, and [`WebFluentError::CodegenError`] with every
    /// finding if it does not check.
    pub fn from_str(source: &str) -> Result<Self> {
        Self::from_sources(&[("<template>", source)])
    }

    /// Create a template from a `.wf` (or `.wfx`) file on disk. A `data`
    /// declaration's file is read from beside it.
    ///
    /// # Errors
    ///
    /// Returns [`WebFluentError::IoError`] if the file cannot be read,
    /// or a parse error if the content is invalid.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_files(&[path])
    }

    /// One template from several files: every component, theme, type and
    /// constant any of them declares is shared, and each `page` is a
    /// document [`page`](Template::page) can pick by name. `data` files are
    /// read from beside the first.
    ///
    /// ```rust,no_run
    /// # use webfluent::Template;
    /// # use serde_json::json;
    /// let site = Template::from_files(&["templates/components.wf", "templates/invoice.wf"]).unwrap();
    /// let html = site.page("Invoice").unwrap().render_html(&json!({ "number": 7 })).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// As [`from_file`](Template::from_file), naming the file a finding is in.
    pub fn from_files<P: AsRef<Path>>(paths: &[P]) -> Result<Self> {
        let files: Vec<&Path> = paths.iter().map(AsRef::as_ref).collect();
        let root = files.first().and_then(|p| p.parent());
        Self::load(&files, root)
    }

    /// Every `.wf` and `.wfx` file under `dir`, in path order, as one
    /// template — a project's worth of components and pages, loaded once:
    ///
    /// ```rust,no_run
    /// # use webfluent::Template;
    /// # use serde_json::json;
    /// let templates = Template::from_dir("templates").unwrap();
    /// for name in templates.pages() {
    ///     println!("{name}");
    /// }
    /// let receipt = templates.page("Receipt").unwrap();
    /// let pdf = receipt.render_pdf(&json!({ "total": 12 })).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// [`WebFluentError::IoError`] when the directory cannot be read or holds
    /// no template; otherwise as [`from_files`](Template::from_files).
    pub fn from_dir(dir: impl AsRef<Path>) -> Result<Self> {
        fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    walk(&path, out)?;
                } else if path.extension().is_some_and(|e| e == "wf" || e == "wfx") {
                    out.push(path);
                }
            }
            Ok(())
        }
        let dir = dir.as_ref();
        let mut files = Vec::new();
        walk(dir, &mut files).map_err(|e| {
            WebFluentError::IoError(format!("Failed to read '{}': {}", dir.display(), e))
        })?;
        if files.is_empty() {
            return Err(WebFluentError::IoError(format!(
                "no .wf or .wfx template under '{}'",
                dir.display()
            )));
        }
        files.sort();
        let files: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
        // `data` files are read from the directory itself.
        Self::load(&files, Some(dir))
    }

    /// Read `files` (a `.wfx` one through its braced spelling) and build
    /// one template of them, `data` files read from `root`.
    fn load(files: &[&Path], root: Option<&Path>) -> Result<Self> {
        let mut sources = Vec::new();
        for path in files {
            let source = fs::read_to_string(path).map_err(|e| {
                WebFluentError::IoError(format!(
                    "Failed to read template '{}': {}",
                    path.display(),
                    e
                ))
            })?;
            let label = path.to_string_lossy().replace('\\', "/");
            // An indented file is read as its braced spelling, which is what
            // the renderers parse.
            let source = if label.ends_with(".wfx") {
                crate::layout::to_braces(&source, &label)?
            } else {
                source
            };
            sources.push((label, source));
        }
        let refs: Vec<(&str, &str)> = sources
            .iter()
            .map(|(l, s)| (l.as_str(), s.as_str()))
            .collect();
        Self::build(&refs, root)
    }

    /// One template from several sources held in memory, each with the name
    /// its findings are reported under — templates embedded in a binary
    /// with `include_str!`, say:
    ///
    /// ```rust
    /// # use webfluent::Template;
    /// # use serde_json::json;
    /// let tpl = Template::from_sources(&[
    ///     ("card.wf", "component Price(_ amount: Number) { Text(format(amount, .currency)).bold }"),
    ///     ("quote.wf", r#"page Quote(path: "/") { Heading("Quote").h1  Price(total) }"#),
    /// ]).unwrap();
    /// assert!(tpl.render_html_fragment(&json!({ "total": 12.5 })).unwrap().contains("$12.50"));
    /// ```
    ///
    /// # Errors
    ///
    /// As [`from_str`](Template::from_str); a `data` declaration is an error,
    /// since there is no directory to read its file from.
    pub fn from_sources(sources: &[(&str, &str)]) -> Result<Self> {
        Self::build(sources, None)
    }

    fn build(sources: &[(&str, &str)], root: Option<&Path>) -> Result<Self> {
        let mut declarations = Vec::new();
        let mut files: Vec<String> = Vec::new();
        for (label, source) in sources {
            let program = crate::syntax::parse_source(source, label)?;
            files.extend(program.declarations.iter().map(|_| label.to_string()));
            declarations.extend(program.declarations);
        }
        let mut program = Program { declarations };
        // Held to what a build is held to: a component nothing declares, a
        // flag or case a component does not take, a value of the wrong type.
        // A name the template reads is its data's, known only at render
        // time, so an undeclared name is not one of them.
        let file_of = |i: usize| {
            files
                .get(i)
                .cloned()
                .unwrap_or_else(|| "<template>".to_string())
        };
        let errors: Vec<_> =
            crate::diagnostics::check::program_checks(&program, &file_of, &|_| None)
                .into_iter()
                .filter(|d| d.is_error() && d.code != "T13")
                .collect();
        if !errors.is_empty() {
            return Err(WebFluentError::Diagnostics(errors));
        }
        match root {
            Some(root) => crate::data::resolve_data(&mut program, root)?,
            None => {
                if let Some(Declaration::Data(d)) = program
                    .declarations
                    .iter()
                    .find(|d| matches!(d, Declaration::Data(_)))
                {
                    return Err(WebFluentError::IoError(format!(
                        "`data {}` reads a file, which a template from a string has no place to read from; use `Template::from_file`",
                        d.name
                    )));
                }
            }
        }
        Ok(Self {
            program: Arc::new(crate::sema::lower(program)),
            page: None,
            theme: None,
            custom_tokens: HashMap::new(),
            lang: "en".to_string(),
            pdf: PdfConfig::default(),
            slides: SlidesConfig::default(),
            root: root.map(|r| r.to_path_buf()),
        })
    }

    /// The names of the pages the template declares, in source order.
    pub fn pages(&self) -> Vec<&str> {
        self.program
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Page(p) => Some(p.name.as_str()),
                _ => None,
            })
            .collect()
    }

    /// The same template, drawing only the page called `name` — what a
    /// template holding several documents (an invoice, a receipt, an email)
    /// renders one of. Cheap: the parsed program is shared, not copied.
    ///
    /// # Errors
    ///
    /// [`WebFluentError::CodegenError`] naming the pages there are, when none
    /// is called `name`.
    pub fn page(&self, name: &str) -> Result<Self> {
        if !self.pages().contains(&name) {
            return Err(WebFluentError::CodegenError(format!(
                "no page called `{name}`; the template has {}",
                self.pages()
                    .iter()
                    .map(|p| format!("`{p}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        let mut chosen = self.clone();
        chosen.page = Some(name.to_string());
        Ok(chosen)
    }

    /// Select which `Theme` declared in the template to render with.
    ///
    /// Only needed when the source declares more than one. A template with a
    /// single `Theme` uses it automatically, and one with none renders on the
    /// baseline tokens.
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = Some(theme.to_string());
        self
    }

    /// Override design tokens (builder pattern).
    ///
    /// Common tokens: `"color-primary"`, `"color-secondary"`, `"font-family"`,
    /// `"radius-md"`, `"spacing-md"`, etc.
    pub fn with_tokens(mut self, tokens: &[(&str, &str)]) -> Self {
        for (k, v) in tokens {
            self.custom_tokens.insert(k.to_string(), v.to_string());
        }
        self
    }

    /// The language a document from [`render_html`](Template::render_html)
    /// declares, `en` unless set: `<html lang="ar">`, which also turns the
    /// document right to left for an RTL language.
    pub fn with_lang(mut self, lang: &str) -> Self {
        self.lang = lang.to_string();
        self
    }

    /// The page size, margins and fonts [`render_pdf`](Template::render_pdf)
    /// lays out with: A4 with 72pt margins unless set.
    ///
    /// ```rust
    /// # use webfluent::{PdfConfig, Template};
    /// let tpl = Template::from_str(r#"page P(path: "/") { Text("Hi") }"#)
    ///     .unwrap()
    ///     .with_pdf(PdfConfig { page_size: "Letter".into(), ..PdfConfig::default() });
    /// assert!(tpl.render_pdf(&serde_json::json!({})).unwrap().starts_with(b"%PDF"));
    /// ```
    pub fn with_pdf(mut self, config: PdfConfig) -> Self {
        self.pdf = config;
        self
    }

    /// The slide size, margin and chrome [`render_slides`](Template::render_slides)
    /// uses: 16:9 unless set.
    pub fn with_slides(mut self, config: SlidesConfig) -> Self {
        self.slides = config;
        self
    }

    /// The stylesheet a rendered document links: the engine's sheet over the
    /// template's tokens, plus the rules its pseudo-state and media blocks
    /// compile to.
    fn stylesheet(&self) -> Result<String> {
        let mut css = generate_css(&self.tokens()?);
        css.push_str(&crate::codegen::scoped_css::scoped_rules(&self.program));
        Ok(css)
    }

    /// The design tokens this template renders with.
    fn tokens(&self) -> Result<HashMap<String, String>> {
        let config = crate::config::project::ThemeConfig {
            name: self.theme.clone(),
            tokens: self.custom_tokens.clone(),
            builtin: Default::default(),
            dark: None,
        };
        crate::themes::resolve_tokens(&self.program, &config)
    }

    /// The program a render draws: every page, or the one chosen.
    fn drawn(&self) -> std::borrow::Cow<'_, Program> {
        match &self.page {
            None => std::borrow::Cow::Borrowed(&self.program),
            Some(name) => std::borrow::Cow::Owned(Program {
                declarations: self
                    .program
                    .declarations
                    .iter()
                    .filter(|d| !matches!(d, Declaration::Page(p) if &p.name != name))
                    .cloned()
                    .collect(),
            }),
        }
    }

    /// The document's `<title>`: the drawn page's, its `{…}` filled from
    /// the data.
    fn title(&self, data: &Value) -> String {
        let program = self.drawn();
        let Some(page) = program.declarations.iter().find_map(|d| match d {
            Declaration::Page(p) => Some(p),
            _ => None,
        }) else {
            return String::new();
        };
        let ctx = RenderContext::for_program(&program, data);
        if let Some(expr) = &page.title_expr {
            return value_to_string(&ctx.eval_expr(expr));
        }
        let Some(title) = &page.title else {
            return String::new();
        };
        // `"Invoice #{number}"`: each name (or path) read from the data.
        let mut out = String::new();
        let mut rest = title.as_str();
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            match after.find('}') {
                Some(close)
                    if after[..close]
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '.') =>
                {
                    let path = &after[..close];
                    let expr = path.split('.').fold(None::<Expr>, |acc, part| {
                        Some(match acc {
                            None => Expr::Identifier(part.to_string()),
                            Some(e) => Expr::PropertyAccess(Box::new(e), part.to_string()),
                        })
                    });
                    if let Some(expr) = expr {
                        out.push_str(&value_to_string(&ctx.eval_expr(&expr)));
                    }
                    rest = &after[close + 1..];
                }
                _ => {
                    out.push('{');
                    rest = after;
                }
            }
        }
        out.push_str(rest);
        out
    }

    /// Render to a full HTML document with embedded CSS.
    ///
    /// Returns a complete `<!DOCTYPE html>` document: `<html lang>`, a
    /// `<title>` from the page's, a `<style>` block with the component CSS
    /// and theme tokens, and the page in `<body>`.
    ///
    /// `data` is anything `serde` serializes — a `serde_json::Value`, or a
    /// struct of your own with `#[derive(Serialize)]`. Its top-level fields
    /// are the names the template reads.
    pub fn render_html<T: Serialize + ?Sized>(&self, data: &T) -> Result<String> {
        let data = to_value(data)?;
        let fragment = self.render_fragment(&data)?;
        let css = self.stylesheet()?;
        let lang = html_escape(&self.lang);
        let dir = if matches!(
            self.lang.split(['-', '_']).next(),
            Some("ar" | "he" | "fa" | "ur")
        ) {
            " dir=\"rtl\""
        } else {
            ""
        };
        let title = html_escape(&self.title(&data));

        Ok(format!(
            r#"<!DOCTYPE html>
<html lang="{lang}"{dir}>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
{css}
    </style>
</head>
<body>
{fragment}
</body>
</html>"#
        ))
    }

    /// Render the stylesheet and the markup separately.
    ///
    /// [`render_html`](Template::render_html) inlines the CSS in a `<style>`
    /// block, which is right for a self-contained document — an email, an
    /// attachment, anything that travels without a server. It is also the one
    /// output of this compiler a strict `Content-Security-Policy` rejects, since
    /// `style-src 'self'` does not cover inline styles.
    ///
    /// A caller serving over HTTP can take the two halves, put the CSS at its own
    /// URL and link it, and keep the policy strict:
    ///
    /// ```rust
    /// # use webfluent::Template;
    /// # use serde_json::json;
    /// let tpl = Template::from_str("page P(path: \"/\") { Text(\"Hi\") }").unwrap();
    /// let (css, body) = tpl.render_html_parts(&json!({})).unwrap();
    /// // Serve `css` at /styles.css, and link it from your own shell.
    /// assert!(css.contains(":root"));
    /// assert!(body.contains("Hi"));
    /// ```
    pub fn render_html_parts<T: Serialize + ?Sized>(&self, data: &T) -> Result<(String, String)> {
        let data = to_value(data)?;
        Ok((self.stylesheet()?, self.render_fragment(&data)?))
    }

    /// Render to an HTML fragment (no `<html>`/`<head>`/`<body>` wrapper).
    ///
    /// Useful for embedding rendered content into an existing page or email template.
    /// Does not include CSS — use [`render_html`](Template::render_html) for a complete
    /// document, or [`render_html_parts`](Template::render_html_parts) to serve the CSS
    /// separately.
    pub fn render_html_fragment<T: Serialize + ?Sized>(&self, data: &T) -> Result<String> {
        self.render_fragment(&to_value(data)?)
    }

    fn render_fragment(&self, data: &Value) -> Result<String> {
        render_program_fragment(&self.drawn(), data)
    }

    /// Render to PDF as raw bytes.
    ///
    /// Returns a valid PDF file as `Vec<u8>`. Write the result to a file
    /// or send it as an HTTP response with `Content-Type: application/pdf`.
    ///
    /// The PDF is the page [`render_html`](Template::render_html) draws,
    /// laid out on paper: flex and grid, the theme's tokens, embedded fonts
    /// (the template's `fonts/` directory, then Liberation, built in), and
    /// pictures read from beside the template. A4 with 72pt margins unless
    /// [`with_pdf`](Template::with_pdf) says otherwise.
    pub fn render_pdf<T: Serialize + ?Sized>(&self, data: &T) -> Result<Vec<u8>> {
        Ok(self.render_pdf_with_notes(data)?.0)
    }

    /// [`render_pdf`](Template::render_pdf), with what the reader of the
    /// render should know: a character no font had, a font taken from this
    /// machine rather than the template's.
    pub fn render_pdf_with_notes<T: Serialize + ?Sized>(
        &self,
        data: &T,
    ) -> Result<(Vec<u8>, Vec<String>)> {
        let report = self.render_pdf_report(data)?;
        Ok((report.bytes, report.notes))
    }

    /// Render to PDF and report what was drawn: the bytes, the page count,
    /// the text of each page a line at a time in reading order, and the
    /// notes — for a test that holds a document to what it should say, or
    /// a server that logs what it sent.
    pub fn render_pdf_report<T: Serialize + ?Sized>(&self, data: &T) -> Result<PdfReport> {
        let data = to_value(data)?;
        let html = self.paged_document(&data)?;
        let read = crate::paged::reader(self.root.clone());
        let options = crate::paged::Options::from_pdf(&self.pdf, &read, self.root.as_deref());
        let out =
            crate::paged::render(&html, "", &options).map_err(WebFluentError::CodegenError)?;
        Ok(PdfReport {
            bytes: out.bytes,
            pages: out.pages,
            text: out.text,
            notes: out.notes,
        })
    }

    /// [`render_slides`](Template::render_slides), reported as
    /// [`render_pdf_report`](Template::render_pdf_report) reports a document.
    pub fn render_slides_report<T: Serialize + ?Sized>(&self, data: &T) -> Result<PdfReport> {
        let data = to_value(data)?;
        let html = self.paged_document(&data)?;
        let read = crate::paged::reader(self.root.clone());
        let (options, chrome) =
            crate::paged::Options::from_slides(&self.slides, &read, self.root.as_deref());
        let out = crate::paged::render_slides(&html, "", &options, &chrome)
            .map_err(WebFluentError::CodegenError)?;
        Ok(PdfReport {
            bytes: out.bytes,
            pages: out.pages,
            text: out.text,
            notes: out.notes,
        })
    }

    /// Render to a PDF slide deck as raw bytes.
    ///
    /// One `Slide` (or layout variant) per page, no flow pagination. The template
    /// must wrap its slides in a `Presentation { ... }` block; content that overflows
    /// a slide is clipped and noted.
    ///
    /// Uses 16:9 (960×540pt) unless [`with_slides`](Template::with_slides) says
    /// otherwise.
    pub fn render_slides<T: Serialize + ?Sized>(&self, data: &T) -> Result<Vec<u8>> {
        let data = to_value(data)?;
        let html = self.paged_document(&data)?;
        let read = crate::paged::reader(self.root.clone());
        let (options, chrome) =
            crate::paged::Options::from_slides(&self.slides, &read, self.root.as_deref());
        let out = crate::paged::render_slides(&html, "", &options, &chrome)
            .map_err(WebFluentError::CodegenError)?;
        Ok(out.bytes)
    }

    /// The document a paged output lays out: the page's markup with its
    /// stylesheet, `page` and `pages` marked in running elements.
    fn paged_document(&self, data: &Value) -> Result<String> {
        let fragment = crate::template::render_program_paged(&self.drawn(), data)?;
        let css = self.stylesheet()?;
        let title = html_escape(&self.title(data));
        let lang = html_escape(&self.lang);
        let dir = if is_rtl(&self.lang) {
            " dir=\"rtl\""
        } else {
            ""
        };
        Ok(format!(
            "<!DOCTYPE html><html lang=\"{lang}\"{dir}><head><title>{title}</title><style>{css}</style></head><body>{fragment}</body></html>"
        ))
    }
}

/// `data` as the JSON value the renderers read.
fn to_value<T: Serialize + ?Sized>(data: &T) -> Result<Value> {
    serde_json::to_value(data)
        .map_err(|e| WebFluentError::ConfigError(format!("the data does not serialize: {e}")))
}

/// The HTML fragment of every page of a lowered program, over `data`:
/// what a template renders, and what `wf test` holds a test to.
pub fn render_program_fragment(program: &Program, data: &Value) -> Result<String> {
    render_program(program, data, false)
}

/// The markup a paged output lays out: as [`render_program_fragment`], with
/// `page` and `pages` in a running element written as the marks each page
/// replaces with its number.
pub fn render_program_paged(program: &Program, data: &Value) -> Result<String> {
    render_program(program, data, true)
}

fn render_program(program: &Program, data: &Value, paged: bool) -> Result<String> {
    let mut ctx = RenderContext::for_program(program, data);
    if paged {
        ctx.locals.insert(PAGED.to_string(), Value::Bool(true));
    }

    let mut html = String::new();
    for decl in &program.declarations {
        if let Declaration::Page(page) = decl {
            html.push_str(&render_statements(&page.body, &mut ctx));
        }
    }
    Ok(html)
}

// ─── Render Context ──────────────────────────────────────────────────

/// The block a caller wrote for a slot: the names it gives the slot's
/// values, what the slot's declaration calls them, and the block.
#[derive(Debug, Clone)]
struct Fill {
    params: Vec<String>,
    handed: Vec<String>,
    body: Vec<Statement>,
}

/// A component as the template engine expands it: its body, its props,
/// and per slot the names of what it hands over.
#[derive(Debug, Clone)]
struct TemplateComponent {
    body: Vec<Statement>,
    handed: HashMap<String, Vec<String>>,
    props: Vec<PropDecl>,
}

#[derive(Clone)]
struct RenderContext<'a> {
    data: &'a Value,
    locals: HashMap<String, Value>,
    components: HashMap<String, TemplateComponent>,
    indent: usize,
    /// Inside a `Thead`, a `Tcell` is a column header (`<th scope="col">`).
    in_thead: bool,
    /// The caller's block for each user component being expanded, innermost
    /// last; `children` renders the top one.
    slots: Vec<HashMap<String, Fill>>,
    /// Fields rendered so far, for their ids.
    fields: usize,
}

impl<'a> RenderContext<'a> {
    /// A context over `data` with the program's constants in scope —
    /// `const LIMIT = 3`, and a `data plans = "plans.json"` file, which is a
    /// constant once read — each in declaration order, so one may read the
    /// one before it.
    fn for_program(program: &Program, data: &'a Value) -> Self {
        let mut ctx = Self::new(data);
        for decl in &program.declarations {
            match decl {
                Declaration::Const(c) => {
                    let value = ctx.eval_expr(&c.value);
                    ctx.locals.insert(c.name.clone(), value);
                }
                // Every component, wherever it is declared, so a page may
                // use one declared after it.
                Declaration::Component(comp) => {
                    let handed = comp
                        .slots
                        .iter()
                        .map(|s| {
                            (
                                s.name.clone().unwrap_or_else(|| "children".to_string()),
                                s.params.iter().map(|p| p.name.clone()).collect(),
                            )
                        })
                        .collect();
                    ctx.components.insert(
                        comp.name.clone(),
                        TemplateComponent {
                            body: comp.body.clone(),
                            handed,
                            props: comp.props.clone(),
                        },
                    );
                }
                _ => {}
            }
        }
        ctx
    }

    /// The props a call of `component` binds, each evaluated here: a
    /// positional argument binds the positional prop (or the first), a
    /// named one its name, and a prop left out takes its default.
    fn props_of(&self, component: &TemplateComponent, ui: &UIElement) -> Vec<(String, Value)> {
        let props = &component.props;
        let mut given: Vec<(String, Value)> = Vec::new();
        for arg in &ui.args {
            match arg {
                Arg::Named(key, val) => given.push((key.clone(), self.eval_expr(val))),
                Arg::Positional(val) => {
                    let target = props
                        .iter()
                        .find(|p| p.positional)
                        .or_else(|| props.first())
                        .map(|p| p.name.clone());
                    if let Some(key) = target {
                        given.push((key, self.eval_expr(val)));
                    }
                }
            }
        }
        for prop in props {
            if !given.iter().any(|(k, _)| *k == prop.name)
                && let Some(default) = &prop.default
            {
                given.push((prop.name.clone(), self.eval_expr(default)));
            }
        }
        given
    }

    fn new(data: &'a Value) -> Self {
        Self {
            data,
            locals: HashMap::new(),
            components: HashMap::new(),
            indent: 1,
            in_thead: false,
            slots: Vec::new(),
            fields: 0,
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    /// A context for an expression that binds a name: the same data and
    /// locals, to add to without touching this one.
    fn child(&self) -> RenderContext<'a> {
        RenderContext {
            data: self.data,
            locals: self.locals.clone(),
            components: self.components.clone(),
            indent: self.indent,
            in_thead: self.in_thead,
            slots: self.slots.clone(),
            fields: self.fields,
        }
    }

    /// Look up a variable: first in locals (loop vars), then in data context.
    fn resolve_var(&self, name: &str) -> Value {
        if let Some(val) = self.locals.get(name) {
            return val.clone();
        }
        if let Some(val) = self.data.get(name) {
            return val.clone();
        }
        Value::Null
    }

    /// Evaluate an expression against the data context.
    /// A `to:` put together as the live page puts it: each spliced value
    /// encoded where it stands for one.
    fn address(&self, expr: &Expr) -> String {
        match expr {
            Expr::InterpolatedString(parts) => {
                let pieces: Vec<crate::codegen::url::AddressPart> = parts
                    .iter()
                    .map(|p| match p {
                        StringPart::Literal(t) => crate::codegen::url::AddressPart::Text(t.clone()),
                        StringPart::Expression(e) => crate::codegen::url::AddressPart::Value(
                            value_to_string(&self.eval_expr(e)),
                        ),
                    })
                    .collect();
                crate::codegen::url::join_address(&pieces)
            }
            other => value_to_string(&self.eval_expr(other)),
        }
    }

    fn eval_expr(&self, expr: &Expr) -> Value {
        match expr {
            // What a string says is settled by the parser, which resolved
            // its escapes and split its splices out; a `{` left in one is
            // a brace the author wrote.
            Expr::StringLiteral(s) => Value::String(s.clone()),
            Expr::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Expression(e) => {
                            let val = self.eval_expr(e);
                            result.push_str(&value_to_string(&val));
                        }
                    }
                }
                Value::String(result)
            }
            Expr::NumberLiteral(n) => Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
            ),
            Expr::BoolLiteral(b) => Value::Bool(*b),
            Expr::Null => Value::Null,
            Expr::Identifier(name) => self.resolve_var(name),
            Expr::PropertyAccess(obj, prop) => {
                let parent = self.eval_expr(obj);
                match &parent {
                    Value::Object(map) => map.get(prop).cloned().unwrap_or(Value::Null),
                    Value::Array(arr) if prop == "length" => {
                        Value::Number(serde_json::Number::from(arr.len()))
                    }
                    _ => Value::Null,
                }
            }
            // `?.` in a template: null is null, and a null base reads as null.
            Expr::OptionalProperty(obj, prop) => match self.eval_expr(obj) {
                Value::Null => Value::Null,
                _ => self.eval_expr(&Expr::PropertyAccess(obj.clone(), prop.clone())),
            },
            Expr::OptionalIndex(obj, idx) => match self.eval_expr(obj) {
                Value::Null => Value::Null,
                _ => self.eval_expr(&Expr::IndexAccess(obj.clone(), idx.clone())),
            },
            Expr::OptionalMethod(obj, method, args) => match self.eval_expr(obj) {
                Value::Null => Value::Null,
                _ => self.eval_expr(&Expr::MethodCall(obj.clone(), method.clone(), args.clone())),
            },
            Expr::IndexAccess(arr_expr, idx_expr) => {
                let arr = self.eval_expr(arr_expr);
                let idx = self.eval_expr(idx_expr);
                match (&arr, &idx) {
                    // A literal index is a float here (`0` lexes as `0.0`).
                    (Value::Array(a), Value::Number(n)) => match n.as_f64() {
                        Some(i) if i >= 0.0 => a.get(i as usize).cloned().unwrap_or(Value::Null),
                        _ => Value::Null,
                    },
                    (Value::Object(map), Value::String(key)) => {
                        map.get(key).cloned().unwrap_or(Value::Null)
                    }
                    _ => Value::Null,
                }
            }
            Expr::BinaryOp(left, op, right) => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                eval_binary_op(&l, op, &r)
            }
            Expr::UnaryOp(op, operand) => {
                let val = self.eval_expr(operand);
                match op {
                    crate::parser::ast::UnaryOp::Not => Value::Bool(!is_truthy(&val)),
                    crate::parser::ast::UnaryOp::Neg => {
                        if let Some(n) = val.as_f64() {
                            Value::Number(
                                serde_json::Number::from_f64(-n)
                                    .unwrap_or(serde_json::Number::from(0)),
                            )
                        } else {
                            Value::Null
                        }
                    }
                }
            }
            // `if c { a } else { b }` and `if let x = e { a } else { b }` as
            // values.
            Expr::MethodCall(cond, method, args) if method == "__if" && args.len() == 2 => {
                if is_truthy(&self.eval_expr(cond)) {
                    self.eval_expr(&args[0])
                } else {
                    self.eval_expr(&args[1])
                }
            }
            Expr::MethodCall(value, method, args) if method == "__iflet" && args.len() == 2 => {
                let v = self.eval_expr(value);
                match (&v, &args[0]) {
                    (Value::Null, _) => self.eval_expr(&args[1]),
                    (_, Expr::Lambda(name, body)) => {
                        let mut inner = self.child();
                        inner.locals.insert(name.clone(), v.clone());
                        inner.eval_expr(body)
                    }
                    _ => Value::Null,
                }
            }
            Expr::MethodCall(_, method, _) if method == "__exhaustive" => Value::Null,
            Expr::MethodCall(subject, method, args) if method == "__case" && args.is_empty() => {
                case_of(&self.eval_expr(subject))
            }
            Expr::MethodCall(subject, method, args) if method == "__is" && args.len() == 1 => {
                let v = self.eval_expr(subject);
                Value::Bool(case_of(&v) == self.eval_expr(&args[0]))
            }
            Expr::MethodCall(subject, method, args) if method == "__payload" && args.len() == 1 => {
                let v = self.eval_expr(subject);
                let case = self.eval_expr(&args[0]);
                payload_of(&v, &case)
            }
            Expr::MethodCall(obj, method, args) => {
                let parent = self.eval_expr(obj);
                match method.as_str() {
                    "length" => match &parent {
                        Value::Array(a) => Value::Number(serde_json::Number::from(a.len())),
                        Value::String(s) => Value::Number(serde_json::Number::from(s.len())),
                        _ => Value::Null,
                    },
                    "toUpperCase" => match &parent {
                        Value::String(s) => Value::String(s.to_uppercase()),
                        _ => Value::Null,
                    },
                    "toLowerCase" => match &parent {
                        Value::String(s) => Value::String(s.to_lowercase()),
                        _ => Value::Null,
                    },
                    "includes" => {
                        let needle = if let Some(a) = args.first() {
                            self.eval_expr(a)
                        } else {
                            Value::Null
                        };
                        match (&parent, &needle) {
                            (Value::String(s), Value::String(n)) => {
                                Value::Bool(s.contains(n.as_str()))
                            }
                            (Value::Array(arr), _) => Value::Bool(arr.contains(&needle)),
                            _ => Value::Bool(false),
                        }
                    }
                    "join" => {
                        let sep = if let Some(a) = args.first() {
                            value_to_string(&self.eval_expr(a))
                        } else {
                            ", ".to_string()
                        };
                        match &parent {
                            Value::Array(arr) => {
                                let parts: Vec<String> = arr.iter().map(value_to_string).collect();
                                Value::String(parts.join(&sep))
                            }
                            _ => Value::Null,
                        }
                    }
                    // Every other method the language has, the build-time
                    // evaluator knows: the data is its scope.
                    _ => self.eval_static(expr),
                }
            }
            Expr::Regex(..) => self.eval_static(expr),
            Expr::FunctionCall(name, _args) => {
                // t() in template mode — not supported, return key
                if name == "t" {
                    if let Some(Expr::StringLiteral(key)) = _args.first() {
                        Value::String(key.clone())
                    } else {
                        Value::Null
                    }
                } else if name == "format" || name == "ago" {
                    // Formatting speaks the data's `locale`, or English.
                    self.eval_static(expr)
                } else {
                    Value::Null
                }
            }
            Expr::ListLiteral(items) => {
                let mut out = Vec::new();
                for e in items {
                    match e {
                        Expr::Spread(inner) => {
                            if let Value::Array(more) = self.eval_expr(inner) {
                                out.extend(more);
                            }
                        }
                        _ => out.push(self.eval_expr(e)),
                    }
                }
                Value::Array(out)
            }
            Expr::MapLiteral(pairs) | Expr::Record(_, pairs) => {
                let mut map = serde_json::Map::new();
                for (k, v) in pairs {
                    if k == "..." {
                        if let Value::Object(more) = self.eval_expr(v) {
                            map.extend(more);
                        }
                    } else {
                        map.insert(k.trim_matches('"').to_string(), self.eval_expr(v));
                    }
                }
                Value::Object(map)
            }
            Expr::Range(..) => self.eval_static(expr),
            Expr::EnumCase(case) => Value::String(case.clone()),
            Expr::CaseValue(case, args) => {
                let mut items = vec![Value::String(case.clone())];
                items.extend(args.iter().map(|a| self.eval_expr(a)));
                Value::Array(items)
            }
            Expr::Token(name) => Value::String(format!("var(--{name})")),
            _ => Value::Null,
        }
    }

    /// Evaluate `expr` with the static evaluator, over this context's data
    /// and locals; `Null` when it cannot.
    fn eval_static(&self, expr: &Expr) -> Value {
        use crate::codegen::static_eval::{Scope, Static, eval};
        let mut names: Vec<(String, Static)> = Vec::new();
        if let Value::Object(map) = self.data {
            for (k, v) in map {
                names.push((k.clone(), Static::from_json(v)));
            }
        }
        for (k, v) in &self.locals {
            names.push((k.clone(), Static::from_json(v)));
        }
        eval(expr, &Scope::of(names))
            .map(|v| v.to_json())
            .unwrap_or(Value::Null)
    }

    /// Resolve a dotted path like "user.address.city" from data context.
    fn resolve_path(&self, path: &str) -> Value {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Value::Null;
        }

        let mut current = self.resolve_var(parts[0]);
        for &part in &parts[1..] {
            current = match &current {
                Value::Object(map) => map.get(part).cloned().unwrap_or(Value::Null),
                _ => Value::Null,
            };
        }
        current
    }
}

// ─── HTML Rendering ──────────────────────────────────────────────────

fn render_statements(stmts: &[Statement], ctx: &mut RenderContext) -> String {
    let mut html = String::new();
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => html.push_str(&render_ui_element(ui, ctx)),
            StatementKind::If(if_stmt) => html.push_str(&render_if(if_stmt, ctx)),
            StatementKind::For(for_stmt) => html.push_str(&render_for(for_stmt, ctx)),
            // A resource has no data at render time: the `loading` arm, or
            // `else`, is what a document can show. An enum's value is at
            // hand, so its arm is the case's, with the payload bound.
            StatementKind::Match(m) => {
                let over_resource = m.arms.iter().any(|a| {
                    matches!(
                        a.pattern,
                        ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready
                    )
                });
                let value = if over_resource {
                    Value::Null
                } else {
                    ctx.eval_expr(&m.scrutinee)
                };
                let case = case_of(&value);
                let arm = m
                    .arms
                    .iter()
                    .find(|a| matches!(&a.pattern, ArmPattern::Case(c) if Value::String(c.clone()) == case))
                    .or_else(|| m.arms.iter().find(|a| a.pattern == ArmPattern::Loading))
                    .or_else(|| m.arms.iter().find(|a| a.pattern == ArmPattern::Else));
                if let Some(arm) = arm {
                    let mut saved = Vec::new();
                    if let Value::Array(items) = &value {
                        for (name, item) in arm.bindings.iter().zip(items.iter().skip(1)) {
                            saved.push((
                                name.clone(),
                                ctx.locals.insert(name.clone(), item.clone()),
                            ));
                        }
                    }
                    html.push_str(&render_statements(&arm.body, ctx));
                    for (name, old) in saved.into_iter().rev() {
                        match old {
                            Some(v) => ctx.locals.insert(name, v),
                            None => ctx.locals.remove(&name),
                        };
                    }
                }
            }
            // Skip state, derived, effect, action, use, events, navigate, etc.
            _ => {}
        }
    }
    html
}

fn render_if(if_stmt: &IfStmt, ctx: &mut RenderContext) -> String {
    let cond = ctx.eval_expr(&if_stmt.condition);
    if is_truthy(&cond) {
        // `if let p = post { … }`: the branch reads `p` as the value.
        let Some(name) = &if_stmt.binding else {
            return render_statements(&if_stmt.then_body, ctx);
        };
        let old = ctx.locals.insert(name.clone(), cond.clone());
        let html = render_statements(&if_stmt.then_body, ctx);
        match old {
            Some(v) => ctx.locals.insert(name.clone(), v),
            None => ctx.locals.remove(name),
        };
        return html;
    }
    // Check else-if branches
    for (branch_cond, branch_body) in &if_stmt.else_if_branches {
        let val = ctx.eval_expr(branch_cond);
        if is_truthy(&val) {
            return render_statements(branch_body, ctx);
        }
    }
    // Else branch
    if let Some(else_body) = &if_stmt.else_body {
        render_statements(else_body, ctx)
    } else {
        String::new()
    }
}

fn render_for(for_stmt: &ForStmt, ctx: &mut RenderContext) -> String {
    let collection = ctx.eval_expr(&for_stmt.iterable);
    let mut html = String::new();

    if let Value::Array(items) = &collection {
        for (i, item) in items.iter().enumerate() {
            // Push loop variable into locals
            let old_item = ctx.locals.insert(for_stmt.item.clone(), item.clone());
            let old_index = if let Some(idx_var) = &for_stmt.index {
                ctx.locals
                    .insert(idx_var.clone(), Value::Number(serde_json::Number::from(i)))
            } else {
                None
            };

            html.push_str(&render_statements(&for_stmt.body, ctx));

            // Restore previous locals
            if let Some(old) = old_item {
                ctx.locals.insert(for_stmt.item.clone(), old);
            } else {
                ctx.locals.remove(&for_stmt.item);
            }
            if let Some(idx_var) = &for_stmt.index {
                if let Some(old) = old_index {
                    ctx.locals.insert(idx_var.clone(), old);
                } else {
                    ctx.locals.remove(idx_var);
                }
            }
        }
    }
    html
}

fn render_ui_element(ui: &UIElement, ctx: &mut RenderContext) -> String {
    match &ui.component {
        ComponentRef::BuiltIn(name) => render_builtin(name, ui, ctx),
        ComponentRef::SubComponent(parent, sub) => {
            let class = format!("wf-{}__{}", parent.to_lowercase(), camel_to_kebab(sub));
            // A `to:` makes this a link; dropping it left a dead list item.
            if let Some(dest) = ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == "to" => Some(v.clone()),
                _ => None,
            }) {
                // A destination from the data is followed by the browser,
                // so it is held to the scheme check the live page applies.
                let href = crate::codegen::url::guard(&ctx.address(&dest)).to_string();
                let indent = ctx.indent_str();
                let mut out = format!(
                    "{}<a class=\"{}\" href=\"{}\">\n",
                    indent,
                    class,
                    html_escape(&href)
                );
                ctx.indent += 1;
                out.push_str(&render_statements(&ui.children, ctx));
                ctx.indent -= 1;
                out.push_str(&format!("{}</a>\n", indent));
                return out;
            }
            let tag = match sub.as_str() {
                "Item" => "li",
                _ => "div",
            };
            render_tag(tag, &class, ui, ctx)
        }
        ComponentRef::UserDefined(name) => {
            // Expand user component if registered
            if let Some(component) = ctx.components.get(name).cloned() {
                // The props as locals, restored afterwards.
                let mut old_locals = Vec::new();
                for (key, resolved) in ctx.props_of(&component, ui) {
                    let old = ctx.locals.insert(key.clone(), resolved);
                    old_locals.push((key, old));
                }
                ctx.slots.push(slot_fills(&component, ui));
                let html = render_statements(&component.body, ctx);
                ctx.slots.pop();

                // Restore locals
                for (key, old) in old_locals {
                    if let Some(v) = old {
                        ctx.locals.insert(key, v);
                    } else {
                        ctx.locals.remove(&key);
                    }
                }

                html
            } else {
                format!("{}<!-- unknown component: {} -->\n", ctx.indent_str(), name)
            }
        }
    }
}

fn render_builtin(name: &str, ui: &UIElement, ctx: &mut RenderContext) -> String {
    // The `children` slot: the block the caller of the enclosing component
    // wrote, rendered in its place. Outside a component there is none.
    if let Some(slot) = ui.slot_name() {
        return match ctx.slots.last().and_then(|s| s.get(slot)).cloned() {
            Some(fill) => {
                // A scoped slot's values, under the names the fill gave them.
                let mut saved = Vec::new();
                for (i, param) in fill.params.iter().enumerate() {
                    let key = fill.handed.get(i).cloned().unwrap_or_else(|| param.clone());
                    let value = ui
                        .args
                        .iter()
                        .find_map(|a| match a {
                            Arg::Named(k, v) if *k == key => Some(ctx.eval_expr(v)),
                            _ => None,
                        })
                        .unwrap_or(Value::Null);
                    saved.push((param.clone(), ctx.locals.insert(param.clone(), value)));
                }
                let html = render_statements(&fill.body, ctx);
                for (name, old) in saved.into_iter().rev() {
                    match old {
                        Some(v) => ctx.locals.insert(name, v),
                        None => ctx.locals.remove(&name),
                    };
                }
                html
            }
            None => String::new(),
        };
    }
    // A custom element, by its tag, with the attributes the data gives it.
    if name == "Element" {
        let tag = match ui.args.first() {
            Some(Arg::Positional(Expr::StringLiteral(tag))) => tag.clone(),
            _ => "wf-element".to_string(),
        };
        let mut attrs = String::new();
        for arg in &ui.args {
            let Arg::Named(key, value) = arg else {
                continue;
            };
            if matches!(
                key.as_str(),
                "class" | "ref" | "mount" | "update" | "cleanup"
            ) || key.starts_with("data-wf-")
            {
                continue;
            }
            let attribute = camel_to_kebab(key);
            match ctx.eval_expr(value) {
                Value::Bool(true) => attrs.push_str(&format!(" {attribute}")),
                Value::Bool(false) | Value::Null | Value::Array(_) | Value::Object(_) => {}
                v => {
                    let text = value_to_string(&v);
                    let text = if crate::codegen::url::URL_ATTRS.contains(&attribute.as_str()) {
                        crate::codegen::url::guard(&text).to_string()
                    } else {
                        text
                    };
                    attrs.push_str(&format!(" {attribute}=\"{}\"", html_escape(&text)));
                }
            }
        }
        let classes = extra_classes(ui, ctx).join(" ");
        let class = if classes.is_empty() {
            String::new()
        } else {
            format!(" class=\"{classes}\"")
        };
        let indent = ctx.indent_str();
        let mut out = format!("{indent}<{tag}{class}{attrs}>\n");
        ctx.indent += 1;
        out.push_str(&render_statements(&ui.children, ctx));
        ctx.indent -= 1;
        out.push_str(&format!("{indent}</{tag}>\n"));
        return out;
    }
    let (_, base_class) = builtin_to_html(name);
    let mut classes = class_list(base_class, &ui.modifiers);
    classes.extend(layout_arg_classes(&ui.args));
    classes.extend(extra_classes(ui, ctx));
    let class_str = classes.join(" ");

    // Special handling. These build their tag inline, so they carry the author's
    // `style { }` block themselves — returning early used to drop it.
    let style_attr = style_block_attr(ui, ctx);
    if let Some(html) = render_paged(name, ui, ctx, &class_str, &style_attr) {
        return html;
    }
    match name {
        "Spacer" | "Spinner" => {
            return format!(
                "{}<div class=\"{}\"{}></div>\n",
                ctx.indent_str(),
                class_str,
                style_attr
            );
        }
        "Markdown" => {
            let text = ui
                .args
                .iter()
                .find_map(|a| match a {
                    Arg::Positional(e) => Some(value_to_string(&ctx.eval_expr(e))),
                    _ => None,
                })
                .unwrap_or_default();
            return format!(
                "{}<div class=\"{}\"{}>\n{}{}</div>\n",
                ctx.indent_str(),
                class_str,
                style_attr,
                crate::codegen::markdown::render(&text),
                ctx.indent_str()
            );
        }
        "Divider" => {
            return format!(
                "{}<hr class=\"{}\"{}>\n",
                ctx.indent_str(),
                class_str,
                style_attr
            );
        }
        // A wrapper carrying the component class, an optional label, and the
        // input — the same shape the SPA and SSG renderers build.
        "Slider" | "DatePicker" | "FileUpload" => {
            return render_labelled_input(name, &class_str, &style_attr, ui, ctx);
        }
        "Toast" | "Router" | "Route" => return String::new(),
        _ => {}
    }

    let mut attrs = Vec::new();
    let mut text_content: Option<String> = None;
    let mut inline_style: Option<String> = None;

    if let Some(role) = implicit_role(name, &ui.modifiers) {
        attrs.push(format!("role=\"{}\"", role));
    }
    if let Some(label) = landmark_label(name) {
        attrs.push(format!("aria-label=\"{}\"", label));
    }

    if !class_str.is_empty() {
        attrs.push(format!("class=\"{}\"", class_str));
    }

    for arg in &ui.args {
        match arg {
            Arg::Named(key, val) => match key.as_str() {
                "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max" | "step"
                | "accept" | "role" | "value" | "title" | "width" | "height" | "loading"
                | "decoding" | "fetchpriority" => {
                    let resolved = value_to_string(&ctx.eval_expr(val));
                    // A URL from the data: `javascript:` and its kind are
                    // dropped, as `WF.safeUrl` drops them on a live page.
                    let resolved = if crate::codegen::url::URL_ATTRS.contains(&key.as_str()) {
                        crate::codegen::url::guard(&resolved).to_string()
                    } else {
                        resolved
                    };
                    attrs.push(format!("{}=\"{}\"", key, html_escape(&resolved)));
                }
                "to" => {
                    let resolved = ctx.address(val);
                    attrs.push(format!(
                        "href=\"{}\"",
                        html_escape(crate::codegen::url::guard(&resolved))
                    ));
                }
                "label" if name == "IconButton" => {
                    // The accessible name of an icon-only button, never visible text.
                    let s = value_to_string(&ctx.eval_expr(val));
                    attrs.push(format!("aria-label=\"{}\"", html_escape(&s)));
                    attrs.push(format!("title=\"{}\"", html_escape(&s)));
                }
                "label" => {
                    let resolved = ctx.eval_expr(val);
                    text_content = Some(value_to_string(&resolved));
                }
                "columns" => {
                    if let Value::Number(n) = ctx.eval_expr(val) {
                        attrs.push(format!(
                            "data-cols=\"{}\"",
                            n.as_f64().unwrap_or(1.0) as i32
                        ));
                    }
                }
                "required" | "disabled" | "controls" => {
                    attrs.push(key.to_string());
                }
                "icon" => {
                    let s = value_to_string(&ctx.eval_expr(val));
                    attrs.push(format!("data-icon=\"{}\"", html_escape(&s)));
                }
                k if k.contains('-') => {
                    let s = value_to_string(&ctx.eval_expr(val));
                    attrs.push(format!("{}=\"{}\"", k, html_escape(&s)));
                }
                _ => {}
            },
            Arg::Positional(expr) => {
                // `Icon("home")` names the glyph, drawn at runtime from `data-icon`.
                if name == "Icon" {
                    if !attrs.iter().any(|a| a.starts_with("data-icon=")) {
                        let s = value_to_string(&ctx.eval_expr(expr));
                        attrs.push(format!("data-icon=\"{}\"", html_escape(&s)));
                    }
                    continue;
                }
                // `Option("value", "Label")`: value attribute, then the label.
                if name == "Option" && text_content.is_some() {
                    if !attrs.iter().any(|a| a.starts_with("value=")) {
                        if let Some(v) = text_content.take() {
                            attrs.push(format!("value=\"{}\"", html_escape(&v)));
                        }
                        let s = value_to_string(&ctx.eval_expr(expr));
                        if !s.is_empty() && s != "null" {
                            text_content = Some(s);
                        }
                    }
                    continue;
                }
                if text_content.is_none() {
                    let resolved = ctx.eval_expr(expr);
                    let s = value_to_string(&resolved);
                    if !s.is_empty() && s != "null" {
                        text_content = Some(s);
                    }
                }
            }
        }
    }

    // Inline style blocks: a literal or a token is in the stylesheet under
    // the element's scoped class; a value that reads the data is inline.
    if let Some(style_block) = &ui.style_block {
        let mut style_parts = Vec::new();
        for prop in &style_block.properties {
            if crate::codegen::scoped_css::static_declaration(prop).is_some() {
                continue;
            }
            let val = ctx.eval_expr(&prop.value);
            style_parts.push(format!("{}: {}", prop.name, value_to_string(&val)));
        }
        if let Some(existing) = &inline_style {
            style_parts.insert(0, existing.clone());
        }
        inline_style = Some(style_parts.join("; "));
    }

    if let Some(style) = &inline_style {
        attrs.push(format!("style=\"{}\"", html_escape(style)));
    }

    if name == "Image" {
        for (key, default) in [("loading", "lazy"), ("decoding", "async")] {
            if !ui
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, _) if k == key))
            {
                attrs.push(format!("{}=\"{}\"", key, default));
            }
        }
    }

    // Input type from modifiers
    for m in &ui.modifiers {
        if let Some(t) = input_type(m) {
            attrs.push(format!("type=\"{}\"", t));
        } else if m == "multiple" {
            attrs.push("multiple".to_string());
        }
    }

    let actual_tag =
        if name == "Tcell" && (ctx.in_thead || element_tag(name, &ui.modifiers) == "th") {
            attrs.push("scope=\"col\"".to_string());
            "th"
        } else {
            element_tag(name, &ui.modifiers)
        };
    let indent = ctx.indent_str();
    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    };

    // A field: label, control, hint, error — the shape the bundle builds.
    if matches!(name, "Input" | "Select") {
        let resolve = |e: &Expr| Some(value_to_string(&ctx.eval_expr(e)));
        if let Some(mut parts) = crate::codegen::ssg::field_parts(ui, resolve) {
            ctx.fields += 1;
            let id = format!("wf-field-{}", ctx.fields);
            parts.set_id(&id);
            let control = if actual_tag == "input" {
                format!(
                    "<{}{} id=\"{}\"{}>",
                    actual_tag,
                    attrs_str,
                    id,
                    parts.control_attrs()
                )
            } else {
                ctx.indent += 1;
                let inner = render_statements(&ui.children, ctx);
                ctx.indent -= 1;
                format!(
                    "<{}{} id=\"{}\"{}>\n{}{}</{}>",
                    actual_tag,
                    attrs_str,
                    id,
                    parts.control_attrs(),
                    inner,
                    indent,
                    actual_tag
                )
            };
            return crate::codegen::ssg::static_field(&indent, &id, &control, &parts);
        }
    }

    // Self-closing tags
    if matches!(actual_tag, "input" | "img" | "hr" | "br") {
        return format!("{}<{}{}>\n", indent, actual_tag, attrs_str);
    }

    let caption = if name == "Table" {
        ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "caption" => Some(value_to_string(&ctx.eval_expr(v))),
            _ => None,
        })
    } else {
        None
    };
    let has_children = !ui.children.is_empty();
    let has_text = text_content.is_some();

    if !has_children && !has_text && caption.is_none() {
        return format!("{}<{}{}></{}>\n", indent, actual_tag, attrs_str, actual_tag);
    }

    if let Some(text) = &text_content {
        if !has_children {
            // `Code(…, language: "wf")`: coloured, as every other backend.
            let language = if name == "Code" {
                ui.args.iter().find_map(|a| match a {
                    Arg::Named(k, v) if k == "language" => Some(value_to_string(&ctx.eval_expr(v))),
                    _ => None,
                })
            } else {
                None
            };
            let inner = match language {
                Some(lang) => crate::codegen::highlight::highlight(text, &lang),
                None => html_escape(text),
            };
            return format!(
                "{}<{}{}>{}</{}>\n",
                indent, actual_tag, attrs_str, inner, actual_tag
            );
        }
    }

    let mut result = format!("{}<{}{}>\n", indent, actual_tag, attrs_str);

    if let Some(cap) = &caption {
        result.push_str(&format!(
            "{}    <caption class=\"wf-visually-hidden\">{}</caption>\n",
            indent,
            html_escape(cap)
        ));
    }

    if let Some(text) = &text_content {
        result.push_str(&format!("{}    {}\n", indent, html_escape(text)));
    }

    ctx.indent += 1;
    let was_in_thead = ctx.in_thead;
    if name == "Thead" {
        ctx.in_thead = true;
    } else if name == "Tbody" {
        ctx.in_thead = false;
    }
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.in_thead = was_in_thead;
    ctx.indent -= 1;
    result.push_str(&format!("{}</{}>\n", indent, actual_tag));
    result
}

/// An element's `style { }` block as a ready-to-append ` style="…"` attribute,
/// or the empty string if it has none.
fn style_block_attr(ui: &UIElement, ctx: &mut RenderContext) -> String {
    let Some(block) = &ui.style_block else {
        return String::new();
    };
    let decls: Vec<String> = block
        .properties
        .iter()
        .filter(|p| crate::codegen::scoped_css::static_declaration(p).is_none())
        .map(|p| format!("{}: {}", p.name, value_to_string(&ctx.eval_expr(&p.value))))
        .collect();
    if decls.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", html_escape(&decls.join("; ")))
    }
}

/// `Slider`, `DatePicker` and `FileUpload`: a wrapper carrying the component
/// class, an optional `<label>`, and the input itself — the shape every renderer
/// agrees on.
fn render_labelled_input(
    name: &str,
    class_str: &str,
    style_attr: &str,
    ui: &UIElement,
    ctx: &mut RenderContext,
) -> String {
    let named = |key: &str| -> Option<String> {
        let arg = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == key => Some(v.clone()),
            _ => None,
        })?;
        Some(value_to_string(&ctx.eval_expr(&arg)))
    };

    let indent = ctx.indent_str();
    let label = named("label");
    let (min, max, step, accept, value) = (
        named("min"),
        named("max"),
        named("step"),
        named("accept"),
        named("value"),
    );

    let mut out = format!("{}<div class=\"{}\"{}>\n", indent, class_str, style_attr);
    if let Some(l) = label {
        out.push_str(&format!(
            "{}    <label class=\"wf-form-label\">{}</label>\n",
            indent,
            html_escape(&l)
        ));
    }

    let mut attrs = match name {
        "Slider" => vec![
            "type=\"range\"".to_string(),
            format!("min=\"{}\"", min.clone().unwrap_or_else(|| "0".into())),
            format!("max=\"{}\"", max.clone().unwrap_or_else(|| "100".into())),
            format!("step=\"{}\"", step.unwrap_or_else(|| "1".into())),
        ],
        "DatePicker" => {
            let mut a = vec![
                "type=\"date\"".to_string(),
                "class=\"wf-input\"".to_string(),
            ];
            if let Some(v) = min {
                a.push(format!("min=\"{}\"", html_escape(&v)));
            }
            if let Some(v) = max {
                a.push(format!("max=\"{}\"", html_escape(&v)));
            }
            a
        }
        _ => {
            let mut a = vec![
                "type=\"file\"".to_string(),
                "class=\"wf-input\"".to_string(),
            ];
            if let Some(v) = accept {
                a.push(format!("accept=\"{}\"", html_escape(&v)));
            }
            a
        }
    };
    if let Some(v) = value {
        attrs.push(format!("value=\"{}\"", html_escape(&v)));
    }
    if ui.modifiers.iter().any(|m| m == "multiple") {
        attrs.push("multiple".to_string());
    }

    out.push_str(&format!("{}    <input {}>\n", indent, attrs.join(" ")));
    out.push_str(&format!("{}</div>\n", indent));
    out
}

/// The classes an element carries beyond its base and modifier classes: the
/// one its style block's rules live under, and the ones its `class:`
/// argument names.
fn extra_classes(ui: &UIElement, ctx: &RenderContext) -> Vec<String> {
    let mut classes: Vec<String> = ui
        .style_block
        .as_ref()
        .and_then(crate::codegen::scoped_css::scoped_class)
        .into_iter()
        .collect();
    if let Some(Arg::Named(_, value)) = ui
        .args
        .iter()
        .find(|a| matches!(a, Arg::Named(k, _) if k == "class"))
    {
        class_names(&ctx.eval_expr(value), &mut classes);
    }
    classes
}

/// Whether a language is written right to left.
pub fn is_rtl(lang: &str) -> bool {
    matches!(
        lang.split(['-', '_']).next(),
        Some("ar" | "he" | "fa" | "ur" | "ps" | "yi" | "dv" | "ku")
    )
}

/// A local only a paged render sets: inside a running element, `page` and
/// `pages` are the marks the engine replaces on each page.
const PAGED: &str = "\u{0}paged";

/// A `Chart`'s arguments, read from the data.
fn chart_of(ui: &UIElement, ctx: &mut RenderContext) -> crate::codegen::charts::Chart {
    let named: Vec<(String, Value)> = ui
        .args
        .iter()
        .filter_map(|a| match a {
            Arg::Named(k, e) => Some((k.clone(), ctx.eval_expr(e))),
            _ => None,
        })
        .collect();
    crate::codegen::charts::chart_from(&named, &ui.modifiers)
}

/// The paged and slide elements: their markup, with what the engine reads
/// from it as attributes. `None` for any other element.
fn render_paged(
    name: &str,
    ui: &UIElement,
    ctx: &mut RenderContext,
    class_str: &str,
    style_attr: &str,
) -> Option<String> {
    let named = |ctx: &mut RenderContext, key: &str| -> Option<String> {
        ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == key => {
                let v = ctx.eval_expr(v);
                match v {
                    Value::Null | Value::Bool(false) => None,
                    Value::Bool(true) => Some(String::new()),
                    other => Some(value_to_string(&other)),
                }
            }
            _ => None,
        })
    };
    let positional = |ctx: &mut RenderContext| -> Option<String> {
        ui.args.iter().find_map(|a| match a {
            Arg::Positional(e) => Some(value_to_string(&ctx.eval_expr(e))),
            _ => None,
        })
    };
    // `.case` values arrive as their name.
    let case = |s: String| s.trim_start_matches('.').to_string();
    let indent = ctx.indent_str();
    let open = |ctx: &mut RenderContext, tag: &str, attrs: Vec<(String, String)>| -> String {
        let mut a = String::new();
        if !class_str.is_empty() {
            a.push_str(&format!(" class=\"{}\"", class_str));
        }
        for (k, v) in attrs {
            a.push_str(&format!(" {k}=\"{}\"", html_escape(&v)));
        }
        let _ = ctx;
        format!("{indent}<{tag}{a}{style_attr}>\n")
    };
    let children = |ctx: &mut RenderContext| -> String {
        ctx.indent += 1;
        let html = render_statements(&ui.children, ctx);
        ctx.indent -= 1;
        html
    };
    let modifier = |m: &str| ui.modifiers.iter().any(|x| x == m);
    match name {
        "Document" => {
            let mut attrs = Vec::new();
            for (key, attr) in [
                ("size", "data-size"),
                ("page_size", "data-size"),
                ("pageSize", "data-size"),
                ("margin", "data-margin"),
                ("title", "data-title"),
                ("author", "data-author"),
                ("subject", "data-subject"),
                ("keywords", "data-keywords"),
                ("lang", "data-lang"),
            ] {
                if let Some(v) = named(ctx, key) {
                    attrs.push((attr.to_string(), case(v)));
                }
            }
            if modifier("landscape") || named(ctx, "landscape").is_some() {
                attrs.push(("data-landscape".to_string(), "true".to_string()));
            }
            // A document in a right-to-left language reads right to left.
            if let Some(lang) = named(ctx, "lang") {
                if is_rtl(&lang) {
                    attrs.push(("dir".to_string(), "rtl".to_string()));
                }
                attrs.push(("lang".to_string(), lang));
            }
            let mut out = open(ctx, "div", attrs);
            out.push_str(&children(ctx));
            out.push_str(&format!("{indent}</div>\n"));
            Some(out)
        }
        "Header" | "Footer" | "Background" => {
            let tag = match name {
                "Header" => "header",
                "Footer" => "footer",
                _ => "div",
            };
            let mut attrs = Vec::new();
            if let Some(on) = named(ctx, "on") {
                attrs.push(("data-on".to_string(), case(on)));
            }
            let paged = ctx.locals.contains_key(PAGED);
            let saved = paged.then(|| {
                (
                    ctx.locals
                        .insert("page".to_string(), Value::String('\u{F8F0}'.to_string())),
                    ctx.locals
                        .insert("pages".to_string(), Value::String('\u{F8F1}'.to_string())),
                )
            });
            let mut out = open(ctx, tag, attrs);
            out.push_str(&children(ctx));
            out.push_str(&format!("{indent}</{tag}>\n"));
            if let Some((page, pages)) = saved {
                for (k, old) in [("page", page), ("pages", pages)] {
                    match old {
                        Some(v) => ctx.locals.insert(k.to_string(), v),
                        None => ctx.locals.remove(k),
                    };
                }
            }
            Some(out)
        }
        "Watermark" => {
            let mut attrs = Vec::new();
            if let Some(on) = named(ctx, "on") {
                attrs.push(("data-on".to_string(), case(on)));
            }
            let text = positional(ctx).unwrap_or_default();
            Some(format!(
                "{}{}</div>\n",
                open(ctx, "div", attrs).trim_end_matches('\n'),
                html_escape(&text)
            ))
        }
        "PageBreak" => Some(format!(
            "{indent}<div class=\"wf-page-break\"{style_attr}></div>\n"
        )),
        "Chart" => {
            let chart = chart_of(ui, ctx);
            let mut out = open(ctx, "figure", Vec::new());
            out.push_str(&chart.svg());
            out.push_str(&format!("\n{indent}</figure>\n"));
            Some(out)
        }
        "QrCode" => {
            let value = positional(ctx)
                .or_else(|| named(ctx, "value"))
                .unwrap_or_default();
            let dark = named(ctx, "color").unwrap_or_else(|| "#000000".to_string());
            let light = named(ctx, "background").unwrap_or_else(|| "#ffffff".to_string());
            let svg = crate::codegen::charts::qr_svg(&value, &dark, &light).unwrap_or_default();
            let mut out = open(ctx, "figure", Vec::new());
            out.push_str(&svg);
            out.push_str(&format!("\n{indent}</figure>\n"));
            Some(out)
        }
        "TableOfContents" => {
            // The engine fills it in: it alone knows the page each heading
            // lands on.
            let levels = named(ctx, "levels").unwrap_or_else(|| "3".to_string());
            let title = named(ctx, "title");
            let mut out = open(ctx, "nav", vec![("data-levels".to_string(), levels)]);
            if let Some(t) = title {
                out.push_str(&format!(
                    "{indent}  <h2 class=\"wf-toc__title\">{}</h2>\n",
                    html_escape(&t)
                ));
            }
            out.push_str(&format!("{indent}</nav>\n"));
            Some(out)
        }
        "Avatar" => {
            let initials = named(ctx, "initials").unwrap_or_default();
            let mut out = open(ctx, "div", Vec::new());
            if let Some(src) = named(ctx, "src") {
                let alt = named(ctx, "alt").unwrap_or_default();
                let src = crate::codegen::url::guard(&src).to_string();
                out.push_str(&format!(
                    "{indent}  <img src=\"{}\" alt=\"{}\">\n",
                    html_escape(&src),
                    html_escape(&alt)
                ));
            } else {
                out.push_str(&format!("{indent}  {}\n", html_escape(&initials)));
            }
            out.push_str(&format!("{indent}</div>\n"));
            Some(out)
        }
        "List" => {
            // Each child is an item: one written as `List.Item` is one
            // already, anything else is wrapped in one.
            let tag = if modifier("ordered") { "ol" } else { "ul" };
            let mut out = open(ctx, tag, Vec::new());
            ctx.indent += 1;
            for child in &ui.children {
                let is_item = matches!(&child.kind, StatementKind::UIElement(c) if matches!(&c.component, ComponentRef::SubComponent(o, s) if o == "List" && s == "Item"));
                let html = render_statements(std::slice::from_ref(child), ctx);
                if is_item || html.trim().is_empty() {
                    out.push_str(&html);
                } else {
                    out.push_str(&format!(
                        "{}<li class=\"wf-list__item\">\n{}{}</li>\n",
                        ctx.indent_str(),
                        html,
                        ctx.indent_str()
                    ));
                }
            }
            ctx.indent -= 1;
            out.push_str(&format!("{indent}</{tag}>\n"));
            Some(out)
        }
        "Presentation" => {
            let mut out = open(ctx, "div", Vec::new());
            out.push_str(&children(ctx));
            out.push_str(&format!("{indent}</div>\n"));
            Some(out)
        }
        "Slide" | "TwoColumn" => {
            let mut out = open(ctx, "section", Vec::new());
            out.push_str(&children(ctx));
            out.push_str(&format!("{indent}</section>\n"));
            Some(out)
        }
        "TitleSlide" => {
            let title = positional(ctx).unwrap_or_default();
            let subtitle = named(ctx, "subtitle");
            let mut out = open(ctx, "section", Vec::new());
            out.push_str(&format!(
                "{indent}  <h1 class=\"wf-slide__title\">{}</h1>\n",
                html_escape(&title)
            ));
            if let Some(sub) = subtitle {
                out.push_str(&format!(
                    "{indent}  <p class=\"wf-slide__subtitle\">{}</p>\n",
                    html_escape(&sub)
                ));
            }
            out.push_str(&format!("{indent}</section>\n"));
            Some(out)
        }
        "SectionSlide" => {
            let label = positional(ctx).unwrap_or_default();
            let mut out = open(ctx, "section", Vec::new());
            out.push_str(&format!(
                "{indent}  <h2 class=\"wf-slide__label\">{}</h2>\n",
                html_escape(&label)
            ));
            out.push_str(&format!("{indent}</section>\n"));
            Some(out)
        }
        "ImageSlide" => {
            let src = named(ctx, "src")
                .map(|s| crate::codegen::url::guard(&s).to_string())
                .unwrap_or_default();
            let caption = named(ctx, "caption");
            let mut out = open(ctx, "section", Vec::new());
            out.push_str(&format!(
                "{indent}  <img class=\"wf-slide__image\" src=\"{}\" alt=\"{}\">\n",
                html_escape(&src),
                html_escape(caption.as_deref().unwrap_or(""))
            ));
            if let Some(c) = caption {
                out.push_str(&format!(
                    "{indent}  <p class=\"wf-slide__caption\">{}</p>\n",
                    html_escape(&c)
                ));
            }
            out.push_str(&format!("{indent}</section>\n"));
            Some(out)
        }
        _ => None,
    }
}

fn render_tag(tag: &str, class: &str, ui: &UIElement, ctx: &mut RenderContext) -> String {
    let indent = ctx.indent_str();
    let class = std::iter::once(class.to_string())
        .chain(extra_classes(ui, ctx))
        .collect::<Vec<_>>()
        .join(" ");
    let mut result = format!("{}<{} class=\"{}\">\n", indent, tag, class);
    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;
    result.push_str(&format!("{}</{}>\n", indent, tag));
    result
}

/// The blocks a call of `component` fills its slots with: its own block as
/// `children`, and each named fill.
fn slot_fills(component: &TemplateComponent, ui: &UIElement) -> HashMap<String, Fill> {
    let mut slots: HashMap<String, Fill> = HashMap::new();
    slots.insert(
        "children".to_string(),
        Fill {
            params: Vec::new(),
            handed: Vec::new(),
            body: ui.children.clone(),
        },
    );
    for fill in &ui.slot_fills {
        slots.insert(
            fill.name.clone(),
            Fill {
                params: fill.params.clone(),
                handed: component
                    .handed
                    .get(&fill.name)
                    .cloned()
                    .unwrap_or_default(),
                body: fill.body.clone(),
            },
        );
    }
    slots
}

// ─── Helpers ─────────────────────────────────────────────────────────

/// What a `class:` names: a string, a map of class to condition (each key
/// whose value is truthy), or a list of either.
fn class_names(val: &Value, out: &mut Vec<String>) {
    match val {
        Value::Null | Value::Bool(false) => {}
        Value::Array(items) => items.iter().for_each(|v| class_names(v, out)),
        Value::Object(map) => {
            for (key, on) in map {
                if is_truthy(on) {
                    class_names(&Value::String(key.trim_matches('"').to_string()), out);
                }
            }
        }
        other => {
            for c in value_to_string(other).split_whitespace() {
                if !out.iter().any(|o| o == c) {
                    out.push(c.to_string());
                }
            }
        }
    }
}

fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        // A whole number prints without a fraction, as the browser prints it:
        // `8 / 2` is `4`, not `4.0`.
        Value::Number(n) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => format!("{}", i),
            (None, Some(f)) if f == f.trunc() && f.abs() < 1e15 => format!("{}", f as i64),
            _ => format!("{}", n),
        },
        Value::Bool(b) => format!("{}", b),
        Value::Null => String::new(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(val).unwrap_or_default(),
    }
}

/// The case of an enum value: a bare case is its name, a case with a payload
/// the list `["case", …payload]`.
fn case_of(v: &Value) -> Value {
    match v {
        Value::Array(items) => items.first().cloned().unwrap_or(Value::Null),
        other => other.clone(),
    }
}

/// The payload of `v` where its case is `case`: the one value of a single
/// payload, the list of a longer one, null otherwise.
fn payload_of(v: &Value, case: &Value) -> Value {
    match v {
        Value::Array(items) if items.first() == Some(case) => match items.len() {
            2 => items[1].clone(),
            _ => Value::Array(items[1..].to_vec()),
        },
        _ => Value::Null,
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(m) => !m.is_empty(),
    }
}

fn eval_binary_op(left: &Value, op: &crate::parser::ast::BinOp, right: &Value) -> Value {
    use crate::parser::ast::BinOp;

    // `2` from the data and `2.0` from a literal are the same number.
    let same = |l: &Value, r: &Value| match (l, r) {
        (Value::Number(a), Value::Number(b)) => a.as_f64() == b.as_f64(),
        _ => l == r,
    };
    match op {
        BinOp::Eq => Value::Bool(same(left, right)),
        BinOp::Neq => Value::Bool(!same(left, right)),
        BinOp::Lt => Value::Bool(as_f64(left) < as_f64(right)),
        BinOp::Gt => Value::Bool(as_f64(left) > as_f64(right)),
        BinOp::Lte => Value::Bool(as_f64(left) <= as_f64(right)),
        BinOp::Gte => Value::Bool(as_f64(left) >= as_f64(right)),
        BinOp::And => Value::Bool(is_truthy(left) && is_truthy(right)),
        BinOp::Or => Value::Bool(is_truthy(left) || is_truthy(right)),
        BinOp::NullCoalesce => {
            if left.is_null() {
                right.clone()
            } else {
                left.clone()
            }
        }
        BinOp::Add => {
            // String concatenation or numeric addition
            match (left, right) {
                (Value::String(l), _) => Value::String(format!("{}{}", l, value_to_string(right))),
                (_, Value::String(r)) => Value::String(format!("{}{}", value_to_string(left), r)),
                _ => Value::Number(
                    serde_json::Number::from_f64(as_f64(left) + as_f64(right))
                        .unwrap_or(serde_json::Number::from(0)),
                ),
            }
        }
        BinOp::Sub => Value::Number(
            serde_json::Number::from_f64(as_f64(left) - as_f64(right))
                .unwrap_or(serde_json::Number::from(0)),
        ),
        BinOp::Mul => Value::Number(
            serde_json::Number::from_f64(as_f64(left) * as_f64(right))
                .unwrap_or(serde_json::Number::from(0)),
        ),
        BinOp::Div => {
            let r = as_f64(right);
            if r == 0.0 {
                Value::Null
            } else {
                Value::Number(
                    serde_json::Number::from_f64(as_f64(left) / r)
                        .unwrap_or(serde_json::Number::from(0)),
                )
            }
        }
        BinOp::Mod => {
            let r = as_f64(right);
            if r == 0.0 {
                Value::Null
            } else {
                Value::Number(
                    serde_json::Number::from_f64(as_f64(left) % r)
                        .unwrap_or(serde_json::Number::from(0)),
                )
            }
        }
    }
}

fn as_f64(val: &Value) -> f64 {
    match val {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        Value::Bool(true) => 1.0,
        _ => 0.0,
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        // Also `'`: the renderers quote every attribute with `"`, but that is an
        // invariant nothing enforced, and one single-quoted attribute would have
        // turned this into an injection point.
        .replace('\'', "&#x27;")
}

fn camel_to_kebab(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}
