use crate::base::text::camel_to_kebab;
use crate::diagnostics::error::{Result, WebFluentError};
use crate::project::builtin::{builtin_to_html, class_list, layout_arg_classes, span_class};
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
            WebFluentError::io(format!("Failed to read '{}': {}", dir.display(), e))
        })?;
        if files.is_empty() {
            return Err(WebFluentError::io(format!(
                "no .wf or .wfx template under '{}'",
                dir.display()
            )));
        }
        files.sort();
        let files: Vec<&Path> = files.iter().map(std::path::PathBuf::as_path).collect();
        // `data` files are read from the directory itself.
        Self::load(&files, Some(dir))
    }

    /// Read `files` (a `.wfx` one through its braced spelling) and build
    /// one template of them, `data` files read from `root`.
    fn load(files: &[&Path], root: Option<&Path>) -> Result<Self> {
        let mut sources = Vec::new();
        for path in files {
            let source = fs::read_to_string(path).map_err(|e| {
                WebFluentError::io(format!(
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
        crate::syntax::prefer_own_components(&mut program);
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
                    return Err(WebFluentError::io(format!(
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
            root: root.map(std::path::Path::to_path_buf),
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
            return Err(WebFluentError::codegen(format!(
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
            builtin: crate::project::themes::BuiltinCss::default(),
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
    ///
    /// # Errors
    ///
    /// [`WebFluentError::ConfigError`] when `data` does not serialize or the
    /// theme asked for is not declared, and [`WebFluentError::Diagnostics`]
    /// holding an `E120` or `E121` when the render nests too deep or does too
    /// much.
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
    ///
    /// # Errors
    ///
    /// [`WebFluentError::ConfigError`] when `data` does not serialize or the
    /// theme asked for is not declared, and [`WebFluentError::Diagnostics`]
    /// holding an `E120` or `E121` when the render nests too deep or does too
    /// much.
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
    ///
    /// # Errors
    ///
    /// [`WebFluentError::ConfigError`] when `data` does not serialize or the
    /// theme asked for is not declared, and [`WebFluentError::Diagnostics`]
    /// holding an `E120` or `E121` when the render nests too deep or does too
    /// much.
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
    ///
    /// # Errors
    ///
    /// As [`render_html`](Template::render_html), and
    /// [`WebFluentError::CodegenError`] when the paged engine cannot lay the
    /// page out.
    pub fn render_pdf<T: Serialize + ?Sized>(&self, data: &T) -> Result<Vec<u8>> {
        Ok(self.render_pdf_with_notes(data)?.0)
    }

    /// [`render_pdf`](Template::render_pdf), with what the reader of the
    /// render should know: a character no font had, a font taken from this
    /// machine rather than the template's.
    ///
    /// # Errors
    ///
    /// As [`render_html`](Template::render_html), and
    /// [`WebFluentError::CodegenError`] when the paged engine cannot lay the
    /// page out.
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
    ///
    /// # Errors
    ///
    /// As [`render_html`](Template::render_html), and
    /// [`WebFluentError::CodegenError`] when the paged engine cannot lay the
    /// page out.
    pub fn render_pdf_report<T: Serialize + ?Sized>(&self, data: &T) -> Result<PdfReport> {
        let data = to_value(data)?;
        let html = self.paged_document(&data)?;
        let read = crate::paged::reader(self.root.clone());
        let options = crate::paged::Options::from_pdf(&self.pdf, &read, self.root.as_deref());
        let out = crate::paged::render(&html, "", &options).map_err(WebFluentError::codegen)?;
        Ok(PdfReport {
            bytes: out.bytes,
            pages: out.pages,
            text: out.text,
            notes: out.notes,
        })
    }

    /// [`render_slides`](Template::render_slides), reported as
    /// [`render_pdf_report`](Template::render_pdf_report) reports a document.
    ///
    /// # Errors
    ///
    /// As [`render_html`](Template::render_html), and
    /// [`WebFluentError::CodegenError`] when the paged engine cannot lay the
    /// page out.
    pub fn render_slides_report<T: Serialize + ?Sized>(&self, data: &T) -> Result<PdfReport> {
        let data = to_value(data)?;
        let html = self.paged_document(&data)?;
        let read = crate::paged::reader(self.root.clone());
        let (options, chrome) =
            crate::paged::Options::from_slides(&self.slides, &read, self.root.as_deref());
        let out = crate::paged::render_slides(&html, "", &options, &chrome)
            .map_err(WebFluentError::codegen)?;
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
    ///
    /// # Errors
    ///
    /// As [`render_html`](Template::render_html), and
    /// [`WebFluentError::CodegenError`] when the paged engine cannot lay the
    /// page out.
    pub fn render_slides<T: Serialize + ?Sized>(&self, data: &T) -> Result<Vec<u8>> {
        let data = to_value(data)?;
        let html = self.paged_document(&data)?;
        let read = crate::paged::reader(self.root.clone());
        let (options, chrome) =
            crate::paged::Options::from_slides(&self.slides, &read, self.root.as_deref());
        let out = crate::paged::render_slides(&html, "", &options, &chrome)
            .map_err(WebFluentError::codegen)?;
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
        .map_err(|e| WebFluentError::config_with(format!("the data does not serialize: {e}"), e))
}

/// The HTML fragment of every page of a lowered program, over `data`:
/// what a template renders, and what `wf test` holds a test to.
///
/// # Errors
///
/// [`WebFluentError::Diagnostics`] holding an `E120` or `E121` when the
/// render nests too deep or does too much.
pub fn render_program_fragment(program: &Program, data: &Value) -> Result<String> {
    render_program(program, data, false, std::rc::Rc::default())
}

/// As [`render_program_fragment`], with the messages `t("key")` reads,
/// in English.
///
/// # Errors
///
/// [`WebFluentError::Diagnostics`] holding an `E120` or `E121` when the
/// render nests too deep or does too much.
pub fn render_program_fragment_with(
    program: &Program,
    data: &Value,
    messages: HashMap<String, String>,
) -> Result<String> {
    render_program_fragment_in(program, data, messages, "en")
}

/// As [`render_program_fragment_with`], the messages being `locale`'s: a
/// `t()` with a `count` picks the form that locale's plural rules give it.
///
/// # Errors
///
/// [`WebFluentError::Diagnostics`] holding an `E120` or `E121` when the
/// render nests too deep or does too much.
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
///
/// # Errors
///
/// [`WebFluentError::Diagnostics`] holding an `E120` or `E121` when the
/// render nests too deep or does too much.
pub fn render_program_paged(program: &Program, data: &Value) -> Result<String> {
    render_program(program, data, true, std::rc::Rc::default())
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
    ctx.paint.default_messages = (*how.messages).clone();
    // A template is no page of a site: no link in it is the current one.
    ctx.paint.current_path = "\u{0}".to_string();
    ctx.messages = how.messages;
    ctx.locale = std::rc::Rc::from(if how.locale.is_empty() {
        "en"
    } else {
        how.locale.as_str()
    });
    // The constants were worked out before the messages were known; the
    // data frame is built again with them.
    ctx.data_static = std::rc::Rc::default();
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
    /// What drawing a built-in reads and keeps — the indent, the counters
    /// ids are made from — shared with the build's static paint, which
    /// draws every built-in for both.
    paint: crate::web::codegen::ssg::PaintState,
    /// The caller's block for each user component being expanded, innermost
    /// last; `children` renders the top one.
    slots: Vec<HashMap<String, Fill>>,
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
            paint: crate::web::codegen::ssg::PaintState {
                indent: 1,
                ..Default::default()
            },
            slots: Vec::new(),
            messages: std::rc::Rc::default(),
            locale: std::rc::Rc::from("en"),
            budget: std::rc::Rc::default(),
            depth: 0,
            studio: None,
            data_static: std::rc::Rc::default(),
        }
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

    /// The names in reach as the build's evaluator reads them, in template
    /// mode: the data, then the locals, innermost last, then the derived
    /// functions. Shared, not copied.
    fn scope_over(&self, data: &DataFrame) -> crate::project::static_eval::Scope {
        let mut frames = Vec::with_capacity(self.locals.frames.len() + 2);
        frames.push(Arc::clone(data));
        frames.extend(self.locals.frames.iter().cloned());
        frames.push(Arc::clone(&self.functions.frame));
        crate::project::static_eval::Scope::over_frames(frames).lenient()
    }

    /// The data, as the evaluator's outermost frame, converted once per
    /// render with the messages `t(…)` reads.
    fn data_frame(&self) -> &DataFrame {
        use crate::project::static_eval::Static;
        self.data_static.get_or_init(|| {
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
                        .collect::<Vec<_>>()
                        .into(),
                ),
            );
            // The locale those messages are in, whose plural rules pick a
            // form; `format` and `ago` speak the data's `locale`.
            frame.insert(
                "__messages_locale".to_string(),
                Static::Str(self.locale.to_string()),
            );
            Arc::new(frame)
        })
    }

    /// The scope a built-in is drawn with.
    fn scope(&self) -> crate::project::static_eval::Scope {
        self.scope_over(self.data_frame())
    }

    /// Evaluate `expr` with the static evaluator, over this context's data
    /// and locals. What it spends comes out of the render's budget; an
    /// evaluation that runs out of its own fuel stops the render, rather
    /// than showing nothing where a value was asked for.
    fn eval_static(&self, expr: &Expr) -> Value {
        use crate::project::static_eval::eval_metered;
        if self.budget.stopped() {
            return Value::Null;
        }
        let data = self.data_frame();
        // The locals and the derived functions are frames of their own,
        // converted once when each name is bound: an evaluation copies
        // none of them, however long a list a prop holds.
        let metered = eval_metered(expr, &self.scope_over(data));
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
        metered.value.map_or(Value::Null, |v| v.to_json())
    }
}

impl std::ops::Deref for RenderContext<'_> {
    type Target = crate::web::codegen::ssg::PaintState;
    fn deref(&self) -> &Self::Target {
        &self.paint
    }
}

impl std::ops::DerefMut for RenderContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.paint
    }
}

/// A template draws a built-in's block with its own control flow, which
/// works every condition and list out; the names in reach come back as
/// they were for the rest of the built-in.
impl crate::web::codegen::ssg::Paint for RenderContext<'_> {
    fn statements(&mut self, stmts: &[Statement]) -> String {
        let scope = self.paint.scope.clone();
        let html = render_statements(stmts, self);
        self.paint.scope = scope;
        html
    }

    /// A value from the data, held to one declaration ([`safe_style_piece`]).
    fn style_value(&self, value: &Expr) -> Option<String> {
        style_value(value, self)
    }
}

// ─── HTML Rendering ──────────────────────────────────────────────────

/// A body's `state` and `derived` values, as a static render sees them: a
/// state is its first value and a derived value what that works out to —
/// on paper nothing changes. Data handed to the render wins over either.
/// Derived values are worked out twice, so one may read one declared below
/// it. What was shadowed comes back when the body is done.
fn seed_values(stmts: &[Statement], ctx: &mut RenderContext<'_>) -> Seeded {
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
    let given = |ctx: &RenderContext<'_>, name: &str| matches!(ctx.data, Value::Object(m) if m.contains_key(name));
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
    fn restore(self, ctx: &mut RenderContext<'_>) {
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

fn render_statements(stmts: &[Statement], ctx: &mut RenderContext<'_>) -> String {
    let seeded = seed_values(stmts, ctx);
    let html = render_statements_seeded(stmts, ctx);
    seeded.restore(ctx);
    html
}

fn render_statements_seeded(stmts: &[Statement], ctx: &mut RenderContext<'_>) -> String {
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

fn render_if(if_stmt: &IfStmt, ctx: &mut RenderContext<'_>) -> String {
    let cond = ctx.eval_expr(&if_stmt.condition);
    if is_truthy(&cond) {
        // `if let p = post { … }`: the branch reads `p` as the value.
        let Some(name) = &if_stmt.binding else {
            return render_statements(&if_stmt.then_body, ctx);
        };
        let old = ctx.locals.insert(name.clone(), cond);
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

fn render_for(for_stmt: &ForStmt, ctx: &mut RenderContext<'_>) -> String {
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

fn render_ui_element(ui: &UIElement, ctx: &mut RenderContext<'_>) -> String {
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

fn render_ui_element_unmarked(ui: &UIElement, ctx: &mut RenderContext<'_>) -> String {
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
                out.push_str(&format!("{indent}</a>\n"));
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
                ctx.slots.push(slot_fills(&component, ui, &caller));
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

fn render_builtin(name: &str, ui: &UIElement, ctx: &mut RenderContext<'_>) -> String {
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
    // What only a document has is drawn here. In a document — a PDF, a
    // deck — so are a list, an avatar, a chart and a QR code, whose shape
    // the paged engine relies on; on a web page they are drawn as the
    // build draws them.
    let paged = ctx.locals.contains_key(PAGED);
    if paged || !matches!(name, "List" | "Avatar" | "Chart" | "QrCode") {
        let (_, base_class) = builtin_to_html(name);
        let mut classes = class_list(base_class, &ui.modifiers);
        classes.extend(layout_arg_classes(&ui.args));
        classes.extend(span_class(base_class, &ui.args));
        classes.extend(extra_classes(ui, ctx));
        let class_str = classes.join(" ");
        let style_attr = style_block_attr(ui, ctx);
        if let Some(html) = render_paged(name, ui, ctx, &class_str, &style_attr) {
            return html;
        }
    }
    // Every other built-in is drawn by the build's own code, over the
    // names in reach as a template reads them: one renderer, so a template
    // and a page cannot drift apart.
    ctx.paint.scope = ctx.scope();
    crate::web::codegen::ssg::render_builtin(name, ui, ctx)
}

/// An element's `style { }` block as a ready-to-append ` style="…"` attribute,
/// or the empty string if it has none.
fn style_block_attr(ui: &UIElement, ctx: &mut RenderContext<'_>) -> String {
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
fn style_value(value: &Expr, ctx: &RenderContext<'_>) -> Option<String> {
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

    if text.contains([';', '{', '}', '\\', '<', '>']) {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    !FETCHING.iter().any(|f| lower.contains(f))
}

/// An element's opening tag with what it carries as any element does
/// (`own_attrs`) beside its class.
fn with_own(open: &str, tag: &str, name: &str, ui: &UIElement, ctx: &RenderContext<'_>) -> String {
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
    ctx: &RenderContext<'_>,
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
fn own_attrs(name: &str, ui: &UIElement, ctx: &RenderContext<'_>, reads: &[&str]) -> String {
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
fn extra_classes(ui: &UIElement, ctx: &RenderContext<'_>) -> Vec<String> {
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
fn chart_of(ui: &UIElement, ctx: &mut RenderContext<'_>) -> crate::web::codegen::charts::Chart {
    let named: Vec<(String, Value)> = ui
        .args
        .iter()
        .filter_map(|a| match a {
            Arg::Named(k, e) => Some((k.clone(), ctx.eval_expr(e))),
            Arg::Positional(_) => None,
        })
        .collect();
    crate::web::codegen::charts::chart_from(&named, &ui.modifiers)
}

/// The paged and slide elements: their markup, with what the engine reads
/// from it as attributes. `None` for any other element.
fn render_paged(
    name: &str,
    ui: &UIElement,
    ctx: &mut RenderContext<'_>,
    class_str: &str,
    style_attr: &str,
) -> Option<String> {
    let named = |ctx: &mut RenderContext<'_>, key: &str| -> Option<String> {
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
    let positional = |ctx: &mut RenderContext<'_>| -> Option<String> {
        ui.args.iter().find_map(|a| match a {
            Arg::Positional(e) => Some(value_to_string(&ctx.eval_expr(e))),
            Arg::Named(..) => None,
        })
    };
    // `.case` values arrive as their name.
    let case = |s: String| s.trim_start_matches('.').to_string();
    let indent = ctx.indent_str();
    let open = |ctx: &mut RenderContext<'_>, tag: &str, attrs: Vec<(String, String)>| -> String {
        let mut a = String::new();
        if !class_str.is_empty() {
            a.push_str(&format!(" class=\"{class_str}\""));
        }
        for (k, v) in attrs {
            a.push_str(&format!(" {k}=\"{}\"", html_escape(&v)));
        }
        let _ = ctx;
        format!("{indent}<{tag}{a}{style_attr}>\n")
    };
    let children = |ctx: &mut RenderContext<'_>| -> String {
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
            let mut out = with_own(&open(ctx, "figure", Vec::new()), "figure", name, ui, ctx);
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
            let mut out = with_own(&open(ctx, "figure", Vec::new()), "figure", name, ui, ctx);
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
            let mut out = with_own(&open(ctx, "div", Vec::new()), "div", name, ui, ctx);
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

fn render_tag(tag: &str, class: &str, ui: &UIElement, ctx: &mut RenderContext<'_>) -> String {
    let indent = ctx.indent_str();
    let class = std::iter::once(class.to_string())
        .chain(extra_classes(ui, ctx))
        .collect::<Vec<_>>()
        .join(" ");
    let mut result = format!("{indent}<{tag} class=\"{class}\">\n");
    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;
    result.push_str(&format!("{indent}</{tag}>\n"));
    result
}

/// The blocks a call of `component` fills its slots with: its own block as
/// `children`, and each named fill.
fn slot_fills(
    component: &TemplateComponent,
    ui: &UIElement,
    scope: &Locals,
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
