use crate::codegen::builtin::{
    builtin_to_html, element_tag, implicit_role, input_type, is_void, landmark_label,
    layout_arg_classes, modifier_to_class,
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
    /// Parameters of the lambdas being emitted, innermost last.
    ///
    /// `items.filter(x => x.done)` binds `x` as a plain JavaScript parameter,
    /// so a reference to it inside the body must stay plain. It used to fall
    /// through to the signal path and become `_x()` — a `ReferenceError` in any
    /// page or component. (`emit_expr` takes `&self`, hence the cell.)
    lambda_params: std::cell::RefCell<Vec<String>>,
    /// Locals declared so far in the store action being emitted, so a second
    /// assignment does not redeclare.
    store_locals: std::cell::RefCell<Vec<String>>,
    /// i18n: default locale and translations (locale -> key -> value)
    i18n_default_locale: Option<String>,
    i18n_translations: HashMap<String, HashMap<String, String>>,
    /// SSG mode: emit hydration instead of full mount
    ssg_mode: bool,
    /// Base path for deployment (e.g., "/WebFluent")
    base_path: String,
    /// Each page's title, so the router can set `document.title` when it
    /// shows the page; a SPA otherwise keeps the entry page's title forever.
    page_titles: HashMap<String, String>,
    /// Whether the element being emitted sits inside a `Thead`, where a
    /// `Tcell` is a column header (`<th scope="col">`), not a data cell.
    in_thead: bool,
    /// Studio mode: stamp `data-wf-node` ids on rendered elements. Off for
    /// export/release builds, which must contain no debug attributes.
    studio: bool,
    /// Deterministic node ids keyed by element span (empty unless in studio mode).
    node_ids: NodeMap,
    /// Whether each page is written as its own chunk (`pages/<Name>.js`),
    /// registered with `WF.definePage`, rather than into the main bundle.
    split_pages: bool,
    /// The page chunks, `(name, source)`, when `split_pages` is on.
    chunks: Vec<(String, String)>,
    /// The pages with a stylesheet of their own (`pages/<Name>.css`), which
    /// the router loads before drawing them.
    page_sheets: std::collections::BTreeSet<String>,
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
            lambda_params: std::cell::RefCell::new(Vec::new()),
            store_locals: std::cell::RefCell::new(Vec::new()),
            i18n_default_locale: None,
            i18n_translations: HashMap::new(),
            ssg_mode: false,
            base_path: String::new(),
            page_titles: HashMap::new(),
            in_thead: false,
            studio: false,
            node_ids: NodeMap::default(),
            split_pages: false,
            chunks: Vec::new(),
            page_sheets: std::collections::BTreeSet::new(),
        }
    }

    /// Write each page as its own chunk, loaded when its route shows.
    pub fn set_split_pages(&mut self, enabled: bool) {
        self.split_pages = enabled;
    }

    /// The page chunks `generate` set aside, `(page name, source)`.
    pub fn take_chunks(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.chunks)
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

    /// One entry of the router's table: the path, the page's title (so the
    /// router can set `document.title`), and how to render it.
    fn route_entry(&self, path: &str, page: &str) -> String {
        let title = match self.page_titles.get(page) {
            Some(t) => format!(
                "title: \"{}\", ",
                t.replace('\\', "\\\\").replace('"', "\\\"")
            ),
            None => String::new(),
        };
        if self.split_pages {
            let css = if self.page_sheets.contains(page) {
                format!("css: \"{}\", ", page)
            } else {
                String::new()
            };
            format!(
                "{{ path: \"{}\", {}{}page: \"{}\" }},",
                path, title, css, page
            )
        } else {
            format!(
                "{{ path: \"{}\", {}render: (params) => Page_{}(params) }},",
                path, title, page
            )
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // Emit runtime
        self.emit_line(runtime::RUNTIME_JS);
        self.emit_line("");

        if self.split_pages {
            self.page_sheets = crate::codegen::scoped_css::split_rules(program)
                .pages
                .into_keys()
                .collect();
        }

        // First pass: collect component and store names
        for decl in &program.declarations {
            match decl {
                Declaration::Component(c) => self.components.push(c.name.clone()),
                Declaration::Store(s) => self.stores.push(s.name.clone()),
                Declaration::Page(p) => {
                    if let Some(title) = &p.title {
                        self.page_titles.insert(p.name.clone(), title.clone());
                    }
                }
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

        // Emit pages: into the bundle, or each into its own chunk that
        // registers itself with the runtime when it runs.
        for decl in &program.declarations {
            if let Declaration::Page(p) = decl {
                if self.split_pages {
                    let saved = std::mem::take(&mut self.output);
                    let indent = std::mem::replace(&mut self.indent, 0);
                    self.emit_page(p);
                    self.emit_line(&format!("WF.definePage(\"{}\", Page_{});", p.name, p.name));
                    let chunk = std::mem::replace(&mut self.output, saved);
                    self.indent = indent;
                    self.chunks.push((p.name.clone(), chunk));
                } else {
                    self.emit_page(p);
                }
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
                if self.split_pages {
                    self.emit_line(&format!(
                        "WF.loadPage(\"{}\", (page) => WF.{}(() => page({{}}), WF.mainOf(document.getElementById('app'))));",
                        pages[0].name, mount_fn
                    ));
                } else {
                    self.emit_line(&format!(
                        "WF.{}(() => Page_{}({{}}), WF.mainOf(document.getElementById('app')));",
                        mount_fn, pages[0].name
                    ));
                }
            } else if !pages.is_empty() {
                // Auto-create router from page paths
                self.emit_line("(function() {");
                self.indent += 1;
                self.emit_line("const routes = [");
                self.indent += 1;
                for p in &pages {
                    let entry = self.route_entry(&p.path, &p.name);
                    self.emit_line(&entry);
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

        // Every name the store exposes on itself: state, derived and actions.
        // A derived value used to know only the state names, so a derived
        // built on another derived, or on an action, read an undefined
        // variable at run time.
        let store_state_names: Vec<String> = store
            .body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::State(st) => Some(st.name.clone()),
                StatementKind::Derived(d) => Some(d.name.clone()),
                StatementKind::Action(a) => Some(a.name.clone()),
                _ => None,
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
                self.store_locals.borrow_mut().clear();
                // A parameter shadows a store member of the same name: an
                // action `move(step)` beside an action `step` used to read
                // `store.step` — the function — where its argument was meant.
                let visible: Vec<String> = store_state_names
                    .iter()
                    .filter(|n| !params.contains(n))
                    .cloned()
                    .collect();
                for stmt in &a.body {
                    self.emit_store_statement(stmt, &visible, &params);
                }
                self.store_locals.borrow_mut().clear();
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
                if method == "__if" && args.len() == 2 {
                    // An if-expression inside a store used to fall through to
                    // the page emitter, whose operands read `_x()` signals.
                    let cond = self.emit_store_expr(obj, store_states);
                    let then_val = self.emit_store_expr(&args[0], store_states);
                    let else_val = self.emit_store_expr(&args[1], store_states);
                    return format!("({} ? {} : {})", cond, then_val, else_val);
                }
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
                format!("(({}) => {})", param, body_str)
            }
            Expr::FunctionCall(name, args) => {
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_store_expr(a, store_states))
                    .collect();
                // A call to one of the store's own actions goes through the
                // store; anything else is a global (`Math.round` is a method
                // call, but `parseInt` is a plain function).
                if store_states.contains(name) {
                    format!("store.{}({})", name, args_str.join(", "))
                } else {
                    format!("{}({})", name, args_str.join(", "))
                }
            }
            Expr::InterpolatedString(parts) => {
                let mut out = String::from("`");
                for part in parts {
                    match part {
                        StringPart::Literal(t) => out.push_str(&t.replace('`', "\\`")),
                        StringPart::Expression(e) => {
                            out.push_str("${");
                            out.push_str(&self.emit_store_expr(e, store_states));
                            out.push('}');
                        }
                    }
                }
                out.push('`');
                out
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
                // Parenthesised, so a map literal is an object wherever it
                // lands — as an arrow function's body a bare `{` is a block.
                format!("({{ {} }})", entries_str.join(", "))
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
                    } else if action_params.contains(name)
                        || self.store_locals.borrow().contains(name)
                    {
                        self.emit_line(&format!("{} = {};", name, value));
                    } else {
                        // A name that is neither state nor a parameter is a
                        // local of this action. It used to be assigned bare,
                        // which the bundle's strict mode refuses.
                        self.store_locals.borrow_mut().push(name.clone());
                        self.emit_line(&format!("let {} = {};", name, value));
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
            StatementKind::Return(expr) => {
                // Used to fall through to the page emitter, so a returned
                // expression read `_x()` signals that do not exist in a store.
                match expr {
                    Some(e) => {
                        let value = self.emit_store_expr(e, store_states);
                        self.emit_line(&format!("return {};", value));
                    }
                    None => self.emit_line("return;"),
                }
            }
            StatementKind::MethodCall(mc) => {
                let obj = self.emit_store_expr(&mc.object, store_states);
                let args: Vec<String> = mc
                    .args
                    .iter()
                    .map(|a| self.emit_store_expr(a, store_states))
                    .collect();
                self.emit_line(&format!("{}.{}({});", obj, mc.method, args.join(", ")));
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
        // Set current props so emit_expr treats them as plain variables, not signals
        self.current_props = params.clone();
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
        // The second parameter is the caller's block, as a thunk that builds
        // it, so `children` can be placed anywhere in the body — including
        // inside a conditional or a loop, whose closures see the parameter.
        self.emit_line(&format!(
            "function Component_{}(_p, _children) {{",
            comp.name
        ));
        self.indent += 1;
        self.emit_line(&format!(
            "_p = WF.props(_p, {{ {} }});",
            defaults.join(", ")
        ));

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
                let entry = self.route_entry(clean_path, &page_name);
                self.emit_line(&entry);
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
                    classes.extend(layout_arg_classes(&ui.args));
                    self.emit_line(&format!(
                        "const {} = WF.h(\"{}\", {{ className: \"{}\"{} }});",
                        var,
                        tag,
                        classes.join(" "),
                        self.wf_node_inline(ui)
                    ));
                    self.emit_style_and_transition(&var, ui);
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

                let (_, class) = builtin_to_html(name);
                // A heading's level is part of the document outline, so it has to
                // reach the tag; a class cannot express it.
                let tag = if name == "Tcell" && self.in_thead {
                    "th"
                } else {
                    element_tag(name, &ui.modifiers)
                };

                // Collect attributes
                let mut attrs = Vec::new();
                let mut link_to: Option<String> = None;
                let mut link_prefix = false;
                let mut inner_text: Option<String> = None;

                // Build class string from base class + modifiers
                let mut classes = vec![class.to_string()];
                for m in &ui.modifiers {
                    let mod_class = modifier_to_class(class, m);
                    classes.push(mod_class);
                }

                // An `Input` or `Select` with a label, hint or error is wrapped
                // in a field that carries them.
                let is_field = matches!(name.as_str(), "Input" | "Select")
                    && ui.args.iter().any(|a| {
                        matches!(a, Arg::Named(k, _) if k == "label" || k == "hint" || k == "error")
                    });

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
                                // A field's label, hint and error are elements
                                // beside the control, built by `WF.field` below.
                                "label" | "hint" | "error" if is_field => {}
                                "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max"
                                | "step" | "accept" | "label" | "required" | "disabled"
                                | "controls" | "autoplay" | "role" | "width" | "height"
                                | "loading" | "decoding" | "fetchpriority" => {
                                    // A value that reads state follows it: a
                                    // `placeholder` or `disabled` bound to a
                                    // store used to be painted once.
                                    let v = self.emit_expr(val);
                                    if self.is_reactive(&v) {
                                        attrs.push(format!("{}: () => {}", key, v));
                                    } else {
                                        attrs.push(format!("{}: {}", key, v));
                                    }
                                }
                                "to" => {
                                    let v = self.emit_expr(val);
                                    link_to = Some(v.clone());
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
                                "active" => {
                                    // `active: "prefix"` also matches routes under `to`.
                                    link_prefix =
                                        matches!(val, Expr::StringLiteral(s) if s == "prefix");
                                }
                                "span" => {
                                    if let Expr::NumberLiteral(n) = val {
                                        classes.push(format!("{}--{}", class, *n as i32));
                                    }
                                }
                                "gap" | "align" | "justify" => {
                                    // Handled once for the whole element below,
                                    // via `layout_arg_classes`.
                                }
                                "class" => {
                                    // The author's own classes join the
                                    // element's in `emit_style_and_transition`,
                                    // beside the style block's; as an
                                    // attribute they would replace the
                                    // engine's.
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
                                "caption" if name == "Table" => {
                                    // Emitted as the table's first child below.
                                }
                                "value" => {
                                    // A value that reads state follows it; the
                                    // runtime runs a function-valued `value` in
                                    // an effect. A Progress bar bound to state
                                    // used to be painted once and never move.
                                    let v = self.emit_expr(val);
                                    if self.is_reactive(&v) {
                                        attrs.push(format!("value: () => {}", v));
                                    } else {
                                        attrs.push(format!("value: {}", v));
                                    }
                                }
                                "icon" => {
                                    let v = self.emit_expr(val);
                                    attrs.push(format!("\"data-icon\": {}", v));
                                }
                                _ => {
                                    // Any other named argument is an HTML
                                    // attribute. A hyphenated name (`aria-*`,
                                    // `data-*`) is quoted; a value that reads
                                    // state is a thunk, which the runtime
                                    // keeps in step with it.
                                    let v = self.emit_expr(val);
                                    let k = if key.contains('-') {
                                        format!("\"{}\"", key)
                                    } else {
                                        key.clone()
                                    };
                                    if self.is_reactive(&v) {
                                        attrs.push(format!("{}: () => {}", k, v));
                                    } else {
                                        attrs.push(format!("{}: {}", k, v));
                                    }
                                }
                            }
                        }
                        Arg::Positional(expr) => {
                            // An Icon's positional argument names the glyph, not
                            // text to show: `Icon("home")` used to render the
                            // word "home" because only `icon:` set `data-icon`.
                            if name == "Icon" {
                                if !attrs.iter().any(|a| a.starts_with("\"data-icon\":")) {
                                    let v = self.emit_expr(expr);
                                    attrs.push(format!("\"data-icon\": {}", v));
                                }
                                continue;
                            }
                            // `Option("value", "Label")`: the first positional
                            // is the value sent with the form, the second the
                            // text shown. A lone positional is both. The label
                            // used to be dropped, so a select showed its values.
                            if name == "Option" && inner_text.is_some() {
                                if !attrs.iter().any(|a| a.starts_with("value:")) {
                                    let v = inner_text.take().unwrap_or_default();
                                    attrs.push(format!("value: {}", v));
                                    inner_text = Some(self.emit_expr(expr));
                                }
                                continue;
                            }
                            // First positional arg is usually the content/label
                            if inner_text.is_none() {
                                inner_text = Some(self.emit_expr(expr));
                            }
                        }
                    }
                }

                classes.extend(layout_arg_classes(&ui.args));

                // Handle input type modifiers
                for m in &ui.modifiers {
                    if let Some(t) = input_type(m) {
                        attrs.push(format!("type: \"{}\"", t));
                    } else if m == "required" {
                        attrs.push("required: true".to_string());
                    } else if m == "multiple" {
                        attrs.push("multiple: true".to_string());
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

                // The author's handlers are attached after the element is
                // built (see below), not written into the attribute object:
                // a `bind:` already puts an "on:input" there, and two keys of
                // one name in one object literal keep only the last — so an
                // Input with both used to lose its binding.

                // If element has a block with just statements (Button shorthand click)
                // A Button's block may mix what it shows with what it does:
                // the action-like statements are its click handler, the rest
                // its content. They used to become a handler only when the
                // block held nothing else; a badge beside an assignment made
                // the assignment run at render time.
                if ui.events.is_empty() && matches!(name.as_str(), "Button" | "IconButton") {
                    let actions: Vec<Statement> = ui
                        .children
                        .iter()
                        .filter(|s| is_action_statement(s))
                        .cloned()
                        .collect();
                    if !actions.is_empty() {
                        let body = self.emit_statements_inline(&actions);
                        attrs.push(format!("\"on:click\": (e) => {{ {} }}", body));
                    }
                }

                // A form never navigates away: the author's submit handler
                // is attached after creation, on top of this.
                if name == "Form" {
                    attrs.push("\"on:submit\": (e) => e.preventDefault()".to_string());
                }

                // Async decoding keeps an image off the critical path. Whether
                // it loads lazily is the runtime's call: the first image of a
                // page is the one the largest paint waits for, and is fetched
                // first; the rest load lazily. An explicit `loading:` wins.
                if name == "Image" {
                    for (key, default) in [("decoding", "async")] {
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

                if tag == "th" {
                    attrs.push("scope: \"col\"".to_string());
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

                // A table's caption is its accessible name. It is rendered
                // visually hidden: the heading above the table already says
                // what it is to a sighted reader.
                if name == "Table" {
                    if let Some(Arg::Named(_, cap)) = ui
                        .args
                        .iter()
                        .find(|a| matches!(a, Arg::Named(k, _) if k == "caption"))
                    {
                        let v = self.emit_expr(cap);
                        children_arr.push(format!(
                            "WF.h(\"caption\", {{ className: \"wf-visually-hidden\" }}, {})",
                            v
                        ));
                    }
                }

                // Inner text content
                if let Some(text) = &inner_text {
                    if self.is_reactive(text) {
                        children_arr.push(format!("() => {}", text));
                    } else {
                        children_arr.push(text.clone());
                    }
                }

                // A select's value only takes once its options exist, so a
                // slot inside a `Select` is passed to `h()` with the element
                // rather than appended afterwards.
                let slot_in_select = name == "Select"
                    && ui.children.iter().any(|c| {
                        matches!(&c.kind, StatementKind::UIElement(u)
                            if matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Children"))
                    });
                if slot_in_select {
                    children_arr
                        .push("(typeof _children === 'function' ? _children() : null)".to_string());
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
                let is_button = matches!(name.as_str(), "Button" | "IconButton");

                {
                    let was_in_thead = self.in_thead;
                    if name == "Thead" {
                        self.in_thead = true;
                    } else if name == "Tbody" {
                        self.in_thead = false;
                    }
                    for child in &ui.children {
                        // A button's action statements are its click handler
                        // (see above), not content.
                        if is_button && ui.events.is_empty() && is_action_statement(child) {
                            continue;
                        }
                        if slot_in_select
                            && matches!(&child.kind, StatementKind::UIElement(u)
                                if matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Children"))
                        {
                            continue;
                        }
                        self.emit_statement_dom(child, &var);
                    }
                    self.in_thead = was_in_thead;
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

                for handler in &ui.events {
                    let body = self.emit_event_body(&handler.body);
                    self.emit_line(&format!(
                        "{}.addEventListener(\"{}\", (event) => {{ {} }});",
                        var, handler.event, body
                    ));
                }

                if let Some(href) = &link_to {
                    if name == "Link" {
                        self.emit_line(&format!(
                            "WF.activeLink({}, {}, {});",
                            var, href, link_prefix
                        ));
                    }
                }

                self.emit_style_and_transition(&var, ui);

                if is_field {
                    let mut opts = Vec::new();
                    for key in ["label", "hint", "error"] {
                        if let Some(Arg::Named(_, val)) = ui
                            .args
                            .iter()
                            .find(|a| matches!(a, Arg::Named(k, _) if k == key))
                        {
                            let v = self.emit_expr(val);
                            if self.is_reactive(&v) {
                                opts.push(format!("{key}: () => {v}"));
                            } else {
                                opts.push(format!("{key}: {v}"));
                            }
                        }
                    }
                    self.emit_line(&format!(
                        "{}.appendChild(WF.field({}, {{ {} }}));",
                        parent,
                        var,
                        opts.join(", ")
                    ));
                } else {
                    self.emit_line(&format!("{}.appendChild({});", parent, var));
                }
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

                // Hyphenated and unknown named arguments are attributes here
                // too (`Card.Header(id: …)`, `List.Item(aria-current: …)`).
                let mut attrs = vec![format!("className: \"{}\"", class)];
                for arg in &ui.args {
                    if let Arg::Named(k, v) = arg {
                        let value = self.emit_expr(v);
                        let key = if k.contains('-') {
                            format!("\"{}\"", k)
                        } else {
                            k.clone()
                        };
                        if self.is_reactive(&value) {
                            attrs.push(format!("{}: () => {}", key, value));
                        } else {
                            attrs.push(format!("{}: {}", key, value));
                        }
                    }
                }
                if let Some(entry) = self.wf_node_entry(ui) {
                    attrs.push(entry);
                }
                self.emit_line(&format!(
                    "const {} = WF.h(\"{}\", {{ {} }});",
                    var,
                    tag,
                    attrs.join(", ")
                ));
                for child in &ui.children {
                    self.emit_statement_dom(child, &var);
                }
                for handler in &ui.events {
                    let body = self.emit_event_body(&handler.body);
                    self.emit_line(&format!(
                        "{}.addEventListener(\"{}\", (event) => {{ {} }});",
                        var, handler.event, body
                    ));
                }
                // A sub-component used to drop its style block and its
                // handlers; every other element honours both.
                self.emit_style_and_transition(&var, ui);
                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }

            ComponentRef::UserDefined(name) => {
                let args_obj = self.emit_component_args(&ui.args);
                // A block of nothing but actions is a click handler, as it is
                // on a Button — `Save(label: "x") { save() }` — rather than a
                // slot filled with statements that render nothing.
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
                    });
                if ui.children.is_empty() || is_action_shorthand {
                    self.emit_line(&format!(
                        "const {} = Component_{}({});",
                        var, name, args_obj
                    ));
                } else {
                    // The block is compiled here, in the caller's scope, so it
                    // reads the caller's state and loop bindings; the component
                    // only decides where it lands.
                    self.emit_line(&format!(
                        "const {} = Component_{}({}, () => {{",
                        var, name, args_obj
                    ));
                    self.indent += 1;
                    self.emit_line("const _cf = document.createDocumentFragment();");
                    for child in &ui.children {
                        self.emit_statement_dom(child, "_cf");
                    }
                    self.emit_line("return _cf;");
                    self.indent -= 1;
                    self.emit_line("});");
                }
                // Handlers written on the call attach to the component's root
                // element, so a styled button component is clickable where it
                // is used. They used to be dropped.
                if is_action_shorthand {
                    let body = self.emit_statements_inline(&ui.children);
                    self.emit_line(&format!(
                        "WF.onRoot({}, \"click\", (event) => {{ {} }});",
                        var, body
                    ));
                }
                for handler in &ui.events {
                    let body = self.emit_event_body(&handler.body);
                    self.emit_line(&format!(
                        "WF.onRoot({}, \"{}\", (event) => {{ {} }});",
                        var, handler.event, body
                    ));
                }
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
            // A literal or a token is the same on every instance, so it lives
            // in styles.css under a class named by the block's content (see
            // `scoped_css`), along with the block's pseudo-state and media
            // rules; the element only carries the class. Only a value that
            // reads state is assigned here, inside an effect, so
            // `width: "{pct}%"` follows `pct` rather than painting once.
            for prop in &style.properties {
                if crate::codegen::scoped_css::static_declaration(prop).is_some() {
                    continue;
                }
                let (css_prop, val) = self.emit_style_decl(prop);
                // A custom property has no camel-cased field; it is set by name.
                let assign = if prop.name.starts_with("--") {
                    format!("{}.style.setProperty(\"{}\", {});", var, prop.name, val)
                } else {
                    format!("{}.style.{} = {};", var, css_prop, val)
                };
                if self.is_reactive(&val) {
                    self.emit_line(&format!("WF.effect(() => {{ {} }});", assign));
                } else {
                    self.emit_line(&assign);
                }
            }
            if let Some(class) = crate::codegen::scoped_css::scoped_class(style) {
                self.emit_line(&format!("{}.classList.add(\"{}\");", var, class));
            }
        }

        // `class:` names rules in the author's own stylesheet. A literal is
        // added once, beside the engine's classes; a value that reads state
        // is followed, and the classes it named last time are taken off.
        match crate::codegen::builtin::class_arg(&ui.args) {
            Some(Some(_)) => {
                let classes = crate::codegen::builtin::author_classes(&ui.args);
                if !classes.is_empty() {
                    let list = classes
                        .iter()
                        .map(|c| format!("\"{}\"", c.replace('\\', "\\\\").replace('"', "\\\"")))
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.emit_line(&format!("{}.classList.add({});", var, list));
                }
            }
            Some(None) => {
                let expr = ui
                    .args
                    .iter()
                    .find_map(|a| match a {
                        Arg::Named(k, v) if k == "class" => Some(self.emit_expr(v)),
                        _ => None,
                    })
                    .unwrap_or_default();
                self.emit_line(&format!("WF.classes({}, () => {});", var, expr));
            }
            None => {}
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
                if self.is_reactive(&cv) {
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

        // The items are `li`s (the generic `Name.Item`), so their container is
        // a list; `role="menu"` gives it the menu semantics.
        let items_var = self.fresh_var();
        let items_class = format!("{}__items", class);
        self.emit_line(&format!(
            "const {} = WF.h(\"ul\", {{ className: \"{}\", id: \"{}\", role: \"menu\" }});",
            items_var, items_class, items_id
        ));

        for child in &ui.children {
            self.emit_statement_dom(child, &items_var);
        }

        self.emit_line(&format!("{}.appendChild({});", var, items_var));

        // Close on click outside and on Escape with focus returned to the
        // trigger; arrow keys move between the items, which are menuitems.
        self.emit_line(&format!(
            "WF.menu({}, {}, {}, {});",
            var, trigger_var, items_var, open_var
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
                                // In the SPA the item navigates in place, as a
                                // Link does; it used to be a plain href, so
                                // every rail click reloaded the whole app.
                                let click = if self.ssg_mode {
                                    String::new()
                                } else {
                                    format!(
                                        ", \"on:click\": (e) => {{ e.preventDefault(); WF.navigate({}); }}",
                                        href
                                    )
                                };
                                self.emit_line(&format!(
                                        "const {} = WF.h(\"a\", {{ className: \"wf-sidebar__item\", href: {} {}{}{} }});",
                                        item_var, bp, href, click, self.wf_node_inline(ui_child)
                                    ));
                                let prefix = ui_child.args.iter().any(|a| {
                                    matches!(a, Arg::Named(k, Expr::StringLiteral(v)) if k == "active" && v == "prefix")
                                });
                                self.emit_line(&format!(
                                    "WF.activeLink({}, {}, {});",
                                    item_var, href, prefix
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

        let tip_id = format!("wf-tip-{}", var.trim_start_matches("_e"));
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Tooltip", ui),
            self.wf_node_inline(ui)
        ));

        // Render children (the trigger element)
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }

        let tip_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"span\", {{ className: \"wf-tooltip__text\", role: \"tooltip\", id: \"{}\" }}, {});",
            tip_var, tip_id, text
        ));
        self.emit_line(&format!("{}.appendChild({});", var, tip_var));
        // The runtime points the trigger at the tip and makes it reachable
        // and dismissible from the keyboard.
        self.emit_line(&format!("WF.tooltip({}, {});", var, tip_var));
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

        // The slides are the structure; everything else — the controls, the
        // rotation, the ARIA — is the runtime's `carousel`, so one place
        // holds the behaviour.
        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                if matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == "Carousel" && s == "Slide")
                {
                    let slide_var = self.fresh_var();
                    // A slide's own `label:` names it to a screen reader in
                    // place of "n of N".
                    let label = ui_child.args.iter().find_map(|a| match a {
                        Arg::Named(k, v) if k == "label" => Some(self.emit_expr(v)),
                        _ => None,
                    });
                    let label_attr = label
                        .map(|l| format!(", \"aria-label\": {l}"))
                        .unwrap_or_default();
                    self.emit_line(&format!(
                        "const {} = WF.h(\"div\", {{ className: \"wf-carousel__slide\"{}{} }});",
                        slide_var,
                        label_attr,
                        self.wf_node_inline(ui_child)
                    ));
                    for c in &ui_child.children {
                        self.emit_statement_dom(c, &slide_var);
                    }
                    self.emit_line(&format!("{}.appendChild({});", track_var, slide_var));
                } else {
                    self.emit_statement_dom(child, &track_var);
                }
            } else {
                self.emit_statement_dom(child, &track_var);
            }
        }

        self.emit_line(&format!("{}.appendChild({});", var, track_var));

        let autoplay = ui.args.iter().any(|a| {
            matches!(a, Arg::Named(k, v) if k == "autoplay" && matches!(v, Expr::BoolLiteral(true)))
        });
        let interval = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, Expr::NumberLiteral(n)) if k == "interval" => Some(*n as u32),
                _ => None,
            })
            .unwrap_or(5000);
        let label = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, v) if k == "label" => Some(self.emit_expr(v)),
                _ => None,
            })
            .unwrap_or_else(|| "null".to_string());
        self.emit_line(&format!(
            "WF.carousel({var}, {{ autoplay: {autoplay}, interval: {interval}, label: {label} }});"
        ));

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

        // Every other named argument is an attribute, as on a Button: `type`,
        // `disabled`, `aria-haspopup`, `data-variant`. A value that reads
        // state is a thunk the runtime keeps in step with it.
        for arg in &ui.args {
            if let Arg::Named(k, v) = arg {
                if matches!(k.as_str(), "icon" | "label") {
                    continue;
                }
                let value = self.emit_expr(v);
                let key = if k.contains('-') {
                    format!("\"{}\"", k)
                } else {
                    k.clone()
                };
                if self.is_reactive(&value) {
                    btn_attrs.push_str(&format!(", {}: () => {}", key, value));
                } else {
                    btn_attrs.push_str(&format!(", {}: {}", key, value));
                }
            }
        }

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
            // An author's own on:input runs after the binding has written the
            // state, in one handler: two "on:input" keys in one attribute
            // object used to leave only the second.
            let own_input = ui
                .events
                .iter()
                .find(|h| h.event == "input")
                .map(|h| self.emit_event_body(&h.body))
                .unwrap_or_default();
            input_attrs.push_str(&format!(
                ", value: () => _{}(), \"on:input\": (event) => {{ _{}.set(Number(event.target.value)); {} }}",
                state, state, own_input
            ));
        }
        // ARIA and data attributes reach the range input itself, which is the
        // control assistive technology reads. An author who announces the
        // value with aria-valuetext is showing it in their own words too, so
        // the raw number is not repeated beside the track.
        let mut announces_value = false;
        for arg in &ui.args {
            if let Arg::Named(k, v) = arg {
                if k.contains('-') {
                    let value = self.emit_expr(v);
                    if k == "aria-valuetext" {
                        announces_value = true;
                    }
                    if self.is_reactive(&value) {
                        input_attrs.push_str(&format!(", \"{}\": () => {}", k, value));
                    } else {
                        input_attrs.push_str(&format!(", \"{}\": {}", k, value));
                    }
                }
            }
        }
        for handler in &ui.events {
            if handler.event == "input" && bind_var.is_some() {
                continue; // merged into the binding above
            }
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
        if bind_var.is_some() && announces_value {
            // The author shows it.
        } else if let Some(state) = &bind_var {
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

        // Else branch. An `else if` chain nests: the else of this condition
        // is another condRender over the next one, and the final `else` body
        // rides along to the innermost. (Checking `else_body` first used to
        // skip every `else if` whenever a final `else` was present.)
        if !if_stmt.else_if_branches.is_empty() {
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
                animate_span: None,
                then_body: if_stmt.else_if_branches[0].1.clone(),
                else_if_branches: if_stmt.else_if_branches[1..].to_vec(),
                else_body: if_stmt.else_body.clone(),
            };
            self.emit_if_dom(&elif, &elif_var);
            self.emit_line(&format!("return {};", elif_var));
            self.indent -= 1;
            self.emit_line("},");
        } else if let Some(else_body) = &if_stmt.else_body {
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
                if self.current_props.contains(name) {
                    return format!("_p.{}", name);
                }
                if self.stores.contains(name)
                    || self.loop_bindings.contains(name)
                    || self.lambda_params.borrow().contains(name)
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
                // Parenthesised, so a map literal is an object wherever it
                // lands — as an arrow function's body a bare `{` is a block.
                format!("({{ {} }})", entries_str.join(", "))
            }
            Expr::Lambda(param, body) => {
                // `param` is one name or several joined by ", ".
                let names: Vec<String> = param.split(", ").map(|n| n.to_string()).collect();
                let count = names.len();
                self.lambda_params.borrow_mut().extend(names);
                let body_str = self.emit_expr(body);
                for _ in 0..count {
                    self.lambda_params.borrow_mut().pop();
                }
                format!("(({}) => {})", param, body_str)
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
                    let key = if name.contains('-') {
                        format!("\"{}\"", name)
                    } else {
                        name.clone()
                    };
                    let value = self.emit_expr(expr);
                    // A value that reads state is handed over as a getter, so
                    // the component tracks it instead of copying it once.
                    if self.is_reactive(&value) {
                        parts.push(format!("get {}() {{ return {}; }}", key, value));
                    } else {
                        parts.push(format!("{}: {}", key, value));
                    }
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

impl JsCodegen {
    /// Whether a compiled expression reads reactive state.
    ///
    /// [`is_reactive_expr`] recognises page signals (`_x()`) and i18n; a store
    /// field compiles to `Store.field`, a getter over a signal, which that
    /// textual check cannot see. Anything deciding between a one-shot value
    /// and an effect asks here.
    fn is_reactive(&self, expr_str: &str) -> bool {
        is_reactive_expr(expr_str)
            || expr_str.contains("_p.")
            || self
                .stores
                .iter()
                .any(|s| expr_str.contains(&format!("{s}.")))
    }
}

/// Whether a statement does something rather than shows something: the kind
/// a Button's block turns into its click handler.
fn is_action_statement(stmt: &Statement) -> bool {
    matches!(
        &stmt.kind,
        StatementKind::Assignment(_)
            | StatementKind::MethodCall(_)
            | StatementKind::Navigate(_)
            | StatementKind::ExprStatement(_)
    )
}

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

    fn compile(src: &str) -> String {
        let program = crate::syntax::parse_source(src, "<t>").expect("parse");
        JsCodegen::new().generate(&program)
    }

    /// `WF.listRender` hands the body its item as a plain callback parameter, so
    /// every reference to it must stay plain.
    ///
    /// The identifier path treated anything that was not a prop or a store as
    /// state, so a loop over `tasks` bound `task` and then read `_task()` —
    /// `ReferenceError` on the first non-empty list. `wf init -t spa` shipped it.
    #[test]
    fn a_component_receives_its_callers_block_as_children() {
        let out = compile(
            r#"
            Component Panel (title: String) {
                Card { Heading(title, h3) children }
            }
            Page P (path: "/") {
                state n = 1
                Panel(title: "Keys") { Text("count {n}") }
                Panel(title: "Empty")
            }
            "#,
        );
        assert!(
            out.contains("function Component_Panel(_p, _children)"),
            "{out}"
        );
        assert!(
            out.contains("Component_Panel({ title: \"Keys\" }, () => {"),
            "{out}"
        );
        // The block is compiled in the caller's scope.
        assert!(out.contains("_n()"), "{out}");
        assert!(
            out.contains("Component_Panel({ title: \"Empty\" });"),
            "{out}"
        );
    }

    #[test]
    fn a_store_derived_may_build_on_derived_values_actions_and_if_expressions() {
        let out = compile(
            r#"
            Store Pricing {
                state seats = 6
                state annual = true
                derived rate = if annual { 14 } else { 18 }
                derived cost = Math.round(seats * rate)
                derived label = "{cost} units"
                derived share = pctOf(cost)
                action pctOf(part: Number) {
                    if cost == 0 { return 0 }
                    return Math.round(part / cost * 100)
                }
                action bump() { seats = seats + 1  log(share) }
            }
            Page P (path: "/") { use Pricing  Text("{Pricing.label}") }
            "#,
        );
        assert!(
            out.contains("rate: (store) => (store.annual ? 14 : 18)"),
            "{out}"
        );
        assert!(
            out.contains("cost: (store) => Math.round((store.seats * store.rate))"),
            "{out}"
        );
        assert!(
            out.contains("label: (store) => `${store.cost} units`"),
            "{out}"
        );
        assert!(
            out.contains("share: (store) => store.pctOf(store.cost)"),
            "{out}"
        );
        assert!(
            out.contains("return Math.round(((part / store.cost) * 100));"),
            "{out}"
        );
        assert!(!out.contains("_part()"), "{out}");
    }

    #[test]
    fn an_input_keeps_its_binding_beside_the_authors_input_handler() {
        let out = compile(
            r#"
            Store S { state q = ""  action set(v: String) { q = v } }
            Page P (path: "/") {
                use S
                state email = ""
                Input(email, bind: email, placeholder: "e") { on:input { S.set(email) } }
                Form { Text("f")  on:submit { S.set("sent") } }
            }
            "#,
        );
        assert!(
            out.contains("\"on:input\": (e) => _email.set(e.target.value)"),
            "{out}"
        );
        assert!(
            out.contains(".addEventListener(\"input\", (event) => { S.set(_email()); });"),
            "{out}"
        );
        assert!(
            out.contains("\"on:submit\": (e) => e.preventDefault()"),
            "{out}"
        );
        assert!(
            out.contains(".addEventListener(\"submit\", (event) => { S.set(\"sent\"); });"),
            "{out}"
        );
    }

    #[test]
    fn a_buttons_block_may_mix_content_and_actions() {
        let out = compile(
            r#"
            Page P (path: "/") {
                state open = false
                Button("", aria-expanded: open) {
                    Text("Details")
                    Icon("chevron-down")
                    open = !open
                }
            }
            "#,
        );
        assert!(
            out.contains("\"on:click\": (e) => { _open.set(!_open()); }"),
            "{out}"
        );
        // The assignment is not executed at render time.
        assert!(!out.contains("\n  _open.set(!_open());"), "{out}");
        assert!(out.contains("\"Details\""), "{out}");
    }

    #[test]
    fn a_sub_component_keeps_its_style_block_attributes_and_handlers() {
        let out = compile(
            r#"
            Page P (path: "/") {
                state n = 0
                List { List.Item(id: "first", aria-current: "true") { style { padding: "0" hover { color: "red" } } on:click { n = n + 1 } Text("a") } }
            }
            "#,
        );
        assert!(out.contains("WF.h(\"li\", { className: \"wf-list__item\", id: \"first\", \"aria-current\": \"true\" })"), "{out}");
        // `padding: "0"` is a literal: hoisted into the scoped class, not inline.
        assert!(!out.contains(".style.padding"), "{out}");
        assert!(out.contains(".classList.add(\"wf-s"), "{out}");
        assert!(
            out.contains(".addEventListener(\"click\", (event) => { _n.set((_n() + 1)); });"),
            "{out}"
        );
    }

    #[test]
    fn a_map_literal_returned_from_a_lambda_is_an_object_not_a_block() {
        let out = compile(
            r#"
            Store S {
                state items = [1, 2]
                derived pairs = items.map(i => { n: i, twice: i * 2 })
                derived counts = { all: items.length }
            }
            Page P (path: "/") { use S  Text("x") }
            "#,
        );
        assert!(
            out.contains("items.map(((i) => ({ n: i, twice: (i * 2) })))"),
            "{out}"
        );
        assert!(
            out.contains("counts: (store) => ({ all: store.items.length })"),
            "{out}"
        );
    }

    #[test]
    fn a_store_action_declares_its_locals() {
        let out = compile(
            r#"
            Store S {
                state secret = ""
                action headers() {
                    h = {}
                    h["Authorization"] = "Bearer " + secret
                    h = h
                    return h
                }
            }
            Page P (path: "/") { use S  Text("x") }
            "#,
        );
        assert!(out.contains("let h = ({  });"), "{out}");
        assert!(
            out.contains("h[\"Authorization\"] = (\"Bearer \" + store.secret);"),
            "{out}"
        );
        assert_eq!(out.matches("let h = ").count(), 1, "{out}");
    }

    #[test]
    fn an_action_parameter_shadows_a_store_member_of_the_same_name() {
        let out = compile(
            r#"
            Store S {
                state at = 0
                action step(n: Number) { return n + 1 }
                action move(step: Number) { at = at + step }
            }
            Page P (path: "/") { use S  Text("x") }
            "#,
        );
        assert!(out.contains("store.at = (store.at + step);"), "{out}");
        assert!(!out.contains("store.step"), "{out}");
    }

    #[test]
    fn a_known_attribute_that_reads_state_follows_it() {
        let out = compile(
            r#"
            Store S { state busy = false  derived hint = if busy { "wait" } else { "type" } }
            Page P (path: "/") {
                use S
                Input(placeholder: S.hint, disabled: S.busy)
                Input(placeholder: "fixed")
            }
            "#,
        );
        assert!(out.contains("placeholder: () => S.hint"), "{out}");
        assert!(out.contains("disabled: () => S.busy"), "{out}");
        assert!(out.contains("placeholder: \"fixed\""), "{out}");
    }

    #[test]
    fn a_slot_inside_a_select_is_passed_with_the_element() {
        let out = compile(
            r#"
            Component Picker (value: String) { Select(value: value) { children } }
            Page P (path: "/") { Picker(value: "b") { Option("a", "A")  Option("b", "B") } }
            "#,
        );
        assert!(
            out.contains("WF.h(\"select\", { className: \"wf-select\", value: () => _p.value }, (typeof _children === 'function' ? _children() : null))"),
            "{out}"
        );
        assert!(!out.contains("appendChild(_children())"), "{out}");
    }

    #[test]
    fn an_else_if_chain_with_a_final_else_keeps_every_branch() {
        let out = compile(
            r#"
            Page P (path: "/") {
                state v = "no"
                if v == "yes" { Text("Y") } else if v == "no" { Text("N") } else { Text("other") }
            }
            "#,
        );
        assert!(out.contains("(_v() === \"yes\")"), "{out}");
        assert!(out.contains("(_v() === \"no\")"), "{out}");
        assert!(out.contains("\"other\""), "{out}");
    }

    #[test]
    fn a_sliders_own_input_handler_runs_after_the_binding() {
        let out = compile(
            r#"
            Store S { state n = 0  action set(v: Number) { n = v } }
            Page P (path: "/") {
                use S
                state req = 4
                Slider(bind: req, min: 0, max: 8, step: 1, aria-label: "Requests") { on:input { S.set(req) } }
            }
            "#,
        );
        assert_eq!(out.matches("\"on:input\"").count(), 1, "{out}");
        assert!(
            out.contains("_req.set(Number(event.target.value)); S.set(_req());"),
            "{out}"
        );
    }

    #[test]
    fn a_slider_carries_aria_attributes_and_defers_to_aria_valuetext() {
        let out = compile(
            r#"
            Page P (path: "/") {
                state req = 4
                derived shown = "5M"
                Slider(bind: req, min: 0, max: 8, step: 1, aria-labelledby: "d-req", aria-valuetext: shown)
                Slider(bind: req, min: 0, max: 8, step: 1, aria-label: "Seats")
            }
            "#,
        );
        assert!(out.contains("\"aria-labelledby\": \"d-req\""), "{out}");
        assert!(out.contains("\"aria-valuetext\": () => _shown()"), "{out}");
        assert_eq!(out.matches("wf-slider__value").count(), 1, "{out}");
    }

    #[test]
    fn a_prop_that_reads_state_is_passed_as_a_getter_and_read_live() {
        let out = compile(
            r#"
            Component Chip (label: String, pressed: Bool = false) {
                derived bg = if pressed { "a" } else { "b" }
                Button(label, aria-pressed: pressed) { style { background: bg } }
            }
            Page P (path: "/") {
                state on = true
                Chip(label: "Errors", pressed: on) { on = !on }
            }
            "#,
        );
        assert!(
            out.contains("Component_Chip({ label: \"Errors\", get pressed() { return _on(); } })"),
            "{out}"
        );
        assert!(
            out.contains("WF.computed(() => (_p.pressed ? \"a\" : \"b\"))"),
            "{out}"
        );
        assert!(out.contains("\"aria-pressed\": () => _p.pressed"), "{out}");
    }

    #[test]
    fn a_declared_prop_default_reaches_the_spa() {
        let out = compile(
            r#"
            Component Badge (label: String, tone: String = "neutral", dot: Bool = true) { Text(label) }
            Page P (path: "/") { Badge(label: "x") }
            "#,
        );
        assert!(
            out.contains("_p = WF.props(_p, { tone: \"neutral\", dot: true });"),
            "{out}"
        );
    }

    #[test]
    fn a_custom_property_is_set_by_name_and_follows_state() {
        let out = compile(
            r#"
            Page P (path: "/") {
                state tone = "red"
                Card { style { --hover-bg: tone  --edge: "1px"  hover { background: "var(--hover-bg)" } } }
            }
            "#,
        );
        assert!(
            out.contains(".style.setProperty(\"--hover-bg\", _tone()); });"),
            "{out}"
        );
        // The literal custom property is hoisted with the rest of the block.
        assert!(!out.contains("--edge"), "{out}");
        assert!(out.contains(".classList.add(\"wf-s"), "{out}");
    }

    #[test]
    fn handlers_on_a_component_call_attach_to_its_root() {
        let out = compile(
            r#"
            Component Save (label: String) { Button(label) }
            Page P (path: "/") {
                state n = 0
                Save(label: "Go") { n = n + 1 }
                Save(label: "Hover") { on:mouseenter { n = 9 } }
            }
            "#,
        );
        assert!(out.contains("WF.onRoot(_e"), "{out}");
        assert!(
            out.contains(", \"click\", (event) => { _n.set((_n() + 1)); });"),
            "{out}"
        );
        assert!(
            out.contains(", \"mouseenter\", (event) => { _n.set(9); });"),
            "{out}"
        );
        // The action block is not mistaken for slot children.
        assert!(
            !out.contains("Component_Save({ label: \"Go\" }, () =>"),
            "{out}"
        );
    }

    #[test]
    fn every_route_carries_its_pages_title() {
        let out = compile(
            r#"
            Page Home (path: "/", title: "Home") { Text("h") }
            Page Docs (path: "/docs", title: "Routing \"rules\"") { Text("d") }
            App { Router { Route(path: "/", page: Home) Route(path: "/docs", page: Docs) } }
            "#,
        );
        assert!(
            out.contains(
                "{ path: \"/\", title: \"Home\", render: (params) => Page_Home(params) },"
            ),
            "{out}"
        );
        assert!(
            out.contains("title: \"Routing \\\"rules\\\"\", render: (params) => Page_Docs(params)"),
            "{out}"
        );
    }

    #[test]
    fn hyphenated_named_arguments_become_attributes_reactive_when_they_read_state() {
        let out = compile(
            r#"
            Page P (path: "/") {
                state on = true
                Button("Errors", aria-pressed: on, data-tone: "danger")
                Text(a - b)
            }
            "#,
        );
        assert!(out.contains("\"aria-pressed\": () => _on()"), "{out}");
        assert!(out.contains("\"data-tone\": \"danger\""), "{out}");
        // `a - b` is still an expression, not a named argument.
        assert!(out.contains("(_a() - _b())"), "{out}");
    }

    #[test]
    fn a_style_value_that_reads_state_follows_it() {
        let out = compile(
            r##"
            Store S { state tone = "#fff" }
            Page P (path: "/") {
                use S
                state pct = 40
                Card {
                    style {
                        width: "{pct}%"
                        background: S.tone
                        padding: "1rem"
                    }
                }
            }
            "##,
        );
        // Element numbering is process-wide, so match on the tail of each line.
        assert!(out.contains(".style.width = `${_pct()}%`; });"), "{out}");
        assert!(out.contains("WF.effect(() => { _e"), "{out}");
        assert!(out.contains(".style.background = S.tone; });"), "{out}");
        // The literal is in the stylesheet, under the element's class.
        assert!(!out.contains(".style.padding"), "{out}");
        assert!(out.contains(".classList.add(\"wf-s"), "{out}");
    }

    #[test]
    fn a_value_that_reads_state_is_reactive() {
        let out = compile(
            r#"
            Store S { state pct = 5 }
            Page P (path: "/") {
                use S
                state done = 40
                Progress(value: done, max: 100)
                Progress(value: S.pct, max: 100)
                Progress(value: 75, max: 100)
            }
            "#,
        );
        assert!(out.contains("value: () => _done()"), "{out}");
        assert!(out.contains("value: () => S.pct"), "{out}");
        assert!(out.contains("value: 75"), "{out}");
    }

    #[test]
    fn a_lambda_parameter_is_a_plain_binding_not_a_signal() {
        let out = compile(
            r#"
            Page P (path: "/") {
                state items = []
                state limit = 3
                Text("{items.filter(x => x.done && x.n < limit).length}")
            }
            "#,
        );
        assert!(
            out.contains("((x) => (x.done && (x.n < _limit())))"),
            "{out}"
        );
        assert!(!out.contains("_x()"), "{out}");
    }

    #[test]
    fn the_app_wrapper_around_the_router_gets_layout_classes() {
        let out = compile(
            r#"
            Page Home (path: "/") { Text("hi") }
            App {
                Row(gap: lg, align: center) {
                    Router { Route(path: "/", page: Home) }
                }
            }
            "#,
        );
        assert!(out.contains("wf-row wf-gap--lg wf-align--center"), "{out}");
        assert!(!out.contains("--gap-lg"), "{out}");
    }

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
