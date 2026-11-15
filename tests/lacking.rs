//! What the built-ins lacked at the end of the plan: marked lines and a tab
//! a file in a `CodeBlock`, a `Stat`'s trend, a `ClientOnly` placeholder,
//! columns per breakpoint, a `Tour`'s and a `Lightbox`'s own words, a
//! `DataTable`'s filter and pages, a panel that folds, and a group's boxes
//! drawn by a `for`. What a real browser does with them is
//! `tests/browser/leftovers.mjs`.

use crate::common::{page, spa_generated, ssg_html};

#[test]
fn a_code_block_marks_lines_and_makes_a_tab_a_file() {
    let html = ssg_html(&page(
        "CodeBlock(\"a\\nb\\nc\", highlight: [2])\n    CodeBlock(files: [{ name: \"a.wf\", code: \"x\" }, { name: \"b.wf\", code: \"y\" }])",
    ));
    assert!(
        html.contains("<span class=\"wf-code-block__mark\" data-on=\"true\">"),
        "{html}"
    );
    assert_eq!(html.matches("wf-code-block__mark\"").count(), 3, "{html}");
    assert!(
        html.contains(">a.wf</button>") && html.contains(">b.wf</button>"),
        "{html}"
    );
}

#[test]
fn a_stat_has_its_trend_and_client_only_its_placeholder() {
    let html = ssg_html(&page(
        "Stat(\"Sign-ups\", value: 3, trend: [1, 2, 3])\n    ClientOnly { placeholder { Text(\"Soon\") }  Text(\"Now\") }",
    ));
    assert!(html.contains("<polyline"), "{html}");
    assert!(
        html.contains(">Soon</p>") && !html.contains(">Now</p>"),
        "{html}"
    );
}

#[test]
fn a_table_is_painted_a_page_at_a_time_and_filtered() {
    let html = ssg_html(&page(
        "state rows = [{ id: \"1\", n: \"web\" }, { id: \"2\", n: \"api\" }, { id: \"3\", n: \"cron\" }]\n    state q = \"\"\n    DataTable(rows: rows, key: r => r.id, label: \"S\", filter: q, pageSize: 2) { DataTable.Column(\"N\", value: r => r.n) }\n    Button(\"x\") { on click { q = \"a\" } }",
    ));
    assert!(
        html.contains("<td>web</td>") && !html.contains("<td>cron</td>"),
        "{html}"
    );
    assert!(html.contains("1\u{2013}2 of 3"), "{html}");
}

#[test]
fn words_columns_folding_and_options_in_a_for_reach_the_page() {
    let js = spa_generated(&page(
        "state open = false\n    state sizes: [Number] = [50, 50]\n    state picked: [String] = []\n    Button(\"Go\", ref: go) { on click { open = true } }\n    Tour(open: open, next: \"Weiter\") { Tour.Step(target: go) { Text(\"x\") } }\n    Lightbox(close: \"Schließen\") { Image(src: \"https://example.com/a.jpg\", alt: \"A\") }\n    PanelGroup(bind: sizes) { PanelGroup.Panel(collapsible: true) { Text(\"a\") }  PanelGroup.Panel { Text(\"b\") } }\n    CheckboxGroup(bind: picked, label: \"P\") { for p in [\"a\", \"b\"] by p { CheckboxGroup.Option(p) } }\n    Gallery(columns: { base: 1, md: 3 }) { Text(\"g\") }",
    ));
    for want in [
        "next: \"Weiter\"",
        "close: \"Schließen\"",
        "\"data-collapsible\": \"true\"",
        "includes($p())",
    ] {
        assert!(js.contains(want), "{want}: {js}");
    }
}
