//! What a build weighs, reported rather than enforced.
//!
//! A size budget that fails a build is a decision for a project, not for the
//! compiler: a page that needs a carousel needs the carousel. So this test
//! prints what each shape of program carries — run it with `cargo test --test
//! bundle_size -- --nocapture` — and asserts only what must never stop being
//! true: that a program which reaches for nothing does not pay for everything,
//! and that a program which reaches for something gets it.

mod common;

/// One program, and the runtime it draws.
fn weigh(label: &str, src: &str) -> (usize, Vec<String>) {
    let js = common::spa_js(src);
    let runtime = js.split("\n})();\n").next().unwrap_or("").len();
    let kept: Vec<String> = webfluent::runtime::MODULES
        .iter()
        .filter(|m| {
            m.exports
                .iter()
                .any(|(_, local)| js.contains(&format!("function {local}(")))
                || (m.name == "core" && js.contains("function el("))
                || (m.name == "icons" && js.contains("const _ICONS = {"))
        })
        .map(|m| m.name.to_string())
        .collect();
    println!(
        "  {label:<22} {:>8.1} kB  {}",
        runtime as f64 / 1024.0,
        kept.join(" ")
    );
    (runtime, kept)
}

#[test]
fn a_program_carries_what_it_reaches_for() {
    let full = webfluent::runtime::full().len();
    println!(
        "\nRuntime carried, by what the program does (full: {:.1} kB)",
        full as f64 / 1024.0
    );

    let (bare, _) = weigh(
        "one static page",
        r#"page Home(path: "/", title: "H", description: "A page.") { Container { Text("hi") } }"#,
    );
    let (counter, _) = weigh(
        "a counter",
        r#"page Home(path: "/", title: "H", description: "A page.") {
            state n = 0
            Button("+1").primary { on click { n = n + 1 } }
            Text("{n}")
        }"#,
    );
    let (list, list_kept) = weigh(
        "a keyed list",
        r#"page Home(path: "/", title: "H", description: "A page.") {
            state items: [String] = ["a"]
            for it in items by it { Text(it) }
        }"#,
    );
    let (routed, routed_kept) = weigh(
        "two pages and a nav",
        r#"app { Navbar(brand: "X") { Navbar.Links { Link("Home", to: "/") } } Router }
           page Home(path: "/", title: "H", description: "A page.") { Text("home") }
           page About(path: "/about", title: "A", description: "Another page.") { Text("about") }"#,
    );
    let (rich, rich_kept) = weigh(
        "a form, a fetch, a chart of icons",
        r#"store S { state x = 1  action bump() { x = x + 1 } }
           page Home(path: "/", title: "H", description: "A page.") {
               use S
               state q = ""
               resource rows = fetch("/api/rows")
               Form { on submit { S.bump() }  Input(bind: q, label: "Query").search }
               Icon("search")
               match rows { loading { Spinner } error(e) { Alert("{e.message}").danger } ready(r) { Text("{r.length}") } }
           }"#,
    );

    assert!(
        bare < full / 2,
        "a page that does nothing must not carry everything"
    );
    assert!(counter >= bare, "reactivity is in every build");
    assert!(
        list_kept.contains(&"each".to_string()),
        "a `for` gets `each`"
    );
    assert!(list < routed, "a list is lighter than a router");
    assert!(routed_kept.contains(&"router".to_string()));
    assert!(
        rich_kept.contains(&"net".to_string()),
        "a resource gets the network"
    );
    assert!(
        rich_kept.contains(&"icons".to_string()),
        "an Icon gets the icon table"
    );
    assert!(rich <= full, "no build carries more than the whole runtime");
}
