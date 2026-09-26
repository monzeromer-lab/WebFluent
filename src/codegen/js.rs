use crate::codegen::builtin::{
    builtin_to_html, element_tag, implicit_role, input_type, is_void, landmark_label,
    layout_arg_classes, modifier_to_class,
};
use crate::codegen::node_id::NodeMap;
use crate::parser::ast::*;
use crate::runtime;
use std::collections::HashMap;

/// A component's JavaScript name: a part's `Owner.Part` is `Owner_Part`.
fn js_component(name: &str) -> String {
    name.replace('.', "_")
}

/// Every name `body` declares at any depth: state, derived values,
/// resources, actions and the locals of their bodies.
fn declared_names(body: &[Statement]) -> Vec<String> {
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            let name = match &stmt.kind {
                StatementKind::State(s) => Some(&s.name),
                StatementKind::Derived(d) => Some(&d.name),
                StatementKind::Resource(r) => Some(&r.name),
                StatementKind::Action(a) => Some(&a.name),
                _ => None,
            };
            if let Some(name) = name
                && !out.contains(name)
            {
                out.push(name.clone());
            }
            if let StatementKind::Action(a) = &stmt.kind {
                for p in &a.params {
                    if !out.contains(&p.name) {
                        out.push(p.name.clone());
                    }
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
            if let StatementKind::UIElement(el) = &stmt.kind {
                walk(&el.children, out);
                for h in &el.events {
                    walk(&h.body, out);
                }
                for f in &el.slot_fills {
                    walk(&f.body, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(body, &mut out);
    out
}

/// The names of the actions declared directly in `body`.
fn action_names(body: &[Statement]) -> Vec<String> {
    body.iter()
        .filter_map(|s| match &s.kind {
            StatementKind::Action(a) => Some(a.name.clone()),
            _ => None,
        })
        .collect()
}

/// JavaScript code generator — compiles the AST to a JS bundle with reactivity and routing.
pub struct JsCodegen {
    output: String,
    /// `Some(sync)` when the config names `offline`: the service worker is
    /// registered at boot, and `sync` keeps writes made offline.
    offline: Option<bool>,
    /// Ship every runtime module rather than the ones the program reaches
    /// (`build.runtime: "full"`).
    full_runtime: bool,
    /// The runtime modules this build kept, and the bytes they came to, for
    /// `wf build --stats`. Filled by `generate`.
    runtime_modules: Vec<&'static str>,
    runtime_report: Vec<runtime::Kept>,
    runtime_bytes: usize,
    indent: usize,
    /// Track user-defined component names so we can reference them
    components: Vec<String>,
    /// Track store names
    stores: Vec<String>,
    /// `external element Stripe("stripe-pricing-table")`: the tag each
    /// one is placed as.
    external_elements: HashMap<String, String>,
    /// The program's `const` names: plain values, read as written.
    consts: Vec<String>,
    /// `env.NAME` values from the project's config, emitted once.
    env: std::collections::BTreeMap<String, serde_json::Value>,
    /// Track current component/page prop names (not signals)
    current_props: Vec<String>,
    /// The actions the page or component being emitted declares: a call to
    /// one is its own, even where a built-in has the same name.
    own_actions: Vec<String>,
    /// The element handles the page or component declares with `ref:`,
    /// and the form handles it declares with `Form(bind: name)`.
    refs: Vec<String>,
    /// The form handle the body binds, which a `validate` block registers
    /// into, and which decides when a message is shown.
    current_form: Option<String>,
    /// The states a `validate` block guards, so the control bound to one
    /// shows its message without being told to.
    validated: Vec<String>,
    /// The body being emitted, for reading a state's declared condition.
    current_body: Vec<Statement>,
    /// Every name the page, component or store being emitted declares, at
    /// any depth: a declared `query` is its own, not the browser's.
    own_names: Vec<String>,
    /// Names bound by an enclosing `for` loop, innermost last.
    ///
    /// `WF.each` calls the body with the item as a plain parameter, so a
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
    /// A title that splices more than a parameter's name, compiled to a
    /// function of the route's parameters.
    page_title_fns: HashMap<String, String>,
    /// Whether the element being emitted sits inside a `Thead`, where a
    /// `Tcell` is a column header (`<th scope="col">`), not a data cell.
    in_thead: bool,
    /// Studio mode: stamp `data-wf-node` ids on rendered elements. Off for
    /// export/release builds, which must contain no debug attributes.
    studio: bool,
    /// Deterministic node ids keyed by element span (empty unless in studio mode).
    node_ids: NodeMap,
    /// Whether each page is written as its own chunk (`pages/<Name>.js`),
    /// registered with `WF.page`, rather than into the main bundle.
    split_pages: bool,
    /// The page chunks, `(name, source)`, when `split_pages` is on.
    chunks: Vec<(String, String)>,
    /// The pages with a stylesheet of their own (`pages/<Name>.css`), which
    /// the router loads before drawing them.
    page_sheets: std::collections::BTreeSet<String>,
    /// The next fresh variable name. Per generator, so two builds of one
    /// program in one process emit identical code.
    next_var: std::cell::Cell<usize>,
    /// The resources declared in the page or component being emitted: a
    /// reference to one is the resource object, not a signal read.
    resources: Vec<String>,
    /// Each user component's positional prop, when it declares one: a
    /// positional argument at a call binds to it. Absent a declared one,
    /// the first prop, as the static paint has always done.
    component_positional: HashMap<String, String>,
    /// Each user component's declared events, which a call passes handlers
    /// for as `on: { name: fn }`.
    component_events: HashMap<String, Vec<String>>,
    /// The names of what each component's scoped slots hand over, by
    /// component and slot: what a fill's own names stand for.
    component_slot_params: HashMap<String, HashMap<String, Vec<String>>>,
    /// The route parameters of the page being emitted, read as
    /// `params.<name>`.
    page_params: Vec<String>,
    /// Each page's layout component and the compiled props object it is
    /// called with, when the page names one.
    page_layouts: HashMap<String, (String, String)>,
    /// Each page's `guard:` expression as JavaScript and its `redirect:`,
    /// for pages that have one.
    page_guards: HashMap<String, (String, String)>,
    /// Every page's path, in declaration order, for a router that lists no
    /// routes of its own.
    page_paths: Vec<(String, String)>,
}

impl Default for JsCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl JsCodegen {
    pub fn new() -> Self {
        Self {
            offline: None,
            consts: Vec::new(),
            env: Default::default(),
            output: String::new(),
            full_runtime: false,
            runtime_modules: Vec::new(),
            runtime_report: Vec::new(),
            runtime_bytes: 0,
            indent: 0,
            components: Vec::new(),
            stores: Vec::new(),
            current_props: Vec::new(),
            own_actions: Vec::new(),
            refs: Vec::new(),
            current_form: None,
            validated: Vec::new(),
            current_body: Vec::new(),
            own_names: Vec::new(),
            loop_bindings: Vec::new(),
            lambda_params: std::cell::RefCell::new(Vec::new()),
            store_locals: std::cell::RefCell::new(Vec::new()),
            i18n_default_locale: None,
            i18n_translations: HashMap::new(),
            ssg_mode: false,
            base_path: String::new(),
            page_titles: HashMap::new(),
            page_title_fns: HashMap::new(),
            in_thead: false,
            studio: false,
            node_ids: NodeMap::default(),
            split_pages: false,
            chunks: Vec::new(),
            page_sheets: std::collections::BTreeSet::new(),
            next_var: std::cell::Cell::new(0),
            resources: Vec::new(),
            component_positional: HashMap::new(),
            external_elements: HashMap::new(),
            component_events: HashMap::new(),
            component_slot_params: HashMap::new(),
            page_params: Vec::new(),
            page_layouts: HashMap::new(),
            page_guards: HashMap::new(),
            page_paths: Vec::new(),
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

    /// The values `env.NAME` reads, from the project's config.
    /// Register the service worker `offline` in the config asks for, and
    /// whether writes made offline are kept (`sync`).
    pub fn set_offline(&mut self, sync: bool) {
        self.offline = Some(sync);
    }

    pub fn set_env(&mut self, env: std::collections::BTreeMap<String, serde_json::Value>) {
        self.env = env;
    }

    /// Enable studio mode and supply the node-identity map. When enabled, each
    /// element's root gets `data-wf-node="<id>"`.
    /// Ship the whole runtime, for a site whose own scripts reach for `WF`
    /// in ways a build cannot see.
    pub fn set_full_runtime(&mut self, full: bool) {
        self.full_runtime = full;
    }

    /// The runtime modules the last `generate` kept, and what they weighed.
    pub fn runtime_stats(&self) -> (&[&'static str], usize) {
        (&self.runtime_modules, self.runtime_bytes)
    }

    /// Each kept module, with its size and what reached it.
    pub fn runtime_report(&self) -> &[runtime::Kept] {
        &self.runtime_report
    }

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

    /// A `WF.signal` or, for `persist`, a `WF.persist` keyed by the owner
    /// and the name, so a page and a store may each have a `theme`.
    fn state_init(&mut self, owner: &str, s: &StateDecl, value: &str) -> String {
        if s.persist {
            let key = format!("{owner}.{}", s.name);
            // A page's `persist` takes the same policy a store's does:
            // where it is written, the version of its shape, whether
            // another tab's write is adopted, and how an older value is
            // brought forward.
            let policy = match &s.policy {
                Some(_) => format!(", {}", self.persist_policy(s)),
                None => String::new(),
            };
            let init = format!("WF.persist(\"{key}\", {value}{policy})");
            if self.studio {
                format!("WF.__reg(\"{}\", {init})", s.name)
            } else {
                init
            }
        } else {
            self.signal_init(&s.name, value)
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
        let title = match (self.page_title_fns.get(page), self.page_titles.get(page)) {
            (Some(f), _) => format!("title: (params) => {f}, "),
            (None, Some(t)) => format!(
                "title: \"{}\", ",
                t.replace('\\', "\\\\").replace('"', "\\\"")
            ),
            (None, None) => String::new(),
        };
        // The layout that frames the page, called with the page as its
        // default slot.
        let layout = match self.page_layouts.get(page) {
            Some((name, args)) => format!(
                "layout: (page, params) => Component_{}({}, {{ children: () => page(params) }}), ",
                js_component(name),
                args
            ),
            None => String::new(),
        };
        // `guard:` must hold for the route to render; `redirect:` is where
        // the reader goes otherwise.
        let layout = match self.page_guards.get(page) {
            Some((guard, redirect)) => format!(
                "{layout}guard: () => ({guard}), redirect: \"{}\", ",
                redirect.replace('"', "\\\"")
            ),
            None => layout,
        };
        if self.split_pages {
            let css = if self.page_sheets.contains(page) {
                format!("css: \"{}\", ", page)
            } else {
                String::new()
            };
            format!(
                "{{ path: \"{}\", {}{}{}page: \"{}\" }},",
                path, title, css, layout, page
            )
        } else {
            format!(
                "{{ path: \"{}\", {}{}render: (params) => Page_{}(params) }},",
                path, title, layout, page
            )
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        if self.split_pages {
            self.page_sheets = crate::codegen::scoped_css::split_rules(program)
                .pages
                .into_keys()
                .collect();
        }

        // First pass: collect component and store names
        for decl in &program.declarations {
            match decl {
                Declaration::Component(c) => {
                    self.components.push(c.name.clone());
                    let positional = c
                        .props
                        .iter()
                        .find(|p| p.positional)
                        .or_else(|| c.props.first())
                        .map(|p| p.name.clone());
                    if let Some(name) = positional {
                        self.component_positional.insert(c.name.clone(), name);
                    }
                    self.component_events.insert(
                        c.name.clone(),
                        c.events.iter().map(|e| e.name.clone()).collect(),
                    );
                    self.component_slot_params.insert(
                        c.name.clone(),
                        c.slots
                            .iter()
                            .filter(|s| !s.params.is_empty())
                            .map(|s| {
                                (
                                    s.name.clone().unwrap_or_else(|| "children".to_string()),
                                    s.params.iter().map(|p| p.name.clone()).collect(),
                                )
                            })
                            .collect(),
                    );
                }
                Declaration::Store(s) => self.stores.push(s.name.clone()),
                // A service is a name in scope like a store's, not a
                // signal, so it is read as it is written.
                Declaration::Api(a) => self.stores.push(a.name.clone()),
                // An import is a name in scope, read as it is written; an
                // element is a component, placed like one.
                Declaration::External(e) => match e.kind {
                    crate::parser::ast::ExternalKind::Module => self.stores.push(e.name.clone()),
                    crate::parser::ast::ExternalKind::Element => {
                        self.external_elements
                            .insert(e.name.clone(), e.from.clone());
                    }
                },
                Declaration::Const(c) => self.consts.push(c.name.clone()),
                Declaration::Page(p) => {
                    if let Some(title) = &p.title {
                        self.page_titles.insert(p.name.clone(), title.clone());
                    }
                    if !p.path.is_empty() {
                        self.page_paths.push((p.path.clone(), p.name.clone()));
                    }
                }
                _ => {}
            }
        }
        // A layout's props are compiled once the component names are known.
        for decl in &program.declarations {
            if let Declaration::Page(p) = decl
                && let Some(layout) = &p.layout
            {
                let args = self.emit_component_args(&layout.name, &layout.args);
                self.page_layouts
                    .insert(p.name.clone(), (layout.name.clone(), args));
            }
            // `title: "{post.title} — Blog"`: worked out from the route's
            // parameters on each visit, as the build worked it out per file.
            if let Declaration::Page(p) = decl
                && let Some(expr) = &p.title_expr
            {
                self.page_params = p.params.iter().map(|p| p.name.clone()).collect();
                let js = self.emit_expr(expr);
                self.page_params.clear();
                self.page_title_fns
                    .insert(p.name.clone(), format!("String({js})"));
            }
            // A guard reads stores, which are in scope at the route table.
            if let Declaration::Page(p) = decl
                && let Some(guard) = &p.guard
            {
                let js = self.emit_expr(guard);
                let redirect = p.redirect.clone().unwrap_or_else(|| "/".to_string());
                self.page_guards.insert(p.name.clone(), (js, redirect));
            }
        }

        // Emit base path and SSG mode flag
        if !self.base_path.is_empty() {
            self.emit_line(&format!("WF.setBasePath(\"{}\");", self.base_path));
        }
        // The service worker, when the config names `offline`.
        if let Some(sync) = self.offline {
            self.emit_line(&format!("WF.offline({{ sync: {sync} }});"));
        }
        if self.ssg_mode {
            self.emit_line("WF.setSsgMode(true);");
        }

        // Emit i18n setup if configured
        self.emit_i18n_setup();

        // The project's `env` and its constants, before anything reads them.
        self.emit_line(&format!(
            "const env = {};",
            serde_json::to_string(&self.env).unwrap_or_else(|_| "{}".to_string())
        ));
        for decl in &program.declarations {
            if let Declaration::Const(c) = decl {
                let value = self.emit_expr(&c.value);
                self.emit_line(&format!("const {} = {};", c.name, value));
            }
        }

        // Emit services, which a store's actions may call.
        for decl in &program.declarations {
            if let Declaration::Api(a) = decl {
                self.emit_api(a);
            }
        }

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
                    self.emit_line(&format!("WF.page(\"{}\", Page_{});", p.name, p.name));
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
                self.emit_line("WF.router(routes, container);");
                self.indent -= 1;
                self.emit_line("})();");
            }
        }

        // The runtime goes on last, once the program is written: what it
        // needs is read from the program — every `WF.<name>` in the bundle
        // and in every page chunk — so the set cannot drift from the places
        // that emit a call. `full_runtime` ships all of it.
        let mut reached = self.output.clone();
        for (_, chunk) in &self.chunks {
            reached.push_str(chunk);
        }
        if self.studio {
            // The studio drives `WF.__debug` from outside the bundle.
            reached.push_str("WF.__debug");
        }
        self.runtime_report = runtime::report(&reached, self.full_runtime);
        self.runtime_modules = self.runtime_report.iter().map(|k| k.name).collect();
        let mut out = runtime::assemble(&reached, self.full_runtime);
        self.runtime_bytes = out.len();
        out.push('\n');
        out.push_str(&self.output);
        self.output = out;
        self.output.clone()
    }

    // ─── Store ───────────────────────────────────────

    /// `api Backend(base: "/api") { … }` — the service as one object of
    /// callable endpoints, each of which also carries `.invalidate()`,
    /// `.prefetch()` and `.progress`.
    fn emit_api(&mut self, api: &ApiDecl) {
        self.emit_line(&format!("const {} = WF.api({{", api.name));
        self.indent += 1;
        self.emit_line(&format!("name: \"{}\",", api.name));
        for (key, value) in &api.settings {
            // A setting that reads state is read again on every request.
            let emitted = match key.as_str() {
                // `.backoff(times: 3, on: [.network])`, or a plain count.
                "retry" => self.retry_policy(value),
                "cache" => self.cache_policy(value),
                // `.sameOrigin` and the like are what `fetch` calls them.
                "credentials" | "mode" | "redirect" | "referrer" => match value {
                    Expr::EnumCase(name) => format!("\"{}\"", fetch_word(name)),
                    other => self.emit_expr(other),
                },
                _ => self.emit_expr(value),
            };
            let emitted = if self.is_reactive(&emitted) {
                format!("() => {emitted}")
            } else {
                emitted
            };
            self.emit_line(&format!("{}: {emitted},", api_setting(key)));
        }
        if !api.headers.is_empty() {
            self.emit_line("headers: {");
            self.indent += 1;
            for (name, value) in &api.headers {
                // A header is read at the moment of the request, so a token
                // that has just been refreshed is the one that is sent.
                let emitted = self.emit_expr(value);
                self.emit_line(&format!("\"{name}\": () => {emitted},"));
            }
            self.indent -= 1;
            self.emit_line("},");
        }
        for hook in &api.hooks {
            let param = hook.param.clone().unwrap_or_else(|| "r".to_string());
            let name = match hook.event.as_str() {
                "request" => "onRequest",
                "response" => "onResponse",
                "error" => "onError",
                other => other,
            };
            self.emit_line(&format!("{name}: async ({param}) => {{"));
            self.indent += 1;
            // `emit_event_body` hands back what the hook does rather than
            // writing it, and binds the parameter so `r` is `r` and not the
            // signal a bare name would otherwise be read as.
            let body = self.emit_event_body(hook);
            if !body.is_empty() {
                self.emit_line(&body);
            }
            self.emit_line(&format!("return {param};"));
            self.indent -= 1;
            self.emit_line("},");
        }
        self.emit_line("endpoints: {");
        self.indent += 1;
        for endpoint in &api.endpoints {
            let mut parts = vec![
                format!("method: \"{}\"", endpoint.method),
                format!("path: \"{}\"", endpoint.path),
            ];
            for (key, value) in &endpoint.settings {
                parts.push(format!("{}: {}", api_setting(key), self.emit_expr(value)));
            }
            // A file parameter is sent as a form, under its own name.
            if let Some(file) = endpoint
                .params
                .iter()
                .find(|p| matches!(&p.prop_type, TypeRef::Named(n) if n == "File"))
            {
                parts.push(format!("fileField: \"{}\"", file.name));
            }
            self.emit_line(&format!("{}: {{ {} }},", endpoint.name, parts.join(", ")));
        }
        self.indent -= 1;
        self.emit_line("},");
        self.indent -= 1;
        self.emit_line("});");
        self.emit_line("");
    }

    fn emit_store(&mut self, store: &StoreDecl) {
        self.own_names = declared_names(&store.body);
        // The definition is a **thunk**: nothing in it is evaluated until
        // something reads the store. A `state` whose value reads another
        // store used to be evaluated at module scope, in declaration
        // order, and read `undefined` from a store declared below it.
        let options = format!(
            "{{ scope: \"{}\"{} }}",
            store.scope.as_str(),
            if store.eager { ", eager: true" } else { "" }
        );
        self.emit_line(&format!(
            "const {} = WF.store(\"{}\", () => ({{",
            store.name, store.name
        ));
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
            // What is kept across visits, and under what policy: where it
            // is written, the version of its shape, whether another tab's
            // write is adopted, and how a value an older build left is
            // brought forward.
            let persisted: Vec<&&StateDecl> = states.iter().filter(|s| s.persist).collect();
            if !persisted.is_empty() {
                self.emit_line("persist: {");
                self.indent += 1;
                for state in persisted {
                    let policy = self.persist_policy(state);
                    self.emit_line(&format!("{}: {},", state.name, policy));
                }
                self.indent -= 1;
                self.emit_line("},");
            }
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
                    "{}: {}(store{}) => {{",
                    a.name,
                    if crate::parser::ast::awaits(&a.body) {
                        "async "
                    } else {
                        ""
                    },
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
        self.emit_line(&format!("}}), {options});"));
        self.emit_line("");
    }

    /// What a `persist` says about where its value lives and how an older
    /// one is brought forward.
    fn persist_policy(&mut self, state: &StateDecl) -> String {
        let Some(policy) = &state.policy else {
            return "{}".to_string();
        };
        let mut parts = Vec::new();
        if let Some(place) = &policy.storage {
            parts.push(format!("in: \"{place}\""));
        }
        if let Some(version) = policy.version {
            parts.push(format!("version: {version}"));
        }
        if let Some(sync) = policy.sync {
            parts.push(format!("sync: {sync}"));
        }
        if !policy.migrations.is_empty() {
            // `old` is the migration's parameter, not a name of the
            // page's: bound so it is emitted as itself.
            let bound = self.loop_bindings.len();
            self.loop_bindings.push("old".to_string());
            let steps: Vec<String> = policy
                .migrations
                .iter()
                .map(|m| format!("{}: (old) => {}", m.to, self.emit_expr(&m.body)))
                .collect();
            self.loop_bindings.truncate(bound);
            parts.push(format!("migrate: {{ {} }}", steps.join(", ")));
        }
        format!("{{ {} }}", parts.join(", "))
    }

    /// Emit expression inside a store context — identifiers that are store state
    /// are accessed via `store.property` instead of `_name()`.
    fn emit_store_expr(&self, expr: &Expr, store_states: &[String]) -> String {
        match expr {
            Expr::Identifier(name) => {
                if store_states.contains(name) {
                    format!("store.{}", name)
                } else if BROWSER_VALUES.contains(&name.as_str()) && !self.own_names.contains(name)
                {
                    format!("WF.{name}()")
                } else {
                    name.to_string()
                }
            }
            Expr::PropertyAccess(base, prop) if prop == "pending" && self.is_action_ref(base) => {
                format!("{}.pending()", self.emit_store_expr(base, store_states))
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
            Expr::Spread(inner) => format!("...{}", self.emit_store_expr(inner, store_states)),
            Expr::Range(a, b, inclusive) => format!(
                "WF.range({}, {}, {})",
                self.emit_store_expr(a, store_states),
                self.emit_store_expr(b, store_states),
                inclusive
            ),
            Expr::OptionalProperty(base, prop) => {
                format!("{}?.{}", self.emit_store_expr(base, store_states), prop)
            }
            Expr::OptionalIndex(base, index) => format!(
                "{}?.[{}]",
                self.emit_store_expr(base, store_states),
                self.emit_store_expr(index, store_states)
            ),
            Expr::OptionalMethod(obj, method, args) => {
                let args: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_store_expr(a, store_states))
                    .collect();
                format!(
                    "{}?.{}({})",
                    self.emit_store_expr(obj, store_states),
                    method,
                    args.join(", ")
                )
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
                    BinOp::NullCoalesce => "??",
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
                if method == "__case" && args.is_empty() {
                    return format!("WF.caseOf({})", self.emit_store_expr(obj, store_states));
                }
                if (method == "__is" || method == "__payload") && args.len() == 1 {
                    let subject = self.emit_store_expr(obj, store_states);
                    let case = self.emit_store_expr(&args[0], store_states);
                    return if method == "__is" {
                        format!("(WF.caseOf({subject}) === {case})")
                    } else {
                        format!("WF.payload({subject}, {case})")
                    };
                }
                if method == "__if" && args.len() == 2 {
                    // An if-expression inside a store used to fall through to
                    // the page emitter, whose operands read `_x()` signals.
                    let cond = self.emit_store_expr(obj, store_states);
                    let then_val = self.emit_store_expr(&args[0], store_states);
                    let else_val = self.emit_store_expr(&args[1], store_states);
                    return format!("({} ? {} : {})", cond, then_val, else_val);
                }
                if method == "__iflet"
                    && args.len() == 2
                    && let Expr::Lambda(name, then_expr) = &args[0]
                {
                    let value = self.emit_store_expr(obj, store_states);
                    let then_val = self.emit_store_expr(then_expr, store_states);
                    let else_val = self.emit_store_expr(&args[1], store_states);
                    return format!(
                        "(({name}) => {name} != null ? {then_val} : {else_val})({value})"
                    );
                }
                let obj_str = self.emit_store_expr(obj, store_states);
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_store_expr(a, store_states))
                    .collect();
                // The same table the page emitter reads. A bare name that
                // is one of the store's own states is held by the store, so
                // a mutating method assigns back through it.
                let holder = match obj.as_ref() {
                    Expr::Identifier(n) if store_states.contains(n) => Holder::StoreMember,
                    Expr::PropertyAccess(base, _) => match base.as_ref() {
                        Expr::Identifier(s) if self.stores.contains(s) => Holder::StoreMember,
                        _ => Holder::Plain,
                    },
                    _ => Holder::Plain,
                };
                method_to_js(method, &obj_str, &args_str, holder)
            }
            Expr::Lambda(param, body) => {
                let body_str = self.emit_store_expr(body, store_states);
                format!("(({}) => {})", param, body_str)
            }
            Expr::EnumCase(case) => format!("\"{}\"", case),
            Expr::CaseValue(case, args) => {
                let mut items = vec![format!("\"{case}\"")];
                items.extend(args.iter().map(|a| self.emit_store_expr(a, store_states)));
                format!("[{}]", items.join(", "))
            }
            Expr::Token(name) => format!("\"var(--{})\"", name),
            Expr::Await(inner) => format!("(await {})", self.emit_store_expr(inner, store_states)),
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
                } else if matches!(
                    name.as_str(),
                    // The same set a page reaches: a store's action is where
                    // `optimistic` belongs, and `uuid` where an id is made.
                    "format" | "ago" | "setTheme" | "beacon" | "optimistic" | "sanitize" | "uuid"
                ) {
                    format!("WF.{}({})", name, args_str.join(", "))
                } else if name == "fetch" {
                    format!("WF.request({})", args_str.join(", "))
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
            Expr::MapLiteral(entries) | Expr::Record(_, entries) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| match k.as_str() {
                        "..." => format!("...{}", self.emit_store_expr(v, store_states)),
                        _ => format!("{}: {}", k, self.emit_store_expr(v, store_states)),
                    })
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
                for (cond, body) in &if_stmt.else_if_branches {
                    let cond = self.emit_store_expr(cond, store_states);
                    self.emit_line(&format!("}} else if ({cond}) {{"));
                    self.indent += 1;
                    for s in body {
                        self.emit_store_statement(s, store_states, action_params);
                    }
                    self.indent -= 1;
                }
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
            // `try`, `for` and `log` used to fall through to the page
            // emitter, whose statements read `_x()` signals a store does
            // not have.
            StatementKind::Try(t) => {
                self.emit_line("try {");
                self.indent += 1;
                for s in &t.body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                let param = t.param.clone().unwrap_or_else(|| "_error".to_string());
                self.emit_line(&format!("}} catch ({param}) {{"));
                self.store_locals.borrow_mut().push(param.clone());
                self.indent += 1;
                for s in &t.catch_body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                self.store_locals.borrow_mut().pop();
                self.emit_line("}");
            }
            StatementKind::For(f) => {
                let list = self.emit_store_expr(&f.iterable, store_states);
                match &f.index {
                    Some(index) => self.emit_line(&format!(
                        "for (const [{index}, {}] of Array.from({list}).entries()) {{",
                        f.item
                    )),
                    None => self.emit_line(&format!("for (const {} of {list}) {{", f.item)),
                }
                self.store_locals.borrow_mut().push(f.item.clone());
                if let Some(index) = &f.index {
                    self.store_locals.borrow_mut().push(index.clone());
                }
                self.indent += 1;
                for s in &f.body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                if f.index.is_some() {
                    self.store_locals.borrow_mut().pop();
                }
                self.store_locals.borrow_mut().pop();
                self.emit_line("}");
            }
            StatementKind::Log(expr) => {
                let val = self.emit_store_expr(expr, store_states);
                self.emit_line(&format!("console.log({});", val));
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

        self.emit_line("WF.i18n = WF.locales(");
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
        self.own_actions = action_names(&comp.body);
        self.refs = crate::sema::types::ref_names(&comp.body);
        self.own_names = declared_names(&comp.body);
        self.emit_pending_signals(&comp.body);
        // A form handle resolves as a bare name like a ref does, but is
        // declared here, once, as a form.
        let handles = self.refs.clone();
        for name in crate::sema::types::form_names(&comp.body) {
            self.emit_line(&format!(
                "const {name} = WF.form({});",
                form_options(&comp.body)
            ));
            self.current_form = Some(name.clone());
            self.refs.push(name);
        }
        self.validated = validated_names(&comp.body);
        self.current_body = comp.body.clone();
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
        for name in handles {
            self.emit_line(&format!("const {name} = WF.ref();"));
        }

        // Emit state declarations first
        self.resources = resource_names(&comp.body);
        for stmt in &comp.body {
            if let StatementKind::State(s) = &stmt.kind {
                let val = self.emit_expr(&s.value);
                let init = self.state_init(&comp.name, s, &val);
                self.emit_line(&format!("const _{} = {};", s.name, init));
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
        self.page_params = page.params.iter().map(|p| p.name.clone()).collect();
        self.own_actions = action_names(&page.body);
        self.refs = crate::sema::types::ref_names(&page.body);
        self.own_names = declared_names(&page.body);
        self.emit_pending_signals(&page.body);
        let handles = self.refs.clone();
        for name in crate::sema::types::form_names(&page.body) {
            self.emit_line(&format!(
                "const {name} = WF.form({});",
                form_options(&page.body)
            ));
            self.current_form = Some(name.clone());
            self.refs.push(name);
        }
        self.validated = validated_names(&page.body);
        self.current_body = page.body.clone();
        for name in handles {
            self.emit_line(&format!("const {name} = WF.ref();"));
        }

        // Emit state declarations
        self.resources = resource_names(&page.body);
        for stmt in &page.body {
            if let StatementKind::State(s) = &stmt.kind {
                let val = self.emit_expr(&s.value);
                let init = self.state_init(&page.name, s, &val);
                self.emit_line(&format!("const _{} = {};", s.name, init));
            }
        }

        // Build DOM
        self.emit_line("const _root = document.createDocumentFragment();");

        for stmt in &page.body {
            if !matches!(&stmt.kind, StatementKind::State(_)) {
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

        self.emit_line("return _root;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.page_params.clear();
    }

    // ─── App ─────────────────────────────────────────

    fn emit_app(&mut self, app: &AppDecl) {
        self.emit_line("(function() {");
        self.indent += 1;
        self.emit_line("const _app = document.getElementById('app');");
        self.emit_line("_app.innerHTML = '';");

        // The app's own state, declared before anything reads it — as a
        // page's is. An `app { state chosen = "en" … }` compiled every read
        // of `chosen` and never the `const` behind it, so the site threw on
        // load.
        self.own_names = declared_names(&app.body);
        self.own_actions = action_names(&app.body);
        self.resources = resource_names(&app.body);
        for stmt in &app.body {
            if let StatementKind::State(s) = &stmt.kind {
                let val = self.emit_expr(&s.value);
                let init = self.state_init("App", s, &val);
                self.emit_line(&format!("const _{} = {};", s.name, init));
            }
        }

        // The routes: the Router's own `Route` children when it lists any,
        // else every page's declared path — pages own their routes.
        let router_routes = Self::find_router_routes(&app.body);
        let has_router = Self::has_router(&app.body);

        // Recursively emit the app tree, replacing the Router with the route setup
        self.emit_app_tree(&app.body, "_app", has_router);

        if has_router {
            let mut routes: Vec<(String, String)> = Vec::new();
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
                routes.push((path.trim_matches('"').to_string(), page_name));
            }
            if routes.is_empty() {
                routes = self.page_paths.clone();
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
                                Expr::EnumCase(c) | Expr::StringLiteral(c) => format!("\"{c}\""),
                                other => self.emit_expr(other),
                            };
                            parts.push(format!("{k}: {value}"));
                        }
                    }
                    parts
                })
                .unwrap_or_default();
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
                    classes.extend(crate::codegen::scoped_css::responsive_classes(ui));
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
    /// Whether a `Router` element appears anywhere in `body`.
    fn has_router(body: &[Statement]) -> bool {
        body.iter().any(|stmt| match &stmt.kind {
            StatementKind::UIElement(ui) => {
                matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router")
                    || Self::has_router(&ui.children)
            }
            _ => false,
        })
    }

    /// The `Router` element, at any depth of the app body.
    fn find_router(body: &[Statement]) -> Option<&UIElement> {
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
            // The `fetch` block of the original grammar reaches only the
            // migrator, which rewrites it as a resource and a match.
            StatementKind::Fetch(_) => {}
            StatementKind::Resource(r) => self.emit_resource(r),
            StatementKind::Connection(c) => self.emit_connection(c),
            StatementKind::Validate(v) => self.emit_validate(v),
            StatementKind::Match(m) => self.emit_match_dom(m, parent),
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
                // What the effect returns runs before its next run and when
                // its owner leaves.
                if !e.cleanup.is_empty() {
                    self.emit_line("return () => {");
                    self.indent += 1;
                    for s in &e.cleanup {
                        self.emit_statement(s);
                    }
                    self.indent -= 1;
                    self.emit_line("};");
                }
                self.indent -= 1;
                self.emit_line("});");
            }
            // A timer stops with the scope it was declared in.
            StatementKind::Timer(t) => {
                let interval = self.emit_expr(&t.interval);
                let kind = if t.every { "every" } else { "after" };
                let head = if crate::parser::ast::awaits(&t.body) {
                    "async () =>"
                } else {
                    "() =>"
                };
                self.emit_line(&format!("WF.{kind}({interval}, {head} {{"));
                self.indent += 1;
                for s in &t.body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                self.emit_line("});");
            }
            StatementKind::Action(a) => {
                let params: Vec<String> = a.params.iter().map(|p| p.name.clone()).collect();
                // An action that awaits is an async function, and `name.pending`
                // is true while a call of it runs.
                let is_async = crate::parser::ast::awaits(&a.body);
                let kind = if is_async {
                    "async function"
                } else {
                    "function"
                };
                self.emit_line(&format!("{} {}({}) {{", kind, a.name, params.join(", ")));
                self.indent += 1;
                if is_async {
                    self.emit_line(&format!("_{}_pending.set(true);", a.name));
                    self.emit_line("try {");
                    self.indent += 1;
                }
                // An action that shows something before the server has
                // agreed takes it back if anything in it throws.
                let optimistic = shows_optimistically(&a.body);
                if optimistic {
                    self.emit_line("return await WF.attempt(async () => {");
                    self.indent += 1;
                }
                // The parameters are the function's own locals. They were read
                // as the page's signals — `n = n + by` compiled to `_by()`,
                // which does not exist — so an action that took an argument
                // threw the first time it ran.
                let depth = self.lambda_params.borrow().len();
                self.lambda_params
                    .borrow_mut()
                    .extend(params.iter().cloned());
                for s in &a.body {
                    self.emit_statement(s);
                }
                self.lambda_params.borrow_mut().truncate(depth);
                if optimistic {
                    self.indent -= 1;
                    self.emit_line("});");
                }
                if is_async {
                    self.indent -= 1;
                    self.emit_line(&format!("}} finally {{ _{}_pending.set(false); }}", a.name));
                }
                self.indent -= 1;
                self.emit_line("}");
            }
            // `on key("ctrl+k") { … }` on the page itself: the document
            // listens for as long as the page is shown.
            StatementKind::EventHandler(handler) if handler.event == "key" => {
                let body = self.emit_event_body(handler);
                let key = handler.key.clone().unwrap_or_default();
                self.emit_line(&format!(
                    "WF.onKey(document, \"{}\", {} => {{ {} }});",
                    key.replace('"', "\\\""),
                    Self::handler_head(handler, "event"),
                    body
                ));
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
                        let slot = ui.slot_name().unwrap_or("children");
                        // A scoped slot is handed its values, and rendered
                        // again when a value it reads changes.
                        let handed: Vec<String> = ui
                            .args
                            .iter()
                            .filter_map(|a| match a {
                                Arg::Named(k, v) if k != "slot" => {
                                    Some(format!("{k}: {}", self.emit_expr(v)))
                                }
                                _ => None,
                            })
                            .collect();
                        if handed.is_empty() {
                            self.emit_line(&format!(
                                "if (typeof _slots.{slot} === 'function') {}.appendChild(_slots.{slot}());",
                                parent
                            ));
                        } else {
                            self.emit_line(&format!(
                                "WF.slot({parent}, () => ({{ {} }}), (_s) => typeof _slots.{slot} === 'function' ? _slots.{slot}(_s) : null);",
                                handed.join(", ")
                            ));
                        }
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
                } else if name == "Host" {
                    // The library asked for a canvas, or a div by default.
                    crate::codegen::builtin::host_tag(ui.args.iter().find_map(|a| match a {
                        Arg::Named(k, Expr::StringLiteral(t)) if k == "tag" => Some(t.as_str()),
                        _ => None,
                    }))
                } else {
                    element_tag(name, &ui.modifiers)
                };

                // Collect attributes
                let mut attrs = Vec::new();
                let mut link_to: Option<String> = None;
                let mut link_prefix = false;
                let mut inner_text: Option<String> = None;
                // `Text(total, count: "600ms")`: the number counts to its new
                // value rather than jumping, so its positional is the value
                // and not the text.
                let counts = ui
                    .args
                    .iter()
                    .any(|a| matches!(a, Arg::Named(k, _) if k == "count"));

                // Build class string from base class + modifiers
                let mut classes = vec![class.to_string()];
                for m in &ui.modifiers {
                    let mod_class = modifier_to_class(class, m);
                    classes.push(mod_class);
                }

                // An `Input` or `Select` with a label, hint or error is wrapped
                // in a field that carries them.
                let validated = ui.args.iter().any(|a| {
                    matches!(a, Arg::Named(k, Expr::Identifier(bound))
                        if k == "bind" && self.validated.contains(bound))
                });
                // A control is a field when it is labelled, or when a
                // `validate` block guards what it binds — which is what
                // gives it somewhere to show the message.
                let is_field = matches!(name.as_str(), "Input" | "Select" | "Textarea")
                    && (validated
                        || ui.args.iter().any(|a| {
                            matches!(a, Arg::Named(k, _) if k == "label" || k == "hint" || k == "error")
                        }));

                // Process named args as HTML attributes
                for arg in &ui.args {
                    match arg {
                        // A value per breakpoint is carried by its class
                        // (`responsive_classes`, below), not an attribute.
                        Arg::Named(key, val)
                            if key != "class" && crate::codegen::scoped_css::is_responsive(val) => {
                        }
                        Arg::Named(key, val) => {
                            match key.as_str() {
                                // The handle takes the element once drawn.
                                "ref" => {
                                    if let Expr::Identifier(handle) = val {
                                        attrs.push(format!("ref: {handle}"));
                                    }
                                }
                                // `Form(bind: form)`: the handle takes the form.
                                "bind" if name == "Form" => {
                                    if let Expr::Identifier(handle) = val {
                                        attrs.push(format!("ref: {handle}"));
                                    }
                                }
                                "bind" => {
                                    // An empty date picker holds nothing,
                                    // which is `null` — not the empty
                                    // string the input reports.
                                    let read = if name == "DatePicker" {
                                        "e.target.value || null"
                                    } else {
                                        "e.target.value"
                                    };
                                    match val {
                                        Expr::Identifier(state_name) => {
                                            attrs.push(format!(
                                                "value: () => _{}() ?? \"\"",
                                                state_name
                                            ));
                                            attrs.push(format!(
                                                "\"on:input\": (e) => _{state_name}.set({read})"
                                            ));
                                        }
                                        // `bind: Cart.note` — a store's member
                                        // is a property with a getter and a
                                        // setter, so it binds like a state.
                                        // It used to compile to an input with
                                        // no binding at all.
                                        Expr::PropertyAccess(base, field) if matches!(base.as_ref(), Expr::Identifier(n) if self.stores.contains(n)) =>
                                        {
                                            let holder = self.emit_expr(base);
                                            attrs.push(format!(
                                                "value: () => {holder}.{field} ?? \"\""
                                            ));
                                            attrs.push(format!(
                                                "\"on:input\": (e) => {{ {holder}.{field} = {read}; }}"
                                            ));
                                        }
                                        other => {
                                            // Anything else has no setter, so
                                            // it cannot be bound; the check
                                            // reports it where it is written.
                                            let read = self.emit_expr(other);
                                            attrs.push(format!("value: () => {read} ?? \"\""));
                                        }
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
                                    // Modal/Dialog visibility, from any
                                    // expression that reads state.
                                    let read = self.emit_expr(val);
                                    attrs.push(format!(
                                        "className: () => {} ? '{} open' : '{}'",
                                        read,
                                        classes.join(" "),
                                        classes.join(" ")
                                    ));
                                }
                                // A field's label, hint and error are elements
                                // beside the control, built by `WF.field` below.
                                "label" | "hint" | "error" if is_field => {}
                                // Motion the runtime reads, not attributes
                                // the element carries.
                                "shared" | "on" | "count" => {}
                                // `Host(mount:, update:, cleanup:)`: the
                                // three are a lifetime, emitted below.
                                "mount" | "update" | "cleanup" | "tag" if name == "Host" => {}
                                // `maxLength` and `rows` are the textarea's,
                                // written as HTML writes them.
                                "maxLength" => {
                                    let v = self.emit_expr(val);
                                    attrs.push(format!("maxlength: {v}"));
                                }
                                "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max"
                                | "step" | "accept" | "label" | "required" | "disabled"
                                | "controls" | "autoplay" | "role" | "width" | "height"
                                | "loading" | "decoding" | "fetchpriority" | "rows" => {
                                    // A value that reads state follows it: a
                                    // `placeholder` or `disabled` bound to a
                                    // store used to be painted once.
                                    let v =
                                        if crate::codegen::url::URL_ATTRS.contains(&key.as_str()) {
                                            self.url_value(val)
                                        } else {
                                            self.emit_expr(val)
                                        };
                                    if self.is_reactive(&v) {
                                        attrs.push(format!("{}: () => {}", key, v));
                                    } else {
                                        attrs.push(format!("{}: {}", key, v));
                                    }
                                }
                                "to" => {
                                    let v = self.url_value(val);
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
                                // `Grid(columns: 3)`: a `data-cols` the stylesheet
                                // reads, so no inline style is needed under a
                                // strict CSP; a value that reads state follows it.
                                "columns" => {
                                    let v = self.emit_expr(val);
                                    if self.is_reactive(&v) {
                                        attrs.push(format!("\"data-cols\": () => {v}"));
                                    } else {
                                        attrs.push(format!("\"data-cols\": {v}"));
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
                                    if self.is_reactive(&v) {
                                        attrs.push(format!("\"data-icon\": () => {}", v));
                                    } else {
                                        attrs.push(format!("\"data-icon\": {}", v));
                                    }
                                }
                                // `Code(…, language: "wf")`: paired with the
                                // content below, into a `highlight` attribute.
                                "language" if name == "Code" => {}
                                _ => {
                                    // Any other named argument is an HTML
                                    // attribute. A hyphenated name (`aria-*`,
                                    // `data-*`) is quoted; a value that reads
                                    // state is a thunk, which the runtime
                                    // keeps in step with it; and one the
                                    // browser follows goes through the
                                    // scheme check first.
                                    let v =
                                        if crate::codegen::url::URL_ATTRS.contains(&key.as_str()) {
                                            self.url_value(val)
                                        } else {
                                            self.emit_expr(val)
                                        };
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
                            // `Unsafe.html(markup)`: the markup goes in as
                            // markup. It is the one element that does, and it
                            // is named so that a reviewer greps for it.
                            if name == "UnsafeHtml" {
                                let v = self.emit_expr(expr);
                                let getter = if self.is_reactive(&v) {
                                    format!("() => {v}")
                                } else {
                                    v
                                };
                                attrs.push(format!("html: {getter}"));
                                continue;
                            }
                            // Markdown is rendered to HTML by the runtime, and
                            // again when its text changes.
                            if name == "Markdown" {
                                let v = self.emit_expr(expr);
                                let getter = if self.is_reactive(&v) {
                                    format!("() => {v}")
                                } else {
                                    v
                                };
                                attrs.push(format!("markdown: {getter}"));
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
                            // — unless `count:` asked for it to be counted to,
                            // in which case `WF.counted` writes the text.
                            if counts {
                                continue;
                            }
                            if inner_text.is_none() {
                                inner_text = Some(self.emit_expr(expr));
                            }
                        }
                    }
                }

                classes.extend(layout_arg_classes(&ui.args));
                // `Grid(columns: { base: 1, md: 2 })`: the class its rules
                // live under, as the pre-rendered page carries it. Without
                // it, hydrating the page took the layout away.
                classes.extend(crate::codegen::scoped_css::responsive_classes(ui));

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

                // A void element cannot hold text. `WF.el("hr", {}, "label")` asks
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

                // `Code(…, language: "wf")`: the content and the language go
                // to the runtime's highlighter, not in as text.
                if name == "Code"
                    && let Some(lang) = ui.args.iter().find_map(|a| match a {
                        Arg::Named(k, v) if k == "language" => Some(self.emit_expr(v)),
                        _ => None,
                    })
                    && let Some(text) = inner_text.take()
                {
                    let getter = |v: &str| {
                        if self.is_reactive(v) {
                            format!("() => {v}")
                        } else {
                            v.to_string()
                        }
                    };
                    attrs.push(format!(
                        "highlight: {{ code: {}, lang: {} }}",
                        getter(&text),
                        getter(&lang)
                    ));
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
                    // A video's captions are a `<track>` inside it, and a
                    // player's transcript a link beneath it.
                    "Video" | "Audio"
                        if ui.args.iter().any(|a| {
                            matches!(a, Arg::Named(k, _) if k == "captions" || k == "transcript")
                        }) =>
                    {
                        self.emit_media(name, &var, &attrs_str, ui, parent);
                        true
                    }
                    // An `image` the program declares is a `<picture>`: one
                    // `<source>` per format, the widths the build wrote, and
                    // the box already the right size.
                    "Image" if ui.args.iter().any(|a| matches!(a, Arg::Positional(_))) =>
                    {
                        self.emit_picture(&var, ui, parent);
                        true
                    }
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
                            self.emit_line(&format!("WF.toast({}, \"{}\");", text, variant));
                        }
                        return;
                    }
                    "Spacer" => {
                        self.emit_line(&format!(
                            "const {} = WF.el(\"{}\", {});",
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
                    self.emit_motion(&var, ui);
                    self.emit_host(&var, ui);
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
                            "WF.el(\"caption\", {{ className: \"wf-visually-hidden\" }}, {})",
                            v
                        ));
                    }
                }

                // Inner text content — unless a `Code` with a `language:`
                // took it into its `highlight` attribute above.
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
                    children_arr.push(
                        "(typeof _slots.children === 'function' ? _slots.children() : null)"
                            .to_string(),
                    );
                }

                if children_arr.is_empty() && ui.children.is_empty() {
                    self.emit_line(&format!(
                        "const {} = WF.el(\"{}\", {});",
                        var, tag, attrs_str
                    ));
                } else if !children_arr.is_empty() && ui.children.is_empty() {
                    self.emit_line(&format!(
                        "const {} = WF.el(\"{}\", {}, {});",
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
                        "const {} = WF.el(\"{}\", {}{});",
                        var, tag, attrs_str, extra
                    ));
                }

                // Emit children
                {
                    let was_in_thead = self.in_thead;
                    if name == "Thead" {
                        self.in_thead = true;
                    } else if name == "Tbody" {
                        self.in_thead = false;
                    }
                    for child in &ui.children {
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
                        "const {} = WF.el(\"button\", {{ className: \"wf-navbar__toggle\",                          type: \"button\", \"aria-label\": \"Menu\", \"aria-expanded\": \"false\" }}, \"\\u2630\");",
                        toggle_var
                    ));
                    self.emit_line(&format!("{}.appendChild({});", var, toggle_var));
                    self.emit_line(&format!("WF.drawer({}, {}, null);", var, toggle_var));
                }

                for handler in &ui.events {
                    let body = self.emit_event_body(handler);
                    // A form asks the rules before it does anything: a
                    // submit with a fault shows every message, moves focus
                    // to the first field that has one, and stops there.
                    let body = if name == "Form"
                        && handler.event == "submit"
                        && let Some(form) = &self.current_form
                    {
                        format!("if (!{form}.__submitting()) return; {body}")
                    } else {
                        body
                    };
                    self.emit_line(&format!(
                        "{}.addEventListener(\"{}\", {} => {{ {} }});",
                        var,
                        handler.event,
                        Self::handler_head(handler, "event"),
                        body
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
                self.emit_motion(&var, ui);
                self.emit_host(&var, ui);

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
                    // A control bound to a state a `validate` block guards
                    // shows what the rules say, and marks itself touched
                    // when the reader leaves it — so a validated field
                    // needs no `error:` written on it.
                    if !opts.iter().any(|o| o.starts_with("error:"))
                        && let Some(Arg::Named(_, Expr::Identifier(bound))) = ui
                            .args
                            .iter()
                            .find(|a| matches!(a, Arg::Named(k, _) if k == "bind"))
                        && self.validated.contains(bound)
                    {
                        opts.push(format!("error: () => _{bound}_check.shown()"));
                        self.emit_line(&format!(
                            "WF.listen({var}, \"blur\", () => _{bound}_check.touched.set(true));"
                        ));
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
                // A part with a `to:` is a link — a `Sidebar.Item` or a
                // `Breadcrumb.Item` reached through a `for` or an `if`, which
                // the owner's own emitter does not see. It used to become a
                // `<li to="…">` that went nowhere.
                let to = ui.args.iter().find_map(|a| match a {
                    Arg::Named(k, v) if k == "to" => Some(self.emit_expr(v)),
                    _ => None,
                });
                let tag = match (parent_name.as_str(), sub_name.as_str(), &to) {
                    (_, _, Some(_)) => "a",
                    ("Breadcrumb", "Item", None) => "span",
                    (_, "Item", None) => "li",
                    _ => "div",
                };

                // Hyphenated and unknown named arguments are attributes here
                // too (`Card.Header(id: …)`, `List.Item(aria-current: …)`).
                let mut attrs = vec![format!("className: \"{}\"", class)];
                if let Some(href) = &to {
                    let bp = if self.base_path.is_empty() {
                        String::new()
                    } else {
                        "WF._basePath + ".to_string()
                    };
                    let href_attr = if self.is_reactive(href) {
                        format!("() => {bp}{href}")
                    } else {
                        format!("{bp}{href}")
                    };
                    attrs.push(format!("href: {href_attr}"));
                    if !self.ssg_mode {
                        attrs.push(format!(
                            "\"on:click\": (e) => {{ e.preventDefault(); WF.navigate({href}); }}"
                        ));
                    }
                }
                for arg in &ui.args {
                    if let Arg::Named(k, v) = arg {
                        if matches!(k.as_str(), "to" | "active") {
                            continue;
                        }
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
                    "const {} = WF.el(\"{}\", {{ {} }});",
                    var,
                    tag,
                    attrs.join(", ")
                ));
                if let Some(href) = &to {
                    let prefix = ui.args.iter().any(|a| {
                        matches!(a, Arg::Named(k, Expr::StringLiteral(v)) if k == "active" && v == "prefix")
                    });
                    self.emit_line(&format!("WF.activeLink({var}, {href}, {prefix});"));
                }
                for child in &ui.children {
                    self.emit_statement_dom(child, &var);
                }
                for handler in &ui.events {
                    let body = self.emit_event_body(handler);
                    self.emit_line(&format!(
                        "{}.addEventListener(\"{}\", {} => {{ {} }});",
                        var,
                        handler.event,
                        Self::handler_head(handler, "event"),
                        body
                    ));
                }
                // A sub-component used to drop its style block and its
                // handlers; every other element honours both.
                self.emit_style_and_transition(&var, ui);
                self.emit_motion(&var, ui);
                self.emit_host(&var, ui);
                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }

            ComponentRef::UserDefined(name) if self.external_elements.contains_key(name) => {
                // Somebody else's custom element: the tag it registered, the
                // props it declared as attributes, and the events it fires
                // as listeners. Nothing is guessed — the declaration is the
                // whole of what the compiler knows about it.
                let tag = self.external_elements[name].clone();
                let mut attrs: Vec<String> = Vec::new();
                for arg in &ui.args {
                    let Arg::Named(key, value) = arg else {
                        continue;
                    };
                    let attribute = kebab_case(key);
                    let emitted = if crate::codegen::url::URL_ATTRS.contains(&attribute.as_str()) {
                        self.url_value(value)
                    } else {
                        self.emit_expr(value)
                    };
                    if self.is_reactive(&emitted) {
                        attrs.push(format!("\"{attribute}\": () => {emitted}"));
                    } else {
                        attrs.push(format!("\"{attribute}\": {emitted}"));
                    }
                }
                self.emit_line(&format!(
                    "const {var} = WF.el(\"{tag}\", {{ {} }});",
                    attrs.join(", ")
                ));
                for handler in &ui.events {
                    let body = self.emit_event_body(handler);
                    self.emit_line(&format!(
                        "WF.onRoot({}, \"{}\", {} => {{ {} }});",
                        var,
                        handler.event,
                        Self::handler_head(handler, "event"),
                        body
                    ));
                }
                for child in &ui.children {
                    self.emit_statement_dom(child, &var);
                }
                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }
            ComponentRef::UserDefined(name) => {
                // A handler for an event the component declares is passed in
                // as `on: { name: fn }`; a DOM event's handler attaches to the
                // component's root element, so a styled button component is
                // clickable where it is used.
                let declared = self.component_events.get(name).cloned().unwrap_or_default();
                let (emitted, attached): (Vec<&EventHandler>, Vec<&EventHandler>) =
                    ui.events.iter().partition(|h| declared.contains(&h.event));
                let mut args_obj = self.emit_component_args(name, &ui.args);
                if !emitted.is_empty() {
                    let handlers: Vec<String> = emitted
                        .iter()
                        .map(|h| {
                            let body = self.emit_event_body(h);
                            format!(
                                "{}: {} => {{ {} }}",
                                h.event,
                                Self::handler_head(h, "_ev"),
                                body
                            )
                        })
                        .collect();
                    let on = format!("on: {{ {} }}", handlers.join(", "));
                    args_obj = if args_obj == "{}" {
                        format!("{{ {} }}", on)
                    } else {
                        format!("{}, {} }}", &args_obj[..args_obj.len() - 2], on)
                    };
                }
                let has_default_slot = !ui.children.is_empty();
                if !has_default_slot && ui.slot_fills.is_empty() {
                    self.emit_line(&format!(
                        "const {} = Component_{}({});",
                        var,
                        js_component(name),
                        args_obj
                    ));
                } else {
                    // Each fill is compiled here, in the caller's scope, so it
                    // reads the caller's state and loop bindings; the component
                    // only decides where it lands.
                    self.emit_line(&format!(
                        "const {} = Component_{}({}, {{",
                        var,
                        js_component(name),
                        args_obj
                    ));
                    self.indent += 1;
                    let mut fills: Vec<(&str, &[String], &[Statement])> = Vec::new();
                    if has_default_slot {
                        fills.push(("children", &[], &ui.children));
                    }
                    for fill in &ui.slot_fills {
                        fills.push((fill.name.as_str(), &fill.params, &fill.body));
                    }
                    let handed = self
                        .component_slot_params
                        .get(name)
                        .cloned()
                        .unwrap_or_default();
                    for (slot, params, body) in fills {
                        if params.is_empty() {
                            self.emit_line(&format!("{}: () => {{", slot));
                        } else {
                            self.emit_line(&format!("{}: (_s) => {{", slot));
                        }
                        self.indent += 1;
                        // A fill's names stand for what the slot hands over,
                        // in the slot's order, as plain values.
                        let declared = handed.get(slot).cloned().unwrap_or_default();
                        for (i, param) in params.iter().enumerate() {
                            let key = declared.get(i).cloned().unwrap_or_else(|| param.clone());
                            self.emit_line(&format!("const {param} = _s.{key};"));
                            self.loop_bindings.push(param.clone());
                        }
                        self.emit_line("const _cf = document.createDocumentFragment();");
                        for child in body {
                            self.emit_statement_dom(child, "_cf");
                        }
                        self.emit_line("return _cf;");
                        for _ in params {
                            self.loop_bindings.pop();
                        }
                        self.indent -= 1;
                        self.emit_line("},");
                    }
                    self.indent -= 1;
                    self.emit_line("});");
                }
                for handler in attached {
                    let body = self.emit_event_body(handler);
                    self.emit_line(&format!(
                        "WF.onRoot({}, \"{}\", {} => {{ {} }});",
                        var,
                        handler.event,
                        Self::handler_head(handler, "event"),
                        body
                    ));
                }
                // The motion asked of it here: markers on its root element.
                let marks: Vec<String> = ui
                    .args
                    .iter()
                    .filter_map(|a| match a {
                        Arg::Named(k, v) if k.starts_with("data-wf-") => {
                            Some(format!("\"{}\": {}", k, self.emit_expr(v)))
                        }
                        _ => None,
                    })
                    .collect();
                if !marks.is_empty() {
                    self.emit_line(&format!("WF.mark({}, {{ {} }});", var, marks.join(", ")));
                }
                self.emit_line(&format!("{}.appendChild({});", parent, var));
            }
        }
    }

    /// A value going where the browser will follow it.
    ///
    /// A literal has already been checked by the time codegen runs — the
    /// check refuses a `javascript:` URL where it is written. Anything
    /// else is only known at run time, so it goes through the twin of that
    /// check at the moment it is used.
    fn url_value(&mut self, val: &Expr) -> String {
        let emitted = self.emit_expr(val);
        match val {
            Expr::StringLiteral(_) => emitted,
            _ => format!("WF.safeUrl({emitted})"),
        }
    }

    /// `Host(mount:, update:, cleanup:)`: the element handed to somebody
    /// else's code, and given back when what owns it leaves.
    fn emit_host(&mut self, var: &str, ui: &UIElement) {
        if !matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Host") {
            return;
        }
        let lambda = |out: &mut Self, key: &str| {
            ui.args
                .iter()
                .find_map(|a| match a {
                    Arg::Named(k, v) if k == key => Some(out.emit_expr(v)),
                    _ => None,
                })
                .unwrap_or_else(|| "null".to_string())
        };
        let mount = lambda(self, "mount");
        let update = lambda(self, "update");
        let cleanup = lambda(self, "cleanup");
        self.emit_line(&format!("WF.attach({var}, {mount}, {update}, {cleanup});"));
    }

    /// What an element asked of motion beyond its enter animation: when to
    /// play it, and the name it keeps across a route change.
    fn emit_motion(&mut self, var: &str, ui: &UIElement) {
        // The timing props are lowered to `data-wf-*` markers before
        // codegen sees them, so look under both spellings.
        let named = |key: &str| {
            let marker = format!("data-wf-{key}");
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key || *k == marker => Some(v),
                _ => None,
            })
        };
        // `shared: "cover-{id}"`: the browser carries it from one page to
        // the next, which is what a View Transition is for.
        if let Some(name) = named("shared") {
            let value = self.emit_expr(name);
            self.emit_line(&format!("WF.shared({var}, {value});"));
        }
        // `on: .enterView`: it waits until the reader has scrolled to it.
        let plays_on = named("on")
            .and_then(|v| match v {
                Expr::EnumCase(case) => Some(case.clone()),
                _ => None,
            })
            .or_else(|| {
                ui.modifiers
                    .iter()
                    .find(|m| m.as_str() == "enterView")
                    .cloned()
            });
        // `count: "600ms"`: the number counts to where it has got to.
        // `format(v, .currency)` is split apart so the count is over the
        // number and the formatting is applied to each step of it.
        if let Some(over) = named("count") {
            let positional = ui.args.iter().find_map(|a| match a {
                Arg::Positional(e) => Some(e.clone()),
                _ => None,
            });
            if let Some(expr) = positional {
                let (value, format) = match &expr {
                    Expr::FunctionCall(f, args) if f == "format" && !args.is_empty() => {
                        let tail: Vec<String> =
                            args[1..].iter().map(|a| self.emit_expr(a)).collect();
                        let call = if tail.is_empty() {
                            "WF.format(n)".to_string()
                        } else {
                            format!("WF.format(n, {})", tail.join(", "))
                        };
                        (self.emit_expr(&args[0]), format!("(n) => {call}"))
                    }
                    other => (self.emit_expr(other), "null".to_string()),
                };
                let over = self.emit_expr(over);
                self.emit_line(&format!(
                    "WF.counted({var}, () => {value}, {over}, {format});"
                ));
            }
        }
        if plays_on.as_deref() == Some("enterView")
            && let Some(animation) = animation_of(ui)
        {
            let timing = |key: &str| {
                named(key)
                    .map(|v| self.emit_expr(v))
                    .unwrap_or_else(|| "null".to_string())
            };
            self.emit_line(&format!(
                "WF.onEnterView({var}, \"{animation}\", {}, {}, {});",
                timing("duration"),
                timing("delay"),
                timing("easing")
            ));
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
        let mut classes = crate::codegen::builtin::class_list(base, &ui.modifiers);
        // `Grid(columns: { base: 1, md: 2 })` carries the class its rules
        // live under, and nothing else: the widths are the stylesheet's.
        classes.extend(crate::codegen::scoped_css::responsive_classes(ui));
        classes.join(" ")
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
                        .map(crate::codegen::builtin::easing_css)
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

        // `visible:` is any expression that reads state: a page's own
        // (`visible: open`), a store's (`visible: Ui.palette`), or a
        // condition (`visible: Ui.confirmId != null`). The first two are
        // written back when the browser closes the dialog itself.
        let visible = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "visible" => Some(v),
            _ => None,
        });
        let visible_binding = visible.map(|v| {
            let read = self.emit_expr(v);
            let write = match v {
                Expr::Identifier(_) if self.is_state_signal(v) => {
                    Some(format!("(v) => {}.set(v)", read.trim_end_matches("()")))
                }
                Expr::PropertyAccess(..) if self.is_store_member(v, &read) => {
                    Some(format!("(v) => {{ {read} = v; }}"))
                }
                _ => None,
            };
            format!(
                "() => {read}, {}",
                write.unwrap_or_else(|| "null".to_string())
            )
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
            "const {} = WF.el(\"dialog\", {{ className: \"{}\"{}{} }});",
            var,
            root_classes,
            labelled,
            self.wf_node_inline(ui)
        ));

        let content_var = self.fresh_var();
        let content_class = format!("{}__content", class);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\" }});",
            content_var, content_class
        ));

        if let Some(t) = title {
            let header_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"div\", {{ className: \"{}__header\" }}, WF.el(\"h3\", {{ id: \"{}\" }}, {}));",
                header_var, class, title_id, t
            ));
            self.emit_line(&format!("{}.appendChild({});", content_var, header_var));
        }

        let body_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}__body\" }});",
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
                "const {} = WF.el(\"div\", {{ className: \"{}__footer\"{} }});",
                footer_var, class, footer_wf
            ));
            for child in &footer_stmts {
                self.emit_statement_dom(child, &footer_var);
            }
            self.emit_line(&format!("{}.appendChild({});", content_var, footer_var));
        }

        self.emit_line(&format!("{}.appendChild({});", var, content_var));

        // Visibility binding. `WF.dialog` drives `showModal()`/`close()` and
        // writes the signal back when the browser closes the dialog itself — via
        // Escape or the backdrop — so the state cannot drift out of sync with
        // what is on screen.
        if let Some(binding) = visible_binding {
            self.emit_line(&format!("WF.dialog({var}, {binding});"));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    fn emit_tabs(&mut self, var: &str, ui: &UIElement, parent: &str) {
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Tabs", ui),
            self.wf_node_inline(ui)
        ));
        // `role="tablist"` and the tab/panel wiring below are what tell a screen
        // reader these buttons are tabs at all. Without them the widget is a row
        // of unrelated buttons next to unrelated divs.
        let nav_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"wf-tabs__nav\", role: \"tablist\" }});",
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
            // the arrow keys move between them (see `WF.tabs`).
            let btn_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"button\", {{ className: () => {}() === {} ? \"wf-tabs__tab active\" : \"wf-tabs__tab\",                  role: \"tab\", type: \"button\", id: \"wf-tab-{}-{}\",                  \"aria-controls\": \"wf-tabpanel-{}-{}\",                  \"aria-selected\": () => {}() === {} ? \"true\" : \"false\",                  tabindex: () => {}() === {} ? 0 : -1,                  \"on:click\": () => {}.set({}) }}, {});",
                btn_var, active_var, i, group, i, group, i, active_var, i, active_var, i, active_var, i, label
            ));
            self.emit_line(&format!("{}.appendChild({});", nav_var, btn_var));
        }

        self.emit_line(&format!("{}.appendChild({});", var, nav_var));

        // Create tab content
        for (i, (tab, _)) in tab_pages.iter().enumerate() {
            let page_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"div\", {{ className: \"wf-tab-page\", role: \"tabpanel\",                  id: \"wf-tabpanel-{}-{}\", \"aria-labelledby\": \"wf-tab-{}-{}\", tabindex: 0{} }});",
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
        self.emit_line(&format!("WF.tabs({}, {});", nav_var, active_var));

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    /// The attributes of a wrapped control that belong on its `<input>`:
    /// `aria-*`, `data-*`, `name`, `id`, `disabled`, `required`, `tabindex`
    /// — what a screen reader, a form handle or a stylesheet reads off it.
    fn control_input_attrs(attrs: &[String]) -> String {
        attrs
            .iter()
            .filter(|a| {
                let key = a.trim_start_matches('"');
                key.starts_with("aria-")
                    || key.starts_with("data-")
                    || ["name:", "id:", "disabled:", "required:", "tabindex:"]
                        .iter()
                        .any(|k| key.starts_with(k))
            })
            .map(|a| format!(", {a}"))
            .collect()
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
            "const {} = WF.el(\"label\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Switch", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(state) = &bind_var {
            let input_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"input\", {{ type: \"checkbox\", role: \"switch\",                  checked: () => _{}(), \"aria-checked\": () => _{}() ? \"true\" : \"false\",                  \"on:change\": () => _{}.set(!_{}()){} }});",
                input_var,
                state,
                state,
                state,
                state,
                Self::control_input_attrs(attrs)
            ));
            self.emit_line(&format!("{}.appendChild({});", var, input_var));
        }

        let track_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"span\", {{ className: \"wf-switch__track\" }}, WF.el(\"span\", {{ className: \"wf-switch__thumb\" }}));",
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
        attrs: &[String],
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
            "const {} = WF.el(\"label\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr(name, ui),
            wf
        ));

        let input_var = self.fresh_var();
        let mut input_attrs = format!("type: \"{}\"", input_type);
        input_attrs.push_str(&Self::control_input_attrs(attrs));
        // Radios bound to one state are one group: the arrow keys move
        // between them only when they share a `name`.
        if name == "Radio"
            && let Some(state) = &bind_var
            && !attrs.iter().any(|a| a.starts_with("name:"))
        {
            input_attrs.push_str(&format!(", name: \"{state}\""));
        }

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
            let body = self.emit_event_body(handler);
            input_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                Self::handler_head(handler, "e"),
                body
            ));
        }

        self.emit_line(&format!(
            "const {} = WF.el(\"input\", {{ {} }});",
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
            "const {} = WF.el(\"div\", {{ className: () => {}() ? \"{} open\" : \"{}\"{} }});",
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
            "const {} = WF.el(\"button\", {{ className: \"wf-btn\", type: \"button\",              \"aria-haspopup\": \"true\", \"aria-controls\": \"{}\",              \"aria-expanded\": () => {}() ? \"true\" : \"false\",              \"on:click\": () => {}.set(!{}()) }}, {});",
            trigger_var, items_id, open_var, open_var, open_var, label
        ));
        self.emit_line(&format!("{}.appendChild({});", var, trigger_var));

        // The items are `li`s (the generic `Name.Item`), so their container is
        // a list; `role="menu"` gives it the menu semantics.
        let items_var = self.fresh_var();
        let items_class = format!("{}__items", class);
        self.emit_line(&format!(
            "const {} = WF.el(\"ul\", {{ className: \"{}\", id: \"{}\", role: \"menu\" }});",
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
            "const {} = WF.el(\"aside\", {{ className: \"{}\", id: \"wf-sidebar-{}\"{} }});",
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
                                    "const {} = WF.el(\"div\", {{ className: \"wf-sidebar__header\"{} }});",
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
                                        "const {} = WF.el(\"a\", {{ className: \"wf-sidebar__item\", href: {} {}{}{} }});",
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
                                        "const {} = WF.el(\"div\", {{ className: \"wf-sidebar__item\"{} }});",
                                        item_var, self.wf_node_inline(ui_child)
                                    ));
                            }
                            if let Some(ic) = icon {
                                self.emit_line(&format!(
                                        "{}.appendChild(WF.el(\"span\", {{ className: \"wf-icon\", \"data-icon\": {} }}));",
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
                                    "{}.appendChild(WF.el(\"div\", {{ className: \"wf-sidebar__divider\"{} }}));",
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
            "const {} = WF.el(\"div\", {{ className: \"wf-sidebar__scrim\", hidden: true }});",
            scrim_var
        ));
        self.emit_line(&format!(
            "const {} = WF.el(\"button\", {{ className: \"wf-sidebar__toggle\", type: \"button\", \"aria-label\": \"Open navigation\", \"aria-expanded\": \"false\", \"aria-controls\": \"wf-sidebar-{}\" }}, \"\\u2630\");",
            toggle_var, sidebar_id
        ));
        self.emit_line(&format!("{}.appendChild({});", parent, scrim_var));
        self.emit_line(&format!("{}.appendChild({});", parent, toggle_var));
        self.emit_line(&format!(
            "WF.drawer({}, {}, {});",
            var, toggle_var, scrim_var
        ));
    }

    fn emit_breadcrumb(&mut self, var: &str, ui: &UIElement, parent: &str) {
        self.emit_line(&format!(
            "const {} = WF.el(\"nav\", {{ className: \"{}\", \"aria-label\": \"breadcrumb\"{} }});",
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
                            "const {} = WF.el(\"a\", {{ className: \"wf-breadcrumb__item\", href: {}{}{} }});",
                            item_var, bp, href, self.wf_node_inline(ui_child)
                        ));
                    } else {
                        self.emit_line(&format!(
                            "const {} = WF.el(\"span\", {{ className: \"wf-breadcrumb__item\"{} }});",
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
            "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
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
            "const {} = WF.el(\"span\", {{ className: \"wf-tooltip__text\", role: \"tooltip\", id: \"{}\" }}, {});",
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
                "const {} = WF.el(\"div\", {{ className: \"{}\"{} }}, WF.el(\"img\", {{ src: {}, alt: {} }}));",
                var, cls, wf, img_src, alt_val
            ));
        } else if let Some(init) = initials {
            self.emit_line(&format!(
                "const {} = WF.el(\"div\", {{ className: \"{}\"{} }}, {});",
                var, cls, wf, init
            ));
        } else {
            self.emit_line(&format!(
                "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
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
            "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
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
            "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Carousel", ui),
            self.wf_node_inline(ui)
        ));

        let track_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"wf-carousel__track\" }});",
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
                        "const {} = WF.el(\"div\", {{ className: \"wf-carousel__slide\"{}{} }});",
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
        // A `class:` joins the engine's classes; it used to replace them, so
        // the button lost its size and shape.
        if let Some(Arg::Named(_, Expr::StringLiteral(extra))) = ui
            .args
            .iter()
            .find(|a| matches!(a, Arg::Named(k, _) if k == "class"))
        {
            cls.push(' ');
            cls.push_str(extra);
        }

        let icon_attr = if self.is_reactive(&icon) {
            format!("() => {icon}")
        } else {
            icon.clone()
        };
        let mut btn_attrs = format!("className: \"{}\", \"data-icon\": {}", cls, icon_attr);
        if let Some(l) = &label {
            let l = if self.is_reactive(l) {
                format!("() => {l}")
            } else {
                l.clone()
            };
            btn_attrs.push_str(&format!(", \"aria-label\": {}", l));
        }
        let title = label.as_deref().unwrap_or(&icon);
        let title = if self.is_reactive(title) {
            format!("() => {title}")
        } else {
            title.to_string()
        };
        btn_attrs.push_str(&format!(", title: {title}"));

        // Every other named argument is an attribute, as on a Button: `type`,
        // `disabled`, `aria-haspopup`, `data-variant`. A value that reads
        // state is a thunk the runtime keeps in step with it.
        for arg in &ui.args {
            if let Arg::Named(k, v) = arg {
                if matches!(k.as_str(), "icon" | "label")
                    || (k == "class" && matches!(v, Expr::StringLiteral(_)))
                {
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

        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            btn_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                Self::handler_head(handler, "event"),
                body
            ));
        }
        if let Some(entry) = self.wf_node_entry(ui) {
            btn_attrs.push_str(&format!(", {}", entry));
        }

        self.emit_line(&format!(
            "const {} = WF.el(\"button\", {{ {} }}, WF.el(\"span\", {{ className: \"wf-icon\", \"data-icon\": {} }}));",
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
            "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
            var,
            self.class_attr("Slider", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"label\", {{ className: \"wf-form-label\" }}, {});",
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
                .map(|h| self.emit_event_body(h))
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
            let body = self.emit_event_body(handler);
            input_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                Self::handler_head(handler, "event"),
                body
            ));
        }
        self.emit_line(&format!(
            "const {} = WF.el(\"input\", {{ {} }});",
            input_var, input_attrs
        ));
        self.emit_line(&format!("{}.appendChild({});", var, input_var));

        // Show current value if bound
        if bind_var.is_some() && announces_value {
            // The author shows it.
        } else if let Some(state) = &bind_var {
            let val_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"span\", {{ className: \"wf-slider__value\" }}, () => String(_{}()));",
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
            "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
            wrapper_var,
            self.class_attr("DatePicker", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"label\", {{ className: \"wf-form-label\" }}, {});",
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
            let body = self.emit_event_body(handler);
            input_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                Self::handler_head(handler, "event"),
                body
            ));
        }
        self.emit_line(&format!(
            "const {} = WF.el(\"input\", {{ {} }});",
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
            "const {} = WF.el(\"div\", {{ className: \"{}\"{} }});",
            wrapper_var,
            self.class_attr("FileUpload", ui),
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"label\", {{ className: \"wf-form-label\" }}, {});",
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
            let body = self.emit_event_body(handler);
            input_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                Self::handler_head(handler, "event"),
                body
            ));
        }
        self.emit_line(&format!(
            "const {} = WF.el(\"input\", {{ {} }});",
            input_var, input_attrs
        ));
        self.emit_line(&format!("{}.appendChild({});", wrapper_var, input_var));

        self.emit_line(&format!("const {} = {};", var, wrapper_var));
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    // ─── Control flow (DOM) ──────────────────────────

    fn emit_if_dom(&mut self, if_stmt: &IfStmt, parent: &str) {
        let value = self.emit_expr(&if_stmt.condition);
        // `if let x = e`: the branch shows while the value is not null, and
        // reads it as a plain name.
        let cond = match &if_stmt.binding {
            Some(_) => format!("{} != null", value),
            None => value.clone(),
        };

        self.emit_line(&format!("WF.when({},", parent));
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
        let bound = self.loop_bindings.len();
        if let Some(name) = &if_stmt.binding {
            self.emit_line(&format!("const {} = {};", name, value));
            self.loop_bindings.push(name.clone());
        }
        for stmt in &if_stmt.then_body {
            self.emit_statement_dom(stmt, &then_var);
        }
        self.loop_bindings.truncate(bound);
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
                binding: None,
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

        self.emit_line(&format!("WF.each({},", parent));
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

        // The options: the animation, `by key` — the identity of an item
        // across renders — and whether the body reads its index, in which
        // case an item is rebuilt when its position changes.
        let mut parts = Self::animate_config_parts(&for_stmt.animate);
        if let Some(key) = &for_stmt.key {
            self.loop_bindings.push(for_stmt.item.clone());
            let key_js = self.emit_expr(key);
            self.loop_bindings.pop();
            parts.push(format!("key: ({}) => {}", for_stmt.item, key_js));
            if for_stmt.index.is_some() {
                parts.push("index: true".to_string());
            }
        }
        if parts.is_empty() {
            self.emit_line("null");
        } else {
            self.emit_line(&format!("{{ {} }}", parts.join(", ")));
        }

        self.indent -= 1;
        self.emit_line(");");
    }

    fn emit_show_dom(&mut self, show_stmt: &ShowStmt, parent: &str) {
        let cond = self.emit_expr(&show_stmt.condition);

        self.emit_line(&format!("WF.show({},", parent));
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
        let parts = Self::animate_config_parts(config);
        if parts.is_empty() {
            self.emit_line("null");
        } else {
            self.emit_line(&format!("{{ {} }}", parts.join(", ")));
        }
    }

    /// The fields of an animation config object, none for no animation.
    fn animate_config_parts(config: &Option<AnimateConfig>) -> Vec<String> {
        let mut parts = Vec::new();
        if let Some(anim) = config {
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
        }
        parts
    }

    /// `resource rows = fetch(url, opts)`: the request, made once here and
    /// again whenever a URL that reads state changes.
    fn emit_resource(&mut self, r: &ResourceDecl) {
        // `resource rows = Backend.rows(page: n)`: an endpoint, which
        // carries its own address, headers, cache and abort. The arguments
        // are read again on every load, so one that reads state makes the
        // request again when it changes.
        if let Expr::MethodCall(object, endpoint, args) = &r.url {
            let call = format!("{}.{}", self.emit_expr(object), endpoint);
            let named = args.iter().find_map(|a| match a {
                Expr::MapLiteral(pairs) => Some(pairs),
                _ => None,
            });
            let positional: Vec<&Expr> = args
                .iter()
                .filter(|a| !matches!(a, Expr::MapLiteral(_)))
                .collect();
            let mut parts: Vec<String> = Vec::new();
            // `on:` is the resource's — when to look again — and every
            // other argument is the request's.
            let mut refetch = None;
            let mut paginate = None;
            let pairs: Vec<(String, Expr)> = named.into_iter().flatten().cloned().collect();
            for (key, value) in &pairs {
                let key = key.trim_matches('"');
                if key == "on" {
                    refetch = Some(self.refetch_policy(value));
                    continue;
                }
                // `paginate: .page` — `.items` gathers the pages, and
                // `loadMore()` moves the state the page number came from.
                if key == "paginate" {
                    let by = match value {
                        Expr::EnumCase(name) => name.clone(),
                        _ => "page".to_string(),
                    };
                    let set = pairs
                        .iter()
                        .find(|(k, _)| k.trim_matches('"') == by)
                        .and_then(|(_, v)| match v {
                            Expr::Identifier(name) => Some(format!(", set: (v) => _{name}.set(v)")),
                            _ => None,
                        })
                        .unwrap_or_default();
                    paginate = Some(format!("{{ by: \"{by}\"{set} }}"));
                    continue;
                }
                let emitted = match key {
                    "cache" => self.cache_policy(value),
                    _ => self.emit_expr(value),
                };
                parts.push(format!("{key}: {emitted}"));
            }
            // A single positional argument is the one the path names.
            if let Some(first) = positional.first() {
                parts.insert(0, format!("id: {}", self.emit_expr(first)));
            }
            let on = match refetch {
                Some(policy) => format!(", on: {policy}"),
                None => String::new(),
            };
            let paging = match paginate {
                Some(policy) => format!(", paginate: {policy}"),
                None => String::new(),
            };
            self.emit_line(&format!(
                "const _{} = WF.resource({call}, {{ call: true, args: () => ({{ {} }}){on}{paging} }});",
                r.name,
                parts.join(", ")
            ));
            return;
        }
        let url = self.emit_expr(&r.url);
        let url_js = if self.is_reactive(&url) {
            format!("() => {}", url)
        } else {
            url
        };
        let opts: Vec<String> = r
            .options
            .iter()
            .map(|opt| format!("{}: {}", opt.key, self.emit_expr(&opt.value)))
            .collect();
        let opts_js = if opts.is_empty() {
            "null".to_string()
        } else {
            format!("{{ {} }}", opts.join(", "))
        };
        self.emit_line(&format!(
            "const _{} = WF.resource({}, {});",
            r.name, url_js, opts_js
        ));
    }

    /// `.swr(60.seconds)`, `.none`, `.forever`, `.cache(5.minutes)` — how
    /// long an answer stands, as the engine reads it.
    fn cache_policy(&self, value: &Expr) -> String {
        match value {
            Expr::EnumCase(name) => format!("{{ kind: \"{name}\" }}"),
            Expr::CaseValue(name, args) => {
                let ttl = args
                    .first()
                    .map(|a| self.emit_expr(a))
                    .unwrap_or_else(|| "0".to_string());
                format!("{{ kind: \"{name}\", ttl: {ttl} }}")
            }
            other => self.emit_expr(other),
        }
    }

    /// `.backoff(times: 3, on: [.network, .status5xx])`, or a plain count —
    /// how many times to ask again, and for what.
    fn retry_policy(&self, value: &Expr) -> String {
        match value {
            Expr::NumberLiteral(n) => format!("{{ times: {n} }}"),
            Expr::EnumCase(name) if name == "never" => "{ times: 0 }".to_string(),
            Expr::CaseValue(_, args) => {
                let mut parts: Vec<String> = Vec::new();
                for arg in args {
                    let Expr::MapLiteral(pairs) = arg else {
                        continue;
                    };
                    for (key, v) in pairs {
                        let key = key.trim_matches('"');
                        // The kinds it answers to are words the engine knows.
                        let emitted = if key == "on" {
                            match v {
                                Expr::ListLiteral(items) => format!(
                                    "[{}]",
                                    items
                                        .iter()
                                        .map(|i| match i {
                                            Expr::EnumCase(name) => format!("\"{name}\""),
                                            other => self.emit_expr(other),
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ),
                                other => self.emit_expr(other),
                            }
                        } else {
                            self.emit_expr(v)
                        };
                        parts.push(format!("{key}: {emitted}"));
                    }
                }
                format!("{{ {} }}", parts.join(", "))
            }
            other => self.emit_expr(other),
        }
    }

    /// `.focus`, `.reconnect`, `.interval(30.seconds)`, `.never` — when to
    /// look again.
    fn refetch_policy(&self, value: &Expr) -> String {
        let one = |e: &Expr| -> Option<String> {
            match e {
                Expr::EnumCase(name) if name == "never" => Some(String::new()),
                Expr::EnumCase(name) => Some(format!("{name}: true")),
                Expr::CaseValue(name, args) => Some(format!(
                    "{name}: {}",
                    args.first()
                        .map(|a| self.emit_expr(a))
                        .unwrap_or_else(|| "0".to_string())
                )),
                _ => None,
            }
        };
        let parts: Vec<String> = match value {
            Expr::ListLiteral(items) => items.iter().filter_map(one).collect(),
            other => one(other).into_iter().collect(),
        };
        if parts.is_empty() {
            return self.emit_expr(value);
        }
        format!("{{ {} }}", parts.join(", "))
    }

    /// What the state's declared type says its values must be.
    fn refinement_of(&self, name: &str) -> Vec<(String, Expr)> {
        refinement_in(&self.current_body, name)
    }

    /// A `Video` with captions, or an `Audio` with a transcript: what a
    /// reader who cannot hear it needs, beside it rather than instead.
    fn emit_media(&mut self, name: &str, var: &str, attrs: &str, ui: &UIElement, parent: &str) {
        let tag = crate::codegen::builtin::builtin_to_html(name).0;
        self.emit_line(&format!("const {var} = WF.el(\"{tag}\", {attrs});"));
        let named = |key: &str| {
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key => Some(v),
                _ => None,
            })
        };
        if let Some(captions) = named("captions") {
            let src = self.emit_expr(captions);
            self.emit_line(&format!(
                "{var}.appendChild(WF.el(\"track\", {{ kind: \"captions\", src: {src}, srclang: document.documentElement.lang || \"en\", default: true }}));"
            ));
        }
        self.emit_line(&format!("{parent}.appendChild({var});"));
        if let Some(transcript) = named("transcript") {
            let href = self.emit_expr(transcript);
            self.emit_line(&format!(
                "{parent}.appendChild(WF.el(\"a\", {{ className: \"wf-transcript\", href: {href} }}, \"Read the transcript\"));"
            ));
        }
    }

    /// `Image(hero, alt: "…", sizes: "…", placeholder: .blur)` — the
    /// picture the build made, at every width it made.
    fn emit_picture(&mut self, var: &str, ui: &UIElement, parent: &str) {
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
        let source = positional
            .or_else(|| named("source"))
            .map(|v| self.emit_expr(v))
            .unwrap_or_default();
        let mut opts: Vec<String> = Vec::new();
        for key in ["alt", "sizes", "loading", "className"] {
            if let Some(value) = named(key) {
                opts.push(format!("{key}: {}", self.emit_expr(value)));
            }
        }
        // `placeholder: .blur` is a case by the time it is here.
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
        if let Some(kind) = placeholder {
            opts.push(format!("placeholder: \"{kind}\""));
        }
        let classes = crate::codegen::builtin::builtin_to_html("Image").1;
        opts.push(format!("className: \"{classes}\""));
        self.emit_line(&format!(
            "const {var} = WF.picture({source}, {{ {} }});",
            opts.join(", ")
        ));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// `validate email { required  email }`: the rules, registered against
    /// the state they guard and the form it is in.
    ///
    /// What comes back is the message to show, which the control picks up
    /// on its own — so a validated field needs no `error:` written on it.
    fn emit_validate(&mut self, v: &ValidateDecl) {
        let mut rules: Vec<String> = Vec::new();
        // A refined type validates itself: `state age: Number(18..=120)`
        // needs no rule written for the range it already declares.
        for (name, bound) in self.refinement_of(&v.name) {
            let rule = match name.as_str() {
                "min" | "max" | "minLength" | "maxLength" | "pattern" => name.clone(),
                "below" => "max".to_string(),
                _ => continue,
            };
            rules.push(format!(
                "{{ name: \"{rule}\", args: [{}] }}",
                self.emit_expr(&bound)
            ));
        }
        for rule in &v.rules {
            let mut parts = vec![format!("name: \"{}\"", rule.name)];
            if !rule.args.is_empty() {
                let args: Vec<String> = rule
                    .args
                    .iter()
                    .map(|a| {
                        let emitted = self.emit_expr(a);
                        // A rule that names another value follows it.
                        if self.is_reactive(&emitted) {
                            format!("() => {emitted}")
                        } else {
                            emitted
                        }
                    })
                    .collect();
                parts.push(format!("args: [{}]", args.join(", ")));
            }
            if let Some(message) = &rule.message {
                parts.push(format!("message: () => {}", self.emit_expr(message)));
            }
            if let Some(body) = &rule.body {
                let check = self.emit_expr(body);
                parts.push(if rule.name == "async" {
                    format!("check: async () => {check}")
                } else {
                    format!("check: () => {check}")
                });
            }
            rules.push(format!("{{ {} }}", parts.join(", ")));
        }
        let form = match &self.current_form {
            Some(name) => format!(", form: {name}"),
            None => String::new(),
        };
        self.emit_line(&format!(
            "const _{}_check = WF.validate(() => _{}(), [{}], {{ name: \"{}\"{form} }});",
            v.name,
            v.name,
            rules.join(", "),
            v.name
        ));
    }

    /// `socket chat = ws(…)`, `stream t = sse(…)`, `channel c =
    /// broadcast(…)`: a connection the scope owns, and closes.
    fn emit_connection(&mut self, c: &ConnectionDecl) {
        let opener = match c.kind {
            ConnectionKind::Socket => "ws",
            ConnectionKind::Stream => "sse",
            ConnectionKind::Channel => "broadcast",
            ConnectionKind::Peer => "rtc",
        };
        // A peer's only argument is its options; the rest open an address.
        let url_js = c.url.as_ref().map(|url| {
            let url = self.emit_expr(url);
            if self.is_reactive(&url) {
                format!("() => {url}")
            } else {
                url
            }
        });
        let mut options: Vec<String> = c
            .options
            .iter()
            .map(|(key, value)| format!("{key}: {}", self.emit_expr(value)))
            .collect();
        // `on message(m) { … }`: what arrives, handled where it is opened.
        for handler in &c.handlers {
            let param = handler
                .param
                .clone()
                .unwrap_or_else(|| "message".to_string());
            let body = self.emit_event_body(handler);
            options.push(format!(
                "on{}: ({param}) => {{ {body} }}",
                capitalize_first(&handler.event)
            ));
        }
        let args = match url_js {
            Some(url) => format!("{url}, {{ {} }}", options.join(", ")),
            None => format!("{{ {} }}", options.join(", ")),
        };
        self.emit_line(&format!("const _{} = WF.{opener}({args});", c.name));
    }

    /// `match x { … }`: one arm shown at a time, chosen by a resource's
    /// state or an enum's case, and re-chosen when it changes.
    fn emit_match_dom(&mut self, m: &MatchStmt, parent: &str) {
        let over_resource = m.arms.iter().any(|a| {
            matches!(
                a.pattern,
                ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready
            )
        });
        // A connection is matched by the state it is in, and its arms are
        // handed what that state carries: the closure, or the failure.
        let over_connection = m
            .arms
            .iter()
            .any(|a| matches!(a.pattern, ArmPattern::State(_)));
        let subject = self.emit_expr(&m.scrutinee);
        let (key, arg) = if over_connection {
            (
                format!("() => {subject}.state()"),
                format!(
                    "() => {subject}.state() === \"error\" ? {subject}.error() : {subject}.closure()"
                ),
            )
        } else if over_resource {
            (
                format!("() => {}.state()", subject),
                format!(
                    "() => {}.state() === \"error\" ? {}.error() : {}.data()",
                    subject, subject, subject
                ),
            )
        } else {
            // An enum: the arm is chosen by the case, and handed the payload.
            (
                format!("() => WF.caseOf({})", subject),
                format!("() => {}", subject),
            )
        };
        self.emit_line(&format!("WF.match({}, {}, {}, {{", parent, key, arg));
        self.indent += 1;
        for arm in &m.arms {
            let name = match &arm.pattern {
                ArmPattern::Loading => "loading".to_string(),
                ArmPattern::Error => "error".to_string(),
                ArmPattern::Ready => "ready".to_string(),
                ArmPattern::Case(c) => c.clone(),
                ArmPattern::State(s) => s.clone(),
                ArmPattern::Else => "else".to_string(),
            };
            let param = arm.binding.clone().unwrap_or_else(|| "_v".to_string());
            self.emit_line(&format!("{}: ({}) => {{", name, param));
            self.indent += 1;
            // The bound name is a plain parameter inside the arm, as is each
            // name a `.case(a, b)` arm binds to the payload.
            self.loop_bindings.push(param.clone());
            let bound = arm.bindings.len();
            for (i, bound_name) in arm.bindings.iter().enumerate() {
                self.emit_line(&format!("const {} = {}[{}];", bound_name, param, i + 1));
                self.loop_bindings.push(bound_name.clone());
            }
            let var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = document.createDocumentFragment();",
                var
            ));
            for stmt in &arm.body {
                self.emit_statement_dom(stmt, &var);
            }
            self.emit_line(&format!("return {};", var));
            for _ in 0..bound {
                self.loop_bindings.pop();
            }
            self.loop_bindings.pop();
            self.indent -= 1;
            self.emit_line("},");
        }
        self.indent -= 1;
        self.emit_line("});");
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
                let path = self.url_value(expr);
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
                    "WF.animate(\"{}\", \"{}\"{});",
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
                // `if let x = e` binds the value for the branch, which runs
                // when it is not null.
                let bound = self.loop_bindings.len();
                let cond = match &if_stmt.binding {
                    Some(name) => {
                        let value = self.emit_expr(&if_stmt.condition);
                        self.emit_line(&format!("const {} = {};", name, value));
                        // The branch reads it as a plain name — it is a
                        // `const` here, not a signal of the body's own.
                        self.loop_bindings.push(name.clone());
                        format!("{} != null", name)
                    }
                    None => self.emit_expr(&if_stmt.condition),
                };
                self.emit_line(&format!("if ({}) {{", cond));
                self.indent += 1;
                for s in &if_stmt.then_body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                self.loop_bindings.truncate(bound);
                // An `else if` chain used to be dropped here.
                for (cond, body) in &if_stmt.else_if_branches {
                    let cond = self.emit_expr(cond);
                    self.emit_line(&format!("}} else if ({}) {{", cond));
                    self.indent += 1;
                    for s in body {
                        self.emit_statement(s);
                    }
                    self.indent -= 1;
                }
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
            StatementKind::Fetch(_) => {}
            // `for x in xs { … }` in an action: the loop runs once, in order;
            // the item and the index are plain names inside it.
            StatementKind::For(f) => {
                let list = self.emit_expr(&f.iterable);
                match &f.index {
                    Some(index) => self.emit_line(&format!(
                        "for (const [{index}, {}] of Array.from({list}).entries()) {{",
                        f.item
                    )),
                    None => self.emit_line(&format!("for (const {} of {list}) {{", f.item)),
                }
                self.loop_bindings.push(f.item.clone());
                if let Some(index) = &f.index {
                    self.loop_bindings.push(index.clone());
                }
                self.indent += 1;
                for s in &f.body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                if f.index.is_some() {
                    self.loop_bindings.pop();
                }
                self.loop_bindings.pop();
                self.emit_line("}");
            }
            StatementKind::Return(expr) => {
                if let Some(e) = expr {
                    let val = self.emit_expr(e);
                    self.emit_line(&format!("return {};", val));
                } else {
                    self.emit_line("return;");
                }
            }
            StatementKind::Try(t) => {
                self.emit_line("try {");
                self.indent += 1;
                for s in &t.body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                let param = t.param.clone().unwrap_or_else(|| "_error".to_string());
                self.emit_line(&format!("}} catch ({param}) {{"));
                self.loop_bindings.push(param);
                self.indent += 1;
                for s in &t.catch_body {
                    self.emit_statement(s);
                }
                self.indent -= 1;
                self.loop_bindings.pop();
                self.emit_line("}");
            }
            // `emit toggle(id)`: the handler the caller passed for the
            // event, when it passed one.
            StatementKind::Emit(e) => {
                let args: Vec<String> = e.args.iter().map(|a| self.emit_expr(a)).collect();
                let rest = if args.is_empty() {
                    String::new()
                } else {
                    format!(", {}", args.join(", "))
                };
                self.emit_line(&format!("WF.emit(_p, \"{}\"{});", e.event, rest));
            }
            // Statements that render — an element in an action body — have
            // nowhere to go; the parsers keep them out of imperative blocks.
            _ => {}
        }
    }

    /// The statements of a handler, action or effect as one line of code,
    /// for a handler written inside an attribute object.
    fn emit_statements_inline(&mut self, stmts: &[Statement]) -> String {
        let saved = std::mem::take(&mut self.output);
        let indent = std::mem::replace(&mut self.indent, 0);
        for stmt in stmts {
            self.emit_statement(stmt);
        }
        let out = std::mem::replace(&mut self.output, saved);
        self.indent = indent;
        out.split('\n')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A handler's body: the same as any imperative block. The handler's
    /// named parameter (`on click(e)`) is a plain name inside it; a handler
    /// of the original grammar reads an implicit `event`.
    fn emit_event_body(&mut self, handler: &EventHandler) -> String {
        match &handler.param {
            Some(param) => {
                self.loop_bindings.push(param.clone());
                let body = self.emit_statements_inline(&handler.body);
                self.loop_bindings.pop();
                body
            }
            None => self.emit_statements_inline(&handler.body),
        }
    }

    /// The head of a handler's arrow function: its parameter, and `async`
    /// when the body awaits.
    fn handler_head(handler: &EventHandler, default_param: &str) -> String {
        let param = handler.param.as_deref().unwrap_or(default_param);
        if crate::parser::ast::awaits(&handler.body) {
            format!("async ({})", param)
        } else {
            format!("({})", param)
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
            // A scalar is its carrier: a plain JSON value, and nothing else.
            Expr::Typed(_, carrier) => self.emit_expr(carrier),
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
            Expr::Regex(pattern, flags) => format!("/{pattern}/{flags}"),
            Expr::Spread(inner) => format!("...{}", self.emit_expr(inner)),
            Expr::Range(a, b, inclusive) => format!(
                "WF.range({}, {}, {})",
                self.emit_expr(a),
                self.emit_expr(b),
                inclusive
            ),
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
                // `BROWSER_GLOBALS`, below: shared with the type checker.
                // A global, unless the program declared the name itself: a
                // `state history` is the page's, not the browser's.
                if BROWSER_GLOBALS.contains(&name.as_str())
                    && !self.own_names.contains(name)
                    && !self.current_props.contains(name)
                    && !self.page_params.contains(name)
                    && !self.loop_bindings.contains(name)
                    && !self.lambda_params.borrow().contains(name)
                {
                    return name.to_string();
                }
                // The browser as values, kept current by the runtime, unless
                // the name is the writer's own.
                if BROWSER_VALUES.contains(&name.as_str())
                    && !self.own_names.contains(name)
                    && !self.current_props.contains(name)
                    && !self.page_params.contains(name)
                    && !self.loop_bindings.contains(name)
                    && !self.lambda_params.borrow().contains(name)
                {
                    return format!("WF.{name}()");
                }
                // Store references, component props, and built-in names stay as-is
                if self.current_props.contains(name) {
                    return format!("_p.{}", name);
                }
                if self.resources.contains(name) {
                    return format!("_{}", name);
                }
                if self.page_params.contains(name) {
                    return format!("params.{}", name);
                }
                // The names the generated code uses for a route's
                // parameters, a handler's event and a keyed loop's
                // bindings. They are read as themselves — unless the page
                // declared one of them, in which case the declaration
                // wins: a `state key` used to compile to a bare global,
                // so it rendered as nothing and never updated.
                const IMPLICIT: &[&str] = &["params", "value", "key", "event", "e"];
                // An action named as a value — `addEventListener("scroll",
                // track)`, a callback handed to a library — is the function
                // itself. It used to be read as a signal, `_track()`, which
                // does not exist.
                if self.own_actions.contains(name) && !self.lambda_params.borrow().contains(name) {
                    return name.to_string();
                }
                if self.stores.contains(name)
                    || self.consts.contains(name)
                    || self.refs.contains(name)
                    || name == "env"
                    || self.loop_bindings.contains(name)
                    || self.lambda_params.borrow().contains(name)
                    || (IMPLICIT.contains(&name.as_str()) && !self.own_names.contains(name))
                    || name.starts_with("_")
                {
                    name.to_string()
                } else {
                    // State variable (signal) — access via _name()
                    format!("_{}()", name)
                }
            }
            // `save.pending` and `Store.save.pending`: whether a call of an
            // async action is under way, a signal.
            Expr::PropertyAccess(base, prop) if prop == "pending" && self.is_action_ref(base) => {
                match base.as_ref() {
                    Expr::Identifier(name) => format!("_{name}_pending()"),
                    other => format!("{}.pending()", self.emit_expr(other)),
                }
            }
            // What a resource or a connection holds — `rows.state`,
            // `rows.data`, `rows.error`, a paged resource's `rows.items` and
            // `rows.hasMore`, a socket's `chat.messages` and `chat.closure` —
            // is a signal on the handle. It is read where it is written, so
            // `Text("{rows.state}")` shows the state and follows it; it used to
            // show the signal's own source, once.
            Expr::PropertyAccess(base, prop)
                if matches!(
                    prop.as_str(),
                    "state" | "data" | "error" | "items" | "hasMore" | "messages" | "closure"
                ) && matches!(base.as_ref(), Expr::Identifier(n)
                    if self.resources.contains(n)
                        && !self.lambda_params.borrow().contains(n)
                        && !self.loop_bindings.contains(n)) =>
            {
                format!("{}.{}()", self.emit_expr(base), prop)
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
            // `?.` is JavaScript's own.
            Expr::OptionalProperty(base, prop) => format!("{}?.{}", self.emit_expr(base), prop),
            Expr::OptionalIndex(base, index) => {
                format!("{}?.[{}]", self.emit_expr(base), self.emit_expr(index))
            }
            Expr::OptionalMethod(obj, method, args) => {
                let args: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                format!("{}?.{}({})", self.emit_expr(obj), method, args.join(", "))
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
                    BinOp::NullCoalesce => "??",
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
                // The case of an enum value, its payload aside.
                if method == "__case" && args.is_empty() {
                    return format!("WF.caseOf({})", self.emit_expr(obj));
                }
                // `match` over an enum as a value: the case, and the payload
                // where the case is the one asked for.
                if (method == "__is" || method == "__payload") && args.len() == 1 {
                    let subject = self.emit_expr(obj);
                    let case = self.emit_expr(&args[0]);
                    return if method == "__is" {
                        format!("(WF.caseOf({subject}) === {case})")
                    } else {
                        format!("WF.payload({subject}, {case})")
                    };
                }
                if method == "__if" && args.len() == 2 {
                    // Conditional expression
                    let cond = self.emit_expr(obj);
                    let then_val = self.emit_expr(&args[0]);
                    let else_val = self.emit_expr(&args[1]);
                    return format!("({} ? {} : {})", cond, then_val, else_val);
                }
                // `if let x = e { a } else { b }`: the value is bound once,
                // and read as a plain name in `a`.
                if method == "__iflet"
                    && args.len() == 2
                    && let Expr::Lambda(name, then_expr) = &args[0]
                {
                    let value = self.emit_expr(obj);
                    self.lambda_params.borrow_mut().push(name.clone());
                    let then_val = self.emit_expr(then_expr);
                    self.lambda_params.borrow_mut().pop();
                    let else_val = self.emit_expr(&args[1]);
                    return format!(
                        "(({name}) => {name} != null ? {then_val} : {else_val})({value})"
                    );
                }

                let obj_str = self.emit_expr(obj);
                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();

                // A store's own actions come first. `Todos.remove(id)` calls
                // the action the store declares, not the list method of the
                // name; a store is an object, and mapping the call onto a
                // list's `splice` leaves it calling something that is not
                // there. The list methods stay for `Todos.items.remove(0)`,
                // which reaches the list rather than the store.
                if matches!(obj.as_ref(), Expr::Identifier(s) if self.stores.contains(s)) {
                    return format!("{}.{}({})", obj_str, method, args_str.join(", "));
                }

                // Map WebFluent methods to JS, through the one table the
                // store emitter reads too.
                let holder = if self.is_state_signal(obj) {
                    Holder::Signal
                } else if self.is_store_member(obj, &obj_str) {
                    Holder::StoreMember
                } else {
                    Holder::Plain
                };
                method_to_js(method, &obj_str, &args_str, holder)
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

                // `now(every: 1.seconds)`: the browser's clock, ticking as often
                // as the page asks. It compiled to a bare `now(…)`, which
                // nothing declares, and threw where the page read it.
                if name == "now" && !self.own_names.contains(name) {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.now({})", args_str.join(", "));
                }
                // WF runtime functions
                if name == "replayAnimation" {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.replay({})", args_str.join(", "));
                }
                // `format(n, .currency)`, `ago(date)`: the runtime's, unless
                // the page declares an action of the name.
                if matches!(
                    name.as_str(),
                    // `beacon(url, data)` outlives the page that sends it,
                    // which is the whole of why it exists.
                    // `uuid()` is where a `Uuid` comes from: the platform's
                    // generator, or a version-4 layout where there is none.
                    "format" | "ago" | "setTheme" | "beacon" | "optimistic" | "sanitize" | "uuid"
                ) && !self.own_actions.contains(name)
                {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.{name}({})", args_str.join(", "));
                }

                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                // `await fetch(url, opts)`: the parsed body, as a `resource`
                // reads it; a failed response throws.
                if name == "fetch" && !self.own_actions.contains(name) {
                    return format!("WF.request({})", args_str.join(", "));
                }
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
            Expr::MapLiteral(entries) | Expr::Record(_, entries) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| match k.as_str() {
                        "..." => format!("...{}", self.emit_expr(v)),
                        _ => format!("{}: {}", k, self.emit_expr(v)),
                    })
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
            // An enum case is its name: the code generators read cases as
            // the words they always were (`"primary"`), and a user enum's
            // case is a string at run time.
            Expr::EnumCase(case) => format!("\"{}\"", case),
            Expr::CaseValue(case, args) => {
                let mut items = vec![format!("\"{case}\"")];
                items.extend(args.iter().map(|a| self.emit_expr(a)));
                format!("[{}]", items.join(", "))
            }
            // A design token is its custom property.
            Expr::Token(name) => format!("\"var(--{})\"", name),
            Expr::Await(inner) => format!("(await {})", self.emit_expr(inner)),
        }
    }

    // ─── Helpers ─────────────────────────────────────

    fn emit_line(&mut self, text: &str) {
        let indent = "  ".repeat(self.indent);
        self.output.push_str(&format!("{}{}\n", indent, text));
    }

    fn fresh_var(&self) -> String {
        let n = self.next_var.get();
        self.next_var.set(n + 1);
        format!("_e{}", n)
    }

    fn emit_component_args(&self, component: &str, args: &[Arg]) -> String {
        let mut parts = Vec::new();
        for arg in args {
            match arg {
                // A motion marker lands on the component's root, not in its props.
                Arg::Named(name, _) if name.starts_with("data-wf-") => {}
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
                // A positional argument binds to the component's positional
                // prop. It used to be written into the object bare, which is
                // not a property.
                Arg::Positional(expr) => {
                    if let Some(prop) = self.component_positional.get(component) {
                        let value = self.emit_expr(expr);
                        if self.is_reactive(&value) {
                            parts.push(format!("get {}() {{ return {}; }}", prop, value));
                        } else {
                            parts.push(format!("{}: {}", prop, value));
                        }
                    }
                }
            }
        }
        if parts.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", parts.join(", "))
        }
    }
}

/// The order routes are matched in: the more specific first — every static
/// segment before a `:param`, and the catch-all `*` last — so that neither
/// the order of `Route`s nor of pages, which is file-system order, can
/// shadow a route.
fn route_order(path: &str) -> (bool, usize, std::cmp::Reverse<usize>) {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let wildcard = path.trim() == "*" || segments.contains(&"*");
    let params = segments.iter().filter(|s| s.starts_with(':')).count();
    let statics = segments
        .iter()
        .filter(|s| !s.starts_with(':') && **s != "*")
        .count();
    (wildcard, params, std::cmp::Reverse(statics))
}

/// The names of the resources a body declares, at any depth.
fn resource_names(stmts: &[Statement]) -> Vec<String> {
    let mut names = Vec::new();
    fn walk(stmts: &[Statement], names: &mut Vec<String>) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::Resource(r) => names.push(r.name.clone()),
                // A socket, a stream and a channel are held the same
                // way: the handle itself, not a signal over one, so
                // `chat.state()` is what a `match` over it reads.
                StatementKind::Connection(c) => names.push(c.name.clone()),
                StatementKind::If(i) => {
                    walk(&i.then_body, names);
                    for (_, b) in &i.else_if_branches {
                        walk(b, names);
                    }
                    if let Some(b) = &i.else_body {
                        walk(b, names);
                    }
                }
                StatementKind::For(f) => walk(&f.body, names),
                StatementKind::Show(s) => walk(&s.body, names),
                StatementKind::Match(m) => {
                    for arm in &m.arms {
                        walk(&arm.body, names);
                    }
                }
                StatementKind::UIElement(el) => walk(&el.children, names),
                _ => {}
            }
        }
    }
    walk(stmts, &mut names);
    names
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
    /// A `pending` signal for each async action of the body, declared
    /// before anything reads it.
    fn emit_pending_signals(&mut self, body: &[Statement]) {
        for stmt in body {
            if let StatementKind::Action(a) = &stmt.kind
                && crate::parser::ast::awaits(&a.body)
            {
                self.emit_line(&format!("const _{}_pending = WF.signal(false);", a.name));
            }
        }
    }

    /// Whether `expr` is a bare name that resolves to one of the body's own
    /// signals — a `state`, `persist` or `let` — rather than a prop, a loop
    /// binding, a constant or a global.
    fn is_state_signal(&self, expr: &Expr) -> bool {
        let Expr::Identifier(name) = expr else {
            return false;
        };
        let emitted = self.emit_expr(expr);
        emitted == format!("_{name}()")
    }

    /// Whether `expr` reaches a store's state: `Cart.items` from a page, or
    /// bare `items` inside the store's own body, emitted as `store.items`.
    fn is_store_member(&self, expr: &Expr, emitted: &str) -> bool {
        match expr {
            Expr::PropertyAccess(base, _) => {
                matches!(base.as_ref(), Expr::Identifier(s) if self.stores.contains(s))
            }
            Expr::Identifier(name) => emitted == format!("store.{name}"),
            _ => false,
        }
    }

    /// Whether `expr` names an action: one of the page's own, or a store's.
    fn is_action_ref(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(name) => self.own_actions.contains(name),
            Expr::PropertyAccess(base, _) => {
                matches!(base.as_ref(), Expr::Identifier(store) if self.stores.contains(store))
            }
            _ => false,
        }
    }

    fn is_reactive(&self, expr_str: &str) -> bool {
        is_reactive_expr(expr_str)
            || expr_str.contains("_p.")
            || self
                .stores
                .iter()
                .any(|s| expr_str.contains(&format!("{s}.")))
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
    if expr_str.contains("WF.i18n.t(")
        || expr_str.contains("WF.i18n.locale()")
        || expr_str.contains("WF.i18n.dir()")
    {
        return true;
    }
    // The browser as values — each a signal the runtime keeps current. Only
    // conditions were drawn live without this, so `Text("{viewport.width}")`,
    // a clock reading `now` or `network.online` in text was computed once
    // and never moved.
    BROWSER_VALUES
        .iter()
        .any(|name| expr_str.contains(&format!("WF.{name}(")))
}

/// The names the language reads from the browser, each a live value:
/// `viewport.md`, `query.tab`, `now`, `network.online`, `update.available`.
/// The browser's own names a program may use unprefixed: each compiles to
/// itself. The type checker reads the same list, so a name here is never
/// reported as declared by nothing.
pub const BROWSER_GLOBALS: &[&str] = &[
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
    "navigator",
    "location",
    "history",
    "screen",
    "performance",
    "crypto",
    "globalThis",
    "Intl",
    "Symbol",
    "Reflect",
    "URL",
    "URLSearchParams",
    "FormData",
    "Blob",
    "File",
    "FileReader",
    "Event",
    "CustomEvent",
    "AbortController",
    "TextEncoder",
    "TextDecoder",
    "Notification",
    "matchMedia",
    "getComputedStyle",
    "structuredClone",
    "queueMicrotask",
];

pub const BROWSER_VALUES: &[&str] = &[
    "viewport", "query", "hash", "theme", "now", "network", "update",
];

/// The runtime function a method of one of the language's own types
/// compiles to: `due.plus(days: 3)` is `WF.plus(due, { days: 3 })`.
///
/// Routed by name, because the value carries its own kind at run time —
/// and each function leaves a receiver that has a method of that name to
/// answer for itself, so a record with its own `plus` is never taken over.
/// The helpers the runtime adds to lists and strings, each a runtime function
/// of the same name: `items.sortBy(f)` is `WF.sortBy(items, f)`.
pub const LIST_AND_STRING_HELPERS: &[&str] = &[
    "sortBy",
    "groupBy",
    "unique",
    "take",
    "first",
    "last",
    "capitalize",
    "truncate",
    "dedent",
    "lines",
    "words",
];

/// How the receiver of a method call is held, which decides whether a
/// mutating method assigns back through a signal, through a store's setter,
/// or mutates the value in place.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// A local, a parameter, a literal: nothing reactive reads it.
    Plain,
    /// A page or component `state`, read as `_name()`.
    Signal,
    /// A store's state: `Cart.items` from outside, `store.items` within.
    StoreMember,
}

/// The one table mapping a WebFluent method onto JavaScript.
///
/// Both the page emitter (`emit_expr`) and the store emitter
/// (`emit_store_expr`) read it. They used to keep a copy each, and the
/// copies drifted: `remove`, `contains`, `toUpper`, `toLower` and all 41 of
/// the scalar methods were in one and not the other, so inside a store
/// action they compiled to `store.items.remove(i)` and `store.due.plus(…)`
/// — methods no JavaScript array, string or JSON value has. Every one threw
/// the first time its action ran, and nothing before this point could see
/// it: parsing, the types and the linters all pass on the source that
/// produces them.
pub fn method_to_js(method: &str, obj: &str, args: &[String], holder: Holder) -> String {
    let joined = args.join(", ");
    let with_obj = |name: &str| {
        if args.is_empty() {
            format!("WF.{name}({obj})")
        } else {
            format!("WF.{name}({obj}, {joined})")
        }
    };

    match method {
        // `items.push(x)` and `items.remove(i)` mutate. On a signal or a
        // store's state the new list is assigned back, so what reads it
        // repaints and a persisted one is written; an in-place change would
        // leave the holder with the array it already had.
        "push" => match holder {
            Holder::Signal => {
                format!("{}.set([...{obj}, {joined}])", obj.trim_end_matches("()"))
            }
            Holder::StoreMember => format!("({obj} = [...{obj}, {joined}])"),
            Holder::Plain => format!("{obj}.push({joined})"),
        },
        "remove" => match holder {
            // The comma leaves the new list as the value of the expression.
            // `set` itself returns nothing, so `items = items.remove(i)`
            // used to set the list to the nothing the inner call handed
            // back — the removal happened, then the list became undefined.
            // Reading the signal again costs no new subscription: the
            // argument above already read it.
            Holder::Signal => {
                let sig = obj.trim_end_matches("()");
                format!("({sig}.set(WF.removeAt({obj}, {joined})), {obj})")
            }
            Holder::StoreMember => format!("({obj} = WF.removeAt({obj}, {joined}))"),
            Holder::Plain => format!("{obj}.splice({joined}, 1)"),
        },
        "filter" => format!("{obj}.filter({joined})"),
        "map" => format!("{obj}.map({joined})"),
        "sum" => format!("{obj}.reduce((a,b) => a+b, 0)"),
        "length" => format!("{obj}.length"),
        "toUpper" => format!("{obj}.toUpperCase()"),
        "toLower" => format!("{obj}.toLowerCase()"),
        "contains" => format!("{obj}.includes({joined})"),
        "trim" => format!("{obj}.trim()"),
        "split" => format!("{obj}.split({joined})"),
        // What the language's own types can do: a date's arithmetic,
        // money's, a URL's parts, a colour's mixing. Each is a plain JSON
        // value at run time, so the work is the runtime's, never a method
        // on the value.
        m if scalar_method(m).is_some() => with_obj(scalar_method(m).expect("just checked")),
        // The helpers the runtime adds to lists and strings.
        m if LIST_AND_STRING_HELPERS.contains(&m) => with_obj(m),
        _ => format!("{obj}.{method}({joined})"),
    }
}

/// What the language's own types can do — a date's arithmetic, money's, a
/// URL's parts, a colour's mixing — each as the runtime function that does
/// it. Each is a plain JSON value at run time, so the work is always the
/// runtime's, never a method on the value. A list rather than a `match`, so
/// `tests/build_and_runtime_agree.rs` can hold every one of them to the
/// runtime and to the static paint.
pub const SCALAR_METHODS: &[(&str, &str)] = &[
    ("year", "year"),
    ("month", "month"),
    ("day", "day"),
    ("weekday", "weekday"),
    ("hour", "hour"),
    ("minute", "minute"),
    ("second", "second"),
    ("native", "native"),
    ("plus", "plus"),
    ("minus", "minus"),
    ("isBefore", "isBefore"),
    ("isAfter", "isAfter"),
    ("isSame", "isSame"),
    ("until", "until"),
    ("startOfDay", "startOfDay"),
    ("startOfWeek", "startOfWeek"),
    ("startOfMonth", "startOfMonth"),
    ("endOfDay", "endOfDay"),
    ("inZone", "inZone"),
    ("days", "days"),
    ("hours", "hours"),
    ("minutes", "minutes"),
    ("seconds", "seconds"),
    ("ms", "ms"),
    ("times", "times"),
    ("convert", "convert"),
    ("host", "host"),
    ("path", "path"),
    ("domain", "domain"),
    ("mix", "mix"),
    ("lighten", "lighten"),
    ("darken", "darken"),
    ("alpha", "alpha"),
    ("contrast", "contrast"),
    ("preview", "preview"),
    // Named apart from the browser's own `query`, `date` and `time`.
    ("date", "dateOf"),
    ("time", "timeOf"),
    ("query", "urlQuery"),
    ("with", "urlWith"),
];

/// The runtime function a scalar method compiles to.
pub fn scalar_method(name: &str) -> Option<&'static str> {
    SCALAR_METHODS
        .iter()
        .find(|(method, _)| *method == name)
        .map(|(_, runtime)| *runtime)
}

/// Whether a body shows something before the server has agreed, which is
/// what makes the action one that can be taken back.
fn shows_optimistically(stmts: &[Statement]) -> bool {
    fn in_expr(e: &Expr) -> bool {
        if matches!(e, Expr::FunctionCall(name, _) if name == "optimistic") {
            return true;
        }
        e.children().into_iter().any(in_expr)
    }
    stmts.iter().any(|s| {
        s.kind.exprs().into_iter().any(in_expr)
            || s.kind.bodies().iter().any(|b| shows_optimistically(b))
    })
}

/// What a `state`'s declared type says its values must be, when it says
/// anything: `state nights: Number(1..=30)` is a rule before a `validate`
/// block adds one.
fn refinement_in(stmts: &[Statement], name: &str) -> Vec<(String, Expr)> {
    for stmt in stmts {
        if let StatementKind::State(s) = &stmt.kind
            && s.name == name
            && let Some(ty) = &s.ty
        {
            return ty.refinement().to_vec();
        }
        let found = refinement_in_bodies(stmt, name);
        if !found.is_empty() {
            return found;
        }
    }
    Vec::new()
}

fn refinement_in_bodies(stmt: &Statement, name: &str) -> Vec<(String, Expr)> {
    for body in stmt.kind.bodies() {
        let found = refinement_in(body, name);
        if !found.is_empty() {
            return found;
        }
    }
    Vec::new()
}

/// The states a body's `validate` blocks guard.
fn validated_names(stmts: &[Statement]) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            if let StatementKind::Validate(v) = &stmt.kind {
                out.push(v.name.clone());
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    walk(stmts, &mut out);
    out
}

/// `Form(show: .live)` — when a message is shown — as the runtime takes it.
fn form_options(stmts: &[Statement]) -> String {
    fn find(stmts: &[Statement]) -> Option<String> {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &stmt.kind
                && matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Form")
            {
                for arg in &el.args {
                    if let Arg::Named(key, Expr::EnumCase(case)) = arg
                        && key == "show"
                    {
                        return Some(case.clone());
                    }
                }
                // An enum prop written as a flag is a modifier by now.
                for modifier in &el.modifiers {
                    if matches!(modifier.as_str(), "onBlur" | "onSubmit" | "live") {
                        return Some(modifier.clone());
                    }
                }
            }
            for body in stmt.kind.bodies() {
                if let Some(found) = find(body) {
                    return Some(found);
                }
            }
            if let StatementKind::UIElement(el) = &stmt.kind
                && let Some(found) = find(&el.children)
            {
                return Some(found);
            }
        }
        None
    }
    match find(stmts) {
        Some(show) => format!("{{ show: \"{show}\" }}"),
        None => String::new(),
    }
}

/// The enter animation an element asked for, however it was written.
/// `publishableKey` as an attribute is `publishable-key`: what a custom
/// element's `observedAttributes` says, and what HTML can carry.
fn kebab_case(name: &str) -> String {
    if name.contains('-') {
        return name.to_string();
    }
    let mut out = String::with_capacity(name.len() + 4);
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn animation_of(ui: &UIElement) -> Option<String> {
    for arg in &ui.args {
        match arg {
            // An element that waits to be scrolled to keeps its animation
            // here, where no class picks it up at the first paint.
            Arg::Named(key, Expr::StringLiteral(name)) if key == "data-wf-enter" => {
                return Some(name.clone());
            }
            Arg::Named(key, Expr::EnumCase(case)) if key == "animate" => {
                return Some(case.clone());
            }
            _ => {}
        }
    }
    ui.modifiers
        .iter()
        .find(|m| crate::themes::prune::ANIMATIONS.contains(&m.as_str()))
        .cloned()
}

/// A word with its first letter upper-cased: `message` → `Message`.
fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The word `fetch` knows a setting's case by: `.sameOrigin` is
/// `same-origin`, and the rest are themselves.
fn fetch_word(case: &str) -> String {
    match case {
        "sameOrigin" => "same-origin".to_string(),
        "noCors" => "no-cors".to_string(),
        "noReferrer" => "no-referrer".to_string(),
        other => other.to_string(),
    }
}

/// The name the runtime knows a service's setting by.
fn api_setting(key: &str) -> &str {
    match key {
        "referrer" => "referrerPolicy",
        other => other,
    }
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

/// The module a program's `external` declarations compile to.
///
/// It is a **separate file**, not part of the bundle, for one reason: a
/// module has its own scope, and the page chunks a split build writes are
/// classic scripts that read the bundle's names from the global one. So
/// the imports live here, each bound to a global, and the bundle stays
/// what it was. A module script and a deferred classic script run in the
/// order they appear in the document, so this has run before `app.js`
/// reads any of it.
pub fn externals_module(program: &Program) -> Option<String> {
    let modules: Vec<&crate::parser::ast::ExternalDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::External(e) if e.kind == crate::parser::ast::ExternalKind::Module => {
                Some(e)
            }
            _ => None,
        })
        .collect();
    if modules.is_empty() {
        return None;
    }
    let mut out = String::from("// Somebody else's code, named as this project names it.\n");
    for e in &modules {
        out.push_str(&format!(
            "import * as {} from {};\n",
            e.name,
            serde_json::to_string(&e.from).unwrap_or_default()
        ));
    }
    // A module's export may be a class — Chart.js's `Chart`, most modern
    // libraries' main export — and a WebFluent call has no `new`. Calling a
    // class without it throws, so each export is copied onto a plain object
    // (a module namespace cannot be proxied), a class behind a proxy that
    // constructs it when it is called, its static members as they were.
    out.push_str(
        "const __wfCallable = (ns) => { const o = {}; for (const k of Object.keys(ns)) { const v = ns[k]; \
         o[k] = typeof v === \"function\" && /^class[\\s{]/.test(Function.prototype.toString.call(v)) \
         ? new Proxy(v, { apply: (c, _, args) => Reflect.construct(c, args) }) : v; } return o; };\n",
    );
    for e in &modules {
        out.push_str(&format!("globalThis.{0} = __wfCallable({0});\n", e.name));
    }
    Some(out)
}

/// Every module a program imports, with the hash it declared for it.
pub fn external_modules(program: &Program) -> Vec<(String, Option<String>)> {
    program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::External(e) if e.kind == crate::parser::ast::ExternalKind::Module => {
                Some((e.from.clone(), e.integrity.clone()))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(src: &str) -> String {
        let program = crate::syntax::parse_source(src, "<t>").expect("parse");
        JsCodegen::new().generate(&crate::sema::lower(program))
    }

    #[test]
    fn a_services_hook_runs_what_it_says_with_its_parameter_bound() {
        let out = compile(concat!(
            "store L { state last = \"\"\n action note(s: String) { last = s } }\n",
            "api B(base: \"/api\") {\n",
            "  on request(r) { r.headers[\"X-Id\"] = \"abc\"  L.note(r.url) }\n",
            "  get me() -> Map\n}\n",
            "page P(path: \"/\") { resource m = B.me()\n Text(L.last)\n",
            "  match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
        ));
        // The body is emitted at all, and `r` is the parameter, not a signal.
        assert!(out.contains("r.headers[\"X-Id\"] = \"abc\""), "{out}");
        assert!(out.contains("L.note(r.url)"), "{out}");
        assert!(!out.contains("_r()"), "{out}");
    }

    #[test]
    fn a_match_over_a_connection_reads_the_handle_not_a_signal() {
        let out = compile(concat!(
            "page P(path: \"/\") {\n",
            "  socket chat = ws(\"wss://e.com/c\")\n",
            "  match chat { open { Text(\"on\") } else { Text(\"off\") } } }",
        ));
        assert!(out.contains("_chat.state()"), "{out}");
        assert!(!out.contains("_chat()"), "{out}");
    }

    #[test]
    fn an_if_let_in_an_imperative_body_binds_a_plain_name() {
        let out = compile(concat!(
            "page P(path: \"/\") {\n",
            "  state price = 0\n",
            "  stream ticks = sse(\"/events\")\n",
            "  effect { if let p = ticks.last(\"price\") { price = p } }\n",
            "  Text(\"{price}\") }",
        ));
        assert!(out.contains("_price.set(p)"), "{out}");
        assert!(!out.contains("_p()"), "{out}");
    }

    #[test]
    fn a_sockets_handler_reads_what_arrived_by_name() {
        let out = compile(concat!(
            "store F { state items = []\n action add(m: Map) { items = items.concat([m]) } }\n",
            "page P(path: \"/\") {\n",
            "  socket chat = ws(\"wss://e.com/c\") { on message(m) { F.add(m) } }\n",
            "  Text(\"{F.items.length}\")\n",
            "  match chat { open { Text(\"on\") } else { Text(\"off\") } } }",
        ));
        assert!(out.contains("onMessage: (m) => { F.add(m); }"), "{out}");
    }

    // ─── Nodes the new grammar produces, built by hand until it parses ───

    fn stmt(kind: StatementKind) -> Statement {
        Statement::new(kind, Span::dummy())
    }

    fn element(name: &str, args: Vec<Arg>, children: Vec<Statement>) -> UIElement {
        UIElement {
            component: ComponentRef::BuiltIn(name.to_string()),
            args,
            modifiers: Vec::new(),
            children,
            style_block: None,
            transition_block: None,
            events: Vec::new(),
            slot_fills: Vec::new(),
            span: Span::dummy(),
            paren_span: None,
            body_span: None,
            style_span: None,
            arg_spans: Vec::new(),
            modifier_spans: Vec::new(),
        }
    }

    fn text(expr: Expr) -> Statement {
        stmt(StatementKind::UIElement(element(
            "Text",
            vec![Arg::Positional(expr)],
            Vec::new(),
        )))
    }

    fn page(name: &str, body: Vec<Statement>) -> Declaration {
        Declaration::Page(PageDecl {
            name: name.to_string(),
            path: "/".to_string(),
            title: None,
            title_expr: None,
            guard: None,
            redirect: None,
            description: None,
            image: None,
            page_type: None,
            noindex: false,
            layout: None,
            params: Vec::new(),
            head: Vec::new(),
            paths: None,
            body,
            span: Span::dummy(),
            header_span: Span::dummy(),
            body_span: Span::dummy(),
        })
    }

    fn generate(declarations: Vec<Declaration>) -> String {
        JsCodegen::new().generate(&Program { declarations })
    }

    #[test]
    fn a_resource_is_declared_once_and_a_match_renders_its_states() {
        let ident = |n: &str| Expr::Identifier(n.to_string());
        let arm = |pattern, binding: Option<&str>, body| MatchArm {
            pattern,
            binding: binding.map(str::to_string),
            bindings: Vec::new(),
            body,
            span: Span::dummy(),
        };
        let out = generate(vec![page(
            "Home",
            vec![
                stmt(StatementKind::Resource(ResourceDecl {
                    name: "rows".to_string(),
                    ty: None,
                    url: Expr::StringLiteral("/api/rows".to_string()),
                    options: Vec::new(),
                })),
                stmt(StatementKind::Match(MatchStmt {
                    scrutinee: ident("rows"),
                    arms: vec![
                        arm(
                            ArmPattern::Loading,
                            None,
                            vec![text(Expr::StringLiteral("…".into()))],
                        ),
                        arm(
                            ArmPattern::Error,
                            Some("e"),
                            vec![text(Expr::PropertyAccess(
                                Box::new(ident("e")),
                                "message".to_string(),
                            ))],
                        ),
                        arm(
                            ArmPattern::Ready,
                            Some("list"),
                            vec![text(Expr::PropertyAccess(
                                Box::new(ident("list")),
                                "length".to_string(),
                            ))],
                        ),
                    ],
                })),
                stmt(StatementKind::UIElement(element(
                    "Button",
                    vec![Arg::Positional(Expr::StringLiteral("Reload".into()))],
                    vec![stmt(StatementKind::ExprStatement(Expr::MethodCall(
                        Box::new(ident("rows")),
                        "reload".to_string(),
                        Vec::new(),
                    )))],
                ))),
            ],
        )]);
        assert!(
            out.contains("const _rows = WF.resource(\"/api/rows\", null);"),
            "{out}"
        );
        assert!(out.contains("WF.match(_root, () => _rows.state(), () => _rows.state() === \"error\" ? _rows.error() : _rows.data(), {"), "{out}");
        assert!(out.contains("loading: (_v) => {"), "{out}");
        assert!(
            out.contains("error: (e) => {") && out.contains("e.message"),
            "{out}"
        );
        assert!(
            out.contains("ready: (list) => {") && out.contains("list.length"),
            "{out}"
        );
        // A reference to the resource is the object, not a signal read.
        assert!(out.contains("_rows.reload()"), "{out}");
        assert!(!out.contains("_rows()"), "{out}");
    }

    #[test]
    fn a_match_over_an_enum_picks_the_arm_by_case() {
        let out = generate(vec![page(
            "Home",
            vec![
                stmt(StatementKind::State(StateDecl {
                    name: "tone".to_string(),
                    ty: None,
                    value: Expr::EnumCase("danger".to_string()),
                    persist: false,
                    policy: None,
                })),
                stmt(StatementKind::Match(MatchStmt {
                    scrutinee: Expr::Identifier("tone".to_string()),
                    arms: vec![
                        MatchArm {
                            pattern: ArmPattern::Case("danger".to_string()),
                            binding: None,
                            bindings: Vec::new(),
                            body: vec![text(Expr::StringLiteral("red".into()))],
                            span: Span::dummy(),
                        },
                        MatchArm {
                            pattern: ArmPattern::Else,
                            binding: None,
                            bindings: Vec::new(),
                            body: vec![text(Expr::StringLiteral("calm".into()))],
                            span: Span::dummy(),
                        },
                    ],
                })),
            ],
        )]);
        assert!(
            out.contains("WF.signal(\"danger\")"),
            "an enum case is its name: {out}"
        );
        assert!(
            out.contains("WF.match(_root, () => WF.caseOf(_tone()), () => _tone(), {"),
            "{out}"
        );
        assert!(
            out.contains("danger: (_v) => {") && out.contains("else: (_v) => {"),
            "{out}"
        );
    }

    #[test]
    fn a_declared_event_is_emitted_to_the_handler_the_caller_passed() {
        let component = Declaration::Component(ComponentDecl {
            name: "Row".to_string(),
            props: vec![PropDecl {
                name: "label".to_string(),
                prop_type: TypeRef::String,
                optional: false,
                default: None,
                positional: true,
                doc: None,
                span: Span::dummy(),
            }],
            events: vec![EventDecl {
                name: "pick".to_string(),
                params: Vec::new(),
                doc: None,
                span: Span::dummy(),
            }],
            slots: vec![SlotDecl {
                params: Vec::new(),
                name: Some("trailing".to_string()),
                span: Span::dummy(),
            }],
            parts: Vec::new(),
            doc: None,
            body: vec![
                stmt(StatementKind::UIElement({
                    let mut b = element(
                        "Button",
                        vec![Arg::Positional(Expr::Identifier("label".to_string()))],
                        Vec::new(),
                    );
                    b.events.push(EventHandler {
                        event: "click".to_string(),
                        param: Some("ev".to_string()),
                        key: None,
                        body: vec![stmt(StatementKind::Emit(EmitStmt {
                            event: "pick".to_string(),
                            args: vec![Expr::Identifier("label".to_string())],
                        }))],
                        span: Span::dummy(),
                    });
                    b
                })),
                stmt(StatementKind::UIElement(element(
                    "Children",
                    vec![Arg::Named(
                        "slot".to_string(),
                        Expr::StringLiteral("trailing".to_string()),
                    )],
                    Vec::new(),
                ))),
            ],
            span: Span::dummy(),
            header_span: Span::dummy(),
            body_span: Span::dummy(),
        });
        let mut call = UIElement {
            component: ComponentRef::UserDefined("Row".to_string()),
            ..element(
                "Row",
                vec![Arg::Positional(Expr::StringLiteral("Hi".into()))],
                Vec::new(),
            )
        };
        call.events.push(EventHandler {
            event: "pick".to_string(),
            param: Some("which".to_string()),
            key: None,
            body: vec![stmt(StatementKind::Log(Expr::Identifier(
                "which".to_string(),
            )))],
            span: Span::dummy(),
        });
        call.events.push(EventHandler {
            event: "mouseenter".to_string(),
            param: None,
            key: None,
            body: vec![stmt(StatementKind::Log(Expr::StringLiteral("in".into())))],
            span: Span::dummy(),
        });
        call.slot_fills.push(SlotFill {
            params: Vec::new(),
            name: "trailing".to_string(),
            body: vec![text(Expr::StringLiteral("→".into()))],
            span: Span::dummy(),
            body_span: Span::dummy(),
        });
        let out = generate(vec![
            component,
            page("Home", vec![stmt(StatementKind::UIElement(call))]),
        ]);
        // Inside the component: the handler's own parameter, and the emit.
        assert!(
            out.contains(
                "addEventListener(\"click\", (ev) => { WF.emit(_p, \"pick\", _p.label); })"
            ),
            "{out}"
        );
        assert!(
            out.contains(
                "if (typeof _slots.trailing === 'function') _frag.appendChild(_slots.trailing());"
            ),
            "{out}"
        );
        // At the call: the positional prop by name, the declared event as a
        // handler in the props, the DOM event on the root, the named fill.
        assert!(out.contains("Component_Row({ label: \"Hi\", on: { pick: (which) => { console.log(which); } } }, {"), "{out}");
        assert!(out.contains("trailing: () => {"), "{out}");
        assert!(
            out.contains("WF.onRoot(_e2, \"mouseenter\", (event) => { console.log(\"in\"); });"),
            "{out}"
        );
    }

    #[test]
    fn a_router_without_routes_takes_every_page_most_specific_first() {
        let out = compile(
            "page Deploy(path: \"/deploys/:id\", id: String) { Text(id) }\npage NotFound(path: \"*\") { Text(\"?\") }\npage Home(path: \"/\") { Text(\"h\") }\npage Deploys(path: \"/deploys\") { Text(\"d\") }\napp { Router }",
        );
        let table = out.find("const _routes = [").unwrap();
        let order: Vec<usize> = ["\"/deploys\"", "\"/\"", "\"/deploys/:id\"", "\"*\""]
            .iter()
            .map(|p| out[table..].find(&format!("path: {p}")).unwrap())
            .collect();
        assert!(
            order[0] < order[1] && order[1] < order[2] && order[2] < order[3],
            "{out}"
        );
        assert!(
            out.contains("Page_Deploy(params)") && out.contains("params.id"),
            "{out}"
        );
    }

    #[test]
    fn a_page_with_a_layout_is_rendered_inside_it() {
        let out = compile(
            "component Shell(crumb: String) { slot  Text(crumb)  children }\npage Home(path: \"/\", layout: Shell(crumb: \"Home\")) { Text(\"h\") }\napp { Router }",
        );
        assert!(
            out.contains("layout: (page, params) => Component_Shell({ crumb: \"Home\" }, { children: () => page(params) }), render: (params) => Page_Home(params)"),
            "{out}"
        );
    }

    #[test]
    fn if_let_binds_the_value_and_for_by_passes_the_key() {
        let out = generate(vec![page(
            "Home",
            vec![
                stmt(StatementKind::State(StateDecl {
                    name: "hint".to_string(),
                    ty: None,
                    value: Expr::Null,
                    persist: false,
                    policy: None,
                })),
                stmt(StatementKind::State(StateDecl {
                    name: "items".to_string(),
                    ty: None,
                    value: Expr::ListLiteral(Vec::new()),
                    persist: false,
                    policy: None,
                })),
                stmt(StatementKind::If(IfStmt {
                    condition: Expr::Identifier("hint".to_string()),
                    binding: Some("h".to_string()),
                    animate: None,
                    animate_span: None,
                    then_body: vec![text(Expr::Identifier("h".to_string()))],
                    else_if_branches: Vec::new(),
                    else_body: None,
                })),
                stmt(StatementKind::For(ForStmt {
                    item: "it".to_string(),
                    index: None,
                    iterable: Expr::Identifier("items".to_string()),
                    key: Some(Expr::PropertyAccess(
                        Box::new(Expr::Identifier("it".to_string())),
                        "id".to_string(),
                    )),
                    animate: None,
                    animate_span: None,
                    body: vec![text(Expr::Identifier("it".to_string()))],
                })),
                stmt(StatementKind::Action(ActionDecl {
                    name: "load".to_string(),
                    params: Vec::new(),
                    body: vec![stmt(StatementKind::Assignment(Assignment {
                        target: Expr::Identifier("hint".to_string()),
                        value: Expr::Await(Box::new(Expr::FunctionCall(
                            "fetch".to_string(),
                            vec![Expr::StringLiteral("/x".into())],
                        ))),
                    }))],
                })),
            ],
        )]);
        assert!(
            out.contains("() => _hint() != null") || out.contains("const h = _hint();"),
            "{out}"
        );
        assert!(
            out.contains("{ key: (it) => it.id }"),
            "the key function is in the list's options: {out}"
        );
        assert!(
            out.contains("async function load()") && out.contains("(await WF.request(\"/x\"))"),
            "{out}"
        );
    }

    #[test]
    fn motion_off_a_branch_root_is_a_marker_and_the_router_takes_a_transition() {
        let out = compile(
            "component Chip(_ label: String) { Badge(label) }\npage P(path: \"/\") {\n  Card { Text(\"x\", exit: .fadeOut, delay: \"100ms\")  Chip(\"y\", exit: .fadeOut).fadeIn.fast }\n}\napp { Router(transition: .slide, duration: \"150ms\") }",
        );
        assert!(
            out.contains("\"data-wf-exit\": \"fadeOut\", \"data-wf-delay\": \"100ms\""),
            "{out}"
        );
        assert!(
            out.contains("{ \"data-wf-exit\": \"fadeOut\", \"data-wf-animate\": \"fadeIn\", \"data-wf-duration\": \"150ms\" });"),
            "{out}"
        );
        assert!(
            out.contains(
                "WF.router(_routes, _routerEl, { transition: \"slide\", duration: \"150ms\" });"
            ),
            "{out}"
        );
    }

    /// `WF.each` hands the body its item as a plain callback parameter, so
    /// every reference to it must stay plain.
    ///
    /// The identifier path treated anything that was not a prop or a store as
    /// state, so a loop over `tasks` bound `task` and then read `_task()` —
    /// `ReferenceError` on the first non-empty list. `wf init -t spa` shipped it.
    #[test]
    fn a_component_receives_its_callers_block_as_children() {
        let out = compile(
            r#"
            component Panel(title: String) {
                Card { Heading(title).h3 children }
            }
            page P(path: "/") {
                state n = 1
                Panel(title: "Keys") { Text("count {n}") }
                Panel(title: "Empty")
            }
            
            "#,
        );
        assert!(
            out.contains("function Component_Panel(_p, _slots)"),
            "{out}"
        );
        assert!(
            out.contains("Component_Panel({ title: \"Keys\" }, {")
                && out.contains("children: () => {"),
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
            store Pricing {
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
            page P(path: "/") { use Pricing  Text("{Pricing.label}") }
            
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
            store S { state q = ""  action set(v: String) { q = v } }
            page P(path: "/") {
                use S
                state email = ""
                Input(email, bind: email, placeholder: "e") { on input { S.set(email) } }
                Form { on submit { S.set("sent") } Text("f")  }
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
            page P(path: "/") {
                state open = false
                Button("", aria-expanded: open) {
                    on click {
                        open = !open
                    }
                    Text("Details")
                    Icon("chevron-down")
                }
            }
            
            "#,
        );
        assert!(
            out.contains(".addEventListener(\"click\", (event) => { _open.set(!_open()); });"),
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
            page P(path: "/") {
                state n = 0
                List { List.Item(id: "first", aria-current: "true") { style { padding: 0; &:hover { color: red } } on click { n = n + 1 } Text("a") } }
            }
            
            "#,
        );
        assert!(out.contains("WF.el(\"li\", { className: \"wf-list__item\", id: \"first\", \"aria-current\": \"true\" })"), "{out}");
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
            store S {
                state items = [1, 2]
                derived pairs = items.map(i => { n: i, twice: i * 2 })
                derived counts = { all: items.length }
            }
            page P(path: "/") { use S  Text("x") }
            
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
            store S {
                state secret = ""
                action headers() {
                    h = {}
                    h["Authorization"] = "Bearer " + secret
                    h = h
                    return h
                }
            }
            page P(path: "/") { use S  Text("x") }
            
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
            store S {
                state at = 0
                action step(n: Number) { return n + 1 }
                action move(step: Number) { at = at + step }
            }
            page P(path: "/") { use S  Text("x") }
            
            "#,
        );
        assert!(out.contains("store.at = (store.at + step);"), "{out}");
        assert!(!out.contains("store.step"), "{out}");
    }

    #[test]
    fn a_known_attribute_that_reads_state_follows_it() {
        let out = compile(
            r#"
            store S { state busy = false  derived hint = if busy { "wait" } else { "type" } }
            page P(path: "/") {
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
            component Picker(value: String) { Select(value: value) { children } }
            page P(path: "/") { Picker(value: "b") { Select.Option("A", value: "a")  Select.Option("B", value: "b") } }
            
            "#,
        );
        assert!(
            out.contains("WF.el(\"select\", { className: \"wf-select\", value: () => _p.value }, (typeof _slots.children === 'function' ? _slots.children() : null))"),
            "{out}"
        );
        assert!(!out.contains("appendChild(_slots.children())"), "{out}");
    }

    #[test]
    fn an_else_if_chain_with_a_final_else_keeps_every_branch() {
        let out = compile(
            r#"
            page P(path: "/") {
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
            store S { state n = 0  action set(v: Number) { n = v } }
            page P(path: "/") {
                use S
                state req = 4
                Slider(bind: req, min: 0, max: 8, step: 1, aria-label: "Requests") { on input { S.set(req) } }
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
            page P(path: "/") {
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
            component Chip(label: String, pressed: Bool = false) {
                derived bg = if pressed { "a" } else { "b" }
                Button(label, aria-pressed: pressed) { style { background: {bg} } }
            }
            page P(path: "/") {
                state on = true
                Chip(label: "Errors", pressed: on) { on click { on = !on } }
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
            component Badge(label: String, tone: String = "neutral", dot: Bool = true) { Text(label) }
            page P(path: "/") { Badge(label: "x") }
            
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
            page P(path: "/") {
                state tone = "red"
                Card { style { --hover-bg: {tone}; --edge: 1px; &:hover { background: var(--hover-bg) } } }
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
            component Save(label: String) { Button(label) }
            page P(path: "/") {
                state n = 0
                Save(label: "Go") { on click { n = n + 1 } }
                Save(label: "Hover") { on mouseenter { n = 9 } }
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
            page Home(path: "/", title: "Home") { Text("h") }
            page Docs(path: "/docs", title: "Routing \"rules\"") { Text("d") }
            app { Router }
            
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
            page P(path: "/") {
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
            store S { state tone = "#fff" }
            page P(path: "/") {
                use S
                state pct = 40
                Card {
                    style {
                        width: {pct}%
                        background: {S.tone}
                        padding: 1rem
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
            store S { state pct = 5 }
            page P(path: "/") {
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
            page P(path: "/") {
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
            page Home(path: "/") { Text("hi") }
            app {
                Row(gap: .lg, align: .center) {
                    Router
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
            "page P(path: \"/\") {\n    state items = []\n    for item in items { Text(item.title) }\n}",
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
            "store S { state rows = []\n action pick(id: Number) { } }\npage P(path: \"/\") {\n    use S\n    for row in S.rows { Button(\"Pick\") { on click { S.pick(row.id) } } }\n}",
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
            "page P(path: \"/\") {\n    state items = []\n    for item, i in items { Text(\"{i}: {item.name}\") }\n}",
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
            "page P(path: \"/\") {\n    state groups = []\n    for group in groups {\n        for member in group.members { Text(member.name) }\n        Text(group.title)\n    }\n}",
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
        let js = compile("page P(path: \"/\") {\n    state count = 0\n    Text(\"{count}\")\n}");
        assert!(
            js.contains("_count()"),
            "state stopped being reactive:\n{js}"
        );
    }

    #[test]
    fn a_for_loop_in_an_action_runs_once_in_order() {
        let out = compile(
            "page P(path: \"/\") { state items = [1, 2]\n state n = 0\n action all() { for x in items { n = n + x }\n for x, i in items { log(i) } }\n Text(\"{n}\") }",
        );
        assert!(out.contains("for (const x of _items()) {"), "{out}");
        assert!(
            out.contains("_n.set((_n() + x));"),
            "the item is a plain name: {out}"
        );
        assert!(
            out.contains("for (const [i, x] of Array.from(_items()).entries()) {"),
            "{out}"
        );
    }

    #[test]
    fn optional_chaining_is_javascripts_own() {
        let out = compile(
            "page P(path: \"/\") { state user = null\n derived name = user?.profile?.name ?? \"anon\"\n derived first = user?.tags?.[0]\n derived up = user?.name?.toUpperCase()\n Text(name) }",
        );
        assert!(out.contains("_user()?.profile?.name ?? \"anon\""), "{out}");
        assert!(out.contains("_user()?.tags?.[0]"), "{out}");
        assert!(out.contains("_user()?.name?.toUpperCase()"), "{out}");
    }

    #[test]
    fn push_on_a_state_sets_a_new_list_so_readers_repaint() {
        let out = compile(
            "store Cart { state items: [String] = []\n action add(x: String) { items.push(x) } }\npage P(path: \"/\") { use Cart\n state tags: [String] = []\n state form = { tags: [\"a\"] }\n action go() { tags.push(\"x\")  Cart.items.push(\"y\")  form.tags.push(\"z\") }\n Text(\"{tags.length}\") }",
        );
        assert!(out.contains("_tags.set([..._tags(), \"x\"])"), "{out}");
        assert!(
            out.contains("(Cart.items = [...Cart.items, \"y\"])"),
            "{out}"
        );
        assert!(out.contains("(store.items = [...store.items, x])"), "{out}");
        // A nested list is the writer's own object: pushed in place.
        assert!(out.contains("_form().tags.push(\"z\")"), "{out}");
    }

    #[test]
    fn a_wrapped_control_carries_its_aria_data_and_name_attributes_on_the_input() {
        let out = compile(
            "page P(path: \"/\") { state on = false\n state pick = \"a\"\n Checkbox(bind: on, aria-label: \"Done\", name: \"done\", disabled: true)\n Radio(bind: pick, value: \"a\", label: \"A\", aria-describedby: \"h\")\n Switch(bind: on, label: \"S\", data-kind: \"x\") }",
        );
        assert!(out.contains("type: \"checkbox\", \"aria-label\": \"Done\", name: \"done\", disabled: true, checked:"), "{out}");
        assert!(
            out.contains("type: \"radio\", \"aria-describedby\": \"h\", name: \"pick\", checked:"),
            "{out}"
        );
        assert!(
            out.contains("role: \"switch\"") && out.contains("\"data-kind\": \"x\" })"),
            "{out}"
        );
    }

    #[test]
    fn a_store_action_keeps_its_own_names_inside_try_for_and_else_if() {
        let out = compile(
            "store S { state user: Map? = null\n state error = \"\"\n state n = 0\n action login(email: String, rows: [Map]) { try { let r = await fetch(\"/x\")\n user = r.user\n for row, i in rows { if row.ok { n = n + i } else if row.bad { log(row) } else { n = 0 } } } catch e { error = e.message } } }\npage P(path: \"/\") { use S\n Text(\"x\") }",
        );
        let i = out.find("login: async").unwrap();
        let action = &out[i..out[i..].find("},\n").map(|j| i + j).unwrap_or(out.len())];
        for expected in [
            "const r = (await WF.request(\"/x\"));",
            "store.user = r.user;",
            "for (const [i, row] of Array.from(rows).entries()) {",
            "if (row.ok) {",
            "store.n = (store.n + i);",
            "} else if (row.bad) {",
            "console.log(row);",
            "} catch (e) {",
            "store.error = e.message;",
        ] {
            assert!(action.contains(expected), "{expected} in {action}");
        }
        assert!(
            !action.contains("_user") && !action.contains("WF.signal("),
            "{action}"
        );
    }

    #[test]
    fn a_dialogs_visible_may_be_a_state_a_store_member_or_a_condition() {
        let out = compile(
            "store Ui { state palette = false\n state confirmId: String? = null }\npage P(path: \"/\") { use Ui\n state open = false\n Modal(visible: open, title: \"a\") { Text(\"x\") }\n Modal(visible: Ui.palette, title: \"b\") { Text(\"y\") }\n Dialog(visible: Ui.confirmId != null, title: \"c\") { Text(\"z\") } }",
        );
        assert!(
            out.contains("() => _open(), (v) => _open.set(v));"),
            "{out}"
        );
        assert!(
            out.contains("() => Ui.palette, (v) => { Ui.palette = v; });"),
            "{out}"
        );
        assert!(
            out.contains("() => (Ui.confirmId !== null), null);"),
            "{out}"
        );
    }

    #[test]
    fn an_icon_buttons_class_joins_the_engines_and_its_icon_follows_state() {
        let out = compile(
            "page P(path: \"/\") { state dark = false\n IconButton(icon: if dark { \"sun\" } else { \"moon\" }, label: \"Theme\", class: \"site-icon-button\") { on click { dark = !dark } } }",
        );
        assert!(
            out.contains("className: \"wf-icon-btn site-icon-button\""),
            "{out}"
        );
        assert!(
            out.contains("\"data-icon\": () => (_dark() ? \"sun\" : \"moon\")"),
            "{out}"
        );
        assert!(!out.contains("class: \"site-icon-button\""), "{out}");
    }

    #[test]
    fn a_part_with_a_destination_inside_a_loop_is_a_link() {
        let out = compile(
            "page P(path: \"/\") { state names = [\"a\"]\n Sidebar { for n in names by n { Sidebar.Item(to: \"/x/{n}\") { Text(n) } } } }",
        );
        assert!(
            out.contains("WF.el(\"a\", { className: \"wf-sidebar__item\", href: `/x/${n}`"),
            "{out}"
        );
        assert!(out.contains("WF.navigate(`/x/${n}`)"), "{out}");
        assert!(out.contains("WF.activeLink("), "{out}");
        assert!(!out.contains("to: `/x/${n}`"), "{out}");
    }
}
