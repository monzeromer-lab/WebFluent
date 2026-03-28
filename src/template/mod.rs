use std::collections::HashMap;
use std::fs;
use serde_json::Value;
use crate::lexer::Lexer;
use crate::parser::{Parser, Program, Declaration, Statement, UIElement, ComponentRef, Expr, StringPart, Arg};
use crate::parser::ast::{IfStmt, ForStmt};
use crate::codegen::css::generate_css;
use crate::codegen::pdf::PdfCodegen;
use crate::config::project::PdfConfig;
use crate::error::{WebFluentError, Result};

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
/// let tpl = Template::from_str(r#"
///     Page Home (path: "/", title: "Hello") {
///         Container { Heading("Hello, {name}!", h1) }
///     }
/// "#).unwrap();
///
/// let html = tpl.render_html(&json!({"name": "World"})).unwrap();
/// assert!(html.contains("Hello, World!"));
/// ```
///
/// # Theming
///
/// Use [`with_theme`](Template::with_theme) and [`with_tokens`](Template::with_tokens)
/// to customize the design system:
///
/// ```rust,no_run
/// # use webfluent::Template;
/// # use serde_json::json;
/// let html = Template::from_str("Page P (path: \"/\") { Text(\"Hi\") }")
///     .unwrap()
///     .with_theme("dark")
///     .with_tokens(&[("color-primary", "#8B5CF6")])
///     .render_html(&json!({}))
///     .unwrap();
/// ```
pub struct Template {
    source: String,
    theme: String,
    custom_tokens: HashMap<String, String>,
}

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
        // Validate that it parses
        let mut lexer = Lexer::new(source, "<template>");
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens, "<template>");
        let _program = parser.parse()?;

        Ok(Self {
            source: source.to_string(),
            theme: "default".to_string(),
            custom_tokens: HashMap::new(),
        })
    }

    /// Create a template from a `.wf` file on disk.
    ///
    /// # Errors
    ///
    /// Returns [`WebFluentError::IoError`] if the file cannot be read,
    /// or a parse error if the content is invalid.
    pub fn from_file(path: &str) -> Result<Self> {
        let source = fs::read_to_string(path).map_err(|e| {
            WebFluentError::IoError(format!("Failed to read template '{}': {}", path, e))
        })?;
        Self::from_str(&source)
    }

    /// Set the theme for rendering (builder pattern).
    ///
    /// Built-in themes: `"default"`, `"dark"`, `"minimal"`, `"brutalist"`.
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = theme.to_string();
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

    /// Render to a full HTML document with embedded CSS.
    ///
    /// Returns a complete `<!DOCTYPE html>` document with `<html>`, `<head>` (including
    /// a `<style>` block with component CSS and theme tokens), and `<body>`.
    ///
    /// Top-level keys in `data` become template variables accessible via `{key}`
    /// interpolation and in `for`/`if` blocks.
    pub fn render_html(&self, data: &Value) -> Result<String> {
        let fragment = self.render_html_fragment(data)?;
        let css = generate_css(&self.theme, &self.custom_tokens);

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

    /// Render to an HTML fragment (no `<html>`/`<head>`/`<body>` wrapper).
    ///
    /// Useful for embedding rendered content into an existing page or email template.
    /// Does not include CSS — use [`render_html`](Template::render_html) for a complete document.
    pub fn render_html_fragment(&self, data: &Value) -> Result<String> {
        let program = self.parse()?;
        let mut ctx = RenderContext::new(data);

        let mut html = String::new();
        for decl in &program.declarations {
            match decl {
                Declaration::Page(page) => {
                    html.push_str(&render_statements(&page.body, &mut ctx));
                }
                Declaration::Component(comp) => {
                    // Register component for later use
                    ctx.components.insert(comp.name.clone(), comp.body.clone());
                }
                _ => {} // Skip App, Store in template mode
            }
        }
        Ok(html)
    }

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

    fn parse(&self) -> Result<Program> {
        let mut lexer = Lexer::new(&self.source, "<template>");
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens, "<template>");
        parser.parse()
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

        Ok(Program { declarations: new_decls })
    }
}

// ─── Render Context ──────────────────────────────────────────────────

struct RenderContext<'a> {
    data: &'a Value,
    locals: HashMap<String, Value>,
    components: HashMap<String, Vec<Statement>>,
    indent: usize,
}

impl<'a> RenderContext<'a> {
    fn new(data: &'a Value) -> Self {
        Self {
            data,
            locals: HashMap::new(),
            components: HashMap::new(),
            indent: 1,
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
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
            Expr::StringLiteral(s) => Value::String(self.interpolate_string(s)),
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
            Expr::NumberLiteral(n) => Value::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))),
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
            Expr::IndexAccess(arr_expr, idx_expr) => {
                let arr = self.eval_expr(arr_expr);
                let idx = self.eval_expr(idx_expr);
                match (&arr, &idx) {
                    (Value::Array(a), Value::Number(n)) => {
                        if let Some(i) = n.as_u64() {
                            a.get(i as usize).cloned().unwrap_or(Value::Null)
                        } else {
                            Value::Null
                        }
                    }
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
                            Value::Number(serde_json::Number::from_f64(-n).unwrap_or(serde_json::Number::from(0)))
                        } else {
                            Value::Null
                        }
                    }
                }
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
                        let needle = if let Some(a) = args.first() { self.eval_expr(a) } else { Value::Null };
                        match (&parent, &needle) {
                            (Value::String(s), Value::String(n)) => Value::Bool(s.contains(n.as_str())),
                            (Value::Array(arr), _) => Value::Bool(arr.contains(&needle)),
                            _ => Value::Bool(false),
                        }
                    }
                    "join" => {
                        let sep = if let Some(a) = args.first() {
                            value_to_string(&self.eval_expr(a))
                        } else { ", ".to_string() };
                        match &parent {
                            Value::Array(arr) => {
                                let parts: Vec<String> = arr.iter().map(value_to_string).collect();
                                Value::String(parts.join(&sep))
                            }
                            _ => Value::Null,
                        }
                    }
                    _ => Value::Null,
                }
            }
            Expr::FunctionCall(name, _args) => {
                // t() in template mode — not supported, return key
                if name == "t" {
                    if let Some(Expr::StringLiteral(key)) = _args.first() {
                        Value::String(key.clone())
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            }
            Expr::ListLiteral(items) => {
                Value::Array(items.iter().map(|e| self.eval_expr(e)).collect())
            }
            Expr::MapLiteral(pairs) => {
                let map: serde_json::Map<String, Value> = pairs.iter()
                    .map(|(k, v)| (k.clone(), self.eval_expr(v)))
                    .collect();
                Value::Object(map)
            }
            _ => Value::Null,
        }
    }

    /// Interpolate `{var}` references in a plain string.
    fn interpolate_string(&self, s: &str) -> String {
        // Handle Unicode placeholders from lexer: \u{FFFE} = { , \u{FFFF} = }
        let s = s.replace('\u{FFFE}', "{").replace('\u{FFFF}', "}");

        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut var_name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next();
                        break;
                    }
                    var_name.push(c);
                    chars.next();
                }
                // Resolve dotted paths like "user.name"
                let val = self.resolve_path(&var_name);
                result.push_str(&value_to_string(&val));
            } else {
                result.push(ch);
            }
        }
        result
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
        match stmt {
            Statement::UIElement(ui) => html.push_str(&render_ui_element(ui, ctx)),
            Statement::If(if_stmt) => html.push_str(&render_if(if_stmt, ctx)),
            Statement::For(for_stmt) => html.push_str(&render_for(for_stmt, ctx)),
            // Skip state, derived, effect, action, use, events, navigate, etc.
            _ => {}
        }
    }
    html
}

fn render_if(if_stmt: &IfStmt, ctx: &mut RenderContext) -> String {
    let cond = ctx.eval_expr(&if_stmt.condition);
    if is_truthy(&cond) {
        return render_statements(&if_stmt.then_body, ctx);
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
                ctx.locals.insert(idx_var.clone(), Value::Number(serde_json::Number::from(i)))
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
            let class = format!("wf-{}__{}",
                parent.to_lowercase(),
                camel_to_kebab(sub)
            );
            let tag = match sub.as_str() {
                "Item" => "li",
                _ => "div",
            };
            render_tag(tag, &class, ui, ctx)
        }
        ComponentRef::UserDefined(name) => {
            // Expand user component if registered
            if let Some(body) = ctx.components.get(name).cloned() {
                // Push props as locals
                let mut old_locals = Vec::new();
                for arg in &ui.args {
                    if let Arg::Named(key, val) = arg {
                        let resolved = ctx.eval_expr(val);
                        let old = ctx.locals.insert(key.clone(), resolved);
                        old_locals.push((key.clone(), old));
                    }
                }
                // Also handle positional args mapped to prop names
                // (simplified: just render the body)

                let html = render_statements(&body, ctx);

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
    let (tag, base_class) = builtin_to_html_tag(name);

    // Build class string
    let mut classes = vec![base_class.to_string()];
    for m in &ui.modifiers {
        let mc = modifier_to_css_class(base_class, m);
        if !mc.is_empty() {
            classes.push(mc);
        }
    }
    let class_str = classes.iter().filter(|c| !c.is_empty()).cloned().collect::<Vec<_>>().join(" ");

    // Special handling
    match name {
        "Spacer" => return format!("{}<div class=\"{}\"></div>\n", ctx.indent_str(), class_str),
        "Divider" => return format!("{}<hr class=\"{}\">\n", ctx.indent_str(), class_str),
        "Spinner" => return format!("{}<div class=\"{}\"></div>\n", ctx.indent_str(), class_str),
        "Toast" | "Router" | "Route" => return String::new(),
        _ => {}
    }

    let mut attrs = Vec::new();
    let mut text_content: Option<String> = None;
    let mut inline_style: Option<String> = None;

    if !class_str.is_empty() {
        attrs.push(format!("class=\"{}\"", class_str));
    }

    for arg in &ui.args {
        match arg {
            Arg::Named(key, val) => {
                match key.as_str() {
                    "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max" |
                    "step" | "accept" | "role" | "value" | "title" | "width" | "height" => {
                        let resolved = ctx.eval_expr(val);
                        attrs.push(format!("{}=\"{}\"", key, html_escape(&value_to_string(&resolved))));
                    }
                    "to" => {
                        let resolved = ctx.eval_expr(val);
                        attrs.push(format!("href=\"{}\"", html_escape(&value_to_string(&resolved))));
                    }
                    "label" => {
                        let resolved = ctx.eval_expr(val);
                        text_content = Some(value_to_string(&resolved));
                    }
                    "columns" => {
                        if let Expr::NumberLiteral(n) = val {
                            inline_style = Some(format!("grid-template-columns: repeat({}, 1fr)", *n as i32));
                        }
                    }
                    "required" | "disabled" | "controls" => {
                        attrs.push(key.to_string());
                    }
                    _ => {}
                }
            }
            Arg::Positional(expr) => {
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

    // Handle inline style blocks
    if let Some(style_block) = &ui.style_block {
        let mut style_parts = Vec::new();
        for prop in &style_block.properties {
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

    // Input type from modifiers
    for m in &ui.modifiers {
        match m.as_str() {
            "text" | "email" | "password" | "number" | "search" | "tel" | "url" |
            "date" | "time" | "color" => {
                let t = if m == "datetime" { "datetime-local" } else { m.as_str() };
                attrs.push(format!("type=\"{}\"", t));
            }
            "block" if name == "Code" => {
                // Already handled via class
            }
            _ => {}
        }
    }

    let actual_tag = if name == "Heading" { heading_tag(&ui.modifiers) } else { tag };
    let indent = ctx.indent_str();
    let attrs_str = if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) };

    // Self-closing tags
    if matches!(actual_tag, "input" | "img" | "hr" | "br") {
        return format!("{}<{}{}>\n", indent, actual_tag, attrs_str);
    }

    let has_children = !ui.children.is_empty();
    let has_text = text_content.is_some();

    if !has_children && !has_text {
        return format!("{}<{}{}></{}>​\n", indent, actual_tag, attrs_str, actual_tag);
    }

    if let Some(text) = &text_content {
        if !has_children {
            return format!("{}<{}{}>{}</{}>​\n", indent, actual_tag, attrs_str, html_escape(text), actual_tag);
        }
    }

    let mut result = format!("{}<{}{}>\n", indent, actual_tag, attrs_str);

    if let Some(text) = &text_content {
        result.push_str(&format!("{}    {}\n", indent, html_escape(text)));
    }

    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;
    result.push_str(&format!("{}</{}>​\n", indent, actual_tag));
    result
}

fn render_tag(tag: &str, class: &str, ui: &UIElement, ctx: &mut RenderContext) -> String {
    let indent = ctx.indent_str();
    let mut result = format!("{}<{} class=\"{}\">\n", indent, tag, class);
    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;
    result.push_str(&format!("{}</{}>​\n", indent, tag));
    result
}

// ─── Resolve statements for PDF (substitutes data into AST) ─────────

fn resolve_statements(stmts: &[Statement], ctx: &RenderContext) -> Vec<Statement> {
    let mut result = Vec::new();

    for stmt in stmts {
        match stmt {
            Statement::If(if_stmt) => {
                let cond = ctx.eval_expr(&if_stmt.condition);
                if is_truthy(&cond) {
                    result.extend(resolve_statements(&if_stmt.then_body, ctx));
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
            Statement::For(for_stmt) => {
                let collection = ctx.eval_expr(&for_stmt.iterable);
                if let Value::Array(items) = &collection {
                    for (i, item) in items.iter().enumerate() {
                        let mut child_ctx = RenderContext {
                            data: ctx.data,
                            locals: ctx.locals.clone(),
                            components: ctx.components.clone(),
                            indent: ctx.indent,
                        };
                        child_ctx.locals.insert(for_stmt.item.clone(), item.clone());
                        if let Some(idx_var) = &for_stmt.index {
                            child_ctx.locals.insert(idx_var.clone(), Value::Number(serde_json::Number::from(i)));
                        }
                        result.extend(resolve_statements(&for_stmt.body, &child_ctx));
                    }
                }
            }
            Statement::UIElement(ui) => {
                result.push(Statement::UIElement(resolve_ui_element(ui, ctx)));
            }
            other => result.push(other.clone()),
        }
    }
    result
}

fn resolve_ui_element(ui: &UIElement, ctx: &RenderContext) -> UIElement {
    let mut new_ui = ui.clone();

    // Resolve args
    new_ui.args = ui.args.iter().map(|arg| {
        match arg {
            Arg::Positional(expr) => Arg::Positional(resolve_expr(expr, ctx)),
            Arg::Named(key, expr) => Arg::Named(key.clone(), resolve_expr(expr, ctx)),
        }
    }).collect();

    // Resolve children
    new_ui.children = resolve_statements(&ui.children, ctx);

    new_ui
}

fn resolve_expr(expr: &Expr, ctx: &RenderContext) -> Expr {
    match expr {
        Expr::Identifier(_) | Expr::PropertyAccess(_, _) | Expr::IndexAccess(_, _) => {
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
        Expr::StringLiteral(s) => Expr::StringLiteral(ctx.interpolate_string(s)),
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
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                format!("{}", i)
            } else {
                format!("{}", n)
            }
        }
        Value::Bool(b) => format!("{}", b),
        Value::Null => String::new(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(val).unwrap_or_default(),
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

    match op {
        BinOp::Eq => Value::Bool(left == right),
        BinOp::Neq => Value::Bool(left != right),
        BinOp::Lt => Value::Bool(as_f64(left) < as_f64(right)),
        BinOp::Gt => Value::Bool(as_f64(left) > as_f64(right)),
        BinOp::Lte => Value::Bool(as_f64(left) <= as_f64(right)),
        BinOp::Gte => Value::Bool(as_f64(left) >= as_f64(right)),
        BinOp::And => Value::Bool(is_truthy(left) && is_truthy(right)),
        BinOp::Or => Value::Bool(is_truthy(left) || is_truthy(right)),
        BinOp::Add => {
            // String concatenation or numeric addition
            match (left, right) {
                (Value::String(l), _) => Value::String(format!("{}{}", l, value_to_string(right))),
                (_, Value::String(r)) => Value::String(format!("{}{}", value_to_string(left), r)),
                _ => Value::Number(serde_json::Number::from_f64(as_f64(left) + as_f64(right)).unwrap_or(serde_json::Number::from(0))),
            }
        }
        BinOp::Sub => Value::Number(serde_json::Number::from_f64(as_f64(left) - as_f64(right)).unwrap_or(serde_json::Number::from(0))),
        BinOp::Mul => Value::Number(serde_json::Number::from_f64(as_f64(left) * as_f64(right)).unwrap_or(serde_json::Number::from(0))),
        BinOp::Div => {
            let r = as_f64(right);
            if r == 0.0 { Value::Null } else {
                Value::Number(serde_json::Number::from_f64(as_f64(left) / r).unwrap_or(serde_json::Number::from(0)))
            }
        }
        BinOp::Mod => {
            let r = as_f64(right);
            if r == 0.0 { Value::Null } else {
                Value::Number(serde_json::Number::from_f64(as_f64(left) % r).unwrap_or(serde_json::Number::from(0)))
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
     .replace('\u{FFFE}', "{")
     .replace('\u{FFFF}', "}")
}

fn heading_tag(modifiers: &[String]) -> &'static str {
    for m in modifiers {
        match m.as_str() {
            "h1" => return "h1",
            "h2" => return "h2",
            "h3" => return "h3",
            "h4" => return "h4",
            "h5" => return "h5",
            "h6" => return "h6",
            _ => {}
        }
    }
    "h2"
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

fn builtin_to_html_tag(name: &str) -> (&'static str, &'static str) {
    match name {
        "Container" => ("div", "wf-container"),
        "Row" => ("div", "wf-row"),
        "Column" => ("div", "wf-col"),
        "Grid" => ("div", "wf-grid"),
        "Stack" => ("div", "wf-stack"),
        "Spacer" => ("div", "wf-spacer"),
        "Divider" => ("hr", "wf-divider"),
        "Navbar" => ("nav", "wf-navbar"),
        "Sidebar" => ("aside", "wf-sidebar"),
        "Breadcrumb" => ("nav", "wf-breadcrumb"),
        "Link" => ("a", "wf-link"),
        "Menu" => ("div", "wf-menu"),
        "Tabs" => ("div", "wf-tabs"),
        "TabPage" => ("div", "wf-tab-page"),
        "Card" => ("div", "wf-card"),
        "Table" => ("table", "wf-table"),
        "Thead" => ("thead", ""),
        "Tbody" => ("tbody", ""),
        "Trow" => ("tr", ""),
        "Tcell" => ("td", ""),
        "List" => ("ul", "wf-list"),
        "Badge" => ("span", "wf-badge"),
        "Avatar" => ("div", "wf-avatar"),
        "Tooltip" => ("div", "wf-tooltip"),
        "Tag" => ("span", "wf-tag"),
        "Input" => ("input", "wf-input"),
        "Select" => ("select", "wf-select"),
        "Option" => ("option", ""),
        "Checkbox" => ("label", "wf-checkbox"),
        "Radio" => ("label", "wf-radio"),
        "Switch" => ("label", "wf-switch"),
        "Slider" => ("input", "wf-slider"),
        "Form" => ("form", "wf-form"),
        "Alert" => ("div", "wf-alert"),
        "Modal" => ("div", "wf-modal"),
        "Dialog" => ("div", "wf-dialog"),
        "Spinner" => ("div", "wf-spinner"),
        "Progress" => ("progress", "wf-progress"),
        "Skeleton" => ("div", "wf-skeleton"),
        "Button" => ("button", "wf-btn"),
        "IconButton" => ("button", "wf-icon-btn"),
        "ButtonGroup" => ("div", "wf-btn-group"),
        "Dropdown" => ("div", "wf-dropdown"),
        "Image" => ("img", "wf-image"),
        "Video" => ("video", "wf-video"),
        "Icon" => ("i", "wf-icon"),
        "Carousel" => ("div", "wf-carousel"),
        "Text" => ("p", "wf-text"),
        "Heading" => ("h2", "wf-heading"),
        "Code" => ("code", "wf-code"),
        "Blockquote" => ("blockquote", "wf-blockquote"),
        "Section" => ("section", "wf-section"),
        "Paragraph" => ("p", "wf-text"),
        "Document" => ("div", "wf-document"),
        "Header" => ("header", "wf-header"),
        "Footer" => ("footer", "wf-footer"),
        _ => ("div", ""),
    }
}

fn modifier_to_css_class(base_class: &str, modifier: &str) -> String {
    match modifier {
        "small" => format!("{}--small", base_class),
        "medium" => String::new(),
        "large" => format!("{}--large", base_class),
        "primary" => format!("{}--primary", base_class),
        "secondary" => format!("{}--secondary", base_class),
        "success" => format!("{}--success", base_class),
        "danger" => format!("{}--danger", base_class),
        "warning" => format!("{}--warning", base_class),
        "info" => format!("{}--info", base_class),
        "rounded" => format!("{}--rounded", base_class),
        "pill" => format!("{}--pill", base_class),
        "flat" => format!("{}--flat", base_class),
        "elevated" => format!("{}--elevated", base_class),
        "outlined" => format!("{}--outlined", base_class),
        "full" => format!("{}--full", base_class),
        "bold" => "wf-text--bold".to_string(),
        "italic" => "wf-text--italic".to_string(),
        "center" => "wf-text--center".to_string(),
        "right" => "wf-text--right".to_string(),
        "heading" => "wf-text--heading".to_string(),
        "muted" => "wf-text--muted".to_string(),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => format!("wf-heading--{}", modifier),
        "dismissible" => format!("{}--dismissible", base_class),
        "fluid" => format!("{}--fluid", base_class),
        "block" => format!("{}--block", base_class),
        "ordered" => format!("{}--ordered", base_class),
        // Skip animation modifiers in template mode (no JS)
        "fadeIn" | "fadeOut" | "slideUp" | "slideDown" | "slideLeft" | "slideRight" |
        "scaleIn" | "scaleOut" | "bounce" | "shake" | "pulse" | "spin" |
        "fast" | "slow" => String::new(),
        // Skip input types (handled separately)
        "text" | "email" | "password" | "number" | "search" | "tel" | "url" |
        "date" | "time" | "color" | "submit" | "reset" => String::new(),
        _ => String::new(),
    }
}
