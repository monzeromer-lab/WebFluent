//! What the compiler writes into a page is held to what the guide promises
//! about it: text is text, a value never becomes markup or an attribute of
//! its own, a URL a browser would run is never written, and the policy the
//! build ships stands before anything a page's data could put in the head.
//!
//! Each test builds a small project whose data carries the hostile value —
//! a `data` file, a route parameter, front matter, the config — and reads
//! back what the build wrote.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A project on disk, with the files given (relative paths).
fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-outsec-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    dir
}

/// `wf <args>` in `dir`: whether it succeeded, and everything it said.
fn wf(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), said)
}

fn build(dir: &Path) -> String {
    let (ok, said) = wf(dir, &["build"]);
    assert!(ok, "the build failed:\n{said}");
    said
}

fn read(dir: &Path, path: &str) -> String {
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

const STATIC_CSP: &str = r#"{"name":"t","build":{"minify":false,"ssg":true,"csp":true,"compress":false},"meta":{"site_url":"https://x.example"}}"#;
const STATIC: &str = r#"{"name":"t","build":{"minify":false,"ssg":true,"csp":false,"compress":false},"meta":{"site_url":"https://x.example"}}"#;
const SPA_CSP: &str =
    r#"{"name":"t","build":{"minify":false,"ssg":false,"csp":true,"compress":false}}"#;

/// Every `<tag` in `html` whatever its case, lower-cased.
fn tags_named(html: &str, name: &str) -> usize {
    html.to_ascii_lowercase()
        .matches(&format!("<{name}"))
        .count()
}

/// The line of `html` that follows the one holding `marker`.
fn line_after<'a>(html: &'a str, marker: &str) -> &'a str {
    let mut lines = html.lines();
    while let Some(line) = lines.next() {
        if line.contains(marker) {
            return lines.next().unwrap_or("");
        }
    }
    ""
}

// ─── S1: the page's <title>, and the policy before it ───────────────────

#[test]
fn a_title_from_data_is_text_in_every_case_it_is_written() {
    let posts = r#"[
        {"slug":"a","title":"</title><SCRIPT>document.write('pwned')</SCRIPT><title>"},
        {"slug":"b","title":"</TiTlE ><img src=x onerror=alert(1)> & co"},
        {"slug":"c","title":"<base href=\"https://evil.example/\">"}
    ]"#;
    let dir = project(
        "title",
        &[
            ("webfluent.app.json", STATIC_CSP),
            ("posts.json", posts),
            (
                "src/App.wf",
                "data posts = \"posts.json\"\n\
                 page Post(path: \"/p/:slug\", slug: String, title: \"{posts.find(p => p.slug == slug)?.title ?? slug} — Blog\", description: \"d\", paths: posts.map(p => p.slug)) {\n    Heading(\"x\").h1\n}\n\
                 page Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"Home\").h1 }\n",
            ),
        ],
    );
    build(&dir);
    for slug in ["a", "b", "c"] {
        let html = read(&dir, &format!("build/p/{slug}/index.html"));
        assert_eq!(
            tags_named(&html, "title"),
            1,
            "one title, and it is text:\n{html}"
        );
        assert_eq!(tags_named(&html, "img"), 0, "{html}");
        assert_eq!(tags_named(&html, "base"), 0, "{html}");
        // The bundle and the page chunk are the only scripts.
        assert!(!html.contains("<SCRIPT"), "{html}");
    }
    let a = read(&dir, "build/p/a/index.html");
    assert!(
        a.contains("<title>&lt;/title&gt;&lt;SCRIPT&gt;document.write('pwned')&lt;/SCRIPT&gt;&lt;title&gt; — Blog</title>"),
        "{a}"
    );
    let b = read(&dir, "build/p/b/index.html");
    assert!(b.contains("&amp; co — Blog</title>"), "{b}");
}

#[test]
fn a_title_from_a_route_parameter_and_from_front_matter_is_text() {
    let dir = project(
        "title-param",
        &[
            ("webfluent.app.json", STATIC),
            ("posts.json", r#"[{"slug":"x<b>y&z"}]"#),
            (
                "src/App.wf",
                "data posts = \"posts.json\"\n\
                 page Post(path: \"/p/:slug\", slug: String, title: \"T {slug}\", description: \"d\", paths: posts.map(p => p.slug)) {\n    Heading(slug).h1\n}\n\
                 page Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"Home\").h1 }\n",
            ),
            (
                "src/about.md",
                "---\ntitle: About <i>us</i> & them\ndescription: Who.\n---\n# About\n",
            ),
        ],
    );
    build(&dir);
    let about = read(&dir, "build/about/index.html");
    assert!(
        about.contains("<title>About &lt;i&gt;us&lt;/i&gt; &amp; them</title>"),
        "{about}"
    );
    let written: Vec<_> = std::fs::read_dir(dir.join("build/p"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(written.len(), 1, "{written:?}");
    let html = std::fs::read_to_string(written[0].join("index.html")).unwrap();
    assert!(
        html.contains("<title>T x&lt;b&gt;y&amp;z</title>"),
        "{html}"
    );
    assert_eq!(tags_named(&html, "b>"), 0, "{html}");
}

#[test]
fn the_shell_title_from_the_config_is_text() {
    let dir = project(
        "title-shell",
        &[
            (
                "webfluent.app.json",
                r#"{"name":"t","build":{"minify":false,"ssg":false,"csp":false,"compress":false},"meta":{"title":"</title><script>alert(1)</script> & co"}}"#,
            ),
            (
                "src/App.wf",
                "page Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"Home\").h1 }\n",
            ),
        ],
    );
    build(&dir);
    let html = read(&dir, "build/index.html");
    assert!(
        html.contains(
            "<title>&lt;/title&gt;&lt;script&gt;alert(1)&lt;/script&gt; &amp; co</title>"
        ),
        "{html}"
    );
    assert!(!html.contains("<script>alert"), "{html}");
}

#[test]
fn the_custom_elements_page_title_is_text() {
    let program = webfluent::parse_source(
        "component PriceTag(amount: Number) { Text(\"{amount}\") }\n",
        "t.wf",
    )
    .unwrap();
    let mut config: webfluent::config::ProjectConfig =
        serde_json::from_str(r#"{"name":"Shop </title><script>alert(1)</script>"}"#).unwrap();
    config.build.elements = vec!["PriceTag".to_string()];
    let html = webfluent::codegen::elements::demo_page(&program, &config, ".");
    assert!(!html.contains("<script>alert"), "{html}");
    assert!(html.contains("Shop &lt;/title&gt;"), "{html}");
}

#[test]
fn the_policy_comes_right_after_the_charset_on_every_page() {
    let static_dir = project(
        "csp-first",
        &[
            ("webfluent.app.json", STATIC_CSP),
            (
                "src/App.wf",
                "page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    head { meta(name: \"x-note\", content: \"n\") }\n    Heading(\"Home\").h1\n}\n",
            ),
        ],
    );
    build(&static_dir);
    let html = read(&static_dir, "build/index.html");
    assert!(
        line_after(&html, "<meta charset=").contains("Content-Security-Policy"),
        "{html}"
    );
    let policy = html.find("Content-Security-Policy").unwrap();
    for later in ["<title>", "data-wf-head", "og:title", "<link", "<script"] {
        assert!(
            html.find(later).is_none_or(|at| at > policy),
            "`{later}` comes before the policy:\n{html}"
        );
    }

    let spa_dir = project(
        "csp-first-spa",
        &[
            ("webfluent.app.json", SPA_CSP),
            (
                "src/App.wf",
                "page Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"Home\").h1 }\n",
            ),
        ],
    );
    build(&spa_dir);
    let shell = read(&spa_dir, "build/index.html");
    assert!(
        line_after(&shell, "<meta charset=").contains("Content-Security-Policy"),
        "{shell}"
    );
}

// ─── S4: `class:` from data ─────────────────────────────────────────────

#[test]
fn a_class_from_data_stays_inside_the_class_attribute() {
    let payload =
        r#"x"/tabindex="0"/autofocus/onfocus="document.body.setAttribute('data-pwned','class')"#;
    let dir = project(
        "class",
        &[
            ("webfluent.app.json", STATIC),
            (
                "d.json",
                &serde_json::json!({ "c": payload, "q": "a'b<c>" }).to_string(),
            ),
            (
                "src/App.wf",
                "data d = \"d.json\"\n\
                 component Chip(_ label: String) { Text(label) }\n\
                 page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    \
                 Text(\"one\", class: d.c)\n    Text(\"two\", class: [d.c, \"ok\"])\n    \
                 Card(class: { \"plain\": true }) { Text(\"three\", class: d.q) }\n    Chip(\"four\", class: d.c)\n}\n",
            ),
        ],
    );
    build(&dir);
    let html = read(&dir, "build/index.html");
    let body = &html[html.find("<body").unwrap()..];
    assert!(!body.contains("onfocus=\""), "{body}");
    assert!(!body.contains(" tabindex=\"0\"/"), "{body}");
    assert!(body.contains("/autofocus/onfocus=&quot;document"), "{body}");
    assert!(body.contains("a&#x27;b&lt;c&gt;"), "{body}");
    assert_eq!(
        body.matches("/autofocus/onfocus=&quot;document").count(),
        3,
        "{body}"
    );
    assert!(body.contains("class=\"wf-card plain\""), "{body}");
}

// ─── S5: a chart's and a QR code's colours ──────────────────────────────

#[test]
fn chart_and_qr_colours_are_colours() {
    use serde_json::json;
    use webfluent::codegen::charts::graphic;
    let rows = json!([{"m": "a", "v": 3, "w": 5}, {"m": "b", "v": 4, "w": 1}]);
    let hostile = [
        "red\"/><SCRIPT>alert(1)</SCRIPT><rect fill=\"x",
        "#000\" onload=\"alert(2)",
        "url(javascript:alert(1))",
        "red;stroke:url(https://evil.example/x)",
        "expression(alert(1))",
        "",
    ];
    for bad in hostile {
        for kind in ["bar", "line", "pie", "donut", "area"] {
            let svg = graphic(
                "Chart",
                None,
                &[
                    ("kind".into(), json!(kind)),
                    ("data".into(), rows.clone()),
                    ("x".into(), json!("m")),
                    ("y".into(), json!(["v", "w"])),
                    ("colors".into(), json!([bad, "#0F766E"])),
                    ("ink".into(), json!(bad)),
                    ("grid".into(), json!(bad)),
                    ("legend".into(), json!(true)),
                    ("labels".into(), json!(true)),
                ],
                &[],
            )
            .unwrap();
            assert!(
                !svg.to_ascii_lowercase().contains("<script"),
                "{bad}: {svg}"
            );
            assert!(!svg.contains("onload"), "{bad}: {svg}");
            assert!(!svg.contains("javascript"), "{bad}: {svg}");
            assert!(!svg.contains("evil.example"), "{bad}: {svg}");
            assert!(!svg.contains("expression("), "{bad}: {svg}");
            assert!(svg.contains("#0F766E"), "the good colour is kept: {svg}");
        }
        let qr = graphic(
            "QrCode",
            Some(&json!("https://example.com")),
            &[
                ("color".into(), json!(bad)),
                ("background".into(), json!(bad)),
            ],
            &[],
        )
        .unwrap();
        assert!(!qr.to_ascii_lowercase().contains("<script"), "{qr}");
        assert!(!qr.contains("onload"), "{qr}");
        assert!(
            qr.contains("fill=\"#000000\"") && qr.contains("fill=\"#ffffff\""),
            "{qr}"
        );
    }
    // What a colour may be.
    for good in [
        "#0F766E",
        "#abc",
        "#AABBCCDD",
        "red",
        "transparent",
        "currentColor",
        "rgb(1, 2, 3)",
        "rgba(1,2,3,0.5)",
        "hsl(210 40% 50%)",
        "oklch(70% 0.1 200 / 50%)",
        "var(--color-primary)",
    ] {
        assert_eq!(
            webfluent::codegen::charts::css_color(good).as_deref(),
            Some(good),
            "{good} is a colour"
        );
        let qr = graphic(
            "QrCode",
            Some(&json!("x")),
            &[("color".into(), json!(good))],
            &[],
        )
        .unwrap();
        assert!(qr.contains(&format!("fill=\"{good}\"")), "{qr}");
    }
}

#[test]
fn a_chart_from_data_in_a_static_build_draws_no_markup_of_its_own() {
    let dir = project(
        "chart",
        &[
            ("webfluent.app.json", STATIC_CSP),
            (
                "d.json",
                r##"{"rows":[{"m":"a","v":3}],"cs":["red\"/><SCRIPT>alert(1)</SCRIPT><rect fill=\"x"],"ink":"#000\" onload=\"alert(2)"}"##,
            ),
            (
                "src/App.wf",
                "data d = \"d.json\"\n\
                 page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    \
                 Chart(kind: .bar, data: d.rows, x: \"m\", y: \"v\", colors: d.cs, ink: d.ink)\n    \
                 QrCode(\"https://example.com\", color: d.ink, background: d.ink)\n}\n",
            ),
            // A second page, so this one's code is a chunk of its own.
            (
                "src/pages/About.wf",
                "page About(path: \"/about\", title: \"About\", description: \"a\") {\n    Heading(\"About\").h1\n}\n",
            ),
        ],
    );
    build(&dir);
    let html = read(&dir, "build/index.html");
    assert!(!html.contains("<SCRIPT"), "{html}");
    assert!(!html.contains("onload"), "{html}");
    let js = read(&dir, "build/pages/Home.js");
    assert!(!js.contains("SCRIPT>"), "{js}");
    assert!(!js.contains("onload"), "{js}");
}

// ─── S6: a page's head { } tags ─────────────────────────────────────────

fn head_project(name: &str, head: &str, data: &str) -> PathBuf {
    project(
        name,
        &[
            ("webfluent.app.json", STATIC_CSP),
            ("d.json", data),
            (
                "src/App.wf",
                &format!(
                    "data d = \"d.json\"\n\
                     page Home(path: \"/\", title: \"Home\", description: \"d\") {{\n    head {{\n        {head}\n    }}\n    Heading(\"Home\").h1\n}}\n"
                ),
            ),
        ],
    )
}

#[test]
fn a_head_link_from_data_never_names_a_url_a_browser_runs() {
    for url in [
        "javascript:alert(1)",
        " JaVa\tScRiPt:alert(1)",
        "data:text/css,body{}",
        "vbscript:x",
    ] {
        let dir = head_project(
            "head-link",
            "link(rel: \"canonical\", href: d.h)\n        link(rel: \"icon\", href: d.ok)",
            &serde_json::json!({ "h": url, "ok": "/favicon.png" }).to_string(),
        );
        build(&dir);
        let html = read(&dir, "build/index.html");
        let lower = html.to_ascii_lowercase();
        assert!(!lower.contains("javascript"), "{url}: {html}");
        assert!(!lower.contains("vbscript"), "{url}: {html}");
        assert!(!lower.contains("data:text"), "{url}: {html}");
        assert!(html.contains("href=\"/favicon.png\""), "{html}");
        // The live page guards what it writes too.
        let js = read(&dir, "build/app.js");
        assert!(js.contains("WF.head("), "{js}");
        let runtime = read(&dir, "build/app.js");
        assert!(
            runtime.contains("safeUrl(") && runtime.contains("function head("),
            "the runtime's head guards its URLs"
        );
    }
}

#[test]
fn a_head_script_or_refresh_from_data_is_refused() {
    for (head, what) in [
        ("script(src: d.s)", "script"),
        ("script(src: \"{d.s}\")", "script"),
        ("meta(http-equiv: \"refresh\", content: d.r)", "refresh"),
        (
            "meta(http-equiv: \"Refresh\", content: \"0; url={d.r}\")",
            "refresh",
        ),
        ("meta(http-equiv: d.e, content: \"0\")", "http-equiv"),
        (
            "meta(http-equiv: \"REFRESH\", content: \"0; URL=javascript:alert(1)\")",
            "refresh",
        ),
        (
            "link(rel: \"stylesheet\", href: \"javascript:alert(1)\")",
            "javascript",
        ),
    ] {
        let dir = head_project(
            "head-refused",
            head,
            r#"{"s":"data:text/javascript,alert(1)","r":"0; url=https://evil.example","e":"refresh"}"#,
        );
        let (ok, said) = wf(&dir, &["build"]);
        assert!(!ok, "`{head}` must not build:\n{said}");
        assert!(said.contains("E107"), "`{head}`: {said}");
        assert!(said.contains(what), "`{head}` names {what}: {said}");
        let (ok, said) = wf(&dir, &["check"]);
        assert!(!ok && said.contains("E107"), "`wf check` agrees: {said}");
    }
    // What the author wrote out is theirs to write.
    let dir = head_project(
        "head-literal",
        "script(src: \"/analytics.js\", defer: true)\n        meta(http-equiv: \"refresh\", content: \"300\")\n        meta(http-equiv: \"refresh\", content: \"5; url=/home\")",
        "{}",
    );
    build(&dir);
    let html = read(&dir, "build/index.html");
    assert!(
        html.contains("<script data-wf-head src=\"/analytics.js\" defer></script>"),
        "{html}"
    );
}

// ─── S14: a frame drawing HTML from a value ─────────────────────────────

#[test]
fn a_frame_drawing_html_from_a_value_is_sandboxed_unless_it_says_otherwise() {
    let dir = project(
        "srcdoc",
        &[
            ("webfluent.app.json", STATIC),
            (
                "d.json",
                r#"{"h":"<script>parent.document.body.setAttribute('data-pwned','srcdoc')</script>"}"#,
            ),
            (
                "src/App.wf",
                "data d = \"d.json\"\n\
                 page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    \
                 state html = \"<p>hi</p>\"\n    \
                 Frame(srcdoc: d.h, title: \"one\")\n    \
                 Frame(srcdoc: html, title: \"two\")\n    \
                 Frame(srcdoc: d.h, title: \"three\", sandbox: \"allow-scripts\")\n    \
                 Frame(srcdoc: \"<p>written here</p>\", title: \"four\")\n}\n",
            ),
            // A second page, so this one's code is a chunk of its own.
            (
                "src/pages/About.wf",
                "page About(path: \"/about\", title: \"About\", description: \"a\") {\n    Heading(\"About\").h1\n}\n",
            ),
        ],
    );
    let said = build(&dir);
    let html = read(&dir, "build/index.html");
    let frames: Vec<&str> = html.split("<iframe").skip(1).collect();
    assert_eq!(frames.len(), 4, "{html}");
    assert!(frames[0].contains("sandbox=\"\""), "{}", frames[0]);
    assert!(frames[1].contains("sandbox=\"\""), "{}", frames[1]);
    assert!(
        frames[2].contains("sandbox=\"allow-scripts\""),
        "{}",
        frames[2]
    );
    assert!(
        !frames[3].contains("sandbox="),
        "a literal is the author's: {}",
        frames[3]
    );
    let js = read(&dir, "build/pages/Home.js");
    assert_eq!(js.matches("sandbox: \"\"").count(), 2, "{js}");
    assert!(said.contains("V03"), "{said}");
    let (_, check) = wf(&dir, &["check"]);
    assert_eq!(
        check.matches("warning[V03]").count(),
        2,
        "the two frames with no sandbox of their own:\n{check}"
    );
}

#[test]
fn a_frame_hears_only_its_own_origin() {
    let js = webfluent::codegen::JsCodegen::new().generate(
        &webfluent::parse_source(
            "page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    state heard = \"\"\n    \
             Frame(src: \"/preview/\", title: \"p\") { on message(m) { heard = m.data } }\n    \
             Frame(srcdoc: \"<p>x</p>\", title: \"q\", sandbox: \"allow-scripts\") { on message { heard = event.data } }\n    Text(heard)\n}\n",
            "t.wf",
        )
        .unwrap(),
    );
    assert_eq!(
        js.matches("WF.listen(window, \"message\"").count(),
        2,
        "{js}"
    );
    for listener in js.split("WF.listen(window, \"message\"").skip(1) {
        let line = listener.lines().next().unwrap();
        assert!(line.contains(".source !== "), "{line}");
        assert!(line.contains(".origin !== "), "{line}");
    }
    webfluent::codegen::jscheck::check_js(&js).unwrap();
}

// ─── S16: design tokens ─────────────────────────────────────────────────

#[test]
fn a_token_cannot_close_the_rule_or_the_style_element() {
    use std::collections::HashMap;
    let mut tokens: HashMap<String, String> = HashMap::new();
    tokens.insert(
        "color-primary".into(),
        "red}</style><script>alert(1)</script>".into(),
    );
    tokens.insert(
        "color-secondary".into(),
        "red; } body { display:none".into(),
    );
    tokens.insert("x{y".into(), "red".into());
    tokens.insert("a b".into(), "red".into());
    tokens.insert("viz-1".into(), "#ff6a2b".into());
    tokens.insert("font-family".into(), "\"Inter\", sans-serif".into());
    tokens.insert("surface-تجربة".into(), "#fff".into());
    for css in [
        webfluent::codegen::generate_css(&tokens),
        webfluent::codegen::dark_css(&tokens),
    ] {
        assert!(!css.contains("</style"), "{css}");
        assert!(!css.contains("display:none"), "{css}");
        assert!(!css.contains("--x{y"), "{css}");
        assert!(!css.contains("--a b"), "{css}");
        assert!(css.contains("--viz-1: #ff6a2b;"), "{css}");
        assert!(
            css.contains("--font-family: \"Inter\", sans-serif;"),
            "{css}"
        );
        assert!(css.contains("--surface-تجربة: #fff;"), "{css}");
    }
    let page =
        webfluent::Template::from_str("page Hi(path: \"/\", title: \"Hi\") { Heading(\"Hi\").h1 }")
            .unwrap()
            .with_tokens(&[("color-primary", "red}</style><script>alert(1)</script>")])
            .render_html(&serde_json::json!({}))
            .unwrap();
    assert!(!page.contains("<script>alert"), "{page}");
}

// ─── S17: the policy, read back ─────────────────────────────────────────

#[test]
fn the_read_back_sees_markup_in_any_case_and_any_spelling() {
    use webfluent::codegen::csp::check;
    const POLICY: &str =
        "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'";
    let cases: &[(&str, &str)] = &[
        ("<SCRIPT>alert(1)</SCRIPT>", "an inline <script>"),
        ("<ScRiPt>alert(1)</sCrIpT>", "an inline <script>"),
        ("<script >alert(1)</script >", "an inline <script>"),
        ("<STYLE>p{}</STYLE>", "an inline <style>"),
        ("<p STYLE=\"color:red\">x</p>", "a style= attribute"),
        ("<p/style=\"color:red\">x</p>", "a style= attribute"),
        ("<p\nstyle='color:red'>x</p>", "a style= attribute"),
        ("<p style=color:red>x</p>", "a style= attribute"),
        ("<p/onfocus=\"x()\">x</p>", "an `onfocus` attribute"),
        ("<img\nonerror=alert(1)>", "an `onerror` attribute"),
        ("<img src=x ONERROR=alert(1)>", "an `onerror` attribute"),
        ("<img src=x\tonload = alert(1)>", "an `onload` attribute"),
        (
            "<script src=\"data:text/javascript,alert(1)\"></script>",
            "a script from data:",
        ),
        (
            "<SCRIPT SRC=\"javascript:alert(1)\"></SCRIPT>",
            "a script from javascript:",
        ),
        (
            "<script src=\"https://cdn.example.com/a.js\"></script>",
            "a script from https://cdn.example.com",
        ),
        (
            "<LINK REL=stylesheet HREF=\"https://cdn.example.com/a.css\">",
            "a stylesheet from https://cdn.example.com",
        ),
        (
            "<link rel=\"stylesheet\" href=\"data:text/css,p{}\">",
            "a stylesheet from data:",
        ),
        ("<base href=\"https://evil.example/\">", "a <base> element"),
        ("<BASE HREF=//evil.example>", "a <base> element"),
        (
            "<meta http-equiv=\"refresh\" content=\"0; url=https://evil.example\">",
            "a refresh",
        ),
        (
            "<META HTTP-EQUIV=Refresh CONTENT='0;URL=//evil.example'>",
            "a refresh",
        ),
    ];
    for (html, expected) in cases {
        let found = check("index.html", html, POLICY);
        assert!(
            found.iter().any(|v| v.what == *expected),
            "{html}: expected {expected}, found {found:?}"
        );
    }
    // And what the engine writes is still clean.
    let clean = "<html><head><meta charset=\"UTF-8\">\
        <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'\">\
        <link rel=\"stylesheet\" href=\"/styles.css\"><link rel=\"canonical\" href=\"https://x.example/\">\
        <script src=\"./app.js\" defer></script>\
        <script type=\"application/ld+json\">{\"a\":\"b\"}</script>\
        <meta http-equiv=\"refresh\" content=\"300\"><meta http-equiv=\"refresh\" content=\"5; url=/home\">\
        <!-- <script>commented()</script> <p style=\"x\"> --></head>\
        <body><p class=\"wf-text\">only once, onto the style= of text</p>\
        <pre><code>&lt;button onClick={save} style=\"x\"&gt;</code></pre></body></html>";
    assert_eq!(check("index.html", clean, POLICY), Vec::new());
}

#[test]
fn a_page_the_policy_refuses_is_not_left_behind_as_a_built_page() {
    let dir = project(
        "csp-left",
        &[
            ("webfluent.app.json", STATIC_CSP),
            (
                "src/App.wf",
                "page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    Unsafe.Html(\"<SCRIPT>alert(1)</SCRIPT>\")\n}\n\
                 page About(path: \"/about\", title: \"About\", description: \"d\") { Heading(\"About\").h1 }\n",
            ),
        ],
    );
    let (ok, said) = wf(&dir, &["build"]);
    assert!(!ok, "{said}");
    assert!(said.contains("E902"), "{said}");
    assert!(
        !dir.join("build/index.html").exists(),
        "the refused page is not left where a deploy would publish it"
    );
}

// ─── S24: a sanitised link that opens elsewhere ─────────────────────────

#[test]
fn a_sanitised_link_that_opens_elsewhere_never_keeps_its_opener() {
    use webfluent::codegen::sanitize::sanitize;
    let cases: &[(&str, &str)] = &[
        (
            "<a href=\"https://e.example\" target=\"_blank\" rel=\"opener\">x</a>",
            "<a href=\"https://e.example\" target=\"_blank\" rel=\"noopener noreferrer\">x</a>",
        ),
        (
            "<a href=\"https://e.example\" target=\"_blank\" rel=\"\">x</a>",
            "<a href=\"https://e.example\" target=\"_blank\" rel=\"noopener noreferrer\">x</a>",
        ),
        (
            "<a href=\"https://e.example\" target=\"_blank\" rel=\"nofollow OPENER\">x</a>",
            "<a href=\"https://e.example\" target=\"_blank\" rel=\"nofollow noopener noreferrer\">x</a>",
        ),
        (
            "<a href=\"https://e.example\" rel=\"noopener\" target=\"x\">x</a>",
            "<a href=\"https://e.example\" rel=\"noopener noreferrer\" target=\"x\">x</a>",
        ),
        (
            "<a href=\"https://e.example\" target=\"_blank\">x</a>",
            "<a href=\"https://e.example\" target=\"_blank\" rel=\"noopener noreferrer\">x</a>",
        ),
        // No target: `opener` goes, and an empty rel with it.
        (
            "<a href=\"/x\" rel=\"opener\">x</a>",
            "<a href=\"/x\">x</a>",
        ),
        (
            "<a href=\"/x\" rel=\"nofollow opener\">x</a>",
            "<a href=\"/x\" rel=\"nofollow\">x</a>",
        ),
        // No target: the author's rel stands.
        (
            "<a href=\"/x\" rel=\"nofollow\">x</a>",
            "<a href=\"/x\" rel=\"nofollow\">x</a>",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(sanitize(input), *expected, "{input}");
    }
}

// ─── S30: `style:` as an argument ───────────────────────────────────────

#[test]
fn style_as_an_argument_is_refused_for_a_style_block() {
    for (call, data) in [
        ("Text(\"t\", style: d.j)", true),
        ("Stack(style: \"color: red\") { Text(\"x\") }", false),
        ("Element(\"x-y\", style: d.j)", true),
        ("Button(\"b\", STYLE: d.j)", true),
    ] {
        let dir = project(
            "style-arg",
            &[
                ("webfluent.app.json", STATIC),
                ("d.json", r#"{"j":"position:fixed;inset:0"}"#),
                (
                    "src/App.wf",
                    &format!(
                        "{}page Home(path: \"/\", title: \"Home\", description: \"d\") {{\n    Heading(\"Home\").h1\n    {call}\n}}\n",
                        if data { "data d = \"d.json\"\n" } else { "" }
                    ),
                ),
            ],
        );
        let (ok, said) = wf(&dir, &["build"]);
        assert!(!ok, "`{call}` must not build:\n{said}");
        assert!(said.contains("E106"), "{call}: {said}");
        assert!(
            said.contains("style {"),
            "{call}: points at `style {{ }}`: {said}"
        );
    }
}
