use crate::project::builtin::{
    builtin_to_html, element_tag, implicit_role, input_type, is_void, landmark_label,
    layout_arg_classes, modifier_to_class,
};
pub(crate) use crate::registry::browser::is_root_attribute;
pub use crate::registry::browser::{
    BROWSER_GLOBALS, BROWSER_VALUES, DRAG_PROPS, GESTURE_EVENTS, SCALAR_METHODS,
};
use crate::syntax::node_id::NodeMap;
use crate::syntax::parser::ast::*;
use crate::web::runtime;
use std::collections::HashMap;

mod body;
mod dom;
mod expr;
mod externals;
mod names;
mod net;
mod reactive;
mod statement;
mod store;
#[cfg(test)]
mod tests;
mod widgets;

use crate::base::text::{camel_to_kebab, capitalize_first, kebab_case, to_camel_case};
use expr::*;
internal! {
    pub use expr::{Holder, LIST_AND_STRING_HELPERS, method_to_js, scalar_method};
}
use externals::*;
pub use externals::{external_modules, externals_module};
use names::*;
use net::*;
use reactive::*;
use statement::*;
use widgets::*;

/// What a connection's handle holds as signals.
pub const CONNECTION_SIGNALS: &[&str] = &["state", "error", "pending", "messages", "closure"];
/// What a resource's handle holds as signals.
pub const RESOURCE_SIGNALS: &[&str] = &[
    "state",
    "data",
    "error",
    "refreshing",
    "pending",
    "items",
    "hasMore",
];

/// What the emitter of a `Menu` and a `Dropdown` reads itself, whichever of
/// the two it draws: never an attribute of the element.
pub const MENU_READS: &[&str] = &[
    "trigger",
    "label",
    "visible",
    "anchor",
    "at",
    "placement",
    "offset",
];

/// What a control drawn in a box — `Switch`, `Checkbox`, `Radio`, `Slider`,
/// `DatePicker`, `FileUpload` — takes on the box rather than on its input:
/// what is the whole control's, its label and its input together. Every
/// other attribute is its input's, where its handlers are.
pub const BOX_ATTRS: &[&str] = &["hidden", "lang", "dir"];

/// The modes a bundle is built in, chosen once before
/// [`JsCodegen::generate`]. Each is off by default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct JsOptions {
    /// The pages were pre-rendered: the bundle takes each over in place
    /// rather than drawing it.
    pub ssg: bool,
    /// A build for the dev server: responses are held to their declared
    /// types at the network boundary.
    pub dev: bool,
    /// Each page is its own chunk, loaded when its route shows.
    pub split_pages: bool,
    /// Every runtime module ships, for a site whose own scripts reach for
    /// `WF` in ways a build cannot see.
    pub full_runtime: bool,
    /// The service worker `offline` in the config asks for is registered;
    /// `Some(true)` also keeps writes made offline (`sync`).
    pub offline: Option<bool>,
}

/// JavaScript code generator — compiles the AST to a JS bundle with reactivity and routing.
#[derive(Debug)]
pub struct JsCodegen {
    output: String,
    /// `build.sourcemap`: each declaration's source (by `map_key`) as an
    /// index into the map's sources; the one being emitted; and the marker
    /// the next line written starts with (`codegen::sourcemap`).
    map_files: Option<HashMap<String, usize>>,
    map_file: Option<usize>,
    pending_mark: Option<String>,
    /// A build for `wf serve`: each response a type is declared for carries
    /// that type's shape, which the runtime holds the response to.
    dev: bool,
    /// The program's records and enums, for those shapes.
    shapes: (HashMap<String, TypeDecl>, HashMap<String, Vec<String>>),
    /// `Some(sync)` when the config names `offline`: the service worker is
    /// registered at boot, and `sync` keeps writes made offline.
    offline: Option<bool>,
    /// `meta.scripts` entries loaded `"after"` the page, with their
    /// integrity hashes: asked for by the runtime once the page has painted.
    later: Vec<(String, Option<String>)>,
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
    /// The program's `const` names: plain values, read as written.
    consts: Vec<String>,
    /// The program's constants as build-time values, for what is drawn when
    /// the page is built (a chart, a QR code).
    static_scope: Option<crate::project::static_eval::Scope>,
    /// `env.NAME` values from the project's config, emitted once.
    env: std::collections::BTreeMap<String, serde_json::Value>,
    /// The menu lists being drawn, innermost last: a `Menu.Sub` opens
    /// beside the one it is in.
    menu_lists: Vec<String>,
    /// Every store's connections, as `(store, name)`: `Lang.css.pending`
    /// is a signal on the handle, as a page's own connection's is.
    store_connections: Vec<(String, String)>,
    /// The connections of the store being emitted.
    own_connections: Vec<String>,
    /// Every store's resources, as `(store, name)`: `Feed.rows.state` is a
    /// signal on the handle, as a page's own resource's is.
    store_resources: Vec<(String, String)>,
    /// The resources of the store being emitted.
    own_resources: Vec<String>,
    /// The options of the `VirtualList` whose `for` is drawn next.
    window_opts: Option<Vec<String>>,
    /// Names bound by an enclosing `for` loop, innermost last.
    ///
    /// `WF.each` calls the body with the item as a plain parameter, so a
    /// reference to it must stay plain. Treating it as state instead emitted
    /// `_item()` against a binding named `item` — a `ReferenceError` the moment
    /// a non-empty list rendered.
    loop_bindings: Vec<String>,
    /// The `let`s of the imperative block being emitted, innermost last: a
    /// signal of the block's own, which shadows a prop, a route parameter
    /// or a resource of the same name until the block ends.
    block_lets: Vec<String>,
    /// The positions in `loop_bindings` of the names bound live: a `match`
    /// arm's binding, an `if let`'s and a keyed `for`'s item and index. Each
    /// is handed to its body as an accessor `$name`, and read as `$name()`,
    /// so what reads it is drawn as what reads state is — and follows a
    /// refreshed response, a new payload or a changed row in place. A
    /// program's names cannot hold a `$`, so the accessor never stands in
    /// for a state's `_name`, a component's `_p` or an element's `_e3`.
    live_at: Vec<usize>,
    /// The `for` loops being emitted, innermost last: the item's name, the
    /// list it comes from and whether the loop is keyed — so `bind:
    /// t.title` writes the item and tells the list it changed.
    loop_sources: Vec<(String, Expr, bool)>,
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
    /// The app's own schemes a frame's address may use.
    app_schemes: Vec<String>,
    /// Each page's title, so the router can set `document.title` when it
    /// shows the page; a SPA otherwise keeps the entry page's title forever.
    page_titles: HashMap<String, String>,
    /// A title that splices more than a parameter's name, compiled to a
    /// function of the route's parameters.
    page_title_fns: HashMap<String, String>,
    /// Each page's own `lang:`, which the router puts on `<html>` when it
    /// shows the page.
    page_langs: HashMap<String, String>,
    /// The site's language (`config.page_lang()`): what `<html lang>` goes
    /// back to on a page with no `lang:` of its own.
    site_lang: String,
    /// Whether the element being emitted sits inside a `Thead`, where a
    /// `Tcell` is a column header (`<th scope="col">`), not a data cell.
    in_thead: bool,
    /// Studio mode: stamp `data-wf-node` ids on rendered elements. Off for
    /// export/release builds, which must contain no debug attributes.
    studio: bool,
    /// Deterministic node ids keyed by element span (empty unless in studio mode).
    node_ids: NodeMap,
    /// The page, component or app being emitted: whose element a span is.
    node_owner: String,
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
    /// Each user component's positional prop, when it declares one: a
    /// positional argument at a call binds to it. Absent a declared one,
    /// the first prop, as the static paint has always done.
    component_positional: HashMap<String, String>,
    /// Each user component's declared events, which a call passes handlers
    /// for as `on: { name: fn }`.
    component_events: HashMap<String, Vec<String>>,
    /// The classes the project's scripts declare: a call to one is a
    /// construction, `new C(…)`, since the language has no `new`.
    script_classes: std::collections::HashSet<String>,
    /// Every name the project's scripts declare at their top level —
    /// function, class or value: the script's own global, read as itself.
    script_names: std::collections::HashSet<String>,
    /// Each user component's declared prop names.
    component_props: HashMap<String, Vec<String>>,
    /// The names of what each component's scoped slots hand over, by
    /// component and slot: what a fill's own names stand for.
    component_slot_params: HashMap<String, HashMap<String, Vec<String>>>,
    /// Each page's layout component and the compiled props object it is
    /// called with, when the page names one.
    page_layouts: HashMap<String, (String, String)>,
    /// Each page's `guard:` expression as JavaScript and its `redirect:`,
    /// for pages that have one.
    page_guards: HashMap<String, (String, String)>,
    /// Every page's path, in declaration order, for a router that lists no
    /// routes of its own.
    page_paths: Vec<(String, String)>,
    /// What the page, component, store or app being emitted declares:
    /// replaced whole at the start of each body, so nothing of one body
    /// is read while emitting the next.
    body: BodyState,
}

/// The per-body state of [`JsCodegen`]: everything that describes the page,
/// component, store or app being emitted. Each body starts from
/// `BodyState::default()` and fills in what it declares; a field set by
/// one body and not reset by the next used to leak — the last page's
/// validated states made the `app` read a `_x_check` only that page
/// declared, and the app threw at boot.
#[derive(Debug, Default)]
struct BodyState {
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
    /// The handle an unbound `Form` is given when the body has `validate`
    /// rules — so its submit asks them, as a bound form's does.
    implicit_form: Option<String>,
    /// Every `Form(bind: name)` handle of the page or component being
    /// written: `form.valid` reads signals, so what reads it is followed.
    form_handles: Vec<String>,
    /// The states a `validate` block guards, so the control bound to one
    /// shows its message without being told to.
    validated: Vec<String>,
    /// The body being emitted, for reading a state's declared condition.
    current_body: Vec<Statement>,
    /// Every name the page, component or store being emitted declares, at
    /// any depth: a declared `query` is its own, not the browser's.
    own_names: Vec<String>,
    /// The signals the body declares at its top — its states, derived
    /// values, resources and connections — which a top-level `const` of the
    /// same name gives way to.
    body_signals: Vec<String>,
    /// The resources declared in the page or component being emitted: a
    /// reference to one is the resource object, not a signal read.
    resources: Vec<String>,
    /// The route parameters of the page being emitted, read as
    /// `params.<name>`.
    page_params: Vec<String>,
}

impl Default for JsCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// The program's stores, each after the stores it `use`s (directly or
/// not), and otherwise in the order they are declared. A cycle keeps its
/// declared order: each side then reads the other only once both exist.
fn stores_in_use_order(program: &Program) -> Vec<&StoreDecl> {
    let stores: Vec<&StoreDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Store(s) => Some(s),
            _ => None,
        })
        .collect();
    let uses = |s: &StoreDecl| -> Vec<String> {
        s.body
            .iter()
            .filter_map(|st| match &st.kind {
                StatementKind::Use(u) => Some(u.store_name.clone()),
                _ => None,
            })
            .collect()
    };
    let mut out: Vec<&StoreDecl> = Vec::new();
    let mut visiting: Vec<String> = Vec::new();
    fn visit<'a>(
        s: &'a StoreDecl,
        stores: &[&'a StoreDecl],
        uses: &dyn Fn(&StoreDecl) -> Vec<String>,
        out: &mut Vec<&'a StoreDecl>,
        visiting: &mut Vec<String>,
    ) {
        if out.iter().any(|o| o.name == s.name) || visiting.contains(&s.name) {
            return;
        }
        visiting.push(s.name.clone());
        for name in uses(s) {
            if let Some(dep) = stores.iter().find(|d| d.name == name) {
                visit(dep, stores, uses, out, visiting);
            }
        }
        visiting.pop();
        out.push(s);
    }
    for s in &stores {
        visit(s, &stores, &uses, &mut out, &mut visiting);
    }
    out
}

impl JsCodegen {
    pub fn new() -> Self {
        Self {
            map_files: None,
            map_file: None,
            pending_mark: None,
            dev: false,
            shapes: Default::default(),
            offline: None,
            later: Vec::new(),
            consts: Vec::new(),
            static_scope: None,
            env: Default::default(),
            output: String::new(),
            full_runtime: false,
            runtime_modules: Vec::new(),
            runtime_report: Vec::new(),
            runtime_bytes: 0,
            indent: 0,
            components: Vec::new(),
            stores: Vec::new(),
            menu_lists: Vec::new(),
            store_connections: Vec::new(),
            own_connections: Vec::new(),
            store_resources: Vec::new(),
            own_resources: Vec::new(),
            window_opts: None,
            loop_bindings: Vec::new(),
            block_lets: Vec::new(),
            live_at: Vec::new(),
            loop_sources: Vec::new(),
            lambda_params: std::cell::RefCell::new(Vec::new()),
            store_locals: std::cell::RefCell::new(Vec::new()),
            i18n_default_locale: None,
            i18n_translations: HashMap::new(),
            ssg_mode: false,
            base_path: String::new(),
            app_schemes: Vec::new(),
            page_titles: HashMap::new(),
            page_title_fns: HashMap::new(),
            page_langs: HashMap::new(),
            site_lang: String::new(),
            in_thead: false,
            studio: false,
            node_ids: NodeMap::default(),
            node_owner: String::new(),
            split_pages: false,
            chunks: Vec::new(),
            page_sheets: std::collections::BTreeSet::new(),
            next_var: std::cell::Cell::new(0),
            component_positional: HashMap::new(),
            component_events: HashMap::new(),
            script_classes: std::collections::HashSet::new(),
            script_names: std::collections::HashSet::new(),
            component_props: HashMap::new(),
            component_slot_params: HashMap::new(),
            page_layouts: HashMap::new(),
            page_guards: HashMap::new(),
            page_paths: Vec::new(),
            body: BodyState::default(),
        }
    }

    /// A generator for a bundle built in `options`' modes.
    #[must_use]
    pub fn with_options(mut self, options: JsOptions) -> Self {
        let JsOptions {
            ssg,
            dev,
            split_pages,
            full_runtime,
            offline,
        } = options;
        self.ssg_mode = ssg;
        self.dev = dev;
        self.split_pages = split_pages;
        self.full_runtime = full_runtime;
        self.offline = offline;
        self
    }

    /// Write each page as its own chunk, loaded when its route shows
    /// ([`JsOptions::split_pages`]).
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

    /// Take pre-rendered pages over in place ([`JsOptions::ssg`]).
    pub fn set_ssg(&mut self, enabled: bool) {
        self.ssg_mode = enabled;
    }

    /// A build for the dev server: responses are held to their declared
    /// types at the network boundary ([`JsOptions::dev`]).
    pub fn set_dev(&mut self, enabled: bool) {
        self.dev = enabled;
    }

    /// The shape the runtime holds a response to (`shapeProblem` in
    /// `http.js`), for a declared type: `"s"`, `"n"`, `"b"`, `"m"`, `"*"`,
    /// `["?", s]`, `["l", s]`, `["r", "Name", { field: s }]`, `["e", [cases]]`.
    fn shape_of(&self, ty: &TypeRef, depth: usize) -> String {
        if depth > 6 {
            return "\"*\"".to_string();
        }
        match ty {
            TypeRef::String => "\"s\"".into(),
            TypeRef::Number => "\"n\"".into(),
            TypeRef::Bool => "\"b\"".into(),
            TypeRef::Map => "\"m\"".into(),
            TypeRef::Any => "\"*\"".into(),
            TypeRef::List(t) => format!("[\"l\", {}]", self.shape_of(t, depth + 1)),
            TypeRef::Optional(t) => format!("[\"?\", {}]", self.shape_of(t, depth + 1)),
            TypeRef::Refined(t, _) => self.shape_of(t, depth),
            TypeRef::Named(name) => {
                if let Some(cases) = self.shapes.1.get(name) {
                    let cases: Vec<String> = cases.iter().map(|c| format!("\"{c}\"")).collect();
                    return format!("[\"e\", [{}]]", cases.join(", "));
                }
                if let Some(decl) = self.shapes.0.get(name) {
                    let mut fields: Vec<String> = Vec::new();
                    let mut chain = vec![decl];
                    while let Some(parent) = chain
                        .last()
                        .and_then(|d| d.extends.as_ref())
                        .and_then(|p| self.shapes.0.get(p))
                    {
                        if chain.len() > 8 {
                            break;
                        }
                        chain.push(parent);
                    }
                    let mut seen: Vec<&str> = Vec::new();
                    for d in chain {
                        for f in &d.fields {
                            if seen.contains(&f.name.as_str()) {
                                continue;
                            }
                            seen.push(&f.name);
                            // A field with a default may be left out.
                            let shape = self.shape_of(&f.ty, depth + 1);
                            let shape = if f.default.is_some() {
                                format!("[\"?\", {shape}]")
                            } else {
                                shape
                            };
                            fields.push(format!(
                                "{}: {shape}",
                                serde_json::to_string(&f.name).unwrap_or_default()
                            ));
                        }
                    }
                    return format!("[\"r\", \"{name}\", {{ {} }}]", fields.join(", "));
                }
                // The types the language brings: each is a plain JSON value.
                match name.as_str() {
                    "Date" | "Time" | "DateTime" | "Url" | "Email" | "Uuid" | "Color"
                    | "Secret" => "\"s\"".into(),
                    "Duration" => "\"n\"".into(),
                    "Money" => {
                        "[\"r\", \"Money\", { \"amount\": \"n\", \"currency\": \"s\" }]".into()
                    }
                    _ => "\"*\"".into(),
                }
            }
        }
    }

    pub fn set_base_path(&mut self, path: String) {
        self.base_path = path;
    }

    /// The site's language, `config.page_lang()`: the `<html lang>` a page
    /// with no `lang:` of its own is shown in.
    pub fn set_site_lang(&mut self, lang: String) {
        self.site_lang = lang;
    }

    /// The router's `lang` option when any page has a language of its own:
    /// the site's, which a page without one goes back to.
    fn router_lang_option(&self) -> Option<String> {
        if self.page_langs.is_empty() {
            return None;
        }
        let lang = if self.site_lang.is_empty() {
            "en"
        } else {
            self.site_lang.as_str()
        };
        Some(format!(
            "lang: {}, dir: \"{}\"",
            serde_json::to_string(lang).unwrap_or_default(),
            if crate::project::i18n::table::is_rtl(lang) {
                "rtl"
            } else {
                "ltr"
            }
        ))
    }

    /// The app's own schemes a frame's address may use
    /// ([`crate::project::config::project::app_schemes`]).
    pub fn set_app_schemes(&mut self, schemes: Vec<String>) {
        self.app_schemes = schemes;
    }

    /// Register the service worker `offline` in the config asks for, and
    /// whether writes made offline are kept (`sync`) ([`JsOptions::offline`]).
    pub fn set_offline(&mut self, sync: bool) {
        self.offline = Some(sync);
    }

    /// The scripts `meta.scripts` loads `"after"` the page, each with its
    /// `meta.integrity` hash when it has one.
    pub fn set_later_scripts(&mut self, scripts: Vec<(String, Option<String>)>) {
        self.later = scripts;
    }

    /// The values `env.NAME` reads, from the project's config.
    pub fn set_env(&mut self, env: std::collections::BTreeMap<String, serde_json::Value>) {
        self.env = env;
    }

    /// Ship the whole runtime, for a site whose own scripts reach for `WF`
    /// in ways a build cannot see ([`JsOptions::full_runtime`]).
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

    /// Write source map markers (`build.sourcemap`): `files` names each
    /// declaration's source, by its `map_key`, as an index into the map's
    /// sources.
    pub fn set_sourcemap(&mut self, files: HashMap<String, usize>) {
        self.map_files = Some(files);
    }

    /// The key a declaration's source is found by: its kind and its name.
    pub fn map_key(decl: &Declaration) -> Option<String> {
        Some(match decl {
            Declaration::Component(c) => format!("component:{}", c.name),
            Declaration::Page(p) => format!("page:{}", p.name),
            Declaration::Store(s) => format!("store:{}", s.name),
            Declaration::Api(a) => format!("api:{}", a.name),
            Declaration::App(_) => "app".to_string(),
            _ => return None,
        })
    }

    /// The declaration `key` is being emitted: its statements are marked.
    fn map_start(&mut self, key: String) {
        if let Some(files) = &self.map_files {
            self.map_file = files.get(&key).copied();
        }
    }

    /// Its code is done: what follows is the compiler's own.
    fn map_end(&mut self) {
        if self.map_files.is_some() {
            self.map_file = None;
            self.pending_mark = Some(crate::web::codegen::sourcemap::end_marker());
        }
    }

    /// The statement at `span` begins: the next line written is marked.
    fn map_at(&mut self, span: Span) {
        if let Some(file) = self.map_file
            && span.line > 0
        {
            self.pending_mark = Some(crate::web::codegen::sourcemap::marker(
                file, span.line, span.col,
            ));
        }
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
            format!("WF.__reg(\"{name}\", WF.signal({value}))")
        } else {
            format!("WF.signal({value})")
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
            // `key: id` tells one instance's value from another's, and
            // is followed as state: `key: Project.id` keeps one per project.
            let policy = match &s.policy {
                Some(_) => format!(", {}", self.persist_policy(s, None)),
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
            format!("WF.__reg(\"{name}\", WF.computed(() => {body}))")
        } else {
            format!("WF.computed(() => {body})")
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
            .id_in(&self.node_owner, ui.span)
            .map(|id| format!("\"data-wf-node\": \"{id}\""))
    }

    /// Leading-comma form (`, "data-wf-node": "<id>"`) for splicing into an
    /// existing JS attrs object literal built inline by a special emitter; empty
    /// when not in studio mode / the node has no id.
    fn wf_node_inline(&self, ui: &UIElement) -> String {
        self.wf_node_entry(ui)
            .map(|e| format!(", {e}"))
            .unwrap_or_default()
    }

    /// One entry of the router's table: the path, the page's title (so the
    /// router can set `document.title`), and how to render it.
    fn route_entry(&self, path: &str, page: &str) -> String {
        let title = match (self.page_title_fns.get(page), self.page_titles.get(page)) {
            (Some(f), _) => format!("title: (params) => {f}, "),
            (None, Some(t)) => format!("title: {}, ", js_str(t)),
            (None, None) => String::new(),
        };
        // A page in a language of its own: the router puts it on `<html>`.
        let title = match self.page_langs.get(page) {
            Some(lang) => format!(
                "{title}lang: {}, dir: \"{}\", ",
                serde_json::to_string(lang).unwrap_or_default(),
                if crate::project::i18n::table::is_rtl(lang) {
                    "rtl"
                } else {
                    "ltr"
                }
            ),
            None => title,
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
                "{layout}guard: () => ({guard}), redirect: {}, ",
                js_str(redirect)
            ),
            None => layout,
        };
        if self.split_pages {
            let css = if self.page_sheets.contains(page) {
                format!("css: \"{page}\", ")
            } else {
                String::new()
            };
            format!(
                "{{ path: {}, {}{}{}page: {} }},",
                js_str(path),
                title,
                css,
                layout,
                js_str(page)
            )
        } else {
            format!(
                "{{ path: {}, {}{}render: (params) => Page_{}(params) }},",
                js_str(path),
                title,
                layout,
                page
            )
        }
    }

    /// A `Chart`'s or a `QrCode`'s SVG, when every argument is known at build
    /// time (a literal, a `const`, a `data` file).
    fn static_graphic(&self, name: &str, ui: &UIElement) -> Option<String> {
        let scope = self.static_scope.as_ref()?;
        let mut named = Vec::new();
        let mut positional = None;
        for arg in &ui.args {
            match arg {
                Arg::Named(k, e) => {
                    let v = crate::project::static_eval::eval(e, scope)?;
                    named.push((k.clone(), v.to_json()));
                }
                Arg::Positional(e) => {
                    positional = Some(crate::project::static_eval::eval(e, scope)?.to_json());
                }
            }
        }
        crate::web::codegen::charts::graphic(name, positional.as_ref(), &named, &ui.modifiers)
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.static_scope = Some(crate::project::static_eval::Scope::from_program_with_env(
            program,
            &[],
            &self.env,
        ));
        if self.split_pages {
            self.page_sheets = crate::project::scoped_css::split_rules(program)
                .pages
                .into_keys()
                .collect();
        }

        // The records and enums a dev build's response shapes name.
        if self.dev {
            for decl in &program.declarations {
                match decl {
                    Declaration::Type(t) => {
                        self.shapes.0.insert(t.name.clone(), t.clone());
                    }
                    Declaration::Enum(e) => {
                        self.shapes.1.insert(
                            e.name.clone(),
                            e.cases.iter().map(|c| c.name.clone()).collect(),
                        );
                    }
                    _ => {}
                }
            }
        }

        // First pass: collect component and store names
        for decl in &program.declarations {
            match decl {
                Declaration::Script(script) => {
                    for n in &script.names {
                        if matches!(n.kind, crate::syntax::js_scan::NameKind::Class { .. }) {
                            self.script_classes.insert(n.name.clone());
                        }
                        self.script_names.insert(n.name.clone());
                    }
                }
                Declaration::Component(c) => {
                    self.components.push(c.name.clone());
                    self.component_props.insert(
                        c.name.clone(),
                        c.props.iter().map(|p| p.name.clone()).collect(),
                    );
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
                Declaration::Store(s) => {
                    self.stores.push(s.name.clone());
                    for stmt in &s.body {
                        if let StatementKind::Connection(c) = &stmt.kind {
                            self.store_connections
                                .push((s.name.clone(), c.name.clone()));
                        }
                        if let StatementKind::Resource(r) = &stmt.kind {
                            self.store_resources.push((s.name.clone(), r.name.clone()));
                        }
                    }
                }
                // A service is a name in scope like a store's, not a
                // signal, so it is read as it is written.
                Declaration::Api(a) => self.stores.push(a.name.clone()),
                Declaration::Const(c) => self.consts.push(c.name.clone()),
                Declaration::Page(p) => {
                    if let Some(title) = &p.title {
                        self.page_titles.insert(p.name.clone(), title.clone());
                    }
                    if let Some(lang) = &p.lang {
                        self.page_langs.insert(p.name.clone(), lang.clone());
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
                // The layout is called with the route's parameters, so its
                // arguments read them: `layout: Shell(crumb: "Build {hash}")`.
                // Without them in scope a parameter named after a browser
                // value — `hash` — compiled to `location.hash`.
                self.body.page_params = p.params.iter().map(|p| p.name.clone()).collect();
                let args = self.emit_component_args(&layout.name, &layout.args);
                self.body.page_params.clear();
                self.page_layouts
                    .insert(p.name.clone(), (layout.name.clone(), args));
            }
            // `title: "{post.title} — Blog"`: worked out from the route's
            // parameters on each visit, as the build worked it out per file.
            if let Declaration::Page(p) = decl
                && let Some(expr) = &p.title_expr
            {
                self.body.page_params = p.params.iter().map(|p| p.name.clone()).collect();
                let js = self.emit_expr(expr);
                self.body.page_params.clear();
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
            self.emit_line(&format!("WF.setBasePath({});", js_str(&self.base_path)));
        }
        if !self.app_schemes.is_empty() {
            self.emit_line(&format!(
                "WF.allowSchemes({});",
                serde_json::to_string(&self.app_schemes).unwrap_or_default()
            ));
        }
        // The service worker, when the config names `offline`.
        if let Some(sync) = self.offline {
            self.emit_line(&format!("WF.offline({{ sync: {sync} }});"));
        }
        if self.ssg_mode {
            self.emit_line("WF.setSsgMode(true);");
        }
        // Scripts nothing on the page waits for, asked for once it has
        // painted.
        if !self.later.is_empty() {
            let list: Vec<String> = self
                .later
                .iter()
                .map(|(src, integrity)| {
                    let src = serde_json::to_string(src).unwrap_or_default();
                    match integrity {
                        Some(hash) => format!(
                            "{{ src: {src}, integrity: {} }}",
                            serde_json::to_string(hash).unwrap_or_default()
                        ),
                        None => format!("{{ src: {src} }}"),
                    }
                })
                .collect();
            self.emit_line(&format!("WF.loadAfter([{}]);", list.join(", ")));
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
                self.emit_line(&format!("const {} = {};", top_level(&c.name), value));
            }
        }

        // Emit services, which a store's actions may call.
        for decl in &program.declarations {
            if let Declaration::Api(a) = decl {
                self.map_start(format!("api:{}", a.name));
                self.emit_api(a);
                self.map_end();
            }
        }

        // Emit stores, each after the stores it `use`s: a store is built on
        // its first read, and a read while the script still runs (a page's
        // first paint, another store's persisted key) must not meet a store
        // still in its temporal dead zone. A derived value that first ran
        // there kept what it got, and followed nothing.
        for s in stores_in_use_order(program) {
            self.map_start(format!("store:{}", s.name));
            self.emit_store(s);
            self.map_end();
        }

        // Emit components
        for decl in &program.declarations {
            if let Declaration::Component(c) = decl {
                self.map_start(format!("component:{}", c.name));
                self.emit_component(c);
                self.map_end();
            }
        }

        // Emit pages: into the bundle, or each into its own chunk that
        // registers itself with the runtime when it runs.
        for decl in &program.declarations {
            if let Declaration::Page(p) = decl {
                self.map_start(format!("page:{}", p.name));
                if self.split_pages {
                    let saved = std::mem::take(&mut self.output);
                    let indent = std::mem::replace(&mut self.indent, 0);
                    self.emit_page(p);
                    self.map_end();
                    self.emit_line(&format!("WF.page(\"{}\", Page_{});", p.name, p.name));
                    let chunk = std::mem::replace(&mut self.output, saved);
                    self.indent = indent;
                    self.chunks.push((p.name.clone(), chunk));
                } else {
                    self.emit_page(p);
                    self.map_end();
                }
            }
        }

        // Emit app (router setup)
        for decl in &program.declarations {
            if let Declaration::App(a) = decl {
                self.map_start("app".to_string());
                self.emit_app(a);
                self.map_end();
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
                if self.ssg_mode {
                    self.emit_line(
                        "const container = WF.mainOf(WF.hydrating(document.getElementById('app')));",
                    );
                } else {
                    self.emit_line("const container = WF.mainOf(document.getElementById('app'));");
                }
                match self.router_lang_option() {
                    Some(lang) => {
                        self.emit_line(&format!("WF.router(routes, container, {{ {lang} }});"));
                    }
                    None => self.emit_line("WF.router(routes, container);"),
                }
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

    // ─── i18n ────────────────────────────────────────

    // ─── Component ───────────────────────────────────

    // ─── Page ────────────────────────────────────────

    // ─── App ─────────────────────────────────────────

    // ─── DOM-building statement emitter ──────────────

    // ─── UI Element ──────────────────────────────────

    // ─── Special component emitters ──────────────────

    // ─── Control flow (DOM) ──────────────────────────

    // ─── Statement (imperative, non-DOM) ─────────────

    // ─── Expression emitter ──────────────────────────

    // ─── Helpers ─────────────────────────────────────

    /// `WF.live(…)` for the elements the function begun at `start` hands to
    /// a closure of its own, so a pre-rendered page taken over in place
    /// keeps those as drawn (see `capture`). Only a static build takes a
    /// page over.
    fn emit_held_elements(&mut self, start: usize) {
        if !self.ssg_mode {
            return;
        }
        let held = crate::web::codegen::capture::captured_elements(&self.output[start..]);
        if !held.is_empty() {
            self.emit_line(&format!("WF.live({});", held.join(", ")));
        }
    }

    fn emit_line(&mut self, text: &str) {
        let indent = "  ".repeat(self.indent);
        let mark = self.pending_mark.take().unwrap_or_default();
        self.output.push_str(&format!("{indent}{mark}{text}\n"));
    }

    fn fresh_var(&self) -> String {
        let n = self.next_var.get();
        self.next_var.set(n + 1);
        format!("_e{n}")
    }

    fn emit_component_args(&self, component: &str, args: &[Arg]) -> String {
        let mut parts = Vec::new();
        for arg in args {
            match arg {
                // A motion marker lands on the component's root, not in its props.
                Arg::Named(name, _) if name.starts_with("data-wf-") => {}
                Arg::Named(name, expr) => {
                    let key = if name.contains('-') {
                        format!("\"{name}\"")
                    } else {
                        name.clone()
                    };
                    let value = self.emit_expr(expr);
                    // A value that reads state is handed over as a getter, so
                    // the component tracks it instead of copying it once.
                    if self.is_reactive(&value) {
                        parts.push(format!("get {key}() {{ return {value}; }}"));
                    } else {
                        parts.push(format!("{key}: {value}"));
                    }
                }
                // A positional argument binds to the component's positional
                // prop. It used to be written into the object bare, which is
                // not a property.
                Arg::Positional(expr) => {
                    if let Some(prop) = self.component_positional.get(component) {
                        let value = self.emit_expr(expr);
                        if self.is_reactive(&value) {
                            parts.push(format!("get {prop}() {{ return {value}; }}"));
                        } else {
                            parts.push(format!("{prop}: {value}"));
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

/// Whether `ui` directly contains a `Parent.Sub` element.
fn has_subcomponent(ui: &UIElement, parent: &str, sub: &str) -> bool {
    ui.children.iter().any(|stmt| {
        matches!(&stmt.kind, StatementKind::UIElement(child)
            if matches!(&child.component, ComponentRef::SubComponent(p, s)
                if p == parent && s == sub))
    })
}

// ─── Utility functions ──────────────────────────────────

impl BindTarget {
    /// `$V` alone, not the start of a live binding's accessor (`$Vendor()`).
    fn write(&self, value: &str) -> String {
        let mut out = String::with_capacity(self.write.len() + value.len());
        let mut rest = self.write.as_str();
        while let Some(at) = rest.find("$V") {
            let after = &rest[at + 2..];
            out.push_str(&rest[..at]);
            if after
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
            {
                out.push_str("$V");
            } else {
                out.push_str(value);
            }
            rest = after;
        }
        out.push_str(rest);
        out
    }
}

impl JsCodegen {}

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
        .find(|m| crate::project::themes::prune::ANIMATIONS.contains(&m.as_str()))
        .cloned()
}

/// The keys `WF.el` gives to a runtime module rather than writing them as
/// attributes — every `attrHook` the runtime registers. Each puts markup in
/// the element: `html` (what `Unsafe.Html` draws), `markdown`, `highlight`.
pub const RUNTIME_KEYS: &[&str] = &["html", "markdown", "highlight"];

/// The key an author's named argument goes to `WF.el` under, to be written
/// as an attribute: quoted when it is hyphenated (`"aria-label"`), and
/// spelled `"attr:html"` when it is one of the runtime's own keys, so that
/// `Stack(html: text)` is an `html` attribute, as `C04` says, and never
/// markup — the one way markup goes in is `Unsafe.Html`.
pub(crate) fn attr_key(key: &str) -> String {
    if RUNTIME_KEYS.contains(&key) {
        format!("\"attr:{key}\"")
    } else if key.contains('-') {
        format!("\"{key}\"")
    } else {
        key.to_string()
    }
}

/// The call that keeps a link's `active` class and `aria-current` with the
/// route. A link whose address has a query is the page's only when the
/// query matches too, so it is handed `WF.query` — which brings that module
/// only into a program that has one.
fn active_link(var: &str, href: &str, prefix: impl std::fmt::Display) -> String {
    if href.contains('?') {
        format!("WF.activeLink({var}, {href}, {prefix}, WF.query);")
    } else {
        format!("WF.activeLink({var}, {href}, {prefix});")
    }
}
