//! What reads and writes state: actions that write, effects that feed themselves, live browser values.

use super::*;

impl<'a, 'p> Checker<'a, 'p> {
    /// The names an action writes — its own body's assignments, and the
    /// store's state written as `Store.member` when it is a store's.
    pub(super) fn action_writes(&self, call: &Expr) -> Option<Vec<String>> {
        // What the action assigns that is state, not its own locals — a
        // `let`, or a name a store's action assigns without one, which
        // the store's code declares as a local.
        let locals = |body: &[Statement]| -> HashSet<String> {
            fn walk(stmts: &[Statement], out: &mut HashSet<String>) {
                for s in stmts {
                    if let StatementKind::State(st) = &s.kind {
                        out.insert(st.name.clone());
                    }
                    for b in s.kind.bodies() {
                        walk(b, out);
                    }
                }
            }
            let mut out = HashSet::new();
            walk(body, &mut out);
            out
        };
        match call {
            Expr::FunctionCall(name, _) if self.fixed_kind(name) == Some("an action") => {
                let body = self.own_actions.get(name)?;
                let own = locals(body);
                Some(
                    assigned(body, false)
                        .into_iter()
                        .filter(|n| !own.contains(n) && self.lookup(n).is_some())
                        .collect(),
                )
            }
            Expr::MethodCall(obj, method, _) => match obj.as_ref() {
                Expr::Identifier(store) => {
                    let body = self.world.store_actions.get(store.as_str())?.get(method)?;
                    let own = locals(body);
                    let members = self.world.stores.get(store.as_str());
                    let fixed = self.world.store_fixed.get(store.as_str());
                    Some(
                        assigned(body, false)
                            .into_iter()
                            .filter(|n| {
                                !own.contains(n)
                                    && members.is_some_and(|m| m.contains_key(n))
                                    && !fixed.is_some_and(|f| f.contains_key(n))
                            })
                            .map(|n| format!("{store}.{n}"))
                            .collect(),
                    )
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// X02: an effect that writes, every time it runs, a value it reads.
    /// Writing it runs the effect again, which writes it again: the page
    /// never settles, and the browser overflows its stack.
    pub(super) fn effect_feeds_itself(&mut self, body: &[Statement], span: Span) {
        let locals: HashSet<String> = body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::State(st) => Some(st.name.clone()),
                _ => None,
            })
            .collect();
        let mut writes: Vec<String> = assigned(body, true)
            .into_iter()
            .filter(|n| !locals.contains(n) && self.lookup(n).is_some())
            .collect();
        // What the actions it calls, unconditionally, write.
        for stmt in body {
            let call = match &stmt.kind {
                StatementKind::ExprStatement(e) => Some(e.clone()),
                StatementKind::MethodCall(mc) => Some(Expr::MethodCall(
                    Box::new(mc.object.clone()),
                    mc.method.clone(),
                    mc.args.clone(),
                )),
                _ => None,
            };
            if let Some(call) = call
                && let Some(w) = self.action_writes(&call)
            {
                writes.extend(w);
            }
        }
        let reads = read_names(body);
        if let Some(name) = writes.iter().find(|w| reads.contains(*w)) {
            self.error(
                span,
                "X02",
                format!("this effect writes `{name}`, which it also reads, so each run starts the next and the page never settles"),
                "Work the value out with `derived` instead, or write it from the action that changes what it reads",
            );
        }
    }

    /// X04: a `derived` value that calls an action that assigns. Working a
    /// value out must not change anything — it runs whenever what it reads
    /// changes, as often as the page needs it.
    pub(super) fn derived_assigns(&mut self, name: &str, value: &Expr, span: Span) {
        fn calls(e: &Expr, out: &mut Vec<Expr>) {
            if matches!(e, Expr::FunctionCall(..) | Expr::MethodCall(..)) {
                out.push(e.clone());
            }
            for child in e.children() {
                calls(child, out);
            }
        }
        let mut found = Vec::new();
        calls(value, &mut found);
        for call in found {
            if let Some(writes) = self.action_writes(&call)
                && let Some(w) = writes.first()
            {
                self.error(
                    span,
                    "X04",
                    format!("`{name}` is worked out by calling `{}`, which assigns `{w}`", expr_text(&call)),
                    "A derived value only reads; call the action from a handler, and derive from what it sets",
                );
                return;
            }
        }
    }

    /// X06: a `state` whose initial value reads a value the browser keeps
    /// current, and that nothing assigns, binds or changes in place. A
    /// state takes its value once, so it holds what that value was when it
    /// was made: `state clock = now(every: 1.seconds)` stops at the moment
    /// the page opened, and every "… ago" worked out from it with it. A
    /// `derived` value follows what it reads, and since nothing writes the
    /// state, making it one is the fix. A state the program does write
    /// starts from the live value on purpose — a tab seeded from the
    /// address, then changed by a click — and is left alone.
    pub(super) fn copies_live_value(&mut self, name: &str, value: &Expr, span: Span, body: Body) {
        if self.written.contains(name) {
            return;
        }
        let Some(live) = self.live_value(value) else {
            return;
        };
        let when = match body {
            Body::Store => "when the store is built",
            _ if self.component.is_some() => "when the component is drawn",
            _ => "when the page opens",
        };
        let d = Diagnostic::coded(
            "X06",
            format!("`{name}` copies `{live}` once, {when}, and never follows it"),
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source.as_deref())
        .with_hint(format!(
            "Nothing assigns `{name}`: declare it `derived`, which is worked out again whenever `{live}` changes"
        ))
        .with_plan(
            format!("Change to `derived {name}`"),
            crate::diagnostics::fixes::Plan::Rename {
                from: "state".to_string(),
                to: "derived".to_string(),
            },
        );
        self.info.findings.warnings.push(d);
    }

    /// The value the browser keeps current that `value` reads where it is
    /// worked out — not one read inside a lambda, which reads it each time
    /// it is called.
    pub(super) fn live_value(&self, value: &Expr) -> Option<&'static str> {
        match value {
            Expr::Lambda(..) => None,
            // `if let x = v { a } else { b }` is `v.__iflet(x => a, b)`, and
            // that lambda is worked out on the spot.
            Expr::MethodCall(subject, method, args) if method == "__iflet" => {
                self.live_value(subject).or_else(|| {
                    args.iter().find_map(|a| match a {
                        Expr::Lambda(_, body) => self.live_value(body),
                        other => self.live_value(other),
                    })
                })
            }
            Expr::Identifier(name) => self.browser_value(name),
            Expr::FunctionCall(name, args) => {
                let called = if name == "now" {
                    self.browser_value(name)
                } else {
                    None
                };
                called.or_else(|| args.iter().find_map(|a| self.live_value(a)))
            }
            other => other
                .children()
                .into_iter()
                .find_map(|c| self.live_value(c)),
        }
    }

    /// `name` as one of the values the browser keeps current, when the
    /// program has nothing of its own by that name.
    pub(super) fn browser_value(&self, name: &str) -> Option<&'static str> {
        let live = crate::registry::browser::BROWSER_VALUES
            .iter()
            .copied()
            .find(|v| *v == name)?;
        (self.lookup(name).is_none() && !self.world.consts.contains_key(name)).then_some(live)
    }
}
