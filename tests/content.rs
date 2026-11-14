//! The content built-ins, which the compiler draws as others: what each is
//! drawn as on the live page, in the static paint and by the template
//! engine — one definition for all three — and what a PDF does with them.
//! What a real browser does with them is `tests/browser/content.mjs`.

use crate::common::{page, spa_generated, ssg_html, template_html};

/// The same source, drawn by the static paint and by the template engine.
fn painted(body: &str) -> (String, String) {
    let src = page(body);
    (ssg_html(&src), template_html(&src))
}

#[test]
fn a_callout_is_a_note_with_its_kinds_icon_and_word() {
    for html in <[String; 2]>::from(painted(
        "Callout(\"Check first.\").warning\n    Callout(\"Hi\", title: \"Heads up\")",
    )) {
        assert!(
            html.contains("<div class=\"wf-callout\" role=\"note\" data-kind=\"warning\">"),
            "{html}"
        );
        assert!(html.contains("data-icon=\"warning\""), "{html}");
        assert!(html.contains("Warning\n"), "the kind's word: {html}");
        assert!(html.contains("Heads up"), "{html}");
        assert!(
            html.contains("<p class=\"wf-text\">Check first.</p>"),
            "{html}"
        );
    }
}

#[test]
fn a_stat_says_which_way_its_figure_moved() {
    for html in <[String; 2]>::from(painted(
        "Stat(\"Revenue\", value: 1234.5, format: .currency, change: -0.02)",
    )) {
        assert!(
            html.contains("<p class=\"wf-stat__value\">$1,234.50</p>"),
            "{html}"
        );
        assert!(html.contains("data-trend=\"down\""), "{html}");
        assert!(
            html.contains("<span class=\"wf-visually-hidden\">Down </span>"),
            "{html}"
        );
        assert!(html.contains("2.0%"), "{html}");
    }
    // `-(-0.02)` is a number, not a decrement JavaScript refuses.
    let js = spa_generated(&page(
        "state c = -0.02\n    Stat(\"Churn\", value: 1, change: c)",
    ));
    assert!(!js.contains("--"), "{js}");
}

#[test]
fn a_code_block_numbers_its_lines_and_copies() {
    let body = "CodeBlock(\"a\\nb\", language: \"wf\", title: \"App.wf\").copy.lines";
    for html in <[String; 2]>::from(painted(body)) {
        assert!(
            html.contains("<pre class=\"wf-code-block__lines\" aria-hidden=\"true\">1\n2</pre>"),
            "{html}"
        );
        assert!(
            html.contains("class=\"wf-btn wf-copy-button\" type=\"button\">Copy</button>"),
            "{html}"
        );
        assert!(html.contains("App.wf"), "{html}");
    }
    let js = spa_generated(&page(body));
    assert!(
        js.contains("WF.copyButton(n, (() => \"a\\nb\"), \"Copy\", \"Copied\")"),
        "{js}"
    );
}

#[test]
fn a_chip_can_be_pressed_and_removed() {
    let src = page(
        "state picked = false\n    Chip(\"Rust\", icon: \"code\", selected: picked).removable {\n        on click { picked = !picked }\n        on remove { log(\"gone\") }\n    }",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<button class=\"wf-chip__action\" type=\"button\" aria-pressed=\"false\">"),
        "{html}"
    );
    assert!(html.contains("aria-label=\"Remove Rust\""), "{html}");
    let js = spa_generated(&src);
    assert_eq!(
        js.matches("addEventListener(\"click\"").count(),
        2,
        "the press and the removal: {js}"
    );
}

#[test]
fn an_accordion_keeps_one_open_by_a_name() {
    let body = "Accordion.single {\n        Accordion.Item(\"A\") { Text(\"a\") }\n        Accordion.Item(\"B\", open: true) { Text(\"b\") }\n    }";
    for html in <[String; 2]>::from(painted(body)) {
        assert_eq!(html.matches("name=\"wf-accordion-1\"").count(), 2, "{html}");
        assert!(
            html.contains("<summary class=\"wf-accordion__summary\">A</summary>"),
            "{html}"
        );
        assert!(html.contains("open"), "{html}");
    }
}

#[test]
fn a_key_and_an_empty_state_and_a_fieldset() {
    for html in <[String; 2]>::from(painted(
        "Kbd(\"mod+k\")\n    EmptyState(title: \"Nothing yet\", text: \"Add one.\") { Button(\"Add\") }\n    Fieldset(legend: \"Address\") { Text(\"x\") }",
    )) {
        assert!(
            html.contains("<kbd class=\"wf-kbd\">Ctrl+K</kbd>"),
            "{html}"
        );
        assert!(
            html.contains("<p class=\"wf-empty-state__title\">Nothing yet</p>"),
            "{html}"
        );
        assert!(
            html.contains("<div class=\"wf-empty-state__actions\">"),
            "{html}"
        );
        assert!(
            html.contains("<legend class=\"wf-fieldset__legend\">Address</legend>"),
            "{html}"
        );
    }
}

#[test]
fn the_ones_that_act_are_wired_to_the_runtime() {
    let js = spa_generated(&page(
        "state q = \"\"\n    ThemeToggle\n    LocaleSwitch\n    OnThisPage(levels: 2)\n    InputGroup { Input(bind: q, aria-label: \"Search\")  InputGroup.Clear }\n    Field(label: \"Plan\", error: \"\") { Input(bind: q) }",
    ));
    for call in [
        "WF.themeToggle(n)",
        "WF.localeSwitch(n)",
        "WF.onThisPage(n, 2)",
        "WF.inputClear(n)",
        "WF.fieldFor(n, ",
    ] {
        assert!(js.contains(call), "{call}: {js}");
    }
    assert!(
        !js.contains("mount: "),
        "a lifetime, never an attribute: {js}"
    );
}
