//! Reads that answer more than once: an endpoint that says how it pages
//! (`pages .number(param: page)`, `.cursor(…)`, `.link`), one that answers
//! a line at a time (`streamed`), and what each read holds — the list it
//! gathers. And what a write kept offline tells its caller. What the
//! runtime does is `tests/js/pages.test.mjs`, `tests/js/offline_queue.test.mjs`
//! and `tests/browser/paging.mjs`.

use crate::common::spa_generated;

fn diagnostics(src: &str) -> Vec<String> {
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
    .filter(|d| d.severity == webfluent::diagnostics::Severity::Error)
    .map(|d| format!("[{}] {}", d.code, d.message))
    .collect()
}

const TYPES: &str = "type Post { id: String, title: String }\ntype FeedPage { posts: [Post], nextCursor: String? }\ntype Line { n: Number, text: String }\n";

fn with_api(endpoints: &str, body: &str) -> String {
    format!(
        "{TYPES}api Social(base: \"/api\") {{\n{endpoints}\n}}\napp {{ Router }}\npage Home(path: \"/\", title: \"Home\", description: \"d\") {{\n    Heading(\"Home\").h1\n{body}\n}}\n"
    )
}

const GOOD: &str = "    get feed(after: String?) at \"feed\" -> FeedPage!\n        pages .cursor(param: after, next: p => p.nextCursor, items: p => p.posts)\n    get list(page: Number = 1) at \"posts\" -> [Post]!\n        pages .number(param: page)\n    get search(q: String) at \"search\" -> [Post]!\n        pages .link\n    get tail(id: String) at \"logs/{id}\" -> [Line]! streamed";

#[test]
fn each_way_of_paging_is_a_read_of_what_it_gathers() {
    let src = with_api(
        GOOD,
        "    resource feed = Social.feed()\n    resource list = Social.list()\n    resource found = Social.search(q: \"a\")\n    resource lines = Social.tail(id: \"x\")\n    for p in feed.items by p.id { Text(p.title) }\n    for p in feed.value ?? [] by p.id { Text(p.title) }\n    match list {\n        .loaded(posts) { for p in posts by p.id { Text(p.title) } }\n        else { Spinner }\n    }\n    for p in found.items by p.id { Text(p.title) }\n    for l in lines.value ?? [] by l.n { Text(l.text) }\n    if feed.hasMore { Button(\"More\") { on click { feed.loadMore() } } }",
    );
    assert_eq!(diagnostics(&src), Vec::<String>::new());
    // What a page holds is read as its type: a misspelt field is found.
    let wrong = src.replace(
        "for p in feed.items by p.id { Text(p.title) }",
        "for p in feed.items by p.id { Text(p.titel) }",
    );
    assert!(
        diagnostics(&wrong)
            .iter()
            .any(|d| d.contains("[T05]") && d.contains("titel")),
        "{:?}",
        diagnostics(&wrong)
    );
}

#[test]
fn a_contract_that_cannot_page_says_why() {
    let cases = [
        // A cursor with nowhere to start the next page.
        (
            "    get a(after: String?) at \"a\" -> [Post]!\n        pages .cursor(param: after)",
            "says nothing of where the next page starts",
        ),
        // A first page that cannot be asked for without its parameter.
        (
            "    get a(after: String) at \"a\" -> [Post]!\n        pages .cursor(param: after, next: p => null)",
            "must be given",
        ),
        // A parameter the endpoint does not have.
        (
            "    get a(page: Number = 1) at \"a\" -> [Post]!\n        pages .number(param: pg)",
            "has no parameter `pg`",
        ),
        // A page that is not a list, with nothing saying where its items are.
        (
            "    get a() at \"a\" -> FeedPage!\n        pages .link",
            "A page is a list of what it holds",
        ),
        // A page number that is not a number.
        (
            "    get a(page: String = \"1\") at \"a\" -> [Post]!\n        pages .number(param: page)",
            "counts pages in `page`",
        ),
        // A style there is not.
        (
            "    get a() at \"a\" -> [Post]!\n        pages .scroll",
            "is not a way to page",
        ),
        // A write.
        (
            "    post a(page: Number = 1) at \"a\" -> [Post]!\n        pages .number(param: page)",
            "only a read is gathered a page at a time",
        ),
        // A streamed answer that is not a list, and a streamed write.
        (
            "    get a() at \"a\" -> Post! streamed",
            "it answers a list, one item a line",
        ),
        (
            "    post a() at \"a\" -> [Post]! streamed",
            "only a read is gathered a line at a time",
        ),
    ];
    for (endpoint, says) in cases {
        let found = diagnostics(&with_api(endpoint, ""));
        assert!(
            found
                .iter()
                .any(|d| d.starts_with("[N14]") && d.contains(says)),
            "{endpoint}: {found:?}"
        );
    }
}

#[test]
fn paginate_on_a_call_is_6_x_s() {
    let src = with_api(
        "    get list(page: Number = 1) at \"posts\" -> [Post]!",
        "    resource list = Social.list(paginate: .page)\n    for p in list.items by p.id { Text(p.title) }",
    );
    let found = diagnostics(&src);
    assert!(
        found
            .iter()
            .any(|d| d.starts_with("[N08]") && d.contains("`paginate:`")),
        "{found:?}"
    );
}

#[test]
fn for_await_walks_a_streamed_endpoint_s_lines_typed() {
    let body = "    state seen: [String] = []\n    Button(\"Walk\") {\n        on click {\n            for await line, i in Social.tail(id: \"x\") { seen = seen.concat([\"{i}: {line.text}\"]) }\n        }\n    }\n    for s in seen by s { Text(s) }";
    let src = with_api(GOOD, body);
    assert_eq!(diagnostics(&src), Vec::<String>::new());
    let wrong = src.replace("line.text", "line.txt");
    assert!(
        diagnostics(&wrong)
            .iter()
            .any(|d| d.contains("[T05]") && d.contains("txt")),
        "{:?}",
        diagnostics(&wrong)
    );
    // Only a streamed endpoint's lines are walked.
    let not = src.replace("Social.tail(id: \"x\")", "Social.list()");
    assert!(
        diagnostics(&not)
            .iter()
            .any(|d| d.starts_with("[N14]") && d.contains("`for await`")),
        "{:?}",
        diagnostics(&not)
    );
    let js = spa_generated(&src);
    assert!(js.contains("for await (const line of Social.tail("), "{js}");
}

#[test]
fn an_endpoint_says_how_it_pages_to_the_engine() {
    let src = with_api(
        GOOD,
        "    resource feed = Social.feed()\n    for p in feed.items by p.id { Text(p.title) }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("pages: { style: \"cursor\", param: \"after\", next: ((p) => p.nextCursor), items: ((p) => p.posts) }"),
        "{js}"
    );
    assert!(
        js.contains("pages: { style: \"number\", param: \"page\" }"),
        "{js}"
    );
    assert!(
        js.contains("pages: { style: \"link\" }, withHeaders: true"),
        "{js}"
    );
    assert!(js.contains("streamed: true"), "{js}");
    // A read writes no state of the page's.
    assert!(
        js.contains("WF.resource(Social.feed, { call: true, args: () => ({  }) })"),
        "{js}"
    );
}

#[test]
fn pages_streamed_and_for_await_print_as_written() {
    let src = with_api(
        GOOD,
        "    Button(\"Walk\") { on click { for await line in Social.tail(id: \"x\") { log(line.text) } } }",
    );
    let program = webfluent::parse_source(&src, "t.wf").expect("parses");
    let printed: String = program
        .declarations
        .iter()
        .map(|d| webfluent::syntax::print::declaration(d, &program))
        .collect::<Vec<_>>()
        .join("\n");
    for written in [
        "pages .cursor(param: after, next: p => p.nextCursor, items: p => p.posts)",
        "pages .number(param: page)",
        "pages .link",
        "-> [Line]! streamed",
        "for await line in Social.tail(id: \"x\")",
    ] {
        assert!(printed.contains(written), "{written}\n{printed}");
    }
    let again = webfluent::parse_source(&printed, "t.wf").expect("what it prints parses");
    assert_eq!(again.declarations.len(), program.declarations.len());
}

#[test]
fn a_write_kept_offline_is_a_failure_that_says_so() {
    let src = with_api(
        "    put save(id: String, body: Post) at \"posts/{id}\" -> Post!",
        "    state said = \"\"\n    Button(\"Save\") {\n        on click {\n            match await Social.save(id: \"1\", body: Post(id: \"1\", title: \"t\")) {\n                .ok(p) { said = p.title }\n                .err(.offline(true)) { said = \"Saved here; it will be sent.\" }\n                .err(.offline(_)) { said = \"No connection.\" }\n                .err(e) { said = e.message }\n            }\n        }\n    }\n    Text(said)\n    for r in network.refused by r.at { Text(\"{r.method} {r.url}: {r.status}\") }\n    if network.refused.length > 0 { Button(\"Dismiss\") { on click { network.clearRefused() } } }",
    );
    assert_eq!(diagnostics(&src), Vec::<String>::new());
    let wrong = src.replace("r.status", "r.code");
    assert!(
        diagnostics(&wrong)
            .iter()
            .any(|d| d.contains("[T05]") && d.contains("code")),
        "{:?}",
        diagnostics(&wrong)
    );
}
