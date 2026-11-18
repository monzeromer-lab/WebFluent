//! What a hands-on pass over 7.0 found, in a browser against a real server,
//! each kept from coming back: an endpoint of a group named like a list
//! helper, a parameter's default, a bound `FileUpload`, a generated endpoint
//! with a setting after `!`, `wf render` holding a read to its endpoint, and
//! `wf migrate` writing `Json` only where it helps. The runtime's halves are
//! in `tests/js/hands_on.test.mjs`.

use std::path::PathBuf;

use crate::common::spa_generated;
use webfluent::migrate::seven;

const PEOPLE: &str = "type Person { id: String, name: String }\napi Backend(base: \"/api\") { }\napi Backend.people {\n    get list(pageSize: Number = 2 in query \"page-size\", tone: Tone = .calm) at \"people\" -> [Person]!\n    delete remove(id: String) at \"people/{id}\" -> !\n    invalidates [list]\n}\nenum Tone { calm, loud }\n";

fn page(body: &str) -> String {
    format!(
        "{PEOPLE}app {{ Router }}\npage Home(path: \"/\", title: \"Home\", description: \"d\") {{\n    Heading(\"Home\").h1\n{body}\n}}\n"
    )
}

#[test]
fn an_endpoint_named_like_a_list_helper_is_called_on_its_group() {
    let js = spa_generated(&page(
        "    resource people = Backend.people.list()\n    Button(\"Remove\") { on click { Backend.people.remove(id: \"7\") } }",
    ));
    assert!(
        js.contains("Backend.people.remove("),
        "the endpoint is called: {js}"
    );
    assert!(
        !js.contains("removeAt(Backend"),
        "never the list helper: {js}"
    );
}

#[test]
fn an_endpoint_carries_its_parameters_defaults() {
    let js = spa_generated(&page("    resource people = Backend.people.list()"));
    assert!(
        js.contains("defaults:")
            && (js.contains("\"pageSize\":2") || js.contains("\"pageSize\": 2")),
        "the literal default is sent when a call leaves it out: {js}"
    );
    assert!(js.contains("\"calm\""), "an enum case as its name: {js}");
}

#[test]
fn a_bound_file_upload_writes_what_was_chosen() {
    let src = "app { Router }\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    state files: [File] = []\n    state photo: File? = null\n    Heading(\"Home\").h1\n    FileUpload(bind: files, label: \"Photos\").multiple\n    FileUpload(bind: photo, label: \"Photo\")\n    Text(\"{files.length} chosen, {photo?.name ?? \"none\"}\")\n}\n";
    let js = spa_generated(src);
    assert!(
        js.contains("Array.from($$e.target.files || [])"),
        "`.multiple` writes the list: {js}"
    );
    assert!(
        js.contains("($$e.target.files && $$e.target.files[0]) || null"),
        "one file, or none, otherwise: {js}"
    );
    // And the state is held to what it gets.
    let wrong = src.replace("FileUpload(bind: photo", "FileUpload(bind: files");
    let (program, _) = webfluent::syntax::parse_source_recovering(&wrong, "t.wf");
    let found = format!(
        "{:?}",
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(wrong.clone()),
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics
    );
    assert!(found.contains("F02"), "{found}");
}

#[test]
fn a_setting_after_a_bare_result_ends_the_type() {
    let src = "api Lab(base: \"/api\") {\n    get version at \"version\" -> String! as: .text\n}\napp { Router }\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    resource v = Lab.version()\n    Heading(\"Home\").h1\n    Text(v.value ?? \"\")\n}\n";
    let (_, errors) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(errors.is_empty(), "parses: {errors:?}");
}

#[test]
fn render_data_for_a_read_is_held_to_its_endpoint() {
    let src = "type User { id: String, name: String }\napi Backend(base: \"/api\") {\n    get users at \"users\" -> [User]!\n}\npage Users(path: \"/\", title: \"Users\") {\n    resource users = Backend.users()\n    match users {\n        .loaded(list) { for u in list by u.id { Text(u.name) } }\n        else { Text(\"Loading\") }\n    }\n}\n";
    let tpl = webfluent::Template::from_str(src).expect("checks");
    let good = tpl
        .render_html_fragment(
            &serde_json::json!({ "users": { "loaded": [{ "id": "1", "name": "Ada" }] } }),
        )
        .expect("renders");
    assert!(good.contains(">Ada<"), "{good}");
    let refused = tpl
        .render_html_fragment(&serde_json::json!({ "users": { "loaded": [{ "id": 1 }] } }))
        .expect_err("refused")
        .to_string();
    assert!(refused.contains("users.loaded[0].id"), "{refused}");
}

fn scratch(why: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-hands-on-{why}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    for (path, text) in files {
        std::fs::write(dir.join(path), text).unwrap();
    }
    dir
}

#[test]
fn migrate_writes_json_only_where_it_lowers_the_errors() {
    let dir = scratch(
        "json",
        &[
            (
                "webfluent.app.json",
                r#"{ "name": "json", "build": { "output": "../wf-hands-on-out" } }"#,
            ),
            (
                "src/App.wf",
                "app { Router }\n\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    state raw: Map = {}\n    Heading(\"H\").h1\n    Text(JSON.stringify(raw))\n}\n",
            ),
            (
                "src/Card.wf",
                "component Card(info: Map) {\n    Text(\"{info.name} {info.role} {info.team}\")\n}\n",
            ),
        ],
    );
    // A project whose output is outside it is still migrated: the step
    // writes no output.
    let done = seven::migrate(&dir, false).expect("migrates");
    let app = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    let card = std::fs::read_to_string(dir.join("src/Card.wf")).unwrap();
    assert!(app.contains("state raw: Json"), "helps here: {app}");
    assert!(
        card.contains("info: Map"),
        "a Map read as a record wants a type a person names: {card}"
    );
    assert!(done.left.iter().any(|d| d.code == "T24"), "{:?}", done.left);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_http_answer_is_held_to_the_type_its_let_declares() {
    let src = "type Row { id: String }\ntype Rows { items: [Row] }\napp { Router }\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    state n = 0\n    action pull() {\n        let answer: Rows! = await http.get(\"/api/rows\")\n        match answer { .ok(r) { n = r.items.length } .err(_) { n = -1 } }\n    }\n    action pull2() -> ! {\n        let r: Rows = try await http.get(\"/api/rows\")\n        n = r.items.length\n    }\n    Heading(\"Home\").h1\n    Button(\"Pull\") { on click { pull() } }\n    Button(\"Again\") { on click { pull2() } }\n    Text(\"{n}\")\n}\n";
    let js = spa_generated(src);
    assert_eq!(
        js.matches("shape: [\"r\", \"Rows\"").count(),
        2,
        "both answers are checked against `Rows`: {js}"
    );
}

#[test]
fn migrate_splices_a_number_joined_to_text() {
    let dir = scratch(
        "splice",
        &[
            ("webfluent.app.json", r#"{ "name": "splice" }"#),
            (
                "src/App.wf",
                "type F { add: Number, del: Number }\napp { Router }\n\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    state fs: [F] = []\n    derived files = fs.map(f => { add: \"+\" + f.add, del: \"−\" + f.del })\n    Heading(\"H\").h1\n    Text(\"{files.length}\")\n}\n",
            ),
        ],
    );
    let done = seven::migrate(&dir, false).expect("migrates");
    let text = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    assert!(
        text.contains("add: \"+{f.add}\", del: \"−{f.del}\""),
        "each join is the text it means: {text}"
    );
    assert_eq!(done.by_code.get("T18"), Some(&2), "{:?}", done.by_code);
    assert!(done.left.is_empty(), "{:?}", done.left);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_tone_s_label_and_a_modal_s_width_follow_the_theme() {
    let dir = scratch(
        "theme",
        &[
            (
                "webfluent.app.json",
                r#"{ "name": "theme", "build": { "minify": false } }"#,
            ),
            (
                "src/App.wf",
                "theme Night {\n    color-background: #0B1220\n    color-text: #E5E7EB\n    color-primary: #FDE68A\n    color-on-primary: #1A1300\n}\napp { Router }\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    state open = false\n    Heading(\"H\").h1\n    Button(\"Go\").primary\n    Badge(\"New\").success\n    Alert(\"Saved\").success\n    Modal(visible: open, title: \"Wide\") { Text(\"x\") }\n}\n",
            ),
        ],
    );
    webfluent::build::build(&webfluent::build::BuildOptions::new(
        &dir,
        &webfluent::vfs::FsVfs,
    ))
    .expect("builds");
    let css = std::fs::read_to_string(dir.join("build/styles.css")).unwrap();
    assert!(
        css.contains("--color-on-primary: #1A1300") || css.contains("--color-on-primary:#1A1300"),
        "the theme's label colour is a token: {css}"
    );
    for (class, token) in [
        ("wf-btn--primary", "color-on-primary"),
        ("wf-badge--success", "color-on-success"),
    ] {
        let rule = css
            .split('}')
            .find(|r| r.contains(&format!(".{class}")) && r.contains("background"))
            .unwrap_or_default();
        assert!(
            rule.contains(&format!("var(--{token})")),
            ".{class} writes in {token}: {rule}"
        );
    }
    let alert = css
        .split('}')
        .find(|r| r.contains(".wf-alert--success"))
        .unwrap_or_default();
    assert!(
        alert.contains("color-mix") && !alert.contains('#'),
        "an alert is mixed from the theme: {alert}"
    );
    let modal = css
        .split('}')
        .find(|r| r.contains(".wf-modal__content {") || r.contains(".wf-modal__content{"))
        .unwrap_or_default();
    assert!(
        !modal.contains("max-width: 500px") && !modal.contains("max-width:500px"),
        "the content fills the dialog: {modal}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_check_of_a_project_whose_output_is_outside_it_checks_everything() {
    let dir = scratch(
        "outside",
        &[
            (
                "webfluent.app.json",
                r#"{ "name": "outside", "build": { "output": "../elsewhere" } }"#,
            ),
            (
                "src/App.wf",
                "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    Heading(\"H\").h1\n    Text(nmae)\n}\n",
            ),
        ],
    );
    let found = match webfluent::build::check(&webfluent::build::BuildOptions::new(
        &dir,
        &webfluent::vfs::FsVfs,
    )) {
        Ok(report) => format!("{:?}", report.lines),
        Err(failure) => format!("{:?}\n{failure}", failure.report.lines),
    };
    assert!(found.contains("a build would stop here"), "{found}");
    assert!(found.contains("T13"), "the program is checked too: {found}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn migrate_carries_every_row_of_the_tables_and_changes_no_catch() {
    let dir = scratch(
        "messy",
        &[
            ("webfluent.app.json", r#"{ "name": "messy" }"#),
            ("src/App.wf", include_str!("fixtures/migrate7/messy-6.wf")),
        ],
    );
    let done = seven::migrate(&dir, false).expect("migrates");
    let text = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    for written in [
        "enum UserError {\n    notFound(problem: Problem404) on 404\n    http(error: HttpError)\n}",
        "get user(id: String) at \"users/{id}\" -> User ! UserError",
        "-> [User]! pages .number(param: page)",
        "resource feed = Backend.feed()",
        "try await http.post(\"/api/notes\", body: { text: text })",
        "if users.isLoaded {",
    ] {
        assert!(text.contains(written), "`{written}` in:\n{text}");
    }
    assert!(!text.contains("errors {"), "{text}");
    // Inside `try { } catch`, `try` would skip the catch: left for a person.
    assert!(
        text.contains("let u: User = await http.get(\"/api/users/1\")"),
        "{text}"
    );
    assert!(
        done.left.iter().any(
            |d| d.code == "T01" && d.hint.as_deref().is_some_and(|h| h.contains("match await"))
        ),
        "{:?}",
        done.left
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn errors_of(src: &str) -> Vec<String> {
    let (program, _) = webfluent::syntax::parse_source_recovering(src, "t.wf");
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
    .into_iter()
    .filter(|d| d.severity == webfluent::diagnostics::Severity::Error)
    .map(|d| format!("[{}] {}", d.code, d.message))
    .collect()
}

#[test]
fn a_ref_on_a_component_call_is_its_root() {
    let js = spa_generated(
        "component Launch(label: String) { Button(label) }\napp { Router }\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    state open = false\n    Heading(\"H\").h1\n    Launch(label: \"Menu\", ref: launcher) { on click { open = true } }\n    Menu(visible: open, anchor: launcher, label: \"Actions\") { Menu.Item { Text(\"One\") } }\n}\n",
    );
    assert!(
        !js.contains("ref:launcher") && !js.contains("ref: launcher"),
        "not a prop: {js}"
    );
    assert!(
        js.contains("launcher.el = _r"),
        "the handle takes the root: {js}"
    );
}

#[test]
fn an_optimistic_change_is_typed_by_its_holder() {
    let src = "type Todo { id: String, title: String }\nstore Todos {\n    state items: [Todo] = []\n    action hide(id: String) {\n        optimistic(items, list => list.filter(i => i.id != id))\n    }\n}\napp { Router }\npage Home(path: \"/\", title: \"H\", description: \"d\") { Heading(\"H\").h1 }\n";
    assert_eq!(errors_of(src), Vec::<String>::new());
    let wrong = src.replace("list.filter(i => i.id != id)", "list.length");
    assert!(
        errors_of(&wrong).iter().any(|e| e.contains("optimistic")),
        "{:?}",
        errors_of(&wrong)
    );
}

#[test]
fn a_structural_project_s_popover_has_its_theme_s_surface() {
    let dir = scratch(
        "structural",
        &[
            (
                "webfluent.app.json",
                r#"{ "name": "s", "theme": { "builtin": "structural" }, "build": { "minify": false } }"#,
            ),
            (
                "src/App.wf",
                "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    state open = false\n    Heading(\"H\").h1\n    Button(\"Why\", ref: why) { on click { open = true } }\n    Popover(visible: open, anchor: why, label: \"Why\") { Text(\"Because.\") }\n    Button(\"Toast\") { on click { toast(\"Saved\") } }\n}\n",
            ),
        ],
    );
    webfluent::build::build(&webfluent::build::BuildOptions::new(
        &dir,
        &webfluent::vfs::FsVfs,
    ))
    .expect("builds");
    let css = std::fs::read_to_string(dir.join("build/styles.css")).unwrap();
    let rule = css
        .split('}')
        .find(|r| r.contains(".wf-popover {") || r.contains(".wf-popover{"))
        .unwrap_or_default();
    assert!(rule.contains("var(--color-surface)"), "{rule}");
    let _ = std::fs::remove_dir_all(&dir);
}

// What writing an example for every built-in turned up, kept from coming
// back: each was a page that compiled and then went wrong in the browser,
// or a finding about nothing.

fn home(body: &str) -> String {
    format!(
        "app {{ Router }}\npage Home(path: \"/\", title: \"H\", description: \"d\") {{\n    Heading(\"H\").h1\n{body}\n}}\n"
    )
}

#[test]
fn a_text_inside_a_text_is_a_run_of_its_line() {
    let src = home("    Text { Text(\"Press \")  Kbd(\"mod+k\")  Text(\" to search\") }");
    let html = crate::common::ssg_html(&src);
    assert!(
        html.contains("<span class=\"wf-text\">Press </span>"),
        "the inner Text is a span in the static paint: {html}"
    );
    assert!(!html.contains("<p class=\"wf-text\">Press"), "{html}");
    let js = spa_generated(&src);
    assert!(js.contains("WF.el(\"span\""), "and on the live page: {js}");
}

#[test]
fn log_as_a_value_is_the_consoles() {
    let js = spa_generated(&home(
        "    Host(tag: \"canvas\", mount: (n) => n.getBoundingClientRect(), update: (b) => log(\"moved\"))",
    ));
    assert!(js.contains("console.log(\"moved\")"), "{js}");
    assert!(
        !js.contains(" log(\"moved\")") && !js.contains("(log(\"moved\")"),
        "{js}"
    );
}

#[test]
fn a_ref_on_a_part_takes_the_part() {
    let js = spa_generated(&home(
        "    Card { Card.Body(ref: panel) { Text(\"Inside\") } }\n    Portal(into: panel) { Text(\"Drawn in the body\") }",
    ));
    assert!(
        js.contains("ref: panel"),
        "the part hands itself to the handle: {js}"
    );
}

#[test]
fn a_page_number_on_a_web_page_is_one() {
    let js = spa_generated(&home(
        "    Document(title: \"Invoice\") {\n        Footer { Text(\"Page {page} of {pages}\") }\n        Heading(\"Invoice\").h2\n    }",
    ));
    assert!(!js.contains("_page()") && !js.contains("_pages()"), "{js}");
}

#[test]
fn validate_inside_an_element_is_refused_where_it_is_written() {
    let (_, errors) = webfluent::syntax::parse_source_recovering(
        &home(
            "    state email = \"\"\n    Stack {\n        validate email { email }\n        Input(bind: email, label: \"Email\").email\n    }",
        ),
        "t.wf",
    );
    let said: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();
    assert!(
        said.iter()
            .any(|e| e.contains("E105") && e.contains("validate")),
        "{said:?}"
    );
}

#[test]
fn a_to_that_is_not_a_route_is_not_held_to_one() {
    let errors = errors_of(&home(
        "    state picked: [String] = []\n    TransferList(bind: picked, options: [\"a\", \"b\"], text: o => o, key: o => o, label: \"Reviewers\", to: \"Chosen reviewers\")",
    ));
    assert!(!errors.iter().any(|e| e.starts_with("[R01]")), "{errors:?}");
}

#[test]
fn a_button_named_only_for_a_screen_reader_has_text() {
    let (program, _) = webfluent::syntax::parse_source_recovering(
        &home("    Button { Icon(\"trash\")  VisuallyHidden(\"Delete the draft\") }"),
        "t.wf",
    );
    let found = webfluent::linter::lint_accessibility_in(&program, &|_| "t.wf".to_string());
    assert!(!found.iter().any(|d| d.code == "A05"), "{found:?}");
}

#[test]
fn a_callouts_text_is_markdown_and_its_markup_is_shown() {
    let html = crate::common::ssg_html(&home(
        "    Callout(\"Run `wf check` first, then [read the guide](/).\").tip\n    Callout(\"Plain <b>markup</b>\").warning",
    ));
    assert!(
        html.contains("<code>wf check</code>"),
        "code shows as code: {html}"
    );
    assert!(html.contains("href=\"/\""), "a link is a link: {html}");
    assert!(
        html.contains("&lt;b&gt;markup&lt;/b&gt;"),
        "markup is shown, never run: {html}"
    );
}

#[test]
fn a_sortable_header_is_drawn_as_the_others_are() {
    let css = webfluent::themes::component_css();
    assert!(
        css.contains(".wf-data-table__sort {") && css.contains("text-transform: inherit"),
        "a sortable header's button takes the header's case: {css}"
    );
}
