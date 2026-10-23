//! `Markdown(text)` and `Markdown(text).inert`, through the compiler: what
//! the code generator hands the runtime, what the pre-render and the
//! template engine paint, and what the checks say of the flag.
//!
//! The two renderers themselves — `codegen::markdown` and the runtime's
//! `WF.markdown` — are held to one table of cases
//! (`tests/markdown-cases.json`) and to each other
//! (`tests/build_and_runtime_agree.rs`).

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::*;
use webfluent::diagnostics::Diagnostic;

fn body(lines: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{lines}\n}}\n"
    )
}

/// Every finding for `src`, through the build's pipeline.
fn findings(src: &str) -> Vec<Diagnostic> {
    let (program, mut out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
    out.extend(
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
        .diagnostics,
    );
    out
}

/// The `markdown:` attribute of each Markdown element, in order.
fn markdown_attrs(js: &str) -> Vec<String> {
    js.lines()
        .filter(|l| l.contains("className: \"wf-markdown\""))
        .map(|l| {
            let at = l.find("markdown: ").expect("a markdown attribute");
            let rest = &l[at..];
            rest[..rest.rfind(" });").unwrap_or(rest.len())].to_string()
        })
        .collect()
}

#[test]
fn the_flag_hands_the_runtime_the_inert_form_and_the_plain_form_is_unchanged() {
    let js = spa_generated(&body(
        "    state note = \"[a](/a)\"\n    state strict = true\n    Markdown(note)\n    Markdown(\"**x**\")\n    Markdown(note).inert\n    Markdown(\"x\", inert: true)\n    Markdown(note, inert: strict)\n    Markdown(note, inert: false)",
    ));
    assert_eq!(
        markdown_attrs(&js),
        [
            "markdown: () => _note()",
            "markdown: \"**x**\"",
            "markdown: {text: () => _note(), inert: true}",
            "markdown: {text: \"x\", inert: true}",
            "markdown: {text: () => _note(), inert: () => _strict()}",
            "markdown: () => _note()",
        ],
        "{js}"
    );
    // The flag is how the text is drawn, never HTML's `inert`, which would
    // take the element out of reach.
    assert!(
        !js.lines()
            .any(|l| l.contains("wf-markdown") && l.contains(", inert: true }")),
        "{js}"
    );
    assert!(!js.contains("\"inert\""), "{js}");
}

#[test]
fn the_pre_render_and_the_template_engine_draw_the_inert_form() {
    let src = body(
        "    Markdown(\"See [x](https://x.y) and ![p](/p.png) &lt;T&gt;\")\n    Markdown(\"See [x](https://x.y) and ![p](/p.png) &lt;T&gt;\").inert\n    Markdown(\"[run](javascript:alert(1))\")",
    );
    for (backend, html) in [("ssg", ssg_html(&src)), ("template", template_html(&src))] {
        assert!(
            html.contains("<p>See <a href=\"https://x.y\">x</a> and <img src=\"/p.png\" alt=\"p\"> &amp;lt;T&amp;gt;</p>"),
            "{backend}: {html}"
        );
        assert!(
            html.contains("<p>See x and &lt;T&gt;</p>"),
            "{backend}: {html}"
        );
        assert!(html.contains("<p>run)</p>"), "{backend}: {html}");
        assert!(!html.contains("javascript:"), "{backend}: {html}");
        assert!(!html.contains(" inert"), "{backend}: {html}");
    }
    // An `inert:` the build can work out is drawn as it says.
    let html = ssg_html(&body(
        "    state strict = false\n    Markdown(\"[a](/a)\", inert: strict)\n    Markdown(\"[b](/b)\", inert: !strict)",
    ));
    assert!(html.contains("<p><a href=\"/a\">a</a></p>"), "{html}");
    assert!(html.contains("<p>b</p>"), "{html}");
}

#[test]
fn the_pre_render_and_the_template_engine_draw_backslash_escapes() {
    // In a `.wf` string `\\` is one backslash and `\{` a brace.
    let src = body(concat!(
        r#"    Markdown("\\*not em\\* [a](\\/docs) `C:\\` a\\")"#,
        "\n",
        r#"    Markdown("&lt;'margin\\-top'&gt;\\\{1,4\\\} \\[x\\](/y)").inert"#,
    ));
    for (backend, html) in [("ssg", ssg_html(&src)), ("template", template_html(&src))] {
        assert!(
            html.contains(r#"<p>*not em* <a href="/docs">a</a> <code>C:\</code> a\</p>"#),
            "{backend}: {html}"
        );
        assert!(
            html.contains("<p>&lt;'margin-top'&gt;{1,4} [x](/y)</p>"),
            "{backend}: {html}"
        );
    }
}

#[test]
fn a_markdown_page_takes_backslash_escapes() {
    let dir = scratch("escapes");
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build", "ssg": true, "base_path": "/site" } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/notes.md"),
        "---\ntitle: Notes\ndescription: D\npath: /\n---\n\n# Notes\n\n\\# Not a heading, \\*not em\\*, C:\\Users\\me, [docs](\\/docs)\n\nA line that ends in a backslash\\\nbreaks, and the last one keeps it\\\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(
        html.contains(
            r#"<p># Not a heading, *not em*, C:\Users\me, <a href="/site/docs">docs</a></p>"#
        ),
        "{html}"
    );
    assert!(
        html.contains(
            "<p>A line that ends in a backslash<br>\nbreaks, and the last one keeps it\\</p>"
        ),
        "{html}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-markdown-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    dir
}

fn wf(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("wf runs");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

#[test]
fn a_built_site_takes_its_base_path_and_never_paints_a_link_a_browser_would_run() {
    let dir = scratch("site");
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build", "ssg": true, "base_path": "/site" } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        body(
            "    state docs = \"Takes a `List&lt;T&gt;` &mdash; see [the guide](/guide) ![p](/p.png)\"\n    Markdown(\"[guide](/guide) ![p](/p.png) [x](javascript:alert(1)) ![y](JavaScript:alert(1)) [z](//b.c/d)\")\n    Markdown(docs).inert",
        ),
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(
        html.contains("<p><a href=\"/site/guide\">guide</a> <img src=\"/site/p.png\" alt=\"p\"> x) y) <a href=\"//b.c/d\">z</a></p>"),
        "{html}"
    );
    assert!(
        html.contains("<p>Takes a <code>List&amp;lt;T&amp;gt;</code> — see the guide </p>"),
        "{html}"
    );
    assert!(!html.to_lowercase().contains("javascript:"), "{html}");
    let js = std::fs::read_to_string(dir.join("build/pages/P.js")).unwrap();
    assert!(js.contains("inert:!0") || js.contains("inert:true"), "{js}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_flag_is_one_markdown_has_and_a_misspelt_one_is_still_reported() {
    let clean: Vec<String> = findings(&body(
        "    state strict = true\n    Markdown(\"x\").inert\n    Markdown(\"x\", inert: true)\n    Markdown(\"x\", inert: strict)",
    ))
    .into_iter()
    .filter(|d| d.is_error() || d.code == "C04")
    .map(|d| d.to_string())
    .collect();
    assert!(clean.is_empty(), "{}", clean.join("\n"));

    let found = findings(&body("    Markdown(\"x\").inrt\n    Markdown(\"y\").bogus"));
    let e103: Vec<&Diagnostic> = found.iter().filter(|d| d.code == "E103").collect();
    assert_eq!(e103.len(), 2, "{found:?}");
    assert_eq!(e103[0].message, "Markdown has no flag or enum case `inrt`");
    assert!(
        e103[0]
            .hint
            .as_deref()
            .unwrap_or("")
            .contains("Did you mean `.inert`?"),
        "{:?}",
        e103[0].hint
    );
    assert_eq!(e103[1].message, "Markdown has no flag or enum case `bogus`");
    // Not on another element: it is Markdown's.
    let found = findings(&body("    Text(\"x\").inert"));
    assert!(
        found
            .iter()
            .any(|d| d.code == "E103" && d.message == "Text has no flag or enum case `inert`"),
        "{found:?}"
    );
}

#[test]
fn the_registry_lists_the_flag() {
    let sig = webfluent::registry::component("Markdown").expect("Markdown");
    assert!(matches!(
        sig.flag("inert"),
        webfluent::registry::Flag::Bool(p) if p.name == "inert"
    ));
    let registry = webfluent::registry::json::registry_json();
    let markdown = registry["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Markdown")
        .unwrap();
    assert!(
        markdown["flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"] == "inert" && f["prop"] == "inert"),
        "{markdown}"
    );
}
