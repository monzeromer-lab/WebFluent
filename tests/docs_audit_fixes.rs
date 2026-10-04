//! What the documentation audit found by writing the guide's examples out as
//! a real project and loading it in a browser: each was a program the
//! checks accepted and the page then got wrong.

mod common;

use common::spa_generated;

/// `head { meta(content: post?.title) }` over a `derived post` threw
/// `Cannot access '_post' before initialization`: the head tags were emitted
/// at the top of the page function, before any `derived` was declared, and
/// every detail page that named its own `og:title` drew nothing.
#[test]
fn a_head_tag_may_read_a_derived_of_the_page() {
    let js = spa_generated(
        "const POSTS = [{ slug: \"a\", title: \"A\" }]\n\
         page Post(path: \"/p/:slug\", slug: String, title: \"Post\", description: \"One post.\") {\n\
         \x20   derived post = POSTS.find(p => p.slug == slug)\n\
         \x20   head { meta(property: \"og:title\", content: post?.title ?? \"Post\") }\n\
         \x20   Heading(post?.title ?? \"?\").h1\n\
         }\n",
    );
    let declared = js.find("const _post").expect("the derived is declared");
    let head = js.find("WF.head(").expect("the head tags are set");
    assert!(
        head > declared,
        "the head tags are set before the derived they read exists:\n{js}"
    );
}

/// An action handed over as a value — to `addEventListener`, or to a library
/// that calls back — compiled to `_track()`: the page read it as a signal,
/// which does not exist, and the effect threw where it was set up.
#[test]
fn an_action_named_as_a_value_is_the_function() {
    let js = spa_generated(
        "page P(path: \"/\", title: \"T\", description: \"D\") {\n\
         \x20   state y = 0\n\
         \x20   action track() { y = window.scrollY }\n\
         \x20   effect {\n\
         \x20       window.addEventListener(\"scroll\", track)\n\
         \x20       cleanup { window.removeEventListener(\"scroll\", track) }\n\
         \x20   }\n\
         \x20   Text(\"{y}\")\n\
         }\n",
    );
    assert!(js.contains("addEventListener(\"scroll\", track)"), "{js}");
    assert!(!js.contains("_track()"), "{js}");
}

/// `Text("{rows.state}")` showed the source of the signal, once: what a
/// resource or a connection holds is a signal on its handle, and a field read
/// off it was emitted as a plain property.
#[test]
fn a_resource_or_a_connection_is_read_as_what_it_holds() {
    let js = spa_generated(
        "page P(path: \"/\", title: \"T\", description: \"D\") {\n\
         \x20   resource rows = fetch(\"/api/rows\")\n\
         \x20   socket chat = ws(\"wss://example.com/chat\")\n\
         \x20   Text(\"{rows.state}\")\n\
         \x20   for m in chat.messages { Text(m.text) }\n\
         }\n",
    );
    assert!(js.contains("_rows.state()"), "{js}");
    assert!(js.contains("_chat.messages()"), "{js}");
}

/// `paginate: .page` was refused by the checker as a parameter the endpoint
/// does not take, so the paging the guide documents could not be written.
#[test]
fn a_call_may_ask_for_pages() {
    let src = "type Item { id: String, name: String }\n\
               api Catalog(base: \"/api\") { get items(page: Number = 1) -> [Item] }\n\
               page P(path: \"/\", title: \"T\", description: \"D\") {\n\
               \x20   state page = 1\n\
               \x20   resource list = Catalog.items(page: page, paginate: .page)\n\
               \x20   for item in list.items by item.id { Text(item.name) }\n\
               \x20   if list.hasMore { Button(\"More\") { on click { list.loadMore() } } }\n\
               }\n";
    let program = webfluent::parse_source(src, "t.wf").expect("parses");
    let typed = webfluent::sema::types::check(&program, &|_| "t.wf".to_string());
    assert!(
        typed.findings.errors.is_empty(),
        "{:?}",
        typed.findings.errors
    );
    let js = spa_generated(src);
    assert!(
        js.contains("_list.items()") && js.contains("_list.hasMore()"),
        "{js}"
    );
}

/// A project directory with `config` and one page, built by the binary this
/// test run made; the bundle it wrote.
fn built_bundle(config: &str, dotenv: Option<&str>, page: &str, shell: &[(&str, &str)]) -> String {
    let dir = std::env::temp_dir().join(format!(
        "wf-audit-{}-{}",
        std::process::id(),
        page.len() + config.len()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    if let Some(text) = dotenv {
        std::fs::write(dir.join(".env"), text).unwrap();
    }
    std::fs::write(dir.join("src/App.wf"), page).unwrap();
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_wf"));
    cmd.args(["build", "-d"]).arg(&dir);
    for (k, v) in shell {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("run wf build");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    js
}

/// The compiler refused a page that read `env.STRIPE_SECRET` — and then wrote
/// the whole `env` map into `app.js`, secret and all, for every reader to
/// download. Only the names a page may read may be in the bundle.
#[test]
fn a_private_env_value_never_reaches_the_bundle() {
    let js = built_bundle(
        r#"{ "name": "t", "env": { "PUBLIC_API": "/api/v1", "STRIPE_SECRET": "sk_live_config" } }"#,
        Some("PUBLIC_MODE=beta\nDB_PASSWORD=hunter2-dotenv\n"),
        "page P(path: \"/\", title: \"T\", description: \"D\") {\n    Heading(\"{env.PUBLIC_API} {env.PUBLIC_MODE} {env.PUBLIC_SHELL}\").h1\n}\n",
        &[
            ("PUBLIC_SHELL", "from-the-shell"),
            ("OTHER_SECRET", "shell-secret"),
        ],
    );
    for secret in ["sk_live_config", "hunter2-dotenv", "shell-secret"] {
        assert!(!js.contains(secret), "{secret} is in the bundle");
    }
    for public in ["/api/v1", "beta", "from-the-shell"] {
        assert!(js.contains(public), "{public} is missing from the bundle");
    }
}

/// A page's or a component's action read its own parameters as the page's
/// signals: `n = n + by` compiled to `_n.set(_n() + _by())`, and `_by` does
/// not exist — every action that took an argument threw when it ran. Shipped
/// in 4.0 and 4.0.1; a store's actions were never affected.
#[test]
fn an_actions_parameters_are_its_own() {
    let js = spa_generated(
        "component Stepper(_ start: Number) {\n\
         \x20   state n = start\n\
         \x20   action move(by: Number) { n = n + by }\n\
         \x20   action touch(m: Map) { m.count = 1  m.bump() }\n\
         \x20   Button(\"+\") { on click { move(1) } }\n\
         }\n\
         page P(path: \"/\", title: \"T\", description: \"D\") {\n\
         \x20   state total = 0\n\
         \x20   action add(x: Number) { total = total + x }\n\
         \x20   Stepper(1)\n\
         \x20   Button(\"Add\") { on click { add(2) } }\n\
         }\n",
    );
    for wrong in ["_by()", "_m()", "_x()"] {
        assert!(
            !js.contains(wrong),
            "a parameter read as a signal, {wrong}:\n{js}"
        );
    }
    assert!(js.contains("_n() + by"), "{js}");
    assert!(js.contains("m.count = 1"), "{js}");
}

/// `navigator` and `location` were not among the browser's globals, so
/// `navigator.clipboard.writeText(…)` compiled to `_navigator()`, a signal
/// nothing declared — while the guide listed both as usable.
#[test]
fn navigator_and_location_are_the_browsers() {
    let js = spa_generated(
        "page P(path: \"/\", title: \"T\", description: \"D\") {\n\
         \x20   action copy(text: String) { await navigator.clipboard.writeText(text) }\n\
         \x20   Button(\"Copy\") { on click { copy(location.href) } }\n\
         }\n",
    );
    assert!(js.contains("navigator.clipboard"), "{js}");
    assert!(js.contains("copy(location.href)"), "{js}");
    assert!(
        !js.contains("_navigator()") && !js.contains("_location()"),
        "{js}"
    );
}

/// Every file a `:param` route wrote carried one `<title>`: `title:` was the
/// pattern's, not the route's. A parameter named in it is now that route's
/// value — the pre-rendered file's `<title>` and sharing card, and what the
/// router sets on the live page.
#[test]
fn a_routes_parameter_may_be_in_its_title() {
    let dir = std::env::temp_dir().join(format!("wf-audit-title-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "t", "build": { "ssg": true } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "const SLUGS = [\"button\", \"card\"]\n\
         page Entry(path: \"/ref/:slug\", slug: String, title: \"{slug} — Reference\", description: \"The {slug} entry.\", paths: SLUGS) {\n\
         \x20   Heading(slug).h1\n\
         }\n",
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let card = std::fs::read_to_string(dir.join("build/ref/card/index.html")).unwrap();
    assert!(card.contains("<title>card — Reference</title>"), "{card}");
    assert!(card.contains("The card entry."), "{card}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `title: "{entry?.name ?? slug} — Reference"` was written into every file
/// as those characters: only a bare parameter name was filled in, and a
/// title that looked a value up printed its own source. It is worked out
/// per file at build time, and by the router on each visit.
#[test]
fn a_title_may_splice_an_expression_over_the_parameters() {
    let dir = std::env::temp_dir().join(format!("wf-audit-title-expr-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "t", "build": { "ssg": true } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "const ENTRIES = [{ slug: \"icon-button\", name: \"IconButton\" }, { slug: \"card\", name: \"Card\" }]\n\
         page Entry(path: \"/ref/:slug\", slug: String, title: \"{ENTRIES.find(e => e.slug == slug)?.name ?? slug} — Reference\", description: \"One entry.\", paths: ENTRIES.map(e => e.slug)) {\n\
         \x20   Heading(slug).h1\n\
         }\n\
         page Home(path: \"/\", title: \"Home\", description: \"Home.\") { Heading(\"Home\").h1 }\n",
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let page = std::fs::read_to_string(dir.join("build/ref/icon-button/index.html")).unwrap();
    assert!(
        page.contains("<title>IconButton — Reference</title>"),
        "{page}"
    );
    assert!(
        page.contains(r#"content="IconButton — Reference""#),
        "the sharing card too: {page}"
    );
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(
        js.contains("title:(params)=>") && js.contains("params.slug"),
        "the router works the title out from the route: {js}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `app { state chosen = "en" … }` compiled every read of `chosen` and never
/// the `const` behind it — the site threw on load — though the guide shows an
/// app with state of its own.
#[test]
fn an_apps_own_state_is_declared() {
    let js = spa_generated(
        "app {\n\
         \x20   state open = false\n\
         \x20   Button(\"Menu\") { on click { open = !open } }\n\
         \x20   if open { Text(\"Menu\") }\n\
         \x20   Router\n\
         }\n\
         page P(path: \"/\", title: \"T\", description: \"D\") { Text(\"x\") }\n",
    );
    let declared = js.find("const _open").expect("the app's state is declared");
    let read = js.find("_open()").expect("and read");
    assert!(declared < read, "{js}");
}

/// `now(every: 1.seconds)` compiled to a bare `now(…)`, which nothing
/// declares.
#[test]
fn now_with_a_period_is_the_runtimes_clock() {
    let js = spa_generated(
        "page P(path: \"/\", title: \"T\", description: \"D\") {\n    Text(\"{now(every: 1.seconds)}\")\n}\n",
    );
    assert!(js.contains("WF.now("), "{js}");
}

/// `Grid(columns: { base: 1, md: 2, lg: 3 })` was painted with its
/// responsive class, and the page script then drew it again without one —
/// with `data-cols="[object Object]"` instead — so every responsive grid
/// fell back to one column the moment the page hydrated.
#[test]
fn a_responsive_layout_keeps_its_class_on_the_live_page() {
    let js = spa_generated(
        "page P(path: \"/\", title: \"T\", description: \"D\") {\n\
         \x20   Heading(\"T\").h1\n\
         \x20   Grid(columns: { base: 1, md: 2, lg: 3 }, gap: .md) { style { gap: 12px } Text(\"a\") }\n\
         \x20   Row(direction: { base: .column, md: .row }) { Text(\"b\") }\n\
         }\n",
    );
    let grid = js.find("className: \"wf-grid").expect("the grid is drawn");
    let line = &js[grid..grid + js[grid..].find('\n').unwrap_or(200)];
    assert!(
        line.contains(" wf-r"),
        "the grid carries its responsive class: {line}"
    );
    assert!(
        !js.contains("\"data-cols\""),
        "a map is not an attribute: {js}"
    );
    assert!(!js.contains("direction:"), "nor is `direction`: {js}");
}

/// `description: "{entry?.summary ?? slug}"` was written into every file as
/// its own source — only `title:` worked a splice out. A description is
/// worked out per file as the title is, and one that cannot be falls back
/// to the project's rather than printing braces.
#[test]
fn a_description_may_splice_an_expression_over_the_parameters() {
    let dir = std::env::temp_dir().join(format!("wf-audit-desc-expr-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "t", "build": { "ssg": true }, "meta": { "description": "The site." } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "const NOTES = [{ slug: \"bread\", summary: \"A quick loaf.\" }]\n\
         page Note(path: \"/n/:slug\", slug: String, title: \"{slug}\", description: \"{NOTES.find(n => n.slug == slug)?.summary ?? slug}\", paths: NOTES.map(n => n.slug)) {\n\
         \x20   Heading(slug).h1\n\
         }\n\
         page Home(path: \"/\", title: \"Home\", description: \"Home.\") { Heading(\"Home\").h1 }\n",
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let page = std::fs::read_to_string(dir.join("build/n/bread/index.html")).unwrap();
    assert!(
        page.contains(r#"name="description" content="A quick loaf.""#),
        "{page}"
    );
    assert!(!page.contains("NOTES.find"), "{page}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `query.tag == null` was false when the address had no `tag`: the key is
/// `undefined` in the browser, and `=== null` told the two apart.
#[test]
fn a_comparison_with_null_catches_an_absent_value() {
    let js = spa_generated(
        "page Home(path: \"/\", title: \"H\", description: \"D\") {\n\
         \x20   derived none = query.tag == null\n\
         \x20   derived some = query.tag != null\n\
         \x20   Text(if none { \"none\" } else { \"some\" })\n\
         \x20   Text(if some { \"yes\" } else { \"no\" })\n\
         }\n",
    );
    assert!(js.contains("== null") && !js.contains("=== null"), "{js}");
    assert!(js.contains("!= null") && !js.contains("!== null"), "{js}");
}

/// `disabled: !form.valid` was read once: `form.valid` reads signals, but
/// nothing else in the expression did, so it was written as a value and the
/// button stayed disabled.
#[test]
fn what_reads_a_form_handle_is_followed() {
    let js = spa_generated(
        "page Home(path: \"/\", title: \"H\", description: \"D\") {\n\
         \x20   state n = \"\"\n\
         \x20   validate n { required }\n\
         \x20   Form(bind: form) {\n\
         \x20       Input(bind: n, label: \"N\")\n\
         \x20       Button(\"Go\", type: .submit, disabled: !form.valid)\n\
         \x20   }\n\
         }\n",
    );
    assert!(js.contains("disabled: () => !form.valid"), "{js}");
}

/// `Blob([text])` compiled to a call, which throws: most of the browser's
/// classes must be constructed.
#[test]
fn a_browser_class_is_constructed() {
    let js = spa_generated(
        "page Home(path: \"/\", title: \"H\", description: \"D\") {\n\
         \x20   state csv = \"a,b\"\n\
         \x20   Button(\"Save\") { on click { let b = Blob([csv], { type: \"text/csv\" })  let u = URLSearchParams(\"a=1\")  log(b.size)  log(u) } }\n\
         }\n",
    );
    assert!(js.contains("new Blob(["), "{js}");
    assert!(js.contains("new URLSearchParams("), "{js}");
}

fn i18n_project(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-audit-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/translations")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "t", "i18n": { "default_locale": "en", "locales": ["en", "ar"], "dir": "src/translations" } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/translations/en.json"),
        r#"{ "hello": "Hello", "entries.one": "{count} entry", "entries.other": "{count} entries", "entries.in": "In {amount}" }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/translations/ar.json"),
        r#"{ "hello": "مرحبا", "entries.zero": "لا قيود", "entries.one": "قيد", "entries.two": "قيدان", "entries.few": "{count} قيود", "entries.many": "{count} قيدًا", "entries.other": "{count} قيد", "entries.in": "في {amount}" }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "page Home(path: \"/\", title: \"H\", description: \"D\") {\n\
         \x20   Heading(t(\"hello\")).h1\n\
         \x20   Text(t(\"entries\", { count: 2 }))\n\
         \x20   Text(t(\"entries.in\", { amount: \"$3\" }))\n\
         }\n",
    )
    .unwrap();
    dir
}

/// `t("entries", { count })` was told it passes `count` the message does
/// not use when a sibling message `entries.in` existed, and English was
/// asked for Arabic's `zero`, `two`, `few` and `many`, which fall back to
/// its `other`.
#[test]
fn plural_messages_draw_no_false_translation_findings() {
    let dir = i18n_project("plurals");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["check", "--deny-warnings", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{text}");
    assert!(!text.contains("I02") && !text.contains("I03"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `wf test` drew `t("hello")` as its key: a test reads the project's
/// translations, in its default locale, plural forms and placeholders
/// filled.
#[test]
fn a_test_reads_the_projects_translations() {
    let dir = i18n_project("test-i18n");
    std::fs::write(
        dir.join("tests/t.wf"),
        "test \"translated\" {\n\
         \x20   Heading(t(\"hello\")).h1\n\
         \x20   Text(t(\"entries\", { count: 1 }))\n\
         \x20   Text(t(\"entries\", { count: 3 }))\n\
         \x20   expect \"Hello\"\n\
         \x20   expect \"1 entry\"\n\
         \x20   expect \"3 entries\"\n\
         }\n",
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["test"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("1 passed"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}
