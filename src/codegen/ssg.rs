use crate::codegen::builtin::{
    builtin_to_html, class_list, heading_tag, implicit_role, input_type, landmark_label,
};
use crate::codegen::node_id::NodeMap;
use crate::codegen::static_eval::{Scope, Static, eval};
use crate::config::ProjectConfig;
use crate::parser::ast::*;
use std::collections::HashMap;

/// Renders a page to static HTML for SSG (export mode — no studio attributes).
///
/// `components` are the program's `Component` declarations, so calls to them are
/// **expanded** into the static paint rather than left as placeholders. Pass an
/// empty map to render without them (the pre-expansion behaviour).
pub fn render_page_html(page: &PageDecl, site: &SiteContext) -> String {
    render_page_html_studio(page, site, false, &NodeMap::default())
}

/// Everything a page render needs that is a property of the *site* rather than
/// of the page: the config, the shared app shell, translations, the component
/// library and the program the build-time scope reads.
///
/// These used to be five separate parameters threaded through two public
/// functions, which had grown to eight arguments apiece.
pub struct SiteContext<'a> {
    pub config: &'a ProjectConfig,
    /// The `App` shell's body, if the project has one.
    pub app_body: Option<&'a [Statement]>,
    pub translations: &'a HashMap<String, HashMap<String, String>>,
    /// The program's components, so calls to them expand into the static paint.
    pub components: &'a HashMap<String, ComponentDecl>,
    /// The whole program, for the build-time scope that resolves seeded lists.
    pub program: &'a Program,
}

impl<'a> SiteContext<'a> {
    /// A context with no shell, no translations and no components — enough to
    /// render a single self-contained page.
    pub fn bare(config: &'a ProjectConfig, program: &'a Program) -> Self {
        use std::sync::OnceLock;
        static EMPTY_T: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();
        static EMPTY_C: OnceLock<HashMap<String, ComponentDecl>> = OnceLock::new();
        Self {
            config,
            app_body: None,
            translations: EMPTY_T.get_or_init(HashMap::new),
            components: EMPTY_C.get_or_init(HashMap::new),
            program,
        }
    }
}

/// Like [`render_page_html`], but stamps `data-wf-node="<id>"` on element roots
/// when `studio` is true, using ids from `node_map` (keyed by element span, so
/// they match the JS codegen exactly).
pub fn render_page_html_studio(
    page: &PageDecl,
    site: &SiteContext,
    studio: bool,
    node_map: &NodeMap,
) -> String {
    let SiteContext {
        config,
        app_body,
        translations,
        components,
        program,
    } = *site;
    let title = page.title.as_deref().unwrap_or(&config.name);
    let lang = if config.meta.lang.is_empty() {
        "en"
    } else {
        &config.meta.lang
    };

    let default_locale = config
        .i18n
        .as_ref()
        .map(|i| i.default_locale.as_str())
        .unwrap_or("en");

    let default_messages = translations
        .get(default_locale)
        .cloned()
        .unwrap_or_default();

    // Calculate relative base path from page route depth
    let route = page.path.trim_start_matches('/');
    let base_path = if route.is_empty() || route == "/" {
        ".".to_string()
    } else {
        let depth = route.split('/').filter(|s| !s.is_empty()).count();
        (0..depth).map(|_| "..").collect::<Vec<_>>().join("/")
    };

    let link_base = config.build.base_path.clone();

    let mut ctx = SsgContext {
        default_messages,
        indent: 2,
        base_path,
        link_base,
        studio,
        node_map: node_map.clone(),
        components: components.clone(),
        scope: Scope::from_program(program, &page.body),
        depth: 0,
    };

    // Render app shell (navbar, etc.) if available
    let mut body_html = String::new();
    if let Some(app_stmts) = app_body {
        render_app_shell_ssg(app_stmts, &page.body, &mut ctx, &mut body_html);
    } else {
        // With no shell there is no Router to stand in for `<main>`, so the page
        // body is the main content itself.
        body_html = format!(
            "{}<main id=\"wf-main\">\n{}{}</main>\n",
            ctx.indent_str(),
            {
                ctx.indent += 1;
                let inner = render_statements(&page.body, &mut ctx);
                ctx.indent -= 1;
                inner
            },
            ctx.indent_str()
        );
    }

    // Description, canonical, sharing card, language alternates and JSON-LD, all
    // derived from what the page and the config already say.
    let description_meta = crate::codegen::seo::head_tags(page, config, program);

    // Calculate relative path prefix based on page route depth
    let route = page.path.trim_start_matches('/');
    let base = if route.is_empty() || route == "/" {
        ".".to_string()
    } else {
        let depth = route.split('/').filter(|s| !s.is_empty()).count();
        (0..depth).map(|_| "..").collect::<Vec<_>>().join("/")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
{}{}    <link rel="stylesheet" href="{}/styles.css">
    <script src="{}/app.js" defer></script>
</head>
<body>
{}    <div id="app">
{}    </div>
</body>
</html>"#,
        lang,
        title,
        description_meta,
        crate::codegen::html::csp_meta(config),
        base,
        base,
        crate::codegen::html::SKIP_LINK,
        body_html
    )
}

struct SsgContext {
    default_messages: HashMap<String, String>,
    indent: usize,
    base_path: String, // Relative path to root for assets (e.g., ".." for /about)
    link_base: String, // Config base_path for links (e.g., "/WebFluent")
    /// Studio mode: stamp `data-wf-node` on element roots.
    studio: bool,
    /// Node ids keyed by element span (empty unless in studio mode). Matches the
    /// JS codegen's ids because both consult the same map.
    node_map: NodeMap,
    /// The program's `Component` declarations, so a call to one can be expanded
    /// into the static paint instead of a placeholder comment.
    components: HashMap<String, ComponentDecl>,
    /// Component-expansion depth, so a component that (directly or mutually)
    /// calls itself stops instead of recursing forever.
    depth: usize,
    /// What the compiler could work out about this page's data, so lists and
    /// conditionals over seeded values paint statically instead of waiting for
    /// JavaScript. Anything it could not resolve is simply absent.
    scope: Scope,
}

/// How deep component expansion may nest before it gives up and emits the old
/// placeholder. Real component trees are only a few levels; anything deeper is a
/// cycle, and a static renderer must terminate whatever the source says.
const MAX_COMPONENT_DEPTH: usize = 12;

impl SsgContext {
    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    /// The bare `data-wf-node="<id>"` HTML attribute for this element's root, or
    /// `None` when not in studio mode / the node has no id. Callers building an
    /// attribute list push it directly; callers building a tag inline prepend a
    /// space (see [`SsgContext::wf_node_attr_inline`]).
    fn wf_node_attr(&self, ui: &UIElement) -> Option<String> {
        if !self.studio {
            return None;
        }
        self.node_map
            .id_for(ui.span)
            .map(|id| format!("data-wf-node=\"{}\"", id))
    }

    /// Space-prefixed form for embedding directly inside a `<tag …>` (empty when absent).
    fn wf_node_attr_inline(&self, ui: &UIElement) -> String {
        self.wf_node_attr(ui)
            .map(|a| format!(" {}", a))
            .unwrap_or_default()
    }
}

/// Recursively render the App shell for SSG, handling Router nested inside layout wrappers
fn render_app_shell_ssg(
    stmts: &[Statement],
    page_body: &[Statement],
    ctx: &mut SsgContext,
    html: &mut String,
) {
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            let name = match &ui.component {
                ComponentRef::BuiltIn(n) => n.as_str(),
                _ => "",
            };
            if name == "Router" {
                // Replace Router with page content, inside the `<main>` landmark
                // the skip link targets.
                html.push_str(&format!("{}<main id=\"wf-main\">\n", ctx.indent_str()));
                ctx.indent += 1;
                html.push_str(&render_statements(page_body, ctx));
                ctx.indent -= 1;
                html.push_str(&format!("{}</main>\n", ctx.indent_str()));
            } else if stmt_contains_router(stmt) {
                // This is a layout wrapper (like Row) containing the Router
                // Render the wrapper tag with children, substituting the Router
                let (tag, class) = builtin_to_html(name);
                let indent = ctx.indent_str();
                html.push_str(&format!(
                    "{}<{} class=\"{}\"{}>\n",
                    indent,
                    tag,
                    class,
                    ctx.wf_node_attr_inline(ui)
                ));
                ctx.indent += 1;
                render_app_shell_ssg(&ui.children, page_body, ctx, html);
                ctx.indent -= 1;
                html.push_str(&format!("{}</{}>\n", indent, tag));
            } else {
                html.push_str(&render_ui_element(ui, ctx));
            }
        }
    }
}

fn stmt_contains_router(stmt: &Statement) -> bool {
    if let StatementKind::UIElement(ui) = &stmt.kind {
        if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
            return true;
        }
        for child in &ui.children {
            if stmt_contains_router(child) {
                return true;
            }
        }
    }
    false
}

fn render_statements(stmts: &[Statement], ctx: &mut SsgContext) -> String {
    let mut html = String::new();
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => html.push_str(&render_ui_element(ui, ctx)),
            // A condition over seeded data has a knowable answer, so paint the
            // branch it takes. One that depends on the running page does not, and
            // stays a placeholder for the client to fill.
            StatementKind::If(if_stmt) => match render_if_static(if_stmt, ctx) {
                Some(rendered) => html.push_str(&rendered),
                None => html.push_str(&format!("{}<!--wf-if-->\n", ctx.indent_str())),
            },
            // Likewise a list: a store's seeded rows are in the AST, and leaving
            // them to JavaScript meant the page's main content reached neither a
            // crawler nor the first paint.
            StatementKind::For(for_stmt) => match render_for_static(for_stmt, ctx) {
                Some(rendered) => html.push_str(&rendered),
                None => html.push_str(&format!("{}<!--wf-for-->\n", ctx.indent_str())),
            },
            StatementKind::Show(show) => {
                // Render content but hidden
                let inner = render_statements(&show.body, ctx);
                html.push_str(&format!(
                    "{}<div style=\"display:none\">\n{}{}</div>\n",
                    ctx.indent_str(),
                    inner,
                    ctx.indent_str()
                ));
            }
            StatementKind::Fetch(fetch) => {
                // Render loading block if present
                if let Some(loading) = &fetch.loading_block {
                    html.push_str(&render_statements(loading, ctx));
                } else {
                    html.push_str(&format!("{}<!--wf-fetch-->\n", ctx.indent_str()));
                }
            }
            // Skip state, derived, effect, action, use, events, navigate, log, animate
            _ => {}
        }
    }
    html
}

/// Expand a call to a user-declared `Component` into the static paint.
///
/// Before this, every `ComponentRef::UserDefined` rendered as `<!--wf-component-->`
/// and the content appeared only once JS hydrated — so a page built from
/// components painted empty, the "genuine static site" claim was false of any such
/// page, and the SEO rule of exactly one `h1` was unfulfillable when the `h1` lived
/// in a component. Expanding them substitutes the call's arguments for the
/// component's props and renders its body in place.
///
/// Falls back to the placeholder when the component is unknown (a program that
/// wouldn't pass the semantic gate anyway) or when expansion nests too deeply.
fn render_user_component(name: &str, call: &UIElement, ctx: &mut SsgContext) -> String {
    let Some(decl) = ctx.components.get(name).cloned() else {
        return format!("{}<!--wf-component-->\n", ctx.indent_str());
    };
    if ctx.depth >= MAX_COMPONENT_DEPTH {
        return format!("{}<!--wf-component-->\n", ctx.indent_str());
    }

    let bindings = bind_props(&decl, call);
    // The call's own children fill the component's `children` slot.
    let slot: Vec<Statement> = call.children.clone();
    let body: Vec<Statement> = decl
        .body
        .iter()
        .map(|st| substitute_statement(st, &bindings, &slot))
        .collect();

    ctx.depth += 1;
    let html = render_statements(&body, ctx);
    ctx.depth -= 1;
    html
}

/// Bind a call's arguments to a component's props: positional arguments fill the
/// props in declaration order, named arguments bind by name, and any prop left
/// unbound falls back to its declared default. A prop with neither is left
/// unbound, so it renders as empty text exactly as the client would.
fn bind_props(decl: &ComponentDecl, call: &UIElement) -> HashMap<String, Expr> {
    let mut bound: HashMap<String, Expr> = HashMap::new();
    let mut positional = 0usize;
    for arg in &call.args {
        match arg {
            Arg::Positional(expr) => {
                if let Some(prop) = decl.props.get(positional) {
                    bound.insert(prop.name.clone(), expr.clone());
                }
                positional += 1;
            }
            Arg::Named(key, expr) => {
                bound.insert(key.clone(), expr.clone());
            }
        }
    }
    for prop in &decl.props {
        if !bound.contains_key(&prop.name)
            && let Some(default) = &prop.default
        {
            bound.insert(prop.name.clone(), default.clone());
        }
    }
    bound
}

/// Replace bound prop identifiers inside one statement, and fill `children`.
fn substitute_statement(
    stmt: &Statement,
    bindings: &HashMap<String, Expr>,
    slot: &[Statement],
) -> Statement {
    let mut out = stmt.clone();
    if let StatementKind::UIElement(ui) = &stmt.kind {
        // The `children` keyword renders the caller's own block in its place.
        if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "children") {
            // A slot expands to its first statement; multiple children are wrapped
            // by the caller's own element, so this is the shape the JS produces too.
            if let Some(first) = slot.first() {
                return first.clone();
            }
        }
        out.kind = StatementKind::UIElement(substitute_ui(ui, bindings, slot));
    }
    out
}

/// Deep-substitute bound props through one element: its arguments, its style
/// values, and its children.
fn substitute_ui(
    ui: &UIElement,
    bindings: &HashMap<String, Expr>,
    slot: &[Statement],
) -> UIElement {
    let mut out = ui.clone();
    out.args = ui
        .args
        .iter()
        .map(|a| match a {
            Arg::Positional(e) => Arg::Positional(substitute_expr(e, bindings)),
            Arg::Named(k, e) => Arg::Named(k.clone(), substitute_expr(e, bindings)),
        })
        .collect();
    if let Some(style) = &mut out.style_block {
        for prop in &mut style.properties {
            prop.value = substitute_expr(&prop.value, bindings);
        }
    }
    out.children = ui
        .children
        .iter()
        .map(|st| substitute_statement(st, bindings, slot))
        .collect();
    out
}

/// Replace a bound prop identifier with its value, recursing through the shapes a
/// component body actually uses. An unbound identifier is left alone — it may be
/// component-local state, which stays dynamic and renders empty here.
fn substitute_expr(expr: &Expr, bindings: &HashMap<String, Expr>) -> Expr {
    match expr {
        Expr::Identifier(name) => bindings.get(name).cloned().unwrap_or_else(|| expr.clone()),
        Expr::InterpolatedString(parts) => Expr::InterpolatedString(
            parts
                .iter()
                .map(|p| match p {
                    StringPart::Literal(l) => StringPart::Literal(l.clone()),
                    StringPart::Expression(e) => {
                        StringPart::Expression(substitute_expr(e, bindings))
                    }
                })
                .collect(),
        ),
        Expr::BinaryOp(l, op, r) => Expr::BinaryOp(
            Box::new(substitute_expr(l, bindings)),
            op.clone(),
            Box::new(substitute_expr(r, bindings)),
        ),
        Expr::UnaryOp(op, e) => Expr::UnaryOp(op.clone(), Box::new(substitute_expr(e, bindings))),
        Expr::PropertyAccess(obj, prop) => {
            Expr::PropertyAccess(Box::new(substitute_expr(obj, bindings)), prop.clone())
        }
        Expr::FunctionCall(name, args) => Expr::FunctionCall(
            name.clone(),
            args.iter().map(|a| substitute_expr(a, bindings)).collect(),
        ),
        other => other.clone(),
    }
}

fn render_ui_element(ui: &UIElement, ctx: &mut SsgContext) -> String {
    match &ui.component {
        ComponentRef::BuiltIn(name) => render_builtin(name, ui, ctx),
        ComponentRef::SubComponent(parent, sub) => {
            let class = format!("wf-{}__{}", parent.to_lowercase(), camel_to_kebab(sub));
            // A sub-component with a `to:` is a link, and has to render as one.
            // This used to drop the destination and emit a bare `<li>`, so every
            // `Sidebar.Item` in a static build was dead until JavaScript ran —
            // and a crawler saw a navigation panel containing no links at all.
            if let Some(href) = ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == "to" => expr_to_static_string(v),
                _ => None,
            }) {
                let href = if ctx.link_base.is_empty() {
                    href
                } else {
                    format!("{}{}", ctx.link_base, href)
                };
                return render_linked_item(&class, &href, ui, ctx);
            }
            let tag = match sub.as_str() {
                "Item" => "li",
                _ => "div",
            };
            render_tag(tag, &class, ui, ctx)
        }
        ComponentRef::UserDefined(name) => render_user_component(name, ui, ctx),
    }
}

fn render_builtin(name: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let (tag, base_class) = builtin_to_html(name);
    let class_str = class_list(base_class, &ui.modifiers).join(" ");

    // Special handling for certain components. These build their tag inline
    // (not via the attrs list below), so stamp the node id inline here too, and
    // carry the author's `style { }` block, which returning early used to drop.
    let wf = ctx.wf_node_attr_inline(ui);
    let inline_style = {
        let decls = style_block_decls(ui);
        if decls.is_empty() {
            String::new()
        } else {
            format!(" style=\"{}\"", html_escape(&decls.join("; ")))
        }
    };
    match name {
        "Spacer" | "Spinner" => {
            return format!(
                "{}<div class=\"{}\"{}{}></div>\n",
                ctx.indent_str(),
                class_str,
                wf,
                inline_style
            );
        }
        "Divider" => {
            return format!(
                "{}<hr class=\"{}\"{}{}>\n",
                ctx.indent_str(),
                class_str,
                wf,
                inline_style
            );
        }
        // A label above an input, inside a wrapper carrying the component class.
        // These used to paint a bare `<input>`, which the SPA's wrapper could not
        // hydrate onto.
        "Slider" | "DatePicker" | "FileUpload" => {
            return render_labelled_input(name, &class_str, &wf, &inline_style, ui, ctx);
        }
        "Children" | "_StyleBlock" | "Router" | "Route" => {
            return String::new();
        }
        "Toast" => return String::new(), // Imperative, no SSG output
        _ => {}
    }

    // Extract attributes and text content
    let mut attrs = Vec::new();
    let mut text_content: Option<String> = None;

    if let Some(role) = implicit_role(name, &ui.modifiers) {
        attrs.push(format!("role=\"{}\"", role));
    }
    if let Some(label) = landmark_label(name) {
        attrs.push(format!("aria-label=\"{}\"", label));
    }

    if !class_str.is_empty() {
        attrs.push(format!("class=\"{}\"", class_str));
    }

    // Studio: stamp the node id on this element's root (must match the JS codegen id).
    if let Some(a) = ctx.wf_node_attr(ui) {
        attrs.push(a);
    }

    // Per-element inline styles from a `style { }` block (e.g. inspector / AI edits).
    // Collected here and emitted as ONE `style="…"` attribute below so a grid
    // `columns` arg merges into the same attribute instead of producing a duplicate
    // `style=` (HTML keeps the first and silently drops the rest).
    let mut style_decls = style_block_decls(ui);

    for arg in &ui.args {
        match arg {
            Arg::Named(key, val) => {
                match key.as_str() {
                    "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max" | "step"
                    | "accept" | "role" | "value" | "width" | "height" | "loading" | "decoding"
                    | "fetchpriority" => {
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            attrs.push(format!("{}=\"{}\"", key, html_escape(&s)));
                        }
                    }
                    "to" => {
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            // Use config base_path for absolute links
                            let href = if ctx.link_base.is_empty() {
                                s.clone()
                            } else {
                                format!("{}{}", ctx.link_base, s)
                            };
                            attrs.push(format!("href=\"{}\"", html_escape(&href)));
                        }
                    }
                    "required" => attrs.push("required".to_string()),
                    "disabled" => attrs.push("disabled".to_string()),
                    "controls" => attrs.push("controls".to_string()),
                    "title" => {
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            attrs.push(format!("title=\"{}\"", html_escape(&s)));
                        }
                    }
                    "label" => {
                        // For checkbox/radio/switch/slider, the label is visible text
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            text_content = Some(s);
                        }
                    }
                    "columns" => {
                        if let Expr::NumberLiteral(n) = val {
                            style_decls
                                .push(format!("grid-template-columns: repeat({}, 1fr)", *n as i32));
                        }
                    }
                    "visible" | "bind" | "checked" | "icon" | "span" | "gap" | "align"
                    | "justify" => {} // Skip runtime-only attrs
                    _ => {}
                }
            }
            Arg::Positional(expr) => {
                if text_content.is_none() {
                    text_content = resolve_text_scoped(expr, &ctx.default_messages, &ctx.scope);
                }
            }
        }
    }

    // Reserve space and keep offscreen images off the critical path. Without
    // dimensions the browser allocates none, and the page shifts when the image
    // arrives.
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

    // Handle input type from modifiers
    for m in &ui.modifiers {
        if let Some(t) = input_type(m) {
            attrs.push(format!("type=\"{}\"", t));
        }
    }

    // Heading tag override based on modifier
    let actual_tag = if name == "Heading" {
        heading_tag(&ui.modifiers)
    } else {
        tag
    };

    // Emit the collected inline styles (style block + any grid columns) as one attr.
    if !style_decls.is_empty() {
        attrs.push(format!(
            "style=\"{}\"",
            html_escape(&style_decls.join("; "))
        ));
    }

    let indent = ctx.indent_str();
    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    };

    // Self-closing tags
    if matches!(actual_tag, "input" | "img" | "hr" | "br") {
        return format!("{}<{}{}>\n", indent, actual_tag, attrs_str);
    }

    // Has children?
    let has_children = !ui.children.is_empty();
    let has_text = text_content.is_some();

    if !has_children && !has_text {
        return format!("{}<{}{}></{}>\n", indent, actual_tag, attrs_str, actual_tag);
    }

    let mut result = format!("{}<{}{}>\n", indent, actual_tag, attrs_str);

    if let Some(text) = &text_content {
        // Inline text
        if !has_children {
            return format!(
                "{}<{}{}>{}</{}>\n",
                indent,
                actual_tag,
                attrs_str,
                html_escape(text),
                actual_tag
            );
        }
        result.push_str(&format!("{}    {}\n", indent, html_escape(text)));
    }

    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;

    result.push_str(&format!("{}</{}>\n", indent, actual_tag));
    result
}

/// `Slider`, `DatePicker` and `FileUpload`: a wrapper carrying the component
/// class, an optional `<label>`, and the input itself.
///
/// The SPA builds exactly this shape, so the static paint has to match it or
/// hydration reconciles a wrapper against a bare `<input>`.
fn render_labelled_input(
    name: &str,
    class_str: &str,
    wf: &str,
    inline_style: &str,
    ui: &UIElement,
    ctx: &mut SsgContext,
) -> String {
    let named = |key: &str| -> Option<String> {
        ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == key => expr_to_static_string(v),
            _ => None,
        })
    };

    let indent = ctx.indent_str();
    let mut out = format!(
        "{}<div class=\"{}\"{}{}>\n",
        indent, class_str, wf, inline_style
    );

    if let Some(label) = named("label") {
        out.push_str(&format!(
            "{}    <label class=\"wf-form-label\">{}</label>\n",
            indent,
            html_escape(&label)
        ));
    }

    let mut input_attrs = match name {
        "Slider" => vec![
            "type=\"range\"".to_string(),
            format!("min=\"{}\"", named("min").unwrap_or_else(|| "0".into())),
            format!("max=\"{}\"", named("max").unwrap_or_else(|| "100".into())),
            format!("step=\"{}\"", named("step").unwrap_or_else(|| "1".into())),
        ],
        "DatePicker" => vec![
            "type=\"date\"".to_string(),
            "class=\"wf-input\"".to_string(),
        ],
        _ => vec![
            "type=\"file\"".to_string(),
            "class=\"wf-input\"".to_string(),
        ],
    };
    for key in ["min", "max", "accept", "value"] {
        if name != "Slider" || !matches!(key, "min" | "max") {
            if let Some(v) = named(key) {
                input_attrs.push(format!("{}=\"{}\"", key, html_escape(&v)));
            }
        }
    }
    if ui.modifiers.iter().any(|m| m == "multiple") {
        input_attrs.push("multiple".to_string());
    }

    out.push_str(&format!(
        "{}    <input {}>\n",
        indent,
        input_attrs.join(" ")
    ));
    out.push_str(&format!("{}</div>\n", indent));
    out
}

/// Paint the branch a resolvable condition takes, or `None` to defer to the client.
fn render_if_static(if_stmt: &IfStmt, ctx: &mut SsgContext) -> Option<String> {
    if eval(&if_stmt.condition, &ctx.scope)?.truthy() {
        return Some(render_statements(&if_stmt.then_body, ctx));
    }
    for (cond, body) in &if_stmt.else_if_branches {
        if eval(cond, &ctx.scope)?.truthy() {
            return Some(render_statements(body, ctx));
        }
    }
    Some(match &if_stmt.else_body {
        Some(body) => render_statements(body, ctx),
        None => String::new(),
    })
}

/// Paint one copy of the body per item, or `None` to defer to the client.
fn render_for_static(for_stmt: &ForStmt, ctx: &mut SsgContext) -> Option<String> {
    let Static::List(items) = eval(&for_stmt.iterable, &ctx.scope)? else {
        return None;
    };

    let outer = ctx.scope.clone();
    let mut html = String::new();
    for (i, item) in items.iter().enumerate() {
        ctx.scope = outer.with(&for_stmt.item, item.clone());
        if let Some(index_name) = &for_stmt.index {
            ctx.scope = ctx.scope.with(index_name, Static::Num(i as f64));
        }
        html.push_str(&render_statements(&for_stmt.body, ctx));
    }
    ctx.scope = outer;
    Some(html)
}

/// A navigation item that carries a destination: an `<a>`, as the SPA builds it.
fn render_linked_item(class: &str, href: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let indent = ctx.indent_str();
    let wf = ctx.wf_node_attr_inline(ui);
    let mut out = format!(
        "{}<a class=\"{}\" href=\"{}\"{}>\n",
        indent,
        class,
        html_escape(href),
        wf
    );
    ctx.indent += 1;
    out.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;
    out.push_str(&format!("{}</a>\n", indent));
    out
}

fn render_tag(tag: &str, class: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let indent = ctx.indent_str();
    let wf = ctx.wf_node_attr_inline(ui);
    let decls = style_block_decls(ui);
    let style_attr = if decls.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", html_escape(&decls.join("; ")))
    };
    let mut result = format!(
        "{}<{} class=\"{}\"{}{}>\n",
        indent, tag, class, wf, style_attr
    );
    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;
    result.push_str(&format!("{}</{}>\n", indent, tag));
    result
}

/// Static CSS declarations from an element's `style { }` block, for inlining as a
/// `style="…"` attribute so per-element inspector/AI style edits appear in the
/// static SSG paint. The JS bundle applies the same styles at hydration, but in SSG
/// mode the pre-painted DOM is kept, so the static HTML must carry them too.
fn style_block_decls(ui: &UIElement) -> Vec<String> {
    use super::style_tokens::{canonical_style_prop, resolve_style_token};
    let Some(sb) = &ui.style_block else {
        return Vec::new();
    };
    sb.properties
        .iter()
        .filter_map(|p| {
            // Apply the property aliases (radius→border-radius, shadow→box-shadow), then
            // resolve a bare design-token keyword (`font-size: xl`) to its `var(--…)`;
            // otherwise fall back to a static literal (quoted CSS / number).
            let prop = canonical_style_prop(&p.name);
            let value =
                resolve_style_token(&prop, &p.value).or_else(|| expr_to_static_string(&p.value))?;
            Some(format!("{prop}: {value}"))
        })
        .collect()
}

/// An attribute value the compiler can write out, consulting the build-time
/// scope so a loop binding reaches `src=`, `href=` and the rest.
fn static_attr(expr: &Expr, scope: &Scope) -> Option<String> {
    expr_to_static_string(expr).or_else(|| match eval(expr, scope)? {
        Static::List(_) | Static::Map(_) => None,
        value => Some(value.to_text()),
    })
}

/// Try to resolve an expression to a static string.
fn expr_to_static_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::StringLiteral(s) => Some(s.clone()),
        Expr::NumberLiteral(n) => Some(format!("{}", n)),
        Expr::BoolLiteral(b) => Some(format!("{}", b)),
        _ => None, // Dynamic — can't resolve
    }
}

/// Resolve text content, including i18n t() calls and anything the build-time
/// scope knows — which is what lets `Tcell(row.title)` inside a resolved loop
/// paint the actual title rather than nothing.
fn resolve_text_scoped(
    expr: &Expr,
    messages: &HashMap<String, String>,
    scope: &Scope,
) -> Option<String> {
    resolve_text(expr, messages).or_else(|| match eval(expr, scope)? {
        // A collection has no text form; painting "" would be a lie about what
        // the running page shows.
        Static::List(_) | Static::Map(_) => None,
        value => Some(value.to_text()),
    })
}

/// Resolve text content, including i18n t() calls.
fn resolve_text(expr: &Expr, messages: &HashMap<String, String>) -> Option<String> {
    match expr {
        Expr::StringLiteral(s) => Some(s.clone()),
        Expr::NumberLiteral(n) => {
            if *n == (*n as i64) as f64 {
                Some(format!("{}", *n as i64))
            } else {
                Some(format!("{}", n))
            }
        }
        Expr::BoolLiteral(b) => Some(format!("{}", b)),
        Expr::FunctionCall(name, args) if name == "t" => {
            // i18n: resolve from default locale
            if let Some(Expr::StringLiteral(key)) = args.first() {
                messages.get(key).cloned().or_else(|| Some(key.clone()))
            } else {
                None
            }
        }
        _ => None, // Dynamic expression — leave empty for client
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
        .replace('\u{FFFE}', "{")
        .replace('\u{FFFF}', "}")
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

#[cfg(test)]
mod component_expansion_tests {
    //! A page built from components must PAINT its content, not a placeholder.
    //!
    //! Before this, every user component rendered as `<!--wf-component-->` and its
    //! content existed only after JS hydrated — so a component-built page painted
    //! empty, "genuine static site" was false of it, and an `h1` inside a component
    //! never reached the served HTML.
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn render(src: &str) -> String {
        let tokens = Lexer::new(src, "<t>").tokenize().expect("lex");
        let program = Parser::new(tokens, "<t>").parse().expect("parse");
        let components: HashMap<String, ComponentDecl> = program
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Component(c) => Some((c.name.clone(), c.clone())),
                _ => None,
            })
            .collect();
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
            .expect("a page");
        let cfg: ProjectConfig = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
        render_page_html(
            page,
            &SiteContext {
                config: &cfg,
                app_body: None,
                translations: &Default::default(),
                components: &components,
                program: &program,
            },
        )
    }

    #[test]
    fn a_component_call_is_expanded_with_its_props_bound() {
        let html = render(
            "Component Hero (title: String, tagline: String) {\n  Container { Heading(title, h1) Text(tagline) }\n}\n\
             Page Home (path: \"/\") { Hero(\"Beit Qahwa\", \"Slow roasted\") }\n",
        );
        assert!(
            !html.contains("wf-component"),
            "no placeholder should survive: {html}"
        );
        assert!(
            html.contains("Beit Qahwa"),
            "the positional prop must render: {html}"
        );
        assert!(html.contains("Slow roasted"));
        assert!(
            html.contains("<h1"),
            "an h1 inside a component must reach the HTML (SEO)"
        );
    }

    #[test]
    fn named_arguments_and_defaults_both_bind() {
        let html = render(
            "Component Item (name: String, note: String = \"none\") {\n  Text(name) Text(note)\n}\n\
             Page Home (path: \"/\") { Item(name: \"Latte\") }\n",
        );
        assert!(html.contains("Latte"), "named arg: {html}");
        assert!(
            html.contains("none"),
            "declared default fills an unbound prop: {html}"
        );
    }

    #[test]
    fn a_component_calling_a_component_expands_both() {
        let html = render(
            "Component Inner (t: String) { Text(t) }\n\
             Component Outer (t: String) { Container { Inner(t) } }\n\
             Page Home (path: \"/\") { Outer(\"nested\") }\n",
        );
        assert!(
            html.contains("nested"),
            "props thread through both levels: {html}"
        );
        assert!(!html.contains("wf-component"));
    }

    /// A component that calls itself must terminate — a static renderer cannot
    /// loop forever whatever the source says.
    #[test]
    fn self_recursion_stops_at_the_depth_limit() {
        let html = render(
            "Component Loop (t: String) { Container { Loop(t) } }\n\
             Page Home (path: \"/\") { Loop(\"x\") }\n",
        );
        assert!(
            html.contains("wf-component"),
            "the guard emits the placeholder at the limit"
        );
    }

    /// An undeclared component keeps the old placeholder rather than panicking —
    /// such a program fails the semantic gate anyway.
    #[test]
    fn an_unknown_component_still_renders_a_placeholder() {
        let tokens = Lexer::new("Page Home (path: \"/\") { Ghost() }", "<t>")
            .tokenize()
            .unwrap();
        let program = Parser::new(tokens, "<t>").parse().unwrap();
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
            .unwrap();
        let cfg: ProjectConfig = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
        let html = render_page_html(page, &SiteContext::bare(&cfg, &program));
        assert!(html.contains("wf-component"));
    }
}
