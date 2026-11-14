//! The built-ins that draw their rows from data: `Tree`, `DataTable` and
//! `Combobox`. Each hands the runtime its data as reads, its `bind:` as a
//! read and a write, and the blocks the author wrote for a row
//! (`row(f) { … }`, `cell(r) { … }`, `option(o) { … }`, `empty { … }`) as
//! functions the runtime calls for each one it draws.

use super::super::{Arg, ComponentRef, Expr, JsCodegen, StatementKind, UIElement};

impl JsCodegen {
    /// The named argument `key` as JavaScript.
    fn named_js(&mut self, ui: &UIElement, key: &str) -> Option<String> {
        let value = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == key => Some(v.clone()),
            _ => None,
        })?;
        Some(self.emit_expr(&value))
    }

    /// Whether a `Bool` prop is given, as a flag or as `true`.
    pub(in super::super) fn flag_given(ui: &UIElement, key: &str) -> bool {
        ui.modifiers.iter().any(|m| m == key)
            || ui
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, Expr::BoolLiteral(true)) if k == key))
    }

    /// `bind:` as `read, write` for the runtime, or `null, null`.
    fn bind_pair(&self, ui: &UIElement) -> (String, String) {
        match self.bind_arg(ui) {
            Some(t) => (
                format!("() => {}", t.read),
                format!(
                    "(v) => {{ {} }}",
                    Self::with_commit(&t, &t.write.replace("$V", "v"))
                ),
            ),
            None => ("null".to_string(), "null".to_string()),
        }
    }

    /// The block `name(params) { … }` written on `ui`, emitted as a function
    /// of those params that draws it and hands back its nodes; `None` where
    /// it is not written.
    fn fill_function(&mut self, ui: &UIElement, name: &str) -> Option<String> {
        let fill = ui.slot_fills.iter().find(|f| f.name == name)?.clone();
        let var = self.fresh_var();
        let params = fill.params.join(", ");
        self.emit_line(&format!("const {var} = ({params}) => {{"));
        self.indent += 1;
        let bound = self.loop_bindings.len();
        self.loop_bindings.extend(fill.params.iter().cloned());
        let frag = self.fresh_var();
        self.emit_line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        for stmt in &fill.body {
            self.emit_statement_dom(stmt, &frag);
        }
        self.emit_line(&format!("return {frag};"));
        self.loop_bindings.truncate(bound);
        self.indent -= 1;
        self.emit_line("};");
        Some(var)
    }

    /// The root element of one of these, with its own and the author's
    /// attributes.
    fn collection_root(&mut self, name: &str, tag: &str, var: &str, ui: &UIElement) {
        let extra = self.root_attrs(name, ui, &[]);
        self.emit_line(&format!(
            "const {var} = WF.el(\"{tag}\", {{ className: \"{}\"{extra}{} }});",
            self.class_attr(name, ui),
            self.wf_node_inline(ui)
        ));
        self.emit_root_handlers(var, ui, false);
    }

    /// A control the runtime draws whole, its `bind:` handed as a read and a
    /// write and each of `keys` as a read of what it was given (`true` for a
    /// flag): `WF.rating(box, read, write, { max: () => 5, half: () => true })`.
    pub(in super::super) fn emit_bound_widget(
        &mut self,
        var: &str,
        ui: &UIElement,
        parent: &str,
        name: &str,
        func: &str,
        keys: &[&str],
    ) {
        let tag = crate::registry::component(name).map_or("div", |c| c.tag);
        self.collection_root(name, tag, var, ui);
        let (read, write) = self.bind_pair(ui);
        let mut opts = Vec::new();
        for key in keys {
            if let Some(v) = self.named_js(ui, key) {
                opts.push(format!("{key}: () => {v}"));
            } else if Self::flag_given(ui, key) {
                opts.push(format!("{key}: () => true"));
            }
        }
        self.emit_line(&format!(
            "WF.{func}({var}, {read}, {write}, {{ {} }});",
            opts.join(", ")
        ));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// `Tree(items:, children:, key:, bind:, label:) { row(f) { … } }`.
    pub(in super::super) fn emit_tree(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let row = self.fill_function(ui, "row");
        self.collection_root("Tree", "ul", var, ui);
        let (read, write) = self.bind_pair(ui);
        let items = self.named_js(ui, "items").unwrap_or_else(|| "[]".into());
        let children = self
            .named_js(ui, "children")
            .unwrap_or_else(|| "null".into());
        let key = self.named_js(ui, "key").unwrap_or_else(|| "null".into());
        let open = self.named_js(ui, "open").unwrap_or_else(|| "[]".into());
        let label = self
            .named_js(ui, "label")
            .unwrap_or_else(|| "\"Tree\"".into());
        self.emit_line(&format!(
            "WF.tree({var}, {{ items: () => {items}, children: {children}, key: {key}, open: () => {open}, read: {read}, write: {write}, multiple: {}, label: () => {label}, row: {} }});",
            Self::flag_given(ui, "multiple"),
            row.unwrap_or_else(|| "null".into())
        ));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// `DataTable(rows:, key:, label:) { DataTable.Column(…) { cell(r) { … } }  empty { … } }`.
    pub(in super::super) fn emit_data_table(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let mut columns = Vec::new();
        for child in &ui.children {
            let StatementKind::UIElement(col) = &child.kind else {
                continue;
            };
            if !matches!(&col.component, ComponentRef::SubComponent(p, s) if p == "DataTable" && s == "Column")
            {
                continue;
            }
            let title = col
                .args
                .iter()
                .find_map(|a| match a {
                    Arg::Positional(e) => Some(e.clone()),
                    Arg::Named(..) => None,
                })
                .map_or_else(|| "\"\"".to_string(), |e| self.emit_expr(&e));
            let value = self.named_js(col, "value").unwrap_or_else(|| "null".into());
            let cell = self
                .fill_function(col, "cell")
                .unwrap_or_else(|| "null".into());
            columns.push(format!(
                "{{ title: () => {title}, value: {value}, sortable: {}, cell: {cell} }}",
                Self::flag_given(col, "sortable")
            ));
        }
        let empty = self
            .fill_function(ui, "empty")
            .unwrap_or_else(|| "null".into());
        self.collection_root("DataTable", "div", var, ui);
        let (read, write) = self.bind_pair(ui);
        let rows = self.named_js(ui, "rows").unwrap_or_else(|| "[]".into());
        let key = self.named_js(ui, "key").unwrap_or_else(|| "null".into());
        let label = self.named_js(ui, "label").unwrap_or_else(|| "\"\"".into());
        self.emit_line(&format!(
            "WF.dataTable({var}, {{ rows: () => {rows}, key: {key}, read: {read}, write: {write}, multiple: {}, label: () => {label}, columns: [{}], empty: {empty} }});",
            Self::flag_given(ui, "multiple"),
            columns.join(", ")
        ));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// `Combobox(bind:, options: | search:, text:, key:, label:) { option(o) { … }  empty { … } }`.
    pub(in super::super) fn emit_combobox(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let option = self
            .fill_function(ui, "option")
            .unwrap_or_else(|| "null".into());
        let empty = self
            .fill_function(ui, "empty")
            .unwrap_or_else(|| "null".into());
        self.collection_root("Combobox", "div", var, ui);
        let (read, write) = self.bind_pair(ui);
        let options = self
            .named_js(ui, "options")
            .map_or_else(|| "null".into(), |o| format!("() => {o}"));
        let search = self.named_js(ui, "search").unwrap_or_else(|| "null".into());
        let text = self.named_js(ui, "text").unwrap_or_else(|| "null".into());
        let key = self.named_js(ui, "key").unwrap_or_else(|| "null".into());
        let debounce = self
            .named_js(ui, "debounce")
            .unwrap_or_else(|| "200".into());
        let label = self.named_js(ui, "label").unwrap_or_else(|| "\"\"".into());
        let placeholder = self
            .named_js(ui, "placeholder")
            .unwrap_or_else(|| "\"\"".into());
        self.emit_line(&format!(
            "WF.combobox({var}, {{ options: {options}, search: {search}, text: {text}, key: {key}, read: {read}, write: {write}, multiple: {}, debounce: {debounce}, label: () => {label}, placeholder: () => {placeholder}, option: {option}, empty: {empty} }});",
            Self::flag_given(ui, "multiple")
        ));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }
}
