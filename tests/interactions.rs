//! Interactions that hold up: a call with the arguments of one still
//! running is that call (`concurrent` opts out), a button or a form busy
//! while the work its handler started runs, `Button(loading:)`, a handler
//! run as often as `debounce:` or `throttle:` says, and `on visible`. What
//! a real browser does with them is `tests/browser/busy.mjs`; the runtime's
//! side is `tests/js/busy.test.mjs`.

use crate::common::{page, spa_generated};

fn store_js(src: &str) -> String {
    spa_generated(src)
}

#[test]
fn an_async_action_hands_its_calls_to_same_call() {
    let src = page(
        "state n = 0\n    action save(id: String) { await fetch(\"/x\")  n = n + 1 }\n    action poll() concurrent { await fetch(\"/y\") }\n    Button(\"Save\") { on click { save(\"a\") } }\n    Button(\"Poll\") { on click { poll() } }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("return WF.sameCall(_save_pending, [id], async () => {"),
        "{js}"
    );
    assert!(js.contains("}, false);"), "{js}");
    assert!(
        js.contains("}, true);"),
        "a concurrent action runs every call: {js}"
    );
}

#[test]
fn a_stores_concurrent_actions_are_listed() {
    let src = format!(
        "store Feed {{\n    action poll() concurrent {{ await fetch(\"/p\") }}\n    action save() {{ await fetch(\"/s\") }}\n}}\n{}",
        page("use Feed\n    Button(\"Poll\") { on click { Feed.poll() } }")
    );
    let js = store_js(&src);
    assert!(js.contains("concurrent: [\"poll\"],"), "{js}");
}

#[test]
fn a_button_or_form_is_busy_only_where_its_handler_starts_work() {
    let src = page(
        "state n = 0\n    action go() { await fetch(\"/y\") }\n    Button(\"Go\") { on click { go() } }\n    Button(\"Count\") { on click { n = n + 1 } }\n    Form { on submit { go() }  Button(\"Send\", type: .submit) }",
    );
    let js = spa_generated(&src);
    assert_eq!(
        js.matches("WF.busy(").count(),
        2,
        "the Go button and the form: {js}"
    );
    assert!(
        js.contains("addEventListener(\"click\", (event) => { _n.set"),
        "a handler that only counts is plain: {js}"
    );
}

#[test]
fn a_loading_button_is_busy_while_its_state_holds() {
    let src = page(
        "state paying = false\n    state n = 0\n    Button(\"Pay\", loading: paying) { on click { n = n + 1 } }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("\"aria-busy\": () => (_paying()) ? \"true\" : null"),
        "{js}"
    );
    assert!(
        js.contains("WF.busy("),
        "a press while loading does nothing: {js}"
    );
    assert!(!js.contains("loading:"), "not an attribute: {js}");
}

#[test]
fn a_handler_runs_as_often_as_it_says() {
    let src = page(
        "state typed = \"\"\n    state q = \"\"\n    state y = 0\n    Input(bind: typed, label: \"Search\") { on input(debounce: 300.ms) { q = typed } }\n    Stack { on scroll(e, throttle: 100.ms) { y = e.target.scrollTop } }\n    on resize(throttle: 200) { y = 0 }",
    );
    let js = spa_generated(&src);
    assert!(js.contains("WF.debounce((event) => {"), "{js}");
    assert!(js.contains(", 300)"), "{js}");
    assert!(js.contains("WF.throttle((e) => {"), "{js}");
    assert!(
        js.contains("WF.listen(window, \"resize\", WF.throttle("),
        "{js}"
    );
}

#[test]
fn a_handler_says_its_pace_back_as_it_was_written() {
    let src = "page P(path: \"/\", title: \"T\") {\n    state q = \"\"\n    Input(bind: q, label: \"Q\") { on input(e, debounce: 300.ms) { log(q) } }\n}\n";
    let print = |src: &str| {
        let program = webfluent::parse_source(src, "t.wf").unwrap();
        webfluent::syntax::print::declaration(&program.declarations[0], &program)
    };
    let printed = print(src);
    assert!(
        printed.contains("on input(e, debounce: 300.ms)"),
        "{printed}"
    );
    let concurrent = "page P(path: \"/\", title: \"T\") {\n    action poll() concurrent { await fetch(\"/p\") }\n}\n";
    let printed = print(concurrent);
    // An action with no parameters is printed without its `()`, and reads
    // back the same.
    assert!(printed.contains("action poll concurrent {"), "{printed}");
    assert_eq!(print(&printed), printed);
}

#[test]
fn on_visible_is_heard_from_the_runtime() {
    let src = page("state loads = 0\n    Text(\"End\") { on visible { loads = loads + 1 } }");
    let js = spa_generated(&src);
    assert!(js.contains("WF.gesture("), "{js}");
    assert!(js.contains("\"visible\""), "{js}");
}
