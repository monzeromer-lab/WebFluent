//! Stores and API declarations: their state, actions, persistence, and the expressions and statements inside them.

use super::{
    ActionDecl, ApiDecl, BodyState, ConnectionDecl, Cx, DerivedDecl, Expr, JsCodegen, ResourceDecl,
    StateDecl, Statement, StatementKind, StmtCx, StoreDecl, TypeRef, api_setting, bare,
    body_signals, declared_names, fetch_word, js_str, top_level,
};

impl JsCodegen {
    /// `api Backend(base: "/api") { … }` — the service as one object of
    /// callable endpoints, each of which also carries `.invalidate()`,
    /// `.prefetch()` and `.progress`.
    /// `middleware name(req, next) { … }`: a function of the request and
    /// the rest of the chain, returning what the chain returns.
    pub(super) fn emit_middleware(&mut self, m: &crate::syntax::parser::ast::MiddlewareDecl) {
        let head = self.handler_head(&m.handler, "req");
        let head = if head.starts_with("async") {
            head
        } else {
            format!("async {head}")
        };
        self.emit_line(&format!("const {} = {head} => {{", top_level(&m.name)));
        self.indent += 1;
        let body = self.emit_event_body(&m.handler);
        if !body.is_empty() {
            self.emit_line(&body);
        }
        self.indent -= 1;
        self.emit_line("};");
    }

    /// The fakes, as what the runtime asks before it sends anything:
    /// `window.__wfFakes["Backend.users.list"]`, each handed the call's
    /// arguments by name and giving the result. The table is there even
    /// with no fake in it, so a call no fake answers is reported, not sent.
    pub(super) fn emit_fakes(&mut self, program: &crate::syntax::parser::ast::Program) {
        use crate::syntax::parser::ast::Declaration;
        self.emit_line("window.__wfFakes = {};");
        for decl in &program.declarations {
            let Declaration::Fake(f) = decl else {
                continue;
            };
            let api = program.declarations.iter().find_map(|d| match d {
                Declaration::Api(a) if a.name == f.target => Some(a),
                _ => None,
            });
            for answer in &f.answers {
                let endpoint =
                    api.and_then(|a| a.endpoints.iter().find(|e| e.name == answer.endpoint));
                let mut lets = Vec::new();
                for name in &answer.params {
                    let key = serde_json::to_string(name).unwrap_or_default();
                    // A parameter the call left out is its default, as the
                    // server would take it.
                    let default = endpoint
                        .and_then(|e| e.params.iter().find(|p| p.name == *name))
                        .and_then(|p| p.default.as_ref())
                        .map(|d| self.emit_expr(d));
                    lets.push(match default {
                        Some(d) => format!(
                            "const {n} = $args[{key}] === undefined ? {d} : $args[{key}];",
                            n = super::names::bare(name)
                        ),
                        None => format!(
                            "const {n} = $args[{key}] === undefined ? null : $args[{key}];",
                            n = super::names::bare(name)
                        ),
                    });
                }
                let body = self.with_locals(super::expr::Cx::Page, answer.params.clone(), || {
                    self.emit_expr(&answer.body)
                });
                self.emit_line(&format!(
                    "window.__wfFakes[{}] = ($args) => {{ {} return {body}; }};",
                    serde_json::to_string(&format!("{}.{}", f.target, answer.endpoint))
                        .unwrap_or_default(),
                    lets.join(" ")
                ));
            }
        }
    }

    /// One step of a service's `middleware [ … ]`: a middleware the
    /// project declares, or one of the runtime's, its arguments read when a
    /// request is made — a token refreshed a moment ago is the one sent.
    fn middleware_step(&self, step: &Expr) -> String {
        match step {
            Expr::Identifier(n) if matches!(n.as_str(), "requestId" | "log") => {
                format!("WF.mw.{n}")
            }
            Expr::Identifier(n) => top_level(n),
            Expr::FunctionCall(name, args) if name == "bearer" || name == "header" => {
                let read: Vec<String> = args
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let e = self.emit_expr(a);
                        if name == "header" && i == 0 {
                            e
                        } else {
                            format!("() => {e}")
                        }
                    })
                    .collect();
                format!("WF.mw.{name}({})", read.join(", "))
            }
            Expr::FunctionCall(name, args) if name == "refreshOn401" => {
                let action = args
                    .first()
                    .map_or_else(|| "null".to_string(), |a| self.emit_expr(a));
                format!("WF.mw.refreshOn401(() => ({action})())")
            }
            other => self.emit_expr(other),
        }
    }

    pub(super) fn emit_api(&mut self, api: &ApiDecl) {
        // `api Backend.users { … }`: a group, reached through its service.
        if let Some((service, group)) = api.name.split_once('.') {
            self.emit_line(&format!(
                "WF.group({}, {}, {{",
                top_level(service),
                js_str(group)
            ));
            self.indent += 1;
            self.emit_endpoints(api);
            self.indent -= 1;
            self.emit_line("});");
            self.emit_line("");
            return;
        }
        self.emit_line(&format!("const {} = WF.api({{", top_level(&api.name)));
        self.indent += 1;
        self.emit_line(&format!("name: \"{}\",", api.name));
        for (key, value) in &api.settings {
            if key == "middleware" {
                let steps: Vec<String> = match value {
                    Expr::ListLiteral(items) => {
                        items.iter().map(|s| self.middleware_step(s)).collect()
                    }
                    other => vec![self.emit_expr(other)],
                };
                self.emit_line(&format!("middleware: [{}],", steps.join(", ")));
                continue;
            }
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
            let param = hook.param().unwrap_or("r").to_string();
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
        self.emit_endpoints(api);
        self.indent -= 1;
        self.emit_line("});");
        self.emit_line("");
    }

    /// `endpoints: { … }`: each endpoint of a service or a group — its
    /// method and path, what it answers and how it fails, where its
    /// parameters go, and what it makes stale.
    fn emit_endpoints(&mut self, api: &ApiDecl) {
        self.emit_line("endpoints: {");
        self.indent += 1;
        for endpoint in &api.endpoints {
            let mut parts = vec![
                format!("method: {}", js_str(&endpoint.method)),
                format!("path: {}", js_str(&endpoint.path)),
            ];
            for (key, value) in &endpoint.settings {
                // Written below, as what a success makes stale.
                if key == "invalidates" {
                    continue;
                }
                // `pages .cursor(param: after, next: p => p.next)`: how a
                // read of it is gathered — the parameter the next page goes
                // in, where the next page starts, and where a page's items
                // are. A page's `Link` header is read with its answer.
                if key == "pages" {
                    if let Some(paging) = endpoint.pages() {
                        let mut how = vec![format!("style: {}", js_str(paging.style))];
                        if let Some(param) = paging.param() {
                            how.push(format!("param: {}", js_str(param)));
                        }
                        for name in ["next", "items"] {
                            if let Some(f) = paging.arg(name) {
                                how.push(format!("{name}: {}", self.emit_expr(f)));
                            }
                        }
                        parts.push(format!("pages: {{ {} }}", how.join(", ")));
                        if paging.style == "link" {
                            parts.push("withHeaders: true".to_string());
                        }
                    }
                    continue;
                }
                parts.push(format!("{}: {}", api_setting(key), self.emit_expr(value)));
            }
            // `-> User ! CreateUserError`: the call answers with a result,
            // each status the error enum names going to its case.
            let mut error_shapes: Vec<String> = Vec::new();
            let answer = match &endpoint.returns {
                Some(TypeRef::Result(value, error)) => {
                    let mut result = Vec::new();
                    if **value == TypeRef::Unit {
                        result.push("unit: true".to_string());
                    }
                    if let Some(error) = error
                        && let TypeRef::Named(name) = &**error
                        && name != "HttpError"
                    {
                        let cases = self.error_enums.get(name).cloned().unwrap_or_default();
                        let mapped: Vec<String> = cases
                            .iter()
                            .map(|(status, case, _)| format!("{status}: \"{case}\""))
                            .collect();
                        for (status, _, body) in &cases {
                            error_shapes.push(format!("{status}: {}", self.shape_of(body, 0)));
                        }
                        result.push("enum: true".to_string());
                        result.push(format!("cases: {{ {} }}", mapped.join(", ")));
                    }
                    parts.push(if result.is_empty() {
                        "result: {}".to_string()
                    } else {
                        format!("result: {{ {} }}", result.join(", "))
                    });
                    (**value != TypeRef::Unit).then(|| (**value).clone())
                }
                other => other.clone(),
            };
            // What it answers, held to its type in every build.
            if let Some(ty) = &answer {
                let shape = self.shape_of(ty, 0);
                if shape != "\"j\"" && shape != "\"*\"" {
                    parts.push(format!("shape: {shape}"));
                }
            }
            if !error_shapes.is_empty() {
                parts.push(format!("errorShapes: {{ {} }}", error_shapes.join(", ")));
            }
            // `errors { 422 -> T }`: under `wf serve`, the body of a failure
            // is held to what it declares, as an answer is.
            if self.dev && !endpoint.errors.is_empty() {
                let shapes: Vec<String> = endpoint
                    .errors
                    .iter()
                    .map(|(status, ty)| format!("{status}: {}", self.shape_of(ty, 0)))
                    .collect();
                parts.push(format!("errorShapes: {{ {} }}", shapes.join(", ")));
            }
            // A file parameter is sent as a form, under its own name.
            if let Some(file) = endpoint
                .params
                .iter()
                .find(|p| matches!(&p.prop_type, TypeRef::Named(n) if n == "File"))
            {
                parts.push(format!("fileField: \"{}\"", file.name));
            }
            // Where a parameter goes that is not in the path or the query.
            if !endpoint.locations.is_empty() {
                let placed: Vec<String> = endpoint
                    .locations
                    .iter()
                    .map(|(p, l)| match l {
                        crate::syntax::parser::ast::ParamLocation::Header(h) => {
                            format!("{}: [\"header\", {}]", js_str(p), js_str(h))
                        }
                        crate::syntax::parser::ast::ParamLocation::Cookie => {
                            format!("{}: [\"cookie\"]", js_str(p))
                        }
                    })
                    .collect();
                parts.push(format!("locations: {{ {} }}", placed.join(", ")));
            }
            // `invalidates [list, byId(id)]`: each read, and what it is
            // asked with — the write's own arguments, the read's in order.
            if let Some((_, Expr::ListLiteral(items))) =
                endpoint.settings.iter().find(|(k, _)| k == "invalidates")
            {
                let stale: Vec<String> = items
                    .iter()
                    .filter_map(|item| {
                        let (name, args) = match item {
                            Expr::Identifier(n) => (n.as_str(), &[][..]),
                            Expr::FunctionCall(n, args) => (n.as_str(), args.as_slice()),
                            _ => return None,
                        };
                        let read = api.endpoints.iter().find(|e| e.name == name)?;
                        if args.is_empty() {
                            return Some(format!("[{}, null]", js_str(name)));
                        }
                        let given: Vec<String> = args
                            .iter()
                            .zip(&read.params)
                            .filter_map(|(a, p)| match a {
                                Expr::Identifier(own) => {
                                    Some(format!("{}: args.{own}", js_str(&p.name)))
                                }
                                _ => None,
                            })
                            .collect();
                        Some(format!("[{}, {{ {} }}]", js_str(name), given.join(", ")))
                    })
                    .collect();
                parts.push(format!("invalidates: (args) => [{}]", stale.join(", ")));
            }
            self.emit_line(&format!("{}: {{ {} }},", endpoint.name, parts.join(", ")));
        }
        self.indent -= 1;
        self.emit_line("},");
    }

    pub(super) fn emit_store(&mut self, store: &StoreDecl) {
        self.body = BodyState::default();
        self.body.own_names = declared_names(&store.body);
        self.body.body_signals = body_signals(&store.body);
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
            top_level(&store.name),
            store.name
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
                StatementKind::Connection(c) => Some(c.name.clone()),
                StatementKind::Resource(r) => Some(r.name.clone()),
                _ => None,
            })
            .collect();
        self.own_connections = store
            .body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Connection(c) => Some(c.name.clone()),
                _ => None,
            })
            .collect();
        self.own_resources = store
            .body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Resource(r) => Some(r.name.clone()),
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
                    let policy = self.persist_policy(state, Some(&store_state_names));
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
                    if crate::syntax::parser::ast::awaits(&a.body) {
                        "async "
                    } else {
                        ""
                    },
                    if params.is_empty() {
                        String::new()
                    } else {
                        let named: Vec<String> = params.iter().map(|p| bare(p)).collect();
                        format!(", {}", named.join(", "))
                    }
                ));
                self.indent += 1;
                self.store_locals.borrow_mut().clear();
                // The parameters are the action's locals, read as declared.
                self.store_locals
                    .borrow_mut()
                    .extend(params.iter().cloned());
                // A parameter shadows a store member of the same name: an
                // action `move(step)` beside an action `step` used to read
                // `store.step` — the function — where its argument was meant.
                let visible: Vec<String> = store_state_names
                    .iter()
                    .filter(|n| !params.contains(n))
                    .cloned()
                    .collect();
                // What `try` passes up comes back as the action's `.err`.
                self.open_result_action(
                    a.returns.as_ref(),
                    crate::syntax::parser::ast::awaits(&a.body),
                );
                // What it showed before the server agreed is taken back if
                // anything in it throws, as a page's action does.
                let optimistic = super::shows_optimistically(&a.body);
                if optimistic {
                    self.emit_line(
                        self.attempt_opener(crate::syntax::parser::ast::awaits(&a.body)),
                    );
                    self.indent += 1;
                }
                for stmt in &a.body {
                    self.emit_store_statement(stmt, &visible, &params);
                }
                if optimistic {
                    self.indent -= 1;
                    self.emit_line("});");
                }
                self.close_result_action(None);
                self.store_locals.borrow_mut().clear();
                self.indent -= 1;
                self.emit_line("},");
            }
            self.indent -= 1;
            self.emit_line("},");
            // The actions every call of which runs, overlapping or not.
            let concurrent: Vec<String> = actions
                .iter()
                .filter(|a| a.concurrent)
                .map(|a| format!("\"{}\"", a.name))
                .collect();
            if !concurrent.is_empty() {
                self.emit_line(&format!("concurrent: [{}],", concurrent.join(", ")));
            }
            // The actions that keep their last outcome, `.result`.
            let results: Vec<String> = actions
                .iter()
                .filter(|a| Self::declares_result(a))
                .map(|a| format!("\"{}\"", a.name))
                .collect();
            if !results.is_empty() {
                self.emit_line(&format!("results: [{}],", results.join(", ")));
            }
        }

        // The connections the store holds open — a socket, a worker — made
        // when the store is, and closed with it.
        let connections: Vec<&ConnectionDecl> = store
            .body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Connection(c) => Some(c),
                _ => None,
            })
            .collect();
        if !connections.is_empty() {
            self.emit_line("connections: {");
            self.indent += 1;
            for c in connections {
                self.emit_store_connection(c, &store_state_names);
            }
            self.indent -= 1;
            self.emit_line("},");
        }

        // The requests the store holds — made when the store is, outside
        // any page, and abandoned with it.
        let resources: Vec<&ResourceDecl> = store
            .body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Resource(r) => Some(r),
                _ => None,
            })
            .collect();
        if !resources.is_empty() {
            self.emit_line("resources: {");
            self.indent += 1;
            for r in resources {
                let call = self.resource_call(r, Some(&store_state_names));
                self.emit_line(&format!("{}: (store) => {call},", r.name));
            }
            self.indent -= 1;
            self.emit_line("},");
        }
        self.own_connections.clear();
        self.own_resources.clear();

        self.indent -= 1;
        self.emit_line(&format!("}}), {options});"));
        self.emit_line("");
    }

    /// What a `persist` says about where its value lives and how an older
    /// one is brought forward.
    /// `store` is the store's states, when it is a store's: its `key:` is
    /// then a function of the store.
    pub(super) fn persist_policy(&mut self, state: &StateDecl, store: Option<&[String]>) -> String {
        let mut parts = Vec::new();
        // What a value read back must be: the declared type, or what a
        // plain first value says it is.
        let shape = match (&state.ty, &state.value) {
            (Some(ty), _) => self.shape_of(ty, 0),
            (None, Expr::NumberLiteral(_)) => "\"n\"".to_string(),
            (None, Expr::StringLiteral(_)) => "\"s\"".to_string(),
            (None, Expr::BoolLiteral(_)) => "\"b\"".to_string(),
            _ => "\"*\"".to_string(),
        };
        let shape = (shape != "\"*\"").then(|| format!("shape: {shape}"));
        let Some(policy) = &state.policy else {
            parts.extend(shape);
            return format!("{{ {} }}", parts.join(", "));
        };
        if let Some(key) = &policy.key {
            parts.push(match store {
                Some(states) => format!("key: (store) => {}", self.emit_store_expr(key, states)),
                None => format!("key: () => {}", self.emit_expr(key)),
            });
        }
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
        parts.extend(shape);
        format!("{{ {} }}", parts.join(", "))
    }

    /// Whether `name`, inside a store, is the store's own member: one of
    /// `store_states` that no local in reach — a `let`, a `for` binding, a
    /// lambda's parameter, a caught error — shadows. `let refs = …` in an
    /// action of a store with `state refs` is the action's own.
    pub(super) fn is_own_member(&self, name: &str, store_states: &[String]) -> bool {
        store_states.iter().any(|s| s == name)
            && !self.store_locals.borrow().iter().any(|l| l == name)
    }

    /// An expression inside a store: a name that is one of its own
    /// members is read through `store.`.
    pub(super) fn emit_store_expr(&self, expr: &Expr, store_states: &[String]) -> String {
        self.emit_expr_in(expr, Cx::Store(store_states))
    }

    /// A statement in a store's action: its own members are read and
    /// written through `store.`, and `action_params` are the action's
    /// parameters, which a nested statement assigns as plain names.
    pub(super) fn emit_store_statement(
        &mut self,
        stmt: &Statement,
        store_states: &[String],
        action_params: &[String],
    ) {
        self.emit_statement_in(
            stmt,
            StmtCx::Store {
                states: store_states,
                params: action_params,
            },
        );
    }
}
