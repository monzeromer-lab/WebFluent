//! Stores and API declarations: their state, actions, persistence, and the expressions and statements inside them.

use super::*;

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

    /// Emit expression inside a store context — identifiers that are store state
    /// are accessed via `store.property` instead of `_name()`.
    pub(super) fn emit_store_expr(&self, expr: &Expr, store_states: &[String]) -> String {
        match expr {
            Expr::Identifier(name) => {
                if self.is_own_member(name, store_states) {
                    format!("store.{}", name)
                } else if self.store_locals.borrow().contains(name) {
                    bare(name)
                } else if self.consts.contains(name) || self.stores.contains(name) {
                    top_level(name)
                } else if BROWSER_VALUES.contains(&name.as_str())
                    && !self.body.own_names.contains(name)
                {
                    format!("WF.{name}()")
                } else {
                    name.to_string()
                }
            }
            // A connection of this store's: what it holds is a signal.
            Expr::PropertyAccess(base, prop)
                if CONNECTION_SIGNALS.contains(&prop.as_str())
                    && matches!(base.as_ref(), Expr::Identifier(n)
                        if self.own_connections.contains(n) && self.is_own_member(n, store_states)) =>
            {
                format!("{}.{prop}()", self.emit_store_expr(base, store_states))
            }
            // A resource of this store's: what it holds is a signal.
            Expr::PropertyAccess(base, prop)
                if RESOURCE_SIGNALS.contains(&prop.as_str())
                    && matches!(base.as_ref(), Expr::Identifier(n)
                        if self.own_resources.contains(n) && self.is_own_member(n, store_states)) =>
            {
                format!("{}.{prop}()", self.emit_store_expr(base, store_states))
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
                    // Against `null`, a value that is absent — a query key
                    // the address lacks, a field a record leaves out — is
                    // `undefined` in the browser, and counts as null.
                    BinOp::Eq if null_side(left, right) => "==",
                    BinOp::Neq if null_side(left, right) => "!=",
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
                // The arm a `match` with no `else` cannot reach.
                if method == "__exhaustive" {
                    return "null".to_string();
                }
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
                    let bound = self.store_locals.borrow().len();
                    self.store_locals.borrow_mut().push(name.clone());
                    let then_val = self.emit_store_expr(then_expr, store_states);
                    self.store_locals.borrow_mut().truncate(bound);
                    let else_val = self.emit_store_expr(&args[1], store_states);
                    let name = bare(name);
                    // A branch that awaits needs its arrow to be async, and
                    // the arrow awaited, or the page's code doesn't parse.
                    if then_val.contains("await ") || else_val.contains("await ") {
                        return format!(
                            "(await (async ({name}) => {name} != null ? {then_val} : {else_val})({value}))"
                        );
                    }
                    return format!(
                        "(({name}) => {name} != null ? {then_val} : {else_val})({value})"
                    );
                }
                let obj_str = self.emit_store_expr(obj, store_states);
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_store_expr(a, store_states))
                    .collect();
                // Another store's own actions come first, as they do on a
                // page: `OpenFiles.preview(path)` calls the action the
                // store declares. Through the table it was the runtime's
                // `WF.preview(OpenFiles, path)`, and the action never ran
                // with its arguments.
                if let Expr::Identifier(s) = obj.as_ref()
                    && self.stores.contains(s)
                    && !store_states.contains(s)
                    && !self.store_locals.borrow().contains(s)
                {
                    return format!("{}.{}({})", obj_str, method, args_str.join(", "));
                }
                // The same table the page emitter reads. A bare name that
                // is one of the store's own states is held by the store, so
                // a mutating method assigns back through it.
                let holder = match obj.as_ref() {
                    Expr::Identifier(n) if self.is_own_member(n, store_states) => {
                        Holder::StoreMember
                    }
                    Expr::PropertyAccess(base, _) => match base.as_ref() {
                        Expr::Identifier(s) if self.stores.contains(s) => Holder::StoreMember,
                        _ => Holder::Plain,
                    },
                    _ => Holder::Plain,
                };
                method_to_js(method, &obj_str, &args_str, holder)
            }
            // A parameter shadows a store member of its name in the body.
            Expr::Lambda(param, body) => {
                let bound = self.store_locals.borrow().len();
                self.store_locals.borrow_mut().extend(
                    param
                        .split(',')
                        .map(str::trim)
                        .filter(|p| !p.is_empty())
                        .map(str::to_string),
                );
                let body_str = self.emit_store_expr(body, store_states);
                self.store_locals.borrow_mut().truncate(bound);
                format!("(({}) => {})", bare_list(param), body_str)
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
                if self.is_own_member(name, store_states) {
                    format!("store.{}({})", name, args_str.join(", "))
                } else if (name == "t" || name == "setLocale") && self.has_i18n() {
                    // A store's words, translated as a page's are.
                    format!("WF.i18n.{}({})", name, args_str.join(", "))
                } else if matches!(
                    name.as_str(),
                    // The same set a page reaches: a store's action is where
                    // `optimistic` belongs, and `uuid` where an id is made.
                    "format"
                        | "ago"
                        | "setTheme"
                        | "beacon"
                        | "optimistic"
                        | "sanitize"
                        | "uuid"
                        | "keyIs"
                        | "keyLabel"
                ) {
                    format!("WF.{}({})", name, args_str.join(", "))
                } else if name == "fetch" {
                    format!("WF.request({})", args_str.join(", "))
                } else if self.store_locals.borrow().contains(name) {
                    format!("{}({})", bare(name), args_str.join(", "))
                } else if self.consts.contains(name) {
                    format!("{}({})", top_level(name), args_str.join(", "))
                } else if self.script_classes.contains(name)
                    || BROWSER_CLASSES.contains(&name.as_str())
                {
                    format!("new {}({})", name, args_str.join(", "))
                } else {
                    format!("{}({})", name, args_str.join(", "))
                }
            }
            Expr::InterpolatedString(parts) => {
                let mut out = String::from("`");
                for part in parts {
                    match part {
                        StringPart::Literal(t) => out.push_str(&template_text(t)),
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
    pub(super) fn emit_store_statement(
        &mut self,
        stmt: &Statement,
        store_states: &[String],
        action_params: &[String],
    ) {
        match &stmt.kind {
            StatementKind::Assignment(a) => {
                let value = self.emit_store_expr(&a.value, store_states);
                if let Expr::Identifier(name) = &a.target {
                    if self.is_own_member(name, store_states) {
                        self.emit_line(&format!("store.{} = {};", name, value));
                    } else if action_params.contains(name)
                        || self.store_locals.borrow().contains(name)
                    {
                        self.emit_line(&format!("{} = {};", bare(name), value));
                    } else {
                        // A name that is neither state nor a parameter is a
                        // local of this action. It used to be assigned bare,
                        // which the bundle's strict mode refuses.
                        self.store_locals.borrow_mut().push(name.clone());
                        self.emit_line(&format!("let {} = {};", bare(name), value));
                    }
                } else if let Some((Expr::Identifier(root), steps)) = path_of(&a.target)
                    && self.is_own_member(root, store_states)
                    && !steps.is_empty()
                {
                    // `items[0].done = v` in a store action: the state
                    // assigned a copy with that place changed, so what reads
                    // it repaints.
                    let path = self.path_js(&steps, &|e| self.emit_store_expr(e, store_states));
                    self.emit_line(&format!(
                        "store.{root} = WF.setIn(store.{root}, {path}, {value});"
                    ));
                } else {
                    let target = self.emit_store_expr(&a.target, store_states);
                    self.emit_line(&format!("{} = {};", target, value));
                }
            }
            // `let n = x` in an action: a local the action may reassign. It
            // was emitted `const` and never recorded, so `n = n * 2` became a
            // second declaration — a SyntaxError that blanked the site, or a
            // TDZ error inside a loop.
            StatementKind::State(s) => {
                let val = self.emit_store_expr(&s.value, store_states);
                self.store_locals.borrow_mut().push(s.name.clone());
                self.emit_line(&format!("let {} = {};", bare(&s.name), val));
            }
            StatementKind::Navigate(expr) => {
                let path = self.emit_store_expr(expr, store_states);
                self.emit_line(&format!("WF.navigate({});", path));
            }
            StatementKind::ExprStatement(expr) => {
                // `items.sort(f)` on the store's state: run on a copy, which
                // is assigned back.
                if let Expr::MethodCall(obj, method, args) = expr
                    && MUTATORS.contains(&method.as_str())
                    && let Expr::Identifier(name) = obj.as_ref()
                    && self.is_own_member(name, store_states)
                {
                    let args: Vec<String> = args
                        .iter()
                        .map(|a| self.emit_store_expr(a, store_states))
                        .collect();
                    self.emit_line(&format!(
                        "store.{name} = WF.mutated(store.{name}, \"{method}\", [{}]);",
                        args.join(", ")
                    ));
                    return;
                }
                let val = self.emit_store_expr(expr, store_states);
                self.emit_line(&format!("{};", val));
            }
            StatementKind::If(if_stmt) => {
                // `if let x = e` binds the value for the branch, which runs
                // when it is not null; the binding was dropped here, so the
                // branch read a name nothing declared. Its own block keeps two
                // `if let x` in one action from declaring `x` twice.
                let bound = self.store_locals.borrow().len();
                let cond = match &if_stmt.binding {
                    Some(name) => {
                        let value = self.emit_store_expr(&if_stmt.condition, store_states);
                        self.emit_line("{");
                        self.indent += 1;
                        self.emit_line(&format!("const {} = {value};", bare(name)));
                        self.store_locals.borrow_mut().push(name.clone());
                        format!("{} != null", bare(name))
                    }
                    None => self.emit_store_expr(&if_stmt.condition, store_states),
                };
                self.emit_line(&format!("if ({}) {{", cond));
                self.indent += 1;
                for s in &if_stmt.then_body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                self.store_locals.borrow_mut().truncate(bound);
                // Each branch is a block: what it declares ends with it.
                for (cond, body) in &if_stmt.else_if_branches {
                    let cond = self.emit_store_expr(cond, store_states);
                    self.emit_line(&format!("}} else if ({cond}) {{"));
                    self.indent += 1;
                    for s in body {
                        self.emit_store_statement(s, store_states, action_params);
                    }
                    self.indent -= 1;
                    self.store_locals.borrow_mut().truncate(bound);
                }
                if let Some(else_body) = &if_stmt.else_body {
                    self.emit_line("} else {");
                    self.indent += 1;
                    for s in else_body {
                        self.emit_store_statement(s, store_states, action_params);
                    }
                    self.indent -= 1;
                    self.store_locals.borrow_mut().truncate(bound);
                }
                self.emit_line("}");
                if if_stmt.binding.is_some() {
                    self.indent -= 1;
                    self.emit_line("}");
                }
            }
            // `try`, `for` and `log` used to fall through to the page
            // emitter, whose statements read `_x()` signals a store does
            // not have.
            StatementKind::Try(t) => {
                let bound = self.store_locals.borrow().len();
                self.emit_line("try {");
                self.indent += 1;
                for s in &t.body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                self.store_locals.borrow_mut().truncate(bound);
                let param = t.param.clone().unwrap_or_else(|| "_error".to_string());
                self.emit_line(&format!("}} catch ({}) {{", bare(&param)));
                self.store_locals.borrow_mut().push(param.clone());
                self.indent += 1;
                for s in &t.catch_body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                self.store_locals.borrow_mut().truncate(bound);
                self.emit_line("}");
            }
            StatementKind::For(f) => {
                let list = self.emit_store_expr(&f.iterable, store_states);
                match &f.index {
                    Some(index) => self.emit_line(&format!(
                        "for (const [{}, {}] of Array.from({list}).entries()) {{",
                        bare(index),
                        bare(&f.item)
                    )),
                    None => self.emit_line(&format!("for (const {} of {list}) {{", bare(&f.item))),
                }
                // The bindings, and what the body declares, are the loop's.
                let bound = self.store_locals.borrow().len();
                self.store_locals.borrow_mut().push(f.item.clone());
                if let Some(index) = &f.index {
                    self.store_locals.borrow_mut().push(index.clone());
                }
                self.indent += 1;
                for s in &f.body {
                    self.emit_store_statement(s, store_states, action_params);
                }
                self.indent -= 1;
                self.store_locals.borrow_mut().truncate(bound);
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
}
