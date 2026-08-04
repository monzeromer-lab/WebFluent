use crate::codegen::builtin::{
    builtin_to_html, heading_tag, implicit_role, input_type, is_void, landmark_label,
    modifier_to_class,
};
use crate::codegen::node_id::NodeMap;
use crate::parser::ast::*;
use crate::runtime;
use std::collections::HashMap;

/// JavaScript code generator — compiles the AST to a JS bundle with reactivity and routing.
pub struct JsCodegen {
    output: String,
    indent: usize,
    /// Track user-defined component names so we can reference them
    components: Vec<String>,
    /// Track store names
    stores: Vec<String>,
    /// Track current component/page prop names (not signals)
    current_props: Vec<String>,
    /// Names bound by an enclosing `for` loop, innermost last.
    ///
    /// `WF.listRender` calls the body with the item as a plain parameter, so a
    /// reference to it must stay plain. Treating it as state instead emitted
    /// `_item()` against a binding named `item` — a `ReferenceError` the moment
    /// a non-empty list rendered.
    loop_bindings: Vec<String>,
    /// i18n: default locale and translations (locale -> key -> value)
    i18n_default_locale: Option<String>,
    i18n_translations: HashMap<String, HashMap<String, String>>,
    /// SSG mode: emit hydration instead of full mount
    ssg_mode: bool,
    /// Base path for deployment (e.g., "/WebFluent")
    base_path: String,
    /// Studio mode: stamp `data-wf-node` ids on rendered elements. Off for
    /// export/release builds, which must contain no debug attributes.
    studio: bool,
    /// Deterministic node ids keyed by element span (empty unless in studio mode).
    node_ids: NodeMap,
}

impl Default for JsCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl JsCodegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            components: Vec::new(),
            stores: Vec::new(),
            current_props: Vec::new(),
            loop_bindings: Vec::new(),
            i18n_default_locale: None,
            i18n_translations: HashMap::new(),
            ssg_mode: false,
            base_path: String::new(),
            studio: false,
            node_ids: NodeMap::default(),
        }
    }

    pub fn set_i18n(
        &mut self,
        default_locale: String,
        translations: HashMap<String, HashMap<String, String>>,
    ) {
        self.i18n_default_locale = Some(default_locale);
        self.i18n_translations = translations;
    }

    pub fn set_ssg(&mut self, enabled: bool) {
        self.ssg_mode = enabled;
    }

    pub fn set_base_path(&mut self, path: String) {
        self.base_path = path;
    }

    /// Enable studio mode and supply the node-identity map. When enabled, each
    /// element's root gets `data-wf-node="<id>"`.
    pub fn set_studio(&mut self, node_ids: NodeMap) {
        self.studio = true;
        self.node_ids = node_ids;
    }

    /// A `WF.signal(value)` initializer, wrapped so `WF.__debug.state()` can see it
    /// by name in studio mode. Export builds emit the plain signal (unchanged).
    fn signal_init(&self, name: &str, value: &str) -> String {
        if self.studio {
            format!("WF.__reg(\"{}\", WF.signal({}))", name, value)
        } else {
            format!("WF.signal({})", value)
        }
    }

    /// A `WF.computed(() => body)` initializer, registered in studio mode.
    fn computed_init(&self, name: &str, body: &str) -> String {
        if self.studio {
            format!("WF.__reg(\"{}\", WF.computed(() => {}))", name, body)
        } else {
            format!("WF.computed(() => {})", body)
        }
    }

    /// The `data-wf-node` attrs-object entry for this element, or `None` when not
    /// in studio mode or the node has no id. Formatted as a JS object property
    /// (`"data-wf-node": "<id>"`) ready to push into an element's attrs list.
    fn wf_node_entry(&self, ui: &UIElement) -> Option<String> {
        if !self.studio {
            return None;
        }
        self.node_ids
            .id_for(ui.span)
            .map(|id| format!("\"data-wf-node\": \"{}\"", id))
    }

    /// Leading-comma form (`, "data-wf-node": "<id>"`) for splicing into an
    /// existing JS attrs object literal built inline by a special emitter; empty
    /// when not in studio mode / the node has no id.
    fn wf_node_inline(&self, ui: &UIElement) -> String {
        self.wf_node_entry(ui)
            .map(|e| format!(", {}", e))
            .unwrap_or_default()
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // Emit runtime
        self.emit_line(runtime::RUNTIME_JS);
        self.emit_line("");

        // First pass: collect component and store names
        for decl in &program.declarations {
            match decl {
                Declaration::Component(c) => self.components.push(c.name.clone()),
                Declaration::Store(s) => self.stores.push(s.name.clone()),
                _ => {}
            }
        }

        // Emit base path and SSG mode flag
        if !self.base_path.is_empty() {
            self.emit_line(&format!("WF.setBasePath(\"{}\");", self.base_path));
        }
        if self.ssg_mode {
            self.emit_line("WF.setSsgMode(true);");
        }

        // Emit i18n setup if configured
        self.emit_i18n_setup();

        // Emit stores
        for decl in &program.declarations {
            if let Declaration::Store(s) = decl {
                self.emit_store(s);
            }
        }

        // Emit components
        for decl in &program.declarations {
            if let Declaration::Component(c) = decl {
                self.emit_component(c);
            }
        }

        // Emit pages
        for decl in &program.declarations {
            if let Declaration::Page(p) = decl {
                self.emit_page(p);
            }
        }

        // Emit app (router setup)
        for decl in &program.declarations {
            if let Declaration::App(a) = decl {
                self.emit_app(a);
            }
        }

        // If no App declaration, auto-mount first page
        let has_app = program
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::App(_)));
        if !has_app {
            let pages: Vec<&PageDecl> = program
                .declarations
                .iter()
                .filter_map(|d| {
                    if let Declaration::Page(p) = d {
                        Some(p)
                    } else {
                        None
                    }
                })
                .collect();

            if pages.len() == 1 {
                let mount_fn = if self.ssg_mode { "hydrate" } else { "mount" };
                // Into the main landmark, not the bare container: a page with no
                // router still needs one, and the skip link still has to land.
                self.emit_line(&format!(
                    "WF.{}(() => Page_{}({{}}), WF.mainOf(document.getElementById('app')));",
                    mount_fn, pages[0].name
                ));
            } else if !pages.is_empty() {
                // Auto-create router from page paths
                self.emit_line("(function() {");
                self.indent += 1;
                self.emit_line("const routes = [");
                self.indent += 1;
                for p in &pages {
                    self.emit_line(&format!(
                        "{{ path: \"{}\", render: (params) => Page_{}(params) }},",
                        p.path, p.name
                    ));
                }
                self.indent -= 1;
                self.emit_line("];");
                self.emit_line("const container = WF.mainOf(document.getElementById('app'));");
                self.emit_line("WF.createRouter(routes, container);");
                self.indent -= 1;
                self.emit_line("})();");
            }
        }

        self.output.clone()
    }

    // ─── Store ───────────────────────────────────────

    fn emit_store(&mut self, store: &StoreDecl) {
        self.emit_line(&format!("const {} = WF.createStore({{", store.name));
        self.indent += 1;

        // Collect store state names for context
        let store_state_names: Vec<String> = store
            .body
            .iter()
            .filter_map(|s| {
                if let StatementKind::State(st) = &s.kind {
                    Some(st.name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Collect state
        let states: Vec<&StateDecl> = store
            .body
            .iter()
            .filter_map(|s| {
                if let StatementKind::State(st) = &s.kind {
                    Some(st)
                } else {
                    None
                }
            })
            .collect();

        if !states.is_empty() {
            self.emit_line("state: {");
            self.indent += 1;
            for s in &states {
                let val = self.emit_store_expr(&s.value, &store_state_names);
                self.emit_line(&format!("{}: {},", s.name, val));
            }
            self.indent -= 1;
            self.emit_line("},");
        }

        // Collect derived
        let derived: Vec<&DerivedDecl> = store
            .body
            .iter()
            .filter_map(|s| {
                if let StatementKind::Derived(d) = &s.kind {
                    Some(d)
                } else {
                    None
                }
            })
            .collect();

        if !derived.is_empty() {
            self.emit_line("derived: {");
            self.indent += 1;
            for d in &derived {
                let val = self.emit_store_expr(&d.value, &store_state_names);
                self.emit_line(&format!("{}: (store) => {},", d.name, val));
            }
            self.indent -= 1;
            self.emit_line("},");
        }

        // Collect actions
        let actions: Vec<&ActionDecl> = store
            .body
            .iter()
            .filter_map(|s| {
                if let StatementKind::Action(a) = &s.kind {
                    Some(a)
                } else {
                    None
                }
            })
            .collect();

        if !actions.is_empty() {
            self.emit_line("actions: {");
            self.indent += 1;
            for a in &actions {
                let params: Vec<String> = a.params.iter().map(|p| p.name.clone()).collect();
                self.emit_line(&format!(
                    "{}: (store{}) => {{",
                    a.name,
                    if params.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", params.join(", "))
                    }
                ));
                self.indent += 1;
                for stmt in &a.body {
                    self.emit_store_statement(stmt, &store_state_names, &params);
                }
                self.indent -= 1;
                self.emit_line("},");
            }
            self.indent -= 1;
            self.emit_line("},");
        }

        self.indent -= 1;
        self.emit_line("});");
        self.emit_line("");
    }

    /// Emit expression inside a store context — identifiers that are store state
    /// are accessed via `store.property` instead of `_name()`.
    fn emit_store_expr(&self, expr: &Expr, store_states: &[String]) -> String {
        match expr {
            Expr::Identifier(name) => {
                if store_states.contains(name) {
                    format!("store.{}", name)
                } else {
                    name.to_string()
                }
            }
            Expr::PropertyAccess(base, prop) => {
                let base_str = self.emit_store_expr(base, store_states);
                format!("{}.{}", base_str, prop)
            }
            Expr::IndexAccess(base, index) => {
                let base_str = self.emit_store_expr(base, store_states);
                let idx_str = self.emit_store_expr(index, store_states);
                format!("{}[{}]", base_str, idx_str)
            }
            Expr::BinaryOp(left, op, right) => {
                let l = self.emit_store_expr(left, store_states);
                let r = self.emit_store_expr(right, store_states);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq => "===",
                    BinOp::Neq => "!==",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::Lte => "<=",
                    BinOp::Gte => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::UnaryOp(op, e) => {
                let e_str = self.emit_store_expr(e, store_states);
                match op {
                    UnaryOp::Not => format!("!{}", e_str),
                    UnaryOp::Neg => format!("-{}", e_str),
                }
            }
            Expr::MethodCall(obj, method, args) => {
                let obj_str = self.emit_store_expr(obj, store_states);
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_store_expr(a, store_states))
                    .collect();
                match method.as_str() {
                    "push" => format!("{}.push({})", obj_str, args_str.join(", ")),
                    "filter" => format!("{}.filter({})", obj_str, args_str.join(", ")),
                    "map" => format!("{}.map({})", obj_str, args_str.join(", ")),
                    "sum" => format!("{}.reduce((a,b) => a+b, 0)", obj_str),
                    _ => format!("{}.{}({})", obj_str, method, args_str.join(", ")),
                }
            }
            Expr::Lambda(param, body) => {
                let body_str = self.emit_store_expr(body, store_states);
                format!("({} => {})", param, body_str)
            }
            Expr::ListLiteral(items) => {
                let items_str: Vec<String> = items
                    .iter()
                    .map(|i| self.emit_store_expr(i, store_states))
                    .collect();
                format!("[{}]", items_str.join(", "))
            }
            Expr::MapLiteral(entries) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.emit_store_expr(v, store_states)))
                    .collect();
                format!("{{ {} }}", entries_str.join(", "))
            }
            // For other expr types, fall back to the regular emitter
            _ => self.emit_expr(expr),
        }
    }

    // `action_params` is scope threaded through the recursion: a nested statement
    // needs to know which names are the enclosing action's parameters, even
    // though this level never reads it.
    #[allow(clippy::only_used_in_recursion)]
    fn emit_store_statement(
        &mut self,
        stmt: &Statement,
        store_states: &[String],
        action_params: &[String],
    ) {
        match &stmt.kind {
            StatementKind::Assignment(a) => {
                let value = self.emit_store_expr(&a.value, store_states);
                if let Expr::Identifier(name) = &a.target {
                    if store_states.contains(name) {
                        self.emit_line(&format!("store.{} = {};", name, value));
                    } else {
                        self.emit_line(&format!("{} = {};", name, value));
                    }
                } else {
                    let target = self.emit_store_expr(&a.target, store_states);
                    self.emit_line(&format!("{} = {};", target, value));
                }
            }
            StatementKind::State(s) => {
                let val = self.emit_store_expr(&s.value, store_states);
                self.emit_line(&format!("const {} = {};", s.name, val));
            }
            StatementKind::Navigate(expr) => {
                let path = self.emit_store_expr(expr, store_states);
                self.emit_line(&format!("WF.navigate({});", path));
            }
            StatementKind::ExprStatement(expr) => {
                let val = self.emit_store_expr(expr, store_states);
                self.emit_line(&format!("{};", val));
            }
            StatementKind::If(if_stmt) => {
                let cond = self.emit_store_expr(&if_stmt.condition, store_states);
                self.emit_line(&format!("if ({}) {{", cond));
                self.indent += 1;
                for s in &if_stmt.then_body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                if let Some(else_body) = &if_stmt.else_body {
                    self.emit_line("} else {");
                    self.indent += 1;
                    for s in else_body {
                        self.emit_store_statement(s, store_states, action_params);
                    }
                    self.indent -= 1;
                }
                self.emit_line("}");
            }
            _ => self.emit_statement(stmt),
        }
    }

    // ─── i18n ────────────────────────────────────────

    fn emit_i18n_setup(&mut self) {
        if self.i18n_translations.is_empty() {
            return;
        }

        let default_locale = self
            .i18n_default_locale
            .clone()
            .unwrap_or_else(|| "en".to_string());
        let translations = self.i18n_translations.clone();

        self.emit_line("WF.i18n = WF.createI18n(");
        self.indent += 1;
        self.emit_line(&format!("\"{}\",", default_locale));
        self.emit_line("{");
        self.indent += 1;

        let mut locales: Vec<&String> = translations.keys().collect();
        locales.sort();

        for locale in &locales {
            let messages = &translations[*locale];
            self.emit_line(&format!("\"{}\": {{", locale));
            self.indent += 1;

            let mut keys: Vec<&String> = messages.keys().collect();
            keys.sort();

            for key in &keys {
                let value = &messages[*key];
                let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
                self.emit_line(&format!("\"{}\": \"{}\",", key, escaped));
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

    fn has_i18n(&self) -> bool {
        !self.i18n_translations.is_empty()
    }

    // ─── Component ───────────────────────────────────

    fn emit_component(&mut self, comp: &ComponentDecl) {
        let params: Vec<String> = comp.props.iter().map(|p| p.name.clone()).collect();
        let destructure = if params.is_empty() {
            String::new()
        } else {
            format!("{{ {} }}", params.join(", "))
        };

        // Set current props so emit_expr treats them as plain variables, not signals
        self.current_props = params.clone();

        self.emit_line(&format!(
            "function Component_{}({}) {{",
            comp.name, destructure
        ));
        self.indent += 1;

        // Emit state declarations first
        for stmt in &comp.body {
            if let StatementKind::State(s) = &stmt.kind {
                let val = self.emit_expr(&s.value);
                self.emit_line(&format!(
                    "const _{} = {};",
                    s.name,
                    self.signal_init(&s.name, &val)
                ));
            }
        }

        // Build DOM
        self.emit_line("const _frag = document.createDocumentFragment();");

        for stmt in &comp.body {
            if !matches!(&stmt.kind, StatementKind::State(_)) {
                self.emit_statement_dom(stmt, "_frag");
            }
        }

        self.emit_line("return _frag;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.current_props.clear();
    }

    // ─── Page ────────────────────────────────────────

    fn emit_page(&mut self, page: &PageDecl) {
        self.emit_line(&format!("function Page_{}(params) {{", page.name));
        self.indent += 1;

        // Emit state declarations
        for stmt in &page.body {
            if let StatementKind::State(s) = &stmt.kind {
                let val = self.emit_expr(&s.value);
                self.emit_line(&format!(
                    "const _{} = {};",
                    s.name,
                    self.signal_init(&s.name, &val)
                ));
            }
        }

        // Build DOM
        self.emit_line("const _root = document.createDocumentFragment();");

        for stmt in &page.body {
            if !matches!(&stmt.kind, StatementKind::State(_)) {
                self.emit_statement_dom(stmt, "_root");
            }
        }

        self.emit_line("return _root;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    // ─── App ─────────────────────────────────────────

    fn emit_app(&mut self, app: &AppDecl) {
        self.emit_line("(function() {");
        self.indent += 1;
        self.emit_line("const _app = document.getElementById('app');");
        self.emit_line("_app.innerHTML = '';");

        // Find Route declarations (may be nested at any depth)
        let router_routes = Self::find_router_routes(&app.body);
        let has_router = !router_routes.is_empty();

        // Recursively emit the app tree, replacing the Router with the route setup
        self.emit_app_tree(&app.body, "_app", has_router);

        if has_router {
            // Emit route definitions
            self.emit_line("const _routes = [");
            self.indent += 1;
            for route in &router_routes {
                let mut path = String::new();
                let mut page_name = String::new();
                for arg in &route.args {
                    if let Arg::Named(name, expr) = arg {
                        if name == "path" {
                            path = self.emit_expr(expr);
                        } else if name == "page" {
                            if let Expr::Identifier(id) = expr {
                                page_name = id.clone();
                            }
                        }
                    }
                }
                let clean_path = path.trim_matches('"');
                self.emit_line(&format!(
                    "{{ path: \"{}\", render: (params) => Page_{}(params) }},",
                    clean_path, page_name
                ));
            }
            self.indent -= 1;
            self.emit_line("];");
            self.emit_line("WF.createRouter(_routes, _routerEl);");
        }

        self.indent -= 1;
        self.emit_line("})();");
    }

    /// Recursively emit App body statements, replacing Router with a router element
    // `has_router` is the same: it tells a deeper level whether a Router has
    // already been placed, which only matters below this one.
    #[allow(clippy::only_used_in_recursion)]
    fn emit_app_tree(&mut self, stmts: &[Statement], parent: &str, has_router: bool) {
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
                    self.emit_line(&format!("{}.appendChild(_routerEl);", parent));
                    continue;
                }

                if Self::stmt_contains_router(stmt) {
                    // This element wraps the Router — emit it as a container and recurse
                    let var = self.fresh_var();
                    let comp_name = Self::component_name_str(&ui.component);
                    let (tag, class) = builtin_to_html(&comp_name);
                    let mut classes: Vec<String> = vec![class.to_string()];
                    for m in &ui.modifiers {
                        classes.push(format!("{}--{}", class, m));
                    }
                    for arg in &ui.args {
                        if let Arg::Named(k, v) = arg {
                            if k == "gap" {
                                if let Expr::Identifier(g) = v {
                                    classes.push(format!("wf-gap--{}", g));
                                }
                            }
                        }
                    }
                    self.emit_line(&format!(
                        "const {} = WF.h(\"{}\", {{ className: \"{}\"{} }});",
                        var,
                        tag,
                        classes.join(" "),
                        self.wf_node_inline(ui)
                    ));
                    // Apply style block if present
                    if let Some(style) = &ui.style_block {
                        for prop in &style.properties {
                            let (css_prop, val) = self.emit_style_decl(prop);
                            self.emit_line(&format!("{}.style.{} = {};", var, css_prop, val));
                        }
                    }
                    self.emit_line(&format!("{}.appendChild({});", parent, var));
                    // Recurse into children
                    self.emit_app_tree(&ui.children, &var, has_router);
                    continue;
                }
            }

            // Non-router statement — emit normally
            self.emit_statement_dom(stmt, parent);
        }
    }

    /// Check if a statement is or contains a Router
    fn stmt_contains_router(stmt: &Statement) -> bool {
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
    fn find_router_routes(body: &[Statement]) -> Vec<&UIElement> {
        for stmt in body {
            if let StatementKind::UIElement(ui) = &stmt.kind {
                if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                    return ui.children.iter().filter_map(|s| {
                        if let StatementKind::UIElement(child_ui) = &s.kind {
                            if matches!(&child_ui.component, ComponentRef::BuiltIn(n) if n == "Route") {
                                return Some(child_ui);
                            }
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

    fn component_name_str(comp: &ComponentRef) -> String {
        match comp {
            ComponentRef::BuiltIn(n) => n.clone(),
            ComponentRef::UserDefined(n) => n.clone(),
            ComponentRef::SubComponent(p, s) => format!("{}.{}", p, s),
        }
    }

    // ─── DOM-building statement emitter ──────────────

    fn emit_statement_dom(&mut self, stmt: &Statement, parent: &str) {
        match &stmt.kind {
            StatementKind::UIElement(ui) => self.emit_ui_element(ui, parent),
            StatementKind::If(if_stmt) => self.emit_if_dom(if_stmt, parent),
            StatementKind::For(for_stmt) => self.emit_for_dom(for_stmt, parent),
            StatementKind::Show(show_stmt) => self.emit_show_dom(show_stmt, parent),
            StatementKind::Fetch(fetch) => self.emit_fetch_dom(fetch, parent),
            StatementKind::Use(_) => {} // Stores are global, no DOM output
            StatementKind::State(_) => {} // Already handled
            StatementKind::Derived(d) => {
                let val = self.emit_expr(&d.value);
                self.emit_line(&format!(
                    "const _{} = {};",
                    d.name,
                    self.computed_init(&d.name, &val)
                ));
            }
            StatementKind::Effect(e) => {
                self.emit_line("WF.effect(() => {");
                self.indent += 1;
                for s in &e.body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                self.emit_line("});");
            }
            StatementKind::Action(a) => {
                let params: Vec<String> = a.params.iter().map(|p| p.name.clone()).collect();
                self.emit_line(&format!("function {}({}) {{", a.name, params.join(", ")));
                self.indent += 1;
                for s in &a.body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }
            StatementKind::EventHandler(handler) => {
                // Standalone event handlers at component level — unusual but handle it
                self.emit_line(&format!("// event handler: on:{}", handler.event));
            }
            _ => {
                self.emit_statement(stmt);
            }
        }
    }

    // ─── UI Element ──────────────────────────────────

    fn emit_ui_element(&mut self, ui: &UIElement, parent: &str) {
        let var = self.fresh_var();

        match &ui.component {
            ComponentRef::BuiltIn(name) => {
                match name.as_str() {
                    "Children" => {
                        self.emit_line(&format!(
                            "if (typeof _children === 'function') {}.appendChild(_children());",
                            parent
                        ));
                        return;
                    }
                    "_StyleBlock" => return, // Style blocks handled via attrs
                    _ => {}
                }

                let (base_tag, class) = builtin_to_html(name);
                // A heading's level is part of the document outline, so it has to
                // reach the tag; a class cannot express it.
                let tag = if name == "Heading" {
                    heading_tag(&ui.modifiers)
                } else {
                    base_tag
                };

                // Collect attributes
                let mut attrs = Vec::new();
                let mut inner_text: Option<String> = None;

                // Build class string from base class + modifiers
                let mut classes = vec![class.to_string()];
                for m in &ui.modifiers {
                    let mod_class = modifier_to_class(class, m);
                    classes.push(mod_class);
                }

                // Process named args as HTML attributes
                for arg in &ui.args {
                    match arg {
                        Arg::Named(key, val) => {
                            match key.as_str() {
                                "bind" => {
                                    if let Expr::Identifier(state_name) = val {
                                        attrs.push(format!("value: () => _{}()", state_name));
                                        attrs.push(format!(
                                            "\"on:input\": (e) => _{}.set(e.target.value)",
                                            state_name
                                        ));
                                    }
                                }
                                "checked" => {
                                    if let Expr::Identifier(state_name) = val {
                                        attrs.push(format!("checked: () => _{}()", state_name));
                                    } else {
                                        let v = self.emit_expr(val);
                                        attrs.push(format!("checked: {}", v));
                                    }
                                }
                                "visible" => {
                                    if let Expr::Identifier(state_name) = val {
                                        // Handle Modal/Dialog visibility
                                        attrs.push(format!(
                                            "className: () => _{}() ? '{} open' : '{}'",
                                            state_name,
                                            classes.join(" "),
                                            classes.join(" ")
                                        ));
                                    }
                                }
                                "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max"
                                | "step" | "accept" | "label" | "required" | "disabled"
                                | "controls" | "autoplay" | "role" | "width" | "height"
                                | "loading" | "decoding" | "fetchpriority" => {
                                    let v = self.emit_expr(val);
                                    attrs.push(format!("{}: {}", key, v));
                                }
                                "to" => {
                                    let v = self.emit_expr(val);
                                    if self.ssg_mode {
                                        // SSG: plain links with base path prepended
                                        attrs.push(format!("href: WF._basePath + {}", v));
                                    } else {
                                        attrs.push(format!("href: {}", v));
                                        attrs.push(format!(
                                            "\"on:click\": (e) => {{ e.preventDefault(); WF.navigate({}); }}",
                                            v
                                        ));
                                    }
                                }
                                "span" => {
                                    if let Expr::NumberLiteral(n) = val {
                                        classes.push(format!("{}--{}", class, *n as i32));
                                    }
                                }
                                "gap" | "align" | "justify" => {
                                    // Extract raw identifier name (not as signal)
                                    let v_str = match val {
                                        Expr::Identifier(id) => id.clone(),
                                        Expr::StringLiteral(s) => s.clone(),
                                        _ => self.emit_expr(val).trim_matches('"').to_string(),
                                    };
                                    match key.as_str() {
                                        "gap" => classes.push(format!("{}--gap-{}", class, v_str)),
                                        "align" => {
                                            if v_str == "center" {
                                                classes.push(format!("{}--center", class));
                                            }
                                        }
                                        "justify" => {
                                            if v_str == "between" {
                                                classes.push(format!("{}--between", class));
                                            }
                                            if v_str == "end" {
                                                classes.push(format!("{}--end", class));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                "columns" => {
                                    if let Expr::NumberLiteral(n) = val {
                                        attrs.push(format!(
                                            "style: {{ gridTemplateColumns: 'repeat({}, 1fr)' }}",
                                            *n as i32
                                        ));
                                    }
                                }
                                "title" => {
                                    // For Modal/Dialog title
                                    let v = self.emit_expr(val);
                                    attrs.push(format!("\"data-title\": {}", v));
                                }
                                "value" => {
                                    let v = self.emit_expr(val);
                                    attrs.push(format!("value: {}", v));
                                }
                                "icon" => {
                                    let v = self.emit_expr(val);
                                    attrs.push(format!("\"data-icon\": {}", v));
                                }
                                _ => {
                                    let v = self.emit_expr(val);
                                    attrs.push(format!("{}: {}", key, v));
                                }
                            }
                        }
                        Arg::Positional(expr) => {
                            // First positional arg is usually the content/label
                            if inner_text.is_none() {
                                inner_text = Some(self.emit_expr(expr));
                            }
                        }
                    }
                }

                // Handle input type modifiers
                for m in &ui.modifiers {
                    if let Some(t) = input_type(m) {
                        attrs.push(format!("type: \"{}\"", t));
                    } else if m == "required" {
                        attrs.push("required: true".to_string());
                    }
                }

                // Classes attr
                let class_str = classes
                    .iter()
                    .filter(|c| !c.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                if !class_str.is_empty() {
                    // Check if we already have a className (from visible binding)
                    let has_class = attrs.iter().any(|a| a.starts_with("className:"));
                    if !has_class {
                        attrs.insert(0, format!("className: \"{}\"", class_str));
                    }
                }

                // Handle events
                for handler in &ui.events {
                    let body = self.emit_event_body(&handler.body);
                    attrs.push(format!(
                        "\"on:{}\": (event) => {{ {} }}",
                        handler.event, body
                    ));
                }

                // If element has a block with just statements (Button shorthand click)
                if ui.events.is_empty() && !ui.children.is_empty() {
                    // Check if children are all action-like statements (assignments, method calls)
                    let all_actions = ui.children.iter().all(|s| {
                        matches!(
                            &s.kind,
                            StatementKind::Assignment(_)
                                | StatementKind::MethodCall(_)
                                | StatementKind::Navigate(_)
                                | StatementKind::ExprStatement(_)
                        )
                    });

                    if all_actions && matches!(name.as_str(), "Button" | "IconButton") {
                        let body = self.emit_statements_inline(&ui.children);
                        attrs.push(format!("\"on:click\": (e) => {{ {} }}", body));
                    }
                }

                // Handle form submit
                if name == "Form" {
                    if let Some(handler) = ui.events.iter().find(|h| h.event == "submit") {
                        // Already handled above
                        let _ = handler;
                    }
                    // Prevent default on forms
                    if !attrs.iter().any(|a| a.contains("on:submit")) {
                        attrs.push("\"on:submit\": (e) => e.preventDefault()".to_string());
                    }
                }

                // An image with no intrinsic size gets no space reserved, so the
                // page reflows around it when it lands — the single largest
                // source of layout shift on a content site. Lazy loading and
                // async decoding keep offscreen images off the critical path.
                if name == "Image" {
                    for (key, default) in [("loading", "lazy"), ("decoding", "async")] {
                        if !attrs.iter().any(|a| a.starts_with(&format!("{}:", key)))
                            && !ui
                                .args
                                .iter()
                                .any(|a| matches!(a, Arg::Named(k, _) if k == key))
                        {
                            attrs.push(format!("{}: \"{}\"", key, default));
                        }
                    }
                }

                // A live region has to be announced when it appears; a class
                // alone tells assistive technology nothing.
                if let Some(role) = implicit_role(name, &ui.modifiers) {
                    if !attrs.iter().any(|a| a.starts_with("role:")) {
                        attrs.push(format!("role: \"{}\"", role));
                    }
                }

                // Two `<nav>` landmarks on a page are two entries called
                // "navigation" in a screen reader's landmark list, with nothing
                // to choose between them.
                if let Some(label) = landmark_label(name) {
                    if !attrs.iter().any(|a| a.starts_with("\"aria-label\":")) {
                        attrs.push(format!("\"aria-label\": \"{}\"", label));
                    }
                }

                // A void element cannot hold text. `WF.h("hr", {}, "label")` asks
                // the runtime to append into a node that takes no children, and
                // the text is simply lost; carry it as the accessible name.
                if is_void(tag) {
                    if let Some(text) = inner_text.take() {
                        if !attrs
                            .iter()
                            .any(|a| a.starts_with("alt:") || a.starts_with("title:"))
                        {
                            attrs.push(format!("title: {}", text));
                        }
                    }
                }

                // Studio: stamp the node id on the element's root. Injecting here
                // covers the standard path and every special emitter that reuses
                // `attrs`/`attrs_str` (Modal, Switch, Checkbox, Dropdown, Spacer).
                if let Some(entry) = self.wf_node_entry(ui) {
                    attrs.push(entry);
                }

                let attrs_str = if attrs.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{ {} }}", attrs.join(", "))
                };

                // Components with a structure too complex for the generic path
                // build their own subtree and append it themselves. They still
                // have to honour the author's `style { }` and `transition { }`
                // blocks, so they fall through to the shared tail below rather
                // than returning — applying those blocks only on the generic
                // path is what used to drop styling on nineteen components.
                let built_by_special_emitter = match name.as_str() {
                    "Modal" | "Dialog" => {
                        self.emit_modal_dialog(name, &var, &attrs_str, ui, parent);
                        true
                    }
                    "Tabs" => {
                        self.emit_tabs(&var, ui, parent);
                        true
                    }
                    "Switch" => {
                        self.emit_switch(&var, &attrs, ui, parent);
                        true
                    }
                    "Checkbox" | "Radio" => {
                        self.emit_check_radio(name, &var, &attrs, ui, parent);
                        true
                    }
                    "Dropdown" | "Menu" => {
                        self.emit_dropdown_menu(name, &var, &attrs, ui, parent);
                        true
                    }
                    "Toast" => {
                        // Imperative, not DOM-based: there is no element to style.
                        if let Some(text) = &inner_text {
                            let variant =
                                ui.modifiers.first().map(|m| m.as_str()).unwrap_or("info");
                            self.emit_line(&format!("WF.showToast({}, \"{}\");", text, variant));
                        }
                        return;
                    }
                    "Spacer" => {
                        self.emit_line(&format!(
                            "const {} = WF.h(\"{}\", {});",
                            var, tag, attrs_str
                        ));
                        self.emit_line(&format!("{}.appendChild({});", parent, var));
                        true
                    }
                    "Sidebar" => {
                        self.emit_sidebar(&var, ui, parent);
                        true
                    }
                    "Breadcrumb" => {
                        self.emit_breadcrumb(&var, ui, parent);
                        true
                    }
                    "Tooltip" => {
                        self.emit_tooltip(&var, ui, parent);
                        true
                    }
                    "Avatar" => {
                        self.emit_avatar(&var, ui, parent);
                        true
                    }
                    "Skeleton" => {
                        self.emit_skeleton(&var, ui, parent);
                        true
                    }
                    "Carousel" => {
                        self.emit_carousel(&var, ui, parent);
                        true
                    }
                    "IconButton" => {
                        self.emit_icon_button(&var, ui, parent);
                        true
                    }
                    "Slider" => {
                        self.emit_slider(&var, ui, parent);
                        true
                    }
                    "DatePicker" => {
                        self.emit_datepicker(&var, ui, parent);
                        true
                    }
                    "FileUpload" => {
                        self.emit_file_upload(&var, ui, parent);
                        true
                    }
                    _ => false,
                };

                if built_by_special_emitter {
                    self.emit_style_and_transition(&var, ui);
                    return;
                }

                // Standard element creation
                let mut children_arr = Vec::new();

                // Inner text content
                if let Some(text) = &inner_text {
                    if is_reactive_expr(text) {
                        children_arr.push(format!("() => {}", text));
                    } else {
                        children_arr.push(text.clone());
                    }
                }

                if children_arr.is_empty() && ui.children.is_empty() {
                    self.emit_line(&format!(
                        "const {} = WF.h(\"{}\", {});",
                        var, tag, attrs_str
                    ));
                } else if !children_arr.is_empty() && ui.children.is_empty() {
                    self.emit_line(&format!(
                        "const {} = WF.h(\"{}\", {}, {});",
                        var,
                        tag,
                        attrs_str,
                        children_arr.join(", ")
                    ));
                } else {
                    let extra = if children_arr.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", children_arr.join(", "))
                    };
                    self.emit_line(&format!(
                        "const {} = WF.h(\"{}\", {}{});",
                        var, tag, attrs_str, extra
                    ));
                }

                // Emit children
                let is_action_shorthand = ui.events.is_empty()
                    && !ui.children.is_empty()
                    && ui.children.iter().all(|s| {
                        matches!(
                            &s.kind,
                            StatementKind::Assignment(_)
                                | StatementKind::MethodCall(_)
                                | StatementKind::Navigate(_)
                                | StatementKind::ExprStatement(_)
                        )
                    })
                    && matches!(name.as_str(), "Button" | "IconButton");

                if !is_action_shorthand {
                    for child in &ui.children {
                        self.emit_statement_dom(child, &var);
                    }
                }

                // A `Navbar.Links` group collapses behind a toggle on a narrow
                // screen. The rule that used to apply there was `display: none`
                // with nothing to bring it back, so the links were simply gone.
                // Navbars whose links are direct children wrap instead and need
                // no control, so none is emitted for them.
                if name == "Navbar" && has_subcomponent(ui, "Navbar", "Links") {
                    let toggle_var = self.fresh_var();
                    self.emit_line(&format!(
                        "const {} = WF.h(\"button\", {{ className: \"wf-navbar__toggle\",                          type: \"button\", \"aria-label\": \"Menu\", \"aria-expanded\": \"false\" }}, \"\\u2630\");",
                        toggle_var
                    ));
                    self.emit_line(&format!("{}.appendChild({});", var, toggle_var));
                    self.emit_line(&format!("WF.offCanvas({}, {}, null);", var, toggle_var));
                }

                self.emit_style_and_transition(&var, ui);

                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }

            ComponentRef::SubComponent(parent_name, sub_name) => {
                let class = format!(
                    "wf-{}__{}",
                    parent_name.to_lowercase(),
                    camel_to_kebab(sub_name)
                );
                let tag = match (parent_name.as_str(), sub_name.as_str()) {
                    (_, "Item") => "li",
                    _ => "div",
                };

                self.emit_line(&format!(
                    "const {} = WF.h(\"{}\", {{ className: \"{}\"{} }});",
                    var,
                    tag,
                    class,
                    self.wf_node_inline(ui)
                ));
                for child in &ui.children {
                    self.emit_statement_dom(child, &var);
                }
                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }

            ComponentRef::UserDefined(name) => {
                let args_obj = self.emit_component_args(&ui.args);
                self.emit_line(&format!(
                    "const {} = Component_{}({});",
                    var, name, args_obj
                ));
                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }
        }
    }

    /// The class attribute for a built-in's root: its base class plus every
    /// modifier class.
    ///
    /// The special emitters below used to hardcode the bare base class, so a
    /// `Modal(large)` or a `Sidebar(elevated)` lost its variant while the same
    /// modifier on a `Card` worked.
    fn class_attr(&self, name: &str, ui: &UIElement) -> String {
        let (_, base) = builtin_to_html(name);
        crate::codegen::builtin::class_list(base, &ui.modifiers).join(" ")
    }

    /// Apply an element's `style { }` and `transition { }` blocks to `var`.
    ///
    /// Every path that creates an element root calls this — the generic one and
    /// each special emitter — so an author's styling reaches a `Modal` or a
    /// `Slider` as surely as it reaches a `Card`.
    fn emit_style_and_transition(&mut self, var: &str, ui: &UIElement) {
        if let Some(style) = &ui.style_block {
            for prop in &style.properties {
                let (css_prop, val) = self.emit_style_decl(prop);
                self.emit_line(&format!("{}.style.{} = {};", var, css_prop, val));
            }
            // Emit @media queries as a scoped <style> element
            if !style.media_queries.is_empty() {
                let scope_var = self.fresh_var();
                let scope_class = scope_var.replace("_e", "wf-s");
                self.emit_line(&format!("{}.classList.add(\"{}\");", var, scope_class));
                let mut css = String::new();
                for mq in &style.media_queries {
                    css.push_str(&format!("{} {{ .{} {{ ", mq.condition, scope_class));
                    for prop in &mq.properties {
                        let val = self.emit_expr(&prop.value);
                        let val_str = val.trim_matches('"');
                        css.push_str(&format!("{}: {}; ", prop.name, val_str));
                    }
                    css.push_str("} } ");
                }
                self.emit_line(&format!(
                    "{{ const _s = document.createElement('style'); _s.textContent = \"{}\"; document.head.appendChild(_s); }}",
                    css.replace('"', "\\\"")
                ));
            }
        }

        if let Some(transition) = &ui.transition_block {
            let transitions: Vec<String> = transition
                .properties
                .iter()
                .map(|p| {
                    let easing = p
                        .easing
                        .as_deref()
                        .map(|e| match e {
                            "ease" => "ease",
                            "linear" => "linear",
                            "easeIn" => "ease-in",
                            "easeOut" => "ease-out",
                            "easeInOut" => "ease-in-out",
                            "spring" => "cubic-bezier(0.175, 0.885, 0.32, 1.275)",
                            "bouncy" => "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
                            "smooth" => "cubic-bezier(0.4, 0, 0.2, 1)",
                            other => other,
                        })
                        .unwrap_or("ease");
                    format!("{} {} {}", p.property, p.duration, easing)
                })
                .collect();
            self.emit_line(&format!(
                "{}.style.transition = \"{}\";",
                var,
                transitions.join(", ")
            ));
        }
    }

    // ─── Special component emitters ──────────────────

    fn emit_modal_dialog(
        &mut self,
        name: &str,
        var: &str,
        _attrs_str: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        let class = if name == "Modal" {
            "wf-modal"
        } else {
            "wf-dialog"
        };
        let title = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "title" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Check for visible binding
        let visible_state = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "visible" {
                    if let Expr::Identifier(s) = v {
                        return Some(s.clone());
                    }
                }
                None
            } else {
                None
            }
        });

        // The root carries the modifier classes too; `class` stays the bare base
        // because the sub-part classes (`__content`, `__header`) derive from it.
        let root_classes = self.class_attr(name, ui);
        // A real `<dialog>`: the browser then traps focus inside it, makes the
        // rest of the page inert, closes on Escape and exposes `aria-modal`.
        let title_id = format!("wf-dlg-{}", var.trim_start_matches("_e"));
        let labelled = if title.is_some() {
            format!(", \"aria-labelledby\": \"{}\"", title_id)
        } else {
            String::new()
        };
        self.emit_line(&format!(
            "const {} = WF.h(\"dialog\", {{ className: \"{}\"{}{} }});",
            var,
            root_classes,
            labelled,
            self.wf_node_inline(ui)
        ));

        let content_var = self.fresh_var();
        let content_class = format!("{}__content", class);
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\" }});",
            content_var, content_class
        ));

        if let Some(t) = title {
            let header_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"{}__header\" }}, WF.h(\"h3\", {{ id: \"{}\" }}, {}));",
                header_var, class, title_id, t
            ));
            self.emit_line(&format!("{}.appendChild({});", content_var, header_var));
        }

        let body_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}__body\" }});",
            body_var, class
        ));

        // Check for Modal.Footer
        let mut footer_stmts = Vec::new();
        let mut footer_wf = String::new();
        let mut body_stmts = Vec::new();
        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                if matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == name && s == "Footer")
                {
                    footer_stmts = ui_child.children.clone();
                    footer_wf = self.wf_node_inline(ui_child);
                    continue;
                }
            }
            body_stmts.push(child);
        }

        for child in &body_stmts {
            self.emit_statement_dom(child, &body_var);
        }
        self.emit_line(&format!("{}.appendChild({});", content_var, body_var));

        if !footer_stmts.is_empty() {
            let footer_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"{}__footer\"{} }});",
                footer_var, class, footer_wf
            ));
            for child in &footer_stmts {
                self.emit_statement_dom(child, &footer_var);
            }
            self.emit_line(&format!("{}.appendChild({});", content_var, footer_var));
        }

        self.emit_line(&format!("{}.appendChild({});", var, content_var));

        // Visibility binding. `WF.bindDialog` drives `showModal()`/`close()` and
        // writes the signal back when the browser closes the dialog itself — via
        // Escape or the backdrop — so the state cannot drift out of sync with
        // what is on screen.
        if let Some(state_name) = visible_state {
            self.emit_line(&format!("WF.bindDialog({}, _{});", var, state_name));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_tabs(&mut self, var: &str, ui: &UIElement, parent: &str) {
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Tabs", ui),
            self.wf_node_inline(ui)
        ));
        // `role="tablist"` and the tab/panel wiring below are what tell a screen
        // reader these buttons are tabs at all. Without them the widget is a row
        // of unrelated buttons next to unrelated divs.
        let nav_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"wf-tabs__nav\", role: \"tablist\" }});",
            nav_var
        ));
        let group = var.trim_start_matches("_e").to_string();

        // Collect tab pages
        let tab_pages: Vec<(&UIElement, usize)> = ui
            .children
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if let StatementKind::UIElement(ui_child) = &s.kind {
                    if matches!(&ui_child.component, ComponentRef::BuiltIn(n) if n == "TabPage") {
                        return Some((ui_child, i));
                    }
                }
                None
            })
            .collect();

        let active_var = self.fresh_var();
        self.emit_line(&format!("const {} = WF.signal(0);", active_var));

        // Create tab buttons
        for (i, (tab, _)) in tab_pages.iter().enumerate() {
            let label = tab
                .args
                .first()
                .map(|a| {
                    if let Arg::Positional(expr) = a {
                        self.emit_expr(expr)
                    } else {
                        format!("\"Tab {}\"", i)
                    }
                })
                .unwrap_or_else(|| format!("\"Tab {}\"", i));

            // Roving tabindex: only the selected tab is in the tab order, and
            // the arrow keys move between them (see `WF.tablist`).
            let btn_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"button\", {{ className: () => {}() === {} ? \"wf-tabs__tab active\" : \"wf-tabs__tab\",                  role: \"tab\", type: \"button\", id: \"wf-tab-{}-{}\",                  \"aria-controls\": \"wf-tabpanel-{}-{}\",                  \"aria-selected\": () => {}() === {} ? \"true\" : \"false\",                  tabindex: () => {}() === {} ? 0 : -1,                  \"on:click\": () => {}.set({}) }}, {});",
                btn_var, active_var, i, group, i, group, i, active_var, i, active_var, i, active_var, i, label
            ));
            self.emit_line(&format!("{}.appendChild({});", nav_var, btn_var));
        }

        self.emit_line(&format!("{}.appendChild({});", var, nav_var));

        // Create tab content
        for (i, (tab, _)) in tab_pages.iter().enumerate() {
            let page_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"wf-tab-page\", role: \"tabpanel\",                  id: \"wf-tabpanel-{}-{}\", \"aria-labelledby\": \"wf-tab-{}-{}\", tabindex: 0{} }});",
                page_var, group, i, group, i, self.wf_node_inline(tab)
            ));
            for child in &tab.children {
                self.emit_statement_dom(child, &page_var);
            }
            self.emit_line(&format!(
                "WF.effect(() => {{ {}.style.display = {}() === {} ? 'block' : 'none'; }});",
                page_var, active_var, i
            ));
            self.emit_line(&format!("{}.appendChild({});", var, page_var));
        }

        // Arrow keys, Home and End move between tabs, which is what the WAI-ARIA
        // pattern requires and what a keyboard user will try.
        self.emit_line(&format!("WF.tablist({}, {});", nav_var, active_var));

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_switch(&mut self, var: &str, attrs: &[String], ui: &UIElement, parent: &str) {
        let bind_var = attrs.iter().find_map(|a| {
            if a.starts_with("value: () => _") {
                Some(a.replace("value: () => _", "").replace("()", ""))
            } else {
                None
            }
        });

        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        self.emit_line(&format!(
            "const {} = WF.h(\"label\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Switch", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(state) = &bind_var {
            let input_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"input\", {{ type: \"checkbox\", role: \"switch\",                  checked: () => _{}(), \"aria-checked\": () => _{}() ? \"true\" : \"false\",                  \"on:change\": () => _{}.set(!_{}()) }});",
                input_var, state, state, state, state
            ));
            self.emit_line(&format!("{}.appendChild({});", var, input_var));
        }

        let track_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"span\", {{ className: \"wf-switch__track\" }}, WF.h(\"span\", {{ className: \"wf-switch__thumb\" }}));",
            track_var
        ));
        self.emit_line(&format!("{}.appendChild({});", var, track_var));

        if let Some(l) = label {
            self.emit_line(&format!("{}.appendChild(WF.text({}));", var, l));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_check_radio(
        &mut self,
        name: &str,
        var: &str,
        _attrs: &[String],
        ui: &UIElement,
        parent: &str,
    ) {
        let input_type = if name == "Checkbox" {
            "checkbox"
        } else {
            "radio"
        };
        let wf = self.wf_node_inline(ui);

        let bind_var = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "bind" {
                    if let Expr::Identifier(s) = v {
                        return Some(s.clone());
                    }
                }
                None
            } else {
                None
            }
        });

        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let radio_value = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "value" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        self.emit_line(&format!(
            "const {} = WF.h(\"label\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr(name, ui),
            wf
        ));

        let input_var = self.fresh_var();
        let mut input_attrs = format!("type: \"{}\"", input_type);

        if let Some(state) = &bind_var {
            if name == "Checkbox" {
                input_attrs.push_str(&format!(
                    ", checked: () => _{}(), \"on:change\": () => _{}.set(!_{}())",
                    state, state, state
                ));
            } else if let Some(val) = &radio_value {
                input_attrs.push_str(&format!(
                    ", checked: () => _{}() === {}, \"on:change\": () => _{}.set({})",
                    state, val, state, val
                ));
            }
        }

        // Handle checked prop (non-bind)
        let checked_val = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "checked" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        if bind_var.is_none() {
            if let Some(cv) = checked_val {
                if is_reactive_expr(&cv) {
                    input_attrs.push_str(&format!(", checked: () => {}", cv));
                } else {
                    input_attrs.push_str(&format!(", checked: {}", cv));
                }
            }
        }

        // Emit events from ui.events
        for handler in &ui.events {
            let body = self.emit_event_body(&handler.body);
            input_attrs.push_str(&format!(
                ", \"on:{}\": (e) => {{ {} }}",
                handler.event, body
            ));
        }

        self.emit_line(&format!(
            "const {} = WF.h(\"input\", {{ {} }});",
            input_var, input_attrs
        ));
        self.emit_line(&format!("{}.appendChild({});", var, input_var));

        if let Some(l) = label {
            self.emit_line(&format!("{}.appendChild(WF.text({}));", var, l));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_dropdown_menu(
        &mut self,
        name: &str,
        var: &str,
        _attrs: &[String],
        ui: &UIElement,
        parent: &str,
    ) {
        let class = if name == "Dropdown" {
            "wf-dropdown"
        } else {
            "wf-menu"
        };

        let label = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, v) if k == "label" || k == "trigger" => Some(self.emit_expr(v)),
                _ => None,
            })
            .unwrap_or_else(|| "\"Menu\"".to_string());

        let root_classes = self.class_attr(name, ui);
        let open_var = self.fresh_var();
        self.emit_line(&format!("const {} = WF.signal(false);", open_var));
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: () => {}() ? \"{} open\" : \"{}\"{} }});",
            var,
            open_var,
            root_classes,
            root_classes,
            self.wf_node_inline(ui)
        ));

        // `aria-expanded` is the only thing that tells a screen reader whether
        // the menu is open; the class ternary is visual only.
        let items_id = format!("wf-menu-{}", var.trim_start_matches("_e"));
        let trigger_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"button\", {{ className: \"wf-btn\", type: \"button\",              \"aria-haspopup\": \"true\", \"aria-controls\": \"{}\",              \"aria-expanded\": () => {}() ? \"true\" : \"false\",              \"on:click\": () => {}.set(!{}()) }}, {});",
            trigger_var, items_id, open_var, open_var, open_var, label
        ));
        self.emit_line(&format!("{}.appendChild({});", var, trigger_var));

        let items_var = self.fresh_var();
        let items_class = format!("{}__items", class);
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\", id: \"{}\", role: \"menu\" }});",
            items_var, items_class, items_id
        ));

        for child in &ui.children {
            self.emit_statement_dom(child, &items_var);
        }

        self.emit_line(&format!("{}.appendChild({});", var, items_var));

        // Close on click outside, and on Escape with focus returned to the
        // trigger — a keyboard user who opens a menu must be able to leave it.
        self.emit_line(&format!(
            "WF.bindPopup({}, {}, {});",
            var, trigger_var, open_var
        ));

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_sidebar(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let sidebar_id = var.trim_start_matches("_e").to_string();
        self.emit_line(&format!(
            "const {} = WF.h(\"aside\", {{ className: \"{}\", id: \"wf-sidebar-{}\"{} }});",
            var,
            self.class_attr("Sidebar", ui),
            sidebar_id,
            self.wf_node_inline(ui)
        ));

        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                match &ui_child.component {
                    ComponentRef::SubComponent(p, sub) if p == "Sidebar" => match sub.as_str() {
                        "Header" => {
                            let h_var = self.fresh_var();
                            self.emit_line(&format!(
                                    "const {} = WF.h(\"div\", {{ className: \"wf-sidebar__header\"{} }});",
                                    h_var, self.wf_node_inline(ui_child)
                                ));
                            for c in &ui_child.children {
                                self.emit_statement_dom(c, &h_var);
                            }
                            self.emit_line(&format!("{}.appendChild({});", var, h_var));
                        }
                        "Item" => {
                            let item_var = self.fresh_var();
                            let to = ui_child.args.iter().find_map(|a| {
                                if let Arg::Named(k, v) = a {
                                    if k == "to" {
                                        Some(self.emit_expr(v))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });
                            let icon = ui_child.args.iter().find_map(|a| {
                                if let Arg::Named(k, v) = a {
                                    if k == "icon" {
                                        Some(self.emit_expr(v))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });
                            if let Some(href) = to {
                                let bp = if self.base_path.is_empty() {
                                    String::new()
                                } else {
                                    "WF._basePath + ".to_string()
                                };
                                self.emit_line(&format!(
                                        "const {} = WF.h(\"a\", {{ className: \"wf-sidebar__item\", href: {} {}{} }});",
                                        item_var, bp, href, self.wf_node_inline(ui_child)
                                    ));
                            } else {
                                self.emit_line(&format!(
                                        "const {} = WF.h(\"div\", {{ className: \"wf-sidebar__item\"{} }});",
                                        item_var, self.wf_node_inline(ui_child)
                                    ));
                            }
                            if let Some(ic) = icon {
                                self.emit_line(&format!(
                                        "{}.appendChild(WF.h(\"span\", {{ className: \"wf-icon\", \"data-icon\": {} }}));",
                                        item_var, ic
                                    ));
                            }
                            for c in &ui_child.children {
                                self.emit_statement_dom(c, &item_var);
                            }
                            self.emit_line(&format!("{}.appendChild({});", var, item_var));
                        }
                        "Divider" => {
                            self.emit_line(&format!(
                                    "{}.appendChild(WF.h(\"div\", {{ className: \"wf-sidebar__divider\"{} }}));",
                                    var, self.wf_node_inline(ui_child)
                                ));
                        }
                        _ => {
                            self.emit_statement_dom(child, var);
                        }
                    },
                    _ => {
                        self.emit_statement_dom(child, var);
                    }
                }
            } else {
                self.emit_statement_dom(child, var);
            }
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));

        // Below the sidebar breakpoint the panel slides over the page, so it
        // needs something to open it — without a control the 768px rule simply
        // removed a site's navigation on a phone. The control follows the panel
        // so the panel stays the component's root; CSS fixes it to the corner on
        // a narrow screen and hides it on a wide one.
        let scrim_var = self.fresh_var();
        let toggle_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"wf-sidebar__scrim\", hidden: true }});",
            scrim_var
        ));
        self.emit_line(&format!(
            "const {} = WF.h(\"button\", {{ className: \"wf-sidebar__toggle\", type: \"button\", \"aria-label\": \"Open navigation\", \"aria-expanded\": \"false\", \"aria-controls\": \"wf-sidebar-{}\" }}, \"\\u2630\");",
            toggle_var, sidebar_id
        ));
        self.emit_line(&format!("{}.appendChild({});", parent, scrim_var));
        self.emit_line(&format!("{}.appendChild({});", parent, toggle_var));
        self.emit_line(&format!(
            "WF.offCanvas({}, {}, {});",
            var, toggle_var, scrim_var
        ));
    }

    fn emit_breadcrumb(&mut self, var: &str, ui: &UIElement, parent: &str) {
        self.emit_line(&format!(
            "const {} = WF.h(\"nav\", {{ className: \"{}\", \"aria-label\": \"breadcrumb\"{} }});",
            var,
            self.class_attr("Breadcrumb", ui),
            self.wf_node_inline(ui)
        ));

        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                if matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == "Breadcrumb" && s == "Item")
                {
                    let item_var = self.fresh_var();
                    let to = ui_child.args.iter().find_map(|a| {
                        if let Arg::Named(k, v) = a {
                            if k == "to" {
                                Some(self.emit_expr(v))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    });
                    if let Some(href) = to {
                        let bp = if self.base_path.is_empty() {
                            String::new()
                        } else {
                            "WF._basePath + ".to_string()
                        };
                        self.emit_line(&format!(
                            "const {} = WF.h(\"a\", {{ className: \"wf-breadcrumb__item\", href: {}{}{} }});",
                            item_var, bp, href, self.wf_node_inline(ui_child)
                        ));
                    } else {
                        self.emit_line(&format!(
                            "const {} = WF.h(\"span\", {{ className: \"wf-breadcrumb__item\"{} }});",
                            item_var, self.wf_node_inline(ui_child)
                        ));
                    }
                    for c in &ui_child.children {
                        self.emit_statement_dom(c, &item_var);
                    }
                    self.emit_line(&format!("{}.appendChild({});", var, item_var));
                } else {
                    self.emit_statement_dom(child, var);
                }
            } else {
                self.emit_statement_dom(child, var);
            }
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_tooltip(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let text = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "text" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "\"\"".to_string());

        // The tip is only reachable if the thing it describes points at it;
        // `role="tooltip"` alone announces nothing, because nothing refers to it.
        let tip_id = format!("wf-tip-{}", var.trim_start_matches("_e"));
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\", \"aria-describedby\": \"{}\"{} }});",
            var,
            self.class_attr("Tooltip", ui),
            tip_id,
            self.wf_node_inline(ui)
        ));

        // Render children (the trigger element)
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }

        // Add tooltip text span
        let tip_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"span\", {{ className: \"wf-tooltip__text\", role: \"tooltip\", id: \"{}\" }}, {});",
            tip_var, tip_id, text
        ));
        self.emit_line(&format!("{}.appendChild({});", var, tip_var));
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_avatar(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let src = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "src" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let alt = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "alt" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let initials = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "initials" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let mut cls = "wf-avatar".to_string();
        for m in &ui.modifiers {
            match m.as_str() {
                "small" => cls.push_str(" wf-avatar--small"),
                "large" => cls.push_str(" wf-avatar--large"),
                "primary" => cls.push_str(" wf-avatar--primary"),
                _ => {}
            }
        }

        let wf = self.wf_node_inline(ui);
        if let Some(img_src) = src {
            let alt_val = alt.unwrap_or_else(|| "\"\"".to_string());
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"{}\"{} }}, WF.h(\"img\", {{ src: {}, alt: {} }}));",
                var, cls, wf, img_src, alt_val
            ));
        } else if let Some(init) = initials {
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"{}\"{} }}, {});",
                var, cls, wf, init
            ));
        } else {
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
                var, cls, wf
            ));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_skeleton(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let height = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "height" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let width = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "width" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let size = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "size" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let is_circle = ui.modifiers.iter().any(|m| m == "circle");
        let cls = if is_circle {
            "wf-skeleton wf-skeleton--circle"
        } else {
            "wf-skeleton"
        };

        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
            var,
            cls,
            self.wf_node_inline(ui)
        ));
        if let Some(h) = &height {
            self.emit_line(&format!("{}.style.height = {};", var, h));
        }
        if let Some(w) = &width {
            self.emit_line(&format!("{}.style.width = {};", var, w));
        }
        if is_circle {
            if let Some(s) = &size {
                if height.is_none() {
                    self.emit_line(&format!("{}.style.height = {};", var, s));
                }
                if width.is_none() {
                    self.emit_line(&format!("{}.style.width = {};", var, s));
                }
            }
        }
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_carousel(&mut self, var: &str, ui: &UIElement, parent: &str) {
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Carousel", ui),
            self.wf_node_inline(ui)
        ));

        let track_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"wf-carousel__track\" }});",
            track_var
        ));

        // Collect slides
        let mut slide_count = 0;
        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                if matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == "Carousel" && s == "Slide")
                {
                    let slide_var = self.fresh_var();
                    self.emit_line(&format!(
                        "const {} = WF.h(\"div\", {{ className: \"wf-carousel__slide\"{} }});",
                        slide_var,
                        self.wf_node_inline(ui_child)
                    ));
                    for c in &ui_child.children {
                        self.emit_statement_dom(c, &slide_var);
                    }
                    self.emit_line(&format!("{}.appendChild({});", track_var, slide_var));
                    slide_count += 1;
                } else {
                    self.emit_statement_dom(child, &track_var);
                }
            } else {
                self.emit_statement_dom(child, &track_var);
            }
        }

        self.emit_line(&format!("{}.appendChild({});", var, track_var));

        // Navigation dots
        if slide_count > 1 {
            let idx_var = self.fresh_var();
            self.emit_line(&format!("const {} = WF.signal(0);", idx_var));

            let nav_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"wf-carousel__nav\" }});",
                nav_var
            ));

            for i in 0..slide_count {
                let dot_var = self.fresh_var();
                self.emit_line(&format!(
                    "const {} = WF.h(\"button\", {{ className: () => {}() === {} ? \"wf-carousel__dot active\" : \"wf-carousel__dot\", \"on:click\": () => {{ {}.set({}); {}.style.transform = `translateX(-${{{}*100}}%)`; }} }});",
                    dot_var, idx_var, i, idx_var, i, track_var, i
                ));
                self.emit_line(&format!("{}.appendChild({});", nav_var, dot_var));
            }
            self.emit_line(&format!("{}.appendChild({});", var, nav_var));

            // Autoplay
            let autoplay = ui.args.iter().any(|a| {
                matches!(a, Arg::Named(k, v) if k == "autoplay" && matches!(v, Expr::BoolLiteral(true)))
            });
            let interval = ui
                .args
                .iter()
                .find_map(|a| {
                    if let Arg::Named(k, v) = a {
                        if k == "interval" {
                            if let Expr::NumberLiteral(n) = v {
                                return Some(*n as u32);
                            }
                        }
                        None
                    } else {
                        None
                    }
                })
                .unwrap_or(5000);

            if autoplay {
                self.emit_line(&format!(
                    "setInterval(() => {{ const n = ({}() + 1) % {}; {}.set(n); {}.style.transform = `translateX(-${{n*100}}%)`; }}, {});",
                    idx_var, slide_count, idx_var, track_var, interval
                ));
            }
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_icon_button(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let icon = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "icon" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "\"\"".to_string());

        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let mut cls = "wf-icon-btn".to_string();
        for m in &ui.modifiers {
            match m.as_str() {
                "small" => cls.push_str(" wf-icon-btn--small"),
                "large" => cls.push_str(" wf-icon-btn--large"),
                "primary" => cls.push_str(" wf-icon-btn--primary"),
                "danger" => cls.push_str(" wf-icon-btn--danger"),
                _ => {}
            }
        }

        let mut btn_attrs = format!("className: \"{}\", \"data-icon\": {}", cls, icon);
        if let Some(l) = &label {
            btn_attrs.push_str(&format!(", \"aria-label\": {}", l));
        }
        btn_attrs.push_str(&format!(", title: {}", label.as_deref().unwrap_or(&icon)));

        // Click handler from children (same as Button shorthand)
        if ui.events.is_empty() && !ui.children.is_empty() {
            let all_actions = ui.children.iter().all(|s| {
                matches!(
                    &s.kind,
                    StatementKind::Assignment(_)
                        | StatementKind::MethodCall(_)
                        | StatementKind::Navigate(_)
                        | StatementKind::ExprStatement(_)
                )
            });
            if all_actions {
                let body = self.emit_statements_inline(&ui.children);
                btn_attrs.push_str(&format!(", \"on:click\": (e) => {{ {} }}", body));
            }
        }
        for handler in &ui.events {
            let body = self.emit_event_body(&handler.body);
            btn_attrs.push_str(&format!(
                ", \"on:{}\": (event) => {{ {} }}",
                handler.event, body
            ));
        }
        if let Some(entry) = self.wf_node_entry(ui) {
            btn_attrs.push_str(&format!(", {}", entry));
        }

        self.emit_line(&format!(
            "const {} = WF.h(\"button\", {{ {} }}, WF.h(\"span\", {{ className: \"wf-icon\", \"data-icon\": {} }}));",
            var, btn_attrs, icon
        ));
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_slider(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let bind_var = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "bind" {
                    if let Expr::Identifier(s) = v {
                        return Some(s.clone());
                    }
                }
                None
            } else {
                None
            }
        });
        let min_val = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "min" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "0".to_string());
        let max_val = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "max" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "100".to_string());
        let step = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "step" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "1".to_string());
        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Slider", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"label\", {{ className: \"wf-form-label\" }}, {});",
                label_var, l
            ));
            self.emit_line(&format!("{}.appendChild({});", var, label_var));
        }

        let input_var = self.fresh_var();
        let mut input_attrs = format!(
            "type: \"range\", min: {}, max: {}, step: {}",
            min_val, max_val, step
        );
        if let Some(state) = &bind_var {
            input_attrs.push_str(&format!(
                ", value: () => _{}(), \"on:input\": (e) => _{}.set(Number(e.target.value))",
                state, state
            ));
        }
        for handler in &ui.events {
            let body = self.emit_event_body(&handler.body);
            input_attrs.push_str(&format!(
                ", \"on:{}\": (event) => {{ {} }}",
                handler.event, body
            ));
        }
        self.emit_line(&format!(
            "const {} = WF.h(\"input\", {{ {} }});",
            input_var, input_attrs
        ));
        self.emit_line(&format!("{}.appendChild({});", var, input_var));

        // Show current value if bound
        if let Some(state) = &bind_var {
            let val_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"span\", {{ className: \"wf-slider__value\" }}, () => String(_{}()));",
                val_var, state
            ));
            self.emit_line(&format!("{}.appendChild({});", var, val_var));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_datepicker(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let bind_var = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "bind" {
                    if let Expr::Identifier(s) = v {
                        return Some(s.clone());
                    }
                }
                None
            } else {
                None
            }
        });
        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let min = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "min" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let max = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "max" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Root on the component's own class, not the generic form-group: the
        // static renderers key on `wf-datepicker`, and hydration has to find the
        // same root they painted.
        let wrapper_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
            wrapper_var,
            self.class_attr("DatePicker", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"label\", {{ className: \"wf-form-label\" }}, {});",
                label_var, l
            ));
            self.emit_line(&format!("{}.appendChild({});", wrapper_var, label_var));
        }

        let input_var = self.fresh_var();
        let mut input_attrs = "type: \"date\", className: \"wf-input\"".to_string();
        if let Some(state) = &bind_var {
            input_attrs.push_str(&format!(
                ", value: () => _{}(), \"on:change\": (e) => _{}.set(e.target.value)",
                state, state
            ));
        }
        if let Some(mn) = min {
            input_attrs.push_str(&format!(", min: {}", mn));
        }
        if let Some(mx) = max {
            input_attrs.push_str(&format!(", max: {}", mx));
        }
        for handler in &ui.events {
            let body = self.emit_event_body(&handler.body);
            input_attrs.push_str(&format!(
                ", \"on:{}\": (event) => {{ {} }}",
                handler.event, body
            ));
        }
        self.emit_line(&format!(
            "const {} = WF.h(\"input\", {{ {} }});",
            input_var, input_attrs
        ));
        self.emit_line(&format!("{}.appendChild({});", wrapper_var, input_var));

        self.emit_line(&format!("const {} = {};", var, wrapper_var));
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_file_upload(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let accept = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "accept" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let multiple = ui.modifiers.iter().any(|m| m == "multiple");

        let wrapper_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
            wrapper_var,
            self.class_attr("FileUpload", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"label\", {{ className: \"wf-form-label\" }}, {});",
                label_var, l
            ));
            self.emit_line(&format!("{}.appendChild({});", wrapper_var, label_var));
        }

        let input_var = self.fresh_var();
        let mut input_attrs = "type: \"file\", className: \"wf-input\"".to_string();
        if let Some(acc) = &accept {
            input_attrs.push_str(&format!(", accept: {}", acc));
        }
        if multiple {
            input_attrs.push_str(", multiple: true");
        }
        for handler in &ui.events {
            let body = self.emit_event_body(&handler.body);
            input_attrs.push_str(&format!(
                ", \"on:{}\": (event) => {{ {} }}",
                handler.event, body
            ));
        }
        self.emit_line(&format!(
            "const {} = WF.h(\"input\", {{ {} }});",
            input_var, input_attrs
        ));
        self.emit_line(&format!("{}.appendChild({});", wrapper_var, input_var));

        self.emit_line(&format!("const {} = {};", var, wrapper_var));
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    // ─── Control flow (DOM) ──────────────────────────

    fn emit_if_dom(&mut self, if_stmt: &IfStmt, parent: &str) {
        let cond = self.emit_expr(&if_stmt.condition);

        self.emit_line(&format!("WF.condRender({},", parent));
        self.indent += 1;
        self.emit_line(&format!("() => {},", cond));

        // Then branch
        self.emit_line("() => {");
        self.indent += 1;
        let then_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = document.createDocumentFragment();",
            then_var
        ));
        for stmt in &if_stmt.then_body {
            self.emit_statement_dom(stmt, &then_var);
        }
        self.emit_line(&format!("return {};", then_var));
        self.indent -= 1;
        self.emit_line("},");

        // Else branch
        if let Some(else_body) = &if_stmt.else_body {
            self.emit_line("() => {");
            self.indent += 1;
            let else_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = document.createDocumentFragment();",
                else_var
            ));
            for stmt in else_body {
                self.emit_statement_dom(stmt, &else_var);
            }
            self.emit_line(&format!("return {};", else_var));
            self.indent -= 1;
            self.emit_line("},");
        } else if !if_stmt.else_if_branches.is_empty() {
            self.emit_line("() => {");
            self.indent += 1;
            let elif_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = document.createDocumentFragment();",
                elif_var
            ));
            let elif = IfStmt {
                condition: if_stmt.else_if_branches[0].0.clone(),
                animate: if_stmt.animate.clone(),
                then_body: if_stmt.else_if_branches[0].1.clone(),
                else_if_branches: if_stmt.else_if_branches[1..].to_vec(),
                else_body: if_stmt.else_body.clone(),
            };
            self.emit_if_dom(&elif, &elif_var);
            self.emit_line(&format!("return {};", elif_var));
            self.indent -= 1;
            self.emit_line("},");
        } else {
            self.emit_line("null,");
        }

        // Animation config (5th argument)
        self.emit_animate_config(&if_stmt.animate);

        self.indent -= 1;
        self.emit_line(");");
    }

    fn emit_for_dom(&mut self, for_stmt: &ForStmt, parent: &str) {
        let list = self.emit_expr(&for_stmt.iterable);

        self.emit_line(&format!("WF.listRender({},", parent));
        self.indent += 1;
        self.emit_line(&format!("() => {},", list));

        let index_param = if let Some(idx) = &for_stmt.index {
            format!(", {}", idx)
        } else {
            ", _idx".to_string()
        };

        self.emit_line(&format!("({}{}) => {{", for_stmt.item, index_param));
        self.indent += 1;

        // The item and index are ordinary parameters of this callback for as
        // long as we are inside it, not signals. Pushed as a stack so a nested
        // loop's binding does not erase the outer one's.
        let bound = self.loop_bindings.len();
        self.loop_bindings.push(for_stmt.item.clone());
        if let Some(idx) = &for_stmt.index {
            self.loop_bindings.push(idx.clone());
        }

        let item_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = document.createDocumentFragment();",
            item_var
        ));
        for stmt in &for_stmt.body {
            self.emit_statement_dom(stmt, &item_var);
        }
        self.emit_line(&format!("return {};", item_var));

        self.loop_bindings.truncate(bound);
        self.indent -= 1;
        self.emit_line("},");

        // Animation config (4th argument)
        self.emit_animate_config(&for_stmt.animate);

        self.indent -= 1;
        self.emit_line(");");
    }

    fn emit_show_dom(&mut self, show_stmt: &ShowStmt, parent: &str) {
        let cond = self.emit_expr(&show_stmt.condition);

        self.emit_line(&format!("WF.showRender({},", parent));
        self.indent += 1;
        self.emit_line(&format!("() => {},", cond));
        self.emit_line("() => {");
        self.indent += 1;
        let content_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = document.createDocumentFragment();",
            content_var
        ));
        for stmt in &show_stmt.body {
            self.emit_statement_dom(stmt, &content_var);
        }
        self.emit_line(&format!("return {};", content_var));
        self.indent -= 1;
        self.emit_line("},");

        // Animation config (4th argument)
        self.emit_animate_config(&show_stmt.animate);

        self.indent -= 1;
        self.emit_line(");");
    }

    fn emit_animate_config(&mut self, config: &Option<AnimateConfig>) {
        if let Some(anim) = config {
            let mut parts = Vec::new();
            parts.push(format!("enter: \"{}\"", anim.enter));
            if let Some(exit) = &anim.exit {
                parts.push(format!("exit: \"{}\"", exit));
            }
            if let Some(dur) = &anim.duration {
                parts.push(format!("duration: \"{}\"", dur));
            }
            if let Some(delay) = &anim.delay {
                parts.push(format!("delay: \"{}\"", delay));
            }
            if let Some(stagger) = &anim.stagger {
                parts.push(format!("stagger: \"{}\"", stagger));
            }
            if let Some(easing) = &anim.easing {
                parts.push(format!("easing: \"{}\"", easing));
            }
            self.emit_line(&format!("{{ {} }}", parts.join(", ")));
        } else {
            self.emit_line("null");
        }
    }

    fn emit_fetch_dom(&mut self, fetch: &FetchDecl, parent: &str) {
        let url = self.emit_expr(&fetch.url);
        let var = self.fresh_var();

        // Build options
        let mut opts = Vec::new();
        for opt in &fetch.options {
            let val = self.emit_expr(&opt.value);
            opts.push(format!("{}: {}", opt.key, val));
        }
        let opts_str = if opts.is_empty() {
            "null".to_string()
        } else {
            format!("{{ {} }}", opts.join(", "))
        };

        self.emit_line(&format!(
            "const {} = WF.wfFetch({}, {}, {{",
            var, url, opts_str
        ));
        self.indent += 1;

        if let Some(loading) = &fetch.loading_block {
            self.emit_line("loading: () => {");
            self.indent += 1;
            let l_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = document.createDocumentFragment();",
                l_var
            ));
            for stmt in loading {
                self.emit_statement_dom(stmt, &l_var);
            }
            self.emit_line(&format!("return {};", l_var));
            self.indent -= 1;
            self.emit_line("},");
        }

        if let Some((err_var, error_body)) = &fetch.error_block {
            self.emit_line(&format!("error: ({}) => {{", err_var));
            self.indent += 1;
            // Create signal alias so _{err_var}() resolves inside the callback body
            self.emit_line(&format!("const _{} = () => {};", err_var, err_var));
            let e_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = document.createDocumentFragment();",
                e_var
            ));
            for stmt in error_body {
                self.emit_statement_dom(stmt, &e_var);
            }
            self.emit_line(&format!("return {};", e_var));
            self.indent -= 1;
            self.emit_line("},");
        }

        if let Some(success_body) = &fetch.success_block {
            self.emit_line(&format!("success: ({}) => {{", fetch.variable));
            self.indent += 1;
            // Create signal alias so _{variable}() resolves inside the callback body
            self.emit_line(&format!(
                "const _{} = () => {};",
                fetch.variable, fetch.variable
            ));
            let s_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = document.createDocumentFragment();",
                s_var
            ));
            for stmt in success_body {
                self.emit_statement_dom(stmt, &s_var);
            }
            self.emit_line(&format!("return {};", s_var));
            self.indent -= 1;
            self.emit_line("},");
        }

        self.indent -= 1;
        self.emit_line("});");
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    // ─── Statement (imperative, non-DOM) ─────────────

    fn emit_statement(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementKind::Assignment(a) => {
                let target = self.emit_expr(&a.target);
                let value = self.emit_expr(&a.value);
                // Check if target is a signal (state variable)
                if let Expr::Identifier(name) = &a.target {
                    self.emit_line(&format!("_{}.set({});", name, value));
                } else if let Expr::PropertyAccess(base, prop) = &a.target {
                    let base_str = self.emit_expr(base);
                    self.emit_line(&format!("{}.{} = {};", base_str, prop, value));
                } else {
                    self.emit_line(&format!("{} = {};", target, value));
                }
            }
            StatementKind::MethodCall(mc) => {
                let obj = self.emit_expr(&mc.object);
                let args: Vec<String> = mc.args.iter().map(|a| self.emit_expr(a)).collect();
                self.emit_line(&format!("{}.{}({});", obj, mc.method, args.join(", ")));
            }
            StatementKind::Navigate(expr) => {
                let path = self.emit_expr(expr);
                self.emit_line(&format!("WF.navigate({});", path));
            }
            StatementKind::Log(expr) => {
                let val = self.emit_expr(expr);
                self.emit_line(&format!("console.log({});", val));
            }
            StatementKind::Animate(anim) => {
                let dur = anim
                    .duration
                    .as_deref()
                    .map(|d| format!(", \"{}\"", d))
                    .unwrap_or_default();
                self.emit_line(&format!(
                    "WF.animateEl(\"{}\", \"{}\"{});",
                    anim.target, anim.animation, dur
                ));
            }
            StatementKind::ExprStatement(expr) => {
                let val = self.emit_expr(expr);
                self.emit_line(&format!("{};", val));
            }
            StatementKind::State(s) => {
                let val = self.emit_expr(&s.value);
                self.emit_line(&format!("const _{} = WF.signal({});", s.name, val));
            }
            StatementKind::If(if_stmt) => {
                let cond = self.emit_expr(&if_stmt.condition);
                self.emit_line(&format!("if ({}) {{", cond));
                self.indent += 1;
                for s in &if_stmt.then_body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                if let Some(else_body) = &if_stmt.else_body {
                    self.emit_line("} else {");
                    self.indent += 1;
                    for s in else_body {
                        self.emit_statement(s);
                    }
                    self.indent -= 1;
                }
                self.emit_line("}");
            }
            StatementKind::Fetch(fetch) => {
                self.emit_imperative_fetch(fetch);
            }
            StatementKind::Return(expr) => {
                if let Some(e) = expr {
                    let val = self.emit_expr(e);
                    self.emit_line(&format!("return {};", val));
                } else {
                    self.emit_line("return;");
                }
            }
            _ => {}
        }
    }

    fn emit_imperative_fetch(&mut self, fetch: &FetchDecl) {
        let url = self.emit_expr(&fetch.url);
        let mut opts = Vec::new();
        for opt in &fetch.options {
            let val = self.emit_expr(&opt.value);
            opts.push(format!("{}: {}", opt.key, val));
        }

        let method = fetch
            .options
            .iter()
            .find_map(|o| {
                if o.key == "method" {
                    Some(self.emit_expr(&o.value))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "\"GET\"".to_string());

        let body = fetch.options.iter().find_map(|o| {
            if o.key == "body" {
                Some(self.emit_expr(&o.value))
            } else {
                None
            }
        });

        self.emit_line(&format!("fetch({}, {{", url));
        self.indent += 1;
        self.emit_line(&format!("method: {},", method));
        if let Some(b) = body {
            self.emit_line("headers: { \"Content-Type\": \"application/json\" },");
            self.emit_line(&format!("body: JSON.stringify({}),", b));
        }
        self.indent -= 1;
        self.emit_line("})");
        self.emit_line(".then(r => r.json())");
        self.emit_line(&format!(".then({} => {{", fetch.variable));
        self.indent += 1;
        if let Some(success_body) = &fetch.success_block {
            for s in success_body {
                self.emit_statement(s);
            }
        }
        self.indent -= 1;
        self.emit_line("})");

        if let Some((err_var, error_body)) = &fetch.error_block {
            self.emit_line(&format!(".catch({} => {{", err_var));
            self.indent += 1;
            for s in error_body {
                self.emit_statement(s);
            }
            self.indent -= 1;
            self.emit_line("});");
        } else {
            self.emit_line(".catch(e => console.error(e));");
        }
    }

    // ─── Expression emitter ──────────────────────────

    /// Compile one `style { prop: value }` declaration to `(camelCaseProperty, jsValue)`.
    ///
    /// A bare design-token keyword (`font-size: xl`, `padding: md`) resolves to a
    /// `"var(--…)"` string literal; property aliases (`radius`, `shadow`) map to their
    /// real CSS property. Anything else keeps the normal expression path, so quoted
    /// CSS, numbers, and reactive-state identifiers (`font-size: someState` → `_someState()`)
    /// are unchanged.
    fn emit_style_decl(&self, prop: &StyleProperty) -> (String, String) {
        use crate::codegen::style_tokens::{canonical_style_prop, resolve_style_token};
        let css_prop = canonical_style_prop(&prop.name);
        let val = match resolve_style_token(&css_prop, &prop.value) {
            Some(css) => format!("\"{css}\""),
            None => self.emit_expr(&prop.value),
        };
        (to_camel_case(&css_prop), val)
    }

    fn emit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLiteral(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
                    .replace('\u{FFFE}', "{")
                    .replace('\u{FFFF}', "}");
                format!("\"{}\"", escaped)
            }
            Expr::InterpolatedString(parts) => {
                let mut out = String::from("`");
                for part in parts {
                    match part {
                        StringPart::Literal(s) => out.push_str(&s.replace('`', "\\`")),
                        StringPart::Expression(e) => {
                            out.push_str("${");
                            out.push_str(&self.emit_expr(e));
                            out.push('}');
                        }
                    }
                }
                out.push('`');
                out
            }
            Expr::NumberLiteral(n) => {
                if *n == (*n as i64) as f64 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Expr::BoolLiteral(b) => format!("{}", b),
            Expr::Null => "null".to_string(),
            Expr::Identifier(name) => {
                // i18n: locale and dir are reactive i18n signals
                if self.has_i18n() && (name == "locale" || name == "dir") {
                    return format!("WF.i18n.{}()", name);
                }
                // Browser globals should NOT be prefixed
                const BROWSER_GLOBALS: &[&str] = &[
                    "window",
                    "document",
                    "console",
                    "localStorage",
                    "sessionStorage",
                    "JSON",
                    "Math",
                    "Date",
                    "setTimeout",
                    "setInterval",
                    "clearTimeout",
                    "clearInterval",
                    "parseInt",
                    "parseFloat",
                    "Array",
                    "Object",
                    "String",
                    "Number",
                    "Boolean",
                    "Promise",
                    "Error",
                    "Map",
                    "Set",
                    "RegExp",
                    "Infinity",
                    "NaN",
                    "undefined",
                    "encodeURIComponent",
                    "decodeURIComponent",
                    "encodeURI",
                    "decodeURI",
                    "atob",
                    "btoa",
                    "fetch",
                    "alert",
                    "confirm",
                    "prompt",
                    "requestAnimationFrame",
                    "cancelAnimationFrame",
                ];
                if BROWSER_GLOBALS.contains(&name.as_str()) {
                    return name.to_string();
                }
                // Store references, component props, and built-in names stay as-is
                if self.stores.contains(name)
                    || self.current_props.contains(name)
                    || self.loop_bindings.contains(name)
                    || name == "params"
                    || name == "value"
                    || name == "key"
                    || name == "event"
                    || name == "e"
                    || name.starts_with("_")
                {
                    name.to_string()
                } else {
                    // State variable (signal) — access via _name()
                    format!("_{}()", name)
                }
            }
            Expr::PropertyAccess(base, prop) => {
                let base_str = self.emit_expr(base);
                format!("{}.{}", base_str, prop)
            }
            Expr::IndexAccess(base, index) => {
                let base_str = self.emit_expr(base);
                let idx_str = self.emit_expr(index);
                format!("{}[{}]", base_str, idx_str)
            }
            Expr::BinaryOp(left, op, right) => {
                let l = self.emit_expr(left);
                let r = self.emit_expr(right);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq => "===",
                    BinOp::Neq => "!==",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::Lte => "<=",
                    BinOp::Gte => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::UnaryOp(op, expr) => {
                let e = self.emit_expr(expr);
                match op {
                    UnaryOp::Not => format!("!{}", e),
                    UnaryOp::Neg => format!("-{}", e),
                }
            }
            Expr::MethodCall(obj, method, args) => {
                if method == "__if" && args.len() == 2 {
                    // Conditional expression
                    let cond = self.emit_expr(obj);
                    let then_val = self.emit_expr(&args[0]);
                    let else_val = self.emit_expr(&args[1]);
                    return format!("({} ? {} : {})", cond, then_val, else_val);
                }

                let obj_str = self.emit_expr(obj);
                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();

                // Map WebFluent methods to JS
                match method.as_str() {
                    "push" => format!("{}.push({})", obj_str, args_str.join(", ")),
                    "remove" => format!("{}.splice({}, 1)", obj_str, args_str.join(", ")),
                    "filter" => format!("{}.filter({})", obj_str, args_str.join(", ")),
                    "map" => format!("{}.map({})", obj_str, args_str.join(", ")),
                    "sum" => format!("{}.reduce((a,b) => a+b, 0)", obj_str),
                    "length" => format!("{}.length", obj_str),
                    "toUpper" => format!("{}.toUpperCase()", obj_str),
                    "toLower" => format!("{}.toLowerCase()", obj_str),
                    "contains" => format!("{}.includes({})", obj_str, args_str.join(", ")),
                    "trim" => format!("{}.trim()", obj_str),
                    "split" => format!("{}.split({})", obj_str, args_str.join(", ")),
                    _ => format!("{}.{}({})", obj_str, method, args_str.join(", ")),
                }
            }
            Expr::FunctionCall(name, args) => {
                // i18n: t("key") or t("key", name: value, ...)
                if name == "t" && self.has_i18n() {
                    if args.is_empty() {
                        return "\"\"".to_string();
                    }
                    let key = self.emit_expr(&args[0]);
                    if args.len() == 1 {
                        return format!("WF.i18n.t({})", key);
                    }
                    // Remaining args are named params for interpolation
                    // They come as FunctionCall args — could be positional expressions
                    // In practice the parser sees t("key", name: value) where name: value
                    // is parsed as separate expressions. We need to handle both positional
                    // and the case where the parser gave us the values.
                    let params: Vec<String> = args[1..].iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.i18n.t({}, {})", key, params.join(", "));
                }
                // i18n: setLocale("ar")
                if name == "setLocale" && self.has_i18n() {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.i18n.setLocale({})", args_str.join(", "));
                }

                // WF runtime functions
                if name == "replayAnimation" {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.replayAnimation({})", args_str.join(", "));
                }

                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                // Check if it's a store function
                if self.stores.contains(name) {
                    format!(
                        "{}.{}({})",
                        name,
                        args_str.first().unwrap_or(&String::new()),
                        args_str.get(1..).unwrap_or(&[]).join(", ")
                    )
                } else {
                    format!("{}({})", name, args_str.join(", "))
                }
            }
            Expr::ListLiteral(items) => {
                let items_str: Vec<String> = items.iter().map(|i| self.emit_expr(i)).collect();
                format!("[{}]", items_str.join(", "))
            }
            Expr::MapLiteral(entries) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.emit_expr(v)))
                    .collect();
                format!("{{ {} }}", entries_str.join(", "))
            }
            Expr::Lambda(param, body) => {
                let body_str = self.emit_expr(body);
                format!("({} => {})", param, body_str)
            }
        }
    }

    // ─── Helpers ─────────────────────────────────────

    fn emit_line(&mut self, text: &str) {
        let indent = "  ".repeat(self.indent);
        self.output.push_str(&format!("{}{}\n", indent, text));
    }

    fn fresh_var(&self) -> String {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        format!("_e{}", COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    fn emit_component_args(&self, args: &[Arg]) -> String {
        let mut parts = Vec::new();
        for arg in args {
            match arg {
                Arg::Named(name, expr) => {
                    parts.push(format!("{}: {}", name, self.emit_expr(expr)));
                }
                Arg::Positional(expr) => {
                    parts.push(self.emit_expr(expr));
                }
            }
        }
        if parts.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", parts.join(", "))
        }
    }

    fn emit_event_body(&mut self, stmts: &[Statement]) -> String {
        let mut parts = Vec::new();
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::Assignment(a) => {
                    let value = self.emit_expr(&a.value);
                    if let Expr::Identifier(name) = &a.target {
                        parts.push(format!("_{}.set({});", name, value));
                    } else {
                        let target = self.emit_expr(&a.target);
                        parts.push(format!("{} = {};", target, value));
                    }
                }
                StatementKind::Navigate(expr) => {
                    let path = self.emit_expr(expr);
                    parts.push(format!("WF.navigate({});", path));
                }
                StatementKind::ExprStatement(expr) => {
                    let val = self.emit_expr(expr);
                    parts.push(format!("{};", val));
                }
                StatementKind::MethodCall(mc) => {
                    let obj = self.emit_expr(&mc.object);
                    let args: Vec<String> = mc.args.iter().map(|a| self.emit_expr(a)).collect();
                    parts.push(format!("{}.{}({});", obj, mc.method, args.join(", ")));
                }
                StatementKind::If(if_stmt) => {
                    let cond = self.emit_expr(&if_stmt.condition);
                    let then_body = self.emit_statements_inline(&if_stmt.then_body);
                    if let Some(else_body) = &if_stmt.else_body {
                        let else_str = self.emit_statements_inline(else_body);
                        parts.push(format!(
                            "if ({}) {{ {} }} else {{ {} }}",
                            cond, then_body, else_str
                        ));
                    } else {
                        parts.push(format!("if ({}) {{ {} }}", cond, then_body));
                    }
                }
                _ => {}
            }
        }
        parts.join(" ")
    }

    fn emit_statements_inline(&mut self, stmts: &[Statement]) -> String {
        let mut parts = Vec::new();
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::Assignment(a) => {
                    let value = self.emit_expr(&a.value);
                    if let Expr::Identifier(name) = &a.target {
                        parts.push(format!("_{}.set({});", name, value));
                    } else {
                        let target = self.emit_expr(&a.target);
                        parts.push(format!("{} = {};", target, value));
                    }
                }
                StatementKind::Navigate(expr) => {
                    let path = self.emit_expr(expr);
                    parts.push(format!("WF.navigate({});", path));
                }
                StatementKind::ExprStatement(expr) => {
                    let val = self.emit_expr(expr);
                    parts.push(format!("{};", val));
                }
                _ => {}
            }
        }
        parts.join(" ")
    }
}

/// Whether `ui` directly contains a `Parent.Sub` element.
fn has_subcomponent(ui: &UIElement, parent: &str, sub: &str) -> bool {
    ui.children.iter().any(|stmt| {
        matches!(&stmt.kind, StatementKind::UIElement(child)
            if matches!(&child.component, ComponentRef::SubComponent(p, s)
                if p == parent && s == sub))
    })
}

// ─── Utility functions ──────────────────────────────────

fn is_reactive_expr(expr_str: &str) -> bool {
    // Check for signal access pattern: _identifier()
    let bytes = expr_str.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'_' && i + 1 < bytes.len() && (bytes[i + 1] as char).is_alphanumeric() {
            // Found _identifier, check if it's followed by ()
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] as char).is_alphanumeric() {
                j += 1;
            }
            if j + 1 < bytes.len() && bytes[j] == b'(' && bytes[j + 1] == b')' {
                return true;
            }
        }
    }
    // Also check for WF.i18n.t( which is reactive (locale changes)
    if expr_str.contains("WF.i18n.t(")
        || expr_str.contains("WF.i18n.locale()")
        || expr_str.contains("WF.i18n.dir()")
    {
        return true;
    }
    false
}

fn to_camel_case(kebab: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for ch in kebab.chars() {
        if ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
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
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile(src: &str) -> String {
        let tokens = Lexer::new(src, "<t>").tokenize().expect("lex");
        let program = Parser::new(tokens, "<t>").parse().expect("parse");
        JsCodegen::new().generate(&program)
    }

    /// `WF.listRender` hands the body its item as a plain callback parameter, so
    /// every reference to it must stay plain.
    ///
    /// The identifier path treated anything that was not a prop or a store as
    /// state, so a loop over `tasks` bound `task` and then read `_task()` —
    /// `ReferenceError` on the first non-empty list. `wf init -t spa` shipped it.
    #[test]
    fn a_loop_variable_is_a_plain_binding_not_a_signal() {
        let js = compile(
            "Page P (path: \"/\") {\n\
             \x20   state items = []\n\
             \x20   for item in items { Text(item.title) }\n\
             }",
        );
        assert!(
            js.contains("item.title"),
            "the loop body should read the binding directly:\n{js}"
        );
        assert!(
            !js.contains("_item()"),
            "the loop variable was emitted as a signal:\n{js}"
        );
    }

    /// The same has to hold inside an event handler nested in the loop, which is
    /// where the pattern actually shows up — a row with a button acting on it.
    #[test]
    fn a_loop_variable_survives_into_a_nested_event_handler() {
        let js = compile(
            "Store S { state rows = []\n action pick(id: Number) { } }\n\
             Page P (path: \"/\") {\n\
             \x20   use S\n\
             \x20   for row in S.rows { Button(\"Pick\") { S.pick(row.id) } }\n\
             }",
        );
        assert!(
            js.contains("S.pick(row.id)"),
            "handler lost the binding:\n{js}"
        );
        assert!(
            !js.contains("_row()"),
            "handler treated the binding as state:\n{js}"
        );
    }

    /// An index binding is a parameter too.
    #[test]
    fn a_loop_index_is_a_plain_binding() {
        let js = compile(
            "Page P (path: \"/\") {\n\
             \x20   state items = []\n\
             \x20   for item, i in items { Text(\"{i}: {item.name}\") }\n\
             }",
        );
        assert!(
            !js.contains("_i()"),
            "the index was emitted as a signal:\n{js}"
        );
        assert!(
            !js.contains("_item()"),
            "the item was emitted as a signal:\n{js}"
        );
    }

    /// A nested loop must not erase the outer binding when it finishes.
    #[test]
    fn nested_loops_each_keep_their_own_binding() {
        let js = compile(
            "Page P (path: \"/\") {\n\
             \x20   state groups = []\n\
             \x20   for group in groups {\n\
             \x20       for member in group.members { Text(member.name) }\n\
             \x20       Text(group.title)\n\
             \x20   }\n\
             }",
        );
        assert!(
            !js.contains("_member()"),
            "inner binding became a signal:\n{js}"
        );
        assert!(
            !js.contains("_group()"),
            "the outer binding was lost after the inner loop closed:\n{js}"
        );
        assert!(
            js.contains("group.title"),
            "outer binding missing after nesting:\n{js}"
        );
    }

    /// State that is genuinely state must still be read as a signal — the fix
    /// above must not make everything plain.
    #[test]
    fn state_outside_a_loop_is_still_a_signal() {
        let js = compile(
            "Page P (path: \"/\") {\n\
             \x20   state count = 0\n\
             \x20   Text(\"{count}\")\n\
             }",
        );
        assert!(
            js.contains("_count()"),
            "state stopped being reactive:\n{js}"
        );
    }
}
