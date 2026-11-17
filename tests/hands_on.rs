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
