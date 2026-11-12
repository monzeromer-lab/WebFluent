//! Layers over the page: modals, dialogs, popovers, tooltips, menus and dropdowns.

use super::super::*;

impl JsCodegen {
    pub(in super::super) fn emit_modal_dialog(
        &mut self,
        name: &str,
        var: &str,
        _attrs_str: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        let class = if name == "Modal" {
            "wf-modal"
        } else {
            "wf-dialog"
        };
        let title = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "title" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let visible_binding = self.visibility_binding(ui);

        // The root carries the modifier classes too; `class` stays the bare base
        // because the sub-part classes (`__content`, `__header`) derive from it.
        let root_classes = self.class_attr(name, ui);
        // A real `<dialog>`: the browser then traps focus inside it, makes the
        // rest of the page inert, closes on Escape and exposes `aria-modal`.
        let title_id = format!("wf-dlg-{}", var.trim_start_matches("_e"));
        let labelled = if title.is_some() {
            format!(", \"aria-labelledby\": \"{}\"", title_id)
        } else {
            String::new()
        };
        let extra = self.root_attrs(name, ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"dialog\", {{ className: \"{}\"{}{}{} }});",
            var,
            root_classes,
            labelled,
            extra,
            self.wf_node_inline(ui)
        ));

        let content_var = self.fresh_var();
        let content_class = format!("{}__content", class);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\" }});",
            content_var, content_class
        ));

        // `Modal.Title { … }`: what the heading draws, in place of the
        // title's words, which stay its name.
        let heading_part = ui.children.iter().find_map(|child| match &child.kind {
            StatementKind::UIElement(part) if matches!(&part.component, ComponentRef::SubComponent(p, s) if p == name && s == "Title") => Some(part.children.clone()),
            _ => None,
        });
        if let Some(t) = title {
            let header_var = self.fresh_var();
            if let Some(drawn) = &heading_part {
                let h3 = self.fresh_var();
                self.emit_line(&format!(
                    "const {h3} = WF.el(\"h3\", {{ id: \"{title_id}\" }});"
                ));
                for child in drawn {
                    self.emit_statement_dom(child, &h3);
                }
                self.emit_line(&format!(
                    "const {header_var} = WF.el(\"div\", {{ className: \"{class}__header\" }}, {h3});"
                ));
            } else {
                // A title that reads state follows it, as any text does:
                // `Modal(title: "Explain {code}")` kept the code it opened with.
                let t = if self.is_reactive(&t) {
                    format!("() => {t}")
                } else {
                    t
                };
                self.emit_line(&format!(
                    "const {} = WF.el(\"div\", {{ className: \"{}__header\" }}, WF.el(\"h3\", {{ id: \"{}\" }}, {}));",
                    header_var, class, title_id, t
                ));
            }
            // `.closable`: a close button in the heading row, closing it as
            // Escape does, so `on close` and `visible:` follow.
            if ui.modifiers.iter().any(|m| m == "closable") {
                self.emit_line(&format!(
                    "{header_var}.appendChild(WF.el(\"button\", {{ className: \"{class}__close\", type: \"button\", \"aria-label\": \"Close\", \"on:click\": () => ({var}._wfShut ? {var}._wfShut() : {var}.close()) }}, WF.el(\"i\", {{ className: \"wf-icon\", \"data-icon\": \"close\" }})));"
                ));
            }
            self.emit_line(&format!("{}.appendChild({});", content_var, header_var));
        }

        let body_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}__body\" }});",
            body_var, class
        ));

        // Check for Modal.Footer
        let mut footer_stmts = Vec::new();
        let mut footer_wf = String::new();
        let mut body_stmts = Vec::new();
        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind
                && matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == name && s == "Footer")
            {
                footer_stmts = ui_child.children.clone();
                footer_wf = self.wf_node_inline(ui_child);
                continue;
            }
            if let StatementKind::UIElement(ui_child) = &child.kind
                && matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == name && s == "Title")
            {
                continue;
            }
            body_stmts.push(child);
        }

        for child in &body_stmts {
            self.emit_statement_dom(child, &body_var);
        }
        self.emit_line(&format!("{}.appendChild({});", content_var, body_var));

        if !footer_stmts.is_empty() {
            let footer_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"div\", {{ className: \"{}__footer\"{} }});",
                footer_var, class, footer_wf
            ));
            for child in &footer_stmts {
                self.emit_statement_dom(child, &footer_var);
            }
            self.emit_line(&format!("{}.appendChild({});", content_var, footer_var));
        }

        self.emit_line(&format!("{}.appendChild({});", var, content_var));
        self.emit_root_handlers(var, ui, false);

        // Visibility binding. `WF.dialog` drives `showModal()`/`close()` and
        // writes the signal back when the browser closes the dialog itself — via
        // Escape or the backdrop — so the state cannot drift out of sync with
        // what is on screen.
        if let Some(binding) = visible_binding {
            self.emit_line(&format!("WF.dialog({var}, {binding});"));
        }

        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    /// `visible:` as a read, and the write that follows the screen when the
    /// browser closes a dialog itself, or a layer is dismissed. It is any
    /// expression that reads state: a page's own (`visible: open`), a
    /// store's (`visible: Ui.palette`), or a condition (`visible:
    /// Ui.confirmId != null`); the first two are written back, a condition
    /// is not.
    pub(in super::super) fn visibility_binding(&mut self, ui: &UIElement) -> Option<String> {
        let (read, write) = self.visibility_parts(ui)?;
        Some(format!("{read}, {write}"))
    }

    /// `visible:` as a read function and a write function (`null` when it
    /// is a condition, which cannot be written).
    pub(in super::super) fn visibility_parts(
        &mut self,
        ui: &UIElement,
    ) -> Option<(String, String)> {
        let visible = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "visible" => Some(v),
            _ => None,
        })?;
        let read = self.emit_expr(visible);
        let write = match visible {
            Expr::Identifier(_) if self.is_state_signal(visible) => {
                Some(format!("(v) => {}.set(v)", read.trim_end_matches("()")))
            }
            Expr::PropertyAccess(..) if self.is_store_member(visible, &read) => {
                Some(format!("(v) => {{ {read} = v; }}"))
            }
            _ => None,
        };
        Some((
            format!("() => {read}"),
            write.unwrap_or_else(|| "null".to_string()),
        ))
    }

    /// What a layer is placed against, where, and how far: the options
    /// `WF.layer` takes, from `anchor:`, `at:`, `placement:` and `offset:`.
    pub(in super::super) fn layer_options(&mut self, ui: &UIElement) -> Vec<String> {
        let mut opts = Vec::new();
        for arg in &ui.args {
            let Arg::Named(key, value) = arg else {
                continue;
            };
            match key.as_str() {
                "anchor" | "at" => {
                    let at = self.emit_expr(value);
                    opts.push(format!("anchor: () => {at}"));
                }
                // `.bottomStart` arrives as `"bottomStart"`; the runtime
                // reads it as `bottom-start`.
                "placement" => {
                    let placement = self.emit_expr(value);
                    opts.push(format!("placement: {placement}"));
                }
                "offset" => {
                    let offset = self.emit_expr(value);
                    opts.push(format!("offset: {offset}"));
                }
                _ => {}
            }
        }
        opts
    }

    /// `Popover`: a layer at a control, drawn above everything by
    /// `WF.layer`, which also places it, follows its anchor, and closes it on
    /// `Escape` or a press outside — writing `visible:` back and firing
    /// `on close` with why.
    pub(in super::super) fn emit_popover(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let binding = self
            .visibility_binding(ui)
            .unwrap_or_else(|| "() => false, null".to_string());
        let label = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "label" => Some(v.clone()),
            _ => None,
        });
        let labelled = match label {
            Some(v) => format!(", \"aria-label\": {}", self.emit_expr(&v)),
            None => String::new(),
        };
        // A popover is a dialog unless its author says what it is: a
        // `listbox` of choices, a `tooltip`.
        let role = if Self::gives(ui, "role") {
            ""
        } else {
            ", role: \"dialog\""
        };
        let extra = self.root_attrs("Popover", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{}{}{} }});",
            var,
            self.class_attr("Popover", ui),
            role,
            labelled,
            extra,
            self.wf_node_inline(ui)
        ));
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }
        self.emit_root_handlers(var, ui, true);
        let mut opts = self.layer_options(ui);
        opts.extend(self.close_handler(ui));
        self.emit_line(&format!(
            "WF.layer({var}, {binding}, {{ {} }});",
            opts.join(", ")
        ));
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    pub(in super::super) fn emit_dropdown_menu(
        &mut self,
        name: &str,
        var: &str,
        _attrs: &[String],
        ui: &UIElement,
        parent: &str,
    ) {
        let class = if name == "Dropdown" {
            "wf-dropdown"
        } else {
            "wf-menu"
        };
        // What the button says: `trigger:` (a `Dropdown`'s `label:`). A
        // `Menu`'s `label:` is the list's accessible name, so a menu that
        // state opens — with no trigger — draws no button.
        let label = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "trigger" || (k == "label" && name == "Dropdown") => {
                Some(self.emit_expr(v))
            }
            _ => None,
        });
        let named = if name == "Menu" {
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == "label" => Some(self.emit_expr(v)),
                _ => None,
            })
        } else {
            None
        };
        let aria_name = named
            .map(|n| format!(", \"aria-label\": {n}"))
            .unwrap_or_default();
        let visible = self.visibility_parts(ui);
        let mut opts = self.layer_options(ui);
        opts.extend(self.close_handler(ui));
        let opts = format!("{{ {} }}", opts.join(", "));
        let items_class = format!("{}__items", class);

        // Opened by state alone — a context menu, or a menu a menu bar
        // opens: the list is the whole of it.
        if let (None, Some((read, write))) = (&label, &visible) {
            let classes =
                crate::project::builtin::class_list(&items_class, &ui.modifiers).join(" ");
            let role = if Self::gives(ui, "role") {
                ""
            } else {
                ", role: \"menu\""
            };
            let extra = self.root_attrs(name, ui, MENU_READS);
            self.emit_line(&format!(
                "const {} = WF.el(\"ul\", {{ className: \"{}\"{}{}{}{} }});",
                var,
                classes,
                role,
                aria_name,
                extra,
                self.wf_node_inline(ui)
            ));
            self.emit_line(&format!(
                "WF.menu({var}, null, {var}, {read}, {write}, {opts});"
            ));
            self.menu_lists.push(var.to_string());
            for child in &ui.children {
                self.emit_statement_dom(child, var);
            }
            self.menu_lists.pop();
            self.emit_root_handlers(var, ui, true);
            self.emit_line(&format!("{}.appendChild({});", parent, var));
            return;
        }

        let label = label.unwrap_or_else(|| "\"Menu\"".to_string());
        // A trigger that reads state follows it, as any text does: the
        // button of `Menu(trigger: if wide { "Full" } else { width })`
        // kept the words it was drawn with.
        let label = if self.is_reactive(&label) {
            format!("() => {label}")
        } else {
            label
        };
        let root_classes = self.class_attr(name, ui);
        let extra = self.root_attrs(name, ui, MENU_READS);
        // What says the menu is open: `visible:` when it is given and can
        // be written, else the menu's own.
        let (read, write) = match visible {
            Some((read, write)) if write != "null" => (read, write),
            _ => {
                let open_var = self.fresh_var();
                self.emit_line(&format!("const {} = WF.signal(false);", open_var));
                (open_var.clone(), format!("(v) => {open_var}.set(v)"))
            }
        };
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            var,
            root_classes,
            extra,
            self.wf_node_inline(ui)
        ));
        // `open` is the only class the menu's state owns: toggled on its
        // own, so the classes the author gave it (`class:`, a style block's)
        // stay when it opens. Writing the whole class list each time
        // dropped them.
        self.emit_line(&format!(
            "WF.effect(() => {var}.classList.toggle(\"open\", !!({read})()));"
        ));

        // `aria-expanded` is the only thing that tells a screen reader whether
        // the menu is open; the class ternary is visual only.
        // Each menu drawn has its own: a `for` draws many from one call.
        let items_id = self.fresh_var();
        self.emit_line(&format!("const {items_id} = WF.uid(\"wf-menu\");"));
        let trigger_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"button\", {{ className: \"wf-btn wf-menu__trigger\", type: \"button\", \
             \"aria-haspopup\": \"menu\", \"aria-controls\": {}, \
             \"aria-expanded\": () => ({})() ? \"true\" : \"false\", \
             \"on:click\": () => ({})(!({})()) }}, {});",
            trigger_var, items_id, read, write, read, label
        ));
        self.emit_line(&format!("{}.appendChild({});", var, trigger_var));

        // The items are `li`s, so their container is a list; `role="menu"`
        // gives it the menu semantics. It is drawn in the top layer, at the
        // button, so no `overflow` above it clips it.
        let items_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"ul\", {{ className: \"{}\", id: {}, role: \"menu\"{} }});",
            items_var, items_class, items_id, aria_name
        ));
        self.emit_line(&format!(
            "WF.menu({var}, {trigger_var}, {items_var}, {read}, {write}, {opts});"
        ));
        self.menu_lists.push(items_var.clone());
        for child in &ui.children {
            self.emit_statement_dom(child, &items_var);
        }
        self.menu_lists.pop();
        self.emit_line(&format!("{}.appendChild({});", var, items_var));
        self.emit_root_handlers(var, ui, true);
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    /// `on close(reason)` on a layer, as the `onClose` option.
    pub(in super::super) fn close_handler(&mut self, ui: &UIElement) -> Option<String> {
        let handler = ui.events.iter().find(|h| h.event == "close")?;
        let body = self.emit_event_body(handler);
        Some(format!(
            "onClose: {} => {{ {} }}",
            self.handler_head(handler, "reason"),
            body
        ))
    }

    /// A menu's entry: a `menuitem`, or a `menuitemcheckbox` with
    /// `checked:`; `disabled:` keeps it in sight but out of reach;
    /// `shortcut:` is drawn at its end.
    pub(in super::super) fn emit_menu_item(
        &mut self,
        owner: &str,
        var: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        let base = format!("wf-{}__item", owner.to_lowercase());
        let class = crate::project::builtin::class_list(&base, &ui.modifiers).join(" ");
        let mut attrs = vec![format!("className: \"{class}\"")];
        let mut shortcut = None;
        let mut role = "menuitem";
        for arg in &ui.args {
            let Arg::Named(k, v) = arg else { continue };
            let value = self.emit_expr(v);
            let live = |this: &Self, v: String| {
                if this.is_reactive(&v) {
                    format!("() => {v}")
                } else {
                    v
                }
            };
            match k.as_str() {
                "shortcut" => shortcut = Some(value),
                "checked" => {
                    role = "menuitemcheckbox";
                    let read = live(self, format!("(({value}) ? \"true\" : \"false\")"));
                    attrs.push(format!("\"aria-checked\": {read}"));
                }
                "disabled" => {
                    let read = live(self, format!("(({value}) ? \"true\" : \"false\")"));
                    attrs.push(format!("\"aria-disabled\": {read}"));
                }
                // Added beside the item's own classes with its style, below:
                // as an attribute it would write over them.
                "class" => {}
                _ => {
                    let key = attr_key(k);
                    attrs.push(format!("{key}: {}", live(self, value)));
                }
            }
        }
        attrs.push(format!("role: \"{role}\""));
        attrs.push("tabindex: \"-1\"".to_string());
        if let Some(entry) = self.wf_node_entry(ui) {
            attrs.push(entry);
        }
        self.emit_line(&format!(
            "const {} = WF.el(\"li\", {{ {} }});",
            var,
            attrs.join(", ")
        ));
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }
        // A shortcut is a key spelling, `mod+b`, written as this system
        // writes it: `⌘B` on a Mac, `Ctrl+B` elsewhere.
        if let Some(keys) = shortcut {
            self.emit_line(&format!(
                "{var}.appendChild(WF.el(\"kbd\", {{ className: \"wf-menu__shortcut\", \"aria-hidden\": \"true\" }}, WF.keyLabel({keys})));"
            ));
            // Said once, as the item's own shortcut.
            self.emit_line(&format!(
                "{var}.setAttribute(\"aria-keyshortcuts\", WF.keyAria({keys}));"
            ));
        }
        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            let line = self.element_listener(var, handler, &body);
            self.emit_line(&line);
        }
        self.emit_style_and_transition(var, ui);
        self.emit_motion(var, ui);
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    /// `Menu.Sub(label:)`: an entry with a menu of its own, opened beside
    /// it by `WF.submenu`.
    pub(in super::super) fn emit_menu_sub(
        &mut self,
        owner: &str,
        var: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        let label = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, v) if k == "label" => Some(self.emit_expr(v)),
                _ => None,
            })
            .unwrap_or_else(|| "\"\"".to_string());
        let base = format!("wf-{}", owner.to_lowercase());
        // What any element takes is the entry's; its `role:` and
        // `tabindex:` in place of its own.
        let role = if Self::gives(ui, "role") {
            ""
        } else {
            ", role: \"menuitem\""
        };
        let tabindex = if Self::gives(ui, "tabindex") {
            ""
        } else {
            ", tabindex: \"-1\""
        };
        let extra = self.element_attrs(crate::registry::part(owner, "Sub"), ui, &|_| true);
        self.emit_line(&format!(
            "const {} = WF.el(\"li\", {{ className: \"{base}__item {base}__sub\"{role}{tabindex}{}{} }}, WF.el(\"span\", {{}}, {}), WF.el(\"span\", {{ className: \"{base}__chevron\", \"aria-hidden\": \"true\" }}, \"\\u203a\"));",
            var,
            extra,
            self.wf_node_inline(ui),
            label
        ));
        let list_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"ul\", {{ className: \"{base}__items\", role: \"menu\" }});",
            list_var
        ));
        if let Some(owner_list) = self.menu_lists.last().cloned() {
            self.emit_line(&format!("WF.submenu({var}, {list_var}, {owner_list});"));
        }
        self.menu_lists.push(list_var.clone());
        for child in &ui.children {
            self.emit_statement_dom(child, &list_var);
        }
        self.menu_lists.pop();
        self.emit_line(&format!("{}.appendChild({});", var, list_var));
        // Its handlers, `class:` and style, as a menu item's.
        self.emit_root_handlers(var, ui, false);
        self.emit_style_and_transition(var, ui);
        self.emit_motion(var, ui);
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }

    pub(in super::super) fn emit_tooltip(&mut self, var: &str, ui: &UIElement, parent: &str) {
        // `Tooltip("Save")` or `Tooltip(text: "Save")`.
        let text = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, v) if k == "text" => Some(self.emit_expr(v)),
                Arg::Positional(v) => Some(self.emit_expr(v)),
                _ => None,
            })
            .unwrap_or_else(|| "\"\"".to_string());

        // An id of its own at run time: a component's tooltip, or one a
        // loop draws, is the same code for every copy.
        let tip_id = self.fresh_var();
        self.emit_line(&format!("const {tip_id} = WF.uid(\"wf-tip\");"));
        let extra = self.root_attrs("Tooltip", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            var,
            self.class_attr("Tooltip", ui),
            extra,
            self.wf_node_inline(ui)
        ));

        // Render children (the trigger element)
        for child in &ui.children {
            self.emit_statement_dom(child, var);
        }

        let tip_var = self.fresh_var();
        self.emit_line(&format!(
            "const {} = WF.el(\"span\", {{ className: \"wf-tooltip__text\", role: \"tooltip\", id: {} }}, {});",
            tip_var,
            tip_id,
            self.live(&text)
        ));
        let mut opts = Vec::new();
        for arg in &ui.args {
            let Arg::Named(k, v) = arg else { continue };
            match k.as_str() {
                // The keys that do what the control does, as a chip.
                "shortcut" => {
                    let keys = self.emit_expr(v);
                    self.emit_line(&format!(
                        "{tip_var}.appendChild(WF.el(\"kbd\", {{ className: \"wf-tooltip__shortcut\" }}, WF.keyLabel({keys})));"
                    ));
                    opts.push(format!("keys: {keys}"));
                }
                "placement" | "delay" => {
                    let value = self.emit_expr(v);
                    opts.push(format!("{k}: {value}"));
                }
                _ => {}
            }
        }
        self.emit_line(&format!("{}.appendChild({});", var, tip_var));
        self.emit_root_handlers(var, ui, false);
        // The runtime points the trigger at the tip, shows it above
        // everything at the trigger, and makes it reachable and dismissible
        // from the keyboard.
        self.emit_line(&format!(
            "WF.tooltip({}, {}, {{ {} }});",
            var,
            tip_var,
            opts.join(", ")
        ));
        self.emit_line(&format!("{}.appendChild({});", parent, var));
    }
}
