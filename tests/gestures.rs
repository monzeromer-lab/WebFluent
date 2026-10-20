//! Gestures (W6): `drag:` and `on receive(d)`, `on dragmove`,
//! `on longpress`, `on swipe`, and `Splitter`. What a real browser does with
//! them is `tests/browser/gestures.mjs`; this holds the build to what it
//! hands the runtime.

mod common;

use common::{page, spa_generated, ssg_html};

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
fn a_drag_carries_a_value_to_what_receives_it() {
    let src = page(
        "state rows = [\"a\", \"b\"]\n    state landing = \"\"\n    for r in rows {\n        Card(drag: r, dragGroup: \"tree\", dropValue: r, dropInside: r == \"a\", dropAxis: .y) {\n            on receive(d) { log(d.value) }\n            on dragmove(d) { landing = d.position }\n            Text(r)\n        }\n    }",
    );
    assert_eq!(checks(&src), "");
    let js = spa_generated(&src);
    assert!(
        js.contains("WF.draggable(") && js.contains(", () => r, { group: \"tree\" });"),
        "{js}"
    );
    assert!(
        js.contains("{ value: () => r, axis: \"y\", inside: (r === \"a\"), group: \"tree\" });"),
        "{js}"
    );
    // The runtime recognises the drop and the move, and hands the handler
    // what it saw.
    assert!(js.contains(", \"receive\", (d) => {"), "{js}");
    assert!(js.contains(", \"dragmove\", (d) => {"), "{js}");
    assert!(!js.contains("addEventListener(\"receive\""), "{js}");
    for attr in ["drag", "dragGroup", "dropValue", "dropInside", "dropAxis"] {
        assert!(
            !js.contains(&format!(" {attr}: ")) && !js.contains(&format!("\"{attr}\"")),
            "{attr} is the runtime's, not an attribute: {js}"
        );
    }
    let html = ssg_html(&src);
    assert!(
        !html.contains("drag-group") && !html.contains("drop-value"),
        "nor painted: {html}"
    );
}

#[test]
fn a_long_press_and_a_swipe_are_recognised() {
    let src = page(
        "state said = \"\"\n    Card {\n        on longpress(p) { said = \"held\" }\n        on swipe(s) { said = s.direction }\n        Text(\"x\")\n    }",
    );
    assert_eq!(checks(&src), "");
    let js = spa_generated(&src);
    assert!(js.contains(", \"longpress\", (p) => {"), "{js}");
    assert!(js.contains(", \"swipe\", (s) => {"), "{js}");
}

#[test]
fn a_splitter_sizes_what_it_is_bound_to() {
    let src = page(
        "state width = 240\n    Row {\n        Container { style { width: {width}px } Text(\"Files\") }\n        Splitter(bind: width, min: 160, max: 480, reset: 240, label: \"Sidebar width\").reverse\n        Container { Text(\"Editor\") }\n    }",
    );
    assert_eq!(checks(&src), "");
    let js = spa_generated(&src);
    assert!(
        js.contains("className: \"wf-splitter\", \"aria-label\": \"Sidebar width\""),
        "{js}"
    );
    assert!(
        js.contains("() => _width(), (v) => { _width.set(v) }, { min: () => 160, max: () => 480, reset: () => 240, reverse: true });"),
        "{js}"
    );
    let html = ssg_html(&src);
    assert!(html.contains("class=\"wf-splitter"), "{html}");
}

#[test]
fn a_splitter_writes_a_store_too() {
    let src = "store Layout {\n    state side = 240\n}\npage P(path: \"/\", title: \"T\") {\n    Splitter(bind: Layout.side, min: 160).horizontal\n}\n";
    assert_eq!(checks(src), "");
    let js = spa_generated(src);
    assert!(
        js.contains(
            "() => Layout.side, (v) => { Layout.side = v }, { min: () => 160, horizontal: true });"
        ),
        "{js}"
    );
}
