//! `Portal(into: node) { … }`: a block drawn into a node somebody else's
//! code owns. What the runtime does with it is `tests/js/portal.test.mjs`;
//! what a real browser does, `tests/browser/portal.mjs`.

mod common;

use common::{page, spa_generated, ssg_html, template_html};
use webfluent::diagnostics::Diagnostic;

fn findings(src: &str) -> Vec<Diagnostic> {
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
}

fn with(src: &str, code: &str) -> Vec<Diagnostic> {
    findings(src)
        .into_iter()
        .filter(|d| d.code == code)
        .collect()
}

/// The variable the first portal's box is held in: `_e3`.
#[track_caller]
fn the_box(js: &str) -> String {
    let end = at(js, " = WF.el(\"div\", { className: \"wf-portal\"");
    let start = js[..end].rfind("const ").expect("a declaration") + "const ".len();
    js[start..end].to_string()
}

/// Where `needle` is in `js`, for saying what comes before what.
#[track_caller]
fn at(js: &str, needle: &str) -> usize {
    js.find(needle)
        .unwrap_or_else(|| panic!("`{needle}` is not in:\n{js}"))
}

#[test]
fn a_portal_is_a_box_a_marker_and_its_block_drawn_after_the_box_is_placed() {
    let js = spa_generated(&page(
        "state n = 1\n    Stack(ref: spot)\n    Portal(into: spot, id: \"tip\", data-popup: \"row\", aria-live: \"polite\", data-n: n) {\n        Text(\"inside\")\n    }",
    ));
    // The box: a `div` of its own class, with the attributes it was given —
    // `into:` is where it goes, not one of them.
    assert!(
        js.contains("WF.el(\"div\", { className: \"wf-portal\", id: \"tip\", \"data-popup\": \"row\", \"aria-live\": \"polite\", \"data-n\": () => _n() })"),
        "{js}"
    );
    assert!(!js.contains("into:"), "{js}");
    // Where it is written, a marker: a branch or an item that holds only
    // a portal still has a node of its own.
    let marker = at(&js, ".appendChild(document.createComment(\"wf-portal\"));");
    // The box is placed before its block is drawn, so the block is built
    // into a node that is already in the page when the target is.
    let placed = at(&js, "= WF.portal(_e");
    let drawn = at(&js, "WF.el(\"p\", { className: \"wf-text\" }, \"inside\")");
    assert!(marker < placed && placed < drawn, "{js}");
    let b = the_box(&js);
    assert!(js.contains(&format!("WF.portal({b}, () => spot);")), "{js}");
    // The block goes into the box, in the scope the portal is written in,
    // and the lifetimes it asked for are let go once it is whole. It is
    // written at the portal's own level, never in a block of its own, so
    // what it declares is read after it as a `Stack`'s is.
    let child = at(&js, &format!("{b}.appendChild(_e"));
    assert!(drawn < child, "{js}");
    let release = at(&js, &format!("\n  {b}_drawn();\n"));
    assert!(child < release, "{js}");
    assert!(!js.contains("try {") && !js.contains("finally"), "{js}");
    // The runtime ships what it calls.
    let kept = webfluent::runtime::modules_for(&js);
    assert!(kept.contains("host"), "{kept:?}");
}

#[test]
fn a_portal_reads_where_it_goes_again_and_its_block_belongs_to_where_it_is_written() {
    let src = "component Chip(_ label: String) {\n    event picked(label: String)\n    Button(label) { on click { emit picked(label) } }\n}\n".to_string()
        + &page(
            "    state rows = [{ id: \"a\", name: \"alpha\" }]\n    state chosen = \"\"\n    for r, i in rows by i {\n        Portal(into: document.getElementById(r.id)) {\n            Chip(r.name) { on picked(n) { chosen = n } }\n        }\n    }\n    Text(chosen)",
        );
    let js = spa_generated(&src);
    // Inside a keyed `for`, the item is read through its accessor: when
    // the item for a key changes, where the portal goes is read again.
    let b = the_box(&js);
    assert!(
        js.contains(&format!(
            "WF.portal({b}, () => document.getElementById($r().id));"
        )),
        "{js}"
    );
    // An event the block's component fires is heard where the portal is
    // written: the page's state, not the box's.
    assert!(
        js.contains("on: { picked: (n) => { _chosen.set(n); } }"),
        "{js}"
    );
    assert!(
        findings(&src)
            .iter()
            .all(|d| d.severity != webfluent::diagnostics::Severity::Error)
    );
}

#[test]
fn a_portal_with_handlers_of_its_own_hears_what_its_block_does() {
    let js = spa_generated(&page(
        "state clicks = 0\n    Portal(into: null) {\n        on click { clicks = clicks + 1 }\n        Text(\"in\")\n    }\n    Text(\"{clicks}\")",
    ));
    let b = the_box(&js);
    let release = at(&js, &format!("{b}_drawn();"));
    let listener = at(&js, &format!("{b}.addEventListener(\"click\""));
    assert!(release < listener, "{js}");
}

#[test]
fn a_portal_without_into_is_placed_without_what_it_needs() {
    let src = page("    Portal { Text(\"lost\") }");
    let found = with(&src, "C01");
    assert_eq!(found.len(), 1, "{found:?}");
    let d = &found[0];
    assert_eq!(
        d.message,
        "`Portal` is placed without `into:`, which it needs"
    );
    assert!(
        d.hint.as_deref().unwrap_or("").contains("`ref:` handle"),
        "{d:?}"
    );
    // The fix passes it, with the value that keeps the block aside.
    let fix = d.fixes.first().expect("a fix");
    assert_eq!(fix.title, "Pass `into:`");
    // Given, it is not reported; given `null`, neither.
    for given in [
        "Portal(into: null) { Text(\"x\") }",
        "Stack(ref: s)\n    Portal(into: s) { Text(\"x\") }",
    ] {
        assert!(
            with(&page(&format!("    {given}")), "C01").is_empty(),
            "{given}"
        );
    }
}

#[test]
fn the_fix_for_a_portal_without_into_passes_it() {
    let src = page("    Portal { Text(\"lost\") }");
    let d = with(&src, "C01").remove(0);
    let fixed = webfluent::diagnostics::fixes::apply(&src, &d.fixes[0]);
    assert!(
        fixed.contains("Portal(into: null) { Text(\"lost\") }"),
        "{fixed}"
    );
    assert!(with(&fixed, "C01").is_empty(), "{fixed}");
}

#[test]
fn a_component_named_portal_is_never_drawn_and_says_so() {
    let src = "component Portal {\n    Text(\"mine\")\n}\n".to_string() + &page("    Text(\"x\")");
    let found = with(&src, "V11");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0]
            .message
            .contains("`Portal` is a built-in component"),
        "{found:?}"
    );
}

#[test]
fn the_paint_and_the_template_engine_write_only_where_a_portal_is() {
    let src = page(
        "    Text(\"before\")\n    Portal(into: null) { Text(\"kept for the live page\") }\n    Text(\"after\")",
    );
    for (what, html) in [
        ("the paint", ssg_html(&src)),
        ("a template", template_html(&src)),
    ] {
        assert!(html.contains("<!--wf-portal-->"), "{what}: {html}");
        assert!(!html.contains("kept for the live page"), "{what}: {html}");
        assert!(!html.contains("wf-portal\""), "{what} draws no box: {html}");
        assert!(
            html.find("before").unwrap() < html.find("<!--wf-portal-->").unwrap()
                && html.find("<!--wf-portal-->").unwrap() < html.find("after").unwrap(),
            "{what}: {html}"
        );
    }
}

#[test]
fn the_registry_describes_portal_and_its_required_into() {
    let sig = webfluent::registry::component("Portal").expect("Portal is a built-in");
    let into = sig.props.iter().find(|p| p.name == "into").expect("into:");
    assert!(into.required);
    let described = webfluent::registry::schema::builtin("Portal").unwrap();
    let into = described.props.iter().find(|p| p.name == "into").unwrap();
    assert!(into.required, "{into:?}");
    // And `wf registry --json`, which editors read.
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["registry", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let portal = json["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Portal")
        .expect("Portal in wf registry --json");
    assert_eq!(portal["children"], "elements");
    assert_eq!(portal["class"], "wf-portal");
    let into = portal["props"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "into")
        .unwrap();
    assert_eq!(into["required"], true, "{into}");
}

#[test]
fn the_box_adds_nothing_to_the_layout_where_it_lands() {
    for css in [
        webfluent::themes::component_css(),
        webfluent::themes::structural::structural_css(),
    ] {
        assert!(css.contains(".wf-portal { display: contents; }"));
    }
    // And the rule ships only to a program that draws one.
    let used =
        webfluent::syntax::parse_source(&page("    Portal(into: null) { Text(\"x\") }"), "t.wf")
            .unwrap();
    let unused = webfluent::syntax::parse_source(&page("    Text(\"x\")"), "t.wf").unwrap();
    let prune = |p| {
        webfluent::themes::prune::prune_css(
            webfluent::themes::component_css(),
            &webfluent::themes::prune::Usage::of(p),
        )
    };
    assert!(prune(&used).contains(".wf-portal"));
    assert!(!prune(&unused).contains(".wf-portal"));
}

#[test]
fn a_pdf_refuses_a_portal_as_it_refuses_a_host() {
    assert_eq!(
        webfluent::linter::pdf_validation::drawn_in(webfluent::config::OutputType::Pdf, "Portal"),
        webfluent::linter::pdf_validation::drawn_in(webfluent::config::OutputType::Pdf, "Host"),
    );
}

#[test]
fn a_test_that_looks_at_a_portal_is_run_in_a_browser() {
    let program = webfluent::syntax::parse_source(
        "component Tip {\n    Portal(into: null) { Text(\"tip\") }\n}\ncomponent Plain {\n    Text(\"plain\")\n}\ntest \"direct\" {\n    Stack(ref: s)\n    Portal(into: s) { Text(\"in\") }\n    expect \"in\"\n}\ntest \"in a branch\" {\n    state open = true\n    if open { Portal(into: null) { Text(\"in\") } }\n    expect \"x\"\n}\ntest \"through a component\" {\n    Tip\n    expect \"x\"\n}\ntest \"none\" {\n    Plain\n    expect \"plain\"\n}\n",
        "t.wf",
    )
    .unwrap();
    let tests: Vec<&webfluent::parser::ast::TestDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            webfluent::parser::ast::Declaration::Test(t) => Some(t),
            _ => None,
        })
        .collect();
    let in_browser: Vec<bool> = tests
        .iter()
        .map(|t| webfluent::testing::runs_in_browser(t, &program.declarations))
        .collect();
    assert_eq!(in_browser, [true, true, true, false]);
    assert!(tests.iter().all(|t| !t.acts()), "none of them acts");
}

// ─── A script's element as the target ───────────────────────

fn build(name: &str, files: &[(&str, &str)]) -> (String, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("wf-portal-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "p", "build": { "split": false, "minify": false } }"#,
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d", dir.to_str().unwrap()])
        .output()
        .expect("wf runs");
    let said =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    (said, dir)
}

const EDITOR_JS: &str = "/**\n * The element the editor made for a popup, or null when it has none.\n * @param {string} id\n * @returns {HTMLElement|null}\n */\nfunction editorHost(id) {\n  return document.getElementById(\"host-\" + id);\n}\n";

#[test]
fn a_scripts_element_may_be_where_a_portal_draws() {
    let (said, dir) = build(
        "script",
        &[
            ("src/editor.js", EDITOR_JS),
            (
                "src/App.wf",
                "store Popups {\n    state rows = [{ id: \"a\", name: \"alpha\" }]\n    action again() { rows = [{ id: \"b\", name: \"beta\" }] }\n}\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(\"H\").h1\n    Button(\"Again\") { on click { Popups.again() } }\n    for r, i in Popups.rows by i {\n        Portal(into: editorHost(r.id), data-popup: \"row\") { Text(r.name) }\n    }\n}\n",
            ),
        ],
    );
    assert!(
        said.contains("Build complete") && !said.contains("error"),
        "an element or null from a script is somewhere to draw: {said}"
    );
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(
        js.contains("WF.portal(_e") && js.contains(", () => editorHost($r().id));"),
        "read again when the item for its key changes: {js}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
