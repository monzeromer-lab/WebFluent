//! What the build paints and what the page computes, held to each other.
//!
//! Every method the compiler maps is computed twice: at build time, by the
//! static paint (`codegen::static_eval`), so the first HTML already shows it;
//! and in the browser, by the runtime the bundle carries. Both are written by
//! hand, and three times this release they disagreed without a test to say
//! so — `toUpper` painted empty because the static paint knew only
//! `toUpperCase`; `t(…)` painted empty anywhere but the whole of a text; and
//! `WF.removeAt` was emitted with no runtime function behind it.
//!
//! So: every runtime function the code generator can call must exist, and
//! every method must either paint exactly what the runtime computes or say,
//! in `RUNTIME_ONLY`, why it cannot be known at build time. The method lists
//! are the compiler's own tables, so a method cannot be added without being
//! held to this.

mod common;

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use webfluent::codegen::js::{LIST_AND_STRING_HELPERS, SCALAR_METHODS};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The names the runtime exports, as `WF.<name>`.
fn runtime_exports() -> BTreeSet<String> {
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("src/runtime/modules/manifest.json")).unwrap(),
    )
    .unwrap();
    manifest["modules"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["exports"].as_array().unwrap())
        .map(|pair| pair[0].as_str().unwrap().to_string())
        .collect()
}

/// `WF.<name>` in the code generator's source, outside its tests.
fn emitted_runtime_calls() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(root().join("src/codegen"))
        .unwrap()
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let source = source.split("#[cfg(test)]").next().unwrap();
        for (at, _) in source.match_indices("WF.") {
            let name: String = source[at + 3..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                names.insert(name);
            }
        }
    }
    names
}

/// Read, not called: `WF.i18n` is attached once the translations load, and
/// `WF._basePath` is a getter on the object.
const NOT_FUNCTIONS: &[&str] = &["i18n", "_basePath"];

#[test]
fn every_runtime_function_the_compiler_can_call_exists() {
    let exported = runtime_exports();
    let mut wanted = emitted_runtime_calls();
    wanted.extend(
        SCALAR_METHODS
            .iter()
            .map(|(_, runtime)| runtime.to_string()),
    );
    wanted.extend(LIST_AND_STRING_HELPERS.iter().map(|h| h.to_string()));
    let missing: Vec<&String> = wanted
        .iter()
        .filter(|n| !exported.contains(*n) && !NOT_FUNCTIONS.contains(&n.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "the code generator can call these, and the runtime exports none of them: {missing:?}"
    );
}

/// The methods `method_to_js` maps by name, read from its source: `push`,
/// `remove`, `filter`, `toUpper`, `contains`, …
fn mapped_methods() -> Vec<String> {
    let source = std::fs::read_to_string(root().join("src/codegen/js.rs")).unwrap();
    let start = source.find("pub fn method_to_js").expect("method_to_js");
    let body = &source[start..start + source[start..].find("\n}\n").unwrap()];
    let mut names = Vec::new();
    for line in body.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix('"')
            && let Some(end) = rest.find('"')
            && rest[end + 1..].trim_start().starts_with("=>")
        {
            names.push(rest[..end].to_string());
        }
    }
    names
}

/// A method the build cannot compute, and why.
const RUNTIME_ONLY: &[(&str, &str)] = &[
    (
        "native",
        "hands a library the platform's own object — a JavaScript Date, a URL — which exists only in the browser",
    ),
    (
        "preview",
        "an object URL for a File the reader picked, which exists only in the browser",
    ),
    (
        "inZone",
        "needs the time-zone database, which the browser's Intl carries and the compiler does not",
    ),
    (
        "push",
        "a statement that changes a list, not a value to paint",
    ),
    (
        "remove",
        "a statement that changes a list, not a value to paint",
    ),
];

/// The values every sample reads, written once for the build and once for
/// the runtime, as the carriers the language gives them at run time.
const DECLARATIONS: &str = r#"
const D: Date = @2026-03-14
const T: Time = @09:30
const DT: DateTime = @2026-03-14T09:30:15Z
const DUR: Duration = 90.minutes
const P: Money = $9.99
const U: Url = "https://example.com/a/b?page=1"
const Q: Url = "https://example.com/?q=a+b%26c&n=1"
const E: Email = "ada@example.com"
const C: Color = #0F766E
const W = "hello world"
const L = [3, 1, 2, 1]
"#;

const JS_DECLARATIONS: &str = r##"
const D = "2026-03-14", T = "09:30", DT = "2026-03-14T09:30:15Z", DUR = 5400000;
const P = { amount: 999, currency: "USD" }, U = "https://example.com/a/b?page=1";
const Q = "https://example.com/?q=a+b%26c&n=1";
const E = "ada@example.com", C = "#0F766E", W = "hello world", L = [3, 1, 2, 1];
"##;

/// `(method, as WebFluent writes it, as the runtime computes it)`. A splice
/// opens with a name, `(` or `[`, so a string receiver is parenthesised.
const SAMPLES: &[(&str, &str, &str)] = &[
    // Dates, times and moments.
    ("year", "D.year()", "WF.year(D)"),
    ("month", "D.month()", "WF.month(D)"),
    ("day", "D.day()", "WF.day(D)"),
    ("weekday", "D.weekday()", "WF.weekday(D)"),
    ("hour", "T.hour()", "WF.hour(T)"),
    ("minute", "T.minute()", "WF.minute(T)"),
    ("second", "DT.second()", "WF.second(DT)"),
    ("plus", "D.plus(days: 5)", "WF.plus(D, { days: 5 })"),
    ("plus", "format(P.plus(P))", "WF.format(WF.plus(P, P))"),
    ("minus", "D.minus(days: 20)", "WF.minus(D, { days: 20 })"),
    (
        "isBefore",
        "D.isBefore(@2026-04-01)",
        "WF.isBefore(D, \"2026-04-01\")",
    ),
    (
        "isAfter",
        "D.isAfter(@2026-04-01)",
        "WF.isAfter(D, \"2026-04-01\")",
    ),
    (
        "isSame",
        "D.isSame(@2026-03-14)",
        "WF.isSame(D, \"2026-03-14\")",
    ),
    (
        "until",
        "D.until(@2026-03-20)",
        "WF.until(D, \"2026-03-20\")",
    ),
    ("startOfDay", "DT.startOfDay()", "WF.startOfDay(DT)"),
    ("startOfWeek", "D.startOfWeek()", "WF.startOfWeek(D)"),
    ("startOfMonth", "D.startOfMonth()", "WF.startOfMonth(D)"),
    ("endOfDay", "DT.endOfDay()", "WF.endOfDay(DT)"),
    ("date", "DT.date()", "WF.dateOf(DT)"),
    ("time", "DT.time()", "WF.timeOf(DT)"),
    // Durations.
    ("days", "DUR.days()", "WF.days(DUR)"),
    ("hours", "DUR.hours()", "WF.hours(DUR)"),
    ("minutes", "DUR.minutes()", "WF.minutes(DUR)"),
    ("seconds", "DUR.seconds()", "WF.seconds(DUR)"),
    ("ms", "DUR.ms()", "WF.ms(DUR)"),
    // Money.
    ("times", "format(P.times(3))", "WF.format(WF.times(P, 3))"),
    (
        "convert",
        "format(P.convert(0.5, \"EUR\"))",
        "WF.format(WF.convert(P, 0.5, \"EUR\"))",
    ),
    // Addresses.
    ("host", "U.host()", "WF.host(U)"),
    ("path", "U.path()", "WF.path(U)"),
    ("query", "U.query().page", "WF.urlQuery(U).page"),
    (
        "with",
        "U.with(query: { page: 2 })",
        "WF.urlWith(U, { query: { page: 2 } })",
    ),
    (
        "with",
        "U.with(query: { q: \"a b&c\" })",
        "WF.urlWith(U, { query: { q: \"a b&c\" } })",
    ),
    (
        "with",
        "U.with(query: { page: null })",
        "WF.urlWith(U, { query: { page: null } })",
    ),
    (
        "with",
        "U.with(hash: \"top\", path: \"/c\")",
        "WF.urlWith(U, { hash: \"top\", path: \"/c\" })",
    ),
    ("query", "Q.query().q", "WF.urlQuery(Q).q"),
    ("domain", "E.domain()", "WF.domain(E)"),
    // Colours.
    ("mix", "C.mix(#FFFFFF, 0.5)", "WF.mix(C, \"#FFFFFF\", 0.5)"),
    ("lighten", "C.lighten(0.2)", "WF.lighten(C, 0.2)"),
    ("darken", "C.darken(0.2)", "WF.darken(C, 0.2)"),
    ("alpha", "C.alpha(0.5)", "WF.alpha(C, 0.5)"),
    (
        "contrast",
        "C.contrast(#FFFFFF)",
        "WF.contrast(C, \"#FFFFFF\")",
    ),
    // Lists and strings, through the runtime's helpers.
    (
        "sortBy",
        "L.sortBy(x => x).join(\",\")",
        "WF.sortBy(L, (x) => x).join(\",\")",
    ),
    (
        "groupBy",
        "L.groupBy(x => x)[\"1\"].length",
        "WF.groupBy(L, (x) => x)[\"1\"].length",
    ),
    (
        "unique",
        "L.unique().join(\",\")",
        "WF.unique(L).join(\",\")",
    ),
    ("take", "L.take(2).join(\",\")", "WF.take(L, 2).join(\",\")"),
    ("first", "L.first()", "WF.first(L)"),
    ("last", "L.last()", "WF.last(L)"),
    ("capitalize", "W.capitalize()", "WF.capitalize(W)"),
    ("truncate", "W.truncate(5)", "WF.truncate(W, 5)"),
    (
        "dedent",
        "(\"  a\\n  b\").dedent().lines().join(\"|\")",
        "WF.lines(WF.dedent(\"  a\\n  b\")).join(\"|\")",
    ),
    (
        "lines",
        "(\"a\\nb\\nc\").lines().length",
        "WF.lines(\"a\\nb\\nc\").length",
    ),
    ("words", "W.words().length", "WF.words(W).length"),
    // Lists and strings, mapped onto JavaScript's own.
    (
        "filter",
        "L.filter(x => x > 1).length",
        "L.filter((x) => x > 1).length",
    ),
    (
        "map",
        "L.map(x => x * 2).join(\",\")",
        "L.map((x) => x * 2).join(\",\")",
    ),
    ("sum", "L.sum()", "L.reduce((a, b) => a + b, 0)"),
    ("length", "L.length", "L.length"),
    ("toUpper", "W.toUpper()", "W.toUpperCase()"),
    (
        "toLower",
        "(\"HeLLo\").toLower()",
        "\"HeLLo\".toLowerCase()",
    ),
    ("contains", "L.contains(2)", "L.includes(2)"),
    ("trim", "(\"  x  \").trim()", "\"  x  \".trim()"),
    ("split", "W.split(\" \").length", "W.split(\" \").length"),
];

#[test]
fn every_method_has_a_sample_or_a_reason() {
    let mut methods: BTreeSet<String> = SCALAR_METHODS.iter().map(|(m, _)| m.to_string()).collect();
    methods.extend(LIST_AND_STRING_HELPERS.iter().map(|h| h.to_string()));
    methods.extend(mapped_methods());
    let sampled: BTreeSet<&str> = SAMPLES.iter().map(|(m, _, _)| *m).collect();
    let runtime_only: BTreeSet<&str> = RUNTIME_ONLY.iter().map(|(m, _)| *m).collect();

    let neither: Vec<&String> = methods
        .iter()
        .filter(|m| !sampled.contains(m.as_str()) && !runtime_only.contains(m.as_str()))
        .collect();
    assert!(
        neither.is_empty(),
        "these methods compile, and nothing holds the build's value of them to the runtime's — \
         add a sample, or a reason to RUNTIME_ONLY: {neither:?}"
    );
    let both: Vec<&&str> = sampled.intersection(&runtime_only).collect();
    assert!(
        both.is_empty(),
        "sampled and runtime-only at once: {both:?}"
    );
    let unknown: Vec<&&str> = sampled
        .iter()
        .chain(runtime_only.iter())
        .filter(|m| !methods.contains(**m))
        .collect();
    assert!(
        unknown.is_empty(),
        "not a method the compiler maps: {unknown:?}"
    );
}

/// The text of each `s<n>:` element the static paint wrote, by `n`.
fn painted(html: &str) -> Vec<Option<String>> {
    let mut out = vec![None; SAMPLES.len()];
    for (at, _) in html.match_indices(">s") {
        let rest = &html[at + 2..];
        let Some(colon) = rest.find(':') else {
            continue;
        };
        let Ok(n) = rest[..colon].parse::<usize>() else {
            continue;
        };
        let text = &rest[colon + 1..rest.find('<').unwrap_or(rest.len())];
        let text = text
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .replace("&amp;", "&");
        if n < out.len() {
            out[n] = Some(text);
        }
    }
    out
}

/// What the runtime computes for each sample, as text the way a text node
/// shows a value; `None` where node is not installed.
fn computed() -> Option<Vec<String>> {
    if Command::new("node").arg("--version").output().is_err() {
        return None;
    }
    let calls: Vec<String> = SAMPLES
        .iter()
        .map(|(_, _, js)| format!("  (() => {{ try {{ return String({js}); }} catch (e) {{ return `threw: ${{e.message}}`; }} }})(),"))
        .collect();
    let js_dir = root().join("tests/js");
    let script = format!(
        r#"import {{ fullRuntime }} from {runtime:?};
import {{ makeDom }} from {dom:?};
const {{ window, document, Node, Element, DocumentFragment }} = makeDom();
const WF = new Function("window", "document", "Node", "Element", "DocumentFragment", fullRuntime() + "\nreturn WF;")(window, document, Node, Element, DocumentFragment);
{JS_DECLARATIONS}
console.log(JSON.stringify([
{calls}
]));
"#,
        runtime = format!("file://{}", js_dir.join("runtime.mjs").display()),
        dom = format!("file://{}", js_dir.join("dom.mjs").display()),
        calls = calls.join("\n"),
    );
    let file = std::env::temp_dir().join(format!("wf-agree-{}.mjs", std::process::id()));
    std::fs::write(&file, script).unwrap();
    let out = Command::new("node").arg(&file).output().expect("run node");
    let _ = std::fs::remove_file(&file);
    assert!(
        out.status.success(),
        "the runtime script failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(serde_json::from_slice(&out.stdout).expect("the runtime's values"))
}

#[test]
fn the_static_paint_shows_what_the_runtime_computes() {
    let Some(runtime) = computed() else {
        eprintln!("skipped: node is not installed");
        return;
    };
    let mut page = format!(
        "{DECLARATIONS}\npage P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n"
    );
    for (n, (_, wf, _)) in SAMPLES.iter().enumerate() {
        page.push_str(&format!("    Text(\"s{n}:{{{wf}}}\")\n"));
    }
    page.push_str("}\n");
    let html = common::raw_output(common::Backend::Ssg, &page);
    let painted = painted(&html);

    let mut wrong = Vec::new();
    for (n, (method, wf, _)) in SAMPLES.iter().enumerate() {
        match &painted[n] {
            None => wrong.push(format!(
                "  {method:12} {wf}\n      build:   (painted nothing)\n      runtime: {}",
                runtime[n]
            )),
            Some(text) if *text != runtime[n] => wrong.push(format!(
                "  {method:12} {wf}\n      build:   {text}\n      runtime: {}",
                runtime[n]
            )),
            _ => {}
        }
    }
    assert!(
        wrong.is_empty(),
        "the page shows one value before its script runs and another after:\n{}",
        wrong.join("\n")
    );
}

/// Every runtime module is concatenated into one closure, so a name one
/// module declares at its top level is a name every other module sees — and
/// when two declare the same one, the later wins, silently. `http.js` had a
/// `lines` of its own, the stream reader, and so `text.lines()` handed back
/// an async generator wherever the network module was built in.
#[test]
fn no_two_runtime_modules_declare_the_same_name() {
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("src/runtime/modules/manifest.json")).unwrap(),
    )
    .unwrap();
    let mut seen: std::collections::BTreeMap<String, String> = Default::default();
    let mut clashes = Vec::new();
    for module in manifest["modules"].as_array().unwrap() {
        let name = module["name"].as_str().unwrap();
        let source =
            std::fs::read_to_string(root().join(format!("src/runtime/modules/{name}.js"))).unwrap();
        for line in source.lines() {
            // A module's top level sits two spaces into the shared closure.
            let Some(rest) = line.strip_prefix("  ") else {
                continue;
            };
            if rest.starts_with(' ') {
                continue;
            }
            let rest = rest.strip_prefix("async ").unwrap_or(rest);
            let declared = ["function* ", "function ", "const ", "let ", "var "]
                .iter()
                .find_map(|kw| rest.strip_prefix(kw));
            let Some(declared) = declared else { continue };
            let ident: String = declared
                .trim_start_matches('*')
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
                .collect();
            if ident.is_empty() {
                continue;
            }
            if let Some(first) = seen.get(&ident) {
                if first != name {
                    clashes.push(format!("`{ident}` in {first}.js and {name}.js"));
                }
            } else {
                seen.insert(ident, name.to_string());
            }
        }
    }
    assert!(
        clashes.is_empty(),
        "two runtime modules declare one name, and the later silently wins: {clashes:?}"
    );
}
