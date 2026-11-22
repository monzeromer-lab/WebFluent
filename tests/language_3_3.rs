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
        !out.contains("warning["),
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
fn wf_docs_documents_the_project_s_components_with_their_examples_drawn() {
    let dir = scratch("docs");
    std::fs::write(
        dir.join("src/App.wf"),
        "enum Tone { calm, loud }\ntype Todo { id: String, done: Bool = false }\n/// A chip.\ncomponent Chip(_ label: String, tone: Tone = .calm) {\n    event pick(id: String)\n    slot trailing\n    part Dot { Text(\"·\") }\n    Badge(label) { trailing }\n}\n/// A plain box.\ncomponent Boxed(title: String) { Card { Text(title)  children } }\nstore Todos { state items: [Todo] = []\n action add(todo: Todo) { items = items.concat([todo]) } }\npage Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"x\").h1  Chip(\"a\")  Boxed(title: \"b\") { Text(\"c\") }  Button(\"Add\") { on click { Todos.add(Todo(id: \"1\")) } } }\n/// Loud, for what matters.\nexample Chip \"Loud\" {\n    Chip(\"Urgent\", tone: .loud)\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["docs", "-o", "gallery"]);
    assert!(ok, "{out}");
    assert!(
        out.contains("2 component(s), 1 example(s) and 1 made from props"),
        "{out}"
    );
    let read = |p: &str| {
        std::fs::read_to_string(dir.join("gallery").join(p)).unwrap_or_else(|e| panic!("{p}: {e}"))
    };
    let chip = read("components/chip/index.html");
    for expected in [
        "Chip</h1>",
        "A chip.",
        "component Chip(_ label: String, tone: Tone = .calm)",
        "Loud, for what matters.",
        "src=\"/examples/chip/1/\"",
        "<span class=\"wf-tok-str\">&quot;Urgent&quot;</span>",
        "on pick",
        "trailing",
        "Chip.Dot",
        "Home",
    ] {
        assert!(chip.contains(expected), "{expected} on Chip's page");
    }
    // The example is a page of the project's own, under its path.
    let example = read("examples/chip/1/index.html");
    assert!(
        example.contains("/examples/chip/1/app.js") && example.contains("docs-frame.js"),
        "{example}"
    );
    assert!(read("examples/chip/1/app.js").contains("Urgent"));
    // A component with none is given one from its props, with its content.
    let boxed = read("components/boxed/index.html");
    assert!(
        boxed.contains("From its props") && boxed.contains("&quot;Title&quot;"),
        "{boxed}"
    );
    for (page, expected) in [
        ("types/index.html", ".loud"),
        ("stores/index.html", "add(todo: Todo)"),
        ("routes/index.html", "Home"),
    ] {
        assert!(read(page).contains(expected), "{expected} in {page}");
    }
    // In the reference's design.
    assert!(read("styles.css").contains("--construction"));
    // A second run clears what the first wrote, and nothing else.
    std::fs::write(dir.join("gallery/stale.txt"), "x").unwrap();
    let (ok, out) = wf(&dir, &["docs", "-o", "gallery"]);
    assert!(ok, "{out}");
    assert!(!dir.join("gallery/stale.txt").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_example_is_checked_formatted_and_left_out_of_the_build() {
    let dir = scratch("examples");
    let app = "component Card2(_ title: String) { Card { Text(title) } }\npage Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"x\").h1 }\n/// The usual.\nexample Card2 \"Plain\" {\n    Card2(\"Hello from an example\")\n}\n";
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    // Placed by an example only, the component is not unused.
    let (ok, out) = wf(&dir, &["check"]);
    assert!(ok, "{out}");
    assert!(!out.contains("U03"), "{out}");
    let (ok, out) = wf(&dir, &["build"]);
    assert!(ok, "{out}");
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(
        !js.contains("Hello from an example"),
        "an example is not built"
    );
    let (ok, out) = wf(&dir, &["fmt", "--check"]);
    assert!(ok, "formatted as written: {out}");
    // An example of nothing, or of the wrong values, is refused.
    std::fs::write(
        dir.join("src/App.wf"),
        format!("{app}example Nope \"x\" {{ Text(\"a\") }}\nexample Card2 \"Wrong\" {{ Card2(3) }}\nexample Card2 \"Acts\" {{ Card2(\"a\")  click \"a\" }}\n"),
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["check"]);
    assert!(!ok);
    assert!(out.contains("acts or expects: that is a test"), "{out}");
    std::fs::write(
        dir.join("src/App.wf"),
        format!(
            "{app}example Nope \"x\" {{ Text(\"a\") }}\nexample Card2 \"Wrong\" {{ Card2(3) }}\n"
        ),
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["check"]);
    assert!(!ok);
    assert!(
        out.contains("error[E101]: `Nope` is not a declared component"),
        "{out}"
    );
    assert!(out.contains("error[T01]"), "{out}");
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
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
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
        "type User { name: String }\nstore Auth { state user: User? = null\n derived loggedIn = user != null }\napp { Router }\npage Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"h\").h1 }\npage Login(path: \"/login\", title: \"Login\", description: \"d\") { Heading(\"l\").h1 }\npage Account(path: \"/account\", title: \"Account\", description: \"d\", guard: Auth.loggedIn, redirect: \"/login\") { Heading(\"a\").h1 }\n",
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
         \x20   action remove(id: Number) { items = items.filter(todo => todo.id != id) }\n\
         \x20   action take(id: Number) { items = items.filter(todo => todo.id == id) }\n\
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
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    // A store is an object of state and actions, not a list: mapping these
    // onto `splice`/`WF.take` left the handler calling what was never there.
    assert!(
        js.contains("Todos.remove($todo().id)"),
        "the store's own `remove` action is called, not a list's: {js}"
    );
    assert!(
        js.contains("Todos.take($todo().id)"),
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
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
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

/// `wf test` over `dir`, or `None` (said) where there is no Chrome for a
/// test that acts.
fn wf_test_in_browser(dir: &Path, args: &[&str]) -> Option<(bool, String)> {
    let (ok, out) = wf(dir, args);
    if out.contains("no Chrome or Chromium") {
        eprintln!("\n  SKIPPED: no Chrome or Chromium for `wf test`.\n");
        return None;
    }
    Some((ok, out))
}

#[test]
fn a_store_whose_first_value_is_a_constant_renders_it() {
    let dir = scratch("const-store");
    std::fs::write(
        dir.join("src/App.wf"),
        "const START: [String] = [\"alpha\", \"beta\"]\nconst FIRST = START[0]\n\nstore Kept {\n    persist kept: [String] = START\n    state held: [String] = START\n    state first: String = FIRST\n    derived count = held.length\n}\n\npage P(path: \"/\", title: \"T\", description: \"d\") {\n    use Kept\n    Heading(\"x\").h1\n    Text(\"{Kept.kept.length} {Kept.count} {Kept.first}\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/kept.wf"),
        "test \"a constant is a store's first value\" {\n    use Kept\n    Text(\"kept {Kept.kept.length} held {Kept.held.length} count {Kept.count} first {Kept.first}\")\n    expect \"kept 2 held 2 count 2 first alpha\"\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["test"]);
    assert!(ok, "{out}");
}

const SEEDED_APP: &str = r#"type Svc { name: String }

component Tile(_ s: Svc) {
    Text("Tile {s.name}")
}

page Seeded(path: "/", title: "Seeded", description: "d") {
    state greeting = "start"
    derived loud = greeting.toUpperCase()
    Heading("Seeded").h1
    Text("Greeting {greeting} {loud}")
    Button("Again") { on click { greeting = greeting } }
}
"#;

#[test]
fn a_tests_data_seeds_a_pages_state_and_a_name_in_a_browser_as_in_a_render() {
    let dir = scratch("seeded");
    std::fs::write(dir.join("src/App.wf"), SEEDED_APP).unwrap();
    std::fs::write(
        dir.join("tests/seeded.wf"),
        r#"test "rendered"(data: { greeting: "hello" }) {
    Seeded
    expect "Greeting hello HELLO"
}

test "in a browser"(data: { greeting: "hello" }) {
    Seeded
    click "Again"
    expect "Greeting hello HELLO"
}

test "a derived value seeded in a browser"(data: { loud: "QUIET" }) {
    Seeded
    click "Again"
    expect "Greeting start QUIET"
}

test "a name the data gives, in a browser"(data: { s: { name: "billing" } }) {
    Tile(s)
    Button("Look") { on click { } }
    click "Look"
    expect "Tile billing"
}
"#,
    )
    .unwrap();
    let Some((ok, out)) = wf_test_in_browser(&dir, &["test"]) else {
        return;
    };
    assert!(ok, "{out}");
    assert!(!out.contains("ReferenceError"), "{out}");
}

#[test]
fn a_step_brings_what_it_acts_on_into_view() {
    let dir = scratch("in-view");
    std::fs::write(
        dir.join("src/App.wf"),
        "page P(path: \"/\", title: \"T\", description: \"d\") {\n    Heading(\"x\").h1\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/fold.wf"),
        r#"test "a popconfirm below the fold asks"(data: { gone: false }) {
    Stack { style { height: 4000px } }
    Popconfirm(title: "Delete this file?", yes: "Delete").danger {
        on confirm { gone = true }
        Button("Delete it")
    }
    click "Delete it"
    expect "Delete this file?"
}
"#,
    )
    .unwrap();
    let Some((ok, out)) = wf_test_in_browser(&dir, &["test"]) else {
        return;
    };
    assert!(ok, "{out}");
}

#[test]
fn a_call_is_still_counted_after_a_navigate_loads_another_page() {
    let dir = scratch("calls-navigate");
    std::fs::write(
        dir.join("src/App.wf"),
        r#"api Backend(base: "/api") { }
api Backend.hits {
    post ping() at "ping" -> !
}

page Pinger(path: "/", title: "Pinger", description: "d") {
    Heading("Pinger").h1
    Button("Ping") {
        on click {
            match await Backend.hits.ping() { .ok { navigate("/elsewhere") } .err(_) { } }
        }
    }
}

page Elsewhere(path: "/elsewhere", title: "Elsewhere", description: "d") {
    Heading("Elsewhere").h1
}
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/ping.wf"),
        r#"fake Quiet for Backend.hits {
    ping() { .ok }
}

test "a ping is heard after the page moves on"(fakes: [Quiet]) {
    Pinger
    click "Ping"
    expect called Backend.hits.ping
}
"#,
    )
    .unwrap();
    let Some((ok, out)) = wf_test_in_browser(&dir, &["test"]) else {
        return;
    };
    assert!(ok, "{out}");
}

#[test]
fn fmt_check_says_what_it_checked_and_what_would_change() {
    let dir = scratch("fmt-check");
    std::fs::write(
        dir.join("src/App.wf"),
        "page P(path: \"/\", title: \"T\", description: \"d\") {\n    Heading(\"x\").h1\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["fmt", "--check"]);
    assert!(ok, "{out}");
    assert!(
        out.contains("1 file(s) checked, none would change"),
        "{out}"
    );
    assert!(!out.contains("formatted"), "nothing was formatted: {out}");

    std::fs::write(
        dir.join("src/App.wf"),
        "page P(path: \"/\", title: \"T\", description: \"d\"){\n  Heading(\"x\").h1\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["fmt", "--check"]);
    assert!(!ok, "{out}");
    assert!(out.contains("1 file(s) checked, 1 would change"), "{out}");
    assert!(out.contains("App.wf"), "{out}");
    let before = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    assert!(before.contains("\"d\"){"), "--check writes nothing");
}
