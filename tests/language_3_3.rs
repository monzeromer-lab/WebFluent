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
        r#"{ "name": "scratch", "build": { "output": "build" } }"#,
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
        "data posts = \"posts.json\"\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Posts\").h1\n    for p in posts by p.slug { Link(to: \"/p/{p.slug}\") { Text(p.title) } }\n}\npage Post(path: \"/p/:slug\", title: \"Post\", description: \"d\", slug: String, paths: posts.map(p => p.slug)) {\n    derived post = posts.find(p => p.slug == slug)\n    Heading(post?.title ?? \"?\").h1\n    Text(\"{slug}: {params.slug}\")\n    if let p = post { Text(\"Bound: {p.title}\") } else { Text(\"unbound\") }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("posts.json"),
        r#"[{"slug":"hello","title":"Hello world"},{"slug":"again","title":"Again"}]"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build", "ssg": true }, "meta": { "site_url": "https://x.y", "sitemap": true } }"#,
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let again = std::fs::read_to_string(dir.join("build/p/again/index.html")).unwrap();
    assert!(
        again.contains(">Again</h1>") && again.contains(">again: again<"),
        "{again}"
    );
    assert!(
        again.contains(">Bound: Again<") && !again.contains("unbound"),
        "the static paint binds an `if let` name: {again}"
    );
    assert!(
        again.contains("<link rel=\"canonical\" href=\"https://x.y/p/again\">"),
        "the file's canonical is its own route, not the pattern: {again}"
    );
    assert!(!again.contains(":slug"), "{again}");
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
        r#"{ "name": "scratch", "build": { "output": "build", "ssg": true } }"#,
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
        r#"{ "name": "scratch", "build": { "output": "build", "ssg": true }, "i18n": { "default_locale": "en", "locales": ["en"] } }"#,
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

#[test]
fn a_guarded_page_carries_its_guard_and_redirect_into_the_route_table() {
    let dir = scratch("guard");
    std::fs::write(
        dir.join("src/App.wf"),
        "store Auth { state user = null\n derived loggedIn = user != null }\napp { Router }\npage Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"h\").h1 }\npage Login(path: \"/login\", title: \"Login\", description: \"d\") { Heading(\"l\").h1 }\npage Account(path: \"/account\", title: \"Account\", description: \"d\", guard: Auth.loggedIn, redirect: \"/login\") { Heading(\"a\").h1 }\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(
        js.contains("guard:()=>(Auth.loggedIn),redirect:\"/login\""),
        "{js}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_store_action_wins_over_a_list_method_of_the_same_name() {
    let dir = scratch("store-action-name");
    std::fs::write(
        dir.join("src/App.wf"),
        "type Todo { id: Number, title: String }\n\
         store Todos {\n\
         \x20   state items: [Todo] = []\n\
         \x20   action remove(id: Number) { items = items.filter(t => t.id != id) }\n\
         \x20   action take(id: Number) { items = items.filter(t => t.id == id) }\n\
         }\n\
         app { Router }\n\
         page Home(path: \"/\", title: \"Home\", description: \"d\") {\n\
         \x20   use Todos\n\
         \x20   Heading(\"h\").h1\n\
         \x20   for todo in Todos.items by todo.id {\n\
         \x20       Button(\"x\") { on click { Todos.remove(todo.id) } }\n\
         \x20       Button(\"y\") { on click { Todos.take(todo.id) } }\n\
         \x20   }\n\
         }\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let js = std::fs::read_to_string(dir.join("build/pages/Home.js")).unwrap();
    // A store is an object of state and actions, not a list: mapping these
    // onto `splice`/`WF.take` left the handler calling what was never there.
    assert!(
        js.contains("Todos.remove(todo.id)"),
        "the store's own `remove` action is called, not a list's: {js}"
    );
    assert!(
        js.contains("Todos.take(todo.id)"),
        "the store's own `take` action is called, not the runtime helper: {js}"
    );
    assert!(
        !js.contains("Todos.splice("),
        "a store never gets a list's `splice`: {js}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn remove_on_a_state_list_goes_through_the_signal_so_the_view_repaints() {
    let dir = scratch("remove-repaints");
    std::fs::write(
        dir.join("src/App.wf"),
        "app { Router }\n\
         page Home(path: \"/\", title: \"Home\", description: \"d\") {\n\
         \x20   state nums: [Number] = [1, 2, 3]\n\
         \x20   Heading(\"h\").h1\n\
         \x20   Button(\"drop\") { on click { nums.remove(0) } }\n\
         \x20   for n in nums { Text(\"{n}\") }\n\
         }\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let js = std::fs::read_to_string(dir.join("build/pages/Home.js")).unwrap();
    // An in-place `splice` changes the list the signal already holds, so
    // nothing reading it repaints — `remove` sets the signal, as `push` does.
    assert!(
        js.contains("_nums.set(WF.removeAt(_nums()"),
        "`remove` sets the signal to a new list: {js}"
    );
    assert!(
        !js.contains("_nums().splice("),
        "`remove` never mutates the list in place: {js}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `t(…)` was resolved by the static paint only when it was the whole of a
/// text. Spliced into a string, added to one, compared, or held in a prop a
/// component then spliced, the evaluator had never heard of it — the value
/// came back unknown and the element painted empty until the script ran, for
/// a reader without JavaScript and for a crawler. The documentation site's
/// chapter breadcrumbs and prev/next links could not be translated because of
/// it.
#[test]
fn a_translation_paints_wherever_it_is_written() {
    let dir = scratch("t-anywhere");
    std::fs::create_dir_all(dir.join("src/translations")).unwrap();
    std::fs::write(
        dir.join("src/translations/en.json"),
        r#"{ "name": "Ada", "chapter": "Chapter {n}", "items.one": "{count} item", "items.other": "{count} items" }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        r#"component Shell(chapter: String = "", label: String = "") {
    slot
    Container {
        Text("crumb:{chapter}")
        if label != "" { Text("cond:{label}") }
        children
    }
}

page Home(path: "/", title: "Home", description: "d", layout: Shell(chapter: t("chapter", { n: "19" }), label: t("name"))) {
    Heading("h").h1
    Text("splice:{t("name")}")
    Text("plus:" + t("name"))
    Text("plural:{t("items", { count: 2 })}")
}
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build", "ssg": true }, "i18n": { "default_locale": "en", "locales": ["en"] } }"#,
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    for want in [
        ">splice:Ada<",
        ">plus:Ada<",
        ">plural:2 items<",
        ">crumb:Chapter 19<",
        ">cond:Ada<",
    ] {
        assert!(
            html.contains(want),
            "the static paint should carry `{want}`: {html}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
