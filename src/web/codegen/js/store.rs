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
    pub(super) fn emit_api(&mut self, api: &ApiDecl) {
        self.emit_line(&format!("const {} = WF.api({{", top_level(&api.name)));
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
        self.emit_line("endpoints: {");
        self.indent += 1;
        for endpoint in &api.endpoints {
            let mut parts = vec![
                format!("method: {}", js_str(&endpoint.method)),
                format!("path: {}", js_str(&endpoint.path)),
            ];
            for (key, value) in &endpoint.settings {
                parts.push(format!("{}: {}", api_setting(key), self.emit_expr(value)));
            }
            if self.dev
                && let Some(ty) = &endpoint.returns
            {
                parts.push(format!("shape: {}", self.shape_of(ty, 0)));
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
        let Some(policy) = &state.policy else {
            return "{}".to_string();
        };
        let mut parts = Vec::new();
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
