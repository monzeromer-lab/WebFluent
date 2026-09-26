use crate::codegen::builtin::{
    builtin_to_html, class_list, element_tag, implicit_role, input_type, landmark_label,
    layout_arg_classes,
};
use crate::codegen::css::generate_css;
use crate::codegen::pdf::PdfCodegen;
use crate::codegen::slides::SlidesCodegen;
use crate::config::project::{PdfConfig, SlidesConfig};
use crate::error::{Result, WebFluentError};
use crate::parser::ast::{ArmPattern, ForStmt, IfStmt, PropDecl};
use crate::parser::{
    Arg, ComponentRef, Declaration, Expr, Program, Statement, StatementKind, StringPart, UIElement,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

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
pub struct Template {
    source: String,
    theme: Option<String>,
    custom_tokens: HashMap<String, String>,
    /// Where a `data` declaration's file is looked for: the template file's
    /// own directory. A template from a string has none.
    root: Option<std::path::PathBuf>,
}

// `Template::from_str` is documented public API used throughout the README and
// the docs site; it predates any `FromStr` impl and renaming it would break every
// consumer to satisfy a naming lint.
#[allow(clippy::should_implement_trait)]
impl Template {
    /// Create a template from a `.wf` source string.
    ///
    /// The source is parsed immediately to validate syntax. Returns an error
    /// if the source contains lexer or parser errors.
    ///
    /// # Errors
    ///
    /// Returns [`WebFluentError::LexerError`] or [`WebFluentError::ParseError`]
    /// if the source is invalid.
    pub fn from_str(source: &str) -> Result<Self> {
        let program = crate::syntax::parse_source(source, "<template>")?;
        // Held to what a build is held to: a component nothing declares, a
        // flag or case a component does not take, a value of the wrong type.
        // A name the template reads is its data's, known only at render
        // time, so an undeclared name is not one of them.
        let file_of = |_: usize| "<template>".to_string();
        let mut errors = crate::linter::validate_semantics_in(&program, &file_of);
        errors.extend(crate::sema::check(&program, &file_of).errors);
        errors.extend(
            crate::sema::types::check_in(&program, &file_of, &|_| None)
                .findings
                .errors,
        );
        if !errors.is_empty() {
            return Err(WebFluentError::CodegenError(
                errors
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ));
        }

        Ok(Self {
            source: source.to_string(),
            theme: None,
            custom_tokens: HashMap::new(),
            root: None,
        })
    }

    /// Create a template from a `.wf` (or `.wfx`) file on disk.
    ///
    /// # Errors
    ///
    /// Returns [`WebFluentError::IoError`] if the file cannot be read,
    /// or a parse error if the content is invalid.
    pub fn from_file(path: &str) -> Result<Self> {
        let source = fs::read_to_string(path).map_err(|e| {
            WebFluentError::IoError(format!("Failed to read template '{}': {}", path, e))
        })?;
        // An indented file is read as its braced spelling, which is what
        // the renderers parse.
        let source = if path.ends_with(".wfx") {
            crate::layout::to_braces(&source, path)?
        } else {
            source
        };
        let mut template = Self::from_str(&source)?;
        template.root = std::path::Path::new(path).parent().map(|p| p.to_path_buf());
        Ok(template)
    }

    /// Select which `Theme` declared in the template to render with.
    ///
    /// Only needed when the source declares more than one. A template with a
    /// single `Theme` uses it automatically, and one with none renders on the
    /// baseline tokens.
    ///
    /// This used to name one of four palettes the engine carried; those are now
    /// example `.wf` files you copy into your own source.
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = Some(theme.to_string());
        self
    }

    /// The stylesheet a rendered document links: the engine's sheet over the
    /// template's tokens, plus the rules its pseudo-state and media blocks
    /// compile to.
    fn stylesheet(&self) -> Result<String> {
        let program = self.parse()?;
        let mut css = generate_css(&self.tokens()?);
        css.push_str(&crate::codegen::scoped_css::scoped_rules(&program));
        Ok(css)
    }

    /// The design tokens this template renders with.
    fn tokens(&self) -> Result<HashMap<String, String>> {
        let program = self.parse()?;
        let config = crate::config::project::ThemeConfig {
            name: self.theme.clone(),
            tokens: self.custom_tokens.clone(),
            builtin: Default::default(),
            dark: None,
        };
        crate::themes::resolve_tokens(&program, &config)
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

    /// Render to a full HTML document with embedded CSS.
    ///
    /// Returns a complete `<!DOCTYPE html>` document with `<html>`, `<head>` (including
    /// a `<style>` block with component CSS and theme tokens), and `<body>`.
    ///
    /// Top-level keys in `data` become template variables accessible via `{key}`
    /// interpolation and in `for`/`if` blocks.
    pub fn render_html(&self, data: &Value) -> Result<String> {
        let fragment = self.render_html_fragment(data)?;
        let css = self.stylesheet()?;

        Ok(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
{}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            css, fragment
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
    pub fn render_html_parts(&self, data: &Value) -> Result<(String, String)> {
        Ok((self.stylesheet()?, self.render_html_fragment(data)?))
    }

    /// Render to an HTML fragment (no `<html>`/`<head>`/`<body>` wrapper).
    ///
    /// Useful for embedding rendered content into an existing page or email template.
    /// Does not include CSS — use [`render_html`](Template::render_html) for a complete
    /// document, or [`render_html_parts`](Template::render_html_parts) to serve the CSS
    /// separately.
    pub fn render_html_fragment(&self, data: &Value) -> Result<String> {
        let program = self.parse()?;
        render_program_fragment(&program, data)
    }
}

/// The HTML fragment of every page of a lowered program, over `data`:
/// what a template renders, and what `wf test` holds a test to.
pub fn render_program_fragment(program: &Program, data: &Value) -> Result<String> {
    {
        let mut ctx = RenderContext::new(data);

        let mut html = String::new();
        // Every component first, wherever it is declared, so a page may
        // use one declared after it.
        for decl in &program.declarations {
            if let Declaration::Component(comp) = decl {
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
        }
        for decl in &program.declarations {
            if let Declaration::Page(page) = decl {
                html.push_str(&render_statements(&page.body, &mut ctx));
            }
        }
        Ok(html)
    }
}

#[allow(clippy::should_implement_trait)]
impl Template {
    /// Render to PDF as raw bytes.
    ///
    /// Returns a valid PDF file as `Vec<u8>`. Write the result to a file
    /// or send it as an HTTP response with `Content-Type: application/pdf`.
    ///
    /// Uses A4 page size with 72pt margins by default. The template should use
    /// PDF-compatible components only (no `Button`, `Input`, `Router`, etc.).
    pub fn render_pdf(&self, data: &Value) -> Result<Vec<u8>> {
        let program = self.parse()?;

        // Resolve data into the program by substituting expressions
        let resolved = self.resolve_program(&program, data)?;

        let config = PdfConfig::default();
        let mut pdf = PdfCodegen::new(&config);
        Ok(pdf.generate(&resolved))
    }

    /// Render to a PDF slide deck as raw bytes.
    ///
    /// One `Slide` (or layout variant) per page, no flow pagination. The template
    /// must wrap its slides in a `Presentation { ... }` block; content that overflows
    /// a slide is clipped and a stderr warning is emitted.
    ///
    /// Uses 16:9 page size (960×540pt) by default. The template should use slide-compatible
    /// components only (no `Button`, `Input`, `Router`, etc.).
    pub fn render_slides(&self, data: &Value) -> Result<Vec<u8>> {
        let program = self.parse()?;
        let resolved = self.resolve_program(&program, data)?;

        let config = SlidesConfig::default();
        let mut slides = SlidesCodegen::new(&config);
        Ok(slides.generate(&resolved))
    }

    fn parse(&self) -> Result<Program> {
        let mut program = crate::syntax::parse_source(&self.source, "<template>")?;
        if let Some(root) = &self.root {
            crate::data::resolve_data(&mut program, root)?;
        } else if let Some(Declaration::Data(d)) = program
            .declarations
            .iter()
            .find(|d| matches!(d, Declaration::Data(_)))
        {
            return Err(WebFluentError::IoError(format!(
                "`data {}` reads a file, which a template from a string has no place to read from; use `Template::from_file`",
                d.name
            )));
        }
        Ok(crate::sema::lower(program))
    }

    /// Resolve all data references in the program for PDF rendering.
    fn resolve_program(&self, program: &Program, data: &Value) -> Result<Program> {
        let ctx = RenderContext::new(data);
        let mut new_decls = Vec::new();

        for decl in &program.declarations {
            match decl {
                Declaration::Page(page) => {
                    let mut new_page = page.clone();
                    new_page.body = resolve_statements(&page.body, &ctx);
                    new_decls.push(Declaration::Page(new_page));
                }
                other => new_decls.push(other.clone()),
            }
        }

        Ok(Program {
            declarations: new_decls,
        })
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
}

/// A component as the template engine expands it: its body, its props,
/// and per slot the names of what it hands over.
#[derive(Debug, Clone)]
struct TemplateComponent {
    body: Vec<Statement>,
    handed: HashMap<String, Vec<String>>,
    props: Vec<PropDecl>,
}

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
                let href = value_to_string(&ctx.eval_expr(&dest));
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
            if let Some(TemplateComponent {
                body,
                handed,
                props,
            }) = ctx.components.get(name).cloned()
            {
                // The props as locals: a positional argument binds the
                // positional prop (or the first), a named one its name, and
                // a prop left out takes its default.
                let mut old_locals = Vec::new();
                let mut given: Vec<(String, Value)> = Vec::new();
                for arg in &ui.args {
                    match arg {
                        Arg::Named(key, val) => given.push((key.clone(), ctx.eval_expr(val))),
                        Arg::Positional(val) => {
                            let target = props
                                .iter()
                                .find(|p| p.positional)
                                .or_else(|| props.first())
                                .map(|p| p.name.clone());
                            if let Some(key) = target {
                                given.push((key, ctx.eval_expr(val)));
                            }
                        }
                    }
                }
                for prop in &props {
                    if !given.iter().any(|(k, _)| *k == prop.name)
                        && let Some(default) = &prop.default
                    {
                        given.push((prop.name.clone(), ctx.eval_expr(default)));
                    }
                }
                for (key, resolved) in given {
                    let old = ctx.locals.insert(key.clone(), resolved);
                    old_locals.push((key, old));
                }

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
                            handed: handed.get(&fill.name).cloned().unwrap_or_default(),
                            body: fill.body.clone(),
                        },
                    );
                }
                ctx.slots.push(slots);
                let html = render_statements(&body, ctx);
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
    let (_, base_class) = builtin_to_html(name);
    let mut classes = class_list(base_class, &ui.modifiers);
    classes.extend(layout_arg_classes(&ui.args));
    classes.extend(extra_classes(ui, ctx));
    let class_str = classes.join(" ");

    // Special handling. These build their tag inline, so they carry the author's
    // `style { }` block themselves — returning early used to drop it.
    let style_attr = style_block_attr(ui, ctx);
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
                    let resolved = ctx.eval_expr(val);
                    attrs.push(format!(
                        "{}=\"{}\"",
                        key,
                        html_escape(&value_to_string(&resolved))
                    ));
                }
                "to" => {
                    let resolved = ctx.eval_expr(val);
                    attrs.push(format!(
                        "href=\"{}\"",
                        html_escape(&value_to_string(&resolved))
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
        let text = value_to_string(&ctx.eval_expr(value));
        classes.extend(text.split_whitespace().map(str::to_string));
    }
    classes
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

// ─── Resolve statements for PDF (substitutes data into AST) ─────────

fn resolve_statements(stmts: &[Statement], ctx: &RenderContext) -> Vec<Statement> {
    let mut result = Vec::new();

    for stmt in stmts {
        match &stmt.kind {
            StatementKind::If(if_stmt) => {
                let cond = ctx.eval_expr(&if_stmt.condition);
                if is_truthy(&cond) {
                    // `if let p = post { … }`: the branch reads `p` as the value.
                    match &if_stmt.binding {
                        Some(name) => {
                            let mut child_ctx = RenderContext {
                                data: ctx.data,
                                locals: ctx.locals.clone(),
                                components: ctx.components.clone(),
                                indent: ctx.indent,
                                in_thead: ctx.in_thead,
                                slots: ctx.slots.clone(),
                                fields: ctx.fields,
                            };
                            child_ctx.locals.insert(name.clone(), cond.clone());
                            result.extend(resolve_statements(&if_stmt.then_body, &child_ctx));
                        }
                        None => result.extend(resolve_statements(&if_stmt.then_body, ctx)),
                    }
                } else {
                    let mut matched = false;
                    for (branch_cond, branch_body) in &if_stmt.else_if_branches {
                        if is_truthy(&ctx.eval_expr(branch_cond)) {
                            result.extend(resolve_statements(branch_body, ctx));
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        if let Some(else_body) = &if_stmt.else_body {
                            result.extend(resolve_statements(else_body, ctx));
                        }
                    }
                }
            }
            StatementKind::For(for_stmt) => {
                let collection = ctx.eval_expr(&for_stmt.iterable);
                if let Value::Array(items) = &collection {
                    for (i, item) in items.iter().enumerate() {
                        let mut child_ctx = RenderContext {
                            data: ctx.data,
                            locals: ctx.locals.clone(),
                            components: ctx.components.clone(),
                            indent: ctx.indent,
                            in_thead: ctx.in_thead,
                            slots: ctx.slots.clone(),
                            fields: ctx.fields + i * 1000,
                        };
                        child_ctx.locals.insert(for_stmt.item.clone(), item.clone());
                        if let Some(idx_var) = &for_stmt.index {
                            child_ctx.locals.insert(
                                idx_var.clone(),
                                Value::Number(serde_json::Number::from(i)),
                            );
                        }
                        result.extend(resolve_statements(&for_stmt.body, &child_ctx));
                    }
                }
            }
            StatementKind::UIElement(ui) => {
                // Carry the original statement's source span onto the resolved node.
                result.push(Statement {
                    kind: StatementKind::UIElement(resolve_ui_element(ui, ctx)),
                    span: stmt.span,
                });
            }
            _ => result.push(stmt.clone()),
        }
    }
    result
}

fn resolve_ui_element(ui: &UIElement, ctx: &RenderContext) -> UIElement {
    let mut new_ui = ui.clone();

    // Resolve args
    new_ui.args = ui
        .args
        .iter()
        .map(|arg| match arg {
            Arg::Positional(expr) => Arg::Positional(resolve_expr(expr, ctx)),
            Arg::Named(key, expr) => Arg::Named(key.clone(), resolve_expr(expr, ctx)),
        })
        .collect();

    // Resolve children
    new_ui.children = resolve_statements(&ui.children, ctx);

    new_ui
}

fn resolve_expr(expr: &Expr, ctx: &RenderContext) -> Expr {
    match expr {
        Expr::Identifier(_)
        | Expr::PropertyAccess(_, _)
        | Expr::IndexAccess(_, _)
        | Expr::OptionalProperty(_, _)
        | Expr::OptionalIndex(_, _) => {
            let val = ctx.eval_expr(expr);
            value_to_expr(&val)
        }
        Expr::InterpolatedString(parts) => {
            let mut resolved = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(s) => resolved.push_str(s),
                    StringPart::Expression(e) => {
                        let val = ctx.eval_expr(e);
                        resolved.push_str(&value_to_string(&val));
                    }
                }
            }
            Expr::StringLiteral(resolved)
        }
        Expr::StringLiteral(_) => expr.clone(),
        Expr::BinaryOp(_, _, _) => {
            let val = ctx.eval_expr(expr);
            value_to_expr(&val)
        }
        Expr::FunctionCall(name, args) if name == "t" => {
            // Resolve t() to its key string
            if let Some(Expr::StringLiteral(key)) = args.first() {
                Expr::StringLiteral(key.clone())
            } else {
                expr.clone()
            }
        }
        _ => expr.clone(),
    }
}

fn value_to_expr(val: &Value) -> Expr {
    match val {
        Value::String(s) => Expr::StringLiteral(s.clone()),
        Value::Number(n) => Expr::NumberLiteral(n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => Expr::BoolLiteral(*b),
        Value::Null => Expr::StringLiteral(String::new()),
        _ => Expr::StringLiteral(value_to_string(val)),
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────

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
