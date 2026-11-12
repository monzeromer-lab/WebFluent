//! Statements: declarations, assignments, `if`, `match`, handlers and the names a handler binds.

use super::*;

impl Checker<'_, '_> {
    /// Binds the names a handler gives its event's values (`on menu(key, x,
    /// y)`), each typed as `sends` says, and reports a name past the last
    /// value sent: it would only ever be `undefined`.
    pub(super) fn handler_names(&mut self, handler: &EventHandler, sends: &[Type], what: &str) {
        self.handler_names_hinted(
            handler,
            sends,
            what,
            &format!(
                "Name the one value, `on {}(e)`, and read what it carries from it: `e.target.value`, `e.clientX`",
                handler.event
            ),
        );
    }

    pub(super) fn handler_names_hinted(
        &mut self,
        handler: &EventHandler,
        sends: &[Type],
        what: &str,
        hint: &str,
    ) {
        for (i, name) in handler.params.iter().enumerate() {
            let ty = sends.get(i).cloned().unwrap_or(Type::Any);
            self.bind(name, ty, handler.span);
        }
        if handler.params.len() > sends.len() {
            let n = handler.params.len();
            let span = self.handler_extra_names(handler, sends.len());
            self.error(
                span,
                "T22",
                format!(
                    "`on {}` names {n} value{}, but {what}",
                    handler.event,
                    if n == 1 { "" } else { "s" },
                ),
                hint,
            );
        }
    }

    /// Where the names past the `kept`-th are written in the handler's
    /// head, `on menu(key, x, y, z)` → `z`; the handler's first line when
    /// the source is not at hand.
    pub(super) fn handler_extra_names(&self, handler: &EventHandler, kept: usize) -> Span {
        let head = Span {
            end: handler.span.start,
            ..handler.span
        };
        let Some(source) = self.source else {
            return head;
        };
        let start = handler.span.start as usize;
        let end = (handler.span.end as usize).min(source.len());
        let Some(text) = source.get(start..end) else {
            return head;
        };
        let (Some(open), Some(close)) = (text.find('('), text.find(')')) else {
            return head;
        };
        if close < open {
            return head;
        }
        // The names start after the `(`, past the key's string when there is
        // one; the extra ones after the `kept`-th comma that follows.
        let inside = &text[open + 1..close];
        let mut from = 0;
        let skip = kept + usize::from(handler.key.is_some());
        for _ in 0..skip {
            match inside[from..].find(',') {
                Some(at) => from += at + 1,
                None => return head,
            }
        }
        let lead = inside[from..].len() - inside[from..].trim_start().len();
        let first = start + open + 1 + from + lead;
        let last = start + open + 1 + inside.trim_end().len();
        let before = &source[..first];
        let line = before.matches('\n').count() as u32 + 1;
        // A column counts characters, as every other column does: on a
        // line of Arabic names a byte count lands far to the right.
        let line_start = before.rfind('\n').map_or(0, |n| n + 1);
        let col = before[line_start..].chars().count() as u32 + 1;
        Span::new(first as u32, last.max(first) as u32, line, col)
    }

    pub(super) fn statements(&mut self, stmts: &[Statement], body: Body) {
        // What follows a `return` in the same block never runs.
        if let Some(at) = stmts
            .iter()
            .position(|s| matches!(s.kind, StatementKind::Return(_)))
            && let Some(next) = stmts.get(at + 1)
        {
            self.current_span = next.span;
            self.warn(
                "U06",
                "this never runs: the `return` above leaves first".to_string(),
                "Remove it, or move it above the `return`",
            );
            self.plan_last_warning(
                "Remove what never runs".to_string(),
                crate::diagnostics::fixes::Plan::RemoveStatement,
            );
        }
        for stmt in stmts {
            self.statement(stmt, body);
            // `if x == null { return }`: what follows runs only when `x`
            // is not null.
            if let StatementKind::If(i) = &stmt.kind
                && i.binding.is_none()
                && i.else_if_branches.is_empty()
                && i.else_body.is_none()
                && i.then_body
                    .last()
                    .is_some_and(|s| matches!(s.kind, StatementKind::Return(_)))
            {
                let names = null_names(&i.condition);
                self.narrow_all(&names);
            }
        }
    }

    pub(super) fn statement(&mut self, stmt: &Statement, body: Body) {
        let span = stmt.span;
        self.current_span = span;
        match &stmt.kind {
            StatementKind::State(s) => {
                let declared = s.ty.as_ref().map(|t| self.world.resolve(Type::from_ref(t)));
                let given = self.infer(&s.value, declared.as_ref());
                if let Some(declared) = &declared {
                    self.expect(&given, declared, span, &format!("`{}`", s.name));
                }
                // What `persist` keeps is written as JSON: a function, a
                // file or a promise does not survive it.
                if s.persist {
                    let kept = declared.clone().unwrap_or_else(|| given.clone());
                    if let Some(what) = not_serialisable(&kept) {
                        self.error(
                            span,
                            "D02",
                            format!("`persist {}` keeps {what}, which cannot be written to the browser's storage", s.name),
                            "Keep what it is made from — a name, an id, the data — and build it again when it is read",
                        );
                    }
                }
                // A secret must not outlive the visit.
                if s.persist
                    && declared.as_ref().map(super::ty::Type::unwrapped)
                        == Some(Type::Scalar(Scalar::Secret))
                {
                    self.error(
                        span,
                        "T12",
                        format!("`{}` is a `Secret`, and `persist` would keep it in the browser", s.name),
                        "Hold it in `state`, which the visit ends; what must outlive it belongs on the server",
                    );
                }
                // And what the type says its values must be.
                if let Some(ty) = &s.ty
                    && let Some(fault) = refinement_fault(ty, &s.value)
                {
                    self.error(
                        span,
                        "T01",
                        format!("`{}` {fault}", s.name),
                        "The type says what its values may be; this one is outside it",
                    );
                }
                if body == Body::Imperative {
                    // A local: bound here, in order.
                    self.bind(&s.name, declared.unwrap_or(given), span);
                } else if !s.persist {
                    self.copies_live_value(&s.name, &s.value, span, body);
                }
            }
            StatementKind::Derived(d) => {
                self.derived_assigns(&d.name, &d.value, span);
                let declared = d.ty.as_ref().map(|t| self.world.resolve(Type::from_ref(t)));
                self.in_derived = true;
                let given = self.infer(&d.value, declared.as_ref());
                self.in_derived = false;
                if let Some(declared) = &declared {
                    // A derived value says what it is; what it works out to
                    // must be that.
                    self.expect(&given, declared, span, &format!("`{}`", d.name));
                }
                self.narrow(&d.name, declared.unwrap_or(given));
            }
            StatementKind::Resource(r) => {
                // An `api` endpoint is a call, not an address.
                if matches!(&r.url, Expr::MethodCall(..) | Expr::FunctionCall(..)) {
                    self.infer(&r.url, None);
                } else {
                    let url = self.infer(&r.url, Some(&Type::String));
                    self.expect(&url, &Type::String, span, "a resource's URL");
                }
                for option in &r.options {
                    self.infer(&option.value, None);
                }
            }
            // The rules a value must meet, checked against what it is: a
            // length on a number, or a match against a name that is not
            // there, is a fault where it is written.
            StatementKind::Validate(v) => {
                // The rules are shown on the control bound to the state, and
                // a form is valid only when they pass: with no control, they
                // never can, and the submit stays disabled.
                if self.fixed_kind(&v.name) == Some("a `derived` value") {
                    self.error(
                        span,
                        "F03",
                        format!(
                            "`validate {}` guards a `derived` value, which nobody types into",
                            v.name
                        ),
                        "Validate the state the reader edits; a derived value follows it",
                    );
                } else if self.lookup(&v.name).is_some() && !self.bound.contains(&v.name) {
                    self.error(
                        span,
                        "F03",
                        format!("`validate {}` guards a state no control binds, so its rules can never be met", v.name),
                        &format!("Bind it to the control the reader fills in: `Input(bind: {}, …)`", v.name),
                    );
                }
                let guarded = self.lookup(&v.name);
                if guarded.is_none() {
                    self.error(
                        span,
                        "T05",
                        format!("`{}` is not a name this page declares", v.name),
                        "A `validate` block guards a `state` of the page or component it is in",
                    );
                }
                let ty = guarded.unwrap_or(Type::Any).unwrapped();
                for rule in &v.rules {
                    for e in rule.args.iter().chain(rule.message.iter()) {
                        self.infer(e, None);
                    }
                    if let Some(body) = &rule.body {
                        // An `async` rule's check is an async function: it
                        // may `await` the server.
                        let was = std::mem::replace(&mut self.async_ok, rule.name == "async");
                        let checked = self.infer(body, Some(&Type::Bool));
                        self.async_ok = was;
                        self.expect(&checked, &Type::Bool, rule.span, "a rule's check");
                    }
                    let Some(wanted) = rule_wants(&rule.name) else {
                        self.error(
                            rule.span,
                            "T05",
                            format!("`{}` is not a rule", rule.name),
                            "The rules are `required`, `email`, `url`, `minLength`, `maxLength`, `min`, `max`, `pattern`, `matches`, `oneOf`, `custom` and `async`",
                        );
                        continue;
                    };
                    if !ty.is_any() && !wanted.iter().any(|w| ty.assignable_to(w)) {
                        let names: Vec<String> = wanted
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect();
                        self.error(
                            rule.span,
                            "T01",
                            format!(
                                "`{}` is `{ty}`, and `{}` is a rule for {}",
                                v.name,
                                rule.name,
                                names.join(" or ")
                            ),
                            "",
                        );
                    }
                }
            }
            StatementKind::Connection(c) => {
                if let Some(url) = &c.url {
                    let url = self.infer(url, Some(&Type::String));
                    self.expect(&url, &Type::String, span, "a connection's address");
                }
                for (_, value) in &c.options {
                    self.infer(value, None);
                }
                for handler in &c.handlers {
                    self.push_scope();
                    let ty = match &c.receives {
                        Some(t) => self.world.resolve(Type::from_ref(t)),
                        None => Type::Any,
                    };
                    self.handler_names(
                        handler,
                        &[ty],
                        &format!("a connection's `on {}` receives one value", handler.event),
                    );
                    let was = std::mem::replace(&mut self.async_ok, true);
                    self.in_handler += 1;
                    self.statements(&handler.body, Body::Imperative);
                    self.in_handler -= 1;
                    self.async_ok = was;
                    self.pop_scope();
                }
            }
            StatementKind::Use(_) => {}
            StatementKind::Action(a) => {
                self.push_scope();
                for p in &a.params {
                    let ty = self.world.resolve(Type::from_ref(&p.param_type));
                    self.bind(&p.name, ty, span);
                }
                self.returns.push(Vec::new());
                let was = std::mem::replace(&mut self.async_ok, true);
                self.statements(&a.body, Body::Imperative);
                self.async_ok = was;
                let returned = self.returns.pop().unwrap_or_default();
                self.pop_scope();
                let ret = returned
                    .into_iter()
                    .reduce(Type::join)
                    .unwrap_or(Type::Null);
                let ret = if awaits(&a.body) {
                    Type::Promise(Box::new(ret))
                } else {
                    ret
                };
                let params: Vec<Type> = a
                    .params
                    .iter()
                    .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                    .collect();
                let func = Type::Func(params, Box::new(ret));
                self.narrow(&a.name, func.clone());
                if let Some(t) =
                    self.info.bindings.iter_mut().rev().find(|t| {
                        t.decl == self.decl && t.name == a.name && t.span.start == span.start
                    })
                {
                    t.ty = func;
                }
            }
            StatementKind::Effect(e) => {
                self.effect_feeds_itself(&e.body, span);
                self.push_scope();
                let was = std::mem::replace(&mut self.async_ok, false);
                self.statements(&e.body, Body::Imperative);
                self.statements(&e.cleanup, Body::Imperative);
                self.async_ok = was;
                self.pop_scope();
            }
            StatementKind::Timer(t) => {
                let ty = self.infer(&t.interval, Some(&Type::Number));
                if !ty.assignable_to(&Type::Number) {
                    self.error_at_current(
                        "T01",
                        format!(
                            "`{}` takes milliseconds, but `{}` is `{ty}`",
                            if t.every { "every" } else { "after" },
                            expr_text(&t.interval)
                        ),
                        "",
                    );
                }
                self.push_scope();
                let was = std::mem::replace(&mut self.async_ok, true);
                self.statements(&t.body, Body::Imperative);
                self.async_ok = was;
                self.pop_scope();
            }
            StatementKind::EventHandler(h) => {
                self.push_scope();
                self.handler_names(h, &[Type::Any], DOM_SENDS);
                let was = std::mem::replace(&mut self.async_ok, true);
                self.in_handler += 1;
                self.statements(&h.body, Body::Imperative);
                self.in_handler -= 1;
                self.async_ok = was;
                self.pop_scope();
            }
            StatementKind::UIElement(el) => self.element(el, span),
            StatementKind::If(i) => self.if_statement(i, span, body),
            StatementKind::Try(t) => {
                self.push_scope();
                self.statements(&t.body, body);
                self.pop_scope();
                self.push_scope();
                if let Some(param) = &t.param {
                    self.bind(param, Type::Any, span);
                }
                self.statements(&t.catch_body, body);
                self.pop_scope();
            }
            StatementKind::For(f) => {
                let iterable = self.infer(&f.iterable, None);
                let item = match &iterable {
                    Type::List(inner) => (**inner).clone(),
                    Type::Any | Type::Map => Type::Any,
                    other => {
                        self.error(
                            span,
                            "T08",
                            format!(
                                "`for` loops over a list, but `{}` is `{other}`",
                                expr_text(&f.iterable)
                            ),
                            "Give it a list, or `.split(…)` a string first",
                        );
                        Type::Any
                    }
                };
                self.push_scope();
                self.bind_fixed(&f.item, item, span, "a loop variable");
                if let Some(index) = &f.index {
                    self.bind_fixed(index, Type::Number, span, "a loop variable");
                }
                if let Some(key) = &f.key {
                    self.infer(key, None);
                }
                self.statements(&f.body, body);
                self.pop_scope();
            }
            StatementKind::Show(s) => {
                let cond = self.infer(&s.condition, Some(&Type::Bool));
                self.condition(&cond, &s.condition, span, "show");
                self.constant_condition(&s.condition);
                self.statements(&s.body, body);
            }
            StatementKind::Match(m) => self.match_statement(m, span, body),
            StatementKind::Assignment(a) => {
                self.check_writable(&a.target, span);
                let target = match &a.target {
                    // A name nothing declares compiles to a signal's
                    // `.set`, `_name.set(…)`, which throws: `T13`.
                    Expr::Identifier(name) => {
                        if let Some(ty) = self.assignable(name) {
                            ty
                        } else {
                            if !self.is_known_name(name) {
                                self.unresolved(name, false);
                            }
                            Type::Any
                        }
                    }
                    other => self.infer(other, None),
                };
                let value = self.infer(&a.value, Some(&target));
                let what = format!("`{}`", expr_text(&a.target));
                self.expect(&value, &target, span, &what);
                // A value the checker can read is held to what the state's
                // type says its values must be, here as at its declaration.
                if let Expr::Identifier(name) = &a.target
                    && let Some(ty) = self.refined.get(name).cloned()
                    && let Some(fault) = refinement_fault(&ty, &a.value)
                {
                    self.error(
                        span,
                        "T01",
                        format!("`{name}` {fault}"),
                        "The type says what its values may be; this one is outside it",
                    );
                }
            }
            StatementKind::MethodCall(mc) => {
                self.infer(
                    &Expr::MethodCall(
                        Box::new(mc.object.clone()),
                        mc.method.clone(),
                        mc.args.clone(),
                    ),
                    None,
                );
            }
            StatementKind::ExprStatement(e) => {
                self.infer(e, None);
            }
            StatementKind::Navigate(e) => {
                let ty = self.infer(e, Some(&Type::String));
                self.expect(&ty, &Type::String, span, "`navigate`'s path");
            }
            StatementKind::Log(e) => {
                let ty = self.infer(e, None);
                if ty.unwrapped() == Type::Scalar(Scalar::Secret) {
                    self.error(
                        span,
                        "T12",
                        format!("`{}` is a `Secret`, and this would log it", expr_text(e)),
                        "A log line is read by whoever can open the console, and often shipped to a service",
                    );
                }
            }
            StatementKind::Return(e) => {
                let ty = match e {
                    Some(e) => self.infer(e, None),
                    None => Type::Null,
                };
                if let Some(returns) = self.returns.last_mut() {
                    returns.push(ty);
                }
            }
            StatementKind::Emit(e) => self.emit(e, span),
            StatementKind::Animate(_) => {}
            StatementKind::Fetch(f) => {
                self.infer(&f.url, Some(&Type::String));
                if let Some(b) = &f.loading_block {
                    self.statements(b, body);
                }
                if let Some((name, b)) = &f.error_block {
                    self.push_scope();
                    self.bind(name, Type::Any, span);
                    self.statements(b, body);
                    self.pop_scope();
                }
                if let Some(b) = &f.success_block {
                    self.push_scope();
                    self.bind(&f.variable, Type::Any, span);
                    self.statements(b, body);
                    self.pop_scope();
                }
            }
        }
    }

    /// A condition: a `Bool`, or anything JavaScript reads as one — a
    /// number, a string, a value that may be null — but not a list or a
    /// record, which are always true.
    pub(super) fn condition(&mut self, ty: &Type, expr: &Expr, span: Span, keyword: &str) {
        match ty {
            Type::List(_) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but a list is always true",
                    expr_text(expr)
                ),
                "Ask about its length: `items.length > 0`",
            ),
            Type::Record(name) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but a `{name}` is always true",
                    expr_text(expr)
                ),
                "Compare one of its fields",
            ),
            Type::Func(..) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but an action is always true",
                    expr_text(expr)
                ),
                "Call it: `{}()`",
            ),
            _ => {}
        }
    }

    pub(super) fn if_statement(&mut self, i: &IfStmt, span: Span, body: Body) {
        self.push_scope();
        let facts = self.non_empty.len();
        if i.binding.is_none() {
            self.non_empty.extend(non_empty_lists(&i.condition));
        }
        if let Some(name) = &i.binding {
            let ty = self.infer(&i.condition, None);
            self.bind_fixed(name, ty.unwrapped(), span, "a value `if let` binds");
        } else {
            let ty = self.infer(&i.condition, Some(&Type::Bool));
            self.condition(&ty, &i.condition, span, "if");
            self.constant_condition(&i.condition);
            // `if x != null { }`, `if x { }`: `x` is not null inside.
            for name in narrowed_names(&i.condition) {
                if let Some(Type::Optional(inner)) = self.lookup(&name) {
                    self.narrow(&name, *inner);
                }
            }
        }
        self.statements(&i.then_body, body);
        self.non_empty.truncate(facts);
        self.pop_scope();
        // A branch after `if x == null { … }` runs only when `x` is not
        // null: `else { x.title }` needs no unwrapping.
        let mut ruled_out: Vec<String> = if i.binding.is_none() {
            null_names(&i.condition)
        } else {
            Vec::new()
        };
        for (cond, branch) in &i.else_if_branches {
            self.push_scope();
            self.narrow_all(&ruled_out);
            let ty = self.infer(cond, Some(&Type::Bool));
            self.condition(&ty, cond, span, "else if");
            for name in narrowed_names(cond) {
                if let Some(Type::Optional(inner)) = self.lookup(&name) {
                    self.narrow(&name, *inner);
                }
            }
            self.statements(branch, body);
            self.pop_scope();
            ruled_out.extend(null_names(cond));
        }
        if let Some(b) = &i.else_body {
            self.push_scope();
            self.narrow_all(&ruled_out);
            self.statements(b, body);
            self.pop_scope();
        }
    }

    /// Narrow each of `names` out of its `null`, in the current scope.
    pub(super) fn narrow_all(&mut self, names: &[String]) {
        for name in names {
            if let Some(Type::Optional(inner)) = self.lookup(name) {
                self.narrow(name, *inner);
            }
        }
    }

    pub(super) fn match_statement(&mut self, m: &MatchStmt, span: Span, body: Body) {
        let scrutinee = self.infer(&m.scrutinee, None);
        let has_else = m.arms.iter().any(|a| matches!(a.pattern, ArmPattern::Else));
        // What the arms cover, and what they cover twice.
        let mut cases: Vec<String> = Vec::new();
        let mut dupes: Vec<String> = Vec::new();
        for arm in &m.arms {
            let key = match &arm.pattern {
                ArmPattern::Case(c) => c.clone(),
                ArmPattern::Loading => "loading".into(),
                ArmPattern::Error => "error".into(),
                ArmPattern::Ready => "ready".into(),
                ArmPattern::State(s) => s.clone(),
                ArmPattern::Else => continue,
            };
            if cases.contains(&key) {
                dupes.push(key);
            } else {
                cases.push(key);
            }
        }
        self.current_span = span;
        match scrutinee.unwrapped() {
            Type::Enum(_) => {
                self.match_coverage(
                    &expr_text(&m.scrutinee),
                    &scrutinee.unwrapped(),
                    &cases,
                    &dupes,
                    has_else,
                    true,
                );
            }
            Type::Resource(_) => {
                for state in &dupes {
                    self.error_at_current(
                        "T15",
                        format!(
                            "`{state}` has two arms in this `match`; the second is never reached"
                        ),
                        "Keep one of them",
                    );
                }
                if !has_else && !cases.iter().any(|c| c == "error") {
                    self.warn(
                        "T21",
                        format!(
                            "this `match` on `{}` has no `error` arm, so a failed request shows nothing",
                            expr_text(&m.scrutinee)
                        ),
                        "Add `error(e) { Alert(e.message).danger }`, or `else { … }`",
                    );
                    self.plan_last_warning(
                        "Add an `error` arm".to_string(),
                        crate::diagnostics::fixes::Plan::AddArms {
                            arms: vec!["error(e) { Alert(e.message).danger }".to_string()],
                        },
                    );
                }
            }
            _ => {}
        }
        match &scrutinee {
            Type::Resource(inner) => {
                for arm in &m.arms {
                    self.push_scope();
                    match (&arm.pattern, &arm.binding) {
                        (ArmPattern::Ready, Some(name)) => {
                            self.bind_fixed(name, (**inner).clone(), arm.span, ARM);
                        }
                        (ArmPattern::Error, Some(name)) => {
                            self.bind_fixed(name, Type::NetError, arm.span, ARM);
                        }
                        (ArmPattern::Case(case), _) => self.error(
                            arm.span,
                            "T11",
                            format!(
                                "`{}` is a resource; `.{case}` is not one of its states",
                                expr_text(&m.scrutinee)
                            ),
                            "A resource matches `loading`, `error(e)`, `ready(value)` and `else`",
                        ),
                        _ => {}
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            Type::Enum(name) => {
                let cases: Vec<String> = self
                    .world
                    .enums
                    .get(name.as_str())
                    .map(|e| e.case_names())
                    .unwrap_or_default();
                for arm in &m.arms {
                    match &arm.pattern {
                        ArmPattern::Case(case) if !cases.contains(case) => {
                            self.error(
                                arm.span,
                                "T02",
                                format!("`{name}` has no case `.{case}`"),
                                &format!(
                                    "`{name}` takes {}",
                                    cases.iter().map(|c| format!(".{c}")).collect::<Vec<_>>().join(", ")
                                ),
                            );
                            self.plan_nearest_case(case, &cases);
                        }
                        // `.failed(r)` binds the payload, one name per part.
                        ArmPattern::Case(case) => {
                            let fields = self.payload_of(name, case).unwrap_or_default();
                            if !arm.bindings.is_empty() && arm.bindings.len() != fields.len() {
                                let hint = if fields.is_empty() {
                                    format!("`.{case}` carries nothing: write `.{case} {{ … }}`")
                                } else {
                                    format!("Bind every part: `{}`", case_signature(case, &fields))
                                };
                                self.error(
                                    arm.span,
                                    "T10",
                                    format!(
                                        "`.{case}` carries {} value{}, but {} {} bound",
                                        fields.len(),
                                        if fields.len() == 1 { "" } else { "s" },
                                        arm.bindings.len(),
                                        if arm.bindings.len() == 1 { "is" } else { "are" }
                                    ),
                                    &hint,
                                );
                            }
                        }
                        ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready => self.error(
                            arm.span,
                            "T11",
                            format!(
                                "`{}` is a `{name}`, not a resource; it has no `loading`, `error` or `ready`",
                                expr_text(&m.scrutinee)
                            ),
                            "Match its cases: `.case { … }`",
                        ),
                        _ => {}
                    }
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind_fixed(name, Type::Any, arm.span, ARM);
                    }
                    if let ArmPattern::Case(case) = &arm.pattern {
                        let fields = self.payload_of(name, case).unwrap_or_default();
                        for (i, bound) in arm.bindings.iter().enumerate() {
                            let ty = fields.get(i).map_or(Type::Any, |(_, t)| t.clone());
                            self.bind_fixed(bound, ty, arm.span, ARM);
                        }
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            Type::Any | Type::Case(_) => {
                for arm in &m.arms {
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind_fixed(name, Type::Any, arm.span, ARM);
                    }
                    for bound in &arm.bindings {
                        self.bind_fixed(bound, Type::Any, arm.span, ARM);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            // What a request can fail with, matched by its case.
            Type::NetError => {
                for arm in &m.arms {
                    self.push_scope();
                    if let ArmPattern::Case(case) = &arm.pattern
                        && !matches!(
                            case.as_str(),
                            "offline" | "timeout" | "aborted" | "parse" | "network" | "status"
                        )
                    {
                        self.error(
                            arm.span,
                            "T02",
                            format!("a request cannot fail with `.{case}`"),
                            "It fails with `.offline`, `.timeout`, `.aborted`, `.parse`, `.network` or `.status(code, body)`",
                        );
                        let ways: Vec<String> = [
                            "offline", "timeout", "aborted", "parse", "network", "status",
                        ]
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect();
                        self.plan_nearest_case(case, &ways);
                    }
                    if let Some(name) = &arm.binding {
                        let bound = match &arm.pattern {
                            ArmPattern::Case(case) if case == "status" => Type::Any,
                            _ => Type::Any,
                        };
                        self.bind_fixed(name, bound, arm.span, ARM);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            other => {
                self.error(
                    span,
                    "T11",
                    format!(
                        "`match` needs a resource or an enum, but `{}` is `{other}`",
                        expr_text(&m.scrutinee)
                    ),
                    "Use `if` for a condition; `match` chooses among a resource's states or an enum's cases",
                );
                for arm in &m.arms {
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind_fixed(name, Type::Any, arm.span, ARM);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
        }
    }

    /// A record built by name, `Todo(id: "1", title: "x")`: each value
    /// against its field's type, a field the record lacks, and a field it
    /// needs that is not given.
    pub(super) fn record_fields_given(
        &mut self,
        record: &str,
        given: &[(&str, &Expr)],
        what: &str,
    ) {
        let Some(all) = self.world.record_fields(record) else {
            return;
        };
        let mut spread = false;
        for (key, value) in given {
            if *key == "..." {
                self.infer(value, None);
                spread = true;
                continue;
            }
            match all.iter().find(|f| f.name == *key) {
                Some(field) => {
                    let wanted = self.world.resolve(Type::from_ref(&field.ty));
                    let ty = self.infer(value, Some(&wanted));
                    if !ty.assignable_to(&wanted) {
                        self.error_at_current(
                            "T01",
                            format!(
                                "`{key}` of `{record}` is `{wanted}`, but `{}` is `{ty}`",
                                expr_text(value)
                            ),
                            "",
                        );
                    }
                }
                None => {
                    self.infer(value, None);
                }
            }
        }
        if !spread {
            let keys: Vec<String> = given.iter().map(|(k, _)| k.to_string()).collect();
            self.record_keys_given(record, &keys, what);
        }
    }

    /// The keys given for a record: one it does not have is `T05`, one it
    /// needs and does not get is `C01`.
    pub(super) fn record_keys_given(&mut self, record: &str, keys: &[String], what: &str) {
        let Some(all) = self.world.record_fields(record) else {
            return;
        };
        let names: Vec<String> = all.iter().map(|f| format!("`{}`", f.name)).collect();
        for key in keys {
            if !all.iter().any(|f| &f.name == key) {
                let known: Vec<String> = all.iter().map(|f| f.name.clone()).collect();
                let known: Vec<&str> = known.iter().map(String::as_str).collect();
                self.error_naming(
                    "T05",
                    format!("`{record}` has no field `{key}`"),
                    &format!("Its fields are {}", names.join(", ")),
                    key,
                    &known,
                    false,
                );
            }
        }
        let missing: Vec<String> = all
            .iter()
            .filter(|f| {
                f.default.is_none()
                    && !matches!(f.ty, TypeRef::Optional(_))
                    && !keys.iter().any(|k| k == &f.name)
            })
            .map(|f| format!("`{}`", f.name))
            .collect();
        if !missing.is_empty() {
            self.error_at_current(
                "C01",
                format!(
                    "{what} leaves out {} of `{record}`, which {} no default",
                    missing.join(", "),
                    if missing.len() == 1 { "has" } else { "have" }
                ),
                "Give it, or declare a default in the type: `field: Type = value`",
            );
        }
    }

    /// An assignment's target must be something that can change: a state,
    /// a `persist`, a `let`, a store's state — not a constant, a derived
    /// value, a prop, a route parameter or a loop variable. At run time an
    /// assignment to one throws, or writes a copy nothing reads.
    pub(super) fn check_writable(&mut self, target: &Expr, span: Span) {
        let (name, what) = match target {
            Expr::Identifier(name) => match self.fixed_kind(name) {
                Some(what) => (name.clone(), what),
                None => return,
            },
            Expr::PropertyAccess(base, member) => match base.as_ref() {
                Expr::Identifier(store)
                    if self
                        .lookup(store)
                        .is_none_or(|t| matches!(t, Type::Store(_))) =>
                {
                    match self
                        .world
                        .store_fixed
                        .get(store.as_str())
                        .and_then(|m| m.get(member))
                    {
                        Some(what) => (format!("{store}.{member}"), *what),
                        None => return,
                    }
                }
                _ => return,
            },
            _ => return,
        };
        let hint = match what {
            "a `const`" | "a `data` constant" | "an `image`" => {
                "Declare it `state` if it changes; a constant is the same on every page"
            }
            "a `derived` value" => {
                "It is worked out from what it reads: assign to that, or make it a `state`"
            }
            "a prop" => {
                "The caller owns it: keep a `state` of your own seeded from it, or `emit` an event so the caller changes it"
            }
            "a route parameter" => "The address owns it: `navigate` to the new address",
            "a loop variable" => {
                "Each pass gets its own copy: change the item through the list it came from"
            }
            "an action" => "Call it: `name()`",
            _ => "Hold what changes in a `state`",
        };
        let located = self.located(span, &format!("`{name}`"));
        self.error(
            located,
            "X01",
            format!("`{name}` is {what}, and nothing may assign to it"),
            hint,
        );
    }

    pub(super) fn emit(&mut self, e: &EmitStmt, span: Span) {
        let Some(component) = self.component else {
            return;
        };
        let Some(event) = component.events.iter().find(|ev| ev.name == e.event) else {
            return; // sema reports the undeclared event
        };
        if event.params.len() != e.args.len() {
            self.error(
                span,
                "T09",
                format!(
                    "`emit {}` passes {} argument{}, but the event takes {}",
                    e.event,
                    e.args.len(),
                    if e.args.len() == 1 { "" } else { "s" },
                    event.params.len()
                ),
                &format!(
                    "`event {}({})`",
                    event.name,
                    event
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, Type::from_ref(&p.param_type)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
        for (arg, param) in e.args.iter().zip(&event.params) {
            let wanted = self.world.resolve(Type::from_ref(&param.param_type));
            let given = self.infer(arg, Some(&wanted));
            self.expect(
                &given,
                &wanted,
                span,
                &format!("`{}` of `emit {}`", param.name, e.event),
            );
        }
    }
}
