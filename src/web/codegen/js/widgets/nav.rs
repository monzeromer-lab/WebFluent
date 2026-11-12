//! Navigation: tabs, the sidebar and breadcrumbs.

use super::super::*;

impl JsCodegen {
    pub(in super::super) fn emit_tabs(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let extra = self.root_attrs("Tabs", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            var,
            self.class_attr("Tabs", ui),
            extra,
            self.wf_node_inline(ui)
        ));
        // `role="tablist"` and the tab/panel wiring below are what tell a screen
        // reader these buttons are tabs at all. Without them the widget is a row
        // of unrelated buttons next to unrelated divs.
        let nav_var = self.fresh_var();
        self.emit_line(&format!(
            "const {nav_var} = WF.el(\"div\", {{ className: \"wf-tabs__nav\", role: \"tablist\" }});"
        ));
        let group = var.trim_start_matches("_e").to_string();

        // Collect tab pages
        let tab_pages: Vec<(&UIElement, usize)> = ui
            .children
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if let StatementKind::UIElement(ui_child) = &s.kind
                    && matches!(&ui_child.component, ComponentRef::BuiltIn(n) if n == "TabPage")
                {
                    return Some((ui_child, i));
                }
                None
            })
            .collect();

        let active_var = self.fresh_var();
        self.emit_line(&format!("const {active_var} = WF.signal(0);"));

        // Create tab buttons
        for (i, (tab, _)) in tab_pages.iter().enumerate() {
            let label = tab.args.first().map_or_else(
                || format!("\"Tab {i}\""),
                |a| {
                    if let Arg::Positional(expr) = a {
                        self.emit_expr(expr)
                    } else {
                        format!("\"Tab {i}\"")
                    }
                },
            );
            let label = self.live(&label);

            // Roving tabindex: only the selected tab is in the tab order, and
            // the arrow keys move between them (see `WF.tabs`).
            let btn_var = self.fresh_var();
            self.emit_line(&format!(
                "const {btn_var} = WF.el(\"button\", {{ className: () => {active_var}() === {i} ? \"wf-tabs__tab active\" : \"wf-tabs__tab\",                  role: \"tab\", type: \"button\", id: \"wf-tab-{group}-{i}\",                  \"aria-controls\": \"wf-tabpanel-{group}-{i}\",                  \"aria-selected\": () => {active_var}() === {i} ? \"true\" : \"false\",                  tabindex: () => {active_var}() === {i} ? 0 : -1,                  \"on:click\": () => {active_var}.set({i}) }}, {label});"
            ));
            self.emit_line(&format!("{nav_var}.appendChild({btn_var});"));
        }

        self.emit_line(&format!("{var}.appendChild({nav_var});"));

        // Create tab content
        for (i, (tab, _)) in tab_pages.iter().enumerate() {
            let page_var = self.fresh_var();
            self.emit_line(&format!(
                "const {} = WF.el(\"div\", {{ className: \"wf-tab-page\", role: \"tabpanel\",                  id: \"wf-tabpanel-{}-{}\", \"aria-labelledby\": \"wf-tab-{}-{}\", tabindex: 0{} }});",
                page_var, group, i, group, i, self.wf_node_inline(tab)
            ));
            for child in &tab.children {
                self.emit_statement_dom(child, &page_var);
            }
            self.emit_line(&format!(
                "WF.effect(() => {{ {page_var}.style.display = {active_var}() === {i} ? 'block' : 'none'; }});"
            ));
            self.emit_line(&format!("{var}.appendChild({page_var});"));
        }

        // Arrow keys, Home and End move between tabs, which is what the WAI-ARIA
        // pattern requires and what a keyboard user will try.
        self.emit_line(&format!("WF.tabs({nav_var}, {active_var});"));

        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    pub(in super::super) fn emit_sidebar(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let sidebar_id = var.trim_start_matches("_e").to_string();
        // The panel is named by its own id, which the toggle below points
        // at — the author's `id:`, when there is one, in place of the one
        // made for it.
        let own_id = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "id" => Some(v),
            _ => None,
        });
        let (id_attr, controls) = match own_id {
            Some(v) => {
                let v = self.emit_expr(v);
                (String::new(), self.live(&v))
            }
            None => (
                format!(", id: \"wf-sidebar-{sidebar_id}\""),
                format!("\"wf-sidebar-{sidebar_id}\""),
            ),
        };
        let extra = self.root_attrs("Sidebar", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"aside\", {{ className: \"{}\"{}{}{} }});",
            var,
            self.class_attr("Sidebar", ui),
            id_attr,
            extra,
            self.wf_node_inline(ui)
        ));

        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                match &ui_child.component {
                    ComponentRef::SubComponent(p, sub) if p == "Sidebar" => match sub.as_str() {
                        "Header" => {
                            let h_var = self.fresh_var();
                            self.emit_line(&format!(
                                    "const {} = WF.el(\"div\", {{ className: \"wf-sidebar__header\"{} }});",
                                    h_var, self.wf_node_inline(ui_child)
                                ));
                            for c in &ui_child.children {
                                self.emit_statement_dom(c, &h_var);
                            }
                            self.emit_line(&format!("{var}.appendChild({h_var});"));
                        }
                        "Item" => {
                            let item_var = self.fresh_var();
                            let to = ui_child.args.iter().find_map(|a| {
                                if let Arg::Named(k, v) = a {
                                    if k == "to" {
                                        Some(self.emit_expr(v))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });
                            let icon = ui_child.args.iter().find_map(|a| {
                                if let Arg::Named(k, v) = a {
                                    if k == "icon" {
                                        Some(self.emit_expr(v))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });
                            if let Some(href) = to {
                                let bp = if self.base_path.is_empty() {
                                    String::new()
                                } else {
                                    "WF._basePath + ".to_string()
                                };
                                // In the SPA the item navigates in place, as a
                                // Link does; it used to be a plain href, so
                                // every rail click reloaded the whole app.
                                let click = if self.ssg_mode {
                                    String::new()
                                } else {
                                    format!(
                                        ", \"on:click\": ($$e) => {{ $$e.preventDefault(); WF.navigate({href}); }}"
                                    )
                                };
                                self.emit_line(&format!(
                                        "const {} = WF.el(\"a\", {{ className: \"wf-sidebar__item\", href: {} {}{}{} }});",
                                        item_var, bp, href, click, self.wf_node_inline(ui_child)
                                    ));
                                let prefix = ui_child.args.iter().any(|a| {
                                    matches!(a, Arg::Named(k, Expr::StringLiteral(v)) if k == "active" && v == "prefix")
                                });
                                self.emit_line(&active_link(&item_var, &href, prefix));
                            } else {
                                self.emit_line(&format!(
                                        "const {} = WF.el(\"div\", {{ className: \"wf-sidebar__item\"{} }});",
                                        item_var, self.wf_node_inline(ui_child)
                                    ));
                            }
                            if let Some(ic) = icon {
                                self.emit_line(&format!(
                                        "{item_var}.appendChild(WF.el(\"span\", {{ className: \"wf-icon\", \"data-icon\": {ic} }}));"
                                    ));
                            }
                            for c in &ui_child.children {
                                self.emit_statement_dom(c, &item_var);
                            }
                            self.emit_line(&format!("{var}.appendChild({item_var});"));
                        }
                        "Divider" => {
                            self.emit_line(&format!(
                                    "{}.appendChild(WF.el(\"div\", {{ className: \"wf-sidebar__divider\"{} }}));",
                                    var, self.wf_node_inline(ui_child)
                                ));
                        }
                        _ => {
                            self.emit_statement_dom(child, var);
                        }
                    },
                    _ => {
                        self.emit_statement_dom(child, var);
                    }
                }
            } else {
                self.emit_statement_dom(child, var);
            }
        }

        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));

        // Below the sidebar breakpoint the panel slides over the page, so it
        // needs something to open it — without a control the 768px rule simply
        // removed a site's navigation on a phone. The control follows the panel
        // so the panel stays the component's root; CSS fixes it to the corner on
        // a narrow screen and hides it on a wide one.
        let scrim_var = self.fresh_var();
        let toggle_var = self.fresh_var();
        self.emit_line(&format!(
            "const {scrim_var} = WF.el(\"div\", {{ className: \"wf-sidebar__scrim\", hidden: true }});"
        ));
        self.emit_line(&format!(
            "const {toggle_var} = WF.el(\"button\", {{ className: \"wf-sidebar__toggle\", type: \"button\", \"aria-label\": \"Open navigation\", \"aria-expanded\": \"false\", \"aria-controls\": {controls} }}, \"\\u2630\");"
        ));
        self.emit_line(&format!("{parent}.appendChild({scrim_var});"));
        self.emit_line(&format!("{parent}.appendChild({toggle_var});"));
        self.emit_line(&format!("WF.drawer({var}, {toggle_var}, {scrim_var});"));
    }

    pub(in super::super) fn emit_breadcrumb(&mut self, var: &str, ui: &UIElement, parent: &str) {
        // The trail is named "breadcrumb" unless its author names it.
        let named = if Self::gives(ui, "aria-label") {
            ""
        } else {
            ", \"aria-label\": \"breadcrumb\""
        };
        let extra = self.root_attrs("Breadcrumb", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"nav\", {{ className: \"{}\"{}{}{} }});",
            var,
            self.class_attr("Breadcrumb", ui),
            named,
            extra,
            self.wf_node_inline(ui)
        ));

        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                if matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == "Breadcrumb" && s == "Item")
                {
                    let item_var = self.fresh_var();
                    let to = ui_child.args.iter().find_map(|a| {
                        if let Arg::Named(k, v) = a {
                            if k == "to" {
                                Some(self.emit_expr(v))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    });
                    if let Some(href) = to {
                        let bp = if self.base_path.is_empty() {
                            String::new()
                        } else {
                            "WF._basePath + ".to_string()
                        };
                        self.emit_line(&format!(
                            "const {} = WF.el(\"a\", {{ className: \"wf-breadcrumb__item\", href: {}{}{} }});",
                            item_var, bp, href, self.wf_node_inline(ui_child)
                        ));
                    } else {
                        self.emit_line(&format!(
                            "const {} = WF.el(\"span\", {{ className: \"wf-breadcrumb__item\"{} }});",
                            item_var, self.wf_node_inline(ui_child)
                        ));
                    }
                    for c in &ui_child.children {
                        self.emit_statement_dom(c, &item_var);
                    }
                    self.emit_line(&format!("{var}.appendChild({item_var});"));
                } else {
                    self.emit_statement_dom(child, var);
                }
            } else {
                self.emit_statement_dom(child, var);
            }
        }

        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }
}
