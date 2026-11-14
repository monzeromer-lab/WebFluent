//! Overlays: `Modal`, `Dialog`, `AlertDialog`, `Sheet` and `Popover` on
//! one path — what closes each (`dismiss:`), what takes the focus
//! (`initialFocus:`), what a popover's control says — and the language's
//! `toast(…)`, `toast.promise(…)`, `await ask(…)` and `announce(…)`.
//! What a real browser does with them is `tests/browser/overlays.mjs`; the
//! toast and the live regions are `tests/js/overlays.test.mjs`.

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

/// The findings with `code`, as `line: message`.
fn with(src: &str, code: &str) -> Vec<String> {
    findings(src)
        .into_iter()
        .filter(|d| d.code == code)
        .map(|d| format!("{}: {}", d.line, d.message))
        .collect()
}

#[test]
fn a_modal_says_what_closes_it_and_what_takes_the_focus() {
    let src = page(
        "state editing = false\n    state name = \"\"\n    Modal(visible: editing, title: \"Edit\", dismiss: .none, initialFocus: field) {\n        Input(bind: name, label: \"Name\", ref: field)\n    }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("{ dismiss: \"none\", initial: () => field });"),
        "{js}"
    );
    assert!(!js.contains("\"dismiss\""), "not an attribute: {js}");
}

#[test]
fn an_alert_dialog_is_one_and_is_answered_not_dismissed() {
    let src = page(
        "state sure = false\n    AlertDialog(visible: sure, title: \"Discard?\") {\n        Text(\"Edits are lost.\")\n        AlertDialog.Footer { Button(\"Keep\") { on click { sure = false } } }\n    }",
    );
    let js = spa_generated(&src);
    assert!(js.contains("role: \"alertdialog\""), "{js}");
    assert!(
        js.contains("\"aria-describedby\""),
        "its body is read out: {js}"
    );
    assert!(
        js.contains("{ dismiss: \"escape\" });"),
        "a press beside it is no answer: {js}"
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<dialog class=\"wf-dialog wf-alert-dialog\"")
            && html.contains("role=\"alertdialog\""),
        "{html}"
    );
    assert!(html.contains("class=\"wf-dialog__footer\""), "{html}");
}

#[test]
fn a_sheet_comes_from_its_edge_with_a_handle_and_its_snap_points() {
    let src = page(
        "state open = false\n    Sheet(visible: open, title: \"Filters\", snap: [0.5, 1]) {\n        Text(\"Sort\")\n        Sheet.Footer { Button(\"Show\") { on click { open = false } } }\n    }\n    Sheet(visible: open, side: .end, label: \"Details\") { Text(\"x\") }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("WF.sheet(") && js.contains("{ side: \"bottom\", snap: [0.5, 1] });"),
        "{js}"
    );
    assert!(js.contains("{ side: \"end\" });"), "{js}");
    assert!(js.contains("\"aria-label\": \"Details\""), "{js}");
    assert_eq!(
        js.matches("wf-sheet__handle").count(),
        1,
        "a bottom sheet has a handle; a side sheet does not: {js}"
    );
    let html = ssg_html(&src);
    assert!(html.contains("data-side=\"bottom\""), "{html}");
    assert!(
        html.contains("data-side=\"end\" aria-label=\"Details\""),
        "{html}"
    );
    assert!(
        html.contains("<div class=\"wf-sheet__handle\" aria-hidden=\"true\"></div>"),
        "{html}"
    );
    assert!(html.contains("class=\"wf-sheet__footer\""), "{html}");
}

#[test]
fn a_popovers_control_says_what_it_opens() {
    let src = page(
        "state open = false\n    Button(\"Branches\", ref: b) { on click { open = !open } }\n    Popover(visible: open, anchor: b, label: \"Branches\", dismiss: .escape) { Button(\"main\") }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("dismiss: \"escape\", controls: true });"),
        "{js}"
    );
}

#[test]
fn toast_ask_and_announce_are_the_languages_unless_the_program_has_its_own() {
    let src = page(
        "state said = \"\"\n    action undo() { said = \"undone\" }\n    Button(\"a\") { on click { toast(\"Archived\", action: \"Undo\", onAction: undo) } }\n    Button(\"b\") { on click { if await ask(\"Delete?\", danger: true) { said = \"yes\" } } }\n    Button(\"c\") { on click { announce(\"Lost\", .assertive) } }\n    Button(\"d\") { on click { toast.promise(fetch(\"/x\"), loading: \"Saving\", done: \"Saved\") } }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("WF.toast(\"Archived\", ({ action: \"Undo\", onAction: undo }))"),
        "{js}"
    );
    assert!(js.contains("await WF.ask(\"Delete?\""), "{js}");
    assert!(js.contains("WF.announce(\"Lost\", \"assertive\")"), "{js}");
    assert!(js.contains("WF.toastPromise("), "{js}");
    assert!(
        findings(&src).iter().all(|d| d.code != "T13"),
        "{:?}",
        findings(&src)
    );

    // A program's own `toast` is its own.
    let own = page(
        "state toast = \"\"\n    Button(\"a\") { on click { toast = \"hi\" } }\n    Text(toast)",
    );
    assert!(
        findings(&own).iter().all(|d| d.code != "E118"),
        "{:?}",
        findings(&own)
    );
    assert!(!spa_generated(&own).contains("WF.toast("));
}

#[test]
fn every_dialog_needs_a_name_and_a_popover_drawn_as_one_too() {
    for body in [
        "state o = false\n    AlertDialog(visible: o) { Text(\"x\") }",
        "state o = false\n    Sheet(visible: o) { Text(\"x\") }",
    ] {
        assert_eq!(with(&page(body), "A08").len(), 1, "{body}");
    }
    assert_eq!(
        with(
            &page("state o = false\n    Sheet(visible: o, label: \"Filters\") { Text(\"x\") }"),
            "A08"
        ),
        Vec::<String>::new()
    );
    assert_eq!(
        with(
            &page("state o = false\n    Popover(visible: o) { Button(\"x\") }"),
            "A18"
        )
        .len(),
        1
    );
    for named in [
        "Popover(visible: o, label: \"Branches\") { Button(\"x\") }",
        "Popover(visible: o, role: \"listbox\", aria-label: \"Pick\") { Button(\"x\") }",
        "Popover(visible: o, role: \"tooltip\") { Text(\"x\") }",
    ] {
        assert!(
            with(&page(&format!("state o = false\n    {named}")), "A18").is_empty(),
            "{named}"
        );
    }
}

#[test]
fn calling_toast_or_ask_keeps_what_they_draw_with() {
    let css = |body: &str| {
        let program = webfluent::sema::lower(webfluent::parse_source(&page(body), "t.wf").unwrap());
        webfluent::themes::prune_css(
            webfluent::themes::component_css(),
            &webfluent::themes::Usage::of(&program),
        )
    };
    let plain = css("Text(\"hi\")");
    assert!(
        !plain.contains(".wf-toast {")
            && !plain.contains(".wf-alert-dialog")
            && !plain.contains(".wf-dialog__footer")
    );
    assert!(css("Button(\"a\") { on click { toast(\"x\") } }").contains(".wf-toast__action"));
    let asked = css("Button(\"a\") { on click { if await ask(\"x\") { log(1) } } }");
    assert!(
        asked.contains(".wf-dialog__footer") && asked.contains(".wf-btn--danger"),
        "{asked}"
    );
}
