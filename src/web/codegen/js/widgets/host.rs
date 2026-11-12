//! Nodes somebody else's code owns: custom elements, portals, hosts, virtual lists and splitters.

use super::super::{
    Arg, DRAG_PROPS, EventHandler, Expr, GESTURE_EVENTS, JsCodegen, RUNTIME_KEYS, StatementKind,
    UIElement, kebab_case,
};

impl JsCodegen {
    /// `Element("stripe-pricing-table", publishable-key: key) { on ready { … } }`:
    /// a custom element some script defines, by its tag. Every named
    /// argument is an attribute (a value that reads state follows it), every
    /// handler a DOM event it fires, the block its children — and `class:`,
    /// `style { }`, motion and a lifetime work as on anything else.
    pub(in super::super) fn emit_custom_element(
        &mut self,
        var: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        let tag = match ui.args.first() {
            Some(Arg::Positional(Expr::StringLiteral(tag))) => tag.clone(),
            _ => "wf-element".to_string(),
        };
        let mut attrs: Vec<String> = Vec::new();
        for arg in &ui.args {
            let Arg::Named(key, value) = arg else {
                continue;
            };
            // What the generic paths below write themselves.
            if matches!(
                key.as_str(),
                "class" | "ref" | "mount" | "update" | "cleanup" | "trapFocus" | "shared" | "count"
            ) || DRAG_PROPS.contains(&key.as_str())
                || key.starts_with("data-wf-")
            {
                continue;
            }
            let attribute = kebab_case(key);
            let emitted = if crate::project::url::URL_ATTRS.contains(&attribute.as_str()) {
                self.url_value(value)
            } else {
                self.emit_expr(value)
            };
            let attribute = if RUNTIME_KEYS.contains(&attribute.as_str()) {
                format!("attr:{attribute}")
            } else {
                attribute
            };
            if self.is_reactive(&emitted) {
                attrs.push(format!("\"{attribute}\": () => {emitted}"));
            } else {
                attrs.push(format!("\"{attribute}\": {emitted}"));
            }
        }
        if let Some(entry) = self.wf_node_entry(ui) {
            attrs.push(entry);
        }
        self.emit_line(&format!(
            "const {var} = WF.el({}, {{ {} }});",
            serde_json::to_string(&tag).unwrap_or_default(),
            attrs.join(", ")
        ));
        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            let line = self.element_listener(var, handler, &body);
            self.emit_line(&line);
        }
        self.emit_style_and_transition(var, ui);
        self.emit_motion(var, ui);
        self.emit_host(var, ui);
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// `mount:`, `update:`, `cleanup:` — on `Host` or any other element: the
    /// node handed to somebody else's code, and given back when what owns
    /// it leaves.
    /// An element's `on …` handler. A gesture the runtime recognises —
    /// `longpress`, `swipe`, a drop's `receive`, a drag's `dragmove` — is
    /// set up by `WF.gesture`, and its handler hears what it saw; every
    /// other event is the element's own.
    pub(in super::super) fn element_listener(
        &self,
        var: &str,
        handler: &EventHandler,
        body: &str,
    ) -> String {
        let head = self.handler_head(handler, "event");
        if GESTURE_EVENTS.contains(&handler.event.as_str()) {
            format!(
                "WF.gesture({var}, \"{}\", {head} => {{ {body} }});",
                handler.event
            )
        } else {
            format!(
                "{var}.addEventListener(\"{}\", {head} => {{ {body} }});",
                handler.event
            )
        }
    }

    /// `Portal(into: node) { … }`: the block, drawn into a box that is put
    /// in a node somebody else's code owns, and follows `into:`.
    ///
    /// Where it is written there is a marker and nothing else, so a branch
    /// or a list item that holds only a portal still has a node of its own.
    /// The box is placed before the block is drawn, so the block is built
    /// into a node already in the page when the target is; and the block is
    /// drawn here, in the scope the portal is written in, so its effects,
    /// handlers, `emit` and cleanups are that scope's. The lifetimes it asks
    /// for (`mount:`) are held until it is whole.
    ///
    /// The block is written at the level the portal is, as any element's
    /// children are, never inside a block of its own: a `derived` value or
    /// an action declared in it is read after it as one declared in a
    /// `Stack` is. A block that throws never reaches the line that lets its
    /// lifetimes go, and the runtime lets them go once the drawing is over.
    pub(in super::super) fn emit_portal(
        &mut self,
        var: &str,
        attrs: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        let into = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, v) if k == "into" => Some(self.emit_expr(v)),
                _ => None,
            })
            .unwrap_or_else(|| "null".to_string());
        self.emit_line(&format!("const {var} = WF.el(\"div\", {attrs});"));
        self.emit_line(&format!(
            "{parent}.appendChild(document.createComment(\"wf-portal\"));"
        ));
        let release = format!("{var}_drawn");
        self.emit_line(&format!(
            "const {release} = WF.portal({var}, () => {into});"
        ));
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }
        self.emit_line(&format!("{release}();"));
        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            let line = self.element_listener(var, handler, &body);
            self.emit_line(&line);
        }
    }

    pub(in super::super) fn emit_host(&mut self, var: &str, ui: &UIElement) {
        // `drag: value` picks the element up; `on receive(d)` (or
        // `dropValue:`) takes what is dropped on it.
        let named = |key: &str| {
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key => Some(v.clone()),
                _ => None,
            })
        };
        let group = named("dragGroup").map(|g| self.emit_expr(&g));
        if let Some(value) = named("drag") {
            let value = self.emit_expr(&value);
            let group = group
                .as_ref()
                .map(|g| format!(", {{ group: {g} }}"))
                .unwrap_or_default();
            self.emit_line(&format!("WF.draggable({var}, () => {value}{group});"));
        }
        let receives = ui.events.iter().any(|h| h.event == "receive");
        if receives || named("dropValue").is_some() {
            let mut opts = Vec::new();
            if let Some(v) = named("dropValue") {
                let v = self.emit_expr(&v);
                opts.push(format!("value: () => {v}"));
            }
            if let Some(Expr::EnumCase(axis) | Expr::StringLiteral(axis)) = named("dropAxis") {
                opts.push(format!("axis: \"{axis}\""));
            }
            if let Some(inside) = named("dropInside") {
                let inside = self.emit_expr(&inside);
                opts.push(format!("inside: {inside}"));
            }
            if let Some(g) = &group {
                opts.push(format!("group: {g}"));
            }
            self.emit_line(&format!("WF.dropTarget({var}, {{ {} }});", opts.join(", ")));
        }
        // `trapFocus: open`: while it holds, Tab stays inside the element.
        if let Some(cond) = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "trapFocus" => Some(v),
            _ => None,
        }) {
            let read = self.emit_expr(cond);
            self.emit_line(&format!("WF.trapFocus({var}, () => {read});"));
        }
        // `mount:`, `update:`, `cleanup:` on any element: the lifetime `Host`
        // gives a bare node, given to every element there is.
        if !ui.args.iter().any(|a| {
            matches!(a, Arg::Named(k, _) if matches!(k.as_str(), "mount" | "update" | "cleanup"))
        }) {
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

    /// `VirtualList(rowHeight:, estimate:, reveal:) { for … by … { } }`: the
    /// box scrolls, and its one keyed `for` is drawn by `WF.windowed` a
    /// window at a time. Anything else in it is drawn whole (V12).
    pub(in super::super) fn emit_virtual_list(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let label = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "label" => Some(v.clone()),
            _ => None,
        });
        let labelled = match label {
            Some(v) => format!(", \"aria-label\": {}", self.emit_expr(&v)),
            None => String::new(),
        };
        // What any element takes — `role: "listbox"`, `tabindex:`,
        // `aria-activedescendant:` — is the box's, with its handlers below.
        let extra = self.root_attrs("VirtualList", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{}{} }});",
            var,
            self.class_attr("VirtualList", ui),
            labelled,
            extra,
            self.wf_node_inline(ui)
        ));
        self.emit_style_and_transition(var, ui);
        match ui.children.as_slice() {
            [only] if matches!(&only.kind, StatementKind::For(f) if f.key.is_some()) => {
                let StatementKind::For(for_stmt) = &only.kind else {
                    unreachable!()
                };
                let mut opts = Vec::new();
                for key in ["rowHeight", "estimate"] {
                    if let Some(v) = ui.args.iter().find_map(|a| match a {
                        Arg::Named(k, v) if k == key => Some(v.clone()),
                        _ => None,
                    }) {
                        let v = self.emit_expr(&v);
                        opts.push(format!("{key}: {v}"));
                    }
                }
                if let Some(v) = ui.args.iter().find_map(|a| match a {
                    Arg::Named(k, v) if k == "reveal" => Some(v.clone()),
                    _ => None,
                }) {
                    let v = self.emit_expr(&v);
                    opts.push(format!("reveal: () => {v}"));
                }
                self.window_opts = Some(opts);
                self.emit_for_dom(for_stmt, var);
            }
            children => {
                for child in children {
                    self.emit_statement_dom(child, var);
                }
            }
        }
        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// `Splitter(bind: width, min:, max:, reset:)`: a handle `WF.splitter`
    /// drives — the pointer, a double-click, the keys — writing the size it
    /// is bound to.
    pub(in super::super) fn emit_splitter(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let label = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "label" => Some(v.clone()),
            _ => None,
        });
        let labelled = match label {
            Some(v) => format!(", \"aria-label\": {}", self.emit_expr(&v)),
            None => String::new(),
        };
        // An author's `role:` and `tabindex:` are kept by `WF.splitter`,
        // which gives the handle its own only where none is written.
        let extra = self.root_attrs("Splitter", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{}{} }});",
            var,
            self.class_attr("Splitter", ui),
            labelled,
            extra,
            self.wf_node_inline(ui)
        ));
        let (read, write) = match self.bind_arg(ui) {
            Some(target) => (
                format!("() => {}", target.read),
                format!("($$v) => {{ {} }}", target.write("$$v")),
            ),
            None => ("() => 0".to_string(), "null".to_string()),
        };
        let mut opts = Vec::new();
        for key in ["min", "max", "reset"] {
            if let Some(v) = ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key => Some(v.clone()),
                _ => None,
            }) {
                let v = self.emit_expr(&v);
                opts.push(format!("{key}: () => {v}"));
            }
        }
        for flag in ["horizontal", "reverse"] {
            if ui.modifiers.iter().any(|m| m == flag) {
                opts.push(format!("{flag}: true"));
            }
        }
        self.emit_line(&format!(
            "WF.splitter({var}, {read}, {write}, {{ {} }});",
            opts.join(", ")
        ));
        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }
}
