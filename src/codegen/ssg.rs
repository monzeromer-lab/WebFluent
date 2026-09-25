use crate::codegen::builtin::{
    builtin_to_html, class_list, element_tag, implicit_role, input_type, landmark_label,
    layout_arg_classes,
};
use crate::codegen::node_id::NodeMap;
use crate::codegen::static_eval::{Scope, Static, case_of, eval};
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

/// [`render_page_html`] for one value of a `:param` route: the page's
/// parameters are seeded, by name and as `params.name`, so the body reads
/// them as the live page would.
pub fn render_page_html_with_params(
    page: &PageDecl,
    site: &SiteContext,
    route: &str,
    params: &HashMap<String, Static>,
) -> String {
    let mut seeded = page.clone();
    // The file is one concrete route: its canonical link, its sharing card
    // and its asset paths are that route's, not the pattern's.
    seeded.path = route.to_string();
    // Seed as declarations at the top of the body: `params` as a map, and
    // each parameter as a constant of the page.
    let mut extra: Vec<Statement> = Vec::new();
    let map = Static::Map(params.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    extra.push(Statement::new(
        StatementKind::State(StateDecl {
            name: "params".to_string(),
            ty: None,
            value: static_expr(&map),
            persist: false,
            policy: None,
        }),
        page.span,
    ));
    for (k, v) in params {
        extra.push(Statement::new(
            StatementKind::State(StateDecl {
                name: k.clone(),
                ty: None,
                value: static_expr(v),
                persist: false,
                policy: None,
            }),
            page.span,
        ));
    }
    extra.append(&mut seeded.body);
    seeded.body = extra;
    render_page_html_studio(&seeded, site, false, &NodeMap::default())
}

/// The concrete routes of a `:param` page with `paths:`, each with its
/// parameters: one value fills a route's one parameter; a map fills
/// several by name.
pub fn static_routes(
    page: &PageDecl,
    program: &Program,
    env: &std::collections::BTreeMap<String, serde_json::Value>,
) -> crate::error::Result<Vec<(String, HashMap<String, Static>)>> {
    use crate::error::WebFluentError;
    let Some(paths) = &page.paths else {
        return Ok(Vec::new());
    };
    let scope = Scope::from_program_with_env(program, &[], env);
    let Some(Static::List(values)) = eval(paths, &scope) else {
        return Err(WebFluentError::CodegenError(format!(
            "page {}: `paths:` must be a list known at build time, such as `posts.map(p => p.slug)` over a `data` file",
            page.name
        )));
    };
    let names: Vec<String> = page
        .path
        .split('/')
        .filter_map(|seg| seg.strip_prefix(':').map(str::to_string))
        .collect();
    let mut out = Vec::new();
    for value in values {
        let mut params = HashMap::new();
        match (&value, names.len()) {
            (Static::Map(fields), _) => {
                for name in &names {
                    let Some(v) = fields.iter().find(|(k, _)| k == name).map(|(_, v)| v) else {
                        return Err(WebFluentError::CodegenError(format!(
                            "page {}: a value of `paths:` has no `{name}` for the route {}",
                            page.name, page.path
                        )));
                    };
                    params.insert(name.clone(), v.clone());
                }
            }
            (v, 1) => {
                params.insert(names[0].clone(), v.clone());
            }
            _ => {
                return Err(WebFluentError::CodegenError(format!(
                    "page {}: the route {} has {} parameters, so each value of `paths:` must be a map naming them",
                    page.name,
                    page.path,
                    names.len()
                )));
            }
        }
        let mut route = page.path.clone();
        for name in &names {
            let text = params.get(name).map(|v| v.to_text()).unwrap_or_default();
            route = route.replace(&format!(":{name}"), &text);
        }
        out.push((route, params));
    }
    Ok(out)
}

/// A static value as the expression that writes it.
fn static_expr(value: &Static) -> Expr {
    crate::data::json_expr(&value.to_json())
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
    // The catch-all page is written to the root as 404.html.
    let base_path = if route.is_empty() || route == "/" || route == "*" {
        ".".to_string()
    } else {
        let depth = route.split('/').filter(|s| !s.is_empty()).count();
        (0..depth).map(|_| "..").collect::<Vec<_>>().join("/")
    };

    let link_base = config.build.base_path.clone();

    let mut ctx = SsgContext {
        images: 0,
        fields: 0,
        default_messages,
        indent: 2,
        base_path,
        link_base,
        studio,
        node_map: node_map.clone(),
        components: components.clone(),
        scope: Scope::from_program_with_env(program, &page.body, &site.config.env)
            .with_locale(default_locale),
        depth: 0,
        in_thead: false,
        current_path: page.path.clone(),
    };

    // A page with a layout is that component with the page as its default
    // slot; the shell then wraps the layout as it would wrap the page.
    let framed: Vec<Statement>;
    let page_body: &[Statement] = match &page.layout {
        Some(layout) => {
            framed = vec![Statement::new(
                StatementKind::UIElement(UIElement {
                    component: ComponentRef::UserDefined(layout.name.clone()),
                    args: layout.args.clone(),
                    modifiers: Vec::new(),
                    children: page.body.clone(),
                    style_block: None,
                    transition_block: None,
                    events: Vec::new(),
                    slot_fills: Vec::new(),
                    span: layout.span,
                    paren_span: None,
                    body_span: None,
                    style_span: None,
                    arg_spans: Vec::new(),
                    modifier_spans: Vec::new(),
                }),
                layout.span,
            )];
            &framed
        }
        None => &page.body,
    };

    // Render app shell (navbar, etc.) if available
    let mut body_html = String::new();
    if let Some(app_stmts) = app_body {
        render_app_shell_ssg(app_stmts, page_body, &mut ctx, &mut body_html);
    } else {
        // With no shell there is no Router to stand in for `<main>`, so the page
        // body is the main content itself.
        body_html = format!(
            "{}<main id=\"wf-main\">\n{}{}</main>\n",
            ctx.indent_str(),
            {
                ctx.indent += 1;
                let inner = render_statements(page_body, &mut ctx);
                ctx.indent -= 1;
                inner
            },
            ctx.indent_str()
        );
    }

    // Description, canonical, sharing card, language alternates and JSON-LD, all
    // derived from what the page and the config already say.
    let mut description_meta = crate::codegen::seo::head_tags(page, config, program);
    // The page's own `head { }` tags, with what is known at build time; the
    // runtime replaces them once live, so each is marked as its own.
    let head_scope = Scope::from_program_with_env(program, &page.body, &site.config.env);
    for tag in &page.head {
        let mut attrs = String::new();
        for (k, v) in &tag.attrs {
            match eval(v, &head_scope) {
                Some(Static::Bool(true)) => attrs.push_str(&format!(" {k}")),
                Some(Static::Bool(false)) | Some(Static::Null) | None => {}
                Some(value) => {
                    attrs.push_str(&format!(" {k}=\"{}\"", html_escape(&value.to_text())))
                }
            }
        }
        // `meta` and `link` are void; a `script` closes.
        if tag.tag == "script" {
            description_meta.push_str(&format!("    <script data-wf-head{attrs}></script>\n"));
        } else {
            description_meta.push_str(&format!("    <{} data-wf-head{attrs}>\n", tag.tag));
        }
    }

    // Calculate relative path prefix based on page route depth. The catch-all
    // is served for any path a static host has no file for, at any depth, so
    // its assets are addressed from the site root: a relative `./app.js` from
    // `/app/builds/8f2c41` used to fetch a second 404 page as the script.
    let route = page.path.trim_start_matches('/');
    let base = if route == "*" {
        let root = config.build.base_path.trim_end_matches('/');
        if root.is_empty() {
            String::new()
        } else {
            root.to_string()
        }
    } else if route.is_empty() || route == "/" {
        ".".to_string()
    } else {
        let depth = route.split('/').filter(|s| !s.is_empty()).count();
        (0..depth).map(|_| "..").collect::<Vec<_>>().join("/")
    };

    // The page's own chunk, linked beside app.js so the two load in parallel
    // and the router finds the page registered by the time it runs.
    let page_chunk = if config.build.split {
        format!(
            "    <script src=\"{}/pages/{}.js\" data-wf-page=\"{}\" defer></script>\n",
            base, page.name, page.name
        )
    } else {
        String::new()
    };
    // The rules only this page reaches, in a sheet of its own after the
    // shared one; the router loads it before drawing the page.
    let page_sheet = if config.build.split
        && crate::codegen::scoped_css::split_rules(program)
            .pages
            .contains_key(&page.name)
    {
        format!(
            "    <link rel=\"stylesheet\" href=\"{}/pages/{}.css\" data-wf-page-css=\"{}\">\n",
            base, page.name, page.name
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
{}{}{}    <link rel="stylesheet" href="{}/styles.css">
{}{}    <script src="{}/app.js" defer></script>
{}</head>
<body>
{}    <div id="app">
{}    </div>
</body>
</html>"#,
        lang,
        title,
        description_meta,
        crate::codegen::html::csp_meta(config),
        crate::codegen::html::head_links(config, &base),
        base,
        page_sheet,
        crate::codegen::html::externals_tags(config, program, &base),
        base,
        page_chunk,
        crate::codegen::html::SKIP_LINK,
        body_html
    )
}

struct SsgContext {
    /// Images painted so far on this page; the first is fetched eagerly.
    images: usize,
    /// Fields painted so far, for their ids.
    fields: usize,
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
    /// Inside a `Thead`, a `Tcell` is a column header (`<th scope="col">`).
    in_thead: bool,
    /// The route of the page being painted, so a link to it can be marked
    /// current in the static HTML as the runtime marks it after hydration.
    current_path: String,
}

/// Whether a link to `href` points at `current` — exactly, or, with `prefix`,
/// at a route beneath it. Mirrors `WF.activeLink` in the runtime.
fn link_is_current(href: &str, current: &str, prefix: bool) -> bool {
    let norm = |p: &str| {
        let t = p.trim_end_matches('/');
        if t.is_empty() {
            "/".to_string()
        } else {
            t.to_string()
        }
    };
    let (href, current) = (norm(href), norm(current));
    href == current || (prefix && href != "/" && current.starts_with(&format!("{}/", href)))
}

/// How deep component expansion may nest before it gives up and emits the old
/// placeholder. Real component trees are only a few levels; anything deeper is a
/// cycle, and a static renderer must terminate whatever the source says.
const MAX_COMPONENT_DEPTH: usize = 12;

impl SsgContext {
    fn next_field_id(&mut self) -> usize {
        self.fields += 1;
        self.fields
    }

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
                // A class, not `style="display:none"`: an inline style is
                // the one thing a strict `style-src` forbids, and the
                // runtime sets `display` on this wrapper anyway, which
                // beats a class.
                html.push_str(&format!(
                    "{}<div class=\"wf-hidden\">\n{}{}</div>\n",
                    ctx.indent_str(),
                    inner,
                    ctx.indent_str()
                ));
            }
            // The `fetch` block of the original grammar reaches only the
            // migrator, which rewrites it as a resource and a match.
            StatementKind::Fetch(_) => {}
            // A resource is still loading when the static page is painted;
            // a match over one paints its `loading` arm. A match over an
            // enum paints the arm of the case the value has at build time,
            // with the payload bound, and its `else` arm when the value is
            // not known until run time.
            StatementKind::Match(m) => {
                let over_resource = m.arms.iter().any(|a| {
                    matches!(
                        a.pattern,
                        ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready
                    )
                });
                let value = if over_resource {
                    None
                } else {
                    eval(&m.scrutinee, &ctx.scope)
                };
                let by_case = value.as_ref().and_then(|v| {
                    let case = case_of(v);
                    m.arms
                        .iter()
                        .find(|a| matches!(&a.pattern, ArmPattern::Case(c) if Static::Str(c.clone()) == case))
                });
                let arm = by_case
                    .or_else(|| m.arms.iter().find(|a| a.pattern == ArmPattern::Loading))
                    .or_else(|| m.arms.iter().find(|a| a.pattern == ArmPattern::Else));
                match arm {
                    Some(arm) => {
                        let outer = ctx.scope.clone();
                        if let (Some(Static::List(items)), false) =
                            (&value, arm.bindings.is_empty())
                        {
                            for (name, item) in arm.bindings.iter().zip(items.iter().skip(1)) {
                                ctx.scope = ctx.scope.with(name, item.clone());
                            }
                        }
                        html.push_str(&render_statements(&arm.body, ctx));
                        ctx.scope = outer;
                    }
                    None => html.push_str(&format!("{}<!--wf-match-->\n", ctx.indent_str())),
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
/// The caller's fills by slot: the names a fill gives the slot's values
/// paired with what the declaration calls them, and the fill's block.
type Fills = HashMap<String, (Vec<(String, String)>, Vec<Statement>)>;

fn render_user_component(name: &str, call: &UIElement, ctx: &mut SsgContext) -> String {
    let Some(decl) = ctx.components.get(name).cloned() else {
        return format!("{}<!--wf-component-->\n", ctx.indent_str());
    };
    if ctx.depth >= MAX_COMPONENT_DEPTH {
        return format!("{}<!--wf-component-->\n", ctx.indent_str());
    }

    let bindings = bind_props(&decl, call);
    // The call's own children fill the component's default slot, and each
    // `name { … }` fill in its block a named one. A scoped fill's names
    // stand for what the slot hands over, in the slot's order.
    let mut slots: Fills = HashMap::new();
    slots.insert("children".to_string(), (Vec::new(), call.children.clone()));
    for fill in &call.slot_fills {
        let handed: Vec<String> = decl
            .slots
            .iter()
            .find(|s| s.name.as_deref() == Some(fill.name.as_str()))
            .map(|s| s.params.iter().map(|p| p.name.clone()).collect())
            .unwrap_or_default();
        let params = fill
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    p.clone(),
                    handed.get(i).cloned().unwrap_or_else(|| p.clone()),
                )
            })
            .collect();
        slots.insert(fill.name.clone(), (params, fill.body.clone()));
    }
    let body: Vec<Statement> = decl
        .body
        .iter()
        .flat_map(|st| substitute_statement(st, &bindings, &slots))
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
///
/// The `children` slot expands to every statement of the caller's block — as
/// written, not substituted, since they belong to the caller's scope. The
/// parser names the slot element `Children`; this used to look for the
/// lowercase keyword and so never matched.
fn substitute_statement(
    stmt: &Statement,
    bindings: &HashMap<String, Expr>,
    slots: &Fills,
) -> Vec<Statement> {
    let mut out = stmt.clone();
    match &stmt.kind {
        StatementKind::UIElement(ui) => {
            if let Some(name) = ui.slot_name() {
                let Some((params, body)) = slots.get(name) else {
                    return Vec::new();
                };
                if params.is_empty() {
                    return body.clone();
                }
                // The fill's names stand for the values the slot use hands
                // over, which are in the component's scope, so they are
                // substituted with the props first.
                let mut handed: HashMap<String, Expr> = HashMap::new();
                for (param, key) in params {
                    let value = ui
                        .args
                        .iter()
                        .find_map(|a| match a {
                            Arg::Named(k, v) if k == key => Some(substitute_expr(v, bindings)),
                            _ => None,
                        })
                        .unwrap_or(Expr::Null);
                    handed.insert(param.clone(), value);
                }
                let none = HashMap::new();
                return body
                    .iter()
                    .flat_map(|st| substitute_statement(st, &handed, &none))
                    .collect();
            }
            out.kind = StatementKind::UIElement(substitute_ui(ui, bindings, slots));
        }
        // A slot used under a branch or a loop is filled there too.
        StatementKind::If(i) => {
            let mut i = i.clone();
            i.condition = substitute_expr(&i.condition, bindings);
            i.then_body = substitute_all(&i.then_body, bindings, slots);
            for (cond, branch) in &mut i.else_if_branches {
                *cond = substitute_expr(cond, bindings);
                *branch = substitute_all(branch, bindings, slots);
            }
            if let Some(b) = &i.else_body {
                i.else_body = Some(substitute_all(b, bindings, slots));
            }
            out.kind = StatementKind::If(i);
        }
        StatementKind::For(f) => {
            let mut f = f.clone();
            f.iterable = substitute_expr(&f.iterable, bindings);
            // The loop's own names shadow a prop of the same name.
            let mut inner = bindings.clone();
            inner.remove(&f.item);
            if let Some(index) = &f.index {
                inner.remove(index);
            }
            f.body = substitute_all(&f.body, &inner, slots);
            out.kind = StatementKind::For(f);
        }
        StatementKind::Show(s) => {
            let mut s = s.clone();
            s.condition = substitute_expr(&s.condition, bindings);
            s.body = substitute_all(&s.body, bindings, slots);
            out.kind = StatementKind::Show(s);
        }
        StatementKind::Match(m) => {
            let mut m = m.clone();
            m.scrutinee = substitute_expr(&m.scrutinee, bindings);
            for arm in &mut m.arms {
                arm.body = substitute_all(&arm.body, bindings, slots);
            }
            out.kind = StatementKind::Match(m);
        }
        _ => {}
    }
    vec![out]
}

fn substitute_all(
    stmts: &[Statement],
    bindings: &HashMap<String, Expr>,
    slots: &Fills,
) -> Vec<Statement> {
    stmts
        .iter()
        .flat_map(|st| substitute_statement(st, bindings, slots))
        .collect()
}

/// Deep-substitute bound props through one element: its arguments, its style
/// values, and its children.
fn substitute_ui(ui: &UIElement, bindings: &HashMap<String, Expr>, slots: &Fills) -> UIElement {
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
        .flat_map(|st| substitute_statement(st, bindings, slots))
        .collect();
    for fill in &mut out.slot_fills {
        fill.body = fill
            .body
            .iter()
            .flat_map(|st| substitute_statement(st, bindings, slots))
            .collect();
    }
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
        Expr::MethodCall(obj, method, args) => Expr::MethodCall(
            Box::new(substitute_expr(obj, bindings)),
            method.clone(),
            args.iter().map(|a| substitute_expr(a, bindings)).collect(),
        ),
        Expr::OptionalMethod(obj, method, args) => Expr::OptionalMethod(
            Box::new(substitute_expr(obj, bindings)),
            method.clone(),
            args.iter().map(|a| substitute_expr(a, bindings)).collect(),
        ),
        Expr::OptionalProperty(obj, prop) => {
            Expr::OptionalProperty(Box::new(substitute_expr(obj, bindings)), prop.clone())
        }
        Expr::IndexAccess(obj, index) => Expr::IndexAccess(
            Box::new(substitute_expr(obj, bindings)),
            Box::new(substitute_expr(index, bindings)),
        ),
        Expr::OptionalIndex(obj, index) => Expr::OptionalIndex(
            Box::new(substitute_expr(obj, bindings)),
            Box::new(substitute_expr(index, bindings)),
        ),
        Expr::ListLiteral(items) => {
            Expr::ListLiteral(items.iter().map(|i| substitute_expr(i, bindings)).collect())
        }
        Expr::CaseValue(case, args) => Expr::CaseValue(
            case.clone(),
            args.iter().map(|a| substitute_expr(a, bindings)).collect(),
        ),
        Expr::MapLiteral(pairs) => Expr::MapLiteral(
            pairs
                .iter()
                .map(|(k, v)| (k.clone(), substitute_expr(v, bindings)))
                .collect(),
        ),
        Expr::Record(name, pairs) => Expr::Record(
            name.clone(),
            pairs
                .iter()
                .map(|(k, v)| (k.clone(), substitute_expr(v, bindings)))
                .collect(),
        ),
        Expr::Spread(inner) => Expr::Spread(Box::new(substitute_expr(inner, bindings))),
        Expr::Range(a, b, inclusive) => Expr::Range(
            Box::new(substitute_expr(a, bindings)),
            Box::new(substitute_expr(b, bindings)),
            *inclusive,
        ),
        Expr::Await(inner) => Expr::Await(Box::new(substitute_expr(inner, bindings))),
        // A lambda's own parameters (`a, b` for several) shadow the props.
        Expr::Lambda(param, body) => {
            let mut inner = bindings.clone();
            for p in param.split(',') {
                inner.remove(p.trim());
            }
            Expr::Lambda(param.clone(), Box::new(substitute_expr(body, &inner)))
        }
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
            // The destination is read through the build-time scope, not as a
            // literal alone: inside a resolved `for`, `to: "/docs/{p}"` is
            // only knowable once `p` is bound. Reading it as a literal left
            // every item a loop drew falling through to the `<li>` below —
            // which is how the components reference, written as
            // `for c in shown { if … { Sidebar.Item(to: …) } }`, pre-rendered
            // a navigation panel with no links in it.
            if let Some(href) = ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == "to" => {
                    resolve_text_scoped(v, &ctx.default_messages, &ctx.scope)
                }
                _ => None,
            }) {
                let prefix = ui.args.iter().any(|a| {
                    matches!(a, Arg::Named(k, Expr::StringLiteral(v)) if k == "active" && v == "prefix")
                });
                let current = link_is_current(&href, &ctx.current_path, prefix);
                let href = if ctx.link_base.is_empty() {
                    href
                } else {
                    format!("{}{}", ctx.link_base, href)
                };
                return render_linked_item(&class, &href, current, ui, ctx);
            }
            // A `Breadcrumb.Item` without `to` is the current page, a
            // `<span>` inside the `<nav>` as the SPA draws it; a list's item
            // is an `<li>`.
            let tag = match (parent.as_str(), sub.as_str()) {
                ("Breadcrumb", "Item") => "span",
                (_, "Item") => "li",
                _ => "div",
            };
            render_tag(tag, &class, ui, ctx)
        }
        ComponentRef::UserDefined(name) => render_user_component(name, ui, ctx),
    }
}

fn render_builtin(name: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let (_, base_class) = builtin_to_html(name);
    let mut classes = class_list(base_class, &ui.modifiers);
    classes.extend(layout_arg_classes(&ui.args));
    classes.extend(extra_classes(ui, ctx));
    let mut current_link = false;
    let class_str = classes.join(" ");

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
        // `Unsafe.Html(markup)`: the markup, as markup. The paint and the
        // live page put in the same thing, because `sanitize` runs here as
        // well — a static article body is not an empty box until the
        // script loads, which is the point of pre-rendering it.
        "UnsafeHtml" => {
            let markup = ui
                .args
                .iter()
                .find_map(|a| match a {
                    Arg::Positional(e) => static_attr(e, &ctx.scope),
                    _ => None,
                })
                .unwrap_or_default();
            return format!(
                "{}<div class=\"{}\"{}{}>{}</div>\n",
                ctx.indent_str(),
                class_str,
                wf,
                inline_style,
                markup
            );
        }
        // Markdown known at build time is painted as HTML; the runtime
        // repaints it the same way.
        "Markdown" => {
            let text = ui
                .args
                .iter()
                .find_map(|a| match a {
                    Arg::Positional(e) => resolve_text_scoped(e, &ctx.default_messages, &ctx.scope),
                    _ => None,
                })
                .unwrap_or_default();
            return format!(
                "{}<div class=\"{}\"{}{}>\n{}{}</div>\n",
                ctx.indent_str(),
                class_str,
                wf,
                inline_style,
                crate::codegen::markdown::render_with_base(&text, &ctx.link_base),
                ctx.indent_str()
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
        // `Image(hero, …)` where `hero` is an `image` the program declares:
        // the `<picture>` the build made, painted whole, so the static page
        // shows the right file at the right size with nothing shifting.
        "Image" if ui.args.iter().any(|a| matches!(a, Arg::Positional(_))) => {
            return render_picture(&class_str, ui, ctx);
        }
        // A video's captions are a `<track>` inside it, and a player's
        // transcript a link beneath it.
        "Video" | "Audio"
            if ui
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, _) if k == "captions" || k == "transcript")) =>
        {
            return render_media(name, &class_str, ui, ctx);
        }
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
    let style_decls = style_block_decls(ui);

    for arg in &ui.args {
        match arg {
            Arg::Named(key, val) => {
                match key.as_str() {
                    "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max" | "step"
                    | "accept" | "role" | "value" | "width" | "height" | "loading" | "decoding"
                    | "fetchpriority" | "rows" => {
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            // A value the browser follows goes through the
                            // same scheme check the live page applies.
                            let s = if crate::codegen::url::URL_ATTRS.contains(&key.as_str()) {
                                crate::codegen::url::guard(&s).to_string()
                            } else {
                                s
                            };
                            attrs.push(format!("{}=\"{}\"", key, html_escape(&s)));
                        }
                    }
                    // `maxLength` is `maxlength` in HTML, and the paint
                    // shows the same attribute the live page sets.
                    "maxLength" => {
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            attrs.push(format!("maxlength=\"{}\"", html_escape(&s)));
                        }
                    }
                    "to" => {
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            let s = crate::codegen::url::guard(&s).to_string();
                            // Use config base_path for absolute links
                            let href = if ctx.link_base.is_empty() {
                                s.clone()
                            } else {
                                format!("{}{}", ctx.link_base, s)
                            };
                            attrs.push(format!("href=\"{}\"", html_escape(&href)));
                            let prefix = ui.args.iter().any(|a| {
                                matches!(a, Arg::Named(k, Expr::StringLiteral(v)) if k == "active" && v == "prefix")
                            });
                            if name == "Link" && link_is_current(&s, &ctx.current_path, prefix) {
                                attrs.push("aria-current=\"page\"".to_string());
                                current_link = true;
                            }
                        }
                    }
                    "active" => {} // Consumed with `to` above
                    "required" => attrs.push("required".to_string()),
                    "disabled" => attrs.push("disabled".to_string()),
                    "controls" => attrs.push("controls".to_string()),
                    "title" => {
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            attrs.push(format!("title=\"{}\"", html_escape(&s)));
                        }
                    }
                    "caption" if name == "Table" => {} // Rendered as the first child below
                    "label" if name == "IconButton" => {
                        // An icon button's label is its accessible name, not
                        // visible text: it used to be painted as a word next
                        // to the glyph until the runtime replaced it.
                        if let Some(s) = static_attr(val, &ctx.scope) {
                            attrs.push(format!("aria-label=\"{}\"", html_escape(&s)));
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
                        if let Some(Static::Num(n)) = eval(val, &ctx.scope) {
                            attrs.push(format!("data-cols=\"{}\"", n as i32));
                        }
                    }
                    "icon" => {
                        if let Some(v) = static_attr(val, &ctx.scope) {
                            attrs.push(format!("data-icon=\"{}\"", html_escape(&v)));
                        }
                    }
                    "visible" | "bind" | "checked" | "span" => {} // Runtime-only attrs
                    "gap" | "align" | "justify" => {}             // Utility classes, added below
                    // A hyphenated name is an HTML attribute (`aria-*`, `data-*`);
                    // it is painted when its value is known at build time.
                    k if k.contains('-') => {
                        if let Some(v) = static_attr(val, &ctx.scope) {
                            attrs.push(format!("{}=\"{}\"", k, html_escape(&v)));
                        }
                    }
                    _ => {}
                }
            }
            Arg::Positional(expr) => {
                // `Icon("home")` names the glyph; the runtime draws it from
                // `data-icon`, so it must not become visible text.
                if name == "Icon" {
                    if !attrs.iter().any(|a| a.starts_with("data-icon=")) {
                        if let Some(v) = static_attr(expr, &ctx.scope) {
                            attrs.push(format!("data-icon=\"{}\"", html_escape(&v)));
                        }
                    }
                    continue;
                }
                // `Option("value", "Label")`: value attribute, then the label.
                if name == "Option" && text_content.is_some() {
                    if !attrs.iter().any(|a| a.starts_with("value=")) {
                        if let Some(v) = text_content.take() {
                            attrs.push(format!("value=\"{}\"", html_escape(&v)));
                        }
                        text_content = resolve_text_scoped(expr, &ctx.default_messages, &ctx.scope);
                    }
                    continue;
                }
                if text_content.is_none() {
                    text_content = resolve_text_scoped(expr, &ctx.default_messages, &ctx.scope);
                }
            }
        }
    }

    // A link to the page being painted is the current one: the runtime adds
    // the same class after hydration, so the static paint must agree with it.
    if current_link {
        for a in attrs.iter_mut() {
            if a.starts_with("class=\"") {
                a.insert_str(a.len() - 1, " active");
            }
        }
    }

    // Reserve space and keep offscreen images off the critical path. Without
    // dimensions the browser allocates none, and the page shifts when the image
    // arrives.
    if name == "Image" {
        if !ui
            .args
            .iter()
            .any(|a| matches!(a, Arg::Named(k, _) if k == "decoding"))
        {
            attrs.push("decoding=\"async\"".to_string());
        }
        // The page's first image is the one its largest paint waits for, so
        // it is fetched first; the rest load when they come into view.
        if !ui
            .args
            .iter()
            .any(|a| matches!(a, Arg::Named(k, _) if k == "loading"))
        {
            if ctx.images == 0 {
                attrs.push("loading=\"eager\"".to_string());
                attrs.push("fetchpriority=\"high\"".to_string());
            } else {
                attrs.push("loading=\"lazy\"".to_string());
            }
            ctx.images += 1;
        }
    }

    // Handle input type from modifiers
    for m in &ui.modifiers {
        if let Some(t) = input_type(m) {
            attrs.push(format!("type=\"{}\"", t));
        } else if m == "multiple" {
            attrs.push("multiple".to_string());
        }
    }

    // Heading tag override based on modifier
    let actual_tag =
        if name == "Tcell" && (ctx.in_thead || element_tag(name, &ui.modifiers) == "th") {
            attrs.push("scope=\"col\"".to_string());
            "th"
        } else {
            element_tag(name, &ui.modifiers)
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

    // An Input with a label, hint or error is a field: the label, the
    // control, then the hint and the error message, as the bundle builds it.
    if matches!(name, "Input" | "Select")
        && field_parts(ui, |e| static_attr(e, &ctx.scope)).is_some()
    {
        let mut parts = field_parts(ui, |e| static_attr(e, &ctx.scope)).unwrap_or_default();
        let id = format!("wf-field-{}", ctx.next_field_id());
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
            let inner = {
                ctx.indent += 1;
                let r = render_statements(&ui.children, ctx);
                ctx.indent -= 1;
                r
            };
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
        return static_field(&indent, &id, &control, &parts);
    }

    // Self-closing tags
    if matches!(actual_tag, "input" | "img" | "hr" | "br") {
        return format!("{}<{}{}>\n", indent, actual_tag, attrs_str);
    }

    // Has children?
    let has_children = !ui.children.is_empty();
    let has_text = text_content.is_some();

    let has_caption =
        name == "Table" && table_caption(ui, |e| static_attr(e, &ctx.scope)).is_some();
    if !has_children && !has_text && !has_caption {
        return format!("{}<{}{}></{}>\n", indent, actual_tag, attrs_str, actual_tag);
    }

    let mut result = format!("{}<{}{}>\n", indent, actual_tag, attrs_str);

    if name == "Table" {
        if let Some(cap) = table_caption(ui, |e| static_attr(e, &ctx.scope)) {
            result.push_str(&format!(
                "{}    <caption class=\"wf-visually-hidden\">{}</caption>\n",
                indent,
                html_escape(&cap)
            ));
        }
    }

    if let Some(text) = &text_content {
        // `Code(…, language: "wf")`: coloured, as the runtime colours it.
        let language = if name == "Code" {
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == "language" => static_attr(v, &ctx.scope),
                _ => None,
            })
        } else {
            None
        };
        let inner = match language {
            Some(lang) => crate::codegen::highlight::highlight(text, &lang),
            None => html_escape(text),
        };
        // Inline text
        if !has_children {
            return format!(
                "{}<{}{}>{}</{}>\n",
                indent, actual_tag, attrs_str, inner, actual_tag
            );
        }
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

/// A field's label, hint and error, resolved to text.
#[derive(Default)]
pub struct FieldParts {
    pub label: Option<String>,
    pub hint: Option<String>,
    /// The error argument as written: `Some(None)` when it is there but
    /// cannot be known at build time (the client will fill it in),
    /// `Some(Some(text))` when it can.
    pub error: Option<Option<String>>,
    id: String,
}

impl FieldParts {
    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }

    /// The `aria-describedby` and `aria-invalid` attributes the control
    /// carries, given its id.
    pub fn control_attrs(&self) -> String {
        let mut described = Vec::new();
        if self.hint.is_some() {
            described.push(format!("{}-hint", self.id));
        }
        if self.error.is_some() {
            described.push(format!("{}-error", self.id));
        }
        let mut out = String::new();
        if !described.is_empty() {
            out.push_str(&format!(" aria-describedby=\"{}\"", described.join(" ")));
        }
        if let Some(error) = &self.error {
            let invalid = error.as_deref().is_some_and(|e| !e.is_empty());
            out.push_str(&format!(" aria-invalid=\"{}\"", invalid));
        }
        out
    }
}

/// The field parts of an `Input`/`Select`, or `None` when it has none.
pub fn field_parts(
    ui: &UIElement,
    resolve: impl Fn(&Expr) -> Option<String>,
) -> Option<FieldParts> {
    let arg = |key: &str| {
        ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == key => Some(v),
            _ => None,
        })
    };
    let (label, hint, error) = (arg("label"), arg("hint"), arg("error"));
    if label.is_none() && hint.is_none() && error.is_none() {
        return None;
    }
    Some(FieldParts {
        label: label.and_then(&resolve),
        hint: hint.and_then(&resolve),
        error: error.map(resolve),
        id: String::new(),
    })
}

/// The static markup of a field: label, control, hint, error.
pub fn static_field(indent: &str, id: &str, control: &str, parts: &FieldParts) -> String {
    let mut out = format!("{indent}<div class=\"wf-field\">\n");
    if let Some(label) = &parts.label {
        out.push_str(&format!(
            "{indent}    <label class=\"wf-label\" for=\"{id}\">{}</label>\n",
            html_escape(label)
        ));
    }
    out.push_str(&format!("{indent}    {control}\n"));
    if let Some(hint) = &parts.hint {
        out.push_str(&format!(
            "{indent}    <p class=\"wf-field__hint\" id=\"{id}-hint\">{}</p>\n",
            html_escape(hint)
        ));
    }
    if let Some(error) = &parts.error {
        let text = error.as_deref().unwrap_or("");
        let hidden = if text.is_empty() { " hidden" } else { "" };
        out.push_str(&format!(
            "{indent}    <p class=\"wf-field__error\" id=\"{id}-error\" role=\"alert\"{hidden}>{}</p>\n",
            html_escape(text)
        ));
    }
    out.push_str(&format!("{indent}</div>\n"));
    out
}

/// `Slider`, `DatePicker` and `FileUpload`: a wrapper carrying the component
/// class, an optional `<label>`, and the input itself.
///
/// The SPA builds exactly this shape, so the static paint has to match it or
/// hydration reconciles a wrapper against a bare `<input>`.
/// The `<picture>` an `image` declaration became.
///
/// The twin of `WF.picture` in the runtime: the same element, so a page
/// that hydrates does not repaint what it was already showing.
/// A `Video` with captions, or an `Audio` with a transcript.
fn render_media(name: &str, class: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let tag = crate::codegen::builtin::builtin_to_html(name).0;
    let named = |key: &str| {
        ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == key => Some(v),
            _ => None,
        })
    };
    let mut attrs = vec![format!("class=\"{class}\"")];
    for key in ["src", "poster", "width", "height"] {
        if let Some(value) = named(key).and_then(|v| static_attr(v, &ctx.scope)) {
            attrs.push(format!("{key}=\"{}\"", html_escape(&value)));
        }
    }
    for flag in ["controls", "autoplay", "muted", "loop", "playsinline"] {
        if named(flag).is_some() || ui.modifiers.iter().any(|m| m == flag) {
            attrs.push(flag.to_string());
        }
    }
    let mut out = format!("<{tag} {}>", attrs.join(" "));
    if let Some(captions) = named("captions").and_then(|v| static_attr(v, &ctx.scope)) {
        out.push_str(&format!(
            "<track kind=\"captions\" src=\"{}\" srclang=\"{}\" default />",
            html_escape(&captions),
            // The captions are in the page's language; the browser reads
            // the document's own when this says nothing more.
            "en"
        ));
    }
    out.push_str(&format!("</{tag}>"));
    if let Some(transcript) = named("transcript").and_then(|v| static_attr(v, &ctx.scope)) {
        out.push_str(&format!(
            "<a class=\"wf-transcript\" href=\"{}\">Read the transcript</a>",
            html_escape(&transcript)
        ));
    }
    out
}

fn render_picture(class: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let named = |key: &str| {
        ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == key => Some(v),
            _ => None,
        })
    };
    let positional = ui.args.iter().find_map(|a| match a {
        Arg::Positional(v) => Some(v),
        _ => None,
    });
    let Some(asset) = positional
        .or_else(|| named("source"))
        .and_then(|v| eval(v, &ctx.scope))
    else {
        return String::new();
    };
    let field = |name: &str| match &asset {
        Static::Map(fields) => fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone()),
        _ => None,
    };
    let text = |name: &str| field(name).map(|v| v.to_text()).unwrap_or_default();
    let alt = named("alt")
        .and_then(|v| static_attr(v, &ctx.scope))
        .unwrap_or_default();
    let sizes = named("sizes").and_then(|v| static_attr(v, &ctx.scope));

    let mut img = vec![format!("class=\"{class}\"")];
    img.push(format!("src=\"{}\"", html_escape(&text("src"))));
    img.push(format!("alt=\"{}\"", html_escape(&alt)));
    for key in ["width", "height"] {
        let value = text(key);
        if !value.is_empty() {
            img.push(format!("{key}=\"{}\"", value.trim_end_matches(".0")));
        }
    }
    let srcset = text("srcset");
    if !srcset.is_empty() {
        img.push(format!("srcset=\"{}\"", html_escape(&srcset)));
    }
    if let Some(sizes) = &sizes {
        img.push(format!("sizes=\"{}\"", html_escape(sizes)));
    }
    img.push("decoding=\"async\"".to_string());
    // The page's first image is the one its largest paint waits for.
    if ctx.images == 0 {
        img.push("loading=\"eager\" fetchpriority=\"high\"".to_string());
    } else {
        img.push("loading=\"lazy\"".to_string());
    }
    ctx.images += 1;

    // What fills the box until the image lands.
    let placeholder = named("placeholder")
        .and_then(|v| match v {
            Expr::EnumCase(case) => Some(case.clone()),
            _ => None,
        })
        .or_else(|| {
            ui.modifiers
                .iter()
                .find(|m| matches!(m.as_str(), "blur" | "color" | "none"))
                .cloned()
        });
    match placeholder.as_deref() {
        Some("blur") if !text("placeholder").is_empty() => {
            img.push(format!(
                "style=\"background: url(&quot;{}&quot;) center / cover no-repeat\"",
                text("placeholder")
            ));
        }
        Some("color") if !text("color").is_empty() => {
            img.push(format!("style=\"background: {}\"", text("color")));
        }
        _ => {}
    }

    let img = format!("<img {} />", img.join(" "));
    let sources = match field("sources") {
        Some(Static::List(items)) => items,
        _ => Vec::new(),
    };
    if sources.is_empty() {
        return img;
    }
    let mut out = String::from("<picture>");
    for one in sources {
        let Static::Map(fields) = one else { continue };
        let get = |name: &str| {
            fields
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.to_text())
                .unwrap_or_default()
        };
        out.push_str(&format!(
            "<source type=\"{}\" srcset=\"{}\"{} />",
            html_escape(&get("type")),
            html_escape(&get("srcset")),
            sizes
                .as_ref()
                .map(|s| format!(" sizes=\"{}\"", html_escape(s)))
                .unwrap_or_default()
        ));
    }
    out.push_str(&img);
    out.push_str("</picture>");
    out
}

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
    let value = eval(&if_stmt.condition, &ctx.scope)?;
    if value.truthy() {
        // `if let p = post { … }`: the branch reads `p` as the value.
        let Some(name) = &if_stmt.binding else {
            return Some(render_statements(&if_stmt.then_body, ctx));
        };
        let outer = ctx.scope.clone();
        ctx.scope = outer.with(name, value);
        let out = render_statements(&if_stmt.then_body, ctx);
        ctx.scope = outer;
        return Some(out);
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
fn render_linked_item(
    class: &str,
    href: &str,
    current: bool,
    ui: &UIElement,
    ctx: &mut SsgContext,
) -> String {
    let indent = ctx.indent_str();
    let wf = ctx.wf_node_attr_inline(ui);
    let (class, aria) = if current {
        (format!("{} active", class), " aria-current=\"page\"")
    } else {
        (class.to_string(), "")
    };
    let mut out = format!(
        "{}<a class=\"{}\" href=\"{}\"{}{}>\n",
        indent,
        class,
        html_escape(href),
        aria,
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
    let class = std::iter::once(class.to_string())
        .chain(extra_classes(ui, ctx))
        .collect::<Vec<_>>()
        .join(" ");
    let class = class.as_str();
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

/// Declarations from an element's `style { }` block that the static paint
/// has to carry inline.
///
/// A literal or a token is compiled into `styles.css` under the element's
/// scoped class (see `scoped_css`), the same class the bundle adds at
/// hydration, so the static paint carries the class and no inline text. What
/// remains is a value the static pass can name but the stylesheet cannot —
/// today, nothing: a value that reads state is left to the bundle.
fn style_block_decls(ui: &UIElement) -> Vec<String> {
    use super::scoped_css::static_declaration;
    use super::style_tokens::canonical_style_prop;
    let Some(sb) = &ui.style_block else {
        return Vec::new();
    };
    sb.properties
        .iter()
        .filter(|p| static_declaration(p).is_none())
        .filter_map(|p| {
            let prop = canonical_style_prop(&p.name);
            let value = expr_to_static_string(&p.value)?;
            Some(format!("{prop}: {value}"))
        })
        .collect()
}

/// The classes an element carries beyond its base and modifier classes: the
/// one its style block's rules live under, and the ones its `class:`
/// argument names when the value is known at build time.
fn extra_classes(ui: &UIElement, ctx: &SsgContext) -> Vec<String> {
    let mut classes: Vec<String> = ui
        .style_block
        .as_ref()
        .and_then(crate::codegen::scoped_css::scoped_class)
        .into_iter()
        .collect();
    // The rules a responsive prop compiled to, carried by name.
    classes.extend(crate::codegen::scoped_css::responsive_classes(ui));
    if let Some(Arg::Named(_, value)) = ui
        .args
        .iter()
        .find(|a| matches!(a, Arg::Named(k, _) if k == "class"))
        && let Some(text) = static_attr(value, &ctx.scope)
    {
        classes.extend(text.split_whitespace().map(str::to_string));
    }
    classes
}

/// A `Table(caption: …)` argument, resolved to text by `resolve`.
fn table_caption(ui: &UIElement, resolve: impl Fn(&Expr) -> Option<String>) -> Option<String> {
    ui.args.iter().find_map(|a| match a {
        Arg::Named(k, v) if k == "caption" => resolve(v),
        _ => None,
    })
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
    // `t("key", { count: n, name: x })`: the message, its plural form picked
    // by `count`, and its `{name}` placeholders filled from the scope.
    if let Expr::FunctionCall(name, args) = expr
        && name == "t"
        && args.len() >= 2
        && let Some(Expr::StringLiteral(key)) = args.first()
        && let Some(Static::Map(params)) = eval(&args[1], scope)
    {
        return Some(crate::i18n::message(messages, key, &params));
    }
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

    fn render(src: &str) -> String {
        let program = crate::syntax::parse_source(src, "<t>")
            .map(crate::sema::lower)
            .expect("parse");
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
            "component Hero(_ title: String, tagline: String) {\n  Container { Heading(title).h1 Text(tagline) }\n}\n\
             page Home(path: \"/\") { Hero(\"Beit Qahwa\", tagline: \"Slow roasted\") }\n",
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
    fn a_link_to_the_page_being_painted_is_marked_current() {
        let html = render(
            "page Guide(path: \"/docs/guide\") { Link(\"Home\", to: \"/\") Link(\"Docs\", to: \"/docs\").prefix Link(\"Here\", to: \"/docs/guide\") }\n",
        );
        assert!(
            html.contains("href=\"/docs/guide\" aria-current=\"page\""),
            "exact link current: {html}"
        );
        assert!(
            html.contains("class=\"wf-link active\" href=\"/docs\" aria-current=\"page\"")
                || html.contains("href=\"/docs\" aria-current=\"page\""),
            "prefix link current: {html}"
        );
        assert!(
            !html.contains("href=\"/\" aria-current"),
            "home is not current: {html}"
        );
    }

    #[test]
    fn the_children_slot_renders_every_statement_of_the_callers_block() {
        let html = render(
            "component Panel(title: String) {\n  Card { Heading(title).h3 children Text(\"after\") }\n}\npage Home(path: \"/\") { Panel(title: \"Keys\") { Text(\"first slot\") Text(\"second slot\") } }\n",
        );
        let first = html.find("first slot").expect(&html);
        let second = html.find("second slot").expect(&html);
        let after = html.find("after").expect(&html);
        assert!(first < second && second < after, "slot order: {html}");
        assert!(html.contains("Keys"));
    }

    #[test]
    fn named_arguments_and_defaults_both_bind() {
        let html = render(
            "component Item(name: String, note: String = \"none\") {\n  Text(name) Text(note)\n}\npage Home(path: \"/\") { Item(name: \"Latte\") }\n",
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
            "component Inner(_ t: String) { Text(t) }\ncomponent Outer(_ t: String) { Container { Inner(t) } }\npage Home(path: \"/\") { Outer(\"nested\") }\n",
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
            "component Loop(_ t: String) { Container { Loop(t) } }\npage Home(path: \"/\") { Loop(\"x\") }\n",
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
        let program =
            crate::syntax::parse_source("page Home(path: \"/\") { Ghost }", "<t>").unwrap();
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
