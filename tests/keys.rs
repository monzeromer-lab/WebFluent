//! Keys (W4): `mod`, chords, a page's own `on keydown`, `keyIs` and
//! `keyLabel`, and `trapFocus:`. What a real browser does with them is
//! `tests/browser/keys.mjs`, and how each system writes a combination
//! `tests/js/keys.test.mjs`.

mod common;

use common::{page, spa_generated};

fn parse_error(src: &str) -> String {
    match webfluent::parse_source(src, "t.wf") {
        Ok(_) => String::new(),
        Err(e) => e.to_string(),
    }
}

fn checks(src: &str) -> String {
    webfluent::sema::types::check(&webfluent::parse_source(src, "t.wf").unwrap(), &|_| {
        "t.wf".to_string()
    })
    .findings
    .errors
    .iter()
    .map(|e| e.to_string())
    .collect()
}

#[test]
fn mod_and_a_chord_are_keys_a_keyboard_sends() {
    for spelling in [
        "mod+s",
        "mod+shift+p",
        "mod+alt+p",
        "mod+k mod+s",
        "ctrl+shift+g",
        "shift+F12",
    ] {
        let src = page(&format!(
            "on key(\"{spelling}\") {{ log(1) }}\n    Text(\"x\")"
        ));
        assert_eq!(parse_error(&src), "", "{spelling}");
    }
    let src = page("on key(\"mod+k mod+banana\") { log(1) }\n    Text(\"x\")");
    let err = parse_error(&src);
    assert!(
        err.contains("banana"),
        "a chord's every step is checked: {err}"
    );
    let src = page("on key(\"super+k\") { log(1) }\n    Text(\"x\")");
    assert!(parse_error(&src).contains("not a modifier"));
}

#[test]
fn a_page_hears_keydown_and_the_window_s_events_itself() {
    let src = page(
        "state said = \"\"\n    on keydown(e) {\n        if keyIs(e, \"mod+shift+p\") { said = \"commands\" }\n    }\n    on resize { said = \"resized\" }\n    Text(keyLabel(\"mod+shift+p\"))",
    );
    assert_eq!(checks(&src), "");
    let js = spa_generated(&src);
    assert!(
        js.contains("WF.listen(document, \"keydown\", (e) => {"),
        "it used to compile to a comment: {js}"
    );
    assert!(js.contains("WF.keyIs(e, \"mod+shift+p\")"), "{js}");
    assert!(js.contains("WF.listen(window, \"resize\""), "{js}");
    assert!(js.contains("WF.keyLabel(\"mod+shift+p\")"), "{js}");
    assert!(!js.contains("// event handler"), "{js}");
}

#[test]
fn an_app_s_shortcut_and_chord_are_the_app_s() {
    let src = "store S {\n    state n = 0\n}\napp {\n    on key(\"mod+s\") { S.n = S.n + 1 }\n    on key(\"mod+k mod+s\") { S.n = 0 }\n    Router\n}\npage P(path: \"/\", title: \"T\") { Text(\"x\") }\n";
    let js = spa_generated(src);
    assert!(js.contains("WF.onKey(document, \"mod+s\""), "{js}");
    assert!(js.contains("WF.onKey(document, \"mod+k mod+s\""), "{js}");
}

#[test]
fn key_is_a_condition_and_key_label_a_string() {
    let src =
        page("state n: Number = 0\n    on keydown(e) { n = keyIs(e, \"mod+s\") }\n    Text(\"x\")");
    let errors = checks(&src);
    assert!(
        errors.contains("Bool") || errors.contains("Number"),
        "a Bool is not a Number: {errors}"
    );
    let src = page("state label: String = keyLabel(\"mod+s\")\n    Text(label)");
    assert_eq!(checks(&src), "");
}

#[test]
fn a_focus_trap_is_any_element_s() {
    let src = page(
        "state finding = false\n    if finding {\n        Row(trapFocus: finding) {\n            Input(placeholder: \"Find\")\n            Button(\"Close\") { on click { finding = false } }\n        }\n    }",
    );
    assert_eq!(checks(&src), "");
    let js = spa_generated(&src);
    assert!(js.contains("WF.trapFocus("), "{js}");
    assert!(js.contains(", () => _finding());"), "{js}");
    assert!(!js.contains("trapFocus:"), "not an attribute: {js}");
}
