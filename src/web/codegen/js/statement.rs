//! Statements in handlers and actions, and an event handler's head.

use super::*;

/// Whether a body shows something before the server has agreed, which is
/// what makes the action one that can be taken back.
pub(super) fn shows_optimistically(stmts: &[Statement]) -> bool {
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

/// Whether `stmt` was emitted with the states ([`JsCodegen::emit_states`]):
/// a `state`, or a `derived` one reads as it starts.
pub(super) fn emitted(stmt: &Statement, early: &[String]) -> bool {
    match &stmt.kind {
        StatementKind::State(_) => true,
        StatementKind::Derived(d) => early.contains(&d.name),
        _ => false,
    }
}

/// Where a statement is written: in a handler or a page's action, or in
/// a store's action, with the store's own members and the action's
/// parameters.
#[derive(Clone, Copy)]
pub(super) enum StmtCx<'s> {
    Page,
    Store {
        states: &'s [String],
        params: &'s [String],
    },
}

impl<'s> StmtCx<'s> {
    /// Where an expression in the statement is written.
    fn expr(self) -> Cx<'s> {
        match self {
            StmtCx::Page => Cx::Page,
            StmtCx::Store { states, .. } => Cx::Store(states),
        }
    }
}

impl JsCodegen {
    /// A statement in a handler or a page's action.
    pub(super) fn emit_statement(&mut self, stmt: &Statement) {
        self.emit_statement_in(stmt, StmtCx::Page);
    }

    /// Where a statement's bindings go: the names a block binds and
    /// declares, as a mark to come back to.
    fn scope_mark(&self, cx: StmtCx<'_>) -> (usize, usize) {
        match cx {
            StmtCx::Page => (self.loop_bindings.len(), self.block_lets.len()),
            StmtCx::Store { .. } => (self.store_locals.borrow().len(), 0),
        }
    }

    /// Unbind what was bound since `mark`.
    fn scope_reset(&mut self, cx: StmtCx<'_>, mark: (usize, usize)) {
        match cx {
            StmtCx::Page => {
                self.loop_bindings.truncate(mark.0);
                self.block_lets.truncate(mark.1);
            }
            StmtCx::Store { .. } => self.store_locals.borrow_mut().truncate(mark.0),
        }
    }

    /// Bind `name` as a plain name for what follows: an `if let`'s value, a
    /// loop's item, a caught error.
    fn bind(&mut self, cx: StmtCx<'_>, name: String) {
        match cx {
            StmtCx::Page => self.loop_bindings.push(name),
            StmtCx::Store { .. } => self.store_locals.borrow_mut().push(name),
        }
    }

    /// The statements of a block, each where `cx` says.
    fn emit_block_in(&mut self, stmts: &[Statement], cx: StmtCx<'_>) {
        self.indent += 1;
        for s in stmts {
            self.emit_statement_in(s, cx);
        }
        self.indent -= 1;
    }

    /// A statement where `cx` says it is written: `if`, `for`, `try`,
    /// `return` and a log are written once, and what assigns, declares or
    /// calls is written as its place reads it.
    pub(super) fn emit_statement_in(&mut self, stmt: &Statement, cx: StmtCx<'_>) {
        if matches!(cx, StmtCx::Page) {
            self.map_at(stmt.span);
        }
        let ecx = cx.expr();
        match (&stmt.kind, cx) {
            (StatementKind::Assignment(a), StmtCx::Page) => {
                let target = self.emit_expr(&a.target);
                let value = self.emit_expr(&a.value);
                if let Some(update) = self.immutable_update(&a.target, &value) {
                    self.emit_line(&update);
                    return;
                }
                // Check if target is a signal (state variable)
                if let Expr::Identifier(name) = &a.target {
                    self.emit_line(&format!("{}.set({});", sig(name), value));
                } else if let Expr::PropertyAccess(base, prop) = &a.target {
                    let base_str = self.emit_expr(base);
                    self.emit_line(&format!("{base_str}.{prop} = {value};"));
                } else {
                    self.emit_line(&format!("{target} = {value};"));
                }
            }
            (
                StatementKind::Assignment(a),
                StmtCx::Store {
                    states: store_states,
                    params: action_params,
                },
            ) => {
                let value = self.emit_store_expr(&a.value, store_states);
                if let Expr::Identifier(name) = &a.target {
                    if self.is_own_member(name, store_states) {
                        self.emit_line(&format!("store.{name} = {value};"));
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
                    self.emit_line(&format!("{target} = {value};"));
                }
            }
            (StatementKind::State(s), StmtCx::Page) => {
                let val = self.emit_expr(&s.value);
                self.emit_line(&format!("const {} = WF.signal({});", sig(&s.name), val));
                self.block_lets.push(s.name.clone());
            }
            // `let n = x` in an action: a local the action may reassign. It
            // was emitted `const` and never recorded, so `n = n * 2` became a
            // second declaration — a SyntaxError that blanked the site, or a
            // TDZ error inside a loop.
            (StatementKind::State(s), StmtCx::Store { states, .. }) => {
                let val = self.emit_store_expr(&s.value, states);
                self.store_locals.borrow_mut().push(s.name.clone());
                self.emit_line(&format!("let {} = {};", bare(&s.name), val));
            }
            (StatementKind::ExprStatement(expr), StmtCx::Page) => {
                if let Some(update) = self.mutating_call(expr) {
                    self.emit_line(&update);
                    return;
                }
                let val = self.emit_expr(expr);
                self.emit_line(&format!("{val};"));
            }
            (StatementKind::ExprStatement(expr), StmtCx::Store { states, .. }) => {
                // `items.sort(f)` on the store's state: run on a copy, which
                // is assigned back.
                if let Expr::MethodCall(obj, method, args) = expr
                    && MUTATORS.contains(&method.as_str())
                    && let Expr::Identifier(name) = obj.as_ref()
                    && self.is_own_member(name, states)
                {
                    let args: Vec<String> = args
                        .iter()
                        .map(|a| self.emit_store_expr(a, states))
                        .collect();
                    self.emit_line(&format!(
                        "store.{name} = WF.mutated(store.{name}, \"{method}\", [{}]);",
                        args.join(", ")
                    ));
                    return;
                }
                let val = self.emit_store_expr(expr, states);
                self.emit_line(&format!("{val};"));
            }
            (StatementKind::Navigate(expr), StmtCx::Page) => {
                let path = self.route_value(expr);
                self.emit_line(&format!("WF.navigate({path});"));
            }
            (StatementKind::Navigate(expr), StmtCx::Store { states, .. }) => {
                let path = self.emit_store_expr(expr, states);
                self.emit_line(&format!("WF.navigate({path});"));
            }
            (StatementKind::MethodCall(mc), _) => {
                let obj = self.emit_expr_in(&mc.object, ecx);
                let args: Vec<String> = mc.args.iter().map(|a| self.emit_expr_in(a, ecx)).collect();
                self.emit_line(&format!("{}.{}({});", obj, mc.method, args.join(", ")));
            }
            (StatementKind::Log(expr), _) => {
                let val = self.emit_expr_in(expr, ecx);
                self.emit_line(&format!("console.log({val});"));
            }
            (StatementKind::Return(expr), _) => match expr {
                Some(e) => {
                    let val = self.emit_expr_in(e, ecx);
                    self.emit_line(&format!("return {val};"));
                }
                None => self.emit_line("return;"),
            },
            (StatementKind::If(if_stmt), _) => {
                // `if let x = e` binds the value for the branch, which runs
                // when it is not null.
                let mark = self.scope_mark(cx);
                let cond = match &if_stmt.binding {
                    Some(name) => {
                        let value = self.emit_expr_in(&if_stmt.condition, ecx);
                        // In a block of its own, so two `if let x` in one
                        // action do not declare `x` twice.
                        self.emit_line("{");
                        self.indent += 1;
                        self.emit_line(&format!("const {} = {};", bare(name), value));
                        // The branch reads it as a plain name — it is a
                        // `const` here, not a signal of the body's own.
                        self.bind(cx, name.clone());
                        format!("{} != null", bare(name))
                    }
                    None => self.emit_expr_in(&if_stmt.condition, ecx),
                };
                self.emit_line(&format!("if ({cond}) {{"));
                // Each branch is a block: what it declares ends with it.
                self.emit_block_in(&if_stmt.then_body, cx);
                self.scope_reset(cx, mark);
                // An `else if` chain used to be dropped here.
                for (cond, body) in &if_stmt.else_if_branches {
                    let cond = self.emit_expr_in(cond, ecx);
                    self.emit_line(&format!("}} else if ({cond}) {{"));
                    self.emit_block_in(body, cx);
                    self.scope_reset(cx, mark);
                }
                if let Some(else_body) = &if_stmt.else_body {
                    self.emit_line("} else {");
                    self.emit_block_in(else_body, cx);
                    self.scope_reset(cx, mark);
                }
                self.emit_line("}");
                if if_stmt.binding.is_some() {
                    self.indent -= 1;
                    self.emit_line("}");
                }
            }
            // `for x in xs { … }` in an action: the loop runs once, in order;
            // the item and the index are plain names inside it.
            (StatementKind::For(f), _) => {
                let list = self.emit_expr_in(&f.iterable, ecx);
                match &f.index {
                    Some(index) => self.emit_line(&format!(
                        "for (const [{}, {}] of Array.from({list}).entries()) {{",
                        bare(index),
                        bare(&f.item)
                    )),
                    None => self.emit_line(&format!("for (const {} of {list}) {{", bare(&f.item))),
                }
                // The bindings, and what the body declares, are the loop's.
                let mark = self.scope_mark(cx);
                self.bind(cx, f.item.clone());
                if let Some(index) = &f.index {
                    self.bind(cx, index.clone());
                }
                self.emit_block_in(&f.body, cx);
                self.scope_reset(cx, mark);
                self.emit_line("}");
            }
            (StatementKind::Try(t), _) => {
                let mark = self.scope_mark(cx);
                self.emit_line("try {");
                self.emit_block_in(&t.body, cx);
                self.scope_reset(cx, mark);
                let param = t.param.clone().unwrap_or_else(|| "_error".to_string());
                self.emit_line(&format!("}} catch ({}) {{", bare(&param)));
                self.bind(cx, param);
                self.emit_block_in(&t.catch_body, cx);
                self.scope_reset(cx, mark);
                self.emit_line("}");
            }
            (StatementKind::Animate(anim), _) => {
                let dur = anim
                    .duration
                    .as_deref()
                    .map(|d| format!(", {}", js_str(d)))
                    .unwrap_or_default();
                self.emit_line(&format!(
                    "WF.animate({}, {}{});",
                    js_str(&anim.target),
                    js_str(&anim.animation),
                    dur
                ));
            }
            // `emit toggle(id)`: the handler the caller passed for the
            // event, when it passed one.
            (StatementKind::Emit(e), _) => {
                let args: Vec<String> = e.args.iter().map(|a| self.emit_expr_in(a, ecx)).collect();
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
    pub(super) fn emit_statements_inline(&mut self, stmts: &[Statement]) -> String {
        let saved = std::mem::take(&mut self.output);
        let indent = std::mem::replace(&mut self.indent, 0);
        let lets = self.block_lets.len();
        for stmt in stmts {
            self.emit_statement(stmt);
        }
        self.block_lets.truncate(lets);
        // A marker no line took stays inside: it is not the outer code's.
        self.pending_mark = None;
        let out = std::mem::replace(&mut self.output, saved);
        self.indent = indent;
        out.split('\n')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A handler's body: the same as any imperative block. The handler's
    /// named parameters (`on click(e)`, `on menu(key, x, y)`) are plain
    /// names inside it; a handler
    /// of the original grammar reads an implicit `event`.
    pub(super) fn emit_event_body(&mut self, handler: &EventHandler) -> String {
        let names = handler.params.len();
        self.loop_bindings.extend(handler.params.iter().cloned());
        let body = self.emit_statements_inline(&handler.body);
        self.loop_bindings
            .truncate(self.loop_bindings.len() - names);
        body
    }

    /// The head of a handler's arrow function: its parameter, and `async`
    /// when the body awaits.
    /// A component's event sends as many values as it declares, and the
    /// handler names them in order: `on menu(key, x, y)` is `(key, x, y)`.
    pub(super) fn handler_head(&self, handler: &EventHandler, default_param: &str) -> String {
        let params = if handler.params.len() > 1 {
            bare_list(&handler.params.join(", "))
        } else {
            let param = match handler.param() {
                Some(p) => bare(p),
                None => self.implicit_param(default_param),
            };
            format!("{param}{}", self.implicit_handler_names(handler, &param))
        };
        if crate::syntax::parser::ast::awaits(&handler.body) {
            format!("async ({params})")
        } else {
            format!("({params})")
        }
    }

    /// The parameter a handler gets when it names none — `event`, `reason`
    /// — is read by that name in its body, unless the program binds the
    /// name itself: then the program's wins, and the parameter is `$$event`.
    /// A loop's `for event in events` read in `on click { pick(event) }`
    /// was handed the click, and an action `event()` could not be called.
    pub(super) fn implicit_param(&self, name: &str) -> String {
        if self.program_binds(name) {
            format!("$${name}")
        } else {
            name.to_string()
        }
    }

    /// Whether the program has `name` in reach where it is being read: the
    /// body's own, a loop's or a lambda's, an action, a handle, a constant,
    /// a store or a service.
    pub(super) fn program_binds(&self, name: &str) -> bool {
        let n = name.to_string();
        self.body.own_names.contains(&n)
            || self.loop_bindings.contains(&n)
            || self.lambda_params.borrow().contains(&n)
            || self.body.own_actions.contains(&n)
            || self.consts.contains(&n)
            || self.body.refs.contains(&n)
            || self.stores.contains(&n)
    }

    /// `key` and `value` are names inside a handler — the checker accepts
    /// `on keydown { if key == "Enter" { … } }` — but nothing declared them,
    /// so the handler threw `ReferenceError: key is not defined` on the first
    /// key press. A handler that reads either gets it as a defaulted
    /// parameter, read off its event: `key` the event's key, `value` its
    /// target's value. A name the body already has — a state, a loop item,
    /// the handler's own parameter — is left as it is.
    pub(super) fn implicit_handler_names(&self, handler: &EventHandler, param: &str) -> String {
        let param = param.trim();
        if param.is_empty()
            || !param
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        {
            return String::new();
        }
        let mut body = handler.body.clone();
        let (mut key, mut value) = (false, false);
        crate::syntax::parser::ast::walk_exprs_mut(&mut body, &mut |e| match e {
            Expr::Identifier(n) if n == "key" => key = true,
            Expr::Identifier(n) if n == "value" => value = true,
            _ => {}
        });
        // A store, a constant or an action of the name counts too: `store
        // value` read in `on click { value.add() }` was the input's value.
        let free = |name: &str| param != name && !self.program_binds(name);
        let mut out = String::new();
        if key && free("key") {
            out.push_str(&format!(", key = {param}?.key"));
        }
        if value && free("value") {
            out.push_str(&format!(", value = {param}?.target?.value"));
        }
        out
    }

    /// Compile one `style { prop: value }` declaration to `(camelCaseProperty, jsValue)`.
    ///
    /// A bare design-token keyword (`font-size: xl`, `padding: md`) resolves to a
    /// `"var(--…)"` string literal; property aliases (`radius`, `shadow`) map to their
    /// real CSS property. Anything else keeps the normal expression path, so quoted
    /// CSS, numbers, and reactive-state identifiers (`font-size: someState` → `_someState()`)
    /// are unchanged.
    pub(super) fn emit_style_decl(&self, prop: &StyleProperty) -> (String, String) {
        use crate::registry::style_tokens::{canonical_style_prop, resolve_style_token};
        let css_prop = canonical_style_prop(&prop.name);
        let val = match resolve_style_token(&css_prop, &prop.value) {
            Some(css) => format!("\"{css}\""),
            None => self.emit_expr(&prop.value),
        };
        (to_camel_case(&css_prop), val)
    }
}
