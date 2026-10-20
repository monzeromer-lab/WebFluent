//! A store's `resource`: made with the store, outside any page, and read as
//! signals — `Feed.rows.state` from a page, `rows.data` in the store's own
//! actions and derived values. It used to be dropped from the store, and
//! its name compiled to a bare identifier nothing declared. What the
//! runtime does is `tests/js/store.test.mjs` and
//! `tests/browser/store_resource.mjs`.

mod common;

use common::{page, spa_generated, ssg_html};

fn diagnostics(src: &str, code: &str) -> Vec<String> {
    let (program, out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
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
    .diagnostics
    .into_iter()
    .filter(|d| code.is_empty() || d.code == code)
    .map(|d| format!("[{}] {}", d.code, d.message))
    .collect()
}

const FEED: &str = "store Feed {
    state page = 1
    resource rows = fetch(\"/api/rows?page={page}\", on: .focus)
    derived count = rows.data?.length ?? 0
    derived loaded = rows.state == \"ready\"
    action next() { page = page + 1 }
    action again() { rows.reload() }
    action peek(rows: Map) { page = rows.data }
}
page P(path: \"/\", title: \"T\") {
    Heading(\"Feed\").h1
    Text(Feed.rows.state)
    Text(\"{Feed.count} {Feed.loaded}\")
    if Feed.rows.refreshing { Text(\"Refreshing\") }
    match Feed.rows {
        loading { Text(\"Loading\") }
        error(e) { Text(e.message) }
        ready(rows) { for r in rows { Text(r.name) } }
    }
    Button(\"Next\") { on click { Feed.next()  Feed.again()  Feed.peek({ data: 2 }) } }
}
";

#[test]
fn a_store_makes_its_resource_and_reads_it_in_its_own_terms() {
    let js = spa_generated(FEED);
    assert!(js.contains("resources: {"), "{js}");
    assert!(
        js.contains(
            "rows: (store) => WF.resource(() => `/api/rows?page=${encodeURIComponent(store.page)}`, { on: { focus: true } }),"
        ),
        "the address reads the store's state, and is asked again when it changes: {js}"
    );
    assert!(
        js.contains("count: (store) => (store.rows.data()?.length ?? 0),"),
        "a derived value reads what it holds as a signal: {js}"
    );
    assert!(
        js.contains("loaded: (store) => (store.rows.state() === \"ready\"),"),
        "{js}"
    );
    assert!(js.contains("store.rows.reload();"), "{js}");
    assert!(
        !js.contains("_rows"),
        "nothing reads a page's resource that is not there: {js}"
    );
}

#[test]
fn a_parameter_named_like_the_resource_is_the_parameter() {
    let js = spa_generated(FEED);
    assert!(
        js.contains("store.page = rows.data;"),
        "`peek(rows)`'s `rows` is its argument: {js}"
    );
}

#[test]
fn a_page_reads_a_store_s_resource_as_signals() {
    let js = spa_generated(FEED);
    assert!(js.contains("Feed.rows.state()"), "{js}");
    assert!(js.contains("Feed.rows.refreshing()"), "{js}");
    assert!(
        js.contains(
            "() => Feed.rows.state(), () => Feed.rows.state() === \"error\" ? Feed.rows.error() : Feed.rows.data(), {"
        ),
        "a match over it is chosen by its state: {js}"
    );
    // `Text(Feed.rows.state)` follows the state: it is read in a function.
    assert!(
        js.contains("() => Feed.rows.state()"),
        "what is shown follows it: {js}"
    );
}

#[test]
fn only_the_store_that_declares_it_has_the_resource() {
    let src = "store Feed {\n    resource rows = fetch(\"/api/rows\")\n}\nstore Other {\n    state rows = { state: \"x\" }\n}\npage P(path: \"/\", title: \"T\") {\n    Text(\"{Feed.rows.state} {Other.rows.state}\")\n}\n";
    let js = spa_generated(src);
    assert!(js.contains("Feed.rows.state()"), "{js}");
    assert!(
        js.contains("Other.rows.state}") || js.contains("Other.rows.state)"),
        "another store's state of the same name is a plain value: {js}"
    );
    assert!(!js.contains("Other.rows.state()"), "{js}");
    assert!(
        js.contains("rows: (store) => WF.resource(\"/api/rows\", null),"),
        "an address written out whole is not a function: {js}"
    );
}

#[test]
fn a_store_s_endpoint_resource_takes_its_arguments_from_the_store() {
    let src = "type Item { id: String, name: String }\napi Catalog(base: \"/api\") {\n    get items(page: Number = 1) -> [Item]\n}\nstore Shelf {\n    state page = 1\n    resource list = Catalog.items(page: page, paginate: .page, on: .interval(30000), cache: .swr(60000))\n    derived names = list.items.map(i => i.name)\n}\npage P(path: \"/\", title: \"T\") {\n    for item in Shelf.list.items by item.id { Text(item.name) }\n    if Shelf.list.hasMore { Button(\"More\") { on click { Shelf.list.loadMore() } } }\n    Text(\"{Shelf.names.length}\")\n}\n";
    let js = spa_generated(src);
    assert!(
        js.contains("list: (store) => WF.resource(Catalog.items, { call: true, args: () => ({ page: store.page, cache: { kind: \"swr\", ttl: 60000 } }), on: { interval: 30000 }, paginate: { by: \"page\", set: (v) => { store.page = v; } } }),"),
        "{js}"
    );
    assert!(js.contains("store.list.items().map("), "{js}");
    assert!(js.contains("Shelf.list.items()"), "{js}");
    assert!(js.contains("Shelf.list.hasMore()"), "{js}");
    assert!(js.contains("Shelf.list.loadMore()"), "{js}");
}

#[test]
fn a_page_s_own_resource_is_as_it_was() {
    let js = spa_generated(&page(
        "resource rows = fetch(\"/api/rows\")\n    Text(rows.state)",
    ));
    assert!(
        js.contains("const _rows = WF.resource(\"/api/rows\", null);"),
        "{js}"
    );
    assert!(js.contains("_rows.state()"), "{js}");
}

#[test]
fn the_checker_accepts_a_store_s_resource() {
    let found = diagnostics(FEED, "");
    let errors: Vec<&String> = found
        .iter()
        .filter(|d| d.starts_with("[T") || d.starts_with("[X") || d.starts_with("[E"))
        .collect();
    assert!(errors.is_empty(), "{found:?}");
}

#[test]
fn the_checker_reads_a_store_s_resource_as_one() {
    let src = "type Row { name: String }\nstore Feed {\n    resource rows: [Row] = fetch(\"/api/rows\")\n    resource one: Row = fetch(\"/api/one\")\n}\npage P(path: \"/\", title: \"T\") {\n    Heading(\"Feed\").h1\n    Text(Feed.one.data?.nam ?? \"\")\n    Text(Feed.rowz.state)\n    Button(\"Wipe\") { on click { Feed.rows = null } }\n}\n";
    let found = diagnostics(src, "");
    assert!(
        found.iter().any(|d| d == "[T05] `Row` has no field `nam`"),
        "its data is what it declares: {found:?}"
    );
    assert!(
        found
            .iter()
            .any(|d| d == "[T06] `Feed` has no member `rowz`"),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|d| d == "[X01] `Feed.rows` is a `resource`, and nothing may assign to it"),
        "a page may not replace it, as the store may not: {found:?}"
    );
}

#[test]
fn a_pre_rendered_page_shows_a_store_s_resource_loading() {
    let html = ssg_html(FEED);
    assert!(
        html.contains("<p class=\"wf-text\">Loading</p>"),
        "nothing has arrived at build time: {html}"
    );
    assert!(
        !html.contains("undefined") && !html.contains("[object"),
        "{html}"
    );
}
