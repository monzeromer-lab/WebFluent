use crate::diagnostics::error::{Result, WebFluentError};
use crate::project::builtin::{
    builtin_to_html, class_list, element_tag, implicit_role, input_type, landmark_label,
    layout_arg_classes, span_class,
};
use crate::project::config::project::{PdfConfig, SlidesConfig};
use crate::syntax::parser::ast::{ArmPattern, DerivedDecl, ForStmt, IfStmt, PropDecl};
use crate::syntax::parser::{
    Arg, ComponentRef, Declaration, Expr, Program, Statement, StatementKind, StringPart, UIElement,
};
use crate::web::codegen::css::generate_css;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

#[cfg(test)]
mod corpus_tests;
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
    /// The document's language, for `<html lang>`, when
    /// [`with_lang`](Template::with_lang) set one; the drawn page's `lang:`,
    /// else `en`, otherwise.
    lang: Option<String>,
    pdf: PdfConfig,
    slides: SlidesConfig,
    /// Where the template's files are: what an image's address and the
    /// `fonts/` directory are read from.
    root: Option<std::path::PathBuf>,
    /// Whether each element carries its node id, as the builds' pages do
    /// for the studio: `data-wf-node`, and `data-wf-call` on what a
    /// component's call drew.
    studio: bool,
}

impl std::fmt::Debug for Template {
    /// What the template is — its pages and the choices made on it — not
    /// its whole program.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template")
            .field("pages", &self.pages())
            .field("page", &self.page)
            .field("theme", &self.theme)
            .field("custom_tokens", &self.custom_tokens)
            .field("lang", &self.lang)
            .field("root", &self.root)
            .field("studio", &self.studio)
            .finish_non_exhaustive()
    }
}

// `Template::from_str` is documented public API used throughout the README and
// the docs site; it predates any `FromStr` impl and renaming it would break every
// consumer to satisfy a naming lint.
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
    /// if the source is invalid (and [`WebFluentError::Diagnostics`] when
    /// several declarations do not parse), and
    /// [`WebFluentError::Diagnostics`] with every error finding if it does
    /// not check.
    #[expect(
        clippy::should_implement_trait,
        reason = "documented API since 1.0; `FromStr` cannot return the crate's own error kinds by this name"
    )]
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
                crate::syntax::layout::to_braces(&source, &label)?
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
        let errors: Vec<_> = crate::web::check::program_checks(&program, &file_of, &|_| None)
            .into_iter()
            .filter(|d| d.is_error() && d.code != "T13")
            .collect();
        if !errors.is_empty() {
            return Err(WebFluentError::Diagnostics(errors));
        }
        match root {
            Some(root) => crate::project::data::resolve_data(&mut program, root)?,
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
            program: Arc::new(crate::check::sema::lower(program)),
            page: None,
            theme: None,
            custom_tokens: HashMap::new(),
            lang: None,
            pdf: PdfConfig::default(),
            slides: SlidesConfig::default(),
            root: root.map(|r| r.to_path_buf()),
            studio: false,
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

    /// Mark each element with its node id — the same ids the builds give a
    /// page — so an editor showing the render can tell what was drawn by
    /// what: `data-wf-node="Report:0.1"` on an element, `data-wf-call` on
    /// the root a component's call drew.
    #[must_use]
    pub fn with_studio(mut self, on: bool) -> Self {
        self.studio = on;
        self
    }

    /// Select which `Theme` declared in the template to render with.
    ///
    /// Only needed when the source declares more than one. A template with a
    /// single `Theme` uses it automatically, and one with none renders on the
    /// baseline tokens.
    #[must_use]
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = Some(theme.to_string());
        self
    }

    /// Override design tokens (builder pattern).
    ///
    /// Common tokens: `"color-primary"`, `"color-secondary"`, `"font-family"`,
    /// `"radius-md"`, `"spacing-md"`, etc.
    #[must_use]
    pub fn with_tokens(mut self, tokens: &[(&str, &str)]) -> Self {
        for (k, v) in tokens {
            self.custom_tokens.insert(k.to_string(), v.to_string());
        }
        self
    }

    /// The language a document from [`render_html`](Template::render_html)
    /// declares — the page's own `lang:`, else `en`, unless set:
    /// `<html lang="ar">`, which also turns the document right to left for
    /// an RTL language.
    #[must_use]
    pub fn with_lang(mut self, lang: &str) -> Self {
        self.lang = Some(lang.to_string());
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
    #[must_use]
    pub fn with_pdf(mut self, config: PdfConfig) -> Self {
        self.pdf = config;
        self
    }

    /// The slide size, margin and chrome [`render_slides`](Template::render_slides)
    /// uses: 16:9 unless set.
    #[must_use]
    pub fn with_slides(mut self, config: SlidesConfig) -> Self {
        self.slides = config;
        self
    }

    /// The stylesheet a rendered document links: the engine's sheet over the
    /// template's tokens, plus the rules its pseudo-state and media blocks
    /// compile to.
    fn stylesheet(&self) -> Result<String> {
        let mut css = generate_css(&self.tokens()?);
        css.push_str(&crate::project::scoped_css::scoped_rules(&self.program));
        Ok(css)
    }

    /// The design tokens this template renders with.
    fn tokens(&self) -> Result<HashMap<String, String>> {
        let config = crate::project::config::project::ThemeConfig {
            name: self.theme.clone(),
            tokens: self.custom_tokens.clone(),
            builtin: Default::default(),
            dark: None,
        };
        crate::project::themes::resolve_tokens(&self.program, &config)
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

    /// The document's language: the one [`with_lang`](Template::with_lang)
    /// set, else the drawn page's `lang:`, else `en`.
    fn document_lang(&self) -> String {
        if let Some(lang) = &self.lang {
            return lang.clone();
        }
        self.drawn()
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Page(p) => Some(p.lang.clone()),
                _ => None,
            })
            .flatten()
            .unwrap_or_else(|| "en".to_string())
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
        let doc_lang = self.document_lang();
        let lang = html_escape(&doc_lang);
        let dir = if is_rtl(&doc_lang) {
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
        let drawn = self.drawn();
        if !self.studio {
            return render_program_fragment(&drawn, data);
        }
        // The ids travel with this render's context, so nothing of one
        // render outlives it — a render that panics on a server's thread
        // leaves no ids behind for the next request.
        let mut nodes = HashMap::new();
        crate::syntax::node_id::visit_nodes(&drawn, &mut |ui, id, _| {
            nodes.insert((ui.span.start, ui.span.end, ui.span.line), id.to_string());
        });
        render_program_with(
            &drawn,
            data,
            Rendering {
                studio: Some(std::rc::Rc::new(nodes)),
                ..Rendering::default()
            },
        )
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
        let doc_lang = self.document_lang();
        let lang = html_escape(&doc_lang);
        let dir = if is_rtl(&doc_lang) {
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
    render_program(program, data, false, Default::default())
}

/// As [`render_program_fragment`], with the messages `t("key")` reads,
/// in English.
pub fn render_program_fragment_with(
    program: &Program,
    data: &Value,
    messages: HashMap<String, String>,
) -> Result<String> {
    render_program_fragment_in(program, data, messages, "en")
}

/// As [`render_program_fragment_with`], the messages being `locale`'s: a
/// `t()` with a `count` picks the form that locale's plural rules give it.
pub fn render_program_fragment_in(
    program: &Program,
    data: &Value,
    messages: HashMap<String, String>,
    locale: &str,
) -> Result<String> {
    let mut ctx_locale = locale.to_string();
    if ctx_locale.is_empty() {
        ctx_locale = "en".to_string();
    }
    render_program_in(program, data, false, std::rc::Rc::new(messages), ctx_locale)
}

/// The markup a paged output lays out: as [`render_program_fragment`], with
/// `page` and `pages` in a running element written as the marks each page
/// replaces with its number.
pub fn render_program_paged(program: &Program, data: &Value) -> Result<String> {
    render_program(program, data, true, Default::default())
}

fn render_program(
    program: &Program,
    data: &Value,
    paged: bool,
    messages: std::rc::Rc<HashMap<String, String>>,
) -> Result<String> {
    render_program_in(program, data, paged, messages, "en".to_string())
}

fn render_program_in(
    program: &Program,
    data: &Value,
    paged: bool,
    messages: std::rc::Rc<HashMap<String, String>>,
    locale: String,
) -> Result<String> {
    render_program_with(
        program,
        data,
        Rendering {
            paged,
            messages,
            locale,
            studio: None,
        },
    )
}

/// How one render is asked for.
#[derive(Default)]
struct Rendering {
    paged: bool,
    messages: std::rc::Rc<HashMap<String, String>>,
    locale: String,
    /// The node ids to stamp, for [`Template::with_studio`].
    studio: Option<std::rc::Rc<NodeIds>>,
}

fn render_program_with(program: &Program, data: &Value, how: Rendering) -> Result<String> {
    let mut ctx = RenderContext::for_program(program, data);
    ctx.messages = how.messages;
    ctx.locale = std::rc::Rc::from(if how.locale.is_empty() {
        "en"
    } else {
        how.locale.as_str()
    });
    // The constants were worked out before the messages were known; the
    // data frame is built again with them.
    ctx.data_static = Default::default();
    ctx.studio = how.studio;
    if how.paged {
        ctx.locals.insert(PAGED.to_string(), Value::Bool(true));
    }

    let mut html = String::new();
    for decl in &program.declarations {
        if let Declaration::Page(page) = decl {
            html.push_str(&render_statements(&page.body, &mut ctx));
        }
    }
    match ctx.budget.failure.take() {
        Some(found) => Err(WebFluentError::Diagnostics(vec![found])),
        None => Ok(html),
    }
}

// ─── Render Context ──────────────────────────────────────────────────

/// The block a caller wrote for a slot: the names it gives the slot's
/// values, what the slot's declaration calls them, and the block.
#[derive(Debug, Clone)]
struct Fill {
    params: Vec<String>,
    handed: Vec<String>,
    body: Vec<Statement>,
    /// The names in reach where the block was written: the caller's, not
    /// the component's that draws it.
    scope: Locals,
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
    locals: Locals,
    /// The `derived` values that are functions — `derived countOf = (id) =>
    /// …` — by name, applied where they are called.
    functions: Functions,
    components: HashMap<String, TemplateComponent>,
    indent: usize,
    /// Inside a `Thead`, a `Tcell` is a column header (`<th scope="col">`).
    in_thead: bool,
    /// The caller's block for each user component being expanded, innermost
    /// last; `children` renders the top one.
    slots: Vec<HashMap<String, Fill>>,
    /// Fields rendered so far, for their ids.
    fields: usize,
    /// Modals and dialogs drawn so far, for their headings' ids.
    dialogs: usize,
    /// The messages `t("key")` reads: the project's default locale, when it
    /// has translations. Without them `t` shows its key.
    messages: std::rc::Rc<HashMap<String, String>>,
    /// The locale those messages are, whose plural rules pick a form.
    locale: std::rc::Rc<str>,
    /// What this render may still do, shared by every context made from
    /// it.
    budget: std::rc::Rc<Budget>,
    /// How many elements — and the components they expand to — enclose the
    /// one being drawn.
    depth: usize,
    /// The node ids to stamp, for [`Template::with_studio`].
    studio: Option<std::rc::Rc<NodeIds>>,
    /// The data as the build-time evaluator reads it, converted once per
    /// render rather than once per evaluation.
    data_static: std::rc::Rc<std::cell::OnceCell<DataFrame>>,
}

/// The names a place in a render binds — loop items, props, states — in
/// layers, innermost last: a component's props are a layer of their own,
/// so keeping the caller's names for its blocks copies pointers, not
/// values. Each layer is held as JSON for the renderer and, converted once
/// when a name is bound, as the build-time evaluator's frame.
#[derive(Debug, Clone)]
struct Locals {
    values: Vec<Arc<HashMap<String, Value>>>,
    frames: Vec<Arc<HashMap<String, crate::project::static_eval::Static>>>,
}

impl Default for Locals {
    fn default() -> Self {
        Self {
            values: vec![Arc::default()],
            frames: vec![Arc::default()],
        }
    }
}

impl Locals {
    /// Bind `name` in the innermost layer, returning what it held.
    fn insert(&mut self, name: String, value: Value) -> Option<Value> {
        let old = self.get(&name).cloned();
        if let Some(frame) = self.frames.last_mut() {
            Arc::make_mut(frame).insert(
                name.clone(),
                crate::project::static_eval::Static::from_json(&value),
            );
        }
        if let Some(values) = self.values.last_mut() {
            Arc::make_mut(values).insert(name, value);
        }
        old
    }

    /// Unbind `name` from the innermost layer, returning what it held.
    fn remove(&mut self, name: &str) -> Option<Value> {
        if let Some(frame) = self.frames.last_mut() {
            Arc::make_mut(frame).remove(name);
        }
        self.values
            .last_mut()
            .and_then(|values| Arc::make_mut(values).remove(name))
    }

    /// What `name` holds, from the innermost layer that binds it.
    fn get(&self, name: &str) -> Option<&Value> {
        self.values.iter().rev().find_map(|layer| layer.get(name))
    }

    fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    fn extend(&mut self, names: impl IntoIterator<Item = (String, Value)>) {
        for (name, value) in names {
            self.insert(name, value);
        }
    }

    /// Open a layer of names over these.
    fn push(&mut self) {
        self.values.push(Arc::default());
        self.frames.push(Arc::default());
    }

    /// Close the innermost layer.
    fn pop(&mut self) {
        self.values.pop();
        self.frames.pop();
    }
}

/// The `derived` values and constants that are functions, by name, and the
/// same as the evaluator's frame of lambdas.
#[derive(Debug, Clone, Default)]
struct Functions {
    exprs: HashMap<String, Expr>,
    frame: Arc<HashMap<String, crate::project::static_eval::Static>>,
}

impl Functions {
    fn insert(&mut self, name: String, lambda: Expr) -> Option<Expr> {
        Arc::make_mut(&mut self.frame).insert(
            name.clone(),
            crate::project::static_eval::Static::Lambda(crate::project::static_eval::Lambda(
                Arc::new(lambda.clone()),
            )),
        );
        self.exprs.insert(name, lambda)
    }

    fn remove(&mut self, name: &str) -> Option<Expr> {
        Arc::make_mut(&mut self.frame).remove(name);
        self.exprs.remove(name)
    }
}

/// The data, as the build-time evaluator's outermost scope.
type DataFrame = Arc<HashMap<String, crate::project::static_eval::Static>>;

/// What one render may do: draw so many elements and loop items, work so
/// much out, and write so much markup. A render past any of them stops
/// with an error rather than growing without end — a `for` over a range
/// the data names, nested in another, can otherwise ask for more than a
/// server has.
#[derive(Default)]
struct Budget {
    /// Where the render is: the element being drawn, where a stop is
    /// reported.
    at: std::cell::Cell<crate::syntax::parser::ast::Span>,
    steps: std::cell::Cell<u64>,
    bytes: std::cell::Cell<usize>,
    failure: std::cell::RefCell<Option<crate::diagnostics::error::Diagnostic>>,
}

/// Elements drawn, loop items, and units of evaluation one render may
/// spend: a table of a million cells fits several times over.
pub const RENDER_STEPS: u64 = 10_000_000;
/// The markup one render may write.
pub const RENDER_BYTES: usize = 256 * 1024 * 1024;

impl Budget {
    /// Whether the render has stopped.
    fn stopped(&self) -> bool {
        self.failure.borrow().is_some()
    }

    /// Stop the render with `code` and `message`, at `span`, unless it has
    /// stopped already.
    fn stop(&self, code: &'static str, message: String, span: crate::syntax::parser::ast::Span) {
        let mut failure = self.failure.borrow_mut();
        if failure.is_none() {
            *failure = Some(crate::diagnostics::error::Diagnostic::coded(
                code,
                message,
                "<template>",
                span.line as usize,
                span.col as usize,
            ));
        }
    }

    /// Spend `n` steps at `span`; `false` once the render has stopped.
    fn spend(&self, n: u64, span: crate::syntax::parser::ast::Span) -> bool {
        if self.stopped() {
            return false;
        }
        let steps = self.steps.get().saturating_add(n);
        self.steps.set(steps);
        if steps > RENDER_STEPS {
            self.stop(
                "E121",
                format!(
                    "The render would do more than {RENDER_STEPS} steps — elements, loop items and evaluation"
                ),
                span,
            );
            return false;
        }
        true
    }

    /// Account for `n` bytes of markup written at `span`.
    fn write(&self, n: usize, span: crate::syntax::parser::ast::Span) {
        let bytes = self.bytes.get().saturating_add(n);
        self.bytes.set(bytes);
        if bytes > RENDER_BYTES {
            self.stop(
                "E121",
                format!(
                    "The render would write more than {} MB of markup",
                    RENDER_BYTES / (1024 * 1024)
                ),
                span,
            );
        }
    }
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
                // A constant that is a function is called like a derived
                // one: `const label = (n) => …`, then `label(3)`.
                Declaration::Const(c) if matches!(c.value, Expr::Lambda(..)) => {
                    ctx.functions.insert(c.name.clone(), c.value.clone());
                }
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
            locals: Locals::default(),
            functions: Functions::default(),
            components: HashMap::new(),
            indent: 1,
            in_thead: false,
            slots: Vec::new(),
            fields: 0,
            dialogs: 0,
            messages: Default::default(),
            locale: std::rc::Rc::from("en"),
            budget: Default::default(),
            depth: 0,
            studio: None,
            data_static: Default::default(),
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    /// Evaluate an expression against the data context.
    /// A `to:` put together as the live page puts it: each spliced value
    /// encoded where it stands for one.
    fn address(&self, expr: &Expr) -> String {
        match expr {
            Expr::InterpolatedString(parts) => {
                let pieces: Vec<crate::project::url::AddressPart> = parts
                    .iter()
                    .map(|p| match p {
                        StringPart::Literal(t) => crate::project::url::AddressPart::Text(t.clone()),
                        StringPart::Expression(e) => crate::project::url::AddressPart::Value(
                            value_to_string(&self.eval_expr(e)),
                        ),
                    })
                    .collect();
                crate::project::url::join_address(&pieces)
            }
            other => value_to_string(&self.eval_expr(other)),
        }
    }

    /// What `expr` comes to over the data and the names in reach: the
    /// build's evaluator, reading as a template reads ([`Scope::lenient`]),
    /// so an expression means one thing in a template, a build and a page.
    ///
    /// [`Scope::lenient`]: crate::project::static_eval::Scope::lenient
    fn eval_expr(&self, expr: &Expr) -> Value {
        self.eval_static(expr)
    }

    /// Evaluate `expr` with the static evaluator, over this context's data
    /// and locals. What it spends comes out of the render's budget; an
    /// evaluation that runs out of its own fuel stops the render, rather
    /// than showing nothing where a value was asked for.
    fn eval_static(&self, expr: &Expr) -> Value {
        use crate::project::static_eval::{Scope, Static, eval_metered};
        if self.budget.stopped() {
            return Value::Null;
        }
        let data = self.data_static.get_or_init(|| {
            let mut frame: HashMap<String, Static> = match self.data {
                Value::Object(map) => map
                    .iter()
                    .map(|(k, v)| (k.clone(), Static::from_json(v)))
                    .collect(),
                _ => HashMap::new(),
            };
            // What `t(…)` reads, and the locale a render was asked for.
            frame.insert(
                "__messages".to_string(),
                Static::Map(
                    self.messages
                        .iter()
                        .map(|(k, v)| (k.clone(), Static::Str(v.clone())))
                        .collect(),
                ),
            );
            // The locale those messages are in, whose plural rules pick a
            // form; `format` and `ago` speak the data's `locale`.
            frame.insert(
                "__messages_locale".to_string(),
                Static::Str(self.locale.to_string()),
            );
            Arc::new(frame)
        });
        // The locals and the derived functions are frames of their own,
        // converted once when each name is bound: an evaluation copies
        // none of them, however long a list a prop holds.
        let mut frames = Vec::with_capacity(self.locals.frames.len() + 2);
        frames.push(data.clone());
        frames.extend(self.locals.frames.iter().cloned());
        frames.push(self.functions.frame.clone());
        let scope = Scope::over_frames(frames).lenient();
        let metered = eval_metered(expr, &scope);
        let span = self.budget.at.get();
        if metered.exhausted {
            self.budget.stop(
                "E121",
                "An expression would do more work, or make more, than one evaluation may"
                    .to_string(),
                span,
            );
            return Value::Null;
        }
        self.budget.spend(u64::from(metered.spent), span);
        metered.value.map(|v| v.to_json()).unwrap_or(Value::Null)
    }
}

// ─── HTML Rendering ──────────────────────────────────────────────────

/// A body's `state` and `derived` values, as a static render sees them: a
/// state is its first value and a derived value what that works out to —
/// on paper nothing changes. Data handed to the render wins over either.
/// Derived values are worked out twice, so one may read one declared below
/// it. What was shadowed comes back when the body is done.
fn seed_values(stmts: &[Statement], ctx: &mut RenderContext) -> Seeded {
    let mut saved = Vec::new();
    // A derived value that is a function is held as one, for its calls.
    let mut functions = Vec::new();
    for stmt in stmts {
        if let StatementKind::Derived(d) = &stmt.kind
            && matches!(d.value, Expr::Lambda(..))
        {
            functions.push((
                d.name.clone(),
                ctx.functions.insert(d.name.clone(), d.value.clone()),
            ));
        }
    }
    let given = |ctx: &RenderContext, name: &str| matches!(ctx.data, Value::Object(m) if m.contains_key(name));
    for stmt in stmts {
        if let StatementKind::State(st) = &stmt.kind
            && !given(ctx, &st.name)
        {
            let v = ctx.eval_expr(&st.value);
            saved.push((st.name.clone(), ctx.locals.insert(st.name.clone(), v)));
        }
    }
    let derived: Vec<&DerivedDecl> = stmts
        .iter()
        .filter_map(|s| match &s.kind {
            StatementKind::Derived(d)
                if !given(ctx, &d.name) && !matches!(d.value, Expr::Lambda(..)) =>
            {
                Some(d)
            }
            _ => None,
        })
        .collect();
    for pass in 0..2 {
        for d in &derived {
            let v = ctx.eval_expr(&d.value);
            let old = ctx.locals.insert(d.name.clone(), v);
            if pass == 0 {
                saved.push((d.name.clone(), old));
            }
        }
    }
    Seeded {
        values: saved,
        functions,
    }
}

/// What seeding a body shadowed, to put back when the body is done.
struct Seeded {
    values: Vec<(String, Option<Value>)>,
    functions: Vec<(String, Option<Expr>)>,
}

impl Seeded {
    fn restore(self, ctx: &mut RenderContext) {
        for (name, old) in self.values.into_iter().rev() {
            match old {
                Some(v) => ctx.locals.insert(name, v),
                None => ctx.locals.remove(&name),
            };
        }
        for (name, old) in self.functions.into_iter().rev() {
            match old {
                Some(f) => ctx.functions.insert(name, f),
                None => ctx.functions.remove(&name),
            };
        }
    }
}

fn render_statements(stmts: &[Statement], ctx: &mut RenderContext) -> String {
    let seeded = seed_values(stmts, ctx);
    let html = render_statements_seeded(stmts, ctx);
    seeded.restore(ctx);
    html
}

fn render_statements_seeded(stmts: &[Statement], ctx: &mut RenderContext) -> String {
    let mut html = String::new();
    for stmt in stmts {
        if ctx.budget.stopped() {
            break;
        }
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
            // State and derived values are seeded above; effects, actions,
            // handlers and navigation do nothing on paper.
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
            if !ctx.budget.spend(1, ctx.budget.at.get()) {
                break;
            }
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

/// Each element's node id, by where it is written: start, end, line.
type NodeIds = HashMap<(u32, u32, u32), String>;

fn render_ui_element(ui: &UIElement, ctx: &mut RenderContext) -> String {
    // Elements inside elements, and the components they expand to — a
    // component that places itself, as deep as its data goes — are held to
    // the depth the parser holds a program to.
    if ctx.depth >= crate::syntax::parser::MAX_NESTING {
        ctx.budget.stop(
            "E120",
            format!(
                "{}: elements and the components they place nest more than {} levels deep",
                crate::syntax::parser::NESTING_MESSAGE,
                crate::syntax::parser::MAX_NESTING
            ),
            ui.span,
        );
        return String::new();
    }
    ctx.budget.at.set(ui.span);
    if !ctx.budget.spend(1, ui.span) {
        return String::new();
    }
    let written = ctx.budget.bytes.get();
    ctx.depth += 1;
    let html = render_ui_element_unmarked(ui, ctx);
    ctx.depth -= 1;
    // What this element wrote beyond what the elements inside it did.
    let inner = ctx.budget.bytes.get() - written;
    ctx.budget.write(html.len().saturating_sub(inner), ui.span);
    let id = ctx
        .studio
        .as_ref()
        .and_then(|n| n.get(&(ui.span.start, ui.span.end, ui.span.line)).cloned());
    match id {
        Some(id) => {
            let attr = if matches!(ui.component, ComponentRef::UserDefined(_)) {
                "data-wf-call"
            } else {
                "data-wf-node"
            };
            on_root(html, &format!(" {attr}=\"{}\"", html_escape(&id)), &[])
        }
        None => html,
    }
}

fn render_ui_element_unmarked(ui: &UIElement, ctx: &mut RenderContext) -> String {
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
                let href = crate::project::url::guard(&ctx.address(&dest)).to_string();
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
                // What the call puts on the root, read where it is written.
                let (root_attrs, root_classes) = call_root_attrs(&component, ui, ctx);
                // The caller's names, for its blocks: what it fills a slot
                // with reads them, not the component's props.
                let caller = ctx.locals.clone();
                // The props as a layer of names, closed afterwards.
                let props = ctx.props_of(&component, ui);
                ctx.locals.push();
                ctx.locals.extend(props);
                ctx.slots.push(slot_fills(&component, ui, caller));
                let html = render_statements(&component.body, ctx);
                ctx.slots.pop();
                ctx.locals.pop();

                on_root(html, &root_attrs, &root_classes)
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
                // A scoped slot's values, read in the component's scope,
                // under the names the fill gave them.
                let handed: Vec<(String, Value)> = fill
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, param)| {
                        let key = fill.handed.get(i).cloned().unwrap_or_else(|| param.clone());
                        let value = ui
                            .args
                            .iter()
                            .find_map(|a| match a {
                                Arg::Named(k, v) if *k == key => Some(ctx.eval_expr(v)),
                                _ => None,
                            })
                            .unwrap_or(Value::Null);
                        (param.clone(), value)
                    })
                    .collect();
                // The block is drawn where it was written: in the caller's
                // names and with the caller's own slots, then the
                // component's are put back.
                let mut scope = fill.scope.clone();
                scope.push();
                scope.extend(handed);
                let inner = std::mem::replace(&mut ctx.locals, scope);
                let frame = ctx.slots.pop();
                let html = render_statements(&fill.body, ctx);
                if let Some(frame) = frame {
                    ctx.slots.push(frame);
                }
                ctx.locals = inner;
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
                    let text = if crate::project::url::URL_ATTRS.contains(&attribute.as_str()) {
                        crate::project::url::guard(&text).to_string()
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
    // A portal draws into a node somebody else's code makes in a running
    // page; what is rendered here has only its place, as the pre-rendered
    // page does.
    if name == "Portal" {
        return format!("{}<!--wf-portal-->\n", ctx.indent_str());
    }
    let (_, base_class) = builtin_to_html(name);
    let mut classes = class_list(base_class, &ui.modifiers);
    classes.extend(layout_arg_classes(&ui.args));
    classes.extend(span_class(base_class, &ui.args));
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
                "{}<div class=\"{}\"{}{}></div>\n",
                ctx.indent_str(),
                class_str,
                own_attrs(name, ui, ctx, &[]),
                style_attr
            );
        }
        // A modal or a dialog, as the pre-render and the live page draw it:
        // its title is its heading, not the `title` attribute.
        "Modal" | "Dialog" => {
            let title = ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == "title" => Some(value_to_string(&ctx.eval_expr(v))),
                _ => None,
            });
            let id = title.as_ref().map(|_| {
                ctx.dialogs += 1;
                format!("wf-dlg-s{}", ctx.dialogs)
            });
            let own = own_attrs(name, ui, ctx, &[]);
            let indent = ctx.indent_str();
            ctx.indent += 3;
            let mut body = Vec::new();
            let mut footer = None;
            let mut heading = None;
            for child in &ui.children {
                if let StatementKind::UIElement(part) = &child.kind
                    && matches!(&part.component, ComponentRef::SubComponent(p, s) if p == name && s == "Title")
                {
                    let drawn = render_statements(&part.children, ctx);
                    heading = Some(drawn.lines().map(str::trim).collect::<Vec<_>>().join(""));
                    continue;
                }
                if let StatementKind::UIElement(part) = &child.kind
                    && matches!(&part.component, ComponentRef::SubComponent(p, s) if p == name && s == "Footer")
                {
                    footer = Some(render_statements(&part.children, ctx));
                    continue;
                }
                body.push(child.clone());
            }
            let body = render_statements(&body, ctx);
            ctx.indent -= 3;
            return crate::web::codegen::ssg::static_dialog(
                &indent,
                &crate::web::codegen::ssg::DialogPaint {
                    root_class: &class_str,
                    class: base_class,
                    attrs: &format!("{own}{style_attr}"),
                    title: title.as_deref().zip(id.as_deref()),
                    heading: heading.as_deref(),
                    closable: ui.modifiers.iter().any(|m| m == "closable"),
                    body: &body,
                    footer: footer.as_deref().map(|f| ("", f)),
                },
            );
        }
        // A menu at its button: the button, and the list hidden, as the
        // pre-render and the live page draw it.
        "Menu" | "Dropdown" => {
            let named = |ctx: &RenderContext, key: &str| {
                ui.args.iter().find_map(|a| match a {
                    Arg::Named(k, v) if k == key => Some(value_to_string(&ctx.eval_expr(v))),
                    _ => None,
                })
            };
            let visible = ui
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, _) if k == "visible"));
            let trigger = if name == "Dropdown" {
                named(ctx, "label")
            } else {
                named(ctx, "trigger")
            };
            let label = if name == "Dropdown" {
                None
            } else {
                named(ctx, "label")
            };
            let aria = label
                .map(|l| format!(" aria-label=\"{}\"", html_escape(&l)))
                .unwrap_or_default();
            // What any element takes is the menu's root's, as on the live
            // page.
            let own = own_attrs(name, ui, ctx, crate::web::codegen::js::MENU_READS);
            let gives = |key: &str| {
                ui.args
                    .iter()
                    .any(|a| matches!(a, Arg::Named(k, _) if k == key))
            };
            let items_class = format!("wf-{}__items", name.to_lowercase());
            let indent = ctx.indent_str();
            // Opened by state alone, a menu is its list and nothing else;
            // its author's `role:` and `aria-label:` take the place of its
            // own.
            if trigger.is_none() && visible {
                let classes =
                    crate::project::builtin::class_list(&items_class, &ui.modifiers).join(" ");
                let role = if gives("role") { "" } else { " role=\"menu\"" };
                let aria = if gives("aria-label") {
                    ""
                } else {
                    aria.as_str()
                };
                let mut out = format!("{indent}<ul class=\"{classes}\"{role}{aria}{own} hidden>\n");
                ctx.indent += 1;
                out.push_str(&render_statements(&ui.children, ctx));
                ctx.indent -= 1;
                out.push_str(&format!("{indent}</ul>\n"));
                return out;
            }
            // Given no words for its button, a menu's says "Menu".
            let trigger = trigger.unwrap_or_else(|| "Menu".to_string());
            let mut out = format!("{indent}<div class=\"{class_str}\"{own}{style_attr}>\n");
            ctx.indent += 1;
            let inner = ctx.indent_str();
            out.push_str(&format!(
                "{inner}<button class=\"wf-btn wf-menu__trigger\" type=\"button\" aria-haspopup=\"menu\" aria-expanded=\"false\">{}</button>\n",
                html_escape(&trigger)
            ));
            out.push_str(&format!(
                "{inner}<ul class=\"{items_class}\" role=\"menu\"{aria} hidden>\n"
            ));
            ctx.indent += 1;
            out.push_str(&render_statements(&ui.children, ctx));
            ctx.indent -= 1;
            out.push_str(&format!("{inner}</ul>\n"));
            ctx.indent -= 1;
            out.push_str(&format!("{indent}</div>\n"));
            return out;
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
            let inert = ui.args.iter().any(
                |a| matches!(a, Arg::Named(k, v) if k == "inert" && is_truthy(&ctx.eval_expr(v))),
            );
            let options = crate::web::codegen::markdown::Options {
                inert,
                ..Default::default()
            };
            return format!(
                "{}<div class=\"{}\"{}>\n{}{}</div>\n",
                ctx.indent_str(),
                class_str,
                style_attr,
                crate::web::codegen::markdown::render_with(&text, options),
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
    // A landmark is named unless its author names it, as on the live page.
    if let Some(label) = landmark_label(name)
        && !ui
            .args
            .iter()
            .any(|a| matches!(a, Arg::Named(k, _) if k == "aria-label"))
    {
        attrs.push(format!("aria-label=\"{}\"", label));
    }

    if !class_str.is_empty() {
        attrs.push(format!("class=\"{}\"", class_str));
    }

    // A `Frame` that draws HTML it is handed rather than HTML the template
    // wrote, with no `sandbox:` of its own, is sandboxed whole — as the
    // build and the live page sandbox it.
    if name == "Frame" {
        let named = |k: &str| {
            ui.args
                .iter()
                .find(|a| matches!(a, Arg::Named(n, _) if n == k))
        };
        if let Some(Arg::Named(_, srcdoc)) = named("srcdoc")
            && !matches!(srcdoc, Expr::StringLiteral(_))
            && named("sandbox").is_none()
        {
            attrs.push("sandbox=\"\"".to_string());
        }
    }

    for arg in &ui.args {
        match arg {
            Arg::Named(key, val) => match key.as_str() {
                "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max" | "step"
                | "accept" | "role" | "value" | "title" | "width" | "height" | "loading"
                | "decoding" | "fetchpriority" | "sandbox" | "srcdoc" | "allow" => {
                    let resolved = value_to_string(&ctx.eval_expr(val));
                    // A URL from the data: `javascript:` and its kind are
                    // dropped, as `WF.safeUrl` drops them on a live page.
                    // An image's may be a picture carried in its address
                    // (`data:image/…`), as on the live page.
                    let resolved = if key == "src" && crate::project::url::draws_picture(name) {
                        crate::project::url::guard_picture(&resolved).to_string()
                    } else if crate::project::url::URL_ATTRS.contains(&key.as_str()) {
                        crate::project::url::guard(&resolved).to_string()
                    } else {
                        resolved
                    };
                    attrs.push(format!("{}=\"{}\"", key, html_escape(&resolved)));
                }
                "to" => {
                    let resolved = ctx.address(val);
                    attrs.push(format!(
                        "href=\"{}\"",
                        html_escape(crate::project::url::guard(&resolved))
                    ));
                }
                // What a long list, a splitter or a carousel is, for a
                // screen reader: its accessible name, never words in it.
                "label" if matches!(name, "VirtualList" | "Splitter" | "Carousel") => {
                    if !ui
                        .args
                        .iter()
                        .any(|a| matches!(a, Arg::Named(k, _) if k == "aria-label"))
                    {
                        let s = value_to_string(&ctx.eval_expr(val));
                        attrs.push(format!("aria-label=\"{}\"", html_escape(&s)));
                    }
                }
                "label" if name == "IconButton" => {
                    // The accessible name of an icon-only button, never visible text.
                    let s = value_to_string(&ctx.eval_expr(val));
                    attrs.push(format!("aria-label=\"{}\"", html_escape(&s)));
                    // Its tooltip too, unless its author writes one.
                    if !ui
                        .args
                        .iter()
                        .any(|a| matches!(a, Arg::Named(k, _) if k == "title"))
                    {
                        attrs.push(format!("title=\"{}\"", html_escape(&s)));
                    }
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
                // There or not: `disabled: false` writes nothing.
                "required" | "disabled" | "controls" => {
                    if matches!(ctx.eval_expr(val), Value::Bool(true)) {
                        attrs.push(key.to_string());
                    }
                }
                // What a form sends the control as; an icon's is its glyph.
                "name" | "autocomplete" if name != "Icon" => {
                    let s = value_to_string(&ctx.eval_expr(val));
                    attrs.push(format!("{key}=\"{}\"", html_escape(&s)));
                }
                "icon" => {
                    let s = value_to_string(&ctx.eval_expr(val));
                    attrs.push(format!("data-icon=\"{}\"", html_escape(&s)));
                }
                // A hyphenated name — `aria-*`, `data-*` — as the live page
                // writes it: `true` is `"true"`, `false` none but an ARIA
                // state's, `null` none.
                k if k.contains('-') => {
                    if let Some(one) = crate::web::codegen::ssg::paint_attr(
                        k,
                        val,
                        &|e| painted_value(ctx.eval_expr(e)),
                        &[],
                    ) {
                        attrs.push(one);
                    }
                }
                // Anything else the component has no prop for is an
                // attribute, as `C04` says and the live page writes it: the
                // global attributes (`id`, `tabindex`, `lang`, `dir`,
                // `hidden`), `target:`, `colspan:`, an author's own.
                k if crate::registry::component(name).is_none_or(|s| s.prop(k).is_none()) => {
                    if let Some(one) = crate::web::codegen::ssg::paint_attr(
                        k,
                        val,
                        &|e| painted_value(ctx.eval_expr(e)),
                        &[],
                    ) {
                        attrs.push(one);
                    }
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
            if crate::project::scoped_css::static_declaration(prop).is_some() {
                continue;
            }
            if let Some(val) = style_value(&prop.value, ctx) {
                style_parts.push(format!("{}: {}", prop.name, val));
            }
        }
        if let Some(existing) = &inline_style {
            style_parts.insert(0, existing.clone());
        }
        if !style_parts.is_empty() {
            inline_style = Some(style_parts.join("; "));
        }
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
        if let Some(mut parts) = crate::web::codegen::ssg::field_parts(ui, resolve) {
            let (id, own_attr) = match crate::web::codegen::ssg::own_id(&attrs) {
                Some(own) => (own, String::new()),
                None => {
                    ctx.fields += 1;
                    let id = format!("wf-field-{}", ctx.fields);
                    let attr = format!(" id=\"{id}\"");
                    (id, attr)
                }
            };
            parts.set_id(&id);
            let control = if actual_tag == "input" {
                format!(
                    "<{}{}{}{}>",
                    actual_tag,
                    attrs_str,
                    own_attr,
                    parts.control_attrs()
                )
            } else {
                ctx.indent += 1;
                let inner = render_statements(&ui.children, ctx);
                ctx.indent -= 1;
                format!(
                    "<{}{}{}{}>\n{}{}</{}>",
                    actual_tag,
                    attrs_str,
                    own_attr,
                    parts.control_attrs(),
                    inner,
                    indent,
                    actual_tag
                )
            };
            return crate::web::codegen::ssg::static_field(&indent, &id, &control, &parts);
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

    if let Some(text) = &text_content
        && !has_children
    {
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
            Some(lang) => crate::web::codegen::highlight::highlight(text, &lang),
            None => html_escape(text),
        };
        return format!(
            "{}<{}{}>{}</{}>\n",
            indent, actual_tag, attrs_str, inner, actual_tag
        );
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
        .filter(|p| crate::project::scoped_css::static_declaration(p).is_none())
        .filter_map(|p| Some(format!("{}: {}", p.name, style_value(&p.value, ctx)?)))
        .collect();
    if decls.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", html_escape(&decls.join("; ")))
    }
}

/// A style value as the live page sets it: one property at a time, so what
/// the data puts in it is one value. A spliced piece that would end the
/// declaration (`;`), open or close a block (`{`, `}`), escape its way past
/// either (`\`), or fetch something (`url(`, `image-set(`, …) is refused,
/// and the declaration left out — as a browser leaves out a value it cannot
/// read. What the template itself writes around a splice is its own.
fn style_value(value: &Expr, ctx: &RenderContext) -> Option<String> {
    let checked = |e: &Expr| {
        let text = value_to_string(&ctx.eval_expr(e));
        safe_style_piece(&text).then_some(text)
    };
    match value {
        Expr::InterpolatedString(parts) => {
            let mut out = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(t) => out.push_str(t),
                    StringPart::Expression(e) => out.push_str(&checked(e)?),
                }
            }
            Some(out)
        }
        other => checked(other),
    }
}

/// Whether a piece of a style value from the data is a piece of one value.
fn safe_style_piece(text: &str) -> bool {
    if text.contains([';', '{', '}', '\\', '<', '>']) {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    // Every function that fetches what it names, and the old ones that ran
    // script.
    const FETCHING: &[&str] = &[
        "url(",
        "image(",
        "image-set(",
        "cross-fade(",
        "element(",
        "src(",
        "expression(",
        "@import",
    ];
    !FETCHING.iter().any(|f| lower.contains(f))
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

    // What is the whole control's is its box's; every other attribute is
    // its input's, as on the live page.
    let own = |keep: &dyn Fn(&str) -> bool| {
        crate::web::codegen::ssg::painted_attrs(
            crate::registry::component(name),
            ui,
            keep,
            &|e| painted_value(ctx.eval_expr(e)),
            &[],
        )
    };
    let on_box = own(&|k| crate::web::codegen::js::BOX_ATTRS.contains(&k));
    let on_input = own(&|k| {
        !crate::web::codegen::js::BOX_ATTRS.contains(&k)
            && !matches!(k, "min" | "max" | "accept" | "value" | "multiple")
    });
    let mut out = format!(
        "{}<div class=\"{}\"{}{}>\n",
        indent, class_str, on_box, style_attr
    );
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

    out.push_str(&format!(
        "{}    <input {}{}>\n",
        indent,
        attrs.join(" "),
        on_input
    ));
    out.push_str(&format!("{}</div>\n", indent));
    out
}

/// An element's opening tag with what it carries as any element does
/// (`own_attrs`) beside its class.
fn with_own(open: String, tag: &str, name: &str, ui: &UIElement, ctx: &RenderContext) -> String {
    let own = own_attrs(name, ui, ctx, &[]);
    open.replacen(&format!("<{tag}"), &format!("<{tag}{own}"), 1)
}

/// What a component's call puts on its root element, as the pre-rendered
/// page and the live one put it: each attribute the component takes no prop
/// for — `aria-*`, `data-*`, `id`, `role`, `title`, `hidden` — painted as
/// it would be on an element, and the classes a `class:` adds.
fn call_root_attrs(
    component: &TemplateComponent,
    ui: &UIElement,
    ctx: &RenderContext,
) -> (String, Vec<String>) {
    let mut attrs = String::new();
    let mut classes = Vec::new();
    for arg in &ui.args {
        let Arg::Named(key, value) = arg else {
            continue;
        };
        if component.props.iter().any(|p| &p.name == key) {
            continue;
        }
        if key == "class" {
            class_names(&ctx.eval_expr(value), &mut classes);
            continue;
        }
        if !crate::web::codegen::js::is_root_attribute(key) {
            continue;
        }
        if let Some(one) = crate::web::codegen::ssg::paint_attr(
            key,
            value,
            &|e| painted_value(ctx.eval_expr(e)),
            &[],
        ) {
            attrs.push(' ');
            attrs.push_str(&one);
        }
    }
    (attrs, classes)
}

/// `html` with `attrs` and `classes` on its first element: the classes
/// joined to its own, the attributes after its name.
fn on_root(html: String, attrs: &str, classes: &[String]) -> String {
    if attrs.is_empty() && classes.is_empty() {
        return html;
    }
    let Some(at) = crate::web::codegen::ssg::first_tag_end(&html) else {
        return html;
    };
    let close = html[at..].find('>').map_or(html.len(), |i| at + i);
    let mut html = html;
    if !classes.is_empty() {
        let joined = classes.join(" ");
        html = match html[at..close].find(" class=\"") {
            Some(i) => {
                let value = at + i + " class=\"".len();
                let end = html[value..].find('"').map_or(value, |j| value + j);
                let sep = if end > value { " " } else { "" };
                format!("{}{sep}{joined}{}", &html[..end], &html[end..])
            }
            None => format!("{} class=\"{joined}\"{}", &html[..at], &html[at..]),
        };
    }
    format!("{}{attrs}{}", &html[..at], &html[at..])
}

/// What the data gives an attribute, for `painted_attrs`: text, a boolean,
/// or nothing.
fn painted_value(v: Value) -> Option<crate::web::codegen::ssg::Painted> {
    use crate::web::codegen::ssg::Painted;
    match v {
        Value::Bool(on) => Some(Painted::Flag(on)),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
        other => Some(Painted::Text(value_to_string(&other))),
    }
}

/// What the root of a built-in drawn by its own branch carries as any
/// element does, as the live page writes it (`painted_attrs`): `reads`
/// names what the branch draws itself.
fn own_attrs(name: &str, ui: &UIElement, ctx: &RenderContext, reads: &[&str]) -> String {
    crate::web::codegen::ssg::painted_attrs(
        crate::registry::component(name),
        ui,
        &|k| !reads.contains(&k),
        &|e| painted_value(ctx.eval_expr(e)),
        &[],
    )
}

/// The classes an element carries beyond its base and modifier classes: the
/// one its style block's rules live under, the ones a responsive prop's
/// rules live under (`columns: { base: 1, md: 3 }`), and the ones its
/// `class:` argument names.
fn extra_classes(ui: &UIElement, ctx: &RenderContext) -> Vec<String> {
    let mut classes: Vec<String> = ui
        .style_block
        .as_ref()
        .and_then(crate::project::scoped_css::scoped_class)
        .into_iter()
        .collect();
    // The rules were written into the page's CSS; without the class that
    // names them, a responsive layout stayed at its narrowest.
    classes.extend(crate::project::scoped_css::responsive_classes(ui));
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
    crate::project::i18n::table::is_rtl(lang)
}

/// A local only a paged render sets: inside a running element, `page` and
/// `pages` are the marks the engine replaces on each page.
const PAGED: &str = "\u{0}paged";

/// A `Chart`'s arguments, read from the data.
fn chart_of(ui: &UIElement, ctx: &mut RenderContext) -> crate::web::codegen::charts::Chart {
    let named: Vec<(String, Value)> = ui
        .args
        .iter()
        .filter_map(|a| match a {
            Arg::Named(k, e) => Some((k.clone(), ctx.eval_expr(e))),
            _ => None,
        })
        .collect();
    crate::web::codegen::charts::chart_from(&named, &ui.modifiers)
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
            let mut out = with_own(open(ctx, "figure", Vec::new()), "figure", name, ui, ctx);
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
            let svg =
                crate::web::codegen::charts::qr_svg(&value, &dark, &light).unwrap_or_default();
            let mut out = with_own(open(ctx, "figure", Vec::new()), "figure", name, ui, ctx);
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
            let mut out = with_own(open(ctx, "div", Vec::new()), "div", name, ui, ctx);
            if let Some(src) = named(ctx, "src") {
                let alt = named(ctx, "alt").unwrap_or_default();
                let src = crate::project::url::guard_picture(&src).to_string();
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
                .map(|s| crate::project::url::guard(&s).to_string())
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
fn slot_fills(
    component: &TemplateComponent,
    ui: &UIElement,
    scope: Locals,
) -> HashMap<String, Fill> {
    let mut slots: HashMap<String, Fill> = HashMap::new();
    slots.insert(
        "children".to_string(),
        Fill {
            params: Vec::new(),
            handed: Vec::new(),
            body: ui.children.clone(),
            scope: scope.clone(),
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
                scope: scope.clone(),
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
            // Each class is written into a `class="…"`, so each is escaped:
            // a `"` from the data cannot close the attribute and open another.
            for c in value_to_string(other).split_whitespace() {
                let c = html_escape(c);
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
    }
}

/// The text a value shows as, as JavaScript writes it: a whole number with
/// no fraction, a list as its items joined with commas.
fn value_to_string(val: &Value) -> String {
    crate::project::static_eval::Static::from_json(val).js_text()
}

/// The case of an enum value: a bare case is its name, a case with a payload
/// the list `["case", …payload]`.
fn case_of(v: &Value) -> Value {
    match v {
        Value::Array(items) => items.first().cloned().unwrap_or(Value::Null),
        other => other.clone(),
    }
}

/// Whether a value counts as true, as JavaScript decides it.
fn is_truthy(val: &Value) -> bool {
    crate::project::static_eval::Static::from_json(val).truthy()
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
