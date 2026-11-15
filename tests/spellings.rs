//! The type spellings every later part of the language builds on:
//! dictionaries `[K: V]`, function types `(A) -> R` and `()`, aliases
//! `type X = T`, results `T ! E`, `T!`, `! E` and `!`, `Json` and its cases,
//! `-> T` on an action, typed lambda parameters, and `T.decode`.
//!
//! Each is read back as it was written — parsed, printed and parsed again
//! to the same text, formatted to a fixed point, and carried through the
//! indented layout and back — and checked: what it accepts, and what it
//! refuses with a code.

use webfluent::syntax::fmt::format_source;
use webfluent::syntax::layout::{to_braces, to_offside};
use webfluent::syntax::parse_source;
use webfluent::syntax::print::file_text;

/// One source holding every spelling, as the formatter writes it.
const EVERY_SPELLING: &str = r#"type Settings {
    theme: String
    columns: Number
}

type Scores = [String: Number]

type Check = (String, Number) -> Bool

type Outcome = Settings ! DecodeError

type Loose = Settings!

enum Region {
    north
    south
}

store Prefs {
    state scores: Scores = { ada: 3 }
    state prices: [Region: Number] = { north: 1, south: 2 }
    state raw: Json = Json.null
    state saved: Settings = Settings(theme: "light", columns: 2)

    action best() -> Number {
        return scores.values().sum()
    }

    action restore(text: String) -> Settings ! DecodeError {
        return try Settings.decode(JSON.parse(text))
    }

    action forget() -> ! DecodeError {
        return
    }

    action quiet() -> ! {
        return
    }

    action nothing() -> () {
        return
    }
}

component Picker(describe: (Region) -> String, done: () -> ()) {
    Text(describe(.north))
}

page Home(path: "/", title: "Home", description: "Every spelling.") {
    use Prefs
    derived byTheme = (a: Settings, b: Settings) => a.theme.localeCompare(b.theme)
    derived wide = [Prefs.saved].filter((s: Settings) => s.columns > 1)
    Heading("Home").h1
    Text("{Prefs.best()} {byTheme(Prefs.saved, Prefs.saved)} {wide.length}")
    Picker(describe: r => "{r}", done: () => Prefs.quiet())
    match Prefs.raw {
        .object(fields) { Text("{fields.keys().length}") }
        .array(items) { Text("{items.length}") }
        .string(s) { Text(s) }
        .number(n) { Text("{n}") }
        .bool(b) { Text("{b}") }
        .null { Text("nothing") }
    }
}
"#;

#[test]
fn every_spelling_prints_as_it_was_written() {
    let program = parse_source(EVERY_SPELLING, "spellings.wf").expect("parses");
    let printed = file_text(&program);
    let again = parse_source(&printed, "spellings.wf").expect("the printed text parses");
    assert_eq!(file_text(&again), printed, "printing is a fixed point");
    for spelling in [
        "type Scores = [String: Number]",
        "type Check = (String, Number) -> Bool",
        "type Outcome = Settings ! DecodeError",
        "type Loose = Settings!",
        "state prices: [Region: Number]",
        "action best() -> Number",
        "action restore(text: String) -> Settings ! DecodeError",
        "action forget() -> ! DecodeError",
        "action quiet() -> !",
        "action nothing() -> ()",
        "describe: (Region) -> String",
        "done: () -> ()",
        "(a: Settings, b: Settings) => a.theme",
        "(s: Settings) => s.columns",
        "Json.null",
        ".null {",
    ] {
        assert!(
            printed.contains(spelling),
            "`{spelling}` is lost:\n{printed}"
        );
    }
}

#[test]
fn every_spelling_is_formatted_to_a_fixed_point() {
    let once = format_source(EVERY_SPELLING, "spellings.wf").expect("formats");
    assert_eq!(once, EVERY_SPELLING, "the source is already formatted");
    assert_eq!(format_source(&once, "spellings.wf").expect("formats"), once);
}

#[test]
fn every_spelling_survives_the_indented_layout() {
    let indented = to_offside(EVERY_SPELLING, "spellings.wf").expect("goes to .wfx");
    let back = to_braces(&indented, "spellings.wfx").expect("comes back");
    let a = parse_source(EVERY_SPELLING, "a.wf").expect("parses");
    let b = parse_source(&back, "b.wf").expect("parses");
    assert_eq!(file_text(&a), file_text(&b));
}

#[test]
fn every_spelling_checks() {
    let program = parse_source(EVERY_SPELLING, "spellings.wf").expect("parses");
    let errors: Vec<String> = webfluent::sema::types::check(&program, &|_| "spellings.wf".into())
        .findings
        .errors
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

/// The codes the type checker reports for `src`.
fn codes(src: &str) -> Vec<String> {
    let program = parse_source(src, "t.wf").unwrap_or_else(|e| panic!("{e}"));
    let info = webfluent::sema::types::check(&program, &|_| "t.wf".into());
    info.findings
        .errors
        .iter()
        .chain(info.findings.warnings.iter())
        .map(|d| format!("{} {}", d.code, d.message))
        .collect()
}

fn reports(src: &str, code: &str, words: &str) {
    let found = codes(src);
    assert!(
        found
            .iter()
            .any(|f| f.starts_with(code) && f.contains(words)),
        "expected {code} saying `{words}`, got {found:#?}"
    );
}

fn page(body: &str) -> String {
    format!(
        "type Settings {{ theme: String }}\nenum Region {{ north, south }}\npage P(path: \"/\") {{\n{body}\n    Text(\"x\")\n}}\n"
    )
}

#[test]
fn a_dictionary_holds_values_of_its_type_under_keys_of_its_type() {
    assert!(
        codes(&page("state d: [String: Number] = { a: 1, b: 2 }"))
            .iter()
            .all(|c| !c.starts_with("T0"))
    );
    reports(
        &page("state d: [String: Number] = { a: \"one\" }"),
        "T01",
        "`a` is `String`, but `Number` is wanted",
    );
    reports(
        &page("state d: [Region: Number] = { west: 1 }"),
        "T02",
        "`Region` has no case `.west`",
    );
    // A key may be missing, however it is read.
    reports(
        &page("state d: [String: Number] = { a: 1 }\n    derived n: Number = d.a"),
        "T01",
        "`Number?`",
    );
    reports(
        &page("state d: [String: Number] = { a: 1 }\n    derived n: Number = d[\"a\"]"),
        "T01",
        "`Number?`",
    );
    reports(
        &page("state d: [String: Number] = { a: 1 }\n    derived n = d.sizes()"),
        "T16",
        "has no method `sizes`",
    );
    reports(
        &page("state d: [String: Number] = { a: 1 }\n    derived n = d.with(\"b\", \"two\")"),
        "T01",
        "`String`, but `Number` is wanted",
    );
}

#[test]
fn an_alias_is_the_type_it_names() {
    let src = "type Scores = [String: Number]\npage P(path: \"/\") {\n    state s: Scores = { a: \"x\" }\n    Text(\"x\")\n}\n";
    reports(src, "T01", "`a` is `String`, but `Number` is wanted");
}

#[test]
fn an_action_returns_what_it_declares() {
    reports(
        "store S {\n    action n() -> Number {\n        return \"x\"\n    }\n}\n",
        "T01",
        "`String`, but `Number` is wanted",
    );
    reports(
        "store S {\n    action n() -> Number {\n        log(\"x\")\n    }\n}\n",
        "T01",
        "says it returns `Number`, and returns nothing",
    );
    // A caller sees what it declares.
    reports(
        "store S {\n    action n() -> Number {\n        return 1\n    }\n}\npage P(path: \"/\") {\n    derived t: String = S.n()\n    Text(t)\n}\n",
        "T01",
        "`Number`, but `String` is wanted",
    );
}

#[test]
fn a_function_type_holds_what_it_is_given() {
    let picker = "component Picker(describe: (Region) -> String) { Text(\"x\") }\n";
    let src = format!("{picker}{}", page("Picker(describe: r => \"{r}\")"));
    assert!(
        codes(&src).iter().all(|c| !c.starts_with("T01")),
        "{:?}",
        codes(&src)
    );
    let src = format!("{picker}{}", page("Picker(describe: r => 1)"));
    reports(&src, "T01", "`describe` of `Picker`");
}

#[test]
fn a_lambda_parameter_says_its_type() {
    reports(
        &page("derived n = [1, 2].map((x: String) => x)"),
        "T01",
        "`x` is given `Number`, but says it takes `String`",
    );
    assert!(
        codes(&page(
            "derived f = (a: Settings) => a.theme\n    Text(f(Settings(theme: \"x\")))"
        ))
        .iter()
        .all(|c| !c.starts_with("T0")),
    );
    reports(
        &page("derived f = (a: Settings) => a.thme"),
        "T05",
        "has no field `thme`",
    );
}

#[test]
fn json_is_written_by_its_name_and_matched_by_what_it_is() {
    assert!(
        codes(&page("state raw: Json = Json.string(\"x\")"))
            .iter()
            .all(|c| !c.starts_with("T0"))
    );
    reports(
        &page("state raw: Json = .null"),
        "T01",
        "written by its type's name: `Json.null`",
    );
    reports(
        &page("state raw: Json = Json.nul"),
        "T02",
        "`Json` has no case `.nul`",
    );
    reports(
        &page("state raw: Json = Json.number(\"1\")"),
        "T01",
        "`String`, but `Number` is wanted",
    );
    // Plain data is JSON; JSON is not plain data until it is matched.
    assert!(codes(&page("state raw: Json = Json.null\n    state s: Settings = Settings(theme: \"x\")\n    derived r: Json = s")).iter().all(|c| !c.starts_with("T01")));
    reports(
        &page("state raw: Json = Json.null\n    derived s: String = raw"),
        "T01",
        "`Json`, but `String` is wanted",
    );
    reports(
        &page(
            "state raw: Json = Json.null\n    match raw {\n        .string(s) { Text(s) }\n    }",
        ),
        "T15",
        "",
    );
}

#[test]
fn decode_reads_a_type_out_of_json_as_a_result() {
    let ok = page(
        "match Settings.decode(Json.null) {\n        .ok(s) { Text(s.theme) }\n        .err(e) { Text(\"{e.at} {e.message}\") }\n    }",
    );
    assert!(
        codes(&ok).iter().all(|c| !c.starts_with('T')),
        "{:?}",
        codes(&ok)
    );
    reports(
        &page(
            "match Settings.decode(Json.null) {\n        .ok(s) { Text(s.thme) }\n        .err(e) { Text(e.at) }\n    }",
        ),
        "T05",
        "has no field `thme`",
    );
    reports(
        &page("match Settings.decode(Json.null) {\n        .ok(s) { Text(s.theme) }\n    }"),
        "T15",
        "no arm for `.err`",
    );
    reports(
        &page(
            "match Settings.decode(Json.null) {\n        .done(s) { Text(s.theme) }\n        else { Text(\"x\") }\n    }",
        ),
        "T02",
        "a result has no case `.done`",
    );
}
