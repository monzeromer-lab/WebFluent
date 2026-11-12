//! Control flow in the DOM: `if`, `for`, `show`, `match`, and a statement's place in a body.

use super::super::*;

impl JsCodegen {
    pub(in super::super) fn emit_statement_dom(&mut self, stmt: &Statement, parent: &str) {
        self.map_at(stmt.span);
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
                    "const {} = {};",
                    sig(&d.name),
                    self.computed_init(&d.name, &val)
                ));
            }
            StatementKind::Effect(e) => {
                self.emit_line("WF.effect(() => {");
                self.indent += 1;
                let lets = self.block_lets.len();
                for s in &e.body {
                    self.emit_statement(s);
                }
                self.block_lets.truncate(lets);
                // What the effect returns runs before its next run and when
                // its owner leaves.
                if !e.cleanup.is_empty() {
                    self.emit_line("return () => {");
                    self.indent += 1;
                    for s in &e.cleanup {
                        self.emit_statement(s);
                    }
                    self.block_lets.truncate(lets);
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
                let head = if crate::syntax::parser::ast::awaits(&t.body) {
                    "async () =>"
                } else {
                    "() =>"
                };
                self.emit_line(&format!("WF.{kind}({interval}, {head} {{"));
                self.indent += 1;
                let lets = self.block_lets.len();
                for s in &t.body {
                    self.emit_statement(s);
                }
                self.block_lets.truncate(lets);
                self.indent -= 1;
                self.emit_line("});");
            }
            StatementKind::Action(a) => {
                let params: Vec<String> = a.params.iter().map(|p| p.name.clone()).collect();
                // An action that awaits is an async function, and `name.pending`
                // is true while a call of it runs.
                let is_async = crate::syntax::parser::ast::awaits(&a.body);
                let kind = if is_async {
                    "async function"
                } else {
                    "function"
                };
                self.emit_line(&format!(
                    "{} {}({}) {{",
                    kind,
                    bare(&a.name),
                    bare_list(&params.join(", "))
                ));
                self.indent += 1;
                if is_async {
                    self.emit_line(&format!("{}_pending.set(true);", sig(&a.name)));
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
                let lets = self.block_lets.len();
                for s in &a.body {
                    self.emit_statement(s);
                }
                self.block_lets.truncate(lets);
                self.lambda_params.borrow_mut().truncate(depth);
                if optimistic {
                    self.indent -= 1;
                    self.emit_line("});");
                }
                if is_async {
                    self.indent -= 1;
                    self.emit_line(&format!(
                        "}} finally {{ {}_pending.set(false); }}",
                        sig(&a.name)
                    ));
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
                    "WF.onKey(document, {}, {} => {{ {} }});",
                    js_str(&key),
                    self.handler_head(handler, "event"),
                    body
                ));
            }
            // `on keydown(e) { … }` on the page, a component or the app: the
            // document hears it — the window, for what only the window says
            // — for as long as it is shown. It used to compile to a comment.
            StatementKind::EventHandler(handler) => {
                let body = self.emit_event_body(handler);
                let target = if matches!(
                    handler.event.as_str(),
                    "resize"
                        | "scroll"
                        | "hashchange"
                        | "popstate"
                        | "online"
                        | "offline"
                        | "message"
                        | "storage"
                        | "beforeunload"
                        | "pagehide"
                        | "pageshow"
                        // The window's focus: the document hears neither.
                        | "blur"
                        | "focus"
                ) {
                    "window"
                } else {
                    "document"
                };
                self.emit_line(&format!(
                    "WF.listen({target}, \"{}\", {} => {{ {} }});",
                    handler.event,
                    self.handler_head(handler, "event"),
                    body
                ));
            }
            _ => {
                self.emit_statement(stmt);
            }
        }
    }

    pub(in super::super) fn emit_if_dom(&mut self, if_stmt: &IfStmt, parent: &str) {
        let value = self.emit_expr(&if_stmt.condition);
        // `if let x = e`: the branch shows while the value is not null, and
        // reads it as a plain name.
        let cond = match &if_stmt.binding {
            Some(_) => format!("{value} != null"),
            None => value.clone(),
        };

        self.emit_line(&format!("WF.when({parent},"));
        self.indent += 1;
        self.emit_line(&format!("() => {cond},"));

        // Then branch
        self.emit_line("() => {");
        self.indent += 1;
        let then_var = self.fresh_var();
        self.emit_line(&format!(
            "const {then_var} = document.createDocumentFragment();"
        ));
        // `if let x = v`: the branch reads `x` as `v` is now — a new value
        // reaches what it drew in place — and the branch is drawn again
        // only when `v` goes to null and back.
        let bound = self.loop_bindings.len();
        if let Some(name) = &if_stmt.binding {
            self.emit_line(&format!("const ${name} = WF.held(() => {value});"));
            self.bind_live(name);
        }
        for stmt in &if_stmt.then_body {
            self.emit_statement_dom(stmt, &then_var);
        }
        self.unbind(bound);
        self.emit_line(&format!("return {then_var};"));
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
                "const {elif_var} = document.createDocumentFragment();"
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
            self.emit_line(&format!("return {elif_var};"));
            self.indent -= 1;
            self.emit_line("},");
        } else if let Some(else_body) = &if_stmt.else_body {
            self.emit_line("() => {");
            self.indent += 1;
            let else_var = self.fresh_var();
            self.emit_line(&format!(
                "const {else_var} = document.createDocumentFragment();"
            ));
            for stmt in else_body {
                self.emit_statement_dom(stmt, &else_var);
            }
            self.emit_line(&format!("return {else_var};"));
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

    pub(in super::super) fn emit_for_dom(&mut self, for_stmt: &ForStmt, parent: &str) {
        let list = self.emit_expr(&for_stmt.iterable);
        // A `VirtualList`'s `for` is drawn a window at a time.
        let window = self.window_opts.take();
        let draw = if window.is_some() { "windowed" } else { "each" };

        self.emit_line(&format!("WF.{draw}({parent},"));
        self.indent += 1;
        self.emit_line(&format!("() => {list},"));

        // A keyed loop's item keeps its nodes for as long as its key is in
        // the list: its body is handed the item and the index as accessors
        // too, and reads them through those, so a changed row or a moved
        // one is told so in place. An unkeyed loop is drawn again whole, and
        // its item and index are plain parameters. Pushed as a stack so a
        // nested loop's binding does not erase the outer one's.
        let keyed = for_stmt.key.is_some();
        let bound = self.loop_bindings.len();
        if keyed {
            let index_of = match &for_stmt.index {
                Some(idx) => format!(", ${idx}"),
                None => String::new(),
            };
            self.emit_line(&format!(
                "($$item, $$index, ${}{index_of}) => {{",
                for_stmt.item
            ));
            self.bind_live(&for_stmt.item);
            if let Some(idx) = &for_stmt.index {
                self.bind_live(idx);
            }
        } else {
            let index_param = if let Some(idx) = &for_stmt.index {
                format!(", {}", bare(idx))
            } else {
                ", _idx".to_string()
            };
            self.emit_line(&format!("({}{}) => {{", bare(&for_stmt.item), index_param));
            self.loop_bindings.push(for_stmt.item.clone());
            if let Some(idx) = &for_stmt.index {
                self.loop_bindings.push(idx.clone());
            }
        }
        self.indent += 1;
        self.loop_sources.push((
            for_stmt.item.clone(),
            for_stmt.iterable.clone(),
            for_stmt.key.is_some(),
        ));

        let item_var = self.fresh_var();
        self.emit_line(&format!(
            "const {item_var} = document.createDocumentFragment();"
        ));
        for stmt in &for_stmt.body {
            self.emit_statement_dom(stmt, &item_var);
        }
        self.emit_line(&format!("return {item_var};"));

        self.loop_sources.pop();
        self.unbind(bound);
        self.indent -= 1;
        self.emit_line("},");

        // The options: the animation, `by key` — the identity of an item
        // across renders — and whether the body reads its index, in which
        // case an item is rebuilt when its position changes.
        let mut parts = Self::animate_config_parts(&for_stmt.animate);
        if let Some(key) = &for_stmt.key {
            // The runtime calls the key with the item and its index, so a key
            // may read either. It was given the item alone, and `by n` read
            // a `_n()` nothing declared: the page threw before the list drew.
            let bound = self.loop_bindings.len();
            self.loop_bindings.push(for_stmt.item.clone());
            let params = match &for_stmt.index {
                Some(index) => {
                    self.loop_bindings.push(index.clone());
                    format!("{}, {}", bare(&for_stmt.item), bare(index))
                }
                None => bare(&for_stmt.item),
            };
            let key_js = self.emit_expr(key);
            self.loop_bindings.truncate(bound);
            parts.push(format!("key: ({params}) => {key_js}"));
            if for_stmt.index.is_some() {
                parts.push("index: true".to_string());
            }
            parts.push("live: true".to_string());
        }
        parts.extend(window.unwrap_or_default());
        if parts.is_empty() {
            self.emit_line("null");
        } else {
            self.emit_line(&format!("{{ {} }}", parts.join(", ")));
        }

        self.indent -= 1;
        self.emit_line(");");
    }

    pub(in super::super) fn emit_show_dom(&mut self, show_stmt: &ShowStmt, parent: &str) {
        let cond = self.emit_expr(&show_stmt.condition);

        self.emit_line(&format!("WF.show({parent},"));
        self.indent += 1;
        self.emit_line(&format!("() => {cond},"));
        self.emit_line("() => {");
        self.indent += 1;
        let content_var = self.fresh_var();
        self.emit_line(&format!(
            "const {content_var} = document.createDocumentFragment();"
        ));
        for stmt in &show_stmt.body {
            self.emit_statement_dom(stmt, &content_var);
        }
        self.emit_line(&format!("return {content_var};"));
        self.indent -= 1;
        self.emit_line("},");

        // Animation config (4th argument)
        self.emit_animate_config(&show_stmt.animate);

        self.indent -= 1;
        self.emit_line(");");
    }

    pub(in super::super) fn emit_animate_config(&mut self, config: &Option<AnimateConfig>) {
        let parts = Self::animate_config_parts(config);
        if parts.is_empty() {
            self.emit_line("null");
        } else {
            self.emit_line(&format!("{{ {} }}", parts.join(", ")));
        }
    }

    /// The fields of an animation config object, none for no animation.
    pub(in super::super) fn animate_config_parts(config: &Option<AnimateConfig>) -> Vec<String> {
        let mut parts = Vec::new();
        if let Some(anim) = config {
            parts.push(format!("enter: {}", js_str(&anim.enter)));
            if let Some(exit) = &anim.exit {
                parts.push(format!("exit: {}", js_str(exit)));
            }
            if let Some(dur) = &anim.duration {
                parts.push(format!("duration: {}", js_str(dur)));
            }
            if let Some(delay) = &anim.delay {
                parts.push(format!("delay: {}", js_str(delay)));
            }
            if let Some(stagger) = &anim.stagger {
                parts.push(format!("stagger: {}", js_str(stagger)));
            }
            if let Some(easing) = &anim.easing {
                parts.push(format!("easing: {}", js_str(easing)));
            }
        }
        parts
    }

    /// `match x { … }`: one arm shown at a time, chosen by a resource's
    /// state or an enum's case, and re-chosen when it changes.
    pub(in super::super) fn emit_match_dom(&mut self, m: &MatchStmt, parent: &str) {
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
                format!("() => {subject}.state()"),
                format!(
                    "() => {subject}.state() === \"error\" ? {subject}.error() : {subject}.data()"
                ),
            )
        } else {
            // An enum: the arm is chosen by the case, and handed the payload.
            (
                format!("() => WF.caseOf({subject})"),
                format!("() => {subject}"),
            )
        };
        self.emit_line(&format!("WF.match({parent}, {key}, {arg}, {{"));
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
            // The arm is handed the value and an accessor of it, and reads
            // its binding through the accessor: what it drew follows a
            // refresh of the data, or a new payload of the same case, in
            // place. So does each name a `.case(a, b)` arm binds to a part
            // of the payload.
            let bound = self.loop_bindings.len();
            let live = match &arm.binding {
                Some(binding) => {
                    self.bind_live(binding);
                    format!("${binding}")
                }
                None => "$$arm".to_string(),
            };
            self.emit_line(&format!("{name}: ($$value, {live}) => {{"));
            self.indent += 1;
            for (i, bound_name) in arm.bindings.iter().enumerate() {
                self.emit_line(&format!("const ${bound_name} = () => {live}()[{}];", i + 1));
                self.bind_live(bound_name);
            }
            let var = self.fresh_var();
            self.emit_line(&format!("const {var} = document.createDocumentFragment();"));
            for stmt in &arm.body {
                self.emit_statement_dom(stmt, &var);
            }
            self.emit_line(&format!("return {var};"));
            self.unbind(bound);
            self.indent -= 1;
            self.emit_line("},");
        }
        self.indent -= 1;
        self.emit_line("});");
    }
}
