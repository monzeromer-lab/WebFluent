//! Resources and connections: the request, its cache, retry and refetch policies, sockets, streams, channels and peers.

use super::{
    ConnectionDecl, ConnectionKind, Expr, JsCodegen, ResourceDecl, Statement, StatementKind, bare,
    capitalize_first, refinement_in, sig,
};

/// The names of the resources a body declares, at any depth.
pub(super) fn resource_names(stmts: &[Statement]) -> Vec<String> {
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

    let mut names = Vec::new();
    walk(stmts, &mut names);
    names
}

/// The word `fetch` knows a setting's case by: `.sameOrigin` is
/// `same-origin`, and the rest are themselves.
pub(super) fn fetch_word(case: &str) -> String {
    match case {
        "sameOrigin" => "same-origin".to_string(),
        "noCors" => "no-cors".to_string(),
        "noReferrer" => "no-referrer".to_string(),
        other => other.to_string(),
    }
}

/// The name the runtime knows a service's setting by.
pub(super) fn api_setting(key: &str) -> &str {
    match key {
        "referrer" => "referrerPolicy",
        other => other,
    }
}

impl JsCodegen {
    /// `resource rows = fetch(url, opts)`: the request, made once here and
    /// again whenever a URL that reads state changes.
    pub(super) fn emit_resource(&mut self, r: &ResourceDecl) {
        let call = self.resource_call(r, None);
        self.emit_line(&format!("const {} = {call};", sig(&r.name)));
    }

    /// An expression in a page's terms, or — when `store` names a store's
    /// members — in that store's.
    pub(super) fn term(&self, e: &Expr, store: Option<&[String]>) -> String {
        match store {
            Some(states) => self.emit_store_expr(e, states),
            None => self.emit_expr(e),
        }
    }

    /// `WF.resource(…)` for a resource: a page's, or a store's when `store`
    /// names the store's members, its address and options then read in the
    /// store's terms.
    pub(super) fn resource_call(&self, r: &ResourceDecl, store: Option<&[String]>) -> String {
        // `resource rows = Backend.rows(page: n)`: an endpoint, which
        // carries its own address, headers, cache and abort. The arguments
        // are read again on every load, so one that reads state makes the
        // request again when it changes.
        if let Expr::MethodCall(object, endpoint, args) = &r.url {
            let call = format!("{}.{}", self.term(object, store), endpoint);
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
                    refetch = Some(self.refetch_policy_in(value, store));
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
                        .and_then(|(_, v)| match (v, store) {
                            (Expr::Identifier(name), None) => {
                                Some(format!(", set: (v) => {}.set(v)", sig(name)))
                            }
                            (Expr::Identifier(name), Some(states)) if states.contains(name) => {
                                Some(format!(", set: (v) => {{ store.{name} = v; }}"))
                            }
                            _ => None,
                        })
                        .unwrap_or_default();
                    paginate = Some(format!("{{ by: \"{by}\"{set} }}"));
                    continue;
                }
                let emitted = match key {
                    "cache" => self.cache_policy_in(value, store),
                    _ => self.term(value, store),
                };
                parts.push(format!("{key}: {emitted}"));
            }
            // A single positional argument is the one the path names.
            if let Some(first) = positional.first() {
                parts.insert(0, format!("id: {}", self.term(first, store)));
            }
            let on = match refetch {
                Some(policy) => format!(", on: {policy}"),
                None => String::new(),
            };
            let paging = match paginate {
                Some(policy) => format!(", paginate: {policy}"),
                None => String::new(),
            };
            return format!(
                "WF.resource({call}, {{ call: true, args: () => ({{ {} }}){on}{paging} }})",
                parts.join(", ")
            );
        }
        let url = self.encoded_address_in(&r.url, store);
        // A store's address is read in a function unless it is written out
        // whole: what it reads — the store's state, another store's — is
        // only known to be a signal when it is called.
        let reactive = match store {
            Some(_) => !matches!(r.url, Expr::StringLiteral(_)),
            None => self.is_reactive(&url),
        };
        let url_js = if reactive {
            format!("() => {url}")
        } else {
            url
        };
        // `on:` and `cache:` are policies, read as an endpoint's are: `on:
        // .focus` was handed over as the word "focus", which has no `focus`,
        // so an address's resource never looked again.
        let mut opts: Vec<String> = r
            .options
            .iter()
            .map(|opt| {
                let value = match opt.key.trim_matches('"') {
                    "on" => self.refetch_policy_in(&opt.value, store),
                    "cache" => self.cache_policy_in(&opt.value, store),
                    _ => self.term(&opt.value, store),
                };
                format!("{}: {value}", opt.key)
            })
            .collect();
        if self.dev
            && let Some(ty) = &r.ty
        {
            opts.push(format!("shape: {}", self.shape_of(ty, 0)));
        }
        let opts_js = if opts.is_empty() {
            "null".to_string()
        } else {
            format!("{{ {} }}", opts.join(", "))
        };
        format!("WF.resource({url_js}, {opts_js})")
    }

    /// `.swr(60.seconds)`, `.none`, `.forever`, `.cache(5.minutes)` — how
    /// long an answer stands, as the engine reads it.
    pub(super) fn cache_policy(&self, value: &Expr) -> String {
        self.cache_policy_in(value, None)
    }

    pub(super) fn cache_policy_in(&self, value: &Expr, store: Option<&[String]>) -> String {
        match value {
            Expr::EnumCase(name) => format!("{{ kind: \"{name}\" }}"),
            Expr::CaseValue(name, args) => {
                let ttl = args
                    .first()
                    .map_or_else(|| "0".to_string(), |a| self.term(a, store));
                format!("{{ kind: \"{name}\", ttl: {ttl} }}")
            }
            other => self.term(other, store),
        }
    }

    /// `.backoff(times: 3, on: [.network, .status5xx])`, or a plain count —
    /// how many times to ask again, and for what.
    pub(super) fn retry_policy(&self, value: &Expr) -> String {
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
    /// look again; `store` reads it in a store's terms.
    pub(super) fn refetch_policy_in(&self, value: &Expr, store: Option<&[String]>) -> String {
        let one = |e: &Expr| -> Option<String> {
            match e {
                Expr::EnumCase(name) if name == "never" => Some(String::new()),
                Expr::EnumCase(name) => Some(format!("{name}: true")),
                Expr::CaseValue(name, args) => Some(format!(
                    "{name}: {}",
                    args.first()
                        .map_or_else(|| "0".to_string(), |a| self.term(a, store))
                )),
                _ => None,
            }
        };
        let parts: Vec<String> = match value {
            Expr::ListLiteral(items) => items.iter().filter_map(one).collect(),
            other => one(other).into_iter().collect(),
        };
        if parts.is_empty() {
            return self.term(value, store);
        }
        format!("{{ {} }}", parts.join(", "))
    }

    /// What the state's declared type says its values must be.
    pub(super) fn refinement_of(&self, name: &str) -> Vec<(String, Expr)> {
        refinement_in(&self.body.current_body, name)
    }

    /// A store's connection: `name: (store) => WF.ws(…)`, its options and
    /// handlers read in the store's terms.
    pub(super) fn emit_store_connection(&mut self, c: &ConnectionDecl, store_states: &[String]) {
        let opener = Self::connection_opener(c.kind);
        let url = c
            .url
            .as_ref()
            .map(|u| self.emit_store_expr(u, store_states));
        self.emit_line(&format!(
            "{}: (store) => WF.{opener}({}{{",
            c.name,
            url.map(|u| format!("{u}, ")).unwrap_or_default()
        ));
        self.indent += 1;
        for (key, value) in &c.options {
            let v = self.emit_store_expr(value, store_states);
            self.emit_line(&format!("{key}: {v},"));
        }
        for handler in &c.handlers {
            let param = handler.param().unwrap_or("message").to_string();
            self.emit_line(&format!(
                "on{}: {}({}) => {{",
                capitalize_first(&handler.event),
                if crate::syntax::parser::ast::awaits(&handler.body) {
                    "async "
                } else {
                    ""
                },
                bare(&param)
            ));
            self.indent += 1;
            self.store_locals.borrow_mut().clear();
            self.store_locals.borrow_mut().push(param.clone());
            let visible: Vec<String> = store_states
                .iter()
                .filter(|n| **n != param)
                .cloned()
                .collect();
            for stmt in &handler.body {
                self.emit_store_statement(stmt, &visible, std::slice::from_ref(&param));
            }
            self.store_locals.borrow_mut().clear();
            self.indent -= 1;
            self.emit_line("},");
        }
        self.indent -= 1;
        self.emit_line("}),");
    }

    pub(super) fn connection_opener(kind: ConnectionKind) -> &'static str {
        match kind {
            ConnectionKind::Socket => "ws",
            ConnectionKind::Stream => "sse",
            ConnectionKind::Channel => "broadcast",
            ConnectionKind::Peer => "rtc",
            ConnectionKind::Worker => "worker",
        }
    }

    /// `socket chat = ws(…)`, `stream t = sse(…)`, `channel c =
    /// broadcast(…)`: a connection the scope owns, and closes.
    pub(super) fn emit_connection(&mut self, c: &ConnectionDecl) {
        let opener = Self::connection_opener(c.kind);
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
            let param = match handler.param() {
                Some(p) => bare(p),
                None => self.implicit_param("message"),
            };
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
        self.emit_line(&format!("const {} = WF.{opener}({args});", sig(&c.name)));
    }
}
