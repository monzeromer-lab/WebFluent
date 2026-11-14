//! Groups the arrow keys move through: a `Tabs` bound to a state, opened
//! on a value, vertical or opened by hand; `roving:` on any element; and
//! the static paint of a tab strip, as the live page draws it. What a real
//! browser does with them is `tests/browser/standards.mjs`; the runtime's
//! `roving` is `tests/js/focus.test.mjs`.

use crate::common::{page, spa_generated, ssg_html};

#[test]
fn tabs_bound_to_a_state_open_the_page_whose_value_it_holds() {
    let src = page(
        "state tab = \"billing\"\n    Tabs(bind: tab, label: \"Account\") {\n        Tabs.Page(\"Profile\", value: \"profile\") { Text(\"Your profile\") }\n        Tabs.Page(\"Billing\", value: \"billing\", disabled: false) { Text(\"Your bills\") }\n    }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("= [\"profile\", \"billing\"];"),
        "the pages' values: {js}"
    );
    assert!(
        js.contains(".indexOf(_tab())"),
        "the open tab is the bound value's: {js}"
    );
    assert!(js.contains("WF.tabs("), "{js}");
    assert!(js.contains("\"aria-label\": \"Account\""), "{js}");
    assert!(js.contains("disabled: !!(false)"), "{js}");
    assert!(
        !js.contains("style.display"),
        "a panel is hidden, not styled: {js}"
    );

    // Painted with the bound value's page open, the others hidden.
    let html = ssg_html(&src);
    assert!(
        html.contains("role=\"tablist\" aria-label=\"Account\""),
        "{html}"
    );
    assert!(
        html.contains("aria-selected=\"true\" tabindex=\"0\">Billing</button>"),
        "{html}"
    );
    assert!(
        html.contains("aria-selected=\"false\" tabindex=\"-1\">Profile</button>"),
        "{html}"
    );
    let profile = html.find("Your profile").expect("both panels are painted");
    let before = &html[..profile];
    let panel = &before[before.rfind("<div").unwrap()..];
    assert!(
        panel.contains(" hidden"),
        "the closed panel is hidden: {panel}"
    );
}

#[test]
fn tabs_opened_on_a_value_tell_the_page_which_was_picked() {
    let src = page(
        "state picked = \"\"\n    Tabs(value: \"b\").vertical.manual {\n        on change(v) { picked = v }\n        Tabs.Page(\"A\", value: \"a\") { Text(\"a\") }\n        Tabs.Page(\"B\", value: \"b\") { Text(\"b\") }\n    }\n    Text(picked)",
    );
    let js = spa_generated(&src);
    assert!(js.contains("vertical: true, manual: true"), "{js}");
    assert!(js.contains("\"aria-orientation\": \"vertical\""), "{js}");
    assert!(
        js.contains("_picked.set(v)"),
        "on change hears the value: {js}"
    );
    assert!(
        !js.contains("addEventListener(\"change\""),
        "a change inside a panel is not the tab picked: {js}"
    );
}

#[test]
fn roving_gives_any_element_one_tab_stop_and_the_arrow_keys() {
    let src = page(
        "Row(roving: .x, role: \"toolbar\", aria-label: \"Text\") {\n        Button(\"Bold\")\n        Button(\"Italic\")\n    }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("WF.roving(") && js.contains("axis: \"x\", typeahead: true"),
        "{js}"
    );
    assert!(!js.contains("roving: "), "not an attribute: {js}");
    let html = ssg_html(&src);
    assert!(!html.contains("roving"), "nor in the paint: {html}");
}
