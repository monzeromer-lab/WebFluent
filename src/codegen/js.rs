use std::collections::HashMap;
use crate::parser::ast::*;
use crate::runtime;

pub struct JsCodegen {
    output: String,
    indent: usize,
    /// Track user-defined component names so we can reference them
    components: Vec<String>,
    /// Track store names
    stores: Vec<String>,
    /// Track current component/page prop names (not signals)
    current_props: Vec<String>,
    /// i18n: default locale and translations (locale -> key -> value)
    i18n_default_locale: Option<String>,
    i18n_translations: HashMap<String, HashMap<String, String>>,
    /// SSG mode: emit hydration instead of full mount
    ssg_mode: bool,
    /// Base path for deployment (e.g., "/WebFluent")
    base_path: String,
}

impl JsCodegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            components: Vec::new(),
            stores: Vec::new(),
            current_props: Vec::new(),
            i18n_default_locale: None,
            i18n_translations: HashMap::new(),
            ssg_mode: false,
            base_path: String::new(),
        }
    }

    pub fn set_i18n(&mut self, default_locale: String, translations: HashMap<String, HashMap<String, String>>) {
        self.i18n_default_locale = Some(default_locale);
        self.i18n_translations = translations;
    }

    pub fn set_ssg(&mut self, enabled: bool) {
        self.ssg_mode = enabled;
    }

    pub fn set_base_path(&mut self, path: String) {
        self.base_path = path;
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
        let has_app = program.declarations.iter().any(|d| matches!(d, Declaration::App(_)));
        if !has_app {
            let pages: Vec<&PageDecl> = program.declarations.iter().filter_map(|d| {
                if let Declaration::Page(p) = d { Some(p) } else { None }
            }).collect();

            if pages.len() == 1 {
                let mount_fn = if self.ssg_mode { "hydrate" } else { "mount" };
                self.emit_line(&format!(
                    "WF.{}(() => Page_{}({{}}), document.getElementById('app'));",
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
                self.emit_line("const container = document.getElementById('app');");
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
        let store_state_names: Vec<String> = store.body.iter().filter_map(|s| {
            if let Statement::State(st) = s { Some(st.name.clone()) } else { None }
        }).collect();

        // Collect state
        let states: Vec<&StateDecl> = store.body.iter().filter_map(|s| {
            if let Statement::State(st) = s { Some(st) } else { None }
        }).collect();

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
        let derived: Vec<&DerivedDecl> = store.body.iter().filter_map(|s| {
            if let Statement::Derived(d) = s { Some(d) } else { None }
        }).collect();

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
        let actions: Vec<&ActionDecl> = store.body.iter().filter_map(|s| {
            if let Statement::Action(a) = s { Some(a) } else { None }
        }).collect();

        if !actions.is_empty() {
            self.emit_line("actions: {");
            self.indent += 1;
            for a in &actions {
                let params: Vec<String> = a.params.iter().map(|p| p.name.clone()).collect();
                self.emit_line(&format!("{}: (store{}) => {{", a.name,
                    if params.is_empty() { String::new() } else { format!(", {}", params.join(", ")) }
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
                    format!("{}", name)
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
                    BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*",
                    BinOp::Div => "/", BinOp::Mod => "%", BinOp::Eq => "===",
                    BinOp::Neq => "!==", BinOp::Lt => "<", BinOp::Gt => ">",
                    BinOp::Lte => "<=", BinOp::Gte => ">=", BinOp::And => "&&",
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
                let args_str: Vec<String> = args.iter().map(|a| self.emit_store_expr(a, store_states)).collect();
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
                let items_str: Vec<String> = items.iter().map(|i| self.emit_store_expr(i, store_states)).collect();
                format!("[{}]", items_str.join(", "))
            }
            Expr::MapLiteral(entries) => {
                let entries_str: Vec<String> = entries.iter().map(|(k, v)| {
                    format!("{}: {}", k, self.emit_store_expr(v, store_states))
                }).collect();
                format!("{{ {} }}", entries_str.join(", "))
            }
            // For other expr types, fall back to the regular emitter
            _ => self.emit_expr(expr),
        }
    }

    fn emit_store_statement(&mut self, stmt: &Statement, store_states: &[String], action_params: &[String]) {
        match stmt {
            Statement::Assignment(a) => {
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
            Statement::State(s) => {
                let val = self.emit_store_expr(&s.value, store_states);
                self.emit_line(&format!("const {} = {};", s.name, val));
            }
            Statement::Navigate(expr) => {
                let path = self.emit_store_expr(expr, store_states);
                self.emit_line(&format!("WF.navigate({});", path));
            }
            Statement::ExprStatement(expr) => {
                let val = self.emit_store_expr(expr, store_states);
                self.emit_line(&format!("{};", val));
            }
            Statement::If(if_stmt) => {
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

        let default_locale = self.i18n_default_locale.clone().unwrap_or_else(|| "en".to_string());
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

        self.emit_line(&format!("function Component_{}({}) {{", comp.name, destructure));
        self.indent += 1;

        // Emit state declarations first
        for stmt in &comp.body {
            if let Statement::State(s) = stmt {
                let val = self.emit_expr(&s.value);
                self.emit_line(&format!("const _{name} = WF.signal({val});", name = s.name, val = val));
            }
        }

        // Build DOM
        self.emit_line("const _frag = document.createDocumentFragment();");

        for stmt in &comp.body {
            if !matches!(stmt, Statement::State(_)) {
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
            if let Statement::State(s) = stmt {
                let val = self.emit_expr(&s.value);
                self.emit_line(&format!("const _{name} = WF.signal({val});", name = s.name, val = val));
            }
        }

        // Build DOM
        self.emit_line("const _root = document.createDocumentFragment();");

        for stmt in &page.body {
            if !matches!(stmt, Statement::State(_)) {
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
        // In SSG mode, clear pre-rendered content before mounting the full app
        self.emit_line("_app.innerHTML = '';");

        // Find Router in body
        let mut has_router = false;
        let mut pre_router: Vec<&Statement> = Vec::new();
        let mut post_router: Vec<&Statement> = Vec::new();
        let mut router_stmts: Vec<&Statement> = Vec::new();

        let mut found_router = false;
        for stmt in &app.body {
            if let Statement::UIElement(ui) = stmt {
                if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                    has_router = true;
                    found_router = true;
                    router_stmts = ui.children.iter().collect();
                    continue;
                }
            }
            if found_router {
                post_router.push(stmt);
            } else {
                pre_router.push(stmt);
            }
        }

        // Emit pre-router elements (like Navbar)
        for stmt in &pre_router {
            self.emit_statement_dom(stmt, "_app");
        }

        if has_router {
            // Create router container
            self.emit_line("const _routerEl = document.createElement('div');");
            self.emit_line("_routerEl.id = 'wf-router';");
            self.emit_line("_routerEl.style.flex = '1';");
            self.emit_line("_app.appendChild(_routerEl);");

            // Collect routes
            self.emit_line("const _routes = [");
            self.indent += 1;
            for stmt in &router_stmts {
                if let Statement::UIElement(ui) = stmt {
                    if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Route") {
                        let mut path = String::new();
                        let mut page_name = String::new();
                        for arg in &ui.args {
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
                        // Remove quotes from path
                        let clean_path = path.trim_matches('"');
                        self.emit_line(&format!(
                            "{{ path: \"{}\", render: (params) => Page_{}(params) }},",
                            clean_path, page_name
                        ));
                    }
                }
            }
            self.indent -= 1;
            self.emit_line("];");
            self.emit_line("WF.createRouter(_routes, _routerEl);");
        }

        // Emit post-router elements (like Footer)
        for stmt in &post_router {
            self.emit_statement_dom(stmt, "_app");
        }

        self.indent -= 1;
        self.emit_line("})();");
    }

    // ─── DOM-building statement emitter ──────────────

    fn emit_statement_dom(&mut self, stmt: &Statement, parent: &str) {
        match stmt {
            Statement::UIElement(ui) => self.emit_ui_element(ui, parent),
            Statement::If(if_stmt) => self.emit_if_dom(if_stmt, parent),
            Statement::For(for_stmt) => self.emit_for_dom(for_stmt, parent),
            Statement::Show(show_stmt) => self.emit_show_dom(show_stmt, parent),
            Statement::Fetch(fetch) => self.emit_fetch_dom(fetch, parent),
            Statement::Use(_) => {} // Stores are global, no DOM output
            Statement::State(_) => {} // Already handled
            Statement::Derived(d) => {
                let val = self.emit_expr(&d.value);
                self.emit_line(&format!("const _{} = WF.computed(() => {});", d.name, val));
            }
            Statement::Effect(e) => {
                self.emit_line("WF.effect(() => {");
                self.indent += 1;
                for s in &e.body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                self.emit_line("});");
            }
            Statement::Action(a) => {
                let params: Vec<String> = a.params.iter().map(|p| p.name.clone()).collect();
                self.emit_line(&format!("function {}({}) {{", a.name, params.join(", ")));
                self.indent += 1;
                for s in &a.body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }
            Statement::EventHandler(handler) => {
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
                        self.emit_line(&format!("if (typeof _children === 'function') {}.appendChild(_children());", parent));
                        return;
                    }
                    "_StyleBlock" => return, // Style blocks handled via attrs
                    _ => {}
                }

                let (tag, class) = builtin_to_html(name);

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
                                            state_name, classes.join(" "), classes.join(" ")
                                        ));
                                    }
                                }
                                "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max" |
                                "step" | "accept" | "label" | "required" | "disabled" | "controls" |
                                "autoplay" | "role" => {
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
                                            if v_str == "center" { classes.push(format!("{}--center", class)); }
                                        }
                                        "justify" => {
                                            if v_str == "between" { classes.push(format!("{}--between", class)); }
                                            if v_str == "end" { classes.push(format!("{}--end", class)); }
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
                    match m.as_str() {
                        "text" | "email" | "password" | "number" | "search" | "tel" | "url" |
                        "date" | "time" | "datetime" | "color" => {
                            let t = if m == "datetime" { "datetime-local" } else { m.as_str() };
                            attrs.push(format!("type: \"{}\"", t));
                        }
                        "submit" | "reset" => {
                            attrs.push(format!("type: \"{}\"", m));
                        }
                        "required" => attrs.push("required: true".to_string()),
                        _ => {}
                    }
                }

                // Classes attr
                let class_str = classes.iter()
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
                    attrs.push(format!("\"on:{}\": (e) => {{ {} }}", handler.event, body));
                }

                // If element has a block with just statements (Button shorthand click)
                if ui.events.is_empty() && !ui.children.is_empty() {
                    // Check if children are all action-like statements (assignments, method calls)
                    let all_actions = ui.children.iter().all(|s| matches!(s,
                        Statement::Assignment(_) | Statement::MethodCall(_) |
                        Statement::Navigate(_) | Statement::ExprStatement(_)
                    ));

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

                let attrs_str = if attrs.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{ {} }}", attrs.join(", "))
                };

                // Special components with complex structure
                match name.as_str() {
                    "Modal" | "Dialog" => {
                        self.emit_modal_dialog(name, &var, &attrs_str, ui, parent);
                        return;
                    }
                    "Tabs" => {
                        self.emit_tabs(&var, ui, parent);
                        return;
                    }
                    "Switch" => {
                        self.emit_switch(&var, &attrs, ui, parent);
                        return;
                    }
                    "Checkbox" | "Radio" => {
                        self.emit_check_radio(name, &var, &attrs, ui, parent);
                        return;
                    }
                    "Dropdown" | "Menu" => {
                        self.emit_dropdown_menu(name, &var, &attrs, ui, parent);
                        return;
                    }
                    "Toast" => {
                        // Toast is imperative, not DOM-based
                        if let Some(text) = &inner_text {
                            let variant = ui.modifiers.first().map(|m| m.as_str()).unwrap_or("info");
                            self.emit_line(&format!("WF.showToast({}, \"{}\");", text, variant));
                        }
                        return;
                    }
                    "Spacer" => {
                        self.emit_line(&format!("const {} = WF.h(\"{}\", {});", var, tag, attrs_str));
                        self.emit_line(&format!("{}.appendChild({});", parent, var));
                        return;
                    }
                    _ => {}
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
                    self.emit_line(&format!("const {} = WF.h(\"{}\", {});", var, tag, attrs_str));
                } else if !children_arr.is_empty() && ui.children.is_empty() {
                    self.emit_line(&format!(
                        "const {} = WF.h(\"{}\", {}, {});",
                        var, tag, attrs_str, children_arr.join(", ")
                    ));
                } else {
                    let extra = if children_arr.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", children_arr.join(", "))
                    };
                    self.emit_line(&format!("const {} = WF.h(\"{}\", {}{});", var, tag, attrs_str, extra));
                }

                // Emit children
                let is_action_shorthand = ui.events.is_empty()
                    && !ui.children.is_empty()
                    && ui.children.iter().all(|s| matches!(s,
                        Statement::Assignment(_) | Statement::MethodCall(_) |
                        Statement::Navigate(_) | Statement::ExprStatement(_)
                    ))
                    && matches!(name.as_str(), "Button" | "IconButton");

                if !is_action_shorthand {
                    for child in &ui.children {
                        self.emit_statement_dom(child, &var);
                    }
                }

                // Apply style block
                if let Some(style) = &ui.style_block {
                    for prop in &style.properties {
                        let val = self.emit_expr(&prop.value);
                        let css_prop = to_camel_case(&prop.name);
                        self.emit_line(&format!("{}.style.{} = {};", var, css_prop, val));
                    }
                }

                // Apply transition block
                if let Some(transition) = &ui.transition_block {
                    let transitions: Vec<String> = transition.properties.iter().map(|p| {
                        let easing = p.easing.as_deref().map(|e| match e {
                            "ease" => "ease",
                            "linear" => "linear",
                            "easeIn" => "ease-in",
                            "easeOut" => "ease-out",
                            "easeInOut" => "ease-in-out",
                            "spring" => "cubic-bezier(0.175, 0.885, 0.32, 1.275)",
                            "bouncy" => "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
                            "smooth" => "cubic-bezier(0.4, 0, 0.2, 1)",
                            other => other,
                        }).unwrap_or("ease");
                        format!("{} {} {}", p.property, p.duration, easing)
                    }).collect();
                    self.emit_line(&format!(
                        "{}.style.transition = \"{}\";",
                        var, transitions.join(", ")
                    ));
                }

                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }

            ComponentRef::SubComponent(parent_name, sub_name) => {
                let class = format!("wf-{}__{}",
                    parent_name.to_lowercase(),
                    camel_to_kebab(sub_name)
                );
                let tag = match (parent_name.as_str(), sub_name.as_str()) {
                    (_, "Item") => "li",
                    _ => "div",
                };

                self.emit_line(&format!(
                    "const {} = WF.h(\"{}\", {{ className: \"{}\" }});",
                    var, tag, class
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

    // ─── Special component emitters ──────────────────

    fn emit_modal_dialog(&mut self, name: &str, var: &str, _attrs_str: &str, ui: &UIElement, parent: &str) {
        let class = if name == "Modal" { "wf-modal" } else { "wf-dialog" };
        let title = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "title" { Some(self.emit_expr(v)) } else { None }
            } else { None }
        });

        // Check for visible binding
        let visible_state = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "visible" {
                    if let Expr::Identifier(s) = v { return Some(s.clone()); }
                }
                None
            } else { None }
        });

        self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"{}\" }});", var, class));

        let content_var = self.fresh_var();
        let content_class = format!("{}__content", class);
        self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"{}\" }});", content_var, content_class));

        if let Some(t) = title {
            let header_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"div\", {{ className: \"{}__header\" }}, WF.h(\"h3\", {{}}, {}));",
                header_var, class, t
            ));
            self.emit_line(&format!("{}.appendChild({});", content_var, header_var));
        }

        let body_var = self.fresh_var();
        self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"{}__body\" }});", body_var, class));

        // Check for Modal.Footer
        let mut footer_stmts = Vec::new();
        let mut body_stmts = Vec::new();
        for child in &ui.children {
            if let Statement::UIElement(ui_child) = child {
                if matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == name && s == "Footer") {
                    footer_stmts = ui_child.children.clone();
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
            self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"{}__footer\" }});", footer_var, class));
            for child in &footer_stmts {
                self.emit_statement_dom(&child, &footer_var);
            }
            self.emit_line(&format!("{}.appendChild({});", content_var, footer_var));
        }

        self.emit_line(&format!("{}.appendChild({});", var, content_var));

        // Visibility binding
        if let Some(state_name) = visible_state {
            self.emit_line(&format!(
                "WF.effect(() => {{ {}.className = _{}() ? '{} open' : '{}'; }});",
                var, state_name, class, class
            ));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_tabs(&mut self, var: &str, ui: &UIElement, parent: &str) {
        self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"wf-tabs\" }});", var));
        let nav_var = self.fresh_var();
        self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"wf-tabs__nav\" }});", nav_var));

        // Collect tab pages
        let tab_pages: Vec<(&UIElement, usize)> = ui.children.iter().enumerate().filter_map(|(i, s)| {
            if let Statement::UIElement(ui_child) = s {
                if matches!(&ui_child.component, ComponentRef::BuiltIn(n) if n == "TabPage") {
                    return Some((ui_child, i));
                }
            }
            None
        }).collect();

        let active_var = self.fresh_var();
        self.emit_line(&format!("const {} = WF.signal(0);", active_var));

        // Create tab buttons
        for (i, (tab, _)) in tab_pages.iter().enumerate() {
            let label = tab.args.first().map(|a| {
                if let Arg::Positional(expr) = a { self.emit_expr(expr) } else { format!("\"Tab {}\"", i) }
            }).unwrap_or_else(|| format!("\"Tab {}\"", i));

            let btn_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"button\", {{ className: () => {}() === {} ? \"wf-tabs__tab active\" : \"wf-tabs__tab\", \"on:click\": () => {}.set({}) }}, {});",
                btn_var, active_var, i, active_var, i, label
            ));
            self.emit_line(&format!("{}.appendChild({});", nav_var, btn_var));
        }

        self.emit_line(&format!("{}.appendChild({});", var, nav_var));

        // Create tab content
        for (i, (tab, _)) in tab_pages.iter().enumerate() {
            let page_var = self.fresh_var();
            self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"wf-tab-page\" }});", page_var));
            for child in &tab.children {
                self.emit_statement_dom(child, &page_var);
            }
            self.emit_line(&format!(
                "WF.effect(() => {{ {}.style.display = {}() === {} ? 'block' : 'none'; }});",
                page_var, active_var, i
            ));
            self.emit_line(&format!("{}.appendChild({});", var, page_var));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_switch(&mut self, var: &str, attrs: &[String], _ui: &UIElement, parent: &str) {
        let bind_var = attrs.iter().find_map(|a| {
            if a.starts_with("value: () => _") {
                Some(a.replace("value: () => _", "").replace("()", ""))
            } else { None }
        });

        let label = _ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" { Some(self.emit_expr(v)) } else { None }
            } else { None }
        });

        self.emit_line(&format!("const {} = WF.h(\"label\", {{ className: \"wf-switch\" }});", var));

        if let Some(state) = &bind_var {
            let input_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.h(\"input\", {{ type: \"checkbox\", checked: () => _{}(), \"on:change\": () => _{}.set(!_{}()) }});",
                input_var, state, state, state
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

    fn emit_check_radio(&mut self, name: &str, var: &str, _attrs: &[String], ui: &UIElement, parent: &str) {
        let input_type = if name == "Checkbox" { "checkbox" } else { "radio" };
        let class = if name == "Checkbox" { "wf-checkbox" } else { "wf-radio" };

        let bind_var = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "bind" {
                    if let Expr::Identifier(s) = v { return Some(s.clone()); }
                }
                None
            } else { None }
        });

        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" { Some(self.emit_expr(v)) } else { None }
            } else { None }
        });

        let radio_value = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "value" { Some(self.emit_expr(v)) } else { None }
            } else { None }
        });

        self.emit_line(&format!("const {} = WF.h(\"label\", {{ className: \"{}\" }});", var, class));

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
                if k == "checked" { Some(self.emit_expr(v)) } else { None }
            } else { None }
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
            input_attrs.push_str(&format!(", \"on:{}\": (e) => {{ {} }}", handler.event, body));
        }

        self.emit_line(&format!("const {} = WF.h(\"input\", {{ {} }});", input_var, input_attrs));
        self.emit_line(&format!("{}.appendChild({});", var, input_var));

        if let Some(l) = label {
            self.emit_line(&format!("{}.appendChild(WF.text({}));", var, l));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_dropdown_menu(&mut self, name: &str, var: &str, _attrs: &[String], ui: &UIElement, parent: &str) {
        let class = if name == "Dropdown" { "wf-dropdown" } else { "wf-menu" };

        let label = ui.args.iter().find_map(|a| {
            match a {
                Arg::Named(k, v) if k == "label" || k == "trigger" => Some(self.emit_expr(v)),
                _ => None,
            }
        }).unwrap_or_else(|| "\"Menu\"".to_string());

        let open_var = self.fresh_var();
        self.emit_line(&format!("const {} = WF.signal(false);", open_var));
        self.emit_line(&format!(
            "const {} = WF.h(\"div\", {{ className: () => {}() ? \"{} open\" : \"{}\" }});",
            var, open_var, class, class
        ));

        let trigger_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.h(\"button\", {{ className: \"wf-btn\", \"on:click\": () => {}.set(!{}()) }}, {});",
            trigger_var, open_var, open_var, label
        ));
        self.emit_line(&format!("{}.appendChild({});", var, trigger_var));

        let items_var = self.fresh_var();
        let items_class = format!("{}__items", class);
        self.emit_line(&format!("const {} = WF.h(\"div\", {{ className: \"{}\" }});", items_var, items_class));

        for child in &ui.children {
            self.emit_statement_dom(child, &items_var);
        }

        self.emit_line(&format!("{}.appendChild({});", var, items_var));

        // Close on click outside
        self.emit_line(&format!(
            "document.addEventListener('click', (e) => {{ if (!{}.contains(e.target)) {}.set(false); }});",
            var, open_var
        ));

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
        self.emit_line(&format!("const {} = document.createDocumentFragment();", then_var));
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
            self.emit_line(&format!("const {} = document.createDocumentFragment();", else_var));
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
            self.emit_line(&format!("const {} = document.createDocumentFragment();", elif_var));
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
        let item_var = self.fresh_var();
        self.emit_line(&format!("const {} = document.createDocumentFragment();", item_var));
        for stmt in &for_stmt.body {
            self.emit_statement_dom(stmt, &item_var);
        }
        self.emit_line(&format!("return {};", item_var));
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
        self.emit_line(&format!("const {} = document.createDocumentFragment();", content_var));
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

        self.emit_line(&format!("const {} = WF.wfFetch({}, {}, {{", var, url, opts_str));
        self.indent += 1;

        if let Some(loading) = &fetch.loading_block {
            self.emit_line("loading: () => {");
            self.indent += 1;
            let l_var = self.fresh_var();
            self.emit_line(&format!("const {} = document.createDocumentFragment();", l_var));
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
            let e_var = self.fresh_var();
            self.emit_line(&format!("const {} = document.createDocumentFragment();", e_var));
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
            let s_var = self.fresh_var();
            self.emit_line(&format!("const {} = document.createDocumentFragment();", s_var));
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
        match stmt {
            Statement::Assignment(a) => {
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
            Statement::MethodCall(mc) => {
                let obj = self.emit_expr(&mc.object);
                let args: Vec<String> = mc.args.iter().map(|a| self.emit_expr(a)).collect();
                self.emit_line(&format!("{}.{}({});", obj, mc.method, args.join(", ")));
            }
            Statement::Navigate(expr) => {
                let path = self.emit_expr(expr);
                self.emit_line(&format!("WF.navigate({});", path));
            }
            Statement::Log(expr) => {
                let val = self.emit_expr(expr);
                self.emit_line(&format!("console.log({});", val));
            }
            Statement::Animate(anim) => {
                let dur = anim.duration.as_deref().map(|d| format!(", \"{}\"", d)).unwrap_or_default();
                self.emit_line(&format!(
                    "WF.animateEl(\"{}\", \"{}\"{});",
                    anim.target, anim.animation, dur
                ));
            }
            Statement::ExprStatement(expr) => {
                let val = self.emit_expr(expr);
                self.emit_line(&format!("{};", val));
            }
            Statement::State(s) => {
                let val = self.emit_expr(&s.value);
                self.emit_line(&format!("const _{} = WF.signal({});", s.name, val));
            }
            Statement::If(if_stmt) => {
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
            Statement::Fetch(fetch) => {
                self.emit_imperative_fetch(fetch);
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

        let method = fetch.options.iter().find_map(|o| {
            if o.key == "method" { Some(self.emit_expr(&o.value)) } else { None }
        }).unwrap_or_else(|| "\"GET\"".to_string());

        let body = fetch.options.iter().find_map(|o| {
            if o.key == "body" { Some(self.emit_expr(&o.value)) } else { None }
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

    fn emit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLiteral(s) => {
                let escaped = s.replace('\\', "\\\\")
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
                // Store references, component props, and built-in names stay as-is
                if self.stores.contains(name)
                    || self.current_props.contains(name)
                    || name == "params"
                    || name == "value"
                    || name == "key"
                    || name == "event"
                    || name == "e"
                    || name.starts_with("_")
                {
                    format!("{}", name)
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

                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                // Check if it's a store function
                if self.stores.contains(name) {
                    format!("{}.{}({})", name, args_str.first().unwrap_or(&String::new()), args_str.get(1..).unwrap_or(&[]).join(", "))
                } else {
                    format!("{}({})", name, args_str.join(", "))
                }
            }
            Expr::ListLiteral(items) => {
                let items_str: Vec<String> = items.iter().map(|i| self.emit_expr(i)).collect();
                format!("[{}]", items_str.join(", "))
            }
            Expr::MapLiteral(entries) => {
                let entries_str: Vec<String> = entries.iter().map(|(k, v)| {
                    format!("{}: {}", k, self.emit_expr(v))
                }).collect();
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
            match stmt {
                Statement::Assignment(a) => {
                    let value = self.emit_expr(&a.value);
                    if let Expr::Identifier(name) = &a.target {
                        parts.push(format!("_{}.set({});", name, value));
                    } else {
                        let target = self.emit_expr(&a.target);
                        parts.push(format!("{} = {};", target, value));
                    }
                }
                Statement::Navigate(expr) => {
                    let path = self.emit_expr(expr);
                    parts.push(format!("WF.navigate({});", path));
                }
                Statement::ExprStatement(expr) => {
                    let val = self.emit_expr(expr);
                    parts.push(format!("{};", val));
                }
                Statement::MethodCall(mc) => {
                    let obj = self.emit_expr(&mc.object);
                    let args: Vec<String> = mc.args.iter().map(|a| self.emit_expr(a)).collect();
                    parts.push(format!("{}.{}({});", obj, mc.method, args.join(", ")));
                }
                Statement::If(if_stmt) => {
                    let cond = self.emit_expr(&if_stmt.condition);
                    let then_body = self.emit_statements_inline(&if_stmt.then_body);
                    if let Some(else_body) = &if_stmt.else_body {
                        let else_str = self.emit_statements_inline(else_body);
                        parts.push(format!("if ({}) {{ {} }} else {{ {} }}", cond, then_body, else_str));
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
            match stmt {
                Statement::Assignment(a) => {
                    let value = self.emit_expr(&a.value);
                    if let Expr::Identifier(name) = &a.target {
                        parts.push(format!("_{}.set({});", name, value));
                    } else {
                        let target = self.emit_expr(&a.target);
                        parts.push(format!("{} = {};", target, value));
                    }
                }
                Statement::Navigate(expr) => {
                    let path = self.emit_expr(expr);
                    parts.push(format!("WF.navigate({});", path));
                }
                Statement::ExprStatement(expr) => {
                    let val = self.emit_expr(expr);
                    parts.push(format!("{};", val));
                }
                _ => {}
            }
        }
        parts.join(" ")
    }
}

// ─── Utility functions ──────────────────────────────────

fn builtin_to_html(name: &str) -> (&str, &str) {
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
        "DatePicker" => ("input", "wf-datepicker"),
        "FileUpload" => ("input", "wf-file-upload"),
        "Form" => ("form", "wf-form"),
        "Alert" => ("div", "wf-alert"),
        "Toast" => ("div", "wf-toast"),
        "Modal" => ("dialog", "wf-modal"),
        "Dialog" => ("dialog", "wf-dialog"),
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
        "Router" => ("div", "wf-router"),
        "Route" => ("div", ""),
        _ => ("div", ""),
    }
}

fn modifier_to_class(base_class: &str, modifier: &str) -> String {
    match modifier {
        // Size
        "small" => format!("{}--small", base_class),
        "medium" => String::new(), // default, no class needed
        "large" => format!("{}--large", base_class),
        // Color
        "primary" => format!("{}--primary", base_class),
        "secondary" => format!("{}--secondary", base_class),
        "success" => format!("{}--success", base_class),
        "danger" => format!("{}--danger", base_class),
        "warning" => format!("{}--warning", base_class),
        "info" => format!("{}--info", base_class),
        // Shape
        "rounded" => format!("{}--rounded", base_class),
        "pill" => format!("{}--pill", base_class),
        "square" => format!("{}--square", base_class),
        // Elevation
        "flat" => format!("{}--flat", base_class),
        "elevated" => format!("{}--elevated", base_class),
        "outlined" => format!("{}--outlined", base_class),
        // Width
        "full" => format!("{}--full", base_class),
        "fit" => format!("{}--fit", base_class),
        // Text
        "bold" => "wf-text--bold".to_string(),
        "italic" => "wf-text--italic".to_string(),
        "underline" => "wf-text--underline".to_string(),
        "uppercase" => "wf-text--uppercase".to_string(),
        "lowercase" => "wf-text--lowercase".to_string(),
        // Alignment
        "left" => "wf-text--left".to_string(),
        "center" => "wf-text--center".to_string(),
        "right" => "wf-text--right".to_string(),
        // Typography
        "heading" => "wf-text--heading".to_string(),
        "subtitle" => "wf-text--subtitle".to_string(),
        "muted" => "wf-text--muted".to_string(),
        // Heading levels - change the tag via a class
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => format!("wf-heading--{}", modifier),
        // Dismissible
        "dismissible" => format!("{}--dismissible", base_class),
        // Block
        "block" => format!("{}--block", base_class),
        "bordered" => format!("{}--bordered", base_class),
        // Fluid
        "fluid" => format!("{}--fluid", base_class),
        // Don't add input type modifiers as classes
        "text" | "email" | "password" | "number" | "search" | "tel" | "url" |
        "date" | "time" | "datetime" | "color" | "submit" | "reset" | "required" |
        "controls" | "autoplay" => String::new(),
        // Animation modifiers
        "fadeIn" | "fadeOut" | "slideUp" | "slideDown" |
        "slideLeft" | "slideRight" | "scaleIn" | "scaleOut" |
        "bounce" | "shake" | "pulse" | "spin" => format!("wf-animate-{}", modifier),
        // Animation speed
        "fast" => "wf-animate--fast".to_string(),
        "slow" => "wf-animate--slow".to_string(),
        _ => String::new(),
    }
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
    if expr_str.contains("WF.i18n.t(") || expr_str.contains("WF.i18n.locale()") || expr_str.contains("WF.i18n.dir()") {
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
