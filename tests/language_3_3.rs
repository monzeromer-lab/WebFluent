//! The tooling of 3.3, run as a reader would run it: the `wf` binary over
//! a scratch project.

use std::path::Path;
use std::process::Command;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-3-3-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "entry": "src/App.wf", "output": "build" }"#,
    )
    .unwrap();
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
fn wf_test_renders_each_test_holds_it_to_its_expectations_and_keeps_a_snapshot() {
    let dir = scratch("test");
    std::fs::write(
        dir.join("src/App.wf"),
        "component Greeting(_ name: String, tone: String = \"calm\") {\n    Card { Text(\"Hello, {name}\")  Badge(tone) }\n}\npage P(path: \"/\", title: \"T\", description: \"d\") { Heading(\"x\").h1  Greeting(\"world\") }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/greeting.wf"),
        "test \"greets by name\" {\n    Greeting(\"Sam\")\n    expect \"Hello, Sam\"\n    expect not \"Hello, world\"\n}\ntest \"lists the data\"(data: { items: [\"a\", \"b\"] }) {\n    state open = true\n    for it in items { if open { Text(it) } }\n    expect \"b\"\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["test"]);
    assert!(ok, "{out}");
    assert!(
        out.contains("2 passed, 0 failed, 2 snapshot(s) written"),
        "{out}"
    );
    let snapshot = dir.join("tests/__snapshots__/greeting/greets-by-name.html");
    let html = std::fs::read_to_string(&snapshot).unwrap();
    assert!(
        html.contains(">Hello, Sam<") && html.contains(">calm<"),
        "{html}"
    );
    // A second run compares to the snapshot.
    let (ok, out) = wf(&dir, &["test"]);
    assert!(ok && out.contains("2 passed, 0 failed"), "{out}");
    // A change fails the expectation, and `--update` accepts a new render.
    std::fs::write(
        dir.join("src/App.wf"),
        "component Greeting(_ name: String, tone: String = \"calm\") {\n    Card { Text(\"Hi, {name}\")  Badge(tone) }\n}\npage P(path: \"/\", title: \"T\", description: \"d\") { Heading(\"x\").h1  Greeting(\"world\") }\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["test"]);
    assert!(!ok, "{out}");
    assert!(
        out.contains("expected \"Hello, Sam\", which the render does not hold"),
        "{out}"
    );
    std::fs::write(
        dir.join("tests/greeting.wf"),
        "test \"greets by name\" {\n    Greeting(\"Sam\")\n    expect \"Hi, Sam\"\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["test"]);
    assert!(!ok && out.contains("differs from"), "{out}");
    let (ok, out) = wf(&dir, &["test", "--update"]);
    assert!(ok, "{out}");
    // A build ignores the tests.
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_data_file_is_a_constant_and_paths_render_a_param_page_per_value() {
    let dir = scratch("data");
    std::fs::write(
        dir.join("src/App.wf"),
        "data posts = \"posts.json\"\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Posts\").h1\n    for p in posts by p.slug { Link(to: \"/p/{p.slug}\") { Text(p.title) } }\n}\npage Post(path: \"/p/:slug\", title: \"Post\", description: \"d\", slug: String, paths: posts.map(p => p.slug)) {\n    derived post = posts.find(p => p.slug == slug)\n    Heading(post?.title ?? \"?\").h1\n    Text(\"{slug}: {params.slug}\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("posts.json"),
        r#"[{"slug":"hello","title":"Hello world"},{"slug":"again","title":"Again"}]"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "entry": "src/App.wf", "output": "build", "build": { "ssg": true }, "meta": { "site_url": "https://x.y", "sitemap": true } }"#,
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let again = std::fs::read_to_string(dir.join("build/p/again/index.html")).unwrap();
    assert!(
        again.contains(">Again</h1>") && again.contains(">again: again<"),
        "{again}"
    );
    assert!(dir.join("build/p/hello/index.html").exists());
    let home = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(
        home.contains("href=\"/p/hello\"") && home.contains(">Hello world<"),
        "{home}"
    );
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(
        js.contains("posts=[") && js.contains("\"hello\""),
        "the bundle carries the data: {js}"
    );
    let sitemap = std::fs::read_to_string(dir.join("build/sitemap.xml")).unwrap_or_default();
    assert!(sitemap.contains("https://x.y/p/again"), "{sitemap}");
    // A missing file is an error that names it.
    std::fs::remove_file(dir.join("posts.json")).unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(!ok && out.contains("no file `posts.json`"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn markdown_renders_at_build_time_and_a_md_file_is_a_page() {
    let dir = scratch("md");
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "component Shell { slot  Container { children } }\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    state note = \"# Live\\n\\nSome *text* <b>\"\n    Heading(\"Home\").h1\n    Markdown(note)\n    Markdown(\"Plain **bold** here\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/pages/about.md"),
        "---\ntitle: About us\ndescription: Who we are.\nlayout: Shell\n---\n# About\n\nWe make *things*.\n\n- one\n- two\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "entry": "src/App.wf", "output": "build", "build": { "ssg": true } }"#,
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    assert!(
        !out.contains("Warning"),
        "a markdown page with a heading draws no warning: {out}"
    );
    let about = std::fs::read_to_string(dir.join("build/about/index.html")).unwrap();
    assert!(about.contains("<title>About us</title>"), "{about}");
    assert!(about.contains("<div class=\"wf-container\">\n                <div class=\"wf-markdown\">\n<h1>About</h1>\n<p>We make <em>things</em>.</p>\n<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n"), "{about}");
    let home = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(
        home.contains("<h1>Live</h1>\n<p>Some <em>text</em> &lt;b&gt;</p>"),
        "HTML in the text is shown, not run: {home}"
    );
    assert!(
        home.contains("<p>Plain <strong>bold</strong> here</p>"),
        "{home}"
    );
    let js = std::fs::read_to_string(dir.join("build/pages/Home.js")).unwrap();
    assert!(js.contains("markdown:()=>_note()"), "{js}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn wf_docs_writes_a_gallery_of_the_builtins_and_the_projects_declarations() {
    let dir = scratch("docs");
    std::fs::write(
        dir.join("src/App.wf"),
        "enum Tone { calm, loud }\ntype Todo { id: String, done: Bool = false }\n/// A chip.\ncomponent Chip(_ label: String, tone: Tone = .calm) {\n    event pick(id: String)\n    slot trailing\n    part Dot { Text(\"·\") }\n    Badge(label) { trailing }\n}\nstore Todos { state items: [Todo] = []\n action add(t: Todo) { items.push(t) } }\npage Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"x\").h1  Chip(\"a\") }\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["docs", "-o", "gallery"]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("gallery/index.html")).unwrap();
    for expected in [
        "<h1>Built-in components</h1>",
        "id=\"c-button\"",
        "<code>Table.Row</code>",
        "<code>.primary</code>",
        "<h2>Components</h2>",
        "<h3>Chip <span class=\"muted\">(_ label)</span></h3><p>A chip.</p>",
        "<code>on pick</code>",
        "<code>trailing</code>",
        "<code>Chip.Dot</code>",
        "<h2>Enums</h2>",
        "<code>.loud</code>",
        "<h2>Types</h2>",
        "<h2>Stores</h2>",
        "<code>add</code> <span class=\"muted\">action</span>",
        "<h2>Pages</h2>",
    ] {
        assert!(html.contains(expected), "{expected} in the gallery");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_message_with_a_count_picks_its_plural_form_at_build_time_and_live() {
    let dir = scratch("plural");
    std::fs::create_dir_all(dir.join("src/translations")).unwrap();
    std::fs::write(
        dir.join("src/translations/en.json"),
        r#"{ "items.one": "{count} item for {name}", "items.other": "{count} items for {name}", "hello": "Hello" }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    state n = 3\n    Heading(t(\"hello\")).h1\n    Text(t(\"items\", { count: 1, name: \"Sam\" }))\n    Text(t(\"items\", { count: n, name: \"Sam\" }))\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "entry": "src/App.wf", "output": "build", "build": { "ssg": true }, "i18n": { "default_locale": "en", "locales": ["en"] } }"#,
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(html.contains(">1 item for Sam<"), "{html}");
    assert!(
        html.contains(">3 items for Sam<"),
        "the static paint reads the seeded state: {html}"
    );
    let js = std::fs::read_to_string(dir.join("build/pages/Home.js")).unwrap();
    assert!(
        js.contains("WF.i18n.t(\"items\", ({count:_n(),name:\"Sam\"}))")
            || js.contains("WF.i18n.t(\"items\",({count:_n(),name:\"Sam\"}))"),
        "{js}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
