//! What clicking through the `wf init` starters turned up: programs the
//! compiler accepted and compiled into a page that threw, stood still, or
//! skipped its own rules. Each is held to the JavaScript the browser gets.

mod common;

use common::spa_js;
use webfluent::codegen::jscheck::check_js;

fn page(body: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\n"
    )
}

/// The bundle, read back as the build reads it.
fn js(src: &str) -> String {
    let out = spa_js(src);
    if let Err(fault) = check_js(&out) {
        panic!(
            "the bundle is not a script a browser takes: {} at line {}\n{}",
            fault.message,
            fault.line,
            webfluent::codegen::jscheck::line_of(&out, fault.line)
        );
    }
    out
}

#[test]
fn a_handler_that_reads_key_is_given_it() {
    // The SPA starter's task input: Enter threw `key is not defined`.
    let out = js(&page(
        "    state draft = \"\"\n    Input(bind: draft, label: \"New\").text {\n        on keydown { if key == \"Enter\" { draft = \"\" } }\n    }",
    ));
    assert!(
        out.contains("(event, key = event?.key)"),
        "the handler declares `key`:\n{out}"
    );
}

#[test]
fn a_handler_that_reads_value_is_given_it() {
    let out = js(&page(
        "    state last = \"\"\n    Input(label: \"Q\").text {\n        on input { last = value }\n    }\n    Text(last)",
    ));
    assert!(out.contains("value = event?.target?.value"), "{out}");
}

#[test]
fn a_state_named_key_is_not_shadowed() {
    let out = js(&page(
        "    state key = \"k\"\n    Button(\"Go\") { on click { key = \"j\" } }\n    Text(key)",
    ));
    assert!(!out.contains("key = event?.key"), "{out}");
}

#[test]
fn a_checkbox_label_follows_state() {
    // The static starter's checkbox kept the language the page opened in.
    let out = js(&page(
        "    state agreed = false\n    state who = \"you\"\n    Checkbox(bind: agreed, label: \"I agree, {who}\")\n    Switch(bind: agreed, label: \"On for {who}\")",
    ));
    assert!(
        out.contains("WF.text(() => `I agree, ${_who()}`)"),
        "the checkbox label is live:\n{out}"
    );
    assert!(out.contains("WF.text(() => `On for ${_who()}`)"), "{out}");
}

#[test]
fn every_label_and_size_that_reads_state_follows_it() {
    let out = js(&page(
        "    state v = \"a\"\n    state n = 10\n    state x = 1\n    state d: Date? = @2026-01-01\n    \
         Slider(bind: x, min: 0, max: 9, label: \"S {v}\")\n    \
         DatePicker(bind: d, label: \"D {v}\")\n    \
         FileUpload(label: \"F {v}\")\n    \
         Tooltip(\"T {v}\") { Button(\"tip\") }\n    \
         Tabs { Tabs.Page(\"P {v}\") { Text(\"in\") } }\n    \
         Table(caption: \"C {v}\") { Table.Head { Table.Row { Table.Cell(\"h\") } } }\n    \
         Skeleton(width: \"{n}px\", height: \"4px\")\n    \
         Avatar(initials: v)",
    ));
    for live in [
        "() => `S ${_v()}`",
        "() => `D ${_v()}`",
        "() => `F ${_v()}`",
        "() => `T ${_v()}`",
        "() => `P ${_v()}`",
        "() => `C ${_v()}`",
    ] {
        assert!(out.contains(live), "missing `{live}`:\n{out}");
    }
    assert!(out.contains("WF.effect(() => {"), "the skeleton's width follows `n`");
    assert!(out.contains("() => _v()"), "the avatar's initials follow `v`");
}

#[test]
fn an_unbound_form_with_rules_asks_them_before_submitting() {
    // The SPA starter's settings saved a username its rule refused.
    let out = js(&page(
        "    state name = \"\"\n    state saved = false\n    validate name { required  minLength(3) }\n    \
         Form {\n        on submit { saved = true }\n        Input(bind: name, label: \"Name\").text\n        \
         Button(\"Save\", type: .submit)\n    }\n    Text(\"{saved}\")",
    ));
    assert!(out.contains("const _wfForm = WF.form("), "{out}");
    assert!(out.contains("ref: _wfForm"), "the form takes the handle");
    assert!(
        out.contains("if (!_wfForm.__submitting()) return;"),
        "the submit asks the rules:\n{out}"
    );
    assert!(out.contains("form: _wfForm"), "the rule belongs to the form");
}

#[test]
fn a_form_handle_does_not_leak_into_the_next_page() {
    let src = "page A(path: \"/\", title: \"A\", description: \"A\") {\n    Heading(\"A\").h1\n    \
               state n = \"\"\n    validate n { required }\n    Form(bind: form) {\n        on submit { n = \"\" }\n        \
               Input(bind: n, label: \"N\").text\n    }\n}\n\
               page B(path: \"/b\", title: \"B\", description: \"B\") {\n    Heading(\"B\").h1\n    \
               state sent = false\n    Form {\n        on submit { sent = true }\n        Button(\"Go\", type: .submit)\n    }\n    \
               Text(\"{sent}\")\n}\n";
    let out = js(src);
    let b = &out[out.find("function Page_B").expect("page B")..];
    assert!(
        !b.contains("form.__submitting"),
        "page B reads page A's handle:\n{b}"
    );
}

#[test]
fn a_components_form_is_its_own() {
    // Two components with forms declared one name twice in the bundle.
    let src = "component One {\n    state a = \"\"\n    validate a { required }\n    Form {\n        on submit { a = \"\" }\n        \
               Input(bind: a, label: \"A\").text\n    }\n}\n\
               component Two {\n    state b = \"\"\n    validate b { required }\n    Form {\n        on submit { b = \"\" }\n        \
               Input(bind: b, label: \"B\").text\n    }\n}\n"
        .to_string()
        + &page("    One\n    Two");
    let out = js(&src);
    let one = &out[out.find("function Component_One").expect("One")..];
    let first = one.find("const _wfForm").expect("One's form handle");
    let body_start = one.find('{').unwrap();
    assert!(first > body_start, "the handle is declared inside the component");
}

#[test]
fn a_validated_checkbox_shows_its_message() {
    let out = js(&page(
        "    state agreed = false\n    validate agreed { required \"Please agree\" }\n    \
         Form {\n        on submit { agreed = false }\n        Checkbox(bind: agreed, label: \"I agree\")\n    }",
    ));
    assert!(out.contains("WF.choiceError("), "{out}");
}
