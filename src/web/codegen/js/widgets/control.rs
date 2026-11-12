//! Form controls: inputs in a box, switches, checkboxes, radios, sliders, date pickers, file uploads, and validation.

use super::super::*;

/// What a `state`'s declared type says its values must be, when it says
/// anything: `state nights: Number(1..=30)` is a rule before a `validate`
/// block adds one.
pub(in super::super) fn refinement_in(stmts: &[Statement], name: &str) -> Vec<(String, Expr)> {
    for stmt in stmts {
        if let StatementKind::State(s) = &stmt.kind
            && s.name == name
            && let Some(ty) = &s.ty
        {
            return ty.refinement().to_vec();
        }
        let found = refinement_in_bodies(stmt, name);
        if !found.is_empty() {
            return found;
        }
    }
    Vec::new()
}

pub(in super::super) fn refinement_in_bodies(stmt: &Statement, name: &str) -> Vec<(String, Expr)> {
    for body in stmt.kind.bodies() {
        let found = refinement_in(body, name);
        if !found.is_empty() {
            return found;
        }
    }
    Vec::new()
}

/// The states a body's `validate` blocks guard.
pub(in super::super) fn validated_names(stmts: &[Statement]) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            if let StatementKind::Validate(v) = &stmt.kind {
                out.push(v.name.clone());
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    walk(stmts, &mut out);
    out
}

/// Whether a `Form` is anywhere in the body.
pub(in super::super) fn has_form(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| {
        if let StatementKind::UIElement(el) = &stmt.kind {
            if matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Form") {
                return true;
            }
            if has_form(&el.children) || el.slot_fills.iter().any(|f| has_form(&f.body)) {
                return true;
            }
        }
        stmt.kind.bodies().into_iter().any(has_form)
    })
}

/// `Form(show: .live)` — when a message is shown — as the runtime takes it.
pub(in super::super) fn form_options(stmts: &[Statement]) -> String {
    fn find(stmts: &[Statement]) -> Option<String> {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &stmt.kind
                && matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Form")
            {
                for arg in &el.args {
                    if let Arg::Named(key, Expr::EnumCase(case)) = arg
                        && key == "show"
                    {
                        return Some(case.clone());
                    }
                }
                // An enum prop written as a flag is a modifier by now.
                for modifier in &el.modifiers {
                    if matches!(modifier.as_str(), "onBlur" | "onSubmit" | "live") {
                        return Some(modifier.clone());
                    }
                }
            }
            for body in stmt.kind.bodies() {
                if let Some(found) = find(body) {
                    return Some(found);
                }
            }
            if let StatementKind::UIElement(el) = &stmt.kind
                && let Some(found) = find(&el.children)
            {
                return Some(found);
            }
        }
        None
    }
    match find(stmts) {
        Some(show) => format!("{{ show: \"{show}\" }}"),
        None => String::new(),
    }
}

impl JsCodegen {
    /// The attributes of a wrapped control that belong on its `<input>`:
    /// `aria-*`, `data-*`, `name`, `id`, `disabled`, `required`, `tabindex`
    /// — what a screen reader, a form handle or a stylesheet reads off it.
    pub(in super::super) fn control_input_attrs(attrs: &[String]) -> String {
        attrs
            .iter()
            .filter(|a| {
                let key = a.trim_start_matches('"');
                key.starts_with("aria-")
                    || key.starts_with("data-")
                    || ["name:", "id:", "disabled:", "required:", "tabindex:"]
                        .iter()
                        .any(|k| key.starts_with(k))
            })
            .map(|a| format!(", {a}"))
            .collect()
    }

    /// The rest of what a wrapped control takes as any element does —
    /// `role:`, `title:`, a `ref:`, any other attribute — on its input
    /// beside what `control_input_attrs` puts there: the input is the
    /// control, and its handlers are there too. What is the whole
    /// control's (`BOX_ATTRS`) is its box's.
    pub(in super::super) fn control_extra_attrs(&mut self, name: &str, ui: &UIElement) -> String {
        self.element_attrs(crate::registry::component(name), ui, &|k| {
            !k.contains('-')
                && !BOX_ATTRS.contains(&k)
                && !matches!(
                    k,
                    "name" | "id" | "disabled" | "required" | "tabindex" | "checked"
                )
        })
    }

    /// What a wrapped control's box takes: `hidden:`, `lang:`, `dir:`,
    /// which are the whole control's — its label and its input together.
    pub(in super::super) fn control_box_attrs(&mut self, name: &str, ui: &UIElement) -> String {
        self.element_attrs(crate::registry::component(name), ui, &|k| {
            BOX_ATTRS.contains(&k)
        })
    }

    /// A control's handlers as its input's: `on change` beside a `bind:`
    /// runs after the binding's own write, in one listener — two
    /// `"on:change"` keys in one object keep only the last, which dropped
    /// the binding.
    pub(in super::super) fn control_handlers(
        &mut self,
        ui: &UIElement,
        bound_change: Option<String>,
    ) -> String {
        let mut out = String::new();
        let mut merged = false;
        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            let head = self.handler_head(handler, "e");
            if handler.event == "change"
                && let Some(write) = &bound_change
            {
                out.push_str(&format!(", \"on:change\": {head} => {{ {write}; {body} }}"));
                merged = true;
            } else {
                out.push_str(&format!(
                    ", \"on:{}\": {head} => {{ {body} }}",
                    handler.event
                ));
            }
        }
        if !merged && let Some(write) = bound_change {
            out.push_str(&format!(", \"on:change\": () => {{ {write}; }}"));
        }
        out
    }

    /// The message a validated `Checkbox` or `Switch` shows, beneath it.
    pub(in super::super) fn emit_choice_message(
        &mut self,
        ui: &UIElement,
        var: &str,
        parent: &str,
    ) {
        let Some(Arg::Named(_, Expr::Identifier(bound))) = ui
            .args
            .iter()
            .find(|a| matches!(a, Arg::Named(k, _) if k == "bind"))
        else {
            return;
        };
        if !self.body.validated.contains(bound) {
            return;
        }
        let message = self.fresh_var();
        self.emit_line(&format!(
            "const {message} = WF.choiceError({var}, {}_check);",
            sig(bound)
        ));
        self.emit_line(&format!("{parent}.appendChild({message});"));
    }

    pub(in super::super) fn emit_switch(
        &mut self,
        var: &str,
        attrs: &[String],
        ui: &UIElement,
        parent: &str,
    ) {
        // A store's member or a loop item's field binds as a state does; a
        // switch bound to one used to draw no input at all.
        let bind_var = self.bind_arg(ui);

        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let own = self.control_box_attrs("Switch", ui);
        self.emit_line(&format!(
            "const {} = WF.el(\"label\", {{ className: \"{}\"{}{} }});",
            var,
            self.class_attr("Switch", ui),
            own,
            self.wf_node_inline(ui)
        ));

        // A switch unless its author says what it is.
        let role = if Self::gives(ui, "role") {
            ""
        } else {
            " role: \"switch\","
        };
        let extra = self.control_extra_attrs("Switch", ui);
        if let Some(target) = &bind_var {
            let input_var = self.fresh_var();
            let read = &target.read;
            let write = Self::with_commit(target, &target.write(&format!("!{read}")));
            let handlers = self.control_handlers(ui, Some(write));
            self.emit_line(&format!(
                "const {} = WF.el(\"input\", {{ type: \"checkbox\",{role} checked: () => !!{read}, \"aria-checked\": () => {read} ? \"true\" : \"false\"{}{}{} }});",
                input_var,
                handlers,
                Self::control_input_attrs(attrs),
                extra
            ));
            self.emit_line(&format!("{var}.appendChild({input_var});"));
        } else if let Some(checked) = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "checked" => Some(v),
            _ => None,
        }) {
            // `checked:` without `bind:` — a value down, an event up: the
            // input shows the value, and the change is the caller's, heard
            // by `on change` on the switch as it bubbles.
            let value = self.emit_expr(checked);
            let input_var = self.fresh_var();
            let handlers = self.control_handlers(ui, None);
            self.emit_line(&format!(
                "const {} = WF.el(\"input\", {{ type: \"checkbox\",{role} checked: () => !!({value}), \"aria-checked\": () => ({value}) ? \"true\" : \"false\"{}{}{} }});",
                input_var,
                handlers,
                Self::control_input_attrs(attrs),
                extra
            ));
            self.emit_line(&format!("{var}.appendChild({input_var});"));
        }

        let track_var = self.fresh_var();
        self.emit_line(&format!(
            "const {track_var} = WF.el(\"span\", {{ className: \"wf-switch__track\" }}, WF.el(\"span\", {{ className: \"wf-switch__thumb\" }}));"
        ));
        self.emit_line(&format!("{var}.appendChild({track_var});"));

        if let Some(l) = label {
            // A label that reads state — `t("…")`, which follows the locale,
            // or a signal — is drawn live; it was worked out once, so a
            // `Checkbox` kept the language the page opened in.
            let l = if self.is_reactive(&l) {
                format!("() => {l}")
            } else {
                l
            };
            self.emit_line(&format!("{var}.appendChild(WF.text({l}));"));
        }

        self.emit_line(&format!("{parent}.appendChild({var});"));
        self.emit_choice_message(ui, var, parent);
    }

    pub(in super::super) fn emit_check_radio(
        &mut self,
        name: &str,
        var: &str,
        attrs: &[String],
        ui: &UIElement,
        parent: &str,
    ) {
        let input_type = if name == "Checkbox" {
            "checkbox"
        } else {
            "radio"
        };
        let wf = self.wf_node_inline(ui);

        let bind_var = self.bind_arg(ui);

        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let radio_value = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "value" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let own = self.control_box_attrs(name, ui);
        self.emit_line(&format!(
            "const {} = WF.el(\"label\", {{ className: \"{}\"{}{} }});",
            var,
            self.class_attr(name, ui),
            own,
            wf
        ));

        let input_var = self.fresh_var();
        let mut input_attrs = format!("type: \"{input_type}\"");
        input_attrs.push_str(&Self::control_input_attrs(attrs));
        // Radios bound to one state are one group: the arrow keys move
        // between them only when they share a `name`.
        if name == "Radio"
            && let Some(target) = &bind_var
            && let Some(group) = self.group_name(&target.read)
            && !attrs.iter().any(|a| a.starts_with("name:"))
        {
            input_attrs.push_str(&format!(", name: \"{group}\""));
        }

        let mut bound_change: Option<String> = None;
        if let Some(target) = &bind_var {
            let read = &target.read;
            if name == "Checkbox" {
                input_attrs.push_str(&format!(", checked: () => !!{read}"));
                bound_change = Some(Self::with_commit(
                    target,
                    &target.write(&format!("!{read}")),
                ));
            } else if let Some(val) = &radio_value {
                input_attrs.push_str(&format!(", checked: () => {read} === {val}"));
                bound_change = Some(Self::with_commit(target, &target.write(val)));
            }
        }

        // Handle checked prop (non-bind)
        let checked_val = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "checked" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        if bind_var.is_none()
            && let Some(cv) = checked_val
        {
            if self.is_reactive(&cv) {
                input_attrs.push_str(&format!(", checked: () => {cv}"));
            } else {
                input_attrs.push_str(&format!(", checked: {cv}"));
            }
        }

        // The rest of what any element takes, then the caller's handlers
        // and the binding's write, on the input.
        input_attrs.push_str(&self.control_extra_attrs(name, ui));
        input_attrs.push_str(&self.control_handlers(ui, bound_change));

        self.emit_line(&format!(
            "const {input_var} = WF.el(\"input\", {{ {input_attrs} }});"
        ));
        self.emit_line(&format!("{var}.appendChild({input_var});"));

        if let Some(l) = label {
            // A label that reads state — `t("…")`, which follows the locale,
            // or a signal — is drawn live; it was worked out once, so a
            // `Checkbox` kept the language the page opened in.
            let l = if self.is_reactive(&l) {
                format!("() => {l}")
            } else {
                l
            };
            self.emit_line(&format!("{var}.appendChild(WF.text({l}));"));
        }

        self.emit_line(&format!("{parent}.appendChild({var});"));
        if name == "Checkbox" {
            self.emit_choice_message(ui, var, parent);
        }
    }

    pub(in super::super) fn emit_slider(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let bind_var = self.bind_arg(ui);
        let min_val = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "min" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "0".to_string());
        let max_val = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "max" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "100".to_string());
        let step = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "step" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "1".to_string());
        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let own = self.control_box_attrs("Slider", ui);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            var,
            self.class_attr("Slider", ui),
            own,
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"label\", {{ className: \"wf-form-label\" }}, {});",
                label_var,
                self.live(l)
            ));
            self.emit_line(&format!("{var}.appendChild({label_var});"));
        }

        let input_var = self.fresh_var();
        let mut input_attrs =
            format!("type: \"range\", min: {min_val}, max: {max_val}, step: {step}");
        if let Some(target) = &bind_var {
            // An author's own on:input runs after the binding has written the
            // state, in one handler: two "on:input" keys in one attribute
            // object used to leave only the second.
            let own_input = ui
                .events
                .iter()
                .find(|h| h.event == "input")
                .map(|h| self.emit_event_body(h))
                .unwrap_or_default();
            let event = self.implicit_param("event");
            input_attrs.push_str(&format!(
                ", value: () => {}, \"on:input\": ({event}) => {{ {}; {} }}",
                target.read,
                target.write(&format!("Number({event}.target.value)")),
                own_input
            ));
            if let Some(commit) = &target.commit {
                input_attrs.push_str(&format!(", \"on:change\": () => {{ {commit}; }}"));
            }
        }
        // ARIA and data attributes reach the range input itself, which is the
        // control assistive technology reads. An author who announces the
        // value with aria-valuetext is showing it in their own words too, so
        // the raw number is not repeated beside the track.
        let mut announces_value = false;
        for arg in &ui.args {
            if let Arg::Named(k, v) = arg
                && k.contains('-')
            {
                let value = self.emit_expr(v);
                if k == "aria-valuetext" {
                    announces_value = true;
                }
                if self.is_reactive(&value) {
                    input_attrs.push_str(&format!(", \"{k}\": () => {value}"));
                } else {
                    input_attrs.push_str(&format!(", \"{k}\": {value}"));
                }
            }
        }
        // The rest of what any element takes — `id:`, `role:`, `tabindex:`,
        // `name:`, `disabled:`, a `ref:` — is the control's too, where its
        // handlers already go.
        let sig = crate::registry::component("Slider");
        input_attrs.push_str(
            &self.element_attrs(sig, ui, &|k| !k.contains('-') && !BOX_ATTRS.contains(&k)),
        );
        for handler in &ui.events {
            if handler.event == "input" && bind_var.is_some() {
                continue; // merged into the binding above
            }
            let body = self.emit_event_body(handler);
            input_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                self.handler_head(handler, "event"),
                body
            ));
        }
        self.emit_line(&format!(
            "const {input_var} = WF.el(\"input\", {{ {input_attrs} }});"
        ));
        self.emit_line(&format!("{var}.appendChild({input_var});"));

        // Show current value if bound
        if bind_var.is_some() && announces_value {
            // The author shows it.
        } else if let Some(target) = &bind_var {
            let val_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"span\", {{ className: \"wf-slider__value\" }}, () => String({}));",
                val_var, target.read
            ));
            self.emit_line(&format!("{var}.appendChild({val_var});"));
        }

        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    pub(in super::super) fn emit_datepicker(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let bind_var = self.bind_arg(ui);
        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let min = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "min" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let max = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "max" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Root on the component's own class, not the generic form-group: the
        // static renderers key on `wf-datepicker`, and hydration has to find the
        // same root they painted.
        let wrapper_var = self.fresh_var();
        let own = self.control_box_attrs("DatePicker", ui);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            wrapper_var,
            self.class_attr("DatePicker", ui),
            own,
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"label\", {{ className: \"wf-form-label\" }}, {});",
                label_var,
                self.live(l)
            ));
            self.emit_line(&format!("{wrapper_var}.appendChild({label_var});"));
        }

        let input_var = self.fresh_var();
        let mut input_attrs = "type: \"date\", className: \"wf-input\"".to_string();
        if let Some(target) = &bind_var {
            input_attrs.push_str(&format!(
                ", value: () => {} ?? \"\", \"on:change\": ($$e) => {{ {}; }}",
                target.read,
                Self::with_commit(target, &target.write("$$e.target.value || null"))
            ));
        }
        if let Some(mn) = min {
            input_attrs.push_str(&format!(", min: {mn}"));
        }
        if let Some(mx) = max {
            input_attrs.push_str(&format!(", max: {mx}"));
        }
        // What any element takes is the control's, where its handlers go.
        input_attrs.push_str(&self.root_attrs("DatePicker", ui, BOX_ATTRS));
        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            input_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                self.handler_head(handler, "event"),
                body
            ));
        }
        self.emit_line(&format!(
            "const {input_var} = WF.el(\"input\", {{ {input_attrs} }});"
        ));
        self.emit_line(&format!("{wrapper_var}.appendChild({input_var});"));

        self.emit_line(&format!("const {var} = {wrapper_var};"));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    pub(in super::super) fn emit_file_upload(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let accept = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "accept" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let multiple = ui.modifiers.iter().any(|m| m == "multiple");

        let wrapper_var = self.fresh_var();
        let own = self.control_box_attrs("FileUpload", ui);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            wrapper_var,
            self.class_attr("FileUpload", ui),
            own,
            self.wf_node_inline(ui)
        ));

        if let Some(l) = &label {
            let label_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"label\", {{ className: \"wf-form-label\" }}, {});",
                label_var,
                self.live(l)
            ));
            self.emit_line(&format!("{wrapper_var}.appendChild({label_var});"));
        }

        let input_var = self.fresh_var();
        let mut input_attrs = "type: \"file\", className: \"wf-input\"".to_string();
        if let Some(acc) = &accept {
            input_attrs.push_str(&format!(", accept: {acc}"));
        }
        if multiple {
            input_attrs.push_str(", multiple: true");
        }
        // What any element takes is the control's, where its handlers go.
        input_attrs.push_str(&self.root_attrs(
            "FileUpload",
            ui,
            &["multiple", "hidden", "lang", "dir"],
        ));
        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            input_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                self.handler_head(handler, "event"),
                body
            ));
        }
        self.emit_line(&format!(
            "const {input_var} = WF.el(\"input\", {{ {input_attrs} }});"
        ));
        self.emit_line(&format!("{wrapper_var}.appendChild({input_var});"));

        self.emit_line(&format!("const {var} = {wrapper_var};"));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// `validate email { required  email }`: the rules, registered against
    /// the state they guard and the form it is in.
    ///
    /// What comes back is the message to show, which the control picks up
    /// on its own — so a validated field needs no `error:` written on it.
    pub(in super::super) fn emit_validate(&mut self, v: &ValidateDecl) {
        let mut rules: Vec<String> = Vec::new();
        // A refined type validates itself: `state age: Number(18..=120)`
        // needs no rule written for the range it already declares.
        for (name, bound) in self.refinement_of(&v.name) {
            let rule = match name.as_str() {
                // `below`, `after` and `before` are their own: a bound the
                // value may not reach.
                "min" | "max" | "minLength" | "maxLength" | "pattern" | "below" | "after"
                | "before" => name.clone(),
                _ => continue,
            };
            rules.push(format!(
                "{{ name: \"{rule}\", args: [{}] }}",
                self.emit_expr(&bound)
            ));
        }
        for rule in &v.rules {
            let mut parts = vec![format!("name: \"{}\"", rule.name)];
            if !rule.args.is_empty() {
                let args: Vec<String> = rule
                    .args
                    .iter()
                    .map(|a| {
                        let emitted = self.emit_expr(a);
                        // A rule that names another value follows it.
                        if self.is_reactive(&emitted) {
                            format!("() => {emitted}")
                        } else {
                            emitted
                        }
                    })
                    .collect();
                parts.push(format!("args: [{}]", args.join(", ")));
            }
            if let Some(message) = &rule.message {
                parts.push(format!("message: () => {}", self.emit_expr(message)));
            }
            if let Some(body) = &rule.body {
                let check = self.emit_expr(body);
                parts.push(if rule.name == "async" {
                    format!("check: async () => {check}")
                } else {
                    format!("check: () => {check}")
                });
            }
            rules.push(format!("{{ {} }}", parts.join(", ")));
        }
        let form = match &self.body.current_form {
            Some(name) => format!(", form: {name}"),
            None => String::new(),
        };
        self.emit_line(&format!(
            "const {}_check = WF.validate(() => {}(), [{}], {{ name: \"{}\"{form} }});",
            sig(&v.name),
            sig(&v.name),
            rules.join(", "),
            v.name
        ));
    }
}
