//! Pages, components and the app: their states, forms, translations, and where the router sits.

use super::{
    AppDecl, Arg, BodyState, ComponentDecl, ComponentRef, Expr, JsCodegen, PageDecl, Statement,
    StatementKind, UIElement, action_names, bare, body_signals, builtin_to_html, declared_names,
    emitted, form_options, has_form, js_component, js_str, layout_arg_classes, resource_names,
    route_order, sig, validated_names,
};

impl JsCodegen {
    pub(super) fn emit_i18n_setup(&mut self) {
        if self.i18n_translations.is_empty() {
            return;
        }

        let default_locale = self
            .i18n_default_locale
            .clone()
            .unwrap_or_else(|| "en".to_string());
        let translations = self.i18n_translations.clone();

        self.emit_line("WF.i18n = WF.locales(");
        self.indent += 1;
        self.emit_line(&format!("{},", js_str(&default_locale)));
        self.emit_line("{");
        self.indent += 1;

        let mut locales: Vec<&String> = translations.keys().collect();
        locales.sort();

        for locale in &locales {
            let messages = &translations[*locale];
            self.emit_line(&format!("{}: {{", js_str(locale)));
            self.indent += 1;

            let mut keys: Vec<&String> = messages.keys().collect();
            keys.sort();

            for key in &keys {
                // Each a JSON string: a message may hold a quote, a
                // backslash or a line break, and a key may too.
                let quoted = |t: &str| serde_json::to_string(t).unwrap_or_else(|_| "\"\"".into());
                self.emit_line(&format!("{}: {},", quoted(key), quoted(&messages[*key])));
            }

            self.indent -= 1;
            self.emit_line("},");
        }

        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line(");");
        self.emit_line("");
    }

    pub(super) fn has_i18n(&self) -> bool {
        !self.i18n_translations.is_empty()
    }

    /// Declare the body's form handles. Each page and component starts with
    /// none: the last body's handle used to carry over, so a page whose form
    /// had no `bind:` compiled to a submit that read another page's handle,
    /// which its chunk does not have. A `Form` with no `bind:` in a body that
    /// has `validate` rules is given a handle of its own, so a submit with a
    /// fault stops, as the guide says — it used to run its handler anyway.
    ///
    /// Gives back the declarations, for the caller to write inside the body's
    /// function: a component's used to be written above it, so two
    /// components with a form declared one name twice in the bundle, and
    /// every copy of a component shared one form.
    pub(super) fn open_forms(&mut self, body: &[Statement]) -> Vec<String> {
        self.body.current_form = None;
        self.body.implicit_form = None;
        self.body.form_handles = crate::check::sema::types::form_names(body);
        let mut lines = Vec::new();
        for name in crate::check::sema::types::form_names(body) {
            let handle = bare(&name);
            lines.push(format!("const {handle} = WF.form({});", form_options(body)));
            self.body.current_form = Some(handle);
            self.body.refs.push(name);
        }
        if self.body.current_form.is_none() && has_form(body) && !validated_names(body).is_empty() {
            let name = "_wfForm".to_string();
            lines.push(format!("const {name} = WF.form({});", form_options(body)));
            self.body.current_form = Some(name.clone());
            self.body.implicit_form = Some(name);
        }
        lines
    }

    /// Each `state` of `body`, in the order written, and before each the
    /// `derived` its first value reads — directly or through another, and
    /// the states those read: a state starts from its value as it is
    /// defined, so what it reads is defined first. Returns those `derived`,
    /// for the body's other statements to leave out.
    pub(super) fn emit_states(
        &mut self,
        owner: &str,
        body: &[Statement],
        parent: &str,
    ) -> Vec<String> {
        let mut done: Vec<String> = Vec::new();
        let mut early: Vec<String> = Vec::new();
        for stmt in body {
            if let StatementKind::State(s) = &stmt.kind {
                self.emit_state_after_reads(owner, body, &s.name, parent, &mut done, &mut early);
            }
        }
        early
    }

    /// The state or `derived` `name` of `body`, after what it reads.
    pub(super) fn emit_state_after_reads(
        &mut self,
        owner: &str,
        body: &[Statement],
        name: &str,
        parent: &str,
        done: &mut Vec<String>,
        early: &mut Vec<String>,
    ) {
        if done.iter().any(|d| d == name) {
            return;
        }
        let Some(stmt) = body.iter().find(|s| match &s.kind {
            StatementKind::State(st) => st.name == name,
            StatementKind::Derived(d) => d.name == name,
            _ => false,
        }) else {
            return;
        };
        done.push(name.to_string());
        let value = match &stmt.kind {
            StatementKind::State(st) => &st.value,
            StatementKind::Derived(d) => &d.value,
            _ => return,
        };
        let mut reads = Vec::new();
        crate::syntax::node_id::reads(value, &mut Vec::new(), &mut reads);
        for read in reads {
            self.emit_state_after_reads(owner, body, &read, parent, done, early);
        }
        if let StatementKind::State(s) = &stmt.kind {
            let val = self.emit_expr(&s.value);
            let init = self.state_init(owner, s, &val);
            self.emit_line(&format!("const {} = {};", sig(&s.name), init));
        } else {
            early.push(name.to_string());
            self.emit_statement_dom(stmt, parent);
        }
    }

    pub(super) fn emit_component(&mut self, comp: &ComponentDecl) {
        self.body = BodyState::default();
        self.node_owner.clone_from(&comp.name);
        let params: Vec<String> = comp.props.iter().map(|p| p.name.clone()).collect();
        // Set current props so emit_expr treats them as plain variables, not signals
        self.body.current_props = params;
        self.body.own_actions = action_names(&comp.body);
        self.body.refs = crate::check::sema::types::ref_names(&comp.body);
        self.body.own_names = declared_names(&comp.body);
        self.body.body_signals = body_signals(&comp.body);
        self.emit_pending_signals(&comp.body);
        // A form handle resolves as a bare name like a ref does, but is
        // declared here, once, as a form.
        let handles = self.body.refs.clone();
        let forms = self.open_forms(&comp.body);
        self.body.validated = validated_names(&comp.body);
        self.body.current_body.clone_from(&comp.body);
        // Props are read through `_p.name`, never copied out: a caller passes
        // a value that reads state as a getter, so every read inside the
        // component — a derived, a style value, a condition — tracks the
        // caller's signal. Props used to be destructured once at the call,
        // which froze a Chip's `pressed` at whatever it was on first paint.
        // Declared defaults fill what the caller left out.
        let defaults: Vec<String> = comp
            .props
            .iter()
            .filter_map(|p| {
                p.default
                    .as_ref()
                    .map(|d| format!("{}: {}", p.name, self.emit_expr(d)))
            })
            .collect();
        // The second parameter holds the caller's slot fills, each a thunk
        // that builds its content, so `children` (and a named slot) can be
        // placed anywhere in the body — including inside a conditional or a
        // loop, whose closures see the parameter.
        let start = self.output.len();
        self.emit_line(&format!(
            "function Component_{}(_p, _slots) {{",
            js_component(&comp.name)
        ));
        self.indent += 1;
        self.emit_line("_slots = _slots || {};");
        self.emit_line(&format!(
            "_p = WF.props(_p, {{ {} }});",
            defaults.join(", ")
        ));
        for line in forms {
            self.emit_line(&line);
        }
        for name in handles {
            self.emit_line(&format!("const {} = WF.ref();", bare(&name)));
        }

        // Emit state declarations first
        self.body.resources = resource_names(&comp.body);
        let early = self.emit_states(&comp.name, &comp.body, "_frag");

        // Build DOM
        self.emit_line("const _frag = document.createDocumentFragment();");

        for stmt in &comp.body {
            if !emitted(stmt, &early) {
                self.emit_statement_dom(stmt, "_frag");
            }
        }

        self.emit_held_elements(start);
        self.emit_line("return _frag;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.body.current_props.clear();
    }

    pub(super) fn emit_page(&mut self, page: &PageDecl) {
        self.body = BodyState::default();
        self.node_owner.clone_from(&page.name);
        let start = self.output.len();
        self.emit_line(&format!("function Page_{}(params) {{", page.name));
        self.indent += 1;
        self.body.page_params = page.params.iter().map(|p| p.name.clone()).collect();
        self.body.own_actions = action_names(&page.body);
        self.body.refs = crate::check::sema::types::ref_names(&page.body);
        self.body.own_names = declared_names(&page.body);
        self.body.body_signals = body_signals(&page.body);
        self.emit_pending_signals(&page.body);
        let handles = self.body.refs.clone();
        for line in self.open_forms(&page.body) {
            self.emit_line(&line);
        }
        self.body.validated = validated_names(&page.body);
        self.body.current_body.clone_from(&page.body);
        for name in handles {
            self.emit_line(&format!("const {} = WF.ref();", bare(&name)));
        }

        // Emit state declarations
        self.body.resources = resource_names(&page.body);
        let early = self.emit_states(&page.name, &page.body, "_root");

        // Build DOM
        self.emit_line("const _root = document.createDocumentFragment();");

        for stmt in &page.body {
            if !emitted(stmt, &early) {
                self.emit_statement_dom(stmt, "_root");
            }
        }

        // The page's own head tags, last: a tag may read any `derived` of the
        // page, and those are declared in the body's order as it is built.
        // Emitted first, `meta(content: post?.title)` read `_post` before
        // its `const` and threw, and the page drew nothing.
        if !page.head.is_empty() {
            let tags: Vec<String> = page
                .head
                .iter()
                .map(|t| {
                    let attrs: Vec<String> = t
                        .attrs
                        .iter()
                        .map(|(k, v)| {
                            let value = self.emit_expr(v);
                            let value = if self.is_reactive(&value) {
                                format!("() => {value}")
                            } else {
                                value
                            };
                            format!("\"{k}\": {value}")
                        })
                        .collect();
                    format!("[\"{}\", {{ {} }}]", t.tag, attrs.join(", "))
                })
                .collect();
            self.emit_line(&format!("WF.head([{}]);", tags.join(", ")));
        }

        self.emit_held_elements(start);
        self.emit_line("return _root;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.body.page_params.clear();
    }

    pub(super) fn emit_app(&mut self, app: &AppDecl) {
        self.body = BodyState::default();
        self.node_owner = "App".to_string();
        let start = self.output.len();
        self.emit_line("(function() {");
        self.indent += 1;
        if self.ssg_mode {
            // A pre-rendered page is drawn again beside itself, then taken
            // over in place (runtime/modules/hydrate.js).
            self.emit_line("const _app = WF.hydrating(document.getElementById('app'));");
        } else {
            self.emit_line("const _app = document.getElementById('app');");
            self.emit_line("_app.innerHTML = '';");
        }

        // The app's own state, declared before anything reads it — as a
        // page's is. An `app { state chosen = "en" … }` compiled every read
        // of `chosen` and never the `const` behind it, so the site threw on
        // load.
        self.body.own_names = declared_names(&app.body);
        self.body.body_signals = body_signals(&app.body);
        self.body.own_actions = action_names(&app.body);
        self.body.resources = resource_names(&app.body);
        for stmt in &app.body {
            if let StatementKind::State(s) = &stmt.kind {
                let val = self.emit_expr(&s.value);
                let init = self.state_init("App", s, &val);
                self.emit_line(&format!("const {} = {};", sig(&s.name), init));
            }
        }

        // The routes: the Router's own `Route` children when it lists any,
        // else every page's declared path — pages own their routes.
        let router_routes = Self::find_router_routes(&app.body);
        let has_router = Self::has_router(&app.body);

        // Recursively emit the app tree, replacing the Router with the route setup
        self.emit_app_tree(&app.body, "_app");

        if has_router {
            let mut routes: Vec<(String, String)> = Vec::new();
            for route in &router_routes {
                let mut path = String::new();
                let mut page_name = String::new();
                for arg in &route.args {
                    if let Arg::Named(name, expr) = arg {
                        if name == "path" {
                            path = self.emit_expr(expr);
                        } else if name == "page"
                            && let Expr::Identifier(id) = expr
                        {
                            page_name.clone_from(id);
                        }
                    }
                }
                routes.push((path.trim_matches('"').to_string(), page_name));
            }
            if routes.is_empty() {
                routes.clone_from(&self.page_paths);
            }
            // The table is matched top down, so the more specific route
            // comes first; the rest is a fixed order, so a site builds the
            // same however its pages and routes are laid out.
            routes.sort_by(|(a, _), (b, _)| route_order(a).cmp(&route_order(b)).then(a.cmp(b)));
            // Emit route definitions
            self.emit_line("const _routes = [");
            self.indent += 1;
            for (path, page_name) in &routes {
                let entry = self.route_entry(path, page_name);
                self.emit_line(&entry);
            }
            self.indent -= 1;
            self.emit_line("];");
            // `Router(transition: .fade, duration: "200ms")`: the pages
            // cross-fade or slide on a route change.
            let options = Self::find_router(&app.body)
                .map(|router| {
                    let mut parts = Vec::new();
                    for arg in &router.args {
                        if let Arg::Named(k, v) = arg
                            && matches!(k.as_str(), "transition" | "duration")
                        {
                            let value = match v {
                                Expr::EnumCase(c) | Expr::StringLiteral(c) => js_str(c),
                                other => self.emit_expr(other),
                            };
                            parts.push(format!("{k}: {value}"));
                        }
                    }
                    parts
                })
                .unwrap_or_default();
            let options: Vec<String> = options
                .into_iter()
                .chain(self.router_lang_option())
                .collect();
            self.emit_held_elements(start);
            if options.is_empty() {
                self.emit_line("WF.router(_routes, _routerEl);");
            } else {
                self.emit_line(&format!(
                    "WF.router(_routes, _routerEl, {{ {} }});",
                    options.join(", ")
                ));
            }
        }

        self.indent -= 1;
        self.emit_line("})();");
    }

    /// Recursively emit App body statements, replacing Router with a router element
    pub(super) fn emit_app_tree(&mut self, stmts: &[Statement], parent: &str) {
        for stmt in stmts {
            if let StatementKind::UIElement(ui) = &stmt.kind {
                let name = match &ui.component {
                    ComponentRef::BuiltIn(n) => n.as_str(),
                    _ => "",
                };

                if name == "Router" {
                    // The router's container holds whatever the current route
                    // paints, which is the page's main content — so it is the
                    // `<main>` landmark, and the target the skip link jumps to.
                    self.emit_line("const _routerEl = document.createElement('main');");
                    self.emit_line("_routerEl.id = 'wf-main';");
                    self.emit_line("_routerEl.style.flex = '1';");
                    self.emit_line(&format!("{parent}.appendChild(_routerEl);"));
                    continue;
                }

                if Self::stmt_contains_router(stmt) {
                    // This element wraps the Router — emit it as a container and recurse
                    let var = self.fresh_var();
                    let comp_name = Self::component_name_str(&ui.component);
                    let (tag, class) = builtin_to_html(&comp_name);
                    let mut classes: Vec<String> = vec![class.to_string()];
                    for m in &ui.modifiers {
                        classes.push(format!("{class}--{m}"));
                    }
                    classes.extend(layout_arg_classes(&ui.args));
                    classes.extend(crate::project::scoped_css::responsive_classes(ui));
                    self.emit_line(&format!(
                        "const {} = WF.el(\"{}\", {{ className: \"{}\"{} }});",
                        var,
                        tag,
                        classes.join(" "),
                        self.wf_node_inline(ui)
                    ));
                    self.emit_style_and_transition(&var, ui);
                    self.emit_motion(&var, ui);
                    self.emit_host(&var, ui);
                    self.emit_line(&format!("{parent}.appendChild({var});"));
                    // Recurse into children
                    self.emit_app_tree(&ui.children, &var);
                    continue;
                }
            }

            // Non-router statement — emit normally
            self.emit_statement_dom(stmt, parent);
        }
    }

    /// Check if a statement is or contains a Router
    pub(super) fn stmt_contains_router(stmt: &Statement) -> bool {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                return true;
            }
            for child in &ui.children {
                if Self::stmt_contains_router(child) {
                    return true;
                }
            }
        }
        false
    }

    /// Find Route declarations recursively inside the App body
    /// Whether a `Router` element appears anywhere in `body`.
    pub(super) fn has_router(body: &[Statement]) -> bool {
        body.iter().any(|stmt| match &stmt.kind {
            StatementKind::UIElement(ui) => {
                matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router")
                    || Self::has_router(&ui.children)
            }
            _ => false,
        })
    }

    /// The `Router` element, at any depth of the app body.
    pub(super) fn find_router(body: &[Statement]) -> Option<&UIElement> {
        for stmt in body {
            if let StatementKind::UIElement(ui) = &stmt.kind {
                if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                    return Some(ui);
                }
                if let Some(found) = Self::find_router(&ui.children) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub(super) fn find_router_routes(body: &[Statement]) -> Vec<&UIElement> {
        for stmt in body {
            if let StatementKind::UIElement(ui) = &stmt.kind {
                if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                    return ui.children.iter().filter_map(|s| {
                        if let StatementKind::UIElement(child_ui) = &s.kind
                            && matches!(&child_ui.component, ComponentRef::BuiltIn(n) if n == "Route") {
                                return Some(child_ui);
                            }
                        None
                    }).collect();
                }
                let nested = Self::find_router_routes(&ui.children);
                if !nested.is_empty() {
                    return nested;
                }
            }
        }
        Vec::new()
    }

    pub(super) fn component_name_str(comp: &ComponentRef) -> String {
        match comp {
            ComponentRef::BuiltIn(n) => n.clone(),
            ComponentRef::UserDefined(n) => n.clone(),
            ComponentRef::SubComponent(p, s) => format!("{p}.{s}"),
        }
    }
}
