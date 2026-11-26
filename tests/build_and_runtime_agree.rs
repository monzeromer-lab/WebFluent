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

use crate::common;

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
        &std::fs::read_to_string(root().join("src/web/runtime/modules/manifest.json")).unwrap(),
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
    // The code generator, and the files of it that sit in lower layers.
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path);
            }
        }
    }

    let mut names = BTreeSet::new();
    let mut paths = Vec::new();
    walk(&root().join("src/web/codegen"), &mut paths);
    paths.extend(
        [
            "src/syntax/node_id.rs",
            "src/registry/style_tokens.rs",
            "src/project/builtin.rs",
            "src/project/format.rs",
            "src/project/sanitize.rs",
            "src/project/scoped_css.rs",
            "src/project/static_eval.rs",
            "src/project/url.rs",
            "src/build/offline.rs",
        ]
        .map(|p| root().join(p)),
    );
    for path in paths {
        // A module's tests, in a file of their own or inline, are not the
        // code generator.
        if path.extension().is_none_or(|e| e != "rs") || path.ends_with("tests.rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        // `mod tests;` names a file of its own; anything else under
        // `#[cfg(test)]` is tests written inline.
        let source = source.replace("#[cfg(test)]\nmod tests;", "");
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
    wanted.extend(
        LIST_AND_STRING_HELPERS
            .iter()
            .map(std::string::ToString::to_string),
    );
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
    let source = std::fs::read_to_string(root().join("src/web/codegen/js/expr.rs")).unwrap();
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
    methods.extend(
        LIST_AND_STRING_HELPERS
            .iter()
            .map(std::string::ToString::to_string),
    );
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
    if !node_available("the method parity check") {
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

/// Texts put together from the pieces Markdown is made of, by a fixed
/// sequence: the case table holds what was thought of, and these the
/// orders nobody would think to write — a fence left open at the end, a
/// Windows line ending in a list, an entity against a link's bracket.
fn markdown_mixtures(count: usize) -> Vec<String> {
    #[rustfmt::skip]
    const PIECES: &[&str] = &[
        "a", "b ", " ", "  ", "\n", "\n\n", "\r\n", "`", "``", "*", "**", "_", "[", "]", "(", ")",
        "!", "](", "![", "[a](", "#", "# ", "> ", "- ", "1. ", "```\n", "```wf\n", "---", "\\", "<",
        ">", "\"", "'", "&", ";", "&lt;", "&amp;", "&#42;", "&#x5F;", "&#96;", "&#000060;", "&#0;",
        "&#x110000;", "&nbsp;", "&bogus;", "/x", "//h/y", "x.png", "https://a.b/c", "mailto:",
        "javascript:", "JavaScript:", "data:", "x:y", "\u{1f}", "\u{0}", "\u{1}", "\t", "\u{a0}",
        "é", "\\*", "\\_", "\\`", "\\[", "\\]", "\\(", "\\)", "\\!", "\\#", "\\-", "\\>",
        "\\&", "\\:", "\\/", "\\\\", "\\\n", "\\  \n", "\\a", "javascript\\:", "1\\. ",
        "\r", "\u{85}", "\u{feff}", "\u{2028}", "\u{3000}",
    ];
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed as usize
    };
    (0..count)
        .map(|_| {
            let len = next() % 14 + 1;
            (0..len).map(|_| PIECES[next() % PIECES.len()]).collect()
        })
        .collect()
}

/// The Markdown texts both renderers are given: the shared case table's,
/// every entity the inert form decodes, by name and by number, every
/// character a backslash escapes, and a few thousand mixtures, each drawn
/// plain and inert with and without a base path.
fn markdown_inputs() -> Vec<(String, bool, &'static str)> {
    let raw = std::fs::read_to_string(root().join("tests/markdown-cases.json")).unwrap();
    let cases: Vec<(String, serde_json::Value, String)> = serde_json::from_str(&raw).unwrap();
    let mut texts: Vec<String> = cases.into_iter().map(|(text, _, _)| text).collect();
    for (name, ch) in webfluent::codegen::markdown::ENTITIES {
        texts.push(format!(
            "&{name}; &#{}; &#x{:X}; `&{name};`",
            *ch as u32, *ch as u32
        ));
    }
    // Every character a backslash escapes, everywhere it could mean
    // something: alone, at a line's start, in emphasis, in a link's words
    // and its URL, in code, in a fence and its language, and before a
    // line's end.
    for ch in "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars() {
        texts.push(format!(
            "\\{ch} a\\{ch}b\n\\{ch}{ch} x\n*a\\{ch}* [a\\{ch}](/b\\{ch}c) ![\\{ch}](x\\{ch}.png) `\\{ch}`\\\n\\{ch}\\\\\n\n```\\{ch}\n\\{ch}\n```\n{ch}\\"
        ));
    }
    texts.extend(markdown_mixtures(3000));
    let mut inputs = Vec::new();
    for text in texts {
        for inert in [false, true] {
            for base in ["", "/site"] {
                inputs.push((text.clone(), inert, base));
            }
        }
    }
    inputs
}

/// `Markdown(text)` is painted at build time by `codegen::markdown` and
/// drawn again in the browser by `WF.markdown`: the two must give the same
/// bytes, or a hydrated page changes under the reader — and a link one of
/// them refuses is a link the other draws.
#[test]
fn markdown_draws_the_same_at_build_time_and_in_the_browser() {
    if !node_available("the Markdown parity check") {
        return;
    }
    let inputs = markdown_inputs();
    let js_dir = root().join("tests/js");
    let script = format!(
        r#"import {{ fullRuntime }} from {runtime:?};
import {{ makeDom }} from {dom:?};
const {{ window, document, Node, Element, DocumentFragment }} = makeDom();
const WF = new Function("window", "document", "Node", "Element", "DocumentFragment", fullRuntime() + "\nreturn WF;")(window, document, Node, Element, DocumentFragment);
const inputs = {inputs};
console.log(JSON.stringify(inputs.map(([text, inert, base]) => {{
  WF.setBasePath(base);
  return WF.markdown(text, inert ? {{ inert: true }} : undefined);
}})));
"#,
        runtime = format!("file://{}", js_dir.join("runtime.mjs").display()),
        dom = format!("file://{}", js_dir.join("dom.mjs").display()),
        inputs = serde_json::to_string(&inputs).unwrap(),
    );
    let file = std::env::temp_dir().join(format!("wf-agree-md-{}.mjs", std::process::id()));
    std::fs::write(&file, script).unwrap();
    let out = Command::new("node").arg(&file).output().expect("run node");
    let _ = std::fs::remove_file(&file);
    assert!(
        out.status.success(),
        "the runtime script failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let drawn: Vec<String> = serde_json::from_slice(&out.stdout).expect("the runtime's HTML");
    let mut wrong = Vec::new();
    for ((text, inert, base), runtime) in inputs.iter().zip(&drawn) {
        let built = webfluent::codegen::markdown::render_with(
            text,
            webfluent::codegen::markdown::Options {
                base,
                inert: *inert,
                ..Default::default()
            },
        );
        if built != *runtime {
            wrong.push(format!(
                "  {text:?} (inert: {inert}, base: {base:?})\n      build:   {built:?}\n      runtime: {runtime:?}"
            ));
        }
    }
    assert_eq!(drawn.len(), inputs.len());
    assert!(
        wrong.is_empty(),
        "the page paints one Markdown before its script runs and another after:\n{}",
        wrong.join("\n")
    );
}

/// The runtime decodes the named entities the compiler does, and no
/// others: one name more on either side is a page whose words change when
/// its script runs.
#[test]
fn the_runtime_names_the_entities_the_compiler_does() {
    let source = std::fs::read_to_string(root().join("src/web/runtime/modules/text.js")).unwrap();
    let table = source
        .split("const MD_NAMED = {")
        .nth(1)
        .and_then(|rest| rest.split("};").next())
        .expect("the runtime's MD_NAMED table");
    let runtime: BTreeSet<String> = table
        .split(',')
        .filter_map(|pair| pair.split(':').next())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();
    let compiler: BTreeSet<String> = webfluent::codegen::markdown::ENTITIES
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    assert_eq!(runtime, compiler);
}

/// Every runtime module is concatenated into one closure, so a name one
/// module declares at its top level is a name every other module sees — and
/// when two declare the same one, the later wins, silently. `http.js` had a
/// `lines` of its own, the stream reader, and so `text.lines()` handed back
/// an async generator wherever the network module was built in.
#[test]
fn no_two_runtime_modules_declare_the_same_name() {
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("src/web/runtime/modules/manifest.json")).unwrap(),
    )
    .unwrap();
    let mut seen: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::default();
    let mut clashes = Vec::new();
    for module in manifest["modules"].as_array().unwrap() {
        let name = module["name"].as_str().unwrap();
        let source =
            std::fs::read_to_string(root().join(format!("src/web/runtime/modules/{name}.js")))
                .unwrap();
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

/// Whether `node` can run the runtime here. A machine without it skips the
/// tests that need it — but CI must not: there a missing `node` is a
/// failure, not a pass that held nothing.
fn node_available(what: &str) -> bool {
    let found = Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !found {
        assert!(
            std::env::var_os("CI").is_none(),
            "`node` is not on PATH, and CI is set: {what} would have passed without running"
        );
        eprintln!("skipped: node is not installed; {what} did not run");
    }
    found
}

/// Whether this `node` reads the CLDR the build's tables were written from
/// (`src/project/format_locales.rs` names its ICU). Another ICU words some
/// locales otherwise — ICU 78.2 writes Korean's day period `AM` where 78.3
/// and Chrome write `오전` — so a comparison with it says nothing about the
/// build.
fn node_has_the_tables_icu(what: &str) -> bool {
    if !node_available(what) {
        return false;
    }
    let tables = std::fs::read_to_string(root().join("src/project/format_locales.rs")).unwrap();
    let written = tables
        .split("from ICU ")
        .nth(1)
        .and_then(|rest| rest.split(' ').next())
        .expect("format_locales.rs names the ICU it was written from")
        .to_string();
    let out = Command::new("node")
        .args(["-p", "process.versions.icu"])
        .output()
        .unwrap();
    let running = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // The whole version: ICU 78.2 wrote Korean's day period `AM`, 78.3
    // and the browsers `오전`.
    let same = running == written;
    if !same {
        eprintln!(
            "skipped: node has ICU {running}, the tables were written from ICU {written}; \
             {what} did not run. Run it with that Node, or write the tables again: \
             TZ=UTC node tests/js/format-tables.mjs > src/project/format_locales.rs"
        );
    }
    same
}

/// The locales the build's formatting is held to the browser's in.
const FORMAT_LOCALES: &[&str] = &[
    "en",
    "en-US",
    "en-GB",
    "en-IN",
    "de",
    "de-DE",
    "de-AT",
    "de-CH",
    "fr",
    "fr-CA",
    "es",
    "es-MX",
    "it",
    "pt-BR",
    "pt-PT",
    "nl",
    "ar",
    "ar-EG",
    "ar-SA",
    "ar-MA",
    "ar-SY",
    "ar-u-nu-arab",
    "ar-EG-u-nu-latn",
    "ja",
    "zh",
    "zh-TW",
    "ko",
    "hi",
    "ru",
    "uk",
    "tr",
    "pl",
    "sv",
    "he",
];

type FormatCase = (
    serde_json::Value,
    Option<&'static str>,
    Option<serde_json::Value>,
);

/// `(value, style, option)` for every number style.
fn number_cases() -> Vec<FormatCase> {
    use serde_json::json;
    let values = [
        "0",
        "-0",
        "7",
        "-0.5",
        "0.001",
        "1.005",
        "2.5",
        "-2.5",
        "999",
        "1234",
        "1234.5",
        "12345.678",
        "-9876543.21",
        "999999.5",
        "123456789.123",
        "1e21",
    ];
    let compact = [
        "999.5",
        "1500",
        "3000",
        "10000",
        "12345",
        "99999",
        "999999",
        "2000000",
        "1500000000",
        "33000000000",
        "1e15",
        "-4321",
        "0.0123",
    ];
    let styles: Vec<(Option<&'static str>, Option<serde_json::Value>)> = vec![
        (None, None),
        (Some("integer"), None),
        (Some("decimal"), None),
        (Some("decimal"), Some(json!(0))),
        (Some("decimal"), Some(json!(3))),
        (Some("currency"), None),
        (Some("currency"), Some(json!("EUR"))),
        (Some("currency"), Some(json!("JPY"))),
        (Some("currency"), Some(json!("GBP"))),
        (Some("currency"), Some(json!("SAR"))),
        (Some("currency"), Some(json!("INR"))),
        (Some("currency"), Some(json!("KWD"))),
        (Some("currency"), Some(json!("CHF"))),
        (Some("currency"), Some(json!("XYZ"))),
        (Some("percent"), None),
        (Some("percent"), Some(json!(1))),
        (Some("compact"), None),
    ];
    let mut cases = Vec::new();
    for v in values {
        for (style, option) in &styles {
            cases.push((json!({ "n": v }), *style, option.clone()));
        }
    }
    for v in compact {
        cases.push((json!({ "n": v }), Some("compact"), None));
    }
    for v in ["0.25", "-0.256", "1", "0.0005"] {
        cases.push((json!({ "n": v }), Some("percent"), None));
        cases.push((json!({ "n": v }), Some("percent"), Some(json!(2))));
    }
    cases
}

/// `(value, style, option)` for every date style and a few patterns.
fn date_cases() -> Vec<FormatCase> {
    use serde_json::json;
    let mut cases = Vec::new();
    for value in [
        "2026-03-04",
        "2026-11-14T21:30:07Z",
        "2026-01-01T00:05:00Z",
        "2024-02-29T12:00:00Z",
    ] {
        for style in ["date", "time", "datetime"] {
            cases.push((json!(value), Some(style), None));
            for option in ["short", "medium", "long", "full"] {
                cases.push((json!(value), Some(style), Some(json!(option))));
            }
        }
        for pattern in [
            "yyyy-MM-dd HH:mm:ss",
            "EEE d MMM yy, h a",
            "EEEE MMMM",
            "yyy MM M dd d EE hh",
        ] {
            cases.push((json!(value), Some(pattern), None));
        }
    }
    cases
}

/// `format(value, style, option)` as the runtime computes it, in each
/// case's locale — the browser's `Intl`, here Node's, in UTC.
fn runtime_formats(cases: &[(String, FormatCase)]) -> Vec<String> {
    let js_dir = root().join("tests/js");
    let flat: Vec<_> = cases.iter().map(|(l, (v, s, o))| (l, v, s, o)).collect();
    let script = format!(
        r#"import {{ fullRuntime }} from {runtime:?};
import {{ makeDom }} from {dom:?};
const {{ window, document, Node, Element, DocumentFragment }} = makeDom();
const WF = new Function("window", "document", "Node", "Element", "DocumentFragment", fullRuntime() + "\nreturn WF;")(window, document, Node, Element, DocumentFragment);
const cases = {cases};
console.log(JSON.stringify(cases.map(([locale, value, style, option]) => {{
  document.documentElement.lang = locale;
  const v = value !== null && typeof value === "object" ? Number(value.n) : value;
  try {{ return WF.format(v, style ?? undefined, option ?? undefined); }}
  catch (e) {{ return `threw: ${{e.message}}`; }}
}})));
"#,
        runtime = format!("file://{}", js_dir.join("runtime.mjs").display()),
        dom = format!("file://{}", js_dir.join("dom.mjs").display()),
        cases = serde_json::to_string(&flat).unwrap(),
    );
    let file = std::env::temp_dir().join(format!(
        "wf-agree-format-{}-{}.mjs",
        std::process::id(),
        cases.len()
    ));
    std::fs::write(&file, script).unwrap();
    let out = Command::new("node")
        .arg(&file)
        .env("TZ", "UTC")
        .output()
        .expect("run node");
    let _ = std::fs::remove_file(&file);
    assert!(
        out.status.success(),
        "the runtime script failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("the runtime's values")
}

/// `format(value, style, option)` as the static paint writes it.
fn build_format(
    value: &serde_json::Value,
    style: Option<&str>,
    option: Option<&serde_json::Value>,
    locale: &str,
) -> String {
    use webfluent::codegen::format::{Input, format};
    let input = |v: &serde_json::Value| match v {
        serde_json::Value::Object(o) => Input::Number(o["n"].as_str().unwrap().parse().unwrap()),
        serde_json::Value::Number(n) => Input::Number(n.as_f64().unwrap()),
        serde_json::Value::String(s) => Input::Text(s.clone()),
        other => panic!("not a value: {other}"),
    };
    format(&input(value), style, option.map(input).as_ref(), locale)
        .unwrap_or_else(|| "(nothing)".to_string())
}

fn hold_format_to_runtime(cases: &[FormatCase]) {
    let all: Vec<(String, FormatCase)> = FORMAT_LOCALES
        .iter()
        .flat_map(|l| cases.iter().map(|c| (l.to_string(), c.clone())))
        .collect();
    let runtime = runtime_formats(&all);
    assert_eq!(runtime.len(), all.len());
    let mut wrong = Vec::new();
    let mut count = 0;
    // Two of each locale and style, so one mistake does not hide another.
    let mut shown: std::collections::BTreeMap<(String, Option<&str>), usize> =
        std::collections::BTreeMap::new();
    for ((locale, (value, style, option)), live) in all.iter().zip(&runtime) {
        let built = build_format(value, *style, option.as_ref(), locale);
        if built != *live {
            count += 1;
            let seen = shown.entry((locale.clone(), *style)).or_default();
            *seen += 1;
            if *seen <= 2 && wrong.len() < 80 {
                wrong.push(format!(
                    "  {locale:16} format({value}, {style:?}, {option:?})\n      build:   {built:?}\n      runtime: {live:?}"
                ));
            }
        }
    }
    assert!(
        count == 0,
        "{count} of {} formatted values change when the page takes the static paint over:\n{}",
        all.len(),
        wrong.join("\n")
    );
}

/// `format(n, .currency)` was painted `€1.234,50` in German and became
/// `1.234,50 €` once the page ran: every number style, in every locale the
/// table knows, is what `Intl` writes.
#[test]
fn numbers_are_formatted_as_intl_formats_them_in_every_locale() {
    if !node_has_the_tables_icu("the number-format parity check") {
        return;
    }
    hold_format_to_runtime(&number_cases());
}

/// Dates, times and patterns, in every locale the table knows, are what
/// `Intl` writes (in UTC: the static paint knows no reader's zone).
#[test]
fn dates_are_formatted_as_intl_formats_them_in_every_locale() {
    if !node_has_the_tables_icu("the date-format parity check") {
        return;
    }
    hold_format_to_runtime(&date_cases());
}

/// The words the runtime's highlighter colours as keywords, read from
/// `text.js`'s `HL_KEYWORDS`.
fn runtime_highlight_keywords() -> BTreeSet<String> {
    let source = std::fs::read_to_string(root().join("src/web/runtime/modules/text.js")).unwrap();
    let start = source
        .find("const HL_KEYWORDS = new Set(")
        .expect("the runtime's HL_KEYWORDS");
    let rest = &source[start..];
    let body = &rest[..rest
        .find(".split(")
        .expect("HL_KEYWORDS is split from a string")];
    // The list is string literals joined with `+`: their text, run together.
    let mut words = String::new();
    let mut inside = false;
    for c in body.chars() {
        match (c, inside) {
            ('"', _) => inside = !inside,
            (c, true) => words.push(c),
            _ => {}
        }
    }
    words
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// The keyword list is written twice — the compiler's and the runtime's —
/// and a word on one side only is a block whose colour changes when the
/// page takes it over.
#[test]
fn the_runtime_colours_the_keywords_the_compiler_does() {
    let compiler: BTreeSet<String> = webfluent::codegen::highlight::WF_KEYWORDS
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    let runtime = runtime_highlight_keywords();
    assert_eq!(
        compiler.len(),
        webfluent::codegen::highlight::WF_KEYWORDS.len(),
        "a keyword written twice in codegen::highlight"
    );
    assert_eq!(
        runtime.difference(&compiler).collect::<Vec<_>>(),
        Vec::<&String>::new(),
        "keywords only the runtime colours"
    );
    assert_eq!(
        compiler.difference(&runtime).collect::<Vec<_>>(),
        Vec::<&String>::new(),
        "keywords only the compiler colours"
    );
}

/// `(code, language)`: what each highlighter is held to the other on.
const HIGHLIGHT_CASES: &[(&str, &str)] = &[
    // WebFluent.
    ("page Home(path: \"/\") { // hi\n    state n = 0\n}", "wf"),
    (
        "component Card(_ title: String, open: Bool = false) { children }",
        "wf",
    ),
    (
        "store Todos {\n    state items: [Todo] = []\n    action add(t: String) { items = items.concat([t]) }\n}",
        "wf",
    ),
    (
        "Button(\"Go\", tone: .primary).lg { style { padding: $md; color: #FF0 } }",
        "wf",
    ),
    ("const S = #\"a \"b\" c\"#", "wf"),
    ("const T = ##\"a \"# b\"##", "wf"),
    ("x = \"\"\"\n  a \"b\"\n  \"\"\"", "wf"),
    (
        "Text(\"Hello, {name}! \\\"quoted\\\" {count:.integer}\")",
        "wf",
    ),
    ("t.title t.page item.state", "wf"),
    (
        "for todo, i in Todos.items by todo.id { if todo.done { Text(\"✓\") } else { Text(\"·\") } }",
        "wf",
    ),
    (
        "match users { .loading { Spinner } .failed(e) { Alert(e.message).danger } .loaded(list) { } }",
        "wf",
    ),
    (
        "derived total = items.map(i => i.price * 1.5e3).sum() // tally\n/* block */",
        "wf",
    ),
    (
        "theme Brand { color-primary: #3B82F6  radius-md: 0.5rem }",
        "wf",
    ),
    (
        "every(1000) { tick = tick + 1 }  after(3.seconds) { toast = null }",
        "wf",
    ),
    ("let عدد = 3\nstate المهام = []", "wf"),
    ("a < b && c > d || e == \"<script>\"", "wf"),
    (
        "test \"adds\" { click \"Add\"  expect \"1\"  expect not \"0\" }",
        "wf",
    ),
    (
        "page P(path: \"/\")\n    state n = 0\n    Button(\"+\")\n        on click\n            n = n + 1",
        "wfx",
    ),
    ("\"unterminated string\nnext", "wf"),
    ("@2026-03-14 @09:30 €12.99 $9.99 90.minutes", "wf"),
    // JSON.
    ("{ \"a\": [1, true], \"b\": \"x\" }", "json"),
    (
        "{\n  \"name\": \"My App\",\n  \"build\": { \"ssg\": false, \"budget\": null, \"n\": -1.5e-3 }\n}",
        "json",
    ),
    ("[\"a\\\"b\", \"\\u00e9\", {}, [], false]", "json"),
    ("{ \"html\": \"<b>&amp;</b>\" }", "json"),
    // Shell.
    ("$ wf build\n# then\nwf serve", "bash"),
    (
        "$ wf init my-app -t spa --locales en,ar\n$ cd my-app && wf serve",
        "sh",
    ),
    ("  # indented comment\necho \"<tag>\" > out.txt", "shell"),
    ("$", "bash"),
    // CSS.
    ("color: #FF0", "css"),
    (
        ".card:hover { background: var(--surface); padding: 1rem 2rem }",
        "css",
    ),
    (
        "@media (max-width: 768px) {\n  .a { display: none } /* hide */\n}",
        "css",
    ),
    (
        "a[href^=\"https\"]::after { content: \"↗\"; width: calc(100% - 2px) }",
        "css",
    ),
    // Anything else is escaped.
    ("a < b & c", "python"),
    ("<script>alert(1)</script>", ""),
];

/// Code put together from the pieces the highlighters read, by a fixed
/// sequence, for every language: the orders no case thought to write.
fn highlight_mixtures(count: usize) -> Vec<(String, &'static str)> {
    #[rustfmt::skip]
    const PIECES: &[&str] = &[
        "page", "state", " ", "  ", "\n", "Name", "name", "_x", ".flag", ":", "{", "}", "(", ")",
        "\"", "\"a\"", "\\\"", "#", "#\"", "\"#", "##\"", "\"\"\"", "//", "/*", "*/", "$", "$ ",
        "$md", "#FF0", "0", "1.5", "-2", "1e3", "90.minutes", "@", "<", ">", "&", "'", "→", "é",
        "عدد", "\t", "\r\n", "true", "null", "--x", "--", "-", "=", "=>", "[", "]", ",", ";",
        "url(", "var(", "!important", "%", "px",
    ];
    const LANGS: &[&str] = &["wf", "wfx", "json", "bash", "css"];
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed as usize
    };
    (0..count)
        .map(|_| {
            let len = next() % 16 + 1;
            let code: String = (0..len).map(|_| PIECES[next() % PIECES.len()]).collect();
            (code, LANGS[next() % LANGS.len()])
        })
        .collect()
}

/// `Code(…, language:)` and a fenced block are coloured by
/// `codegen::highlight` in the static paint and by `WF.highlight` in the
/// browser: the two must write the same bytes.
#[test]
fn code_is_coloured_the_same_at_build_time_and_in_the_browser() {
    if !node_available("the highlighter parity check") {
        return;
    }
    let mut inputs: Vec<(String, &str)> = HIGHLIGHT_CASES
        .iter()
        .map(|(code, lang)| (code.to_string(), *lang))
        .collect();
    inputs.extend(highlight_mixtures(2000));
    let js_dir = root().join("tests/js");
    let script = format!(
        r#"import {{ fullRuntime }} from {runtime:?};
import {{ makeDom }} from {dom:?};
const {{ window, document, Node, Element, DocumentFragment }} = makeDom();
const WF = new Function("window", "document", "Node", "Element", "DocumentFragment", fullRuntime() + "\nreturn WF;")(window, document, Node, Element, DocumentFragment);
const inputs = {inputs};
console.log(JSON.stringify(inputs.map(([code, lang]) => WF.highlight(code, lang))));
"#,
        runtime = format!("file://{}", js_dir.join("runtime.mjs").display()),
        dom = format!("file://{}", js_dir.join("dom.mjs").display()),
        inputs = serde_json::to_string(&inputs).unwrap(),
    );
    let file = std::env::temp_dir().join(format!("wf-agree-hl-{}.mjs", std::process::id()));
    std::fs::write(&file, script).unwrap();
    let out = Command::new("node").arg(&file).output().expect("run node");
    let _ = std::fs::remove_file(&file);
    assert!(
        out.status.success(),
        "the runtime script failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let drawn: Vec<String> = serde_json::from_slice(&out.stdout).expect("the runtime's HTML");
    assert_eq!(drawn.len(), inputs.len());
    let wrong: Vec<String> = inputs
        .iter()
        .zip(&drawn)
        .filter_map(|((code, lang), runtime)| {
            let built = webfluent::codegen::highlight::highlight(code, lang);
            (built != *runtime).then(|| {
                format!("  {code:?} ({lang})\n      build:   {built:?}\n      runtime: {runtime:?}")
            })
        })
        .take(30)
        .collect();
    assert!(
        wrong.is_empty(),
        "a code block is coloured one way before its script runs and another after:\n{}",
        wrong.join("\n")
    );
}

/// Markup put together from the pieces a sanitiser reads, by a fixed
/// sequence: the orders no case thought to write — a quote left open, an
/// entity cut short, a tag inside an attribute, a closing tag in capitals.
fn sanitize_mixtures(count: usize) -> Vec<String> {
    #[rustfmt::skip]
    const PIECES: &[&str] = &[
        "<", ">", "</", "/>", "<!--", "-->", "<!", "<?", "\"", "'", "=", " ", "\t", "\n", "\u{a0}",
        "p", "a", "img", "script", "SCRIPT", "style", "svg", "math", "template", "b", "table",
        " href=", " src=", " onclick=", " class=", " rel=", " target=", " title=", " xlink:href=",
        "javascript:", "JaVa\tScRiPt:", "data:image/png;base64,AA", "data:text/html,", "/x", "https://e.com",
        "&", "&#", "&#106;", "&#x6A;", "&#x110000;", "&#xD800;", "&amp;", "&colon;", "&quot;", ";",
        "opener", "noopener", "_blank", "x", "İ", "😀", "alert(1)",
    ];
    let mut seed: u64 = 0x5851_f42d_4c95_7f2d;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed as usize
    };
    (0..count)
        .map(|_| {
            let len = next() % 18 + 1;
            (0..len).map(|_| PIECES[next() % PIECES.len()]).collect()
        })
        .collect()
}

/// `sanitize(html)` is painted at build time by `project::sanitize` and run
/// again in the browser by `WF.sanitize`: the shared case table and a few
/// thousand mixtures must come out the same, byte for byte — markup one
/// side keeps and the other drops is a page that changes as it hydrates,
/// and a door one of them leaves open.
#[test]
fn markup_is_sanitised_the_same_at_build_time_and_in_the_browser() {
    if !node_available("the sanitiser parity check") {
        return;
    }
    let raw = std::fs::read_to_string(root().join("tests/sanitize-cases.json")).unwrap();
    let cases: Vec<(String, String)> = serde_json::from_str(&raw).unwrap();
    let mut inputs: Vec<String> = cases.into_iter().map(|(html, _)| html).collect();
    inputs.extend(sanitize_mixtures(4000));
    let js_dir = root().join("tests/js");
    let script = format!(
        r#"import {{ fullRuntime }} from {runtime:?};
import {{ makeDom }} from {dom:?};
const {{ window, document, Node, Element, DocumentFragment }} = makeDom();
const WF = new Function("window", "document", "Node", "Element", "DocumentFragment", fullRuntime() + "\nreturn WF;")(window, document, Node, Element, DocumentFragment);
const inputs = {inputs};
console.log(JSON.stringify(inputs.map((html) => {{
  try {{ return WF.sanitize(html); }} catch (e) {{ return `threw: ${{e.message}}`; }}
}})));
"#,
        runtime = format!("file://{}", js_dir.join("runtime.mjs").display()),
        dom = format!("file://{}", js_dir.join("dom.mjs").display()),
        inputs = serde_json::to_string(&inputs).unwrap(),
    );
    let file = std::env::temp_dir().join(format!("wf-agree-sanitize-{}.mjs", std::process::id()));
    std::fs::write(&file, script).unwrap();
    let out = Command::new("node").arg(&file).output().expect("run node");
    let _ = std::fs::remove_file(&file);
    assert!(
        out.status.success(),
        "the runtime script failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let drawn: Vec<String> = serde_json::from_slice(&out.stdout).expect("the runtime's HTML");
    assert_eq!(drawn.len(), inputs.len());
    let wrong: Vec<String> = inputs
        .iter()
        .zip(&drawn)
        .filter_map(|(html, runtime)| {
            let built = webfluent::codegen::sanitize::sanitize(html);
            (built != *runtime).then(|| {
                format!("  {html:?}\n      build:   {built:?}\n      runtime: {runtime:?}")
            })
        })
        .take(30)
        .collect();
    assert!(
        wrong.is_empty(),
        "markup is sanitised one way before the script runs and another after:\n{}",
        wrong.join("\n")
    );
}

/// Builds `src` as a static site in a directory of its own, and gives back
/// that directory.
fn static_site(name: &str, src: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-agree-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "t", "build": { "ssg": true } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    dir
}

/// A `:param` page's title was filled by replacing each `{name}` in turn,
/// so a value holding `{other}` was filled again — and which one came first
/// hung on a hash map's order. It is worked out in one pass, each name
/// read through the static evaluator, as the router fills it on the live
/// page: what the file says is what the tab says once the page runs.
#[test]
fn a_routes_title_is_filled_once_as_the_router_fills_it() {
    let dir = static_site(
        "title-once",
        "const PAIRS = [{ a: #\"x{b}\"#, b: \"7\" }, { a: \"one\", b: \"two\" }]\n\
         page Pair(path: \"/p/:a/:b\", a: String, b: String, title: \"{a} and {b} \\{c\\}\", description: \"About {a}.\", paths: PAIRS) {\n\
         \x20   Heading(a).h1\n\
         }\n\
         page Home(path: \"/\", title: \"Home\", description: \"Home.\") { Heading(\"Home\").h1 }\n",
    );
    for (a, b) in [("x{b}", "7"), ("one", "two")] {
        // What the router does: `{name}` of a parameter, once; an escaped
        // brace is a brace.
        let routed = format!("{a} and {b} {{c}}");
        let encoded = a.replace('{', "%7B").replace('}', "%7D");
        let file = ["build/p", &encoded, b, "index.html"].join("/");
        let html = std::fs::read_to_string(dir.join(&file))
            .or_else(|_| std::fs::read_to_string(dir.join(format!("build/p/{a}/{b}/index.html"))))
            .unwrap_or_else(|e| panic!("{file}: {e}"));
        let title = html
            .split("<title>")
            .nth(1)
            .and_then(|t| t.split("</title>").next())
            .unwrap();
        let routed_html = routed.replace('&', "&amp;").replace('<', "&lt;");
        assert_eq!(title, routed_html, "{file}");
        assert!(html.contains(&format!("About {a}.")), "{html}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A title that splices only a constant's name — `"{SITE}"` — was taken for
/// a route parameter's and left as its source, in the file and in the tab.
#[test]
fn a_title_that_names_a_constant_is_worked_out() {
    let dir = static_site(
        "title-const",
        "const SITE = \"Halyard\"\n\
         app { Router }\n\
         page Home(path: \"/\", title: \"{SITE}\", description: \"Home.\") { Heading(\"Home\").h1 }\n\
         page About(path: \"/about\", title: \"About — {SITE}\", description: \"About.\") { Heading(\"About\").h1 }\n",
    );
    for (file, want) in [
        ("build/index.html", "<title>Halyard</title>"),
        ("build/about/index.html", "<title>About — Halyard</title>"),
    ] {
        let html = std::fs::read_to_string(dir.join(file)).unwrap();
        assert!(html.contains(want), "{file}: {html}");
    }
    // The router works it out too, rather than showing `{SITE}`.
    let chunk = std::fs::read_to_string(dir.join("build/pages/About.js")).unwrap();
    assert!(chunk.contains("`About — ${SITE}`"), "{chunk}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A title is worked out where the route table is, so a name it splices
/// must be a parameter, a constant or a store: anything else would be a
/// `ReferenceError` when the route is visited.
#[test]
fn a_title_naming_what_the_route_cannot_see_is_t13() {
    let dir = std::env::temp_dir().join(format!("wf-agree-title-t13-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "t" }"#).unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "page P(path: \"/\", title: \"{count} items\", description: \"x\") { state count = 1  Heading(\"P {count}\").h1 }\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["check", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("T13") && text.contains("`count`"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}
