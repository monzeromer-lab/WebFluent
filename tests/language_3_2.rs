//! The component and reactivity additions of 3.2, held to every backend.

mod common;

use common::{Backend, raw_output};
use serde_json::json;
use webfluent::Template;
use webfluent::codegen::{dark_css, scoped_css::scoped_rules};
use webfluent::config::project::ThemeConfig;
use webfluent::parse_source;
use webfluent::themes::{resolve_dark_tokens, resolve_tokens};

fn page(body: &str) -> String {
    format!("page P(path: \"/\", title: \"T\") {{\n{body}\n}}\n")
}

fn painted(src: &str) -> String {
    raw_output(Backend::Ssg, src)
}

fn templated(src: &str, data: serde_json::Value) -> String {
    Template::from_str(src)
        .expect("template parses")
        .render_html_fragment(&data)
        .expect("renders")
}

#[test]
fn a_scoped_slot_hands_the_fill_its_values() {
    let list = "component Rows(items: [Any]) {\n slot row(item: Any, index: Number)\n slot empty\n if items.length == 0 { empty } else { for it, i in items by it { row(item: it, index: i) } }\n}\n";
    let src = format!(
        "{list}{}",
        page(
            "state names = [\"ann\", \"bob\"]\n  Rows(items: names) { row(n, i) { Text(\"{i}: {n}\") }  empty { Text(\"none\") } }\n  Rows(items: []) { row(n) { Text(n) }  empty { Text(\"none\") } }"
        )
    );
    let html = painted(&src);
    assert!(
        html.contains(">0: ann<") && html.contains(">1: bob<"),
        "the paint: {html}"
    );
    assert!(html.contains(">none<"), "the other slot: {html}");
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("WF.slot(") && js.contains("({ item: it, index: i })"),
        "{js}"
    );
    assert!(
        js.contains("row: (_s) => {")
            && js.contains("const n = _s.item;")
            && js.contains("const i = _s.index;"),
        "{js}"
    );
    let html = templated(&src, json!({ "names": ["ann", "bob"] }));
    assert!(
        html.contains(">0: ann<") && html.contains(">1: bob<"),
        "template: {html}"
    );
    assert!(html.contains(">none<"), "template: {html}");
}

#[test]
fn a_component_declares_parts_that_are_called_under_its_name() {
    let panel = "component Panel(_ title: String) {\n part Header(_ text: String) { Heading(text) }\n part Footer { Text(\"end\") }\n Card { Text(title)  children }\n}\n";
    let src = format!(
        "{panel}{}",
        page("Panel(\"p\") { Panel.Header(\"h\")  Text(\"body\")  Panel.Footer }")
    );
    let html = painted(&src);
    assert!(html.contains(">h</h2>"), "the part's paint: {html}");
    assert!(html.contains(">end<") && html.contains(">body<"), "{html}");
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("function Component_Panel_Header(_p, _slots)"),
        "{js}"
    );
    assert!(
        js.contains("Component_Panel_Header({ text: \"h\" })")
            || js.contains("Component_Panel_Header({ text: () => \"h\" })"),
        "{js}"
    );
    let html = templated(&src, json!({}));
    assert!(
        html.contains(">h</h2>") && html.contains(">end<"),
        "template: {html}"
    );
}

#[test]
fn refs_timers_cleanup_and_key_handlers_compile_to_the_runtime() {
    let src = page(
        "state draft = \"\"\n state n = 0\n Input(bind: draft, ref: nameInput)\n Button(\"Focus\") { on click { nameInput.focus() } }\n Container { on key(\"ctrl+k\", e) { nameInput.focus() } }\n on key(\"Escape\") { draft = \"\" }\n every(1000) { n = n + 1 }\n after(5000) { draft = \"late\" }\n effect { log(draft)  cleanup { log(\"bye\") } }",
    );
    let js = raw_output(Backend::Spa, &src);
    for expected in [
        "const nameInput = WF.ref();",
        "ref: nameInput",
        "nameInput.focus();",
        "addEventListener(\"keydown\", (e) => { if (WF.keyIs(e, \"ctrl+k\")) {",
        "WF.onKey(document, \"Escape\", (event) => {",
        "WF.every(1000, () => {",
        "WF.after(5000, () => {",
        "return () => {",
        "console.log(\"bye\");",
    ] {
        assert!(js.contains(expected), "{expected} in {js}");
    }
    // The static paint ignores what only a live page does.
    let html = painted(&src);
    assert!(
        html.contains("wf-input") && !html.contains("nameInput"),
        "{html}"
    );
}

#[test]
fn persisted_state_and_the_browsers_values() {
    let src = format!(
        "store Settings {{ persist theme = \"light\"  state n = 0 }}\n{}",
        page(
            "use Settings\n persist draft = \"\"\n state wide = false\n Text(if viewport.md { \"wide\" } else { \"narrow\" })\n Text(query.tab ?? \"none\")\n Text(hash)\n Text(Settings.theme)\n Input(bind: draft)"
        )
    );
    let js = raw_output(Backend::Spa, &src);
    for expected in [
        "WF.persist(\"P.draft\", \"\")",
        "persist: { prefix: \"Settings\", names: [\"theme\"] },",
        "WF.viewport().md",
        "WF.query().tab",
        "WF.hash()",
    ] {
        assert!(js.contains(expected), "{expected} in {js}");
    }
    // A declared name is the writer's own.
    let own = raw_output(
        Backend::Spa,
        &page("state query = \"\"\n Input(bind: query)\n Text(query)"),
    );
    assert!(
        own.contains("_query()") && !own.contains("WF.query()"),
        "{own}"
    );
    // The static paint keeps the initial value and leaves the browser's
    // values to the live page.
    let html = painted(&src);
    assert!(html.contains(">light<"), "{html}");
    assert!(
        !html.contains(">wide<") && !html.contains(">narrow<"),
        "{html}"
    );
}

#[test]
fn a_declared_animation_is_keyframes_a_class_and_a_case_of_animate() {
    let src = format!(
        "animation Pulse {{\n from {{ opacity: 1 }}\n 50% {{ opacity: 0.4; transform: scale(0.98) }}\n to {{ opacity: 1 }}\n}}\n{}",
        page(
            "Card(animate: .Pulse) { style { animation: Pulse 2s infinite\n @container (min-width: 400px) { padding: 2rem } }\n Text(\"x\") }"
        )
    );
    let program = parse_source(&src, "t.wf").expect("parses");
    let css = scoped_rules(&program);
    assert!(css.contains("@keyframes Pulse { from { opacity: 1; } 50% { opacity: 0.4; transform: scale(0.98); } to { opacity: 1; } }"), "{css}");
    assert!(css.contains(".wf-animate-Pulse { animation: Pulse var(--animation-duration-normal) var(--animation-easing-default) both; }"), "{css}");
    assert!(
        css.contains("@container (min-width: 400px) {"),
        "a container query passes through: {css}"
    );
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("\"Pulse\""),
        "the case reaches the runtime: {js}"
    );
    let html = painted(&src);
    assert!(html.contains("wf-card"), "{html}");
}

#[test]
fn a_dark_theme_overrides_the_tokens_under_the_readers_preference_or_choice() {
    let src = format!(
        "theme Light {{ color-background: #FFFFFF }}\ntheme Dark {{ color-background: #0B1220\n color-text: #E5E7EB }}\n{}",
        page("Button(\"Dark\") { on click { setTheme(\"dark\") } }\n Text(theme)")
    );
    let program = parse_source(&src, "t.wf").expect("parses");
    let config = ThemeConfig {
        name: None,
        tokens: Default::default(),
        builtin: Default::default(),
        dark: Some("Dark".to_string()),
    };
    // With the dark theme named, the other is the build's own.
    let light = resolve_tokens(&program, &config).expect("resolves");
    assert_eq!(
        light.get("color-background").map(String::as_str),
        Some("#FFFFFF")
    );
    let dark = resolve_dark_tokens(&program, &config)
        .expect("resolves")
        .expect("named");
    let css = dark_css(&dark);
    assert!(css.contains("@media (prefers-color-scheme: dark) {\n:root:not([data-theme=\"light\"]) {\n  --color-background: #0B1220;\n  --color-text: #E5E7EB;\n}\n}"), "{css}");
    assert!(
        css.contains(":root[data-theme=\"dark\"] {\n  --color-background: #0B1220;"),
        "{css}"
    );
    let missing = ThemeConfig {
        dark: Some("Night".to_string()),
        ..config
    };
    let err = resolve_dark_tokens(&program, &missing)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(
        err.contains("No `theme Night` is declared for `theme.dark`"),
        "{err}"
    );
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("WF.setTheme(\"dark\")") && js.contains("WF.theme()"),
        "{js}"
    );
}

#[test]
fn an_enum_prop_is_written_to_the_components_root_as_a_data_attribute() {
    let src = format!(
        "enum Tone {{ calm, loud }}\nenum Status {{ idle, failed(reason: String) }}\ncomponent Chip(_ label: String, tone: Tone = .calm, status: Status = .idle, count: Number = 0) {{ Badge(label) }}\n{}",
        page("Chip(\"a\")\n Chip(\"b\", tone: .loud, status: .failed(\"x\"))")
    );
    let html = painted(&src);
    assert!(
        html.contains("data-tone=\"calm\"") && html.contains("data-status=\"idle\""),
        "the defaults: {html}"
    );
    assert!(
        html.contains("data-tone=\"loud\"") && html.contains("data-status=\"failed\""),
        "the case, its payload aside: {html}"
    );
    assert!(
        !html.contains("data-count"),
        "a number is not a case: {html}"
    );
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("\"data-tone\": () => _p.tone"), "{js}");
    assert!(
        js.contains("\"data-status\": () => WF.caseOf(_p.status)"),
        "{js}"
    );
    let html = templated(&src, json!({}));
    assert!(
        html.contains("data-tone=\"loud\"") && html.contains("data-status=\"failed\""),
        "template: {html}"
    );
}

#[test]
fn a_form_handle_and_a_pending_action() {
    let src = format!(
        "store Api {{ state n = 0\n action sync() {{ let r = await fetch(\"/x\")  n = n + 1 }} }}\n{}",
        page(
            "use Api\n state saved = false\n action save() { let r = await fetch(\"/save\")  saved = true }\n Form(bind: signup) { on submit { save() }\n Input(name: \"email\", type: \"email\", required: true)\n Button(\"Save\", disabled: !signup.valid || save.pending)\n Button(\"Sync\", disabled: Api.sync.pending) { on click { Api.sync() } }\n Button(\"Clear\") { on click { signup.reset() } } }\n Text(signup.values.email ?? \"\")"
        )
    );
    let js = raw_output(Backend::Spa, &src);
    for expected in [
        "const signup = WF.form();",
        "ref: signup",
        "const _save_pending = WF.signal(false);",
        "_save_pending.set(true);",
        "} finally { _save_pending.set(false); }",
        "_save_pending()",
        "Api.sync.pending()",
        "signup.valid",
        "signup.reset()",
        "signup.values.email",
    ] {
        assert!(js.contains(expected), "{expected} in {js}");
    }
    assert!(
        !js.contains("const signup = WF.ref();"),
        "a form handle is declared once, as a form: {js}"
    );
    let html = painted(&src);
    assert!(
        html.contains("<form") && html.contains("wf-input"),
        "{html}"
    );
}

#[test]
fn a_page_declares_its_own_head_tags() {
    let src = page(
        "state img = \"/og.png\"\n head {\n meta(property: \"og:image\", content: img)\n link(rel: \"canonical\", href: \"https://x.y/\")\n script(src: \"/analytics.js\", defer: true)\n }\n Text(\"x\")",
    );
    let html = painted(&src);
    assert!(
        html.contains("<meta data-wf-head property=\"og:image\" content=\"/og.png\">"),
        "{html}"
    );
    assert!(
        html.contains("<link data-wf-head rel=\"canonical\" href=\"https://x.y/\">"),
        "{html}"
    );
    assert!(
        html.contains("<script data-wf-head src=\"/analytics.js\" defer></script>"),
        "{html}"
    );
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("WF.head([[\"meta\", { \"property\": \"og:image\", \"content\": () => _img() }], [\"link\", { \"rel\": \"canonical\", \"href\": \"https://x.y/\" }], [\"script\", { \"src\": \"/analytics.js\", \"defer\": true }]]);"), "{js}");
}

#[test]
fn an_if_let_binding_is_read_by_the_static_paint_and_the_template_engine() {
    let src = page(
        "state user: Map? = { name: \"Ann\" }\n  state none: Map? = null\n  if let u = user { Text(\"Hi {u.name}\") } else { Text(\"nobody\") }\n  if let n = none { Text(\"Hi {n.name}\") } else { Text(\"nobody\") }",
    );
    let html = painted(&src);
    assert!(
        html.contains(">Hi Ann<") && html.contains(">nobody<"),
        "{html}"
    );
    let tpl = page("if let u = user { Text(\"Hi {u.name}\") } else { Text(\"nobody\") }");
    let html = templated(&tpl, json!({ "user": { "name": "Bob" } }));
    assert!(html.contains(">Hi Bob<"), "template: {html}");
    let html = templated(&tpl, json!({ "user": null }));
    assert!(html.contains(">nobody<"), "template: {html}");
}

#[test]
fn a_breadcrumb_item_without_a_destination_is_a_span_in_every_backend() {
    let src = page(
        "Breadcrumb { Breadcrumb.Item(to: \"/docs\") { Text(\"Docs\") }  Breadcrumb.Item { Text(\"Here\") } }",
    );
    let html = painted(&src);
    assert!(
        html.contains("<a class=\"wf-breadcrumb__item\" href=\"/docs\""),
        "{html}"
    );
    assert!(
        html.contains("<span class=\"wf-breadcrumb__item\""),
        "{html}"
    );
    assert!(
        !html.contains("<li "),
        "no list item outside a list: {html}"
    );
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains("WF.el(\"span\", { className: \"wf-breadcrumb__item\""),
        "{js}"
    );
}

#[test]
fn a_code_with_a_language_is_coloured_by_every_backend() {
    let src = page(
        "state lang = \"wf\"\n  Code(\"page P(path: \\\"/\\\") { state n = 1 }\", language: \"wf\").block\n  Code(\"{ \\\"a\\\": 1 }\", language: lang).block\n  Markdown(\"```json\\n{ \\\"k\\\": true }\\n```\")",
    );
    let html = painted(&src);
    for expected in [
        "<span class=\"wf-tok-kw\">page</span> <span class=\"wf-tok-name\">P</span>(<span class=\"wf-tok-prop\">path</span>: <span class=\"wf-tok-str\">&quot;/&quot;</span>) { <span class=\"wf-tok-kw\">state</span> n = <span class=\"wf-tok-num\">1</span> }",
        "{ <span class=\"wf-tok-str\">&quot;a&quot;</span>: <span class=\"wf-tok-num\">1</span> }",
        "<code class=\"language-json\">{ <span class=\"wf-tok-prop\">&quot;k&quot;</span>: <span class=\"wf-tok-kw\">true</span> }",
    ] {
        assert!(html.contains(expected), "{expected}\n---\n{html}");
    }
    let js = raw_output(Backend::Spa, &src);
    assert!(
        js.contains(
            "highlight: { code: \"page P(path: \\\"/\\\") { state n = 1 }\", lang: \"wf\" }"
        ),
        "{js}"
    );
    assert!(
        js.contains("highlight: { code: \"{ \\\"a\\\": 1 }\", lang: () => _lang() }"),
        "{js}"
    );
    assert!(
        !js.contains("language: \"wf\" }"),
        "the language is not an attribute: {js}"
    );
    let html = templated(&src, json!({}));
    assert!(
        html.contains("<span class=\"wf-tok-kw\">page</span>"),
        "template: {html}"
    );
}
