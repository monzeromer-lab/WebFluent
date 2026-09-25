//! Strings that do not fight the author: the raw and block forms, the
//! formatted splice, and the escaped brace that no longer travels as a
//! private sentinel.
//!
//! Every case is held to what the reader sees — the static paint — because
//! that is where a string that is escaped twice, or dedented wrongly, shows.

mod common;

use common::{Backend, raw_output, spa_generated};
use webfluent::parse_source;

fn page(body: &str) -> String {
    format!("page P(path: \"/\", title: \"T\", description: \"D\") {{\n{body}\n}}\n")
}

/// The text between the tags of the painted page.
fn painted(body: &str) -> String {
    let html = raw_output(Backend::Ssg, &page(body));
    let main = html
        .split_once("<main")
        .and_then(|(_, r)| r.split_once("</main>"))
        .map(|(m, _)| m.to_string())
        .unwrap_or(html);
    let mut text = String::new();
    let mut inside = false;
    for c in main.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            c if !inside => text.push(c),
            _ => {}
        }
    }
    text
}

#[test]
fn a_raw_string_holds_quotes_and_braces_as_written() {
    let out = painted(r##"Text(#"page Home(path: "/") { state n = 0 }"#)"##);
    assert!(
        out.contains("page Home(path: &quot;/&quot;) { state n = 0 }"),
        "a raw string is what is written, escaped only for HTML: {out}"
    );
    // As many hashes as the text needs.
    let out = painted(r###"Text(##"a "# inside"##)"###);
    assert!(out.contains("a &quot;# inside"), "{out}");
}

#[test]
fn a_raw_string_has_no_splices() {
    let out = painted(
        r##"state n = 1
    Text(#"{n} stays"#)"##,
    );
    assert!(
        out.contains("{n} stays"),
        "no splice in a raw string: {out}"
    );
}

#[test]
fn a_block_string_loses_the_indentation_the_source_gave_it() {
    let src = "page P(path: \"/\", title: \"T\", description: \"D\") {\n    state name = \"Ada\"\n    Text(\"\"\"\n        Dear {name},\n          thank you.\n        \"\"\")\n}\n";
    let html = raw_output(Backend::Ssg, src);
    assert!(
        html.contains("Dear Ada,\n  thank you."),
        "the closing delimiter says how far the text was indented: {html}"
    );
}

#[test]
fn an_escaped_brace_is_a_brace_everywhere() {
    for backend in [Backend::Spa, Backend::Ssg, Backend::Template] {
        let out = raw_output(backend, &page(r#"Text("function() \{ return 42 \}")"#));
        assert!(
            out.contains("function() { return 42 }"),
            "{backend:?} kept an escape or a sentinel: {out}"
        );
        assert!(
            !out.contains('\u{FFFE}') && !out.contains('\u{FFFF}'),
            "{backend:?} still carries the old placeholder"
        );
    }
}

#[test]
fn a_formatted_splice_is_the_format_call_written_where_it_is_read() {
    let out = painted(
        r#"state total = 1234.5
    state count = 1235
    Text("Total: {total:.currency}, {count:.integer} items")"#,
    );
    assert!(out.contains("Total: $1,234.50, 1,235 items"), "{out}");

    // With an option: the style, then what it takes.
    let out = painted(
        r#"state when = "2026-03-05T10:00:00Z"
    Text("Shipped {when:.date(long)}")"#,
    );
    assert!(out.contains("Shipped March 5, 2026"), "{out}");

    // And it is the same call, so it follows state in the browser too.
    let js = spa_generated(&page(
        r#"state total = 1.0
    Text("{total:.currency}")"#,
    ));
    assert!(
        js.contains("WF.format("),
        "the splice compiles to a format call"
    );
}

#[test]
fn a_colon_that_is_not_a_style_is_still_text() {
    let out = painted(r#"Text("a note: {not an expression, really}")"#);
    assert!(out.contains("a note: {not an expression, really}"), "{out}");
}

#[test]
fn the_new_string_helpers_run_at_build_time_and_in_the_browser() {
    let out = painted(
        r#"derived rows = "  a\n  b".dedent().lines().join("|")
    derived n = "one two  three".words().length
    Text("{rows} / {n}")"#,
    );
    assert!(out.contains("a|b / 3"), "{out}");

    let js = spa_generated(&page(
        r#"state text = "x"
    Text(text.dedent())"#,
    ));
    assert!(
        js.contains("WF.dedent("),
        "the helper is the runtime's: {js}"
    );
}

#[test]
fn the_checker_knows_what_the_new_methods_return() {
    // `lines()` is a list of strings, so `.length` is a number and
    // `.join` is there; a misspelling is still an error.
    let good = page(
        r#"state text = "a"
    Text("{text.trimStart().padStart(3, \" \")} {text.lines().length}")"#,
    );
    assert!(parse_source(&good, "t.wf").is_ok());
}

// ─── The types the language brings with it ───────────────

#[test]
fn the_literals_carry_plain_values() {
    let js = spa_generated(&page(
        r#"state due: Date = @2026-03-14
    state at: Time = @09:30
    state stamp: DateTime = @2026-03-14T09:30:00Z
    state wait: Duration = 3.days
    state price: Money = €12.99
    state brand: Color = #0F766E
    Text("{due} {at} {stamp} {wait} {brand} {price.currency}")"#,
    ));
    // Nothing is wrapped: each is the plain JSON value it is carried by,
    // so it crosses a fetch, a persist and the static paint unchanged.
    assert!(js.contains("\"2026-03-14\""), "{js}");
    assert!(js.contains("\"09:30\""));
    assert!(js.contains("\"2026-03-14T09:30:00Z\""));
    assert!(js.contains("259200000"), "3 days, in milliseconds");
    assert!(js.contains("\"#0F766E\""));
    assert!(js.contains("amount: 1299"), "money is minor units");
    assert!(js.contains("currency: \"EUR\""));
}

#[test]
fn the_arithmetic_is_done_at_build_time() {
    let out = painted(
        r#"state due: Date = @2026-03-14
    state price: Money = €12.99
    state brand: Color = #0F766E
    derived checkout = due.plus(days: 5)
    derived weekday = due.weekday()
    derived soon = due.isBefore(@2026-06-01)
    derived tint = brand.mix(#FFFFFF, 0.2)
    Text("{checkout} {weekday} {soon} {tint} {format(price.times(3))}")"#,
    );
    assert!(
        out.contains("2026-03-19 6 true #3F918B €38.97"),
        "the static paint shows the answer, not a blank that fills in later: {out}"
    );
}

#[test]
fn a_month_lands_on_a_day_that_exists() {
    let out = painted(
        r#"derived end = @2026-01-31.plus(months: 1)
    Text("{end}")"#,
    );
    assert!(out.contains("2026-02-28"), "{out}");
}

#[test]
fn money_formats_itself_without_a_style_or_a_code() {
    let out = painted(
        r#"state price: Money = €12.99
    Text("{format(price)}")"#,
    );
    assert!(out.contains("€12.99"), "{out}");
}

#[test]
fn a_condition_on_a_type_is_checked_where_the_value_is_written() {
    for (source, said) in [
        ("state nights: Number(1..=30) = 45", "is above 30"),
        (
            "state pass: String(minLength: 8) = \"short\"",
            "is shorter than 8",
        ),
        (
            "state start: Date(after: @2026-01-01) = @2025-06-01",
            "is not after 2026-01-01",
        ),
    ] {
        let src = page(&format!("{source}\n    Text(\"x\")"));
        let errors = webfluent::sema::types::check(&parse_source(&src, "t.wf").unwrap(), &|_| {
            "t.wf".to_string()
        })
        .findings
        .errors;
        let text = errors.iter().map(|e| e.to_string()).collect::<String>();
        assert!(text.contains(said), "`{source}` must be refused: {text}");
        assert!(text.contains("[T01]"), "{text}");
    }
    // And a value inside the condition is not.
    let src = page("state nights: Number(1..=30) = 3\n    Text(\"{nights}\")");
    assert!(
        webfluent::sema::types::check(&parse_source(&src, "t.wf").unwrap(), &|_| "t.wf".into())
            .findings
            .errors
            .is_empty()
    );
}

#[test]
fn a_secret_cannot_reach_the_page_the_log_or_the_browser_store() {
    let cases = [
        ("state t: Secret = \"x\"\n    Text(t)", "would show it"),
        (
            "state t: Secret = \"x\"\n    Text(\"Bearer {t}\")",
            "puts it in text",
        ),
        (
            "persist t: Secret = \"x\"\n    Text(\"x\")",
            "would keep it in the browser",
        ),
        (
            "state t: Secret = \"x\"\n    Button(\"go\") { on click { log(t) } }",
            "would log it",
        ),
    ];
    for (body, said) in cases {
        let src = page(body);
        let errors = webfluent::sema::types::check(&parse_source(&src, "t.wf").unwrap(), &|_| {
            "t.wf".to_string()
        })
        .findings
        .errors;
        let text = errors.iter().map(|e| e.to_string()).collect::<String>();
        assert!(text.contains(said), "`{body}` must be refused: {text}");
        assert!(text.contains("[T12]"), "{text}");
    }
}

#[test]
fn a_type_the_program_declares_keeps_its_name() {
    // Nothing the language adds takes a name away: a project's own `Color`
    // is the one its program means.
    let src = format!(
        "type Color {{ name: String }}\n{}",
        page("state c: Color = Color(name: \"teal\")\n    Text(c.name)")
    );
    let errors = webfluent::sema::types::check(&parse_source(&src, "t.wf").unwrap(), &|_| {
        "t.wf".to_string()
    })
    .findings
    .errors;
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn the_checker_knows_what_each_type_can_do() {
    let bad = page("state due: Date = @2026-03-14\n    Text(\"{due.hostname()}\")");
    let errors = webfluent::sema::types::check(&parse_source(&bad, "t.wf").unwrap(), &|_| {
        "t.wf".to_string()
    })
    .findings
    .errors;
    let text = errors.iter().map(|e| e.to_string()).collect::<String>();
    assert!(
        text.contains("`Date` has no method `hostname`"),
        "got: {text:?}"
    );
    assert!(
        text.contains(".plus(days: 1)"),
        "the hint lists what it has: {text}"
    );

    // And a date picker picks a date.
    let bad = page("state when = \"2026-01-01\"\n    DatePicker(bind: when, label: \"When\")");
    let errors = webfluent::sema::types::check(&parse_source(&bad, "t.wf").unwrap(), &|_| {
        "t.wf".to_string()
    })
    .findings
    .errors;
    let text = errors.iter().map(|e| e.to_string()).collect::<String>();
    assert!(text.contains("`bind:` on `DatePicker`"), "{text}");
    assert!(
        text.contains("`Date?` is wanted"),
        "an empty picker holds nothing: {text}"
    );
}

#[test]
fn a_literal_is_held_to_the_shape_of_the_type_it_is_written_for() {
    for (source, said) in [
        ("state site: Url = \"example\"", "is not a `Url`"),
        ("state who: Email = \"ada\"", "is not an `Email`"),
        ("state due: Date = \"14/03/2026\"", "is not a `Date`"),
        ("state brand: Color = \"#GGG\"", "is not a `Color`"),
    ] {
        let src = page(&format!("{source}\n    Text(\"x\")"));
        let errors = webfluent::sema::types::check(&parse_source(&src, "t.wf").unwrap(), &|_| {
            "t.wf".to_string()
        })
        .findings
        .errors;
        let text = errors.iter().map(|e| e.to_string()).collect::<String>();
        assert!(text.contains(said), "`{source}` must be refused: {text}");
    }
    // The well-formed ones pass, and a string is not silently a URL.
    let ok = page(
        "state site: Url = \"https://example.com\"\n    state who: Email = \"ada@example.com\"\n    Text(\"{site} {who}\")",
    );
    assert!(
        webfluent::sema::types::check(&parse_source(&ok, "t.wf").unwrap(), &|_| "t.wf".into())
            .findings
            .errors
            .is_empty()
    );
}

#[test]
fn now_is_the_moment_the_page_is_at() {
    let js = spa_generated(&page(
        r#"derived today = now.date()
    Text("{today}")"#,
    ));
    assert!(js.contains("WF.dateOf(WF.now())"), "{js}");
}

// ─── The network, said in the language ───────────────────

/// A page with a service in front of it.
fn served(body: &str) -> String {
    format!(
        "type User {{ id: String, name: String }}\n\
         api Backend(base: \"/api/v1\") {{\n\
         \x20   headers {{ Authorization: \"Bearer t\" }}\n\
         \x20   timeout: 10.seconds\n\
         \x20   get users(page: Number = 1) -> [User]\n\
         \x20   get user(id: String) at \"users/:id\" -> User\n\
         \x20   post createUser(body: Map) -> User\n\
         }}\n{}",
        page(body)
    )
}

fn errors_of(src: &str) -> String {
    webfluent::sema::types::check(&parse_source(src, "t.wf").unwrap(), &|_| "t.wf".to_string())
        .findings
        .errors
        .iter()
        .map(|e| e.to_string())
        .collect()
}

#[test]
fn a_service_is_one_object_of_endpoints() {
    let js = spa_generated(&served("Text(\"x\")"));
    assert!(js.contains("WF.api({"), "{js}");
    assert!(js.contains("base: \"/api/v1\""));
    assert!(js.contains("timeout: 10000"), "a duration is milliseconds");
    // A header is read at the moment of the request, not at the start.
    assert!(js.contains("\"Authorization\": () =>"));
    assert!(js.contains("users: { method: \"GET\", path: \"users\" }"));
    assert!(js.contains("user: { method: \"GET\", path: \"users/:id\" }"));
}

#[test]
fn an_endpoint_is_checked_like_anything_else() {
    let text = errors_of(&served(
        "resource a = Backend.users(page: \"one\")\n    Text(\"x\")",
    ));
    assert!(
        text.contains("[T01]") && text.contains("`Number` is wanted"),
        "{text}"
    );

    let text = errors_of(&served("resource a = Backend.missing()\n    Text(\"x\")"));
    assert!(
        text.contains("[T06]") && text.contains("no endpoint `missing`"),
        "{text}"
    );
    assert!(
        text.contains("`users`"),
        "the hint lists what it has: {text}"
    );

    let text = errors_of(&served(
        "resource a = Backend.users(pge: 1)\n    Text(\"x\")",
    ));
    assert!(
        text.contains("[T10]") && text.contains("no parameter `pge`"),
        "{text}"
    );
}

#[test]
fn a_resource_over_an_endpoint_is_typed_by_what_it_returns() {
    let text = errors_of(&served(
        "resource rows = Backend.users()\n    match rows { ready(list) { for u in list by u.id { Text(u.nam) } } else { Spinner } }",
    ));
    assert!(text.contains("`User` has no field `nam`"), "{text}");
}

#[test]
fn what_a_request_can_fail_with_is_a_closed_set() {
    let ok = served(
        "resource rows = Backend.users()\n    match rows { error(e) { match e { .offline { Text(\"off\") } .status { Text(\"s\") } else { Text(e.message) } } } else { Spinner } }",
    );
    assert!(errors_of(&ok).is_empty(), "{}", errors_of(&ok));

    let bad = served(
        "resource rows = Backend.users()\n    match rows { error(e) { match e { .teapot { Text(\"?\") } else { Text(\"x\") } } } else { Spinner } }",
    );
    let text = errors_of(&bad);
    assert!(text.contains("cannot fail with `.teapot`"), "{text}");

    let bad = served(
        "resource rows = Backend.users()\n    match rows { error(e) { Text(\"{e.reason}\") } else { Spinner } }",
    );
    assert!(
        errors_of(&bad).contains("no field `reason`"),
        "{}",
        errors_of(&bad)
    );
}

#[test]
fn what_the_call_site_asks_for_reaches_the_engine() {
    let js = spa_generated(&served(
        "state n: Number = 1\n    resource rows = Backend.users(page: n, cache: .swr(60.seconds), on: .focus, paginate: .page)\n    Text(\"{rows.hasMore}\")",
    ));
    assert!(js.contains("cache: { kind: \"swr\", ttl: 60000 }"), "{js}");
    assert!(js.contains("on: { focus: true }"), "{js}");
    // `loadMore()` moves the state the page number came from.
    assert!(
        js.contains("paginate: { by: \"page\", set: (v) => _n.set(v) }"),
        "{js}"
    );
}

#[test]
fn a_connection_is_opened_where_it_is_read_and_matched_by_its_state() {
    let js = spa_generated(&page(
        "socket chat = ws(\"wss://x/chat\", heartbeat: 20.seconds) { on message(m) { log(m) } }\n\
         \x20   stream ticks = sse(\"/events\", events: [\"price\"])\n\
         \x20   channel cart = broadcast(\"cart\")\n\
         \x20   match chat { connecting { Spinner } open { Text(\"live\") } closed(c) { Text(\"{c.code}\") } else { Text(\"?\") } }",
    ));
    assert!(
        js.contains("WF.ws(\"wss://x/chat\", { heartbeat: 20000"),
        "{js}"
    );
    assert!(
        js.contains("onMessage: (m) =>"),
        "what arrives is handled: {js}"
    );
    assert!(
        js.contains("WF.sse(\"/events\", { events: [\"price\"] })"),
        "{js}"
    );
    assert!(js.contains("WF.broadcast(\"cart\""), "{js}");
    // A connection is matched by the state it is in.
    assert!(
        js.contains(".state()") && js.contains("connecting:"),
        "{js}"
    );
}

#[test]
fn an_action_that_shows_before_the_server_agrees_takes_it_back() {
    let js = spa_generated(&page(
        "state rows: [Map] = []\n\
         \x20   action rename(name: String) {\n\
         \x20       optimistic(rows, items => items)\n\
         \x20       await fetch(\"/api/x\")\n\
         \x20   }\n\
         \x20   Button(\"Go\") { on click { rename(\"x\") } }",
    ));
    assert!(js.contains("WF.optimistic("), "{js}");
    assert!(
        js.contains("WF.attempt(async () =>"),
        "the action rolls itself back: {js}"
    );
}

#[test]
fn the_network_is_a_value_and_a_beacon_outlives_the_page() {
    let js = spa_generated(&page(
        "Button(\"Go\") { on click { beacon(\"/a\", { e: 1 }) } }\n\
         \x20   if !network.online { Alert(\"offline\").warning }",
    ));
    assert!(js.contains("WF.beacon("), "{js}");
    assert!(js.contains("WF.network()"), "{js}");

    // And the checker knows its shape.
    let text = errors_of(&page("Text(\"{network.onlien}\")"));
    assert!(text.contains("no field `onlien`"), "{text}");
}

#[test]
fn a_service_read_from_a_specification_is_checked_like_one_written_by_hand() {
    let spec = r##"{
        "openapi": "3.0.0",
        "components": { "schemas": { "User": { "type": "object", "required": ["id"],
            "properties": { "id": { "type": "string" }, "name": { "type": "string" } } } } },
        "paths": { "/users/{id}": { "get": { "operationId": "getUser",
            "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }],
            "responses": { "200": { "content": { "application/json":
                { "schema": { "$ref": "#/components/schemas/User" } } } } } } } }
    }"##;
    let src = format!(
        "api Backend from \"api.json\" (base: \"/api\")\n{}",
        page(
            "resource one = Backend.getUser(id: \"1\")\n    match one { ready(u) { Text(u.name ?? \"?\") } else { Spinner } }"
        )
    );
    let mut program = parse_source(&src, "t.wf").unwrap();
    webfluent::openapi::expand(&mut program, &|_| Some(spec.to_string())).unwrap();

    // The endpoints and the schemas both arrived.
    let api = program.declarations.iter().find_map(|d| match d {
        webfluent::parser::Declaration::Api(a) => Some(a),
        _ => None,
    });
    let api = api.expect("the service");
    assert_eq!(api.endpoints.len(), 1);
    assert_eq!(api.endpoints[0].method, "GET");
    assert_eq!(
        api.endpoints[0].path, "users/:id",
        "a spec path becomes the language's"
    );
    assert!(
        program.declarations.iter().any(|d| matches!(
            d, webfluent::parser::Declaration::Type(t) if t.name == "User"
        )),
        "each named schema is a type the program can name"
    );
    let errors = webfluent::sema::types::check(&program, &|_| "t.wf".to_string())
        .findings
        .errors;
    assert!(errors.is_empty(), "{errors:?}");
}

// ─── Validation the compiler knows about ─────────────────

#[test]
fn a_validate_block_is_the_rules_registered_against_the_state_it_guards() {
    let js = spa_generated(&page(
        "state email = \"\"\n\
         \x20   state password = \"\"\n\
         \x20   validate email { required  email }\n\
         \x20   validate password { minLength(8) \"Use at least 8\"  matches(email) }\n\
         \x20   Form(bind: form) {\n\
         \x20       on submit { log(\"go\") }\n\
         \x20       Input(bind: email, label: \"Email\").email\n\
         \x20       Input(bind: password, label: \"Password\").password\n\
         \x20   }",
    ));
    assert!(js.contains("WF.validate(() => _email()"), "{js}");
    assert!(js.contains("{ name: \"required\" }"));
    assert!(js.contains("message: () => \"Use at least 8\""));
    // A rule that names another value follows it.
    assert!(js.contains("args: [() => _email()]"), "{js}");
    // The control shows what the rules say, without being told to.
    assert!(js.contains("error: () => _email_check.shown()"), "{js}");
    // And marks itself seen when the reader leaves it.
    assert!(js.contains("\"blur\""), "{js}");
    // A submit asks the rules before the author's handler runs.
    assert!(js.contains("if (!form.__submitting()) return;"), "{js}");
}

#[test]
fn a_refined_type_validates_itself() {
    let js = spa_generated(&page(
        "state nights: Number(1..=30) = 2\n\
         \x20   validate nights { required }\n\
         \x20   Form(bind: form) { Input(bind: nights, label: \"Nights\").number }",
    ));
    // The range the type declares is a rule, before one is written.
    assert!(js.contains("{ name: \"min\", args: [1] }"), "{js}");
    assert!(js.contains("{ name: \"max\", args: [30] }"), "{js}");
}

#[test]
fn a_rule_is_checked_against_what_it_guards() {
    let text = errors_of(&page(
        "state age: Number = 1\n    validate age { minLength(3) }\n    Text(\"x\")",
    ));
    assert!(
        text.contains("[T01]") && text.contains("`minLength` is a rule for String"),
        "{text}"
    );

    let text = errors_of(&page(
        "state a = \"\"\n    validate a { nonsense }\n    Text(\"x\")",
    ));
    assert!(text.contains("`nonsense` is not a rule"), "{text}");

    let text = errors_of(&page("validate missing { required }\n    Text(\"x\")"));
    assert!(
        text.contains("`missing` is not a name this page declares"),
        "{text}"
    );
}

#[test]
fn when_a_message_shows_is_the_forms_to_say() {
    let js = spa_generated(&page(
        "state a = \"\"\n\
         \x20   validate a { required }\n\
         \x20   Form(bind: form, show: .live) { Input(bind: a, label: \"A\") }",
    ));
    assert!(js.contains("WF.form({ show: \"live\" })"), "{js}");
}

#[test]
fn a_textarea_is_a_field_that_counts_what_is_left() {
    let html = raw_output(
        Backend::Ssg,
        &page(
            "Textarea(bind: note, label: \"Notes\", rows: 4, maxLength: 200)\n    state note = \"\"",
        ),
    );
    assert!(html.contains("<textarea"), "{html}");
    assert!(html.contains("rows=\"4\""), "{html}");
    assert!(
        html.contains("maxlength=\"200\""),
        "the paint shows the limit: {html}"
    );
}

// ─── Media the build processes ───────────────────────────

/// A tiny PNG of one flat colour, written here so the test needs no fixture.
fn one_colour_png(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    fn crc(data: &[u8]) -> u32 {
        let mut table = [0u32; 256];
        for (n, slot) in table.iter_mut().enumerate() {
            let mut c = n as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            *slot = c;
        }
        let mut c = 0xFFFF_FFFFu32;
        for b in data {
            c = table[((c ^ u32::from(*b)) & 0xFF) as usize] ^ (c >> 8);
        }
        c ^ 0xFFFF_FFFF
    }
    fn chunk(kind: &[u8], data: &[u8]) -> Vec<u8> {
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        let body: Vec<u8> = kind.iter().chain(data).copied().collect();
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc(&body).to_be_bytes());
        out
    }
    let mut row = vec![0u8];
    for _ in 0..width {
        row.extend_from_slice(&rgb);
    }
    let raw: Vec<u8> = row.repeat(height as usize);
    // Stored deflate blocks, which every decoder reads.
    let mut deflated = vec![0x78u8, 0x01];
    for (i, block) in raw.chunks(65_535).enumerate() {
        let last = u8::from((i + 1) * 65_535 >= raw.len());
        deflated.push(last);
        deflated.extend_from_slice(&(block.len() as u16).to_le_bytes());
        deflated.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        deflated.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    deflated.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut header = width.to_be_bytes().to_vec();
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(&chunk(b"IHDR", &header));
    png.extend_from_slice(&chunk(b"IDAT", &deflated));
    png.extend_from_slice(&chunk(b"IEND", b""));
    png
}

#[test]
fn an_image_is_read_at_build_time_and_written_at_every_width_a_page_asks_for() {
    let dir = std::env::temp_dir().join(format!("wf-media-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("public")).unwrap();
    std::fs::write(
        dir.join("public/hero.png"),
        one_colour_png(600, 400, [0x0F, 0x76, 0x6E]),
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "m", "build": { "ssg": true, "media": { "widths": [200, 400] } } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "image hero = \"hero.png\"\n\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(\"H\").h1\n    Image(hero, alt: \"The team\", sizes: \"100vw\", placeholder: .blur)\n    Text(\"{hero.width} by {hero.height}, mostly {hero.color}\")\n}\n",
    )
    .unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d", dir.to_str().unwrap()])
        .output()
        .expect("wf runs");
    let said =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("Build complete"), "{said}");

    // Every width smaller than the original, and the original itself — and
    // nothing larger, because up-scaling makes a bigger file of the same
    // picture.
    let written: Vec<String> = std::fs::read_dir(dir.join("build/img"))
        .unwrap()
        .filter_map(|e| Some(e.ok()?.file_name().to_string_lossy().to_string()))
        .collect();
    assert!(
        written.iter().any(|f| f.contains(".200.webp")),
        "{written:?}"
    );
    assert!(
        written.iter().any(|f| f.contains(".400.webp")),
        "{written:?}"
    );
    assert!(
        written.iter().any(|f| f.contains(".600.webp")),
        "the original's own width: {written:?}"
    );
    assert!(
        written.iter().any(|f| f.ends_with(".png")),
        "the file itself: {written:?}"
    );

    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(html.contains("<picture>"), "{html}");
    assert!(html.contains("type=\"image/webp\""), "{html}");
    assert!(html.contains("200w"), "the srcset lists the widths: {html}");
    // The box is the right shape before the image lands.
    assert!(
        html.contains("width=\"600\"") && html.contains("height=\"400\""),
        "{html}"
    );
    assert!(
        html.contains("data:image/webp;base64,"),
        "the placeholder is inlined: {html}"
    );
    // And the name is a value.
    assert!(html.contains("600 by 400, mostly #0F766E"), "{html}");

    // A second build does the work again from the cache, and says the same.
    let again = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d", dir.to_str().unwrap()])
        .output()
        .expect("wf runs");
    assert!(
        String::from_utf8_lossy(&again.stdout).contains("Build complete"),
        "a second build must not fail on what the first wrote"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_pdf_embeds_the_picture_rather_than_a_box_where_one_should_be() {
    let dir = std::env::temp_dir().join(format!("wf-media-pdf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("public")).unwrap();
    std::fs::write(
        dir.join("public/hero.png"),
        one_colour_png(64, 40, [0x0F, 0x76, 0x6E]),
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "m", "build": { "output_type": "pdf" } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "page R(path: \"/\", title: \"R\", description: \"D\") {\n    Document(page_size: \"A4\") {\n        Section {\n            Heading(\"R\").h1\n            Image(src: \"/hero.png\", alt: \"The team\", width: 200, height: 125)\n        }\n    }\n}\n",
    )
    .unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d", dir.to_str().unwrap()])
        .output()
        .expect("wf runs");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("Build complete"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let pdf = std::fs::read(dir.join("build/m.pdf")).unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Subtype /Image"),
        "the picture itself is in the file"
    );
    assert!(text.contains("/Im0 Do"), "and the page draws it");
    assert!(
        !text.contains("[Image]"),
        "the box that said there was one is gone"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── Responsive, without JavaScript ──────────────────────

#[test]
fn a_value_per_breakpoint_becomes_a_class_and_a_media_query() {
    let src = page(
        "Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg }) { Card { Text(\"a\") } }\n\
         \x20   Row(direction: { base: .column, md: .row }) { Text(\"b\") }",
    );
    let program = parse_source(&src, "t.wf").unwrap();
    let css = webfluent::codegen::scoped_css::scoped_rules(&program);
    // The base, and one query per step, at the theme's own widths.
    assert!(
        css.contains("grid-template-columns: repeat(1, 1fr)"),
        "{css}"
    );
    assert!(
        css.contains("@media (min-width: 768px)") && css.contains("repeat(2, 1fr)"),
        "{css}"
    );
    assert!(
        css.contains("@media (min-width: 1024px)") && css.contains("repeat(3, 1fr)"),
        "{css}"
    );
    assert!(
        css.contains("gap: var(--spacing-sm)") && css.contains("gap: var(--spacing-lg)"),
        "{css}"
    );
    assert!(
        css.contains("flex-direction: column") && css.contains("flex-direction: row"),
        "{css}"
    );
    // Nothing about it is JavaScript.
    let js = spa_generated(&src);
    assert!(!js.contains("matchMedia"), "{js}");
    assert!(js.contains("wf-r"), "the element carries the class: {js}");

    // And the static paint carries the same classes, so hydration finds
    // the page it expects.
    let html = raw_output(Backend::Ssg, &src);
    assert!(html.contains("wf-grid wf-r"), "{html}");
}

#[test]
fn a_theme_moves_a_breakpoint_and_everything_moves_with_it() {
    let src = format!(
        "theme Brand {{ screen-md: 900px }}\n{}",
        page(
            "Grid(columns: { base: 1, md: 2 }) { Card { Text(\"a\") } }\n    Text(\"t\") { style { color: red\n        @md { color: blue } } }"
        )
    );
    let program = parse_source(&src, "t.wf").unwrap();
    let css = webfluent::codegen::scoped_css::scoped_rules(&program);
    assert!(css.contains("@media (min-width: 900px)"), "{css}");
    assert!(
        !css.contains("@media (min-width: 768px)"),
        "the baseline is gone: {css}"
    );
    // `@md { }` is the same token, written as sugar.
    assert!(css.contains("color: blue"), "{css}");
}

#[test]
fn a_layout_stays_as_it_is_written_unless_it_asks_to_reflow() {
    // Nothing in the engine's sheet reflows a layout that did not ask.
    let sheet = webfluent::themes::component_css();
    for rule in [
        ".wf-row { flex-direction: column; }",
        ".wf-grid { grid-template-columns: 1fr !important; }",
    ] {
        assert!(
            !sheet.contains(rule),
            "the engine still reflows every layout: {rule}"
        );
    }
    assert!(
        sheet.contains(".wf-row.wf-stacks"),
        "and `.stacks` is how one asks"
    );

    let js = spa_generated(&page("Row { Text(\"a\") }\n    Row.stacks { Text(\"b\") }"));
    assert!(js.contains("\"wf-row\""), "{js}");
    assert!(js.contains("\"wf-row wf-stacks\""), "{js}");
}

#[test]
fn a_responsive_value_is_checked_at_every_breakpoint() {
    let text = errors_of(&page(
        "Grid(columns: { base: 1, md: \"two\" }) { Card { Text(\"a\") } }",
    ));
    assert!(text.contains("[T01]") && text.contains("at `md`"), "{text}");
    assert!(text.contains("`Number` is wanted"), "{text}");
}

// ─── Motion with a real timing model ─────────────────────

#[test]
fn a_sequence_writes_the_clock_onto_the_elements_of_its_steps() {
    let js = spa_generated(&page(
        "sequence {\n\
         \x20   step { Text(\"one\").fadeIn }\n\
         \x20   step(after: \"120ms\") { Text(\"two\").slideUp }\n\
         \x20   step(after: \"0.12s\") { Text(\"three\").slideUp  Text(\"four\").fadeIn }\n\
         }",
    ));
    // It is sugar and nothing else: three steps become four ordinary
    // elements, each with the delay the sequence's clock had reached.
    assert!(js.contains("\"data-wf-delay\": \"120ms\""), "{js}");
    assert_eq!(
        js.matches("\"data-wf-delay\": \"240ms\"").count(),
        2,
        "both elements of one step start together: {js}"
    );
    assert!(!js.contains("sequence"), "nothing of it survives: {js}");
}

#[test]
fn a_sequences_delay_reaches_the_static_paint_as_a_rule() {
    // A marker only JavaScript reads would let the whole sequence play at
    // once on the first paint, which is the one thing a sequence is for.
    let html = raw_output(
        Backend::Ssg,
        &page(
            "sequence { step { Text(\"a\").fadeIn } step(after: \"120ms\") { Text(\"b\").fadeIn } }",
        ),
    );
    assert!(html.contains("wf-animate-fadeIn"), "{html}");
    assert!(
        html.contains("wf-animate-fadeIn wf-s"),
        "the delayed one carries the class its rule lives under: {html}"
    );
}

#[test]
fn a_step_says_what_it_takes_when_it_is_written_wrongly() {
    let bad = parse_source(&page("sequence { Text(\"a\") }"), "t.wf").unwrap_err();
    assert!(format!("{bad}").contains("A sequence holds steps"), "{bad}");
    let bad = parse_source(
        &page("sequence { step(after: 120) { Text(\"a\") } }"),
        "t.wf",
    )
    .unwrap_err();
    assert!(format!("{bad}").contains("length of time"), "{bad}");
    let bad = parse_source(
        &page("sequence { step(after: \"soon\") { Text(\"a\") } }"),
        "t.wf",
    )
    .unwrap_err();
    assert!(format!("{bad}").contains("not a length of time"), "{bad}");
}

#[test]
fn an_element_that_waits_to_be_scrolled_to_does_not_carry_its_animation_as_a_class() {
    let js = spa_generated(&page(
        "Section(animate: .fadeIn, on: .enterView, easing: .spring) { Text(\"x\") }",
    ));
    assert!(
        js.contains("WF.onEnterView(") && js.contains("\"fadeIn\""),
        "{js}"
    );
    assert!(
        js.contains("var(--ease-spring)"),
        "a named easing is the design's token: {js}"
    );
    assert!(
        !js.contains("wf-animate-fadeIn"),
        "the class would play it at the first paint, which is the one moment it must not: {js}"
    );
    // Without `on:`, it is a class and needs no JavaScript at all.
    let js = spa_generated(&page("Section(animate: .fadeIn) { Text(\"x\") }"));
    assert!(
        js.contains("wf-animate-fadeIn") && !js.contains("WF.onEnterView"),
        "{js}"
    );
}

#[test]
fn a_shared_name_is_carried_across_a_route_change() {
    let js = spa_generated(&page(
        "state id = \"3\"\n    Image(\"/a.png\", alt: \"A\", shared: \"cover-{id}\")",
    ));
    assert!(js.contains("WF.shared("), "{js}");
    assert!(js.contains("cover-"), "{js}");
    assert!(
        !js.contains("shared:\""),
        "it is a handle on the element, not an attribute: {js}"
    );
}

#[test]
fn a_number_counts_to_its_new_value_and_keeps_its_formatting() {
    let js = spa_generated(&page(
        "state revenue = 1200\n    Text(format(revenue, .currency), count: \"600ms\")",
    ));
    assert!(
        js.contains("WF.counted(") && js.contains("\"600ms\""),
        "{js}"
    );
    assert!(
        js.contains("(n) => WF.format(n, \"currency\")"),
        "the count is over the number; the formatting is applied to every step of it: {js}"
    );
    // A plain number counts too, with no formatter.
    let js = spa_generated(&page("state n = 1\n    Text(n, count: \"400ms\")"));
    assert!(js.contains("WF.counted(") && js.contains(", null)"), "{js}");
}

#[test]
fn a_box_opens_to_the_height_of_what_is_inside_it() {
    let js = spa_generated(&page(
        "state open = false\n    show open { Stack(animate: .expand) { Text(\"x\") } }",
    ));
    assert!(js.contains("WF.show("), "{js}");
    assert!(
        js.contains("enter: \"expand\""),
        "the branch plays it, since the box is the thing that opens: {js}"
    );
}

#[test]
fn the_motion_config_sets_what_an_animation_does_by_default() {
    use std::collections::HashMap;
    let motion = webfluent::config::MotionConfig {
        duration: Some("180ms".to_string()),
        easing: Some("$ease-standard".to_string()),
    };
    let mut tokens: HashMap<String, String> = webfluent::themes::tokens::default_tokens();
    webfluent::themes::apply_motion(&mut tokens, &motion);
    assert_eq!(tokens.get("animation-duration-normal").unwrap(), "180ms");
    assert_eq!(
        tokens.get("animation-easing-default").unwrap(),
        "cubic-bezier(0.2, 0, 0, 1)",
        "a `$token` is resolved, as it is in a style value"
    );
}

// ─── Stores: lifetime, location, and a way to see them ───

/// A program with two stores, the second read by the first.
fn stores(extra: &str) -> String {
    format!(
        "store Totals {{\n\
         \x20   derived doubled = Cart.count * 2\n\
         }}\n\
         store Cart{extra} {{\n\
         \x20   state count = 21\n\
         \x20   action add() {{ count = count + 1 }}\n\
         }}\n{}",
        page(
            "use Totals\n    use Cart\n    Text(\"{Totals.doubled}\")\n    Button(\"+\") { on click { Cart.add() } }"
        )
    )
}

#[test]
fn a_store_is_a_thunk_so_the_order_they_are_declared_in_cannot_matter() {
    let js = spa_generated(&stores(""));
    // The whole definition is behind an arrow: nothing in it runs until
    // something reads the store, so `Totals` reading `Cart` — declared
    // below it — is no longer a read of `undefined`.
    assert!(js.contains("WF.store(\"Totals\", () => ({"), "{js}");
    assert!(js.contains("WF.store(\"Cart\", () => ({"), "{js}");
    assert!(js.contains("{ scope: \"app\" }"), "{js}");
}

#[test]
fn a_store_says_how_long_what_it_holds_lives() {
    let js = spa_generated(&stores("(scope: .route)"));
    assert!(js.contains("{ scope: \"route\" }"), "{js}");
    let js = spa_generated(&stores("(scope: .session, eager: true)"));
    assert!(js.contains("{ scope: \"session\", eager: true }"), "{js}");
    // A scope that is not one of the three is an error where it is written.
    let bad = parse_source(&stores("(scope: .forever)"), "t.wf").unwrap_err();
    assert!(format!("{bad}").contains("no scope `.forever`"), "{bad}");
    let bad = parse_source(&stores("(lifetime: .app)"), "t.wf").unwrap_err();
    assert!(format!("{bad}").contains("takes no `lifetime:`"), "{bad}");
}

#[test]
fn what_is_kept_says_where_it_lives_and_how_an_older_value_is_read() {
    let js = spa_generated(&format!(
        "type Item {{ id: String, qty: Number }}\n\
         store Cart {{\n\
         \x20   persist items: [Item] = [] {{\n\
         \x20       in: .local\n\
         \x20       version: 2\n\
         \x20       sync: false\n\
         \x20       migrate 1 -> 2 {{ old.map(i => Item(id: i.id, qty: i.count)) }}\n\
         \x20   }}\n\
         \x20   derived count = items.length\n\
         }}\n{}",
        page("use Cart\n    Text(\"{Cart.count}\")")
    ));
    assert!(js.contains("in: \"local\""), "{js}");
    assert!(js.contains("version: 2"), "{js}");
    assert!(js.contains("sync: false"), "{js}");
    // `old` is the step's own parameter, not a name of the page's.
    assert!(
        js.contains("migrate: { 2: (old) => old.map("),
        "the value the last version wrote, brought forward: {js}"
    );
}

#[test]
fn a_page_keeps_a_value_under_the_same_policy_a_store_does() {
    let js = spa_generated(&page(
        "persist theme = \"light\" { in: .session  sync: false }\n    Text(theme)",
    ));
    assert!(
        js.contains("WF.persist(\"P.theme\", \"light\", { in: \"session\", sync: false })"),
        "{js}"
    );
}

#[test]
fn a_version_with_a_gap_in_its_chain_is_a_warning() {
    let src = page(
        "persist a = \"\" {\n\
         \x20       version: 3\n\
         \x20       migrate 2 -> 3 { old }\n\
         \x20   }\n    Text(a)",
    );
    let program = parse_source(&src, "t.wf").unwrap();
    let text: String = webfluent::linter::lint_accessibility_in(&program, &|_| "t.wf".to_string())
        .iter()
        .map(|w| w.to_string())
        .collect();
    assert!(text.contains("P01") && text.contains("version 1"), "{text}");

    // And a step past the version never runs.
    let src = page("persist a = \"\" { migrate 1 -> 2 { old } }\n    Text(a)");
    let program = parse_source(&src, "t.wf").unwrap();
    let text: String = webfluent::linter::lint_accessibility_in(&program, &|_| "t.wf".to_string())
        .iter()
        .map(|w| w.to_string())
        .collect();
    assert!(text.contains("P02"), "{text}");
}

#[test]
fn a_migration_moves_one_version_at_a_time() {
    let bad = parse_source(
        &page("persist a = \"\" { version: 3  migrate 1 -> 3 { old } }\n    Text(a)"),
        "t.wf",
    )
    .unwrap_err();
    assert!(format!("{bad}").contains("skips a version"), "{bad}");
}

#[test]
fn a_secret_is_never_written_down() {
    let text = errors_of(&page("persist token: Secret = \"\"\n    Text(\"x\")"));
    assert!(text.contains("T12") && text.contains("persist"), "{text}");
}

#[test]
fn a_test_reads_a_store_and_seeds_it_by_name() {
    use webfluent::codegen::static_eval::store_as_value;
    use webfluent::parser::ast::Declaration;
    let program = parse_source(
        "store Cart {\n    state items: [Map] = []\n    derived count = items.length\n}\n",
        "t.wf",
    )
    .unwrap();
    let Declaration::Store(cart) = &program.declarations[0] else {
        panic!("a store")
    };
    // A store's initial state and the values derived from it, which is what
    // a test renders against without being asked.
    let plain = store_as_value(cart, None);
    assert_eq!(plain["count"].as_f64(), Some(0.0));
    assert_eq!(plain["items"], serde_json::json!([]));
    // And `data` by the store's name stands in for what an action would
    // have put there, with the derived values following it.
    let seeded = store_as_value(
        cart,
        Some(&serde_json::json!({ "items": [{ "id": "a" }, { "id": "b" }] })),
    );
    assert_eq!(seeded["count"].as_f64(), Some(2.0));
}

// ─── Security, as a subject the language takes seriously ───

#[test]
fn an_on_attribute_is_script_in_an_attribute_and_is_refused() {
    for written in [
        "Button(\"x\", onclick: \"alert(1)\")",
        "Card(onmouseover: handler) { Text(\"x\") }",
        "Image(src: \"/a.png\", alt: \"A\", onerror: \"go()\")",
    ] {
        let text = errors_of(&page(written));
        let all = format!("{text}{}", sema_errors(&page(written)));
        assert!(
            all.contains("would put script in an attribute"),
            "{written}: {all}"
        );
    }
    // The handler it points at is fine.
    assert!(
        sema_errors(&page("Button(\"x\") { on click { log(1) } }")).is_empty(),
        "a handler is how it is written"
    );
}

#[test]
fn a_literal_url_a_browser_would_run_is_refused_where_it_is_written() {
    for written in [
        "Link(\"x\", to: \"javascript:alert(1)\")",
        "Image(src: \"javascript:alert(1)\", alt: \"A\")",
        "Link(\"x\", to: \"data:text/html,<script>alert(1)</script>\")",
        "Link(\"x\", to: \"JavaScript:alert(1)\")",
    ] {
        let found = sema_errors(&page(written));
        assert!(found.contains("which a browser runs"), "{written}: {found}");
    }
    // Everything a page actually links to still compiles.
    for fine in [
        "Link(\"x\", to: \"/about\")",
        "Link(\"x\", to: \"https://example.com\")",
        "Link(\"x\", to: \"mailto:ada@example.com\")",
        "Image(src: \"/a.png\", alt: \"A\")",
    ] {
        assert!(sema_errors(&page(fine)).is_empty(), "{fine}");
    }
}

#[test]
fn a_url_that_only_exists_at_run_time_is_checked_when_it_is_used() {
    let js = spa_generated(&page(
        "state u = \"/ok\"\n    Link(\"x\", to: u)\n    Image(src: u, alt: \"A\")\n    Button(\"go\") { on click { navigate(u) } }",
    ));
    assert!(
        js.matches("WF.safeUrl(").count() >= 4,
        "the link's href, its click and its active check, the image, the navigate: {js}"
    );
    // A literal was already checked, so it carries no guard.
    let js = spa_generated(&page("Link(\"x\", to: \"/about\")"));
    assert!(!js.contains("WF.safeUrl("), "{js}");
}

#[test]
fn a_link_that_opens_elsewhere_does_not_hand_it_this_page() {
    let js = spa_generated(&page(
        "Link(\"x\", to: \"https://example.com\", target: \"_blank\")",
    ));
    assert!(js.contains("rel: \"noopener noreferrer\""), "{js}");
    // A `rel` the author wrote is theirs.
    let js = spa_generated(&page(
        "Link(\"x\", to: \"https://example.com\", target: \"_blank\", rel: \"me\")",
    ));
    assert!(
        js.contains("rel: \"me\"") && !js.contains("noopener"),
        "{js}"
    );
}

#[test]
fn markup_goes_in_through_one_door_and_the_build_says_where() {
    let src = page(
        "state body = \"<p>hi</p><script>alert(1)</script>\"\n    Unsafe.Html(sanitize(body))",
    );
    let js = spa_generated(&src);
    assert!(js.contains("html: () => WF.sanitize("), "{js}");
    // And the static paint puts in the same thing, sanitised at build time.
    let html = raw_output(Backend::Ssg, &src);
    assert!(
        html.contains("<div class=\"wf-html\"><p>hi</p></div>"),
        "{html}"
    );
    assert!(!html.contains("alert(1)"), "{html}");
    // Every use is named, with whether it went through the sanitiser.
    let program = parse_source(&src, "t.wf").unwrap();
    let text: String = webfluent::linter::lint_accessibility_in(&program, &|_| "t.wf".to_string())
        .iter()
        .map(|w| w.to_string())
        .collect();
    assert!(text.contains("V03") && text.contains("sanitize"), "{text}");
    let bare = parse_source(&page("Unsafe.Html(\"<p>ours</p>\")"), "t.wf").unwrap();
    let text: String = webfluent::linter::lint_accessibility_in(&bare, &|_| "t.wf".to_string())
        .iter()
        .map(|w| w.to_string())
        .collect();
    assert!(
        text.contains("NOT") || text.contains("belongs in `sanitize"),
        "{text}"
    );
}

#[test]
fn the_policy_a_build_ships_is_one_its_pages_satisfy() {
    use webfluent::codegen::csp::check;
    let policy = "default-src 'self'; script-src 'self'; style-src 'self'";
    // What the engine writes.
    let clean = "<html><head><link rel=\"stylesheet\" href=\"/styles.css\">\
                 <script src=\"/app.js\" defer></script></head>\
                 <body><p class=\"wf-text\">x</p><div class=\"wf-hidden\">y</div></body></html>";
    assert!(check("index.html", clean, policy).is_empty());
    // And what it must never write.
    for (html, what) in [
        ("<script>alert(1)</script>", "an inline <script>"),
        ("<p style=\"color:red\">x</p>", "a style= attribute"),
        ("<a onclick=\"go()\">x</a>", "an `onclick` attribute"),
    ] {
        let found = check("index.html", html, policy);
        assert!(found.iter().any(|v| v.what == what), "{what}: {found:?}");
    }
    // `frame-ancestors` is only where a browser honours it.
    let config = webfluent::config::ProjectConfig::default_config("t");
    let meta = webfluent::config::project::csp_meta_policy(&config);
    let headers = webfluent::config::project::csp_policy(&config);
    assert!(!meta.contains("frame-ancestors"), "{meta}");
    assert!(headers.contains("frame-ancestors 'none'"), "{headers}");
    // A page that has to carry an inline style says so in its policy.
    let mut loose_config = webfluent::config::ProjectConfig::default_config("t");
    loose_config.build.inline_styles = true;
    let loose = webfluent::config::project::csp_meta_policy(&loose_config);
    assert!(
        loose.contains("style-src 'self' 'unsafe-inline'"),
        "{loose}"
    );
}

#[test]
fn a_style_value_that_reads_state_is_what_widens_the_policy() {
    use webfluent::codegen::csp::writes_inline_styles;
    // A literal compiles to a shared class and needs nothing.
    let plain = parse_source(&page("Card { style { padding: 2rem } }"), "t.wf").unwrap();
    assert!(!writes_inline_styles(&plain));
    // A value that reads state is the element's own.
    let live = parse_source(
        &page("state pct = 40\n    Card { style { width: {pct}% } }"),
        "t.wf",
    )
    .unwrap();
    assert!(writes_inline_styles(&live));
}

#[test]
fn an_env_name_that_has_not_said_it_is_public_is_not_in_the_bundle() {
    use webfluent::config::ProjectConfig;
    let mut config: ProjectConfig = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
    config.public_env = vec!["SITE_NAME".to_string()];
    assert!(config.env_is_public("PUBLIC_API"));
    assert!(config.env_is_public("SITE_NAME"));
    assert!(!config.env_is_public("STRIPE_SECRET"));
    assert!(!config.env_is_public("API_KEY"));
}

/// What `sema::check` says about a source, as one string.
fn sema_errors(src: &str) -> String {
    webfluent::sema::check(&parse_source(src, "t.wf").unwrap(), &|_| "t.wf".to_string())
        .errors
        .iter()
        .map(|e| e.to_string())
        .collect()
}

// ─── Interop, in both directions ─────────────────────────

/// A program that reaches a module, and one that reaches an element.
fn imports(body: &str) -> String {
    format!(
        "external Chart from \"https://cdn.example.com/chart.js\" {{\n\
         \x20   integrity: \"sha384-abc\"\n\
         \x20   fn Chart(canvas: Any, config: Map) -> ChartHandle\n\
         \x20   type ChartHandle {{\n\
         \x20       update(data: Map)\n\
         \x20       destroy()\n\
         \x20       width: Number\n\
         \x20   }}\n\
         }}\n{}",
        page(body)
    )
}

#[test]
fn an_import_is_a_module_of_its_own_so_the_page_chunks_still_see_the_bundle() {
    let src =
        imports("state d: Map = {}\n    Host(tag: \"canvas\", mount: (n) => Chart.Chart(n, d))");
    let program = parse_source(&src, "t.wf").unwrap();
    let module = webfluent::codegen::js::externals_module(&program).expect("a module");
    assert!(
        module.contains("import * as Chart from \"https://cdn.example.com/chart.js\""),
        "{module}"
    );
    assert!(
        module.contains("globalThis.Chart = Chart;"),
        "a module has its own scope, and the page chunks are classic scripts: {module}"
    );
    // The bundle itself stays a classic script — nothing of the import is
    // in it, so a split build's chunks read what they always did.
    let js = spa_generated(&src);
    assert!(!js.contains("import "), "{js}");
}

#[test]
fn a_page_says_where_it_imports_from_and_what_it_expects_to_get() {
    let src = imports("Text(\"x\")");
    let program = parse_source(&src, "t.wf").unwrap();
    let mut config = webfluent::config::ProjectConfig::default_config("t");
    config.build.script_origins = vec!["https://cdn.example.com".to_string()];
    let tags = webfluent::codegen::html::externals_tags(&config, &program, "");
    // The hash goes on a `modulepreload`, which is the one place the
    // platform lets subresource integrity reach a module.
    assert!(
        tags.contains("<link rel=\"modulepreload\" href=\"https://cdn.example.com/chart.js\" integrity=\"sha384-abc\" crossorigin=\"anonymous\">"),
        "{tags}"
    );
    assert!(
        tags.contains("<script type=\"module\" src=\"/externals.js\">"),
        "{tags}"
    );
    // And the policy names the origin, so a declared import is never
    // blocked by the policy shipped beside it.
    let policy = webfluent::config::project::csp_policy(&config);
    assert!(
        policy.contains("script-src 'self' https://cdn.example.com;"),
        "{policy}"
    );
}

#[test]
fn a_host_is_a_node_with_a_lifetime() {
    let js = spa_generated(&page(
        "state d: Map = {}\n    Host(tag: \"canvas\", mount: (n) => make(n), update: (h) => h.draw(d), cleanup: (h) => h.stop())",
    ));
    assert!(
        js.contains("WF.el(\"canvas\""),
        "the library asked for a canvas: {js}"
    );
    assert!(
        js.contains("WF.attach(") && js.contains("(h) => h.stop()"),
        "{js}"
    );
    // A `Host` that says nothing is a div, and the three are optional.
    let js = spa_generated(&page("Host(mount: (n) => make(n))"));
    assert!(js.contains("WF.el(\"div\""), "{js}");
    assert!(
        js.contains("WF.attach(_e0, ((n) => make(n)), null, null)"),
        "{js}"
    );
}

#[test]
fn somebody_elses_custom_element_is_placed_like_a_component() {
    let src = format!(
        "external element Stripe(\"stripe-pricing-table\") {{\n\
         \x20   prop publishableKey: String\n\
         \x20   event ready()\n\
         }}\n{}",
        page("state k = \"pk\"\n    Stripe(publishableKey: k) { on ready { log(1) } }")
    );
    let js = spa_generated(&src);
    assert!(js.contains("WF.el(\"stripe-pricing-table\""), "{js}");
    assert!(
        js.contains("\"publishable-key\": () => _k()"),
        "a prop is the attribute a framework would write, and follows state: {js}"
    );
    assert!(js.contains("WF.onRoot(_e0, \"ready\""), "{js}");
    // It is a known component, so nothing reports it as undeclared.
    let program = parse_source(&src, "t.wf").unwrap();
    let text: String = webfluent::linter::validate_semantics_in(&program, &|_| "t.wf".to_string())
        .iter()
        .map(|e| e.to_string())
        .collect();
    assert!(!text.contains("unknown component"), "{text}");
    // A tag without a hyphen is not a custom element's name.
    let bad = parse_source(
        &format!(
            "external element X(\"table\") {{ prop a: String }}\n{}",
            page("Text(\"x\")")
        ),
        "t.wf",
    )
    .unwrap_err();
    assert!(
        format!("{bad}").contains("not a custom element's name"),
        "{bad}"
    );
}

#[test]
fn a_published_component_is_a_tag_with_the_attributes_it_declared() {
    use webfluent::codegen::elements::{definitions, tag_name};
    let src = "component PriceTag(_ label: String, amount: Number, sale: Bool = false) {\n\
               \x20   event picked(label: String)\n\
               \x20   Row { Text(label) }\n\
               }\ncomponent Rating(_ score: Number) { Text(\"{score}\") }\n";
    let program = parse_source(src, "t.wf").unwrap();
    let mut config = webfluent::config::ProjectConfig::default_config("t");
    config.build.elements = vec!["PriceTag".to_string(), "Rating".to_string()];
    let js = definitions(&program, &config);
    assert_eq!(tag_name("PriceTag"), "price-tag");
    // A one-word component still needs a hyphen to be a custom element.
    assert_eq!(tag_name("Rating"), "wf-rating");
    assert!(
        js.contains("customElements.define(\"price-tag\", PriceTagElement)"),
        "{js}"
    );
    assert!(
        js.contains("observedAttributes() { return [\"label\", \"amount\", \"sale\"]; }"),
        "{js}"
    );
    assert!(
        js.contains("Number(this.getAttribute(\"amount\"))"),
        "a Number prop arrives as text: {js}"
    );
    assert!(
        js.contains("this.hasAttribute(\"sale\") ? this.getAttribute(\"sale\") !== \"false\""),
        "a bare attribute is true, as HTML reads one: {js}"
    );
    assert!(
        js.contains("dispatchEvent(new CustomEvent(\"picked\""),
        "an event it declares is a DOM event the host page hears: {js}"
    );
    assert!(
        js.contains("WF.scoped(") && js.contains("this._dispose"),
        "it cleans up after itself: {js}"
    );

    // A name the project does not declare is the one thing this cannot
    // paper over.
    config.build.elements = vec!["Nope".to_string()];
    let problems = webfluent::codegen::elements::check(&program, &config);
    assert!(
        problems.iter().any(|p| p.contains("`Nope`")),
        "{problems:?}"
    );
    config.build.elements = Vec::new();
    let problems = webfluent::codegen::elements::check(&program, &config);
    assert!(
        problems.iter().any(|p| p.contains("publishes nothing")),
        "{problems:?}"
    );
}

#[test]
fn a_state_named_like_a_generated_binding_is_still_the_state() {
    // `params`, `value`, `key`, `event` and `e` are the names the
    // generated code uses for a route's parameters, a handler's event and
    // a keyed loop's bindings. A page that declares one of them used to
    // get a bare global: it rendered as nothing and never updated.
    let js = spa_generated(&page(
        "state key = \"pk\"\n    state value = 3\n    state event = \"load\"\n    Text(\"{key} {value} {event}\")",
    ));
    assert!(
        js.contains("_key()") && js.contains("_value()") && js.contains("_event()"),
        "{js}"
    );
    assert!(!js.contains("${key}"), "{js}");
    // And the bindings the generator makes are untouched.
    let js = spa_generated(&page(
        "state n = 0\n    for item, i in [1, 2] by item { Text(\"{i}\") }\n    Button(\"go\") { on click(e) { n = e.x } }",
    ));
    assert!(js.contains("key: (item) => item"), "{js}");
    assert!(js.contains("(e) => { _n.set(e.x); }"), "{js}");
}

#[test]
fn a_store_member_binds_like_a_state() {
    // A store's member is a property with a getter and a setter. It used
    // to compile to a control with no binding at all — a silent no-op,
    // which is the one thing this language does not do.
    let js = spa_generated(&format!(
        "store Filters {{ state q = \"\" }}\n{}",
        page("use Filters\n    Input(bind: Filters.q).text")
    ));
    assert!(js.contains("value: () => Filters.q"), "{js}");
    assert!(
        js.contains("\"on:input\": (e) => { Filters.q = e.target.value; }"),
        "{js}"
    );
}

#[test]
fn a_route_store_that_persists_says_so() {
    let src = "store Filters(scope: .route) { persist q = \"\" }\n".to_string()
        + &page("use Filters\n    Text(Filters.q)");
    let program = parse_source(&src, "t.wf").unwrap();
    let text: String = webfluent::linter::lint_accessibility_in(&program, &|_| "t.wf".to_string())
        .iter()
        .map(|w| w.to_string())
        .collect();
    assert!(text.contains("P03"), "{text}");
    // A `state` in the same store says nothing: that is what the scope is for.
    let src = "store Filters(scope: .route) { state q = \"\" }\n".to_string()
        + &page("use Filters\n    Text(Filters.q)");
    let program = parse_source(&src, "t.wf").unwrap();
    let text: String = webfluent::linter::lint_accessibility_in(&program, &|_| "t.wf".to_string())
        .iter()
        .map(|w| w.to_string())
        .collect();
    assert!(!text.contains("P03"), "{text}");
}

// ─── Tests that act ──────────────────────────────────────

/// The steps a test declares, as words, in the order written.
fn steps_of(src: &str) -> Vec<String> {
    use webfluent::parser::ast::{Declaration, Step};
    let program = parse_source(src, "t.wf").unwrap();
    let Some(Declaration::Test(test)) = program
        .declarations
        .iter()
        .find(|d| matches!(d, Declaration::Test(_)))
        .cloned()
    else {
        panic!("a test");
    };
    let text = |e: &webfluent::parser::ast::Expr| match e {
        webfluent::parser::ast::Expr::StringLiteral(s) => s.clone(),
        other => format!("{other:?}"),
    };
    test.steps
        .iter()
        .map(|step| match step {
            Step::Expect {
                text: t, negated, ..
            } => {
                format!("expect{} {}", if *negated { " not" } else { "" }, text(t))
            }
            Step::Click { target, .. } => format!("click {}", text(target)),
            Step::Type { text: t, into, .. } => format!("type {} into {}", text(t), text(into)),
            Step::Press { key, target, .. } => match target {
                Some(t) => format!("press {} in {}", text(key), text(t)),
                None => format!("press {}", text(key)),
            },
        })
        .collect()
}

#[test]
fn a_test_says_what_it_does_and_what_it_then_expects_in_order() {
    let steps = steps_of(
        "test \"a cart counts\" {\n\
         \x20   Text(\"x\")\n\
         \x20   expect \"0 items\"\n\
         \x20   type \"Milk\" into \"Item\"\n\
         \x20   click \"Add\"\n\
         \x20   expect \"1 items\"\n\
         \x20   press \"Escape\" in \"Search\"\n\
         \x20   press \"Enter\"\n\
         \x20   expect not \"empty\"\n\
         }",
    );
    // The order is the whole meaning: what a click did is only visible in
    // the expect that follows it.
    assert_eq!(
        steps,
        [
            "expect 0 items",
            "type Milk into Item",
            "click Add",
            "expect 1 items",
            "press Escape in Search",
            "press Enter",
            "expect not empty",
        ]
    );
}

#[test]
fn a_test_that_acts_is_told_apart_from_one_that_looks() {
    use webfluent::parser::ast::Declaration;
    let test = |src: &str| {
        let program = parse_source(src, "t.wf").unwrap();
        match program.declarations.into_iter().next() {
            Some(Declaration::Test(t)) => t,
            _ => panic!("a test"),
        }
    };
    assert!(!test("test \"looks\" { Text(\"x\") expect \"x\" }").acts());
    assert!(test("test \"acts\" { Button(\"Go\") click \"Go\" expect \"x\" }").acts());
    assert!(
        test("test \"types\" { Input(label: \"N\") type \"a\" into \"N\" expect \"x\" }").acts()
    );
    assert!(test("test \"presses\" { Input(label: \"N\") press \"Enter\" expect \"x\" }").acts());
}

#[test]
fn a_test_that_expects_nothing_is_not_a_test() {
    let bad = parse_source(
        "test \"does things\" { Button(\"Go\")  click \"Go\" }",
        "t.wf",
    )
    .unwrap_err();
    assert!(format!("{bad}").contains("expects nothing"), "{bad}");
}

#[test]
fn the_step_that_types_is_told_from_the_declaration_that_names_a_type() {
    // `type "Ada" into "Name"` is a step; `type Todo { … }` is a
    // declaration, and the difference is what follows the word.
    let program = parse_source(
        "type Todo { id: String }\n\
         test \"t\" { Input(label: \"N\")  type \"Ada\" into \"N\"  expect \"x\" }",
        "t.wf",
    )
    .unwrap();
    assert!(
        program
            .declarations
            .iter()
            .any(|d| matches!(d, webfluent::parser::ast::Declaration::Type(_))),
        "the record is still a record"
    );
    assert_eq!(
        steps_of("test \"t\" { Input(label: \"N\")  type \"Ada\" into \"N\"  expect \"x\" }"),
        ["type Ada into N", "expect x"]
    );
}
