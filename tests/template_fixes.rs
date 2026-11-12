//! The template engine held to what a server needs of it: data that cannot
//! add CSS or attributes of its own, the same answers a build gives, no
//! render state outliving its render, and a bounded render however deep or
//! wide the template and its data ask it to go.

use serde_json::{Value, json};
use std::sync::mpsc;
use std::time::Duration;
use webfluent::codegen::static_eval::{Scope, Static, eval};
use webfluent::parser::v2::parse_v2;
use webfluent::parser::{Arg, Declaration, Expr, StatementKind};
use webfluent::{Template, WebFluentError};

/// The stack a test thread gets: 2 MB — what a server's worker thread has —
/// in a release build; a debug build's frames are several times larger.
const STACK: usize = if cfg!(debug_assertions) { 16 << 20 } else { 2 << 20 };

/// Run `f` on a thread with [`STACK`], and end the process if it does not
/// finish in `secs`: a hang or a runaway allocation must fail the test,
/// not the machine.
fn within<T: Send + 'static>(secs: u64, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            let _ = tx.send(f());
        })
        .unwrap();
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("did not finish within {secs}s (or overflowed its stack)");
            std::process::exit(1);
        }
    }
}

fn render(src: &str, data: Value) -> String {
    Template::from_str(src)
        .unwrap_or_else(|e| panic!("{e}"))
        .render_html_fragment(&data)
        .unwrap_or_else(|e| panic!("{e}"))
}

fn codes(err: &WebFluentError) -> Vec<&'static str> {
    err.diagnostics().iter().map(|d| d.code).collect()
}

// ─── B6: a style value from the data is one value ────────────────────

const STYLED: &str = r#"page P(path: "/") {
    Text("hi") { style { color: {c} } }
}"#;

#[test]
fn data_cannot_add_declarations_to_a_style() {
    for evil in [
        "red; position:fixed; inset:0; background:url(https://evil.example/t.png)",
        "red;position:fixed",
        "red } body { display:none",
        "red { x",
        "url(https://evil.example/t.png)",
        "URL(https://evil.example/t.png)",
        "image-set(\"https://evil.example/t.png\" 1x)",
        "-webkit-image-set(\"x.png\" 1x)",
        "\\75 rl(https://evil.example/t.png)",
        "red\\3b position:fixed",
        "expression(alert(1))",
        "</style><script>alert(1)</script>",
    ] {
        let html = render(STYLED, json!({ "c": evil }));
        assert!(!html.contains("position"), "{evil}: {html}");
        assert!(!html.to_lowercase().contains("url("), "{evil}: {html}");
        assert!(!html.contains("evil.example"), "{evil}: {html}");
        assert!(!html.contains("color:"), "the declaration is left out: {html}");
        assert!(!html.contains("<script"), "{evil}: {html}");
    }
}

#[test]
fn a_style_value_from_the_data_that_is_one_value_is_kept() {
    for good in [
        "#0F766E",
        "rgb(15, 118, 110)",
        "var(--accent)",
        "linear-gradient(90deg, red, blue)",
        "\"Inter\", sans-serif",
    ] {
        let html = render(STYLED, json!({ "c": good }));
        let escaped = good.replace('"', "&quot;");
        assert!(html.contains(&format!("color: {escaped}")), "{good}: {html}");
    }
}

#[test]
fn what_the_template_writes_around_a_splice_is_its_own() {
    let src = r#"page P(path: "/") {
    Text("hi") { style { background: url({img}) center } }
}"#;
    let html = render(src, json!({ "img": "/photo.png" }));
    assert!(html.contains("background: url(/photo.png) center"), "{html}");
    // The data's piece may not end the declaration the template opened.
    let html = render(src, json!({ "img": "x); position: fixed; (" }));
    assert!(!html.contains("position"), "{html}");
    assert!(!html.contains("background"), "{html}");
}

// ─── S4: a class from the data stays inside its attribute ────────────

#[test]
fn a_class_from_the_data_cannot_close_its_attribute() {
    let evil = "x\"/tabindex=\"0\"/autofocus/onfocus=\"alert(1)";
    let builtin = r#"page P(path: "/") { Text("hi", class: c) }"#;
    let component = r#"component Box2 { Card { Text("in") } }
page P(path: "/") { Box2(class: c) }"#;
    let listed = r#"page P(path: "/") { Card(class: [c, { "on": true }]) { Text("x") } }"#;
    for src in [builtin, component, listed] {
        let html = render(src, json!({ "c": evil }));
        assert!(!html.contains("onfocus=\""), "{html}");
        assert!(!html.contains(" autofocus"), "{html}");
        assert!(!html.contains("tabindex=\""), "{html}");
        assert!(html.contains("x&quot;/tabindex=&quot;0&quot;/autofocus"), "{html}");
    }
    // A class without anything to escape is written as it is.
    let html = render(builtin, json!({ "c": "lead is-on" }));
    assert!(html.contains("lead is-on"), "{html}");
}

// ─── S14: HTML handed to a frame is sandboxed ────────────────────────

#[test]
fn a_frame_drawing_html_from_the_data_is_sandboxed() {
    let given = r#"page P(path: "/") { Frame(srcdoc: html, title: "Preview") }"#;
    let html = render(given, json!({ "html": "<p>hi</p>" }));
    assert!(html.contains("sandbox=\"\""), "{html}");
    assert!(html.contains("srcdoc=\"&lt;p&gt;hi&lt;/p&gt;\""), "{html}");
    // The template's own HTML is its own.
    let literal = r#"page P(path: "/") { Frame(srcdoc: "<p>hi</p>", title: "Preview") }"#;
    assert!(!render(literal, json!({})).contains("sandbox="));
    // A sandbox the template names is the one it gets.
    let named =
        r#"page P(path: "/") { Frame(srcdoc: html, title: "Preview", sandbox: "allow-scripts") }"#;
    let html = render(named, json!({ "html": "<p>hi</p>" }));
    assert_eq!(html.matches("sandbox=").count(), 1, "{html}");
    assert!(html.contains("sandbox=\"allow-scripts\""), "{html}");
}

// ─── B8: node ids live and die with their render ─────────────────────

#[test]
fn a_render_that_panicked_leaves_no_node_ids_for_the_next() {
    let src = r#"page P(path: "/") {
    Heading("Report").h1
    Text("{(1.5).toFixed(digits)}")
}"#;
    let studio = Template::from_str(src).unwrap().with_studio(true);
    let plain = Template::from_str(src).unwrap();
    // A render that ends badly: on the thread a server would reuse.
    let _ = std::panic::catch_unwind(|| studio.render_html_fragment(&json!({ "digits": 70000 })));
    let html = plain.render_html_fragment(&json!({ "digits": 1 })).unwrap();
    assert!(!html.contains("data-wf-node"), "{html}");
    // And a studio render still marks its own.
    let marked = Template::from_str(src)
        .unwrap()
        .with_studio(true)
        .render_html_fragment(&json!({ "digits": 1 }))
        .unwrap();
    assert!(marked.contains("data-wf-node"), "{marked}");
}

// ─── B7: one expression, one answer ──────────────────────────────────

/// Each expression, and what it shows in the browser. The template must
/// show exactly that; the build, where it works the value out at all,
/// must too.
const CORPUS: &[(&str, &str)] = &[
    ("name.length", "5"),
    ("\"héllo\".length", "5"),
    ("\"مرحبا\".length", "5"),
    ("[1, 2, 3].length", "3"),
    ("\"12\" < \"9\"", "true"),
    ("\"a\" < \"b\"", "true"),
    ("\"b\" <= \"a\"", "false"),
    ("\"abc\" > \"abd\"", "false"),
    ("2 < 10", "true"),
    ("[10, 20, 30][1]", "20"),
    ("[10, 20, 30][1.5]", ""),
    ("[10, 20, 30][-1]", ""),
    ("[10, 20, 30][3]", ""),
    ("[1, 2, 2].includes(2)", "true"),
    ("[1, 2].join()", "1,2"),
    ("[1, 2].join(\" · \")", "1 · 2"),
    ("\"ab\".toUpperCase()", "AB"),
    ("\"5\" + 1", "51"),
    ("1 + 2 * 3", "7"),
    ("null ?? \"d\"", "d"),
    ("3 == 3.0", "true"),
    ("\"a-b-c\".split(\"-\").length", "3"),
    ("[3, 1, 2].sortBy(x => x).first()", "1"),
    ("\"Hello\".slice(1, 3)", "el"),
    ("[1, 2, 3].map(x => x * 2).join(\"-\")", "2-4-6"),
    ("[1, 2, 3, 4].filter(x => x % 2 == 0).length", "2"),
    ("(2.5).toFixed(2)", "2.50"),
    ("if 1 < 2 { \"yes\" } else { \"no\" }", "yes"),
    ("!\"\"", "true"),
    ("Math.max(3, 7)", "7"),
    ("items.length", "2"),
    ("items[0].title", "First"),
    ("items.map(i => i.title).join(\", \")", "First, Second"),
];

fn data() -> Value {
    json!({ "name": "مرحبا", "items": [{ "title": "First" }, { "title": "Second" }] })
}

/// `src` parsed as the expression of a page's one `Text`.
fn expr_of(src: &str) -> Expr {
    let program = parse_v2(&format!("page P(path: \"/\") {{ Text({src}) }}"), "x.wf")
        .unwrap_or_else(|e| panic!("{src}: {e}"));
    let Declaration::Page(page) = &program.declarations[0] else {
        unreachable!()
    };
    let StatementKind::UIElement(ui) = &page.body[0].kind else {
        unreachable!()
    };
    let Arg::Positional(e) = &ui.args[0] else {
        unreachable!()
    };
    e.clone()
}

fn by_the_build(src: &str) -> Option<String> {
    let names: Vec<(String, Static)> = match data() {
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| (k.clone(), Static::from_json(v)))
            .collect(),
        _ => unreachable!(),
    };
    eval(&expr_of(src), &Scope::of(names)).map(|v| v.to_text())
}

fn by_the_template(src: &str) -> String {
    let page = format!("page P(path: \"/\") {{\n    derived v = {src}\n    Text(\"⟦{{v}}⟧\")\n}}");
    let html = render(&page, data());
    let start = html.find('⟦').unwrap() + '⟦'.len_utf8();
    let end = html.find('⟧').unwrap();
    html[start..end].to_string()
}

#[test]
fn the_template_and_the_build_agree_on_every_expression_of_the_corpus() {
    let mut wrong = Vec::new();
    for (src, shown) in CORPUS {
        let template = by_the_template(src);
        if template != *shown {
            wrong.push(format!("template: {src} → {template:?}, wants {shown:?}"));
        }
        if let Some(build) = by_the_build(src)
            && build != *shown
        {
            wrong.push(format!("build: {src} → {build:?}, wants {shown:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn the_build_works_out_what_the_review_named() {
    // These the build knows; the template must give the same.
    for src in ["name.length", "\"مرحبا\".length", "[10, 20, 30][1]"] {
        assert!(by_the_build(src).is_some(), "{src}");
    }
    // These the build leaves to the page; the template still answers as
    // the browser would.
    for src in ["\"12\" < \"9\"", "[10, 20, 30][1.5]", "\"x\".length()"] {
        assert_eq!(by_the_build(src), None, "{src}");
    }
    assert_eq!(by_the_template("\"x\".length()"), "");
}

// ─── B2: components that place each other ────────────────────────────

#[test]
fn a_component_that_places_itself_forever_is_e120() {
    let src = r#"component Loop(n: Number) { Stack { Text("{n}") Loop(n: n + 1) } }
page P(path: "/") { Loop(n: 0) }"#;
    let tpl = Template::from_str(src).unwrap();
    let err = within(60, move || tpl.render_html_fragment(&json!({})).unwrap_err());
    assert_eq!(codes(&err), vec!["E120"]);
}

#[test]
fn a_component_that_places_itself_as_deep_as_its_data_draws() {
    let src = r#"component Node(item: Map) {
    Stack {
        Text(item.name)
        for c in item.children { Node(item: c) }
    }
}
page P(path: "/") { Node(item: tree) }"#;
    // Each level draws a component and a `Stack`: thirty deep is sixty
    // levels, under the limit.
    let mut tree = json!({ "name": "leaf", "children": [] });
    for i in 0..30 {
        tree = json!({ "name": format!("n{i}"), "children": [tree] });
    }
    let tpl = Template::from_str(src).unwrap();
    let html = within(60, move || tpl.render_html_fragment(&json!({ "tree": tree })).unwrap());
    assert!(html.contains("leaf") && html.contains("n29"), "{html}");
    // Sixty deep is past it, and says so.
    let mut tree = json!({ "name": "leaf", "children": [] });
    for i in 0..60 {
        tree = json!({ "name": format!("n{i}"), "children": [tree] });
    }
    let tpl = Template::from_str(src).unwrap();
    let err = within(60, move || tpl.render_html_fragment(&json!({ "tree": tree })).unwrap_err());
    assert_eq!(codes(&err), vec!["E120"]);
}

// ─── M12: a render-wide budget ───────────────────────────────────────

#[test]
fn nested_loops_over_vast_ranges_stop_with_e121() {
    // The reviewer's: two ranges of a billion, nested.
    let src = r#"page P(path: "/") {
    for i in 1..=1000000000 {
        for j in 1..=1000000000 {
            Text("x")
        }
    }
}"#;
    let tpl = Template::from_str(src).unwrap();
    let err = within(60, move || tpl.render_html_fragment(&json!({})).unwrap_err());
    assert_eq!(codes(&err), vec!["E121"]);
}

#[test]
fn nested_loops_over_the_data_stop_with_e121_past_the_budget() {
    // Four thousand by four thousand items is sixteen million loop steps,
    // more than one render may take; the data alone is small.
    let src = r#"page P(path: "/") {
    for a in xs { for b in xs { } }
}"#;
    let xs: Vec<u32> = (0..4000).collect();
    let tpl = Template::from_str(src).unwrap();
    let err = within(300, move || tpl.render_html_fragment(&json!({ "xs": xs })).unwrap_err());
    assert_eq!(codes(&err), vec!["E121"]);
}

#[test]
fn a_large_render_under_the_budget_draws_whole() {
    let src = r#"page P(path: "/") {
    Table {
        Table.Head { Table.Row { Table.Cell("n") Table.Cell("double") } }
        Table.Body {
            for r in rows { Table.Row { Table.Cell("{r}") Table.Cell("{r * 2}") } }
        }
    }
}"#;
    let rows: Vec<u32> = (0..5000).collect();
    let tpl = Template::from_str(src).unwrap();
    let html = within(120, move || tpl.render_html_fragment(&json!({ "rows": rows })).unwrap());
    assert!(html.contains(">9998<"), "the last row is drawn");
}

#[test]
fn an_expression_that_would_make_too_much_stops_with_e121() {
    for e in [
        "(1..=1000000).map(i => 1..=1000000).length",
        "(1..=100000).map(i => big).length",
        "big.split(\"\").join(big).length",
    ] {
        let src = format!("page P(path: \"/\") {{\n    derived v = {e}\n    Text(\"{{v}}\")\n}}");
        let tpl = Template::from_str(&src).unwrap();
        let big = "x".repeat(100_000);
        let err = within(60, move || tpl.render_html_fragment(&json!({ "big": big })).unwrap_err());
        assert_eq!(codes(&err), vec!["E121"], "{e}");
    }
}

// ─── M3, M5: what the data says cannot stop the render ───────────────

#[test]
fn deeply_nested_quotes_in_markdown_render_as_text_past_the_cap() {
    let src = r#"page P(path: "/") { Markdown(md) }"#;
    let tpl = Template::from_str(src).unwrap();
    let md = format!("{}deep", ">".repeat(5000));
    let html = within(60, move || tpl.render_html_fragment(&json!({ "md": md })).unwrap());
    assert_eq!(html.matches("<blockquote>").count(), 32, "{}", &html[..400]);
    assert!(html.contains("&gt;"), "what is past the cap is text");
    assert!(html.contains("deep"));
}

#[test]
fn a_precision_from_the_data_cannot_panic_the_render() {
    let src = r#"page P(path: "/") {
    Text("[{(1.5).toFixed(n)}]")
    Text("[{format(1.5, .decimal, n)}]")
}"#;
    let tpl = Template::from_str(src).unwrap();
    for n in [70000, 1000000000, -1, 101] {
        let tpl = tpl.clone();
        let html = within(60, move || tpl.render_html_fragment(&json!({ "n": n })).unwrap());
        // `toFixed` past 100 is a RangeError in the browser: nothing shown.
        assert!(html.contains("[]"), "{n}: {html}");
    }
    let html = tpl.render_html_fragment(&json!({ "n": 100 })).unwrap();
    assert!(html.contains(&format!("[1.5{}]", "0".repeat(99))), "{html}");
    assert!(html.contains(&format!("1.5{}", "0".repeat(99))), "{html}");
    let html = tpl.render_html_fragment(&json!({ "n": 2 })).unwrap();
    assert!(html.contains("[1.50]"), "{html}");
}
