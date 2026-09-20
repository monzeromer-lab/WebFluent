//! The language additions of 3.1, held to every backend: what the static
//! paint (SSG) and the template engine evaluate must agree, and the SPA
//! bundle must carry the JavaScript that means the same thing.

mod common;

use common::{Backend, raw_output};
use serde_json::json;
use webfluent::Template;

fn page(body: &str) -> String {
    format!("page P(path: \"/\", title: \"T\") {{\n{body}\n}}\n")
}

/// The static paint of the page: state and derived values are seeded.
fn painted(src: &str) -> String {
    raw_output(Backend::Ssg, src)
}

/// The template engine's fragment for `body`, over `data`: a template
/// has no state, its names come from the data.
fn templated(body: &str, data: serde_json::Value) -> String {
    Template::from_str(&page(body))
        .expect("template parses")
        .render_html_fragment(&data)
        .expect("renders")
}

#[test]
fn if_let_as_a_value_binds_the_non_null_value() {
    let src = page(
        "state user = { name: \"Sam\" }\n state none = null\n derived a = if let u = user { u.name } else { \"anon\" }\n derived b = if let u = none { u.name } else { \"anon\" }\n Text(a)\n Text(b)",
    );
    let html = painted(&src);
    assert!(html.contains(">Sam<"), "the bound branch: {html}");
    assert!(html.contains(">anon<"), "the else branch: {html}");
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("((u) => u != null ? u.name : \"anon\")(_user())"),
        "{js}"
    );
    let html = templated(
        "Text(if let u = user { u.name } else { \"anon\" })\n Text(if let u = none { u.name } else { \"anon\" })",
        json!({ "user": { "name": "Sam" }, "none": null }),
    );
    assert!(
        html.contains(">Sam<") && html.contains(">anon<"),
        "template: {html}"
    );
}

#[test]
fn optional_chaining_reads_through_null_in_the_static_paint() {
    let body = "Text(user?.profile?.name ?? \"?\")\n Text(none?.profile?.name ?? \"nobody\")\n Text(none?.profile.name ?? \"still nobody\")";
    let src = page(&format!(
        "state user = {{ profile: {{ name: \"Sam\" }} }}\n state none = null\n {body}"
    ));
    let html = painted(&src);
    assert!(html.contains(">Sam<"), "{html}");
    assert!(html.contains(">nobody<"), "{html}");
    assert!(
        html.contains(">still nobody<"),
        "the chain short-circuits: {html}"
    );
    let html = templated(
        body,
        json!({ "user": { "profile": { "name": "Sam" } }, "none": null }),
    );
    assert!(
        html.contains(">Sam<") && html.contains(">nobody<") && html.contains(">still nobody<"),
        "template: {html}"
    );
}

#[test]
fn regex_literals_test_match_replace_and_split_at_build_time() {
    let body = "Text(if /^[a-z]+@[a-z]+\\.[a-z]+$/i.test(email) { \"valid\" } else { \"invalid\" })\n Text(email.replace(/@.*/, \"@…\"))\n Text(\"a1b2\".replace(/\\d/g, \"#\"))\n Text(\"x-y-z\".split(/-/).join(\"+\"))\n Text(\"Price 42\".match(/\\d+/)?.[0] ?? \"none\")\n Text(\"{ratio / 2}\")";
    let src = page(&format!(
        "state email = \"Sam@example.org\"\n state ratio = 8\n {body}"
    ));
    let html = painted(&src);
    for expect in [">valid<", ">Sam@…<", ">a#b#<", ">x+y+z<", ">42<", ">4<"] {
        assert!(html.contains(expect), "ssg lacks {expect}: {html}");
    }
    let html = templated(body, json!({ "email": "Sam@example.org", "ratio": 8 }));
    for expect in [">valid<", ">Sam@…<", ">a#b#<", ">x+y+z<", ">42<", ">4<"] {
        assert!(html.contains(expect), "template lacks {expect}: {html}");
    }
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("/^[a-z]+@[a-z]+\\.[a-z]+$/i.test(_email())"),
        "{js}"
    );
    assert!(
        js.contains("(_ratio() / 2)"),
        "a division stays a division: {js}"
    );
}

#[test]
fn list_and_string_helpers_run_at_build_time_and_in_the_browser() {
    let body = "Text(people.sortBy(p => p.age).map(p => p.name).join(\",\"))\n Text(people.unique().length)\n Text(people.take(1).map(p => p.name).join(\",\"))\n Text(people.first()?.name ?? \"-\")\n Text(people.last()?.name ?? \"-\")\n Text(\"hello world\".capitalize())\n Text(\"hello world\".truncate(5))\n Text(people.groupBy(p => p.team).a.length)\n Text(people.flatMap(p => [p.name, p.team]).join(\"\"))";
    let people = "[{ name: \"Bo\", age: 40, team: \"a\" }, { name: \"Al\", age: 30, team: \"a\" }, { name: \"Cy\", age: 35, team: \"b\" }]";
    let src = page(&format!("state people = {people}\n {body}"));
    let html = painted(&src);
    for expect in [
        ">Al,Cy,Bo<",
        ">3<",
        ">Bo<",
        ">Cy<",
        ">Hello world<",
        ">hello…<",
        ">2<",
        ">BoaAlaCyb<",
    ] {
        assert!(html.contains(expect), "ssg lacks {expect}: {html}");
    }
    let html = templated(
        body,
        json!({ "people": [{ "name": "Bo", "age": 40, "team": "a" }, { "name": "Al", "age": 30, "team": "a" }, { "name": "Cy", "age": 35, "team": "b" }] }),
    );
    for expect in [
        ">Al,Cy,Bo<",
        ">3<",
        ">Bo<",
        ">Cy<",
        ">Hello world<",
        ">hello…<",
        ">2<",
        ">BoaAlaCyb<",
    ] {
        assert!(html.contains(expect), "template lacks {expect}: {html}");
    }
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("WF.sortBy(_people(), ((p) => p.age))"), "{js}");
    assert!(js.contains("WF.truncate(\"hello world\", 5)"), "{js}");
}

#[test]
fn spread_and_ranges_build_lists_and_maps() {
    let body = "Text([...tags, \"c\"].join(\",\"))\n Text([0, ...tags].length)\n Text({ ...user, name: \"Bo\" }.name)\n Text({ ...user, name: \"Bo\" }.age)\n Text((1..4).join(\",\"))\n Text((1..=4).join(\",\"))\n Text((0..tags.length).length)";
    let src = page(&format!(
        "state tags = [\"a\", \"b\"]\n state user = {{ name: \"Al\", age: 3 }}\n {body}"
    ));
    let html = painted(&src);
    for expect in [">a,b,c<", ">3<", ">Bo<", ">1,2,3<", ">1,2,3,4<", ">2<"] {
        assert!(html.contains(expect), "ssg lacks {expect}: {html}");
    }
    let html = templated(
        body,
        json!({ "tags": ["a", "b"], "user": { "name": "Al", "age": 3 } }),
    );
    for expect in [">a,b,c<", ">3<", ">Bo<", ">1,2,3<", ">1,2,3,4<", ">2<"] {
        assert!(html.contains(expect), "template lacks {expect}: {html}");
    }
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("[..._tags(), \"c\"]"), "{js}");
    assert!(js.contains("({ ..._user(), name: \"Bo\" })"), "{js}");
    assert!(
        js.contains("WF.range(1, 4, false)") && js.contains("WF.range(1, 4, true)"),
        "{js}"
    );
}

#[test]
fn destructuring_lets_bind_several_names_at_once() {
    let src = page(
        "state user = { name: \"Al\", age: 3 }\n state pair = [1, 2]\n state out = \"\"\n action go() { let { name, age } = user\n let [x, y] = pair\n out = name + age + x + y }\n Button(\"go\") { on click { go() } }\n Text(out)",
    );
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("const _d__1 = WF.signal(_user());"), "{js}");
    assert!(
        js.contains("const _name = WF.signal(_d__1().name);"),
        "{js}"
    );
    assert!(js.contains("const _x = WF.signal(_d__2()[0]);"), "{js}");
}

#[test]
fn try_catch_guards_an_await_that_may_fail() {
    let src = page(
        "state error = \"\"\n action load() { try { let r = await fetch(\"/x\")\n log(r) } catch e { error = e.message } }\n Button(\"go\") { on click { load() } }\n Text(error)",
    );
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("async function load() {"), "{js}");
    assert!(js.contains("try {"), "{js}");
    assert!(js.contains("} catch (e) {"), "{js}");
    assert!(
        js.contains("_error.set(e.message);"),
        "the error is a plain name: {js}"
    );
}

#[test]
fn constants_are_read_everywhere_and_env_comes_from_the_config() {
    let src = "const API = \"/api\"\nconst LIMIT: Number = 3\nconst PATH = API + \"/todos\"\n"
        .to_string()
        + &page(
            "state n = LIMIT + 1\n Text(API)\n Text(PATH)\n Text(\"{LIMIT * 2}\")\n Text(\"{n}\")",
        );
    let html = painted(&src);
    for expect in [">/api<", ">/api/todos<", ">6<", ">4<"] {
        assert!(html.contains(expect), "ssg lacks {expect}: {html}");
    }
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("const API = \"/api\";"), "{js}");
    assert!(js.contains("const PATH = (API + \"/todos\");"), "{js}");
    assert!(js.contains("const env = {};"), "{js}");
    assert!(
        js.contains("WF.signal((LIMIT + 1))"),
        "a constant reads as written: {js}"
    );
    // A constant of the wrong type is a type error.
    let bad = "const LIMIT: Number = \"three\"\n".to_string() + &page("Text(\"x\")");
    let program = webfluent::parse_source(&bad, "t.wf").unwrap();
    let info = webfluent::sema::types::check(&program, &|_| "t.wf".to_string());
    assert!(
        info.findings.errors.iter().any(|e| e
            .message
            .contains("`const LIMIT` is `String`, but `Number` is wanted")),
        "{:?}",
        info.findings.errors
    );
}

#[test]
fn enum_cases_carry_payloads_that_match_binds() {
    let decl = "enum Status { idle, failed(reason: String), done(count: Number, label: String) }\n";
    let body = "match s { .idle { Text(\"idle\") } .failed(r) { Text(\"failed: {r}\") } .done(n, l) { Text(\"{n} {l}\") } }\n Text(match s { .failed(r) { r } .done { \"done\" } else { \"-\" } })";
    let src = format!(
        "{decl}{}",
        page(&format!("state s: Status = .failed(\"boom\")\n {body}"))
    );
    let html = painted(&src);
    assert!(html.contains(">failed: boom<"), "the case's arm: {html}");
    assert!(html.contains(">boom<"), "the match expression: {html}");
    let src2 = format!(
        "{decl}{}",
        page(&format!("state s: Status = .done(2, \"two\")\n {body}"))
    );
    let html = painted(&src2);
    assert!(html.contains(">2 two<"), "a two-part payload: {html}");
    assert!(html.contains(">done<"), "{html}");
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("[\"failed\", \"boom\"]"), "{js}");
    assert!(js.contains("() => WF.caseOf(_s())"), "{js}");
    assert!(js.contains("const r = _v[1];"), "{js}");
    assert!(js.contains("WF.payload(_s(), \"failed\")"), "{js}");
    assert!(js.contains("(WF.caseOf(_s()) === \"done\")"), "{js}");
    let html = templated(body, json!({ "s": ["failed", "nope"] }));
    assert!(
        html.contains(">failed: nope<") && html.contains(">nope<"),
        "template: {html}"
    );
    let html = templated(body, json!({ "s": "idle" }));
    assert!(
        html.contains(">idle<") && html.contains(">-<"),
        "template: {html}"
    );
}

#[test]
fn format_and_ago_speak_the_locale_at_build_time_and_in_the_browser() {
    let body = "Text(format(n, .currency))\n Text(format(n, .currency, \"EUR\"))\n Text(format(n, .compact))\n Text(format(share, .percent, 1))\n Text(format(d, \"yyyy-MM-dd\"))\n Text(format(d, .date, \"long\"))\n Text(ago(d, now))";
    let src = page(&format!(
        "state n = 1234.5\n state share = 0.256\n state d = \"2024-03-05T14:07:00Z\"\n state now = \"2024-03-07T14:07:00Z\"\n {body}"
    ));
    let html = painted(&src);
    for expected in [
        ">$1,234.50<",
        ">€1,234.50<",
        ">1.2K<",
        ">25.6%<",
        ">2024-03-05<",
        ">March 5, 2024<",
        ">2 days ago<",
    ] {
        assert!(html.contains(expected), "{expected} in {html}");
    }
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("WF.format(_n(), \"currency\")"), "{js}");
    assert!(
        js.contains("WF.format(_n(), \"currency\", \"EUR\")"),
        "{js}"
    );
    assert!(js.contains("WF.ago(_d(), _now())"), "{js}");
    let html = templated(
        body,
        json!({ "n": 1234.5, "share": 0.256, "d": "2024-03-05T14:07:00Z", "now": "2024-03-07T14:07:00Z", "locale": "de" }),
    );
    assert!(html.contains(">$1.234,50<"), "the data's locale: {html}");
    assert!(html.contains(">2 days ago<"), "{html}");
    // A page's own `format` action is its own.
    let src = page("state n = 1\n action format(x: Number) { return \"{x}!\" }\n Text(format(n))");
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("format(_n())") && !js.contains("WF.format"),
        "{js}"
    );
}
