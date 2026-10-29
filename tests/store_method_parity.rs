//! A store's action and a page's handler are compiled by two different
//! emitters. They used to keep a method table each, and the tables drifted:
//! `remove`, `contains`, `toUpper`, `toLower` and every one of the language's
//! scalar methods were in the page's table and not the store's, so inside a
//! store action they compiled to `store.items.remove(i)` and
//! `store.due.plus(…)` — methods no JavaScript array, string or JSON value
//! has. Each threw the first time its action ran.
//!
//! Nothing before code generation could see it: the source parses, type-checks
//! and lints clean. So these hold the generated JavaScript itself, and the
//! last test holds the two emitters to each other rather than to a list
//! someone has to remember to extend.

mod common;

use common::spa_generated;

fn store_js(members: &str, uses: &str) -> String {
    let src = format!(
        r#"
store S {{
{members}
}}

page P(path: "/", title: "T", description: "D") {{
    use S
    Container {{
        Heading("T").h1
        Text("{{S.probe}}")
        Button("go") {{ on click {{ {uses} }} }}
    }}
}}

app {{ Router }}
"#
    );
    spa_generated(&src)
}

/// The action bodies, which is where the broken calls appeared.
fn actions_of(js: &str) -> String {
    let at = js.find("actions:").expect("the store has actions");
    let end = js[at..].find("}),").map(|e| at + e).unwrap_or(js.len());
    js[at..end].to_string()
}

#[test]
fn a_store_removes_from_a_list_through_the_runtime() {
    let js = store_js(
        "    state items = [\"a\", \"b\"]\n    state probe = \"\"\n\
         \n    action drop(i: Number) { items.remove(i) }",
        "S.drop(0)",
    );
    let actions = actions_of(&js);
    assert!(
        actions.contains("WF.removeAt(store.items"),
        "a store's `remove` must go through WF.removeAt, got: {actions}"
    );
    assert!(
        !actions.contains("store.items.remove("),
        "`.remove(` is not a JavaScript array method: {actions}"
    );
}

#[test]
fn a_store_maps_the_string_and_list_methods_onto_real_javascript() {
    let js = store_js(
        "    state word = \"hi\"\n    state items = [\"a\"]\n    state probe = \"\"\n\
         \n    action a() { word = word.toUpper() }\
         \n    action b() { word = word.toLower() }\
         \n    action c() { probe = if items.contains(\"a\") { \"y\" } else { \"n\" } }",
        "S.a()  S.b()  S.c()",
    );
    let actions = actions_of(&js);
    for (wrong, right) in [
        (".toUpper()", ".toUpperCase()"),
        (".toLower()", ".toLowerCase()"),
        (".contains(", ".includes("),
    ] {
        assert!(
            actions.contains(right),
            "expected {right} in the store's actions, got: {actions}"
        );
        assert!(
            !actions.contains(wrong),
            "{wrong} is not a JavaScript method: {actions}"
        );
    }
}

#[test]
fn a_store_computes_the_scalar_types_through_the_runtime() {
    // A Date, Money and a Url are plain JSON values at run time — a string
    // and a map — so the work is the runtime's. A method call on the value
    // itself throws.
    let js = store_js(
        "    state due: Date = @2026-03-14\n    state total: Money = $9.99\n\
         \n    state probe = \"\"\n\
         \n    action bump() { due = due.plus(days: 5) }\
         \n    action twice() { total = total.times(2) }",
        "S.bump()  S.twice()",
    );
    let actions = actions_of(&js);
    assert!(
        actions.contains("WF.plus(store.due") && actions.contains("WF.times(store.total"),
        "a store's scalar methods must go through the runtime, got: {actions}"
    );
    assert!(
        !actions.contains("store.due.plus(") && !actions.contains("store.total.times("),
        "a Date and Money carry no methods of their own: {actions}"
    );
}

#[test]
fn removing_from_a_page_state_leaves_the_list_not_nothing() {
    // `set` returns nothing, so an assignment whose right-hand side was
    // itself a `set` left the list undefined: the item went, and then so did
    // the list. The emitted expression has to evaluate to the new list.
    let src = r#"
page P(path: "/", title: "T", description: "D") {
    state items = ["a", "b", "c"]
    Container {
        Heading("T").h1
        for t in items { Text(t) }
        Button("go") { on click { items = items.remove(0) } }
    }
}
"#;
    let js = spa_generated(src);
    assert!(
        !js.contains("_items.set(_items.set("),
        "a set inside a set sets the list to what `set` returns, which is nothing: {js}"
    );
    assert!(
        js.contains("WF.removeAt(_items()"),
        "the removal itself must still go through the runtime: {js}"
    );
}

/// The guard against the whole class: every method the page emitter maps
/// onto something other than a plain call must be mapped by the store
/// emitter too. Both now read one table, so this holds them to that.
#[test]
fn the_two_emitters_map_the_same_methods() {
    let cases = [
        ("items.remove(0)", "items", "[\"a\", \"b\"]"),
        ("word.toUpper()", "word", "\"hi\""),
        ("word.toLower()", "word", "\"hi\""),
        ("word.trim()", "word", "\"hi\""),
        ("items.unique()", "items", "[\"a\", \"b\"]"),
        ("items.first()", "items", "[\"a\", \"b\"]"),
    ];

    for (call, name, init) in cases {
        let in_store = store_js(
            &format!(
                "    state {name} = {init}\n    state probe = \"\"\n    action go() {{ {call} }}"
            ),
            "S.go()",
        );
        let in_page = spa_generated(&format!(
            "page P(path: \"/\", title: \"T\", description: \"D\") {{\n\
             \x20   state {name} = {init}\n\
             \x20   Container {{ Heading(\"T\").h1  Button(\"go\") {{ on click {{ {call} }} }} }}\n}}\n"
        ));

        // Whatever shape each emitter chose, neither may leave the
        // WebFluent method name standing as a JavaScript call. A runtime
        // helper of the same name — `WF.unique(` — carries `.unique(`
        // inside it, so the runtime's own calls come out before looking.
        let method = call.split('.').nth(1).unwrap().split('(').next().unwrap();
        let bare = format!(".{method}(");
        let js_real = matches!(method, "trim");
        if !js_real {
            let strip = |js: &str| js.replace(&format!("WF{bare}"), "WF__(");
            assert!(
                !strip(&in_store).contains(&bare),
                "`{call}` left `{bare}` in a store action, which JavaScript has no method for"
            );
            assert!(
                !strip(&in_page).contains(&bare),
                "`{call}` left `{bare}` in a page handler, which JavaScript has no method for"
            );
        }
    }
}

/// The static paint runs the same methods at build time. It knew only the
/// JavaScript spellings — `toUpperCase`, `toLowerCase`, `includes` — so a
/// page that wrote WebFluent's own `toUpper`, `toLower` or `contains`
/// painted an empty element and stayed empty for a reader without
/// JavaScript, and for a crawler.
#[test]
fn the_static_paint_runs_the_webfluent_spellings_too() {
    let src = r#"
const WORD = "hello"
const ITEMS = ["a", "b", "a"]

page P(path: "/", title: "T", description: "D") {
    Container {
        Heading("T").h1
        Text("up:{WORD.toUpper()}")
        Text("low:{WORD.toLower()}")
        Text("has:{if ITEMS.contains("a") { "yes" } else { "no" }}")
    }
}
"#;
    let html = common::raw_output(common::Backend::Ssg, src);
    for want in ["up:HELLO", "low:hello", "has:yes"] {
        assert!(
            html.contains(want),
            "the static paint should carry `{want}`, but the element came out empty"
        );
    }
}

/// Two stores, one calling the other's actions, and a page calling both.
fn two_stores(caller: &str, page_calls: &str) -> String {
    let src = format!(
        r#"
store OpenFiles {{
    state x = ""
    state n = 0
    action preview(path: String) {{ x = path }}
    action first(count: Number) {{ n = count }}
    action truncate(text: String, size: Number) {{
        x = text
        n = size
    }}
    action last(kind: String) {{ x = kind }}
}}

store Shell {{
    state names: [String] = ["a", "b", "c"]
    state label = "hello"
    action open(path: String) {{
{caller}
    }}
}}

page P(path: "/", title: "T", description: "D") {{
    use Shell
    use OpenFiles
    Container {{
        Heading("T").h1
        Text("{{OpenFiles.x}} {{OpenFiles.n}} {{Shell.label}}")
        Button("go") {{ on click {{ {page_calls} }} }}
    }}
}}

app {{ Router }}
"#
    );
    spa_generated(&src)
}

/// A store's action named like one of the runtime's helpers — `preview`,
/// `first`, `truncate`, `last` — is the store's own when another store
/// calls it. The store emitter used to send `OpenFiles.preview(path)`
/// through the helper table, as `WF.preview(OpenFiles, path)`: the runtime's
/// file preview, which called the action with no arguments, so `path`
/// arrived `undefined`. A declared member always wins over a helper.
#[test]
fn a_store_action_named_like_a_helper_is_the_stores_own_from_another_store() {
    let js = two_stores(
        "        OpenFiles.preview(path)\n\
         \x20       OpenFiles.first(3)\n\
         \x20       OpenFiles.truncate(path, 4)\n\
         \x20       OpenFiles.last(\"x\")",
        "Shell.open(\"a.wf\")",
    );
    for want in [
        "OpenFiles.preview(path)",
        "OpenFiles.first(3)",
        "OpenFiles.truncate(path, 4)",
        "OpenFiles.last(\"x\")",
    ] {
        assert!(
            js.contains(want),
            "expected `{want}` in the store's action, got:\n{js}"
        );
    }
    for helper in ["preview", "first", "truncate", "last"] {
        assert!(
            !js.contains(&format!("WF.{helper}(OpenFiles")),
            "`OpenFiles.{helper}` went through the runtime's helper:\n{js}"
        );
    }
}

/// And from a page, which already called a store's actions directly.
#[test]
fn a_store_action_named_like_a_helper_is_the_stores_own_from_a_page() {
    let js = two_stores(
        "        label = path",
        "OpenFiles.preview(\"p\")  OpenFiles.first(2)  OpenFiles.truncate(\"t\", 1)",
    );
    for want in [
        "OpenFiles.preview(\"p\")",
        "OpenFiles.first(2)",
        "OpenFiles.truncate(\"t\", 1)",
    ] {
        assert!(
            js.contains(want),
            "expected `{want}` in the page's handler, got:\n{js}"
        );
    }
    assert!(!js.contains("WF.preview(OpenFiles"), "{js}");
    assert!(!js.contains("WF.first(OpenFiles"), "{js}");
}

/// The helpers still work on what they are for: a store's list and string.
#[test]
fn the_helpers_still_compile_on_lists_and_strings_inside_a_store() {
    let js = two_stores(
        "        label = names.first() ?? \"\"\n\
         \x20       label = label.truncate(2)\n\
         \x20       names = names.take(2)\n\
         \x20       label = names.last() ?? \"\"",
        "Shell.open(\"a.wf\")",
    );
    for want in [
        "WF.first(store.names)",
        "WF.truncate(store.label, 2)",
        "WF.take(store.names, 2)",
        "WF.last(store.names)",
    ] {
        assert!(js.contains(want), "expected `{want}`, got:\n{js}");
    }
}
