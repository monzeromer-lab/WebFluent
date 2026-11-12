//! What the JavaScript code generator wrote wrong (review, Phase 0b): a
//! state named in another script that never repainted, a backslash or a
//! `${` in text that changed what the text says, the last page's state
//! leaking into the `app`, numbers cast without a word, and string literals
//! escaped by hand.

use crate::common;

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::spa_js;
use webfluent::codegen::jscheck::check_js;

fn page(body: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\n"
    )
}

/// The bundle, read back as the build reads it.
fn js(src: &str) -> String {
    let out = spa_js(src);
    if let Err(fault) = check_js(&out) {
        panic!(
            "the bundle is not a script a browser takes: {} at line {}\n{}",
            fault.message,
            fault.line,
            webfluent::codegen::jscheck::line_of(&out, fault.line)
        );
    }
    out
}

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-cg0b-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    dir
}

fn wf(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// Runs `script` with node and gives back what it printed, or `None` when
/// there is no node on this machine.
fn node(script: &str) -> Option<String> {
    let out = Command::new("node").arg("-e").arg(script).output().ok()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

// ─── B3: a state named in any script repaints ───────────────────────────

#[test]
fn a_state_named_in_any_script_is_read_live() {
    for name in ["عدد", "été", "数", "zähler", "x_ü"] {
        let out = js(&page(&format!(
            "    state {name} = 0\n    Text(\"{{{name}}}\")\n    Button(\"+\") {{ on click {{ {name} = {name} + 1 }} }}"
        )));
        assert!(
            out.contains(&format!("() => `${{_{name}()}}`")),
            "`{name}` is followed:\n{out}"
        );
    }
}

#[test]
fn a_live_binding_named_in_any_script_is_read_live() {
    let out = js(&page(
        "    state عناصر = [\"a\", \"b\"]\n    for عنصر in عناصر by عنصر {\n        Text(\"{عنصر}\")\n    }",
    ));
    assert!(out.contains("() => `${$عنصر()}`"), "{out}");
}

// ─── B4: text is the text written ───────────────────────────────────────

#[test]
fn a_backslash_and_a_dollar_brace_in_text_stay_what_they_are() {
    let out = js(&page(
        "    state name = \"x\"\n    Text(\"C:\\\\dir\\\\{name}\")\n    Text(\"cost $\\{x\\} for {name}\")\n    Text(\"a `tick` and $ and {name}$\")",
    ));
    assert!(out.contains(r"`C:\\dir\\${_name()}`"), "{out}");
    assert!(out.contains(r"`cost \${x} for ${_name()}`"), "{out}");
    assert!(out.contains(r"`a \`tick\` and $ and ${_name()}$`"), "{out}");
    if let Some(said) = node(
        r#"const _name = () => "N"; console.log(`C:\\dir\\${_name()}`); console.log(`cost \${x} for ${_name()}`)"#,
    ) {
        assert_eq!(said, "C:\\dir\\N\ncost ${x} for N");
    }
}

#[test]
fn a_backslash_in_a_store_value_and_an_address_stays_what_it_is() {
    let out = js(&format!(
        "store S {{\n    state who = \"x\"\n    derived path = \"C:\\\\Users\\\\{{who}}\"\n    derived cost = \"$\\{{n\\}} {{who}}\"\n}}\n{}",
        page(
            "    use S\n    state id = 3\n    Text(S.path)\n    Text(S.cost)\n    Link(\"L\", to: \"/p/$\\{a\\}/{id}\")"
        )
    ));
    assert!(out.contains(r"`C:\\Users\\${"), "{out}");
    assert!(out.contains(r"`\${n} ${"), "{out}");
    assert!(out.contains(r"`/p/\${a}/${encodeURIComponent("), "{out}");
}

// ─── B5: a page's state does not leak into the app ──────────────────────

#[test]
fn the_app_does_not_read_the_last_pages_validation() {
    let out = js(&format!(
        "{}app {{\n    state agree = false\n    Checkbox(bind: agree, label: \"Agree\")\n    Router\n}}\n",
        page(
            "    state agree = false\n    validate agree { custom \"Agree first\" { agree } }\n    Form { Checkbox(bind: agree, label: \"I agree\") Button(\"Go\", type: .submit) }"
        )
    ));
    let app = &out[out.rfind("const _app").expect("the app")..];
    assert!(
        !app.contains("_agree_check"),
        "the app declares no validation of its own:\n{app}"
    );
    assert!(!app.contains("_wfForm"), "{app}");
    // The page still has its own.
    assert!(out.contains("const _agree_check = WF.validate("), "{out}");
}

#[test]
fn the_app_does_not_read_the_last_pages_refs_or_resources() {
    let out = js(&format!(
        "{}app {{\n    state box = 1\n    Text(\"{{box}}\")\n    Router\n}}\n",
        page(
            "    resource box = fetch(\"/api/box\")\n    Input(placeholder: \"x\", ref: field)\n    Button(\"Focus\") { on click { field.focus() } }"
        )
    ));
    let app = &out[out.rfind("const _app").expect("the app")..];
    // `box` is the app's own state, read as a signal, not the page's
    // resource handle.
    assert!(app.contains("_box()"), "{app}");
    assert!(!app.contains("_box.data"), "{app}");
}

// ─── B12: numbers that cannot be what they say ──────────────────────────

#[test]
fn a_carousel_interval_or_column_span_out_of_range_is_refused() {
    for (call, says) in [
        (
            "Carousel(autoplay: true, interval: -5000) { Carousel.Slide { Text(\"a\") } }",
            "interval",
        ),
        (
            "Carousel(autoplay: true, interval: 0) { Carousel.Slide { Text(\"a\") } }",
            "interval",
        ),
        (
            "Carousel(autoplay: true, interval: 99999999999) { Carousel.Slide { Text(\"a\") } }",
            "interval",
        ),
        ("Row { Column(span: 13) { Text(\"a\") } }", "span"),
        ("Row { Column(span: 2.5) { Text(\"a\") } }", "span"),
        ("Row { Column(span: 0) { Text(\"a\") } }", "span"),
    ] {
        let dir = project(
            "numbers",
            &[
                ("webfluent.app.json", r#"{"name":"t"}"#),
                ("src/App.wf", &page(&format!("    {call}"))),
            ],
        );
        let (ok, said) = wf(&dir, &["check"]);
        assert!(!ok, "`{call}` is refused:\n{said}");
        assert!(
            said.contains("T01") && said.contains(says),
            "{call}: {said}"
        );
    }
    let dir = project(
        "numbers-ok",
        &[
            ("webfluent.app.json", r#"{"name":"t"}"#),
            (
                "src/App.wf",
                &page(
                    "    Carousel(autoplay: true, interval: 4000) { Carousel.Slide { Text(\"a\") } }\n    Row { Column(span: 12) { Text(\"a\") } Column(span: 1) { Text(\"b\") } }",
                ),
            ),
        ],
    );
    let (ok, said) = wf(&dir, &["check"]);
    assert!(ok, "{said}");
}

#[test]
fn the_generator_never_writes_a_cast_number() {
    let out = js(&page(
        "    Carousel(autoplay: true, interval: -5000) { Carousel.Slide { Text(\"a\") } }\n    Carousel(autoplay: true, interval: 1000000000000) { Carousel.Slide { Text(\"b\") } }\n    Row { Column(span: 2.5) { Text(\"a\") } Column(span: 99) { Text(\"b\") } Column(span: 6) { Text(\"c\") } }",
    ));
    assert!(!out.contains("interval: 0,"), "{out}");
    assert!(out.contains("interval: 5000,"), "{out}");
    assert!(out.contains("interval: 2147483647,"), "{out}");
    assert!(!out.contains("wf-col--2\""), "{out}");
    assert!(!out.contains("wf-col--99"), "{out}");
    assert!(out.contains("wf-col--6"), "{out}");
}

// ─── JS string literals, one escaper ────────────────────────────────────

#[test]
fn every_string_the_generator_writes_is_a_string_a_browser_reads() {
    let out = js(
        "page P(path: \"/a\\\"b\", title: \"Line one\\nline \\\"two\\\" \\\\ three\",description: \"D\", guard: false, redirect: \"/x\\\\\\\"y\\nz\") {\n    Heading(\"T\").h1\n    on key(\"Enter\") { log(\"x\") }\n}\n\
         page Q(path: \"/\", title: \"Q\", description: \"D\") { Heading(\"Q\").h1 }\n\
         api B(base: \"/api\") {\n    get one(id: String) at \"one/\\\"x\\\"/:id\"\n}\n",
    );
    assert!(
        out.contains(r#"title: "Line one\nline \"two\" \\ three"#),
        "{out}"
    );
    assert!(out.contains(r#"redirect: "/x\\\"y\nz""#), "{out}");
    assert!(out.contains(r#"path: "/a\"b""#), "{out}");
    assert!(out.contains(r#"path: "one/\"x\"/:id""#), "{out}");
}

#[test]
fn a_base_path_and_locale_names_are_strings() {
    let program = common::parse_program(&page("    Text(t(\"hi\"))")).unwrap();
    let mut codegen = webfluent::codegen::JsCodegen::new();
    codegen.set_base_path("/my \"site\"\\".to_string());
    let mut tables = std::collections::HashMap::new();
    tables.insert(
        "e\"n".to_string(),
        std::collections::HashMap::from([("hi".to_string(), "Hi".to_string())]),
    );
    codegen.set_i18n("e\"n".to_string(), tables);
    let out = codegen.generate(&program);
    check_js(&out).unwrap_or_else(|f| panic!("{} at {}\n{out}", f.message, f.line));
    assert!(
        out.contains(r#"WF.setBasePath("/my \"site\"\\");"#),
        "{out}"
    );
    assert!(out.contains(r#""e\"n","#), "{out}");
}
