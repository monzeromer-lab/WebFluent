//! WebFluent 4.2: the project's own JavaScript, and the classes it and the
//! page share (`spec/PROJECT_SCRIPTS.md`).
//!
//! Each case is held to what the reader gets: the static paint, the bundle
//! the browser runs, and the diagnostics a build prints.

mod common;

use common::{spa_generated, ssg_html, template_html};
use webfluent::parse_source;

fn page(body: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\n"
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

fn warnings_of(src: &str, rule: &str) -> Vec<String> {
    let program = webfluent::sema::lower(parse_source(src, "t.wf").unwrap());
    webfluent::linter::lint_vocabulary(&program, "t.wf")
        .into_iter()
        .filter(|w| w.rule_id == rule)
        .map(|w| w.message)
        .collect()
}

// ─── §6a: `class:` takes a map and a list ────────────────────────────

#[test]
fn a_class_map_paints_the_keys_whose_condition_holds() {
    let html = ssg_html(&page(
        "    state done = true\n    state open = false\n    Card(class: { \"is-done\": done, \"is-open\": open }) { Text(\"x\") }",
    ));
    assert!(html.contains("class=\"wf-card is-done\""), "{html}");
}

#[test]
fn a_class_list_paints_its_strings_and_skips_what_is_null() {
    let html = ssg_html(&page(
        "    state open = false\n    state tone = \"calm\"\n    Card(class: [\"feature\", tone, if open { \"is-open\" }, { wide: true }]) { Text(\"x\") }",
    ));
    assert!(
        html.contains("class=\"wf-card feature calm wide\""),
        "{html}"
    );
}

#[test]
fn a_class_value_that_reads_state_is_followed_by_the_runtime() {
    let js = spa_generated(&page(
        "    state done = true\n    Card(class: { \"is-done\": done }) { Text(\"x\") }",
    ));
    assert!(
        js.contains("WF.classes(") && js.contains("\"is-done\": _done()"),
        "{js}"
    );
}

#[test]
fn a_class_map_written_out_in_full_is_a_fixed_list() {
    let js = spa_generated(&page(
        "    Card(class: { fixed: true, off: false, \"a b\": true }) { Text(\"x\") }",
    ));
    assert!(
        js.contains(".classList.add(\"fixed\", \"a\", \"b\");"),
        "{js}"
    );
    assert!(!js.contains("WF.classes("), "{js}");
}

#[test]
fn a_class_value_on_an_icon_button_joins_its_classes_rather_than_replacing_them() {
    let js = spa_generated(&page(
        "    state on = true\n    IconButton(icon: \"close\", label: \"Close\", class: { \"is-on\": on })",
    ));
    assert!(js.contains("WF.classes("), "{js}");
    assert!(
        !js.contains("class: () =>"),
        "a live class is never an attribute: {js}"
    );
}

#[test]
fn the_template_engine_reads_a_class_map_and_list() {
    let html = template_html(&page(
        "    Card(class: [\"feature\", { \"is-on\": true, \"is-off\": false }]) { Text(\"x\") }",
    ));
    assert!(html.contains("wf-card feature is-on"), "{html}");
    assert!(!html.contains("is-off"), "{html}");
}

#[test]
fn an_if_with_no_else_is_null_when_it_fails() {
    let js = spa_generated(&page(
        "    state open = false\n    Text(if open { \"open\" })",
    ));
    assert!(js.contains("? \"open\" : null"), "{js}");
}

#[test]
fn a_class_map_value_must_be_a_condition() {
    let text = errors_of(&page(
        "    state n = 3\n    Card(class: { \"is-many\": n }) { Text(\"x\") }",
    ));
    assert!(text.contains("T01") && text.contains("`is-many`"), "{text}");
}

#[test]
fn a_bare_condition_or_number_is_not_a_class() {
    let text = errors_of(&page(
        "    state n = 3\n    Card(class: n > 1) { Text(\"x\") }",
    ));
    assert!(text.contains("T01") && text.contains("`Bool`"), "{text}");
    let text = errors_of(&page(
        "    state n = 3\n    Card(class: [n]) { Text(\"x\") }",
    ));
    assert!(text.contains("T01") && text.contains("`Number`"), "{text}");
}

#[test]
fn a_class_map_keyed_by_breakpoint_names_is_still_classes() {
    // `md` is a class here, not a width: `class:` is never responsive.
    let text = errors_of(&page(
        "    state on = true\n    Card(class: { md: on }) { Text(\"x\") }",
    ));
    assert!(text.is_empty(), "{text}");
}

// ─── §6d: the engine's own classes ───────────────────────────────────

#[test]
fn naming_an_engine_class_draws_a_warning() {
    let w = warnings_of(
        &page(
            "    state on = true\n    Card(class: [\"mine\", { \"wf-btn\": on }]) { Text(\"x\") }",
        ),
        "V04",
    );
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("`wf-btn`"), "{w:?}");
    assert!(warnings_of(&page("    Card(class: \"feature\") { Text(\"x\") }"), "V04").is_empty());
}

// ─── §3: the project's scripts reach every page ──────────────────────

/// A project on disk with `files` (path under the project, contents), built
/// by the real binary. What it printed, and where it was built.
fn build(name: &str, config: &str, files: &[(&str, &str)]) -> (String, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("wf-4-2-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d", dir.to_str().unwrap()])
        .output()
        .expect("wf runs");
    let said =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    (said, dir)
}

const HOME: &str =
    "page Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(\"H\").h1\n}\n";

#[test]
fn a_script_under_src_is_copied_as_written_and_linked_before_the_page_code() {
    let tilt = "/** Tilt. */\nfunction initTilt(node) {\n  node.dataset.tilt = \"on\"\n}\n";
    for ssg in [true, false] {
        let (said, dir) = build(
            if ssg { "scripts-ssg" } else { "scripts-spa" },
            &format!(r#"{{ "name": "s", "build": {{ "ssg": {ssg}, "csp": true }} }}"#),
            &[
                ("src/App.wf", HOME),
                ("src/tilt.js", tilt),
                ("src/lib/a.js", "var a = 1\n"),
            ],
        );
        assert!(said.contains("Build complete"), "{said}");
        assert_eq!(
            std::fs::read_to_string(dir.join("build/js/tilt.js")).unwrap(),
            tilt,
            "byte for byte"
        );
        let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("no {needle} in {html}"))
        };
        // Path order, both before the compiled code, so their names exist
        // when it runs.
        assert!(
            at("js/lib/a.js\" defer") < at("js/tilt.js\" defer"),
            "{html}"
        );
        assert!(at("js/tilt.js\" defer") < at("app.js\" defer"), "{html}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn an_es_module_under_src_stops_the_build() {
    let (said, dir) = build(
        "scripts-module",
        r#"{ "name": "s" }"#,
        &[
            ("src/App.wf", HOME),
            ("src/m.js", "const x = 1\nexport function f() {}\n"),
        ],
    );
    assert!(!said.contains("Build complete"), "{said}");
    assert!(
        said.contains("`src/m.js` is an ES module") && said.contains("src/m.js:2:1"),
        "{said}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_script_the_compiler_cannot_read_is_linked_with_a_warning() {
    let (said, dir) = build(
        "scripts-unreadable",
        r#"{ "name": "s" }"#,
        &[
            ("src/App.wf", HOME),
            ("src/b.js", "function ok() {}\nconst s = \"open\n"),
        ],
    );
    assert!(said.contains("Build complete"), "{said}");
    assert!(
        said.contains("Warning: a string is never closed") && said.contains("src/b.js:2:11"),
        "{said}"
    );
    assert!(
        std::fs::read_to_string(dir.join("build/index.html"))
            .unwrap()
            .contains("js/b.js")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_public_file_at_a_scripts_address_is_refused() {
    let (said, dir) = build(
        "scripts-public",
        r#"{ "name": "s" }"#,
        &[
            ("src/App.wf", HOME),
            ("src/t.js", "var t = 1\n"),
            ("public/js/t.js", "var other = 2\n"),
        ],
    );
    assert!(!said.contains("Build complete"), "{said}");
    assert!(said.contains("both claim `/js/t.js`"), "{said}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── §1–2: what a script declares is in scope, typed by its JSDoc ─────

const MONEY: &str = "/**\n * Money, in dollars.\n * @param {number} n\n * @param {string} [currency]\n * @returns {string}\n */\nconst money = (n, currency) => \"$\" + n.toFixed(2)\n\nclass Counter {\n  constructor(start) { this.n = start }\n  next() { return ++this.n }\n}\n";

fn page_using(body: &str) -> String {
    format!(
        "page Home(path: \"/\", title: \"H\", description: \"D\") {{\n    Heading(\"H\").h1\n{body}\n}}\n"
    )
}

#[test]
fn a_scripts_names_are_in_scope_and_a_class_is_constructed() {
    let (said, dir) = build(
        "scope",
        r#"{ "name": "s", "build": { "split": false, "minify": false } }"#,
        &[
            (
                "src/App.wf",
                &page_using(
                    "    state shown = \"\"\n    Text(money(12))\n    Button(\"Go\") { on click { shown = \"{Counter(5).next()}\" } }\n    Text(shown)",
                ),
            ),
            ("src/money.js", MONEY),
        ],
    );
    assert!(
        said.contains("Build complete") && !said.contains("T13"),
        "{said}"
    );
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(
        js.contains("money(12)"),
        "a script's function is called as itself"
    );
    assert!(
        js.contains("new Counter(5)"),
        "the language has no `new`; a class is constructed"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_call_is_held_to_the_scripts_parameters_and_its_doc_comment() {
    let (said, dir) = build(
        "calls",
        r#"{ "name": "s" }"#,
        &[
            (
                "src/App.wf",
                &page_using(
                    "    Text(money(1, \"EUR\"))\n    Text(money(1, 2, 3))\n    Text(money(\"12\"))\n    state total: Number = money(1)\n    Text(\"{total}\")",
                ),
            ),
            ("src/money.js", MONEY),
        ],
    );
    assert!(
        said.contains("[T10] `money` takes 1 to 2 arguments, but 3 are given"),
        "{said}"
    );
    assert!(
        said.contains("It is `money(n, currency?)`, at src/money.js:7:7"),
        "{said}"
    );
    assert!(
        said.contains("[T01] `n` of `money` is `Number`, but `\"12\"` is `String`"),
        "{said}"
    );
    assert!(
        said.contains("[T01] `total` is `String`, but `Number` is wanted"),
        "what it returns flows: {said}"
    );
    assert!(
        !said.contains("money(1, \"EUR\")"),
        "an optional parameter may be given: {said}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_script_name_must_mean_one_thing() {
    let (said, dir) = build(
        "clashes",
        r#"{ "name": "s" }"#,
        &[
            (
                "src/App.wf",
                &format!(
                    "{}component Card2 {{ Text(\"x\") }}\n",
                    page_using("    Card2")
                ),
            ),
            ("src/a.js", "function shared() {}\nfunction format() {}\n"),
            ("src/b.js", "var shared = 1\nconst Card2 = 2\n"),
        ],
    );
    assert!(
        said.contains("`shared` is declared by both `src/a.js` and `src/b.js`"),
        "{said}"
    );
    assert!(
        said.contains("`format` in `src/a.js` is a name the language already has"),
        "{said}"
    );
    assert!(
        said.contains("`Card2` in `src/b.js` is also a component the program declares"),
        "{said}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_jsdoc_type_reads_as_the_language_reads_it() {
    use webfluent::sema::types::jsdoc_type;
    let cases = [
        ("number", "Number"),
        ("?string", "String?"),
        ("string | null", "String?"),
        ("string | number", "Any"),
        ("number[]", "[Number]"),
        ("Array.<boolean>", "[Bool]"),
        ("Promise<string>", "String"),
        ("Object<string, number>", "Map"),
        ("HTMLElement", "Any"),
        ("...string", "String"),
    ];
    for (written, means) in cases {
        assert_eq!(jsdoc_type(written).to_string(), means, "{written}");
    }
    assert!(matches!(
        jsdoc_type("(v: boolean) => void"),
        webfluent::sema::types::Type::Func(..)
    ));
    assert!(matches!(
        jsdoc_type("function(string): number"),
        webfluent::sema::types::Type::Func(..)
    ));
    assert!(matches!(
        jsdoc_type("{ max?: number, destroy(): void }"),
        webfluent::sema::types::Type::Shape(..)
    ));
}

// ─── §4: `mount:`, `update:`, `cleanup:` on any element ──────────────

#[test]
fn any_element_takes_a_lifetime() {
    let src = page(
        "    state max = 1\n    Card(class: \"tilt\", mount: (n) => n, update: (h) => max, cleanup: (h) => h) { Text(\"x\") }",
    );
    let js = spa_generated(&src);
    assert!(js.contains("WF.attach(_e"), "{js}");
    assert!(
        !js.contains("mount:") && !js.contains("\"mount\""),
        "never an attribute: {js}"
    );
    let html = ssg_html(&src);
    assert!(
        !html.contains("mount=") && !html.contains("cleanup="),
        "{html}"
    );
    assert!(errors_of(&src).is_empty(), "{}", errors_of(&src));
}

#[test]
fn a_lifetime_on_a_component_call_goes_to_its_root_unless_it_takes_one() {
    let js = spa_generated(&format!(
        "component Chip(_ text: String) {{ Badge(text) }}\ncomponent Own(mount: Any = null) {{ Text(\"o\") }}\n{}",
        page("    Chip(\"a\", mount: (n) => n, cleanup: (h) => h)\n    Own(mount: (n) => n)")
    ));
    let chip = js.find("= Component_Chip(").expect("Chip is called");
    let own = js.find("= Component_Own(").expect("Own is called");
    let between = &js[chip..own];
    assert!(
        between.contains("WF.attach("),
        "Chip's root gets the lifetime: {between}"
    );
    assert!(
        !js[chip..].lines().next().unwrap().contains("mount"),
        "not passed to Chip as a prop"
    );
    assert!(
        js[own..].lines().next().unwrap().contains("mount"),
        "Own declares `mount` and gets it"
    );
}
