//! One element: the generic pass over a built-in or a component call, its attributes, styles, motion and root handlers.

use super::super::{
    Arg, ComponentRef, DRAG_PROPS, EventHandler, Expr, GESTURE_EVENTS, JsCodegen, Statement,
    StatementKind, StringPart, UIElement, active_link, animation_of, attr_key, bare,
    builtin_to_html, camel_to_kebab, element_tag, frame_origin, has_subcomponent, implicit_role,
    input_type, is_root_attribute, is_void, js_component, js_str, landmark_label,
    layout_arg_classes, modifier_to_class, sig, template_text,
};

impl JsCodegen {
    pub(in super::super) fn emit_ui_element(&mut self, ui: &UIElement, parent: &str) {
        if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Frame")
            && let Some(framed) = crate::project::builtin::sandboxed_frame(ui)
        {
            return self.emit_ui_element(&framed, parent);
        }
        let var = self.fresh_var();

        match &ui.component {
            ComponentRef::BuiltIn(name) => self.emit_builtin(name, &var, ui, parent),

            ComponentRef::SubComponent(parent_name, sub_name)
                if matches!(parent_name.as_str(), "Menu" | "Dropdown")
                    && sub_name == "Item"
                    && !ui
                        .args
                        .iter()
                        .any(|a| matches!(a, Arg::Named(k, _) if k == "to")) =>
            {
                self.emit_menu_item(parent_name, &var, ui, parent);
            }
            ComponentRef::SubComponent(parent_name, sub_name)
                if parent_name == "Menu" && sub_name == "Sub" =>
            {
                self.emit_menu_sub(parent_name, &var, ui, parent);
            }
            ComponentRef::SubComponent(parent_name, sub_name) => {
                self.emit_part(parent_name, sub_name, &var, ui, parent);
            }
            ComponentRef::UserDefined(name) => self.emit_component_call(name, &var, ui, parent),
        }
    }

    /// A built-in: what only it does, or the generic pass over its
    /// arguments, then its children, handlers and motion.
    fn emit_builtin(&mut self, name: &str, var: &str, ui: &UIElement, parent: &str) {
        match name {
            "Children" => {
                let slot = ui.slot_name().unwrap_or("children");
                // A scoped slot is handed its values, and rendered
                // again when a value it reads changes.
                let handed: Vec<String> = ui
                    .args
                    .iter()
                    .filter_map(|a| match a {
                        Arg::Named(k, v) if k != "slot" => {
                            Some(format!("{k}: {}", self.emit_expr(v)))
                        }
                        _ => None,
                    })
                    .collect();
                if handed.is_empty() {
                    self.emit_line(&format!(
                        "if (typeof _slots.{slot} === 'function') {parent}.appendChild(_slots.{slot}());"
                    ));
                } else {
                    self.emit_line(&format!(
                                "WF.slot({parent}, () => ({{ {} }}), (_s) => typeof _slots.{slot} === 'function' ? _slots.{slot}(_s) : null);",
                                handed.join(", ")
                            ));
                }
                return;
            }
            "_StyleBlock" => return, // Style blocks handled via attrs
            "Element" => {
                self.emit_custom_element(var, ui, parent);
                return;
            }
            // A chart or a QR code is drawn when the page is built,
            // from values known then, and inserted as its SVG.
            "Chart" | "QrCode" => {
                if let Some(svg) = self.static_graphic(name, ui) {
                    let (_, class) = builtin_to_html(name);
                    let svg = serde_json::to_string(&svg).unwrap_or_default();
                    // The figure takes what any element takes: an
                    // `aria-label:` naming the chart, a `role:`, its
                    // handlers, `class:` and a `style { }` block.
                    let extra = self.root_attrs(name, ui, &[]);
                    self.emit_line(&format!("const {var} = WF.el(\"figure\", {{ className: \"{class}\", html: {svg}{extra} }});"));
                    self.emit_root_handlers(var, ui, false);
                    self.emit_style_and_transition(var, ui);
                    self.emit_motion(var, ui);
                    self.emit_host(var, ui);
                    self.emit_line(&format!("{parent}.appendChild({var});"));
                    return;
                }
            }
            _ => {}
        }

        let ElementParts {
            tag,
            attrs,
            attrs_str,
            inner_text,
            link_to,
            link_prefix,
            is_field,
        } = self.element_parts(name, ui);

        // Components with a structure too complex for the generic path
        // build their own subtree and append it themselves. They still
        // have to honour the author's `style { }` and `transition { }`
        // blocks, so they fall through to the shared tail below rather
        // than returning — applying those blocks only on the generic
        // path is what used to drop styling on nineteen components.
        let built_by_special_emitter = match name {
            // A video's captions are a `<track>` inside it, and a
            // player's transcript a link beneath it.
            "Video" | "Audio"
                if ui.args.iter().any(
                    |a| matches!(a, Arg::Named(k, _) if k == "captions" || k == "transcript"),
                ) =>
            {
                self.emit_media(name, var, &attrs_str, ui, parent);
                true
            }
            // An `image` the program declares is a `<picture>`: one
            // `<source>` per format, the widths the build wrote, and
            // the box already the right size.
            "Image" if ui.args.iter().any(|a| matches!(a, Arg::Positional(_))) => {
                self.emit_picture(var, ui, parent);
                true
            }
            "Modal" | "Dialog" | "AlertDialog" | "Sheet" => {
                self.emit_modal_dialog(name, var, &attrs_str, ui, parent);
                true
            }
            "Splitter" => {
                self.emit_splitter(var, ui, parent);
                true
            }
            "VirtualList" => {
                self.emit_virtual_list(var, ui, parent);
                true
            }
            "Portal" => {
                self.emit_portal(var, &attrs_str, ui, parent);
                true
            }
            "Popover" => {
                self.emit_popover(var, ui, parent);
                true
            }
            "Calendar" => {
                self.emit_calendar(var, ui, parent);
                true
            }
            "Tabs" => {
                self.emit_tabs(var, ui, parent);
                true
            }
            "Switch" => {
                self.emit_switch(var, &attrs, ui, parent);
                true
            }
            "Checkbox" | "Radio" => {
                self.emit_check_radio(name, var, &attrs, ui, parent);
                true
            }
            "Dropdown" | "Menu" => {
                self.emit_dropdown_menu(name, var, &attrs, ui, parent);
                true
            }
            "Toast" => {
                // Imperative, not DOM-based: there is no element to style.
                if let Some(text) = &inner_text {
                    let variant = ui
                        .modifiers
                        .first()
                        .map_or("info", std::string::String::as_str);
                    self.emit_line(&format!("WF.toast({text}, \"{variant}\");"));
                }
                return;
            }
            "Spacer" => {
                self.emit_line(&format!("const {var} = WF.el(\"{tag}\", {attrs_str});"));
                self.emit_root_handlers(var, ui, false);
                self.emit_line(&format!("{parent}.appendChild({var});"));
                true
            }
            "Sidebar" => {
                self.emit_sidebar(var, ui, parent);
                true
            }
            "Breadcrumb" => {
                self.emit_breadcrumb(var, ui, parent);
                true
            }
            "Tooltip" => {
                self.emit_tooltip(var, ui, parent);
                true
            }
            "Avatar" => {
                self.emit_avatar(var, ui, parent);
                true
            }
            "Skeleton" => {
                self.emit_skeleton(var, ui, parent);
                true
            }
            "Carousel" => {
                self.emit_carousel(var, ui, parent);
                true
            }
            "IconButton" => {
                self.emit_icon_button(var, ui, parent);
                true
            }
            "Slider" => {
                self.emit_slider(var, ui, parent);
                true
            }
            "DatePicker" => {
                self.emit_datepicker(var, ui, parent);
                true
            }
            "FileUpload" => {
                self.emit_file_upload(var, ui, parent);
                true
            }
            _ => false,
        };

        if built_by_special_emitter {
            self.emit_style_and_transition(var, ui);
            self.emit_motion(var, ui);
            self.emit_host(var, ui);
            return;
        }

        // Standard element creation
        let mut children_arr = Vec::new();

        // A table's caption is its accessible name. It is rendered
        // visually hidden: the heading above the table already says
        // what it is to a sighted reader.
        if name == "Table"
            && let Some(Arg::Named(_, cap)) = ui
                .args
                .iter()
                .find(|a| matches!(a, Arg::Named(k, _) if k == "caption"))
        {
            let v = self.emit_expr(cap);
            children_arr.push(format!(
                "WF.el(\"caption\", {{ className: \"wf-visually-hidden\" }}, {})",
                self.live(&v)
            ));
        }

        // Inner text content — unless a `Code` with a `language:`
        // took it into its `highlight` attribute above.
        if let Some(text) = &inner_text {
            if self.is_reactive(text) {
                children_arr.push(format!("() => {text}"));
            } else {
                children_arr.push(text.clone());
            }
        }

        // A select's value only takes once its options exist, so a
        // slot inside a `Select` is passed to `h()` with the element
        // rather than appended afterwards.
        let slot_in_select = name == "Select"
            && ui.children.iter().any(|c| {
                matches!(&c.kind, StatementKind::UIElement(u)
                            if matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Children"))
            });
        if slot_in_select {
            children_arr.push(
                "(typeof _slots.children === 'function' ? _slots.children() : null)".to_string(),
            );
        }

        if children_arr.is_empty() && ui.children.is_empty() {
            self.emit_line(&format!("const {var} = WF.el(\"{tag}\", {attrs_str});"));
        } else if !children_arr.is_empty() && ui.children.is_empty() {
            self.emit_line(&format!(
                "const {} = WF.el(\"{}\", {}, {});",
                var,
                tag,
                attrs_str,
                children_arr.join(", ")
            ));
        } else {
            let extra = if children_arr.is_empty() {
                String::new()
            } else {
                format!(", {}", children_arr.join(", "))
            };
            self.emit_line(&format!(
                "const {var} = WF.el(\"{tag}\", {attrs_str}{extra});"
            ));
        }

        // Emit children
        {
            let was_in_thead = self.in_thead;
            if name == "Thead" {
                self.in_thead = true;
            } else if name == "Tbody" {
                self.in_thead = false;
            }
            for child in &ui.children {
                if slot_in_select
                    && matches!(&child.kind, StatementKind::UIElement(u)
                                if matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Children"))
                {
                    continue;
                }
                self.emit_statement_dom(child, var);
            }
            self.in_thead = was_in_thead;
        }

        // A `Navbar.Links` group collapses behind a toggle on a narrow
        // screen. The rule that used to apply there was `display: none`
        // with nothing to bring it back, so the links were simply gone.
        // Navbars whose links are direct children wrap instead and need
        // no control, so none is emitted for them.
        if name == "Navbar" && has_subcomponent(ui, "Navbar", "Links") {
            let toggle_var = self.fresh_var();
            self.emit_line(&format!(
                        "const {toggle_var} = WF.el(\"button\", {{ className: \"wf-navbar__toggle\",                          type: \"button\", \"aria-label\": \"Menu\", \"aria-expanded\": \"false\" }}, \"\\u2630\");"
                    ));
            self.emit_line(&format!("{var}.appendChild({toggle_var});"));
            self.emit_line(&format!("WF.drawer({var}, {toggle_var}, null);"));
        }

        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            // A form asks the rules before it does anything: a
            // submit with a fault shows every message, moves focus
            // to the first field that has one, and stops there.
            let body = if name == "Form"
                && handler.event == "submit"
                && let Some(form) = &self.body.current_form
            {
                format!("if (!{form}.__submitting()) return; {body}")
            } else {
                body
            };
            // What the framed page posts arrives on this window, with
            // every other page's: a frame hears its own, and stops
            // listening when it leaves.
            if name == "Frame" && handler.event == "message" {
                let head = self.handler_head(handler, "event");
                let param = match handler.param() {
                    Some(p) => bare(p),
                    None => self.implicit_param("event"),
                };
                // From the frame's own window, and from the origin
                // that frame has: its address's, the page's own for
                // an unsandboxed `srcdoc`, and the opaque `"null"`
                // for a sandbox without `allow-same-origin`. A page
                // the frame navigated to is no longer the one this
                // listens for.
                self.emit_line(&format!(
                            "WF.listen(window, \"message\", {head} => {{ if ({param}.source !== {var}.contentWindow || {param}.origin !== {}) return; {body} }});",
                            frame_origin(var)
                        ));
                continue;
            }
            let line = if (name == "Button" && handler.event == "click")
                || (name == "Form" && handler.event == "submit")
            {
                self.busy_listener(var, ui, handler, &body)
                    .unwrap_or_else(|| self.element_listener(var, handler, &body))
            } else {
                self.element_listener(var, handler, &body)
            };
            self.emit_line(&line);
        }

        if let Some(href) = &link_to
            && name == "Link"
        {
            let own = self.own_aria_current(ui);
            self.emit_line(&active_link(var, href, link_prefix, own));
        }

        self.emit_style_and_transition(var, ui);
        self.emit_motion(var, ui);
        self.emit_host(var, ui);

        if is_field {
            let mut opts = Vec::new();
            for key in ["label", "hint", "error"] {
                if let Some(Arg::Named(_, val)) = ui
                    .args
                    .iter()
                    .find(|a| matches!(a, Arg::Named(k, _) if k == key))
                {
                    let v = self.emit_expr(val);
                    if self.is_reactive(&v) {
                        opts.push(format!("{key}: () => {v}"));
                    } else {
                        opts.push(format!("{key}: {v}"));
                    }
                }
            }
            // A control bound to a state a `validate` block guards
            // shows what the rules say, and marks itself touched
            // when the reader leaves it — so a validated field
            // needs no `error:` written on it.
            if !opts.iter().any(|o| o.starts_with("error:"))
                && let Some(Arg::Named(_, Expr::Identifier(bound))) = ui
                    .args
                    .iter()
                    .find(|a| matches!(a, Arg::Named(k, _) if k == "bind"))
                && self.body.validated.contains(bound)
            {
                opts.push(format!("error: () => {}_check.shown()", sig(bound)));
                self.emit_line(&format!(
                    "WF.listen({var}, \"blur\", () => {}_check.touched.set(true));",
                    sig(bound)
                ));
            }
            self.emit_line(&format!(
                "{}.appendChild(WF.field({}, {{ {} }}));",
                parent,
                var,
                opts.join(", ")
            ));
        } else {
            self.emit_line(&format!("{parent}.appendChild({var});"));
        }
    }

    /// A part of a built-in (`Card.Header`, `Navbar.Links`) that needs no
    /// emitter of its own.
    fn emit_part(
        &mut self,
        parent_name: &str,
        sub_name: &str,
        var: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        // A part the registry says is drawn as a tag and a class of its own
        // (`Callout.Title`), and whose positional is its text.
        let declared = crate::registry::declared_part(parent_name, sub_name);
        let class = match declared {
            Some(sig) => sig.class.to_string(),
            None => format!(
                "wf-{}__{}",
                parent_name.to_lowercase(),
                camel_to_kebab(sub_name)
            ),
        };
        // A part with a `to:` is a link — a `Sidebar.Item` or a
        // `Breadcrumb.Item` reached through a `for` or an `if`, which
        // the owner's own emitter does not see. It used to become a
        // `<li to="…">` that went nowhere.
        let to = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "to" => Some(self.emit_expr(v)),
            _ => None,
        });
        let tag = match (parent_name, sub_name, &to) {
            (_, _, Some(_)) => "a",
            _ if declared.is_some() => declared.map_or("div", |sig| sig.tag),
            ("Breadcrumb", "Item", None) => "span",
            (_, "Item", None) => "li",
            _ => "div",
        };

        // Hyphenated and unknown named arguments are attributes here
        // too (`Card.Header(id: …)`, `List.Item(aria-current: …)`).
        let mut attrs = vec![format!("className: \"{}\"", class)];
        if let Some(href) = &to {
            let bp = if self.base_path.is_empty() {
                String::new()
            } else {
                "WF._basePath + ".to_string()
            };
            let href_attr = if self.is_reactive(href) {
                format!("() => {bp}{href}")
            } else {
                format!("{bp}{href}")
            };
            attrs.push(format!("href: {href_attr}"));
            if !self.ssg_mode {
                attrs.push(format!(
                    "\"on:click\": ($$e) => {{ $$e.preventDefault(); WF.navigate({href}); }}"
                ));
            }
        }
        for arg in &ui.args {
            if let Arg::Named(k, v) = arg {
                if matches!(k.as_str(), "to" | "active") {
                    continue;
                }
                // What the shared tail below draws — a lifetime, a drag,
                // motion — is not an attribute: `mount:` written as one was
                // called with nothing, and threw.
                if matches!(
                    k.as_str(),
                    "mount" | "update" | "cleanup" | "trapFocus" | "roving"
                ) || DRAG_PROPS.contains(&k.as_str())
                    || crate::registry::UNIVERSAL_PROPS
                        .iter()
                        .any(|p| p.name == k && p.sink == crate::registry::Sink::Special)
                {
                    continue;
                }
                let value = self.emit_expr(v);
                let key = attr_key(k);
                if self.is_reactive(&value) {
                    attrs.push(format!("{key}: () => {value}"));
                } else {
                    attrs.push(format!("{key}: {value}"));
                }
            }
        }
        if let Some(entry) = self.wf_node_entry(ui) {
            attrs.push(entry);
        }
        // Its words come after its block, so an icon in the block comes
        // first: `Callout.Title` is the icon, then the title.
        let text = if declared.is_some() {
            ui.args.iter().find_map(|a| match a {
                Arg::Positional(e) => {
                    let v = self.emit_expr(e);
                    Some(self.live(&v))
                }
                Arg::Named(..) => None,
            })
        } else {
            None
        };
        self.emit_line(&format!(
            "const {} = WF.el(\"{}\", {{ {} }});",
            var,
            tag,
            attrs.join(", ")
        ));
        if let Some(href) = &to {
            let prefix = ui.args.iter().any(|a| {
                        matches!(a, Arg::Named(k, Expr::StringLiteral(v)) if k == "active" && v == "prefix")
                    });
            let own = self.own_aria_current(ui);
            self.emit_line(&active_link(var, href, prefix, own));
        }
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }
        if let Some(text) = text {
            self.emit_line(&format!("{var}.appendChild(WF.text({text}));"));
        }
        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            let line = self.element_listener(var, handler, &body);
            self.emit_line(&line);
        }
        // A sub-component used to drop its style block and its
        // handlers; every other element honours both.
        self.emit_style_and_transition(var, ui);
        self.emit_motion(var, ui);
        self.emit_host(var, ui);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// A call of a component the program declares: its props, its slots,
    /// the handlers of the events it declares, and what the call puts on
    /// its root.
    fn emit_component_call(&mut self, name: &str, var: &str, ui: &UIElement, parent: &str) {
        // A handler for an event the component declares is passed in
        // as `on: { name: fn }`; a DOM event's handler attaches to the
        // component's root element, so a styled button component is
        // clickable where it is used.
        let declared = self.component_events.get(name).cloned().unwrap_or_default();
        let (emitted, attached): (Vec<&EventHandler>, Vec<&EventHandler>) =
            ui.events.iter().partition(|h| declared.contains(&h.event));
        // `mount:`, `update:`, `cleanup:` a component does not take
        // itself are a lifetime for its root element, like a DOM
        // event's handler.
        let own = self.component_props.get(name).cloned().unwrap_or_default();
        let (lifetime, passed): (Vec<Arg>, Vec<Arg>) =
                    ui.args.iter().cloned().partition(|a| {
                        matches!(a, Arg::Named(k, _)
                        if (matches!(k.as_str(), "mount" | "update" | "cleanup" | "trapFocus" | "roving") || DRAG_PROPS.contains(&k.as_str())) && !own.contains(k))
                    });
        // An attribute the component does not take as a prop —
        // `aria-label`, `data-*`, `id`, `hidden`, `title` — is its
        // root element's, as it is a built-in's.
        let (root_attrs, passed): (Vec<Arg>, Vec<Arg>) = passed.into_iter().partition(|a| {
            matches!(a, Arg::Named(k, _)
                            if !own.contains(k) && (is_root_attribute(k) || k == "class"))
        });
        let mut args_obj = self.emit_component_args(name, &passed);
        if !emitted.is_empty() {
            let handlers: Vec<String> = emitted
                .iter()
                .map(|h| {
                    let body = self.emit_event_body(h);
                    format!(
                        "{}: {} => {{ {} }}",
                        h.event,
                        self.handler_head(h, "_ev"),
                        body
                    )
                })
                .collect();
            let on = format!("on: {{ {} }}", handlers.join(", "));
            args_obj = if args_obj == "{}" {
                format!("{{ {on} }}")
            } else {
                format!("{}, {} }}", &args_obj[..args_obj.len() - 2], on)
            };
        }
        let has_default_slot = !ui.children.is_empty();
        if !has_default_slot && ui.slot_fills.is_empty() {
            self.emit_line(&format!(
                "const {} = Component_{}({});",
                var,
                js_component(name),
                args_obj
            ));
        } else {
            // Each fill is compiled here, in the caller's scope, so it
            // reads the caller's state and loop bindings; the component
            // only decides where it lands.
            self.emit_line(&format!(
                "const {} = Component_{}({}, {{",
                var,
                js_component(name),
                args_obj
            ));
            self.indent += 1;
            let mut fills: Vec<(&str, &[String], &[Statement])> = Vec::new();
            if has_default_slot {
                fills.push(("children", &[], &ui.children));
            }
            for fill in &ui.slot_fills {
                fills.push((fill.name.as_str(), &fill.params, &fill.body));
            }
            let handed = self
                .component_slot_params
                .get(name)
                .cloned()
                .unwrap_or_default();
            for (slot, params, body) in fills {
                if params.is_empty() {
                    self.emit_line(&format!("{slot}: () => {{"));
                } else {
                    self.emit_line(&format!("{slot}: (_s) => {{"));
                }
                self.indent += 1;
                // A fill's names stand for what the slot hands over,
                // in the slot's order, as plain values.
                let declared = handed.get(slot).cloned().unwrap_or_default();
                for (i, param) in params.iter().enumerate() {
                    let key = declared.get(i).cloned().unwrap_or_else(|| param.clone());
                    self.emit_line(&format!("const {} = _s.{key};", bare(param)));
                    self.loop_bindings.push(param.clone());
                }
                self.emit_line("const _cf = document.createDocumentFragment();");
                for child in body {
                    self.emit_statement_dom(child, "_cf");
                }
                self.emit_line("return _cf;");
                for _ in params {
                    self.loop_bindings.pop();
                }
                self.indent -= 1;
                self.emit_line("},");
            }
            self.indent -= 1;
            self.emit_line("});");
        }
        // A gesture — `on receive(d)`, `on dragmove(d)`, a long press,
        // a swipe — is recognised on the root by the runtime; any
        // other event is the root's own.
        for handler in attached {
            let body = self.emit_event_body(handler);
            if GESTURE_EVENTS.contains(&handler.event.as_str()) {
                let line = self.element_listener(var, handler, &body);
                self.emit_line(&line);
                continue;
            }
            self.emit_line(&format!(
                "WF.onRoot({}, \"{}\", {} => {{ {} }});",
                var,
                handler.event,
                self.handler_head(handler, "event"),
                body
            ));
        }
        if !lifetime.is_empty() {
            let mut root = ui.clone();
            root.args = lifetime;
            self.emit_host(var, &root);
        }
        // An editor's preview: the instance's root says which call
        // made it, innermost first, so a tap inside it can select
        // the call on the page rather than the component's insides.
        if self.studio
            && let Some(id) = self
                .node_ids
                .id_in(&self.node_owner, ui.span)
                .map(str::to_string)
        {
            self.emit_line(&format!(
                        "(_n => {{ const r = _n && _n.nodeType === 11 ? _n.firstElementChild : _n; if (r && r.setAttribute) r.setAttribute(\"data-wf-call\", ((r.getAttribute(\"data-wf-call\") || \"\") + \" {id}\").trim()); }})({var});"
                    ));
        }
        // The motion asked of it here: markers on its root element.
        let marks: Vec<String> = ui
            .args
            .iter()
            .filter_map(|a| match a {
                Arg::Named(k, v) if k.starts_with("data-wf-") => {
                    Some(format!("\"{}\": {}", k, self.emit_expr(v)))
                }
                _ => None,
            })
            .collect();
        if !marks.is_empty() {
            self.emit_line(&format!("WF.mark({}, {{ {} }});", var, marks.join(", ")));
        }
        // A `style { }` written on the call styles the component's
        // root, as `class:` does: its rule's class joins the root's.
        if let Some(class) = ui
            .style_block
            .as_ref()
            .and_then(crate::project::scoped_css::scoped_class)
        {
            self.emit_line(&format!(
                "WF.rootAttrs({var}, {{ \"class\": \"{class}\" }});"
            ));
        }
        if !root_attrs.is_empty() {
            let attrs: Vec<String> = root_attrs
                .iter()
                .filter_map(|a| match a {
                    Arg::Named(k, v) => {
                        let value = self.emit_expr(v);
                        Some(if self.is_reactive(&value) {
                            format!("\"{k}\": () => {value}")
                        } else {
                            format!("\"{k}\": {value}")
                        })
                    }
                    Arg::Positional(_) => None,
                })
                .collect();
            self.emit_line(&format!(
                "WF.rootAttrs({}, {{ {} }});",
                var,
                attrs.join(", ")
            ));
        }
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// The pass over a built-in's named arguments: each becomes an
    /// attribute, a class, a link, or the element's text, in `pass`.
    fn named_args(&mut self, ui: &UIElement, pass: &mut ArgPass<'_>) {
        let ArgPass {
            name,
            class,
            counts,
            is_field,
            attrs,
            classes,
            link_to,
            link_prefix,
            inner_text,
        } = pass;
        let (name, class, counts, is_field) = (*name, *class, *counts, *is_field);
        for arg in &ui.args {
            match arg {
                // A value per breakpoint is carried by its class
                // (`responsive_classes`, below), not an attribute.
                Arg::Named(key, val)
                    if key != "class" && crate::project::scoped_css::is_responsive(val) => {}
                // A class prop that reads state (`tone: tone`) is a class
                // that follows it, set in `emit_style_and_transition`.
                Arg::Named(key, val) if self.state_class(name, class, key, val).is_some() => {}
                Arg::Named(key, val) => {
                    match key.as_str() {
                        // The handle takes the element once drawn.
                        "ref" => {
                            if let Expr::Identifier(handle) = val {
                                attrs.push(format!("ref: {}", bare(handle)));
                            }
                        }
                        // `Form(bind: form)`: the handle takes the form.
                        "bind" if name == "Form" => {
                            if let Expr::Identifier(handle) = val {
                                attrs.push(format!("ref: {}", bare(handle)));
                            }
                        }
                        "bind" => {
                            // An empty date picker holds nothing,
                            // which is `null` — not the empty
                            // string the input reports.
                            // The event is `$$e`, which no name of
                            // the program's spells: the target's
                            // path may read a loop's item `e`.
                            let read = if name == "DatePicker" {
                                "$$e.target.value || null"
                            } else {
                                // A number field's number, a
                                // select's option as it was given.
                                "WF.bound($$e.target)"
                            };
                            // A state, a store's member or a loop
                            // item's field (`bind: t.title`) — the
                            // last used to compile one-way.
                            if let Some(target) = self.bind_target(val) {
                                attrs.push(format!("value: () => {} ?? \"\"", target.read));
                                attrs.push(format!(
                                    "\"on:input\": ($$e) => {{ {}; }}",
                                    target.write(read)
                                ));
                                if let Some(commit) = &target.commit {
                                    attrs.push(format!("\"on:change\": () => {{ {commit}; }}"));
                                }
                            } else {
                                // Anything else has no setter, so it
                                // cannot be bound; the check reports
                                // it where it is written.
                                let read = self.emit_expr(val);
                                attrs.push(format!("value: () => {read} ?? \"\""));
                            }
                        }
                        "checked" => {
                            if let Expr::Identifier(state_name) = val {
                                attrs.push(format!("checked: () => {}()", sig(state_name)));
                            } else {
                                let v = self.emit_expr(val);
                                attrs.push(format!("checked: {v}"));
                            }
                        }
                        "visible" => {
                            // Modal/Dialog visibility, from any
                            // expression that reads state.
                            let read = self.emit_expr(val);
                            attrs.push(format!(
                                "className: () => {} ? '{} open' : '{}'",
                                read,
                                classes.join(" "),
                                classes.join(" ")
                            ));
                        }
                        // A field's label, hint and error are elements
                        // beside the control, built by `WF.field` below.
                        "label" | "hint" | "error" if is_field => {}
                        // Motion the runtime reads, not attributes
                        // the element carries.
                        "shared" | "on" | "count" => {}
                        // `mount:`, `update:`, `cleanup:`: a lifetime,
                        // emitted below, on any element.
                        "mount" | "update" | "cleanup" | "trapFocus" | "roving" => {}
                        k if DRAG_PROPS.contains(&k) => {}
                        "tag" if name == "Host" => {}
                        // `.inert` goes with the text, below: it is
                        // how the text is drawn, not an attribute
                        // (HTML's own `inert` takes the element
                        // out of reach).
                        "inert" if name == "Markdown" => {}
                        // Where a `Portal` draws, read by its own
                        // emitter; never an attribute.
                        "into" if name == "Portal" => {}
                        // `maxLength` and `rows` are the textarea's,
                        // written as HTML writes them.
                        "maxLength" => {
                            let v = self.emit_expr(val);
                            attrs.push(format!("maxlength: {v}"));
                        }
                        // `Button(loading: Checkout.pay.pending)`: busy while
                        // it holds — a spinner in place of its label, at its
                        // width — and pressing it does nothing (`WF.busy`).
                        "loading" if name == "Button" => {
                            let v = self.emit_expr(val);
                            let read = if self.is_reactive(&v) {
                                format!("() => ({v}) ? \"true\" : null")
                            } else {
                                format!("({v}) ? \"true\" : null")
                            };
                            attrs.push(format!("\"aria-busy\": {read}"));
                            attrs.push(format!("\"aria-disabled\": {read}"));
                        }
                        "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max"
                        | "step" | "accept" | "label" | "required" | "disabled" | "controls"
                        | "autoplay" | "role" | "width" | "height" | "loading" | "decoding"
                        | "fetchpriority" | "rows" => {
                            // A value that reads state follows it: a
                            // `placeholder` or `disabled` bound to a
                            // store used to be painted once.
                            let v = if key == "src" && crate::project::url::draws_picture(name) {
                                self.picture_url_value(val)
                            } else if crate::project::url::URL_ATTRS.contains(&key.as_str()) {
                                self.url_value(val)
                            } else {
                                self.emit_expr(val)
                            };
                            if self.is_reactive(&v) {
                                attrs.push(format!("{key}: () => {v}"));
                            } else {
                                attrs.push(format!("{key}: {v}"));
                            }
                        }
                        "to" => {
                            let v = self.route_value(val);
                            *link_to = Some(v.clone());
                            if self.ssg_mode {
                                // SSG: plain links with base path prepended
                                attrs.push(format!("href: WF._basePath + {v}"));
                            } else {
                                attrs.push(format!("href: {v}"));
                                attrs.push(format!(
                                            "\"on:click\": ($$e) => {{ $$e.preventDefault(); WF.navigate({v}); }}"
                                        ));
                            }
                        }
                        "active" => {
                            // `active: "prefix"` also matches routes under `to`.
                            *link_prefix = matches!(val, Expr::StringLiteral(s) if s == "prefix");
                        }
                        "span" => {
                            if let Expr::NumberLiteral(n) = val
                                && let Some(n) = crate::project::builtin::column_span(*n)
                            {
                                classes.push(format!("{class}--{n}"));
                            }
                        }
                        "gap" | "align" | "justify" => {
                            // Handled once for the whole element below,
                            // via `layout_arg_classes`.
                        }
                        "class" => {
                            // The author's own classes join the
                            // element's in `emit_style_and_transition`,
                            // beside the style block's; as an
                            // attribute they would replace the
                            // engine's.
                        }
                        // `Grid(columns: 3)`: a `data-cols` the stylesheet
                        // reads, so no inline style is needed under a
                        // strict CSP; a value that reads state follows it.
                        "columns" => {
                            let v = self.emit_expr(val);
                            if self.is_reactive(&v) {
                                attrs.push(format!("\"data-cols\": () => {v}"));
                            } else {
                                attrs.push(format!("\"data-cols\": {v}"));
                            }
                        }
                        // A Modal's or Dialog's title is its heading, which
                        // the dialog reads; on anything else `title` is the
                        // global attribute — the tooltip a pointer shows.
                        "title" if matches!(name, "Modal" | "Dialog" | "AlertDialog" | "Sheet") => {
                            let v = self.emit_expr(val);
                            attrs.push(format!("\"data-title\": {v}"));
                        }
                        "title" => {
                            let v = self.emit_expr(val);
                            if self.is_reactive(&v) {
                                attrs.push(format!("title: () => {v}"));
                            } else {
                                attrs.push(format!("title: {v}"));
                            }
                        }
                        "caption" if name == "Table" => {
                            // Emitted as the table's first child below.
                        }
                        "value" => {
                            // A value that reads state follows it; the
                            // runtime runs a function-valued `value` in
                            // an effect. A Progress bar bound to state
                            // used to be painted once and never move.
                            let v = self.emit_expr(val);
                            if self.is_reactive(&v) {
                                attrs.push(format!("value: () => {v}"));
                            } else {
                                attrs.push(format!("value: {v}"));
                            }
                        }
                        "icon" => {
                            let v = self.emit_expr(val);
                            if self.is_reactive(&v) {
                                attrs.push(format!("\"data-icon\": () => {v}"));
                            } else {
                                attrs.push(format!("\"data-icon\": {v}"));
                            }
                        }
                        // `Code(…, language: "wf")`: paired with the
                        // content below, into a `highlight` attribute.
                        "language" if name == "Code" => {}
                        _ => {
                            // Any other named argument is an HTML
                            // attribute. A hyphenated name (`aria-*`,
                            // `data-*`) is quoted, and one of the
                            // runtime's own keys (`html`) is written
                            // as the attribute, not as markup; a
                            // value that reads state is a thunk,
                            // which the runtime keeps in step with
                            // it; and one the browser follows goes
                            // through the scheme check first.
                            let v = if crate::project::url::URL_ATTRS.contains(&key.as_str()) {
                                self.url_value(val)
                            } else {
                                self.emit_expr(val)
                            };
                            let k = attr_key(key);
                            if self.is_reactive(&v) {
                                attrs.push(format!("{k}: () => {v}"));
                            } else {
                                attrs.push(format!("{k}: {v}"));
                            }
                        }
                    }
                }
                Arg::Positional(expr) => {
                    // An Icon's positional argument names the glyph, not
                    // text to show: `Icon("home")` used to render the
                    // word "home" because only `icon:` set `data-icon`.
                    if name == "Icon" {
                        if !attrs.iter().any(|a| a.starts_with("\"data-icon\":")) {
                            // A name that reads state follows it, as
                            // `icon:` does: `Icon(if ok { "check" } else
                            // { "clock" })` was drawn once and kept.
                            let v = self.emit_expr(expr);
                            if self.is_reactive(&v) {
                                attrs.push(format!("\"data-icon\": () => {v}"));
                            } else {
                                attrs.push(format!("\"data-icon\": {v}"));
                            }
                        }
                        continue;
                    }
                    // `Unsafe.html(markup)`: the markup goes in as
                    // markup. It is the one element that does, and it
                    // is named so that a reviewer greps for it.
                    if name == "UnsafeHtml" {
                        let v = self.emit_expr(expr);
                        let getter = if self.is_reactive(&v) {
                            format!("() => {v}")
                        } else {
                            v
                        };
                        attrs.push(format!("html: {getter}"));
                        continue;
                    }
                    // Markdown is rendered to HTML by the runtime, and
                    // again when its text changes; `.inert` text is
                    // somebody else's, drawn with nothing to follow.
                    if name == "Markdown" {
                        let v = self.emit_expr(expr);
                        let getter = if self.is_reactive(&v) {
                            format!("() => {v}")
                        } else {
                            v
                        };
                        match self.markdown_inert(ui) {
                            None => attrs.push(format!("markdown: {getter}")),
                            Some(inert) => {
                                attrs.push(format!("markdown: {{text: {getter}, inert: {inert}}}"));
                            }
                        }
                        continue;
                    }
                    // `Option("value", "Label")`: the first positional
                    // is the value sent with the form, the second the
                    // text shown. A lone positional is both. The label
                    // used to be dropped, so a select showed its values.
                    if name == "Option" && inner_text.is_some() {
                        if !attrs.iter().any(|a| a.starts_with("value:")) {
                            let v = inner_text.take().unwrap_or_default();
                            attrs.push(format!("value: {v}"));
                            *inner_text = Some(self.emit_expr(expr));
                        }
                        continue;
                    }
                    // First positional arg is usually the content/label
                    // — unless `count:` asked for it to be counted to,
                    // in which case `WF.counted` writes the text.
                    if counts {
                        continue;
                    }
                    if inner_text.is_none() {
                        *inner_text = Some(self.emit_expr(expr));
                    }
                }
            }
        }
    }

    /// The generic pass over a built-in's arguments: its tag, the attributes
    /// it is created with, its text, where it links, and whether it is a
    /// field. The special emitters read it as the generic element does.
    fn element_parts(&mut self, name: &str, ui: &UIElement) -> ElementParts {
        let (_, class) = builtin_to_html(name);
        // A heading's level is part of the document outline, so it has to
        // reach the tag; a class cannot express it.
        let tag = if name == "Tcell" && self.in_thead {
            "th"
        } else if name == "Host" {
            // The library asked for a canvas, or a div by default.
            crate::project::builtin::host_tag(ui.args.iter().find_map(|a| match a {
                Arg::Named(k, Expr::StringLiteral(t)) if k == "tag" => Some(t.as_str()),
                _ => None,
            }))
        } else {
            element_tag(name, &ui.modifiers)
        };

        // Collect attributes
        let attrs = Vec::new();
        let link_to: Option<String> = None;
        let link_prefix = false;
        let inner_text: Option<String> = None;
        // `Text(total, count: "600ms")`: the number counts to its new
        // value rather than jumping, so its positional is the value
        // and not the text.
        let counts = ui
            .args
            .iter()
            .any(|a| matches!(a, Arg::Named(k, _) if k == "count"));

        // Build class string from base class + modifiers
        let mut classes = vec![class.to_string()];
        for m in &ui.modifiers {
            let mod_class = modifier_to_class(class, m);
            classes.push(mod_class);
        }

        // An `Input` or `Select` with a label, hint or error is wrapped
        // in a field that carries them.
        let validated = ui.args.iter().any(|a| {
            matches!(a, Arg::Named(k, Expr::Identifier(bound))
                        if k == "bind" && self.body.validated.contains(bound))
        });
        // A control is a field when it is labelled, or when a
        // `validate` block guards what it binds — which is what
        // gives it somewhere to show the message.
        let is_field = matches!(name, "Input" | "Select" | "Textarea")
                    && (validated
                        || ui.args.iter().any(|a| {
                            matches!(a, Arg::Named(k, _) if k == "label" || k == "hint" || k == "error")
                        }));

        // Process named args as HTML attributes
        let mut pass = ArgPass {
            name,
            class,
            counts,
            is_field,
            attrs,
            classes,
            link_to,
            link_prefix,
            inner_text,
        };
        self.named_args(ui, &mut pass);
        let ArgPass {
            mut attrs,
            mut classes,
            link_to,
            link_prefix,
            mut inner_text,
            ..
        } = pass;

        classes.extend(layout_arg_classes(&ui.args));
        // `Grid(columns: { base: 1, md: 2 })`: the class its rules
        // live under, as the pre-rendered page carries it. Without
        // it, hydrating the page took the layout away.
        classes.extend(crate::project::scoped_css::responsive_classes(ui));

        // Handle input type modifiers
        for m in &ui.modifiers {
            if let Some(t) = input_type(m) {
                attrs.push(format!("type: \"{t}\""));
            } else if m == "required" {
                attrs.push("required: true".to_string());
            } else if m == "multiple" {
                attrs.push("multiple: true".to_string());
            }
        }
        // A button submits its form only when it says so: the
        // browser's default, `submit`, made every button in a
        // form — "Add a row", "Show password" — send it.
        if matches!(name, "Button" | "IconButton") && !attrs.iter().any(|a| a.starts_with("type:"))
        {
            attrs.push("type: \"button\"".to_string());
        }

        // Classes attr
        let class_str = classes
            .iter()
            .filter(|c| !c.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        if !class_str.is_empty() {
            // Check if we already have a className (from visible binding)
            let has_class = attrs.iter().any(|a| a.starts_with("className:"));
            if !has_class {
                attrs.insert(0, format!("className: \"{class_str}\""));
            }
        }

        // The author's handlers are attached after the element is
        // built (see below), not written into the attribute object:
        // a `bind:` already puts an "on:input" there, and two keys of
        // one name in one object literal keep only the last — so an
        // Input with both used to lose its binding.

        // A form never navigates away: the author's submit handler
        // is attached after creation, on top of this.
        if name == "Form" {
            attrs.push("\"on:submit\": (e) => e.preventDefault()".to_string());
            if let Some(implicit) = &self.body.implicit_form
                && !ui
                    .args
                    .iter()
                    .any(|a| matches!(a, Arg::Named(k, _) if k == "bind"))
            {
                attrs.push(format!("ref: {implicit}"));
            }
        }

        // Async decoding keeps an image off the critical path. Whether
        // it loads lazily is the runtime's call: the first image of a
        // page is the one the largest paint waits for, and is fetched
        // first; the rest load lazily. An explicit `loading:` wins.
        if name == "Image"
            && !attrs.iter().any(|a| a.starts_with("decoding:"))
            && !ui
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, _) if k == "decoding"))
        {
            attrs.push("decoding: \"async\"".to_string());
        }

        // A live region has to be announced when it appears; a class
        // alone tells assistive technology nothing.
        if let Some(role) = implicit_role(name, &ui.modifiers)
            && !attrs.iter().any(|a| a.starts_with("role:"))
        {
            attrs.push(format!("role: \"{role}\""));
        }

        // Two `<nav>` landmarks on a page are two entries called
        // "navigation" in a screen reader's landmark list, with nothing
        // to choose between them.
        if let Some(label) = landmark_label(name)
            && !attrs.iter().any(|a| a.starts_with("\"aria-label\":"))
        {
            attrs.push(format!("\"aria-label\": \"{label}\""));
        }

        // A void element cannot hold text. `WF.el("hr", {}, "label")` asks
        // the runtime to append into a node that takes no children, and
        // the text is simply lost; carry it as the accessible name.
        if is_void(tag)
            && let Some(text) = inner_text.take()
            && !attrs
                .iter()
                .any(|a| a.starts_with("alt:") || a.starts_with("title:"))
        {
            attrs.push(format!("title: {text}"));
        }

        if tag == "th" {
            attrs.push("scope: \"col\"".to_string());
        }

        // Studio: stamp the node id on the element's root. Injecting here
        // covers the standard path and every special emitter that reuses
        // `attrs`/`attrs_str` (Modal, Switch, Checkbox, Dropdown, Spacer).
        if let Some(entry) = self.wf_node_entry(ui) {
            attrs.push(entry);
        }

        // `Code(…, language: "wf")`: the content and the language go
        // to the runtime's highlighter, not in as text.
        if name == "Code"
            && let Some(lang) = ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == "language" => Some(self.emit_expr(v)),
                _ => None,
            })
            && let Some(text) = inner_text.take()
        {
            let getter = |v: &str| {
                if self.is_reactive(v) {
                    format!("() => {v}")
                } else {
                    v.to_string()
                }
            };
            attrs.push(format!(
                "highlight: {{ code: {}, lang: {} }}",
                getter(&text),
                getter(&lang)
            ));
        }

        let attrs_str = if attrs.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", attrs.join(", "))
        };
        ElementParts {
            tag,
            attrs,
            attrs_str,
            inner_text,
            link_to,
            link_prefix,
            is_field,
        }
    }

    /// What `Markdown(text).inert` hands the runtime with the text: `None`
    /// for the plain form, whose output stays what it always was; `true`
    /// for the flag; and a value, kept in step where it reads state, for
    /// `inert: expression`.
    pub(in super::super) fn markdown_inert(&mut self, ui: &UIElement) -> Option<String> {
        let value = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "inert" => Some(v),
            _ => None,
        })?;
        match value {
            Expr::BoolLiteral(true) => Some("true".to_string()),
            Expr::BoolLiteral(false) => None,
            other => {
                let v = self.emit_expr(other);
                Some(if self.is_reactive(&v) {
                    format!("() => {v}")
                } else {
                    v
                })
            }
        }
    }

    /// A value going where the browser will follow it.
    ///
    /// A literal has already been checked by the time codegen runs — the
    /// check refuses a `javascript:` URL where it is written. Anything
    /// else is only known at run time, so it goes through the twin of that
    /// check at the moment it is used.
    pub(in super::super) fn url_value(&mut self, val: &Expr) -> String {
        let emitted = self.emit_expr(val);
        match val {
            Expr::StringLiteral(_) => emitted,
            _ => format!("WF.safeUrl({emitted})"),
        }
    }

    /// [`Self::url_value`] where a picture is drawn — an `<img>`'s `src` —
    /// which also takes a picture carried in its address (`data:image/…`).
    pub(in super::super) fn picture_url_value(&mut self, val: &Expr) -> String {
        let emitted = self.emit_expr(val);
        match val {
            Expr::StringLiteral(_) => emitted,
            _ => format!("WF.safePicture({emitted})"),
        }
    }

    /// A route or an address a page fetches, its splices encoded where they
    /// stand for a value: `"/team/{name}"` with `R&D/Ops` asks for
    /// `/team/R%26D%2FOps`, not `/team/R&D/Ops`. A splice that opens the
    /// address (`"{API}/rows"`), one that is a whole query (`"?{qs}"`), and
    /// anything after the `#` are written as they are.
    pub(in super::super) fn encoded_address(&self, val: &Expr) -> String {
        self.encoded_address_in(val, None)
    }

    /// An address, its expressions read in a store's terms when `store`
    /// names the store's members.
    pub(in super::super) fn encoded_address_in(
        &self,
        val: &Expr,
        store: Option<&[String]>,
    ) -> String {
        let Expr::InterpolatedString(parts) = val else {
            return self.term(val, store);
        };
        let mut out = String::from("`");
        let mut before = String::new();
        for part in parts {
            match part {
                StringPart::Literal(text) => {
                    out.push_str(&template_text(text));
                    before.push_str(text);
                }
                StringPart::Expression(e) => {
                    let v = self.term(e, store);
                    let raw = before.is_empty()
                        || before.contains('#')
                        || before.ends_with('?')
                        || before.ends_with('&');
                    if raw {
                        out.push_str(&format!("${{{v}}}"));
                    } else {
                        out.push_str(&format!("${{encodeURIComponent({v})}}"));
                    }
                    before.push('x');
                }
            }
        }
        out.push('`');
        out
    }

    /// `to:` and `navigate()`: the address encoded, then held to the schemes
    /// a browser may follow.
    pub(in super::super) fn route_value(&mut self, val: &Expr) -> String {
        match val {
            Expr::InterpolatedString(_) => format!("WF.safeUrl({})", self.encoded_address(val)),
            _ => self.url_value(val),
        }
    }

    /// What an element asked of motion beyond its enter animation: when to
    /// play it, and the name it keeps across a route change.
    pub(in super::super) fn emit_motion(&mut self, var: &str, ui: &UIElement) {
        // The timing props are lowered to `data-wf-*` markers before
        // codegen sees them, so look under both spellings.
        let named = |key: &str| {
            let marker = format!("data-wf-{key}");
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key || *k == marker => Some(v),
                _ => None,
            })
        };
        // `shared: "cover-{id}"`: the browser carries it from one page to
        // the next, which is what a View Transition is for.
        if let Some(name) = named("shared") {
            let value = self.emit_expr(name);
            self.emit_line(&format!("WF.shared({var}, {value});"));
        }
        // `on: .enterView`: it waits until the reader has scrolled to it.
        let plays_on = named("on")
            .and_then(|v| match v {
                Expr::EnumCase(case) => Some(case.clone()),
                _ => None,
            })
            .or_else(|| {
                ui.modifiers
                    .iter()
                    .find(|m| m.as_str() == "enterView")
                    .cloned()
            });
        // `count: "600ms"`: the number counts to where it has got to.
        // `format(v, .currency)` is split apart so the count is over the
        // number and the formatting is applied to each step of it.
        if let Some(over) = named("count") {
            let positional = ui.args.iter().find_map(|a| match a {
                Arg::Positional(e) => Some(e.clone()),
                Arg::Named(..) => None,
            });
            if let Some(expr) = positional {
                let (value, format) = match &expr {
                    Expr::FunctionCall(f, args) if f == "format" && !args.is_empty() => {
                        let tail: Vec<String> =
                            args[1..].iter().map(|a| self.emit_expr(a)).collect();
                        let call = if tail.is_empty() {
                            "WF.format(n)".to_string()
                        } else {
                            format!("WF.format(n, {})", tail.join(", "))
                        };
                        (self.emit_expr(&args[0]), format!("(n) => {call}"))
                    }
                    other => (self.emit_expr(other), "null".to_string()),
                };
                let over = self.emit_expr(over);
                self.emit_line(&format!(
                    "WF.counted({var}, () => {value}, {over}, {format});"
                ));
            }
        }
        if plays_on.as_deref() == Some("enterView")
            && let Some(animation) = animation_of(ui)
        {
            let timing =
                |key: &str| named(key).map_or_else(|| "null".to_string(), |v| self.emit_expr(v));
            self.emit_line(&format!(
                "WF.onEnterView({var}, \"{animation}\", {}, {}, {});",
                timing("duration"),
                timing("delay"),
                timing("easing")
            ));
        }
    }

    /// The class attribute for a built-in's root: its base class plus every
    /// modifier class.
    ///
    /// The special emitters below used to hardcode the bare base class, so a
    /// `Modal(large)` or a `Sidebar(elevated)` lost its variant while the same
    /// modifier on a `Card` worked.
    pub(in super::super) fn class_attr(&self, name: &str, ui: &UIElement) -> String {
        let (_, base) = builtin_to_html(name);
        let mut classes = crate::project::builtin::class_list(base, &ui.modifiers);
        // `Grid(columns: { base: 1, md: 2 })` carries the class its rules
        // live under, and nothing else: the widths are the stylesheet's.
        classes.extend(crate::project::scoped_css::responsive_classes(ui));
        classes.join(" ")
    }

    /// Apply an element's `style { }` and `transition { }` blocks to `var`.
    ///
    /// Every path that creates an element root calls this — the generic one and
    /// each special emitter — so an author's styling reaches a `Modal` or a
    /// `Slider` as surely as it reaches a `Card`.
    pub(in super::super) fn emit_style_and_transition(&mut self, var: &str, ui: &UIElement) {
        if let Some(style) = &ui.style_block {
            // A literal or a token is the same on every instance, so it lives
            // in styles.css under a class named by the block's content (see
            // `scoped_css`), along with the block's pseudo-state and media
            // rules; the element only carries the class. Only a value that
            // reads state is assigned here, inside an effect, so
            // `width: "{pct}%"` follows `pct` rather than painting once.
            for prop in &style.properties {
                if crate::project::scoped_css::static_declaration(prop).is_some() {
                    continue;
                }
                let (css_prop, val) = self.emit_style_decl(prop);
                // A custom property has no camel-cased field; it is set by name.
                let assign = if prop.name.starts_with("--") {
                    format!("{}.style.setProperty(\"{}\", {});", var, prop.name, val)
                } else {
                    format!("{var}.style.{css_prop} = {val};")
                };
                if self.is_reactive(&val) {
                    self.emit_line(&format!("WF.effect(() => {{ {assign} }});"));
                } else {
                    self.emit_line(&assign);
                }
            }
            if let Some(class) = crate::project::scoped_css::scoped_class(style) {
                self.emit_line(&format!("{var}.classList.add(\"{class}\");"));
            }
        }

        // A built-in's variant, size or flag given a value that reads state
        // (`Button("Save", tone: tone, outlined: busy)`) is its class, and
        // follows the state; it was written as an attribute nothing reads.
        if let ComponentRef::BuiltIn(name) = &ui.component {
            let (_, base) = builtin_to_html(name);
            let followed: Vec<String> = ui
                .args
                .iter()
                .filter_map(|a| match a {
                    Arg::Named(k, v) => self.state_class(name, base, k, v),
                    Arg::Positional(_) => None,
                })
                .collect();
            if !followed.is_empty() {
                self.emit_line(&format!(
                    "WF.classes({var}, () => [{}]);",
                    followed.join(", ")
                ));
            }
        }

        // `class:` names rules in the author's own stylesheet. A literal is
        // added once, beside the engine's classes; a value that reads state
        // is followed, and the classes it named last time are taken off.
        let class_value = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "class" => Some(v),
            _ => None,
        });
        // Written out in full — a string, `{ "x": true }`, `["a", "b"]` — it
        // is a fixed list; anything that reads a value is followed.
        match class_value.map(|v| crate::project::builtin::literal_classes(v).is_some()) {
            Some(true) => {
                let classes = crate::project::builtin::author_classes(&ui.args);
                if !classes.is_empty() {
                    let list = classes
                        .iter()
                        .map(|c| js_str(c))
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.emit_line(&format!("{var}.classList.add({list});"));
                }
            }
            Some(false) => {
                let expr = class_value.map(|v| self.emit_expr(v)).unwrap_or_default();
                self.emit_line(&format!("WF.classes({var}, () => {expr});"));
            }
            None => {}
        }

        if let Some(transition) = &ui.transition_block {
            let transitions: Vec<String> = transition
                .properties
                .iter()
                .map(|p| {
                    let easing = p
                        .easing
                        .as_deref()
                        .map_or("ease", crate::project::builtin::easing_css);
                    format!("{} {} {}", p.property, p.duration, easing)
                })
                .collect();
            self.emit_line(&format!(
                "{}.style.transition = {};",
                var,
                js_str(&transitions.join(", "))
            ));
        }
    }

    /// What the root of a built-in its own emitter draws — a layer, a long
    /// list, a splitter, tabs, a sidebar, a chart — takes as any other
    /// element does: every named argument that is not one of the
    /// component's own props, as `, key: value` (`element_attrs`). Its own
    /// props are its emitter's, and so is what `reads` names (a `Dropdown`
    /// is drawn by the menu's emitter, which reads a menu's props).
    pub(in super::super) fn root_attrs(
        &mut self,
        name: &str,
        ui: &UIElement,
        reads: &[&str],
    ) -> String {
        self.element_attrs(crate::registry::component(name), ui, &|k| {
            !reads.contains(&k)
        })
    }

    /// Every named argument of `ui` that `sig` has no prop of its own for,
    /// and that `keep` keeps, as `, key: value`: the global attributes
    /// (`id:`, `role:`, `tabindex:`, `title:` …), `aria-*` and `data-*`, a
    /// `ref:`, and anything else, which `C04` says is written to the
    /// element as an attribute — `html:` too, which is an attribute of that
    /// name and never markup (`attr_key`). A prop written as an attribute
    /// (`disabled:`) is one too. A value that reads state follows it, and
    /// one the browser follows goes through the scheme check first, as on
    /// the generic path. The props every element takes (`class:`, motion,
    /// a lifetime, a drag) are the shared tail's.
    pub(in super::super) fn element_attrs(
        &mut self,
        sig: Option<&'static crate::registry::ComponentSig>,
        ui: &UIElement,
        keep: &dyn Fn(&str) -> bool,
    ) -> String {
        let mut out = String::new();
        for arg in &ui.args {
            let Arg::Named(key, val) = arg else { continue };
            if !keep(key) {
                continue;
            }
            if key == "ref" {
                if let Expr::Identifier(handle) = val {
                    out.push_str(&format!(", ref: {}", bare(handle)));
                }
                continue;
            }
            if sig.is_some_and(|s| {
                s.prop(key)
                    .is_some_and(|p| p.sink != crate::registry::Sink::Attr)
            }) {
                continue;
            }
            // A value per breakpoint is carried by a class, not an attribute.
            if crate::project::scoped_css::is_responsive(val) {
                continue;
            }
            let value = if crate::project::url::URL_ATTRS.contains(&key.as_str()) {
                self.url_value(val)
            } else {
                self.emit_expr(val)
            };
            let k = attr_key(key);
            if self.is_reactive(&value) {
                out.push_str(&format!(", {k}: () => {value}"));
            } else {
                out.push_str(&format!(", {k}: {value}"));
            }
        }
        out
    }

    /// Whether the author gave `key:` — a `role:` that takes the place of
    /// the one a built-in's root would have.
    pub(in super::super) fn gives(ui: &UIElement, key: &str) -> bool {
        ui.args
            .iter()
            .any(|a| matches!(a, Arg::Named(k, _) if k == key))
    }

    /// A built-in's `on …` handlers, on its root as on any element: every
    /// one but `on close`, which a layer hears itself (`close_handler`)
    /// when `own_close` says so.
    /// A button's click or a form's submit that starts work — it awaits,
    /// or calls an action of the page's or a store's that awaits —
    /// listened to through `WF.busy`, which marks the control busy until
    /// that work ends and takes no other press meanwhile. `None` for a
    /// handler that only changes state, which needs none of that.
    pub(in super::super) fn busy_listener(
        &self,
        var: &str,
        ui: &UIElement,
        handler: &crate::syntax::parser::ast::EventHandler,
        body: &str,
    ) -> Option<String> {
        let mut stmts = handler.body.clone();
        let mut works = crate::syntax::parser::ast::awaits(&stmts);
        crate::syntax::parser::ast::walk_exprs_mut(&mut stmts, &mut |e| match e {
            Expr::FunctionCall(f, _) if self.body.async_actions.contains(f) => works = true,
            Expr::MethodCall(obj, action, _)
                if matches!(&**obj, Expr::Identifier(s)
                    if self.store_async_actions.iter().any(|(st, a)| st == s && a == action)) =>
            {
                works = true;
            }
            _ => {}
        });
        // A button that says when it is loading takes no press meanwhile,
        // whatever its handler does.
        if !works && !Self::gives(ui, "loading") {
            return None;
        }
        let head = self.handler_head(handler, "event");
        Some(format!(
            "{var}.addEventListener(\"{}\", WF.busy({var}, {}));",
            handler.event,
            self.paced(handler, format!("{head} => {{ {body} }}"))
        ))
    }

    pub(in super::super) fn emit_root_handlers(
        &mut self,
        var: &str,
        ui: &UIElement,
        own_close: bool,
    ) {
        for handler in &ui.events {
            if own_close && handler.event == "close" {
                continue;
            }
            let body = self.emit_event_body(handler);
            let line = self.element_listener(var, handler, &body);
            self.emit_line(&line);
        }
    }
}

/// What [`JsCodegen::element_parts`] works out about a built-in.
struct ElementParts {
    tag: &'static str,
    attrs: Vec<String>,
    attrs_str: String,
    inner_text: Option<String>,
    link_to: Option<String>,
    link_prefix: bool,
    is_field: bool,
}

/// What [`JsCodegen::named_args`] reads about a built-in, and what it
/// fills in.
struct ArgPass<'a> {
    name: &'a str,
    class: &'a str,
    counts: bool,
    is_field: bool,
    attrs: Vec<String>,
    classes: Vec<String>,
    link_to: Option<String>,
    link_prefix: bool,
    inner_text: Option<String>,
}

impl JsCodegen {
    /// The class a built-in's class prop names when its value reads state:
    /// a lookup from each case to its class for an enum (`tone: tone`), a
    /// condition for a `Bool` (`outlined: busy`). `None` for a literal, or
    /// a prop that is not a class.
    fn state_class(&self, name: &str, base: &str, key: &str, val: &Expr) -> Option<String> {
        use crate::registry::{Legacy, PropType, Sink};
        let prop = crate::registry::component(name)?.prop(key)?;
        let layout = matches!(key, "gap" | "align" | "justify");
        if prop.sink != Sink::Class && !layout {
            return None;
        }
        let value = self.emit_expr(val);
        if !self.is_reactive(&value) {
            return None;
        }
        match prop.ty {
            PropType::Enum(cases) => {
                let pairs: Vec<String> = cases
                    .iter()
                    .filter_map(|c| {
                        let class = if layout {
                            format!("wf-{key}--{}", c.name)
                        } else if c.legacy.is_empty() {
                            return None;
                        } else {
                            modifier_to_class(base, c.legacy)
                        };
                        (!class.is_empty())
                            .then(|| format!("{}: {}", js_str(c.name), js_str(&class)))
                    })
                    .collect();
                Some(format!("({{ {} }})[{value}] ?? null", pairs.join(", ")))
            }
            PropType::Bool => {
                let word = match prop.legacy {
                    Legacy::Modifier(w) => w,
                    Legacy::Attr => return None,
                };
                let class = modifier_to_class(base, word);
                (!class.is_empty()).then(|| format!("({value}) ? {} : null", js_str(&class)))
            }
            _ => None,
        }
    }
}
