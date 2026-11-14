//! The collection built-ins: `Listbox` and `CommandPalette` drawn as other
//! built-ins, `Tree`, `DataTable` and `Combobox` drawn by the runtime from
//! their data with the blocks the author wrote for a row, `AppShell` a
//! frame, and the named blocks a built-in takes. What a real browser does
//! with them is `tests/browser/collections.mjs`.

use crate::common::{page, spa_generated, ssg_html, template_html};
use webfluent::diagnostics::Diagnostic;

/// Every finding for `src`, through the build's pipeline.
fn findings(src: &str) -> Vec<Diagnostic> {
    let (program, mut out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
    out.extend(
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics,
    );
    out
}

/// The findings with `code`, as `line: message`.

#[test]
fn a_listbox_is_options_its_binding_chooses() {
    let src = page(
        "state fruit = \"b\"\n    Listbox(bind: fruit, label: \"Fruit\") {\n        Listbox.Option(\"A\", value: \"a\")\n        Listbox.Option(\"B\", value: \"b\")\n    }",
    );
    for html in [ssg_html(&src), template_html(&src)] {
        assert!(
            html.contains("aria-label=\"Fruit\" role=\"listbox\""),
            "{html}"
        );
        assert!(
            html.contains("role=\"option\" aria-selected=\"true\">B</div>"),
            "{html}"
        );
    }
    assert!(spa_generated(&src).contains("WF.listbox(n, false)"));
}

#[test]
fn a_tree_a_table_and_a_combobox_are_drawn_from_their_data() {
    let src = page(
        "state file = \"\"\n    state rows = [{ id: \"1\", name: \"a\" }]\n    state who = \"\"\n    Tree(items: [], children: f => f.kids, key: f => f.id, bind: file, label: \"Files\") { row(f) { Text(f.name) } }\n    DataTable(rows: rows, key: r => r.id, label: \"Rows\") {\n        empty { Text(\"None\") }\n        DataTable.Column(\"Name\", value: r => r.name).sortable\n        DataTable.Column(\"Id\") { cell(r) { Badge(r.id) } }\n    }\n    Combobox(bind: who, options: [\"x\"], text: p => p, label: \"Who\") { option(o) { Text(o) } }",
    );
    let js = spa_generated(&src);
    for call in [
        "WF.tree(",
        "WF.dataTable(",
        "WF.combobox(",
        "row: _e",
        "cell: _e",
        "option: _e",
        "sortable: true",
    ] {
        assert!(js.contains(call), "{call}: {js}");
    }
    let html = ssg_html(&src);
    assert!(
        html.contains("<ul class=\"wf-tree\"></ul>"),
        "nothing painted to take back: {html}"
    );
    assert!(
        html.contains("<div class=\"wf-data-table\"></div>"),
        "{html}"
    );
    assert!(
        findings(&src)
            .iter()
            .all(|d| d.code != "E103" && d.code != "T13"),
        "{:?}",
        findings(&src)
    );
}

#[test]
fn a_built_in_names_the_blocks_it_takes() {
    let src = page(
        "state f = \"\"\n    Tree(items: [], children: x => x.kids, key: x => x.id, label: \"T\") { rows(x) { Text(x) } }",
    );
    let found: Vec<Diagnostic> = findings(&src)
        .into_iter()
        .filter(|d| d.code == "E103")
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0]
            .hint
            .as_deref()
            .unwrap_or_default()
            .contains("row { … }"),
        "{found:?}"
    );
}

#[test]
fn a_command_palette_is_a_modal_of_a_field_and_its_items() {
    let src = page(
        "state open = false\n    CommandPalette(open: open) {\n        CommandPalette.Group(\"Pages\") { CommandPalette.Item(\"Home\") { on select { log(1) } } }\n    }",
    );
    let html = ssg_html(&src);
    assert!(html.contains("<dialog class=\"wf-modal\""), "{html}");
    assert!(
        html.contains("role=\"group\" aria-label=\"Pages\""),
        "{html}"
    );
    assert!(html.contains("role=\"option\""), "{html}");
    let js = spa_generated(&src);
    assert!(js.contains("addEventListener(\"select\""), "{js}");
    assert!(js.contains("WF.commandPalette(n)"), "{js}");
}

#[test]
fn an_app_shell_frames_the_page() {
    let src = "app {\n    AppShell {\n        AppShell.Header { Text(\"Brand\") }\n        Sidebar { Sidebar.Item(to: \"/\") { Text(\"Home\") } }\n        Router\n    }\n}\npage Home(path: \"/\", title: \"T\") { Heading(\"Home\").h1 }\n";
    let html = ssg_html(src);
    assert!(html.contains("<div class=\"wf-app-shell\">"), "{html}");
    assert!(
        html.contains("<header class=\"wf-app-shell__header\">"),
        "{html}"
    );
}
