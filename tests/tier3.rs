//! The third tier of built-ins and the blocks: what the static paint holds
//! of each built-in, what the runtime is handed, and every block a page
//! that builds with nothing to report, in either layout. What a real browser
//! does with the built-ins is `tests/browser/tier3.mjs`.

use crate::common::{page, spa_generated, ssg_html};
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

#[test]
fn the_static_paint_holds_what_it_can_and_leaves_the_rest() {
    let html = ssg_html(&page(
        "Highlight(\"Find me, then find me again\", query: \"find\")\n    ClientOnly { Text(\"clock\") }\n    Fab(icon: \"plus\", label: \"New\") { Fab.Action(\"Note\", icon: \"file\") }\n    BackToTop",
    ));
    assert!(
        html.contains("<span class=\"wf-highlight\"><mark>Find</mark> me, then <mark>find</mark> me again</span>"),
        "{html}"
    );
    assert!(
        !html.contains("clock"),
        "a client-only block is not painted: {html}"
    );
    assert!(
        html.contains("<button class=\"wf-fab__action\" hidden>"),
        "{html}"
    );
    assert!(
        html.contains("<button class=\"wf-fab__trigger\" aria-label=\"New\" title=\"New\">"),
        "{html}"
    );
    assert!(
        html.contains("<button class=\"wf-back-to-top\" aria-label=\"Back to top\" title=\"Back to top\" hidden>"),
        "{html}"
    );
}

#[test]
fn the_runtime_is_handed_each_control() {
    let js = spa_generated(&page(
        "state c = \"#000000\"\n    state picked: [String] = []\n    state t = \"x\"\n    state heard = \"\"\n    ColorPicker(bind: c, label: \"Accent\", swatches: [\"#ff0000\"])\n    TransferList(bind: picked, options: [\"a\", \"b\"], label: \"Letters\")\n    Editable(bind: t, label: \"Name\")\n    Popconfirm(title: \"Sure?\") { on confirm { heard = \"yes\" }  Button(\"Go\") }\n    Highlight(heard, query: t)\n    Text(heard)",
    ));
    for call in [
        "WF.colorPicker(",
        "WF.transferList(",
        "WF.editable(",
        "WF.popconfirm(n, (() => \"Sure?\")",
        "addEventListener(\"confirm\"",
        "WF.markMatches(",
    ] {
        assert!(js.contains(call), "{call}: {js}");
    }
}

#[test]
fn every_block_builds_with_nothing_to_report() {
    for (name, _, _) in webfluent::generate::BLOCKS {
        let (_, text) = webfluent::generate::block(name).unwrap();
        let found: Vec<String> = findings(text)
            .into_iter()
            .map(|d| format!("{}: {} {}", d.line, d.code, d.message))
            .collect();
        assert!(found.is_empty(), "{name}: {found:?}");
        let indented = webfluent::syntax::layout::to_offside(text, &format!("{name}.wfx"))
            .unwrap_or_else(|e| panic!("{name} in the indented layout: {e}"));
        webfluent::syntax::parse_source(&indented, &format!("{name}.wfx"))
            .unwrap_or_else(|e| panic!("{name}.wfx: {e}"));
    }
}
