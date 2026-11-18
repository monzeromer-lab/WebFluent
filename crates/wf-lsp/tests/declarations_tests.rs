//! What the server knows of every kind of declaration — `middleware`,
//! `fake`, `example` as much as a page — of a service's groups and
//! contract words, of a test file's project, and of the browser's globals.
//!
//! Each test names what used to go wrong.

use std::path::PathBuf;

use lsp_types::*;
use webfluent::vfs::FsVfs;
use wf_lsp::completion::provide_completions;
use wf_lsp::definition::find_definition;
use wf_lsp::hover::provide_hover;
use wf_lsp::project::{FileCache, Project};
use wf_lsp::symbols::{document_symbols, workspace_symbols};

fn project(src: &str) -> Project {
    Project::single(Url::parse("file:///test.wf").expect("a valid URL"), src)
}

fn position(project: &Project, src: &str, offset: usize) -> Position {
    project.files[0].index.offset_to_position(src, offset)
}

/// The completions just after `needle`.
fn labels_after(src: &str, needle: &str) -> Vec<String> {
    items_after(src, needle)
        .into_iter()
        .map(|c| c.label)
        .collect()
}

fn items_after(src: &str, needle: &str) -> Vec<CompletionItem> {
    let project = project(src);
    let offset = src.find(needle).expect("the needle is in the source") + needle.len();
    provide_completions(&project, 0, position(&project, src, offset))
}

/// A buffer mid-edit: `broken` is the text, `valid` the last parse of it
/// that succeeded — what the server holds while the user types.
fn labels_mid_edit(valid: &str, broken: &str, needle: &str) -> Vec<String> {
    let mut project = project(valid);
    project.files[0].source = broken.into();
    project.files[0].index = wf_lsp::line_index::LineIndex::new(broken);
    project.files[0].parsed =
        Project::single(Url::parse("file:///x.wf").expect("a valid URL"), broken)
            .files
            .remove(0)
            .parsed;
    project.files[0].stale = project.files[0].parsed.is_err();
    let offset = broken.find(needle).expect("the needle is in the source") + needle.len();
    let pos = project.files[0].index.offset_to_position(broken, offset);
    provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| c.label)
        .collect()
}

fn hover_text(src: &str, needle: &str) -> Option<String> {
    let project = project(src);
    let offset = src.find(needle).expect("the needle is in the source");
    provide_hover(&project, 0, position(&project, src, offset)).map(|h| match h.contents {
        HoverContents::Markup(m) => m.value,
        _ => panic!("markdown expected"),
    })
}

/// The line the definition of the word at `needle` (its `nth` occurrence)
/// lands on, 0-based.
fn definition_line(src: &str, needle: &str, nth: usize) -> Option<u32> {
    let project = project(src);
    let offset = src
        .match_indices(needle)
        .nth(nth)
        .expect("the needle is in the source")
        .0;
    match find_definition(&project, 0, position(&project, src, offset))? {
        GotoDefinitionResponse::Scalar(l) => Some(l.range.start.line),
        _ => panic!("one location expected"),
    }
}

fn line_of(src: &str, needle: &str) -> u32 {
    let at = src.find(needle).expect("the needle is in the source");
    u32::try_from(src[..at].matches('\n').count()).expect("a short source")
}

const SERVICE: &str = r#"type Todo { id: String, title: String }
type NewTodo { title: String }

middleware timed(req, next) {
    let answer = await next(req)
    return answer
}

api Backend(base: "/api") {
    middleware [timed]
}

api Backend.todos {
    get list() at "todos" -> [Todo]!
    get byId(id: String) at "todos/{id}" -> Todo!
    post create(body: NewTodo) at "todos" -> Todo!
        invalidates [list]
}

fake OneTodo for Backend.todos {
    list() { .ok([Todo(id: "1", title: "Milk")]) }

}

component TodoCard(_ title: String) {
    Text(title)
}

example TodoCard "A short one" {
    TodoCard("Milk")
}

store Todos {
    state items: [Todo] = []
    action add(title: String) {
        let made = await Backend.todos.create(body: NewTodo(title: title))

    }
}

test "lists the todos"(fakes: [OneTodo]) {
    TodoCard("Milk")
    expect "Milk"
}
"#;

// ─── Every kind of declaration ────────────────────────────────────────────

#[test]
fn a_middlewares_body_is_a_handlers_with_its_request_and_chain() {
    let labels = labels_after(SERVICE, "let answer = await next(req)\n");
    assert!(labels.contains(&"req".to_string()), "{labels:?}");
    assert!(labels.contains(&"next".to_string()), "{labels:?}");
    assert!(labels.contains(&"return".to_string()), "{labels:?}");
    // Not the top level's declarations.
    assert!(!labels.contains(&"page".to_string()), "{labels:?}");
}

#[test]
fn a_fakes_block_offers_the_endpoints_it_does_not_answer_yet() {
    let items = items_after(SERVICE, "title: \"Milk\")]) }\n");
    let labels: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();
    assert!(labels.contains(&"byId"), "{labels:?}");
    assert!(labels.contains(&"create"), "{labels:?}");
    assert!(!labels.contains(&"list"), "already answered: {labels:?}");
    assert!(!labels.contains(&"page"), "{labels:?}");
    let by_id = items.iter().find(|c| c.label == "byId").unwrap();
    assert_eq!(
        by_id.insert_text.as_deref(),
        Some("byId(id) { .ok($0) }"),
        "written as an answer, with the endpoint's parameters"
    );
}

#[test]
fn an_examples_body_is_a_render_body_without_a_head() {
    let labels = labels_after(SERVICE, "    TodoCard(\"Milk\")\n");
    assert!(labels.contains(&"TodoCard".to_string()), "{labels:?}");
    assert!(labels.contains(&"Button".to_string()), "{labels:?}");
    assert!(labels.contains(&"state".to_string()), "{labels:?}");
    assert!(!labels.contains(&"head".to_string()), "{labels:?}");
    assert!(!labels.contains(&"page".to_string()), "{labels:?}");
}

#[test]
fn the_outline_and_the_workspace_list_middleware_fakes_and_examples() {
    let project = project(SERVICE);
    let DocumentSymbolResponse::Nested(symbols) = document_symbols(&project, 0, true) else {
        panic!("nested symbols expected");
    };
    let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"timed"), "{names:?}");
    assert!(names.contains(&"OneTodo"), "{names:?}");
    assert!(
        names.contains(&"example TodoCard \"A short one\""),
        "{names:?}"
    );
    let fake = symbols.iter().find(|s| s.name == "OneTodo").unwrap();
    assert_eq!(fake.children.as_ref().map(Vec::len), Some(1), "its answer");

    let all: Vec<String> = workspace_symbols(&project, "")
        .into_iter()
        .map(|s| s.name)
        .collect();
    for name in ["timed", "OneTodo", "example TodoCard \"A short one\""] {
        assert!(all.iter().any(|n| n == name), "{name} in {all:?}");
    }
}

#[test]
fn definition_reaches_a_middleware_a_fake_and_an_examples_component() {
    assert_eq!(
        definition_line(SERVICE, "timed", 1),
        Some(line_of(SERVICE, "middleware timed")),
        "`middleware [timed]`"
    );
    assert_eq!(
        definition_line(SERVICE, "OneTodo", 1),
        Some(line_of(SERVICE, "fake OneTodo")),
        "`fakes: [OneTodo]`"
    );
    assert_eq!(
        definition_line(SERVICE, "TodoCard \"A short one\"", 0),
        Some(line_of(SERVICE, "component TodoCard")),
        "`example TodoCard`"
    );
    // A fake's answer is the endpoint it answers.
    assert_eq!(
        definition_line(SERVICE, "list() { .ok", 0),
        Some(line_of(SERVICE, "get list()")),
    );
}

#[test]
fn references_and_rename_of_a_component_include_an_examples_header() {
    let project = project(SERVICE);
    let offset = SERVICE.find("component TodoCard").unwrap() + "component ".len();
    let at = position(&project, SERVICE, offset);
    let found = wf_lsp::rename::references(&project, 0, at, true);
    let example_line = line_of(SERVICE, "example TodoCard");
    assert!(
        found.iter().any(|l| l.range.start.line == example_line),
        "{found:?}"
    );
    let edit = wf_lsp::rename::rename(&project, 0, at, "TaskCard").expect("a rename");
    let edits = edit.changes.unwrap().into_values().next().unwrap();
    assert!(
        edits.iter().any(|e| e.range.start.line == example_line),
        "{edits:?}"
    );
}

#[test]
fn a_list_of_middleware_offers_the_projects_and_the_runtimes() {
    let src = SERVICE.replace("middleware [timed]", "middleware [timed, ]");
    let labels = labels_mid_edit(SERVICE, &src, "middleware [timed, ");
    for name in [
        "timed",
        "bearer",
        "header",
        "refreshOn401",
        "requestId",
        "log",
    ] {
        assert!(labels.contains(&name.to_string()), "{name} in {labels:?}");
    }
    assert!(!labels.contains(&"Button".to_string()), "{labels:?}");
}

#[test]
fn a_tests_fakes_list_offers_the_projects_fakes() {
    let src = SERVICE.replace("fakes: [OneTodo]", "fakes: []");
    let labels = labels_mid_edit(SERVICE, &src, "fakes: [");
    assert_eq!(labels, vec!["OneTodo".to_string()]);
}

#[test]
fn invalidates_offers_the_groups_reads_and_resolves_to_them() {
    let src = SERVICE.replace("invalidates [list]", "invalidates [list, ]");
    let labels = labels_mid_edit(SERVICE, &src, "invalidates [list, ");
    assert!(labels.contains(&"list".to_string()), "{labels:?}");
    assert!(labels.contains(&"byId".to_string()), "{labels:?}");
    assert!(
        !labels.contains(&"create".to_string()),
        "a write: {labels:?}"
    );
    assert_eq!(
        definition_line(SERVICE, "list]", 0),
        Some(line_of(SERVICE, "get list()"))
    );
}

#[test]
fn hover_explains_the_words_of_a_contract() {
    let src = r#"type Todo { id: String }
type Line { text: String }
api Backend(base: "/api") {
    get list(page: Number = 1) at "todos" -> [Todo]!
        pages .number(param: page)
    get tail(id: String) at "logs/{id}" -> [Line]! streamed
    put save(body: Todo) at "todos" -> Todo! idempotent
        invalidates [list]
}
store S {
    action poll() concurrent { }
}
middleware stamp(req, next) {
    return await next(req)
}
"#;
    for (needle, says) in [
        ("at \"todos\"", "path"),
        ("pages .number", "page at a time"),
        ("streamed", "a line"),
        ("idempotent", "twice"),
        ("invalidates", "stale"),
        ("concurrent", "every call"),
        ("middleware stamp", "pass through"),
    ] {
        let text = hover_text(src, needle).unwrap_or_else(|| panic!("a hover on {needle}"));
        assert!(text.contains(says), "{needle}: {text}");
    }
    let example = hover_text(SERVICE, "example TodoCard").expect("a hover on `example`");
    assert!(example.contains("wf docs"), "{example}");
}

#[test]
fn the_top_level_offers_example_and_middleware_snippets_that_compile() {
    let items = items_after("\n", "\n");
    let snippet = |label: &str| {
        items
            .iter()
            .find(|c| c.label == label)
            .and_then(|c| c.insert_text.clone())
            .unwrap_or_else(|| panic!("{label} offered"))
    };
    assert_eq!(
        snippet("example"),
        "example ${1:Component} \"${2:What it shows}\" {\n\t$1($0)\n}"
    );
    assert!(snippet("middleware").contains("(req, next)"));
    // 7.0's forms: a typed state, an endpoint `at` a path that says how it
    // fails, no `Map`.
    let store = snippet("store");
    assert!(store.contains("state ${2:items}: "), "{store}");
    let api = snippet("api");
    assert!(api.contains(" at \"") && api.contains("!\n"), "{api}");
    assert!(!api.contains("Map"), "{api}");
}

#[test]
fn example_then_a_space_offers_the_projects_components() {
    let valid = "component Card2(_ t: String) { Text(t) }\n";
    let src = format!("{valid}example ");
    let labels = labels_mid_edit(valid, &src, "example ");
    assert_eq!(labels, vec!["Card2".to_string()]);
}

// ─── A service's groups ───────────────────────────────────────────────────

#[test]
fn a_call_of_a_groups_endpoint_goes_to_the_endpoint_in_the_group() {
    assert_eq!(
        definition_line(SERVICE, "create(body: NewTodo(title", 0),
        Some(line_of(SERVICE, "post create")),
    );
    // `Backend.todos`: the group.
    assert_eq!(
        definition_line(SERVICE, "todos.create(body: NewTodo(title", 0),
        Some(line_of(SERVICE, "api Backend.todos")),
    );
}

#[test]
fn a_service_offers_its_groups_and_an_endpoint_what_the_checker_types() {
    let src = SERVICE.replace(
        "NewTodo(title: title))\n\n",
        "NewTodo(title: title))\n        Backend.\n",
    );
    let labels = labels_mid_edit(SERVICE, &src, "        Backend.");
    assert!(labels.contains(&"todos".to_string()), "{labels:?}");

    let src = SERVICE.replace(
        "NewTodo(title: title))\n\n",
        "NewTodo(title: title))\n        Backend.todos.list.\n",
    );
    let labels = labels_mid_edit(SERVICE, &src, "Backend.todos.list.");
    for member in ["update", "invalidate", "prefetch", "key", "url", "pending"] {
        assert!(
            labels.contains(&member.to_string()),
            "{member} in {labels:?}"
        );
    }
    assert!(!labels.contains(&"lines".to_string()), "{labels:?}");
}

#[test]
fn a_service_body_offers_70s_forms_and_a_group_only_endpoints() {
    let src = "api Backend(base: \"/api\") {\n\n}\napi Backend.todos {\n\n}\n";
    let service = items_after(src, "\"/api\") {\n");
    let labels: Vec<&str> = service.iter().map(|c| c.label.as_str()).collect();
    for word in [
        "middleware",
        "timeout",
        "retry",
        "credentials",
        "invalidations",
        "pagination",
        "get",
    ] {
        assert!(labels.contains(&word), "{word} in {labels:?}");
    }
    assert!(!labels.contains(&"on"), "6.x hooks: {labels:?}");
    let get = service.iter().find(|c| c.label == "get").unwrap();
    assert_eq!(
        get.insert_text.as_deref(),
        Some("get ${1:name}(${2}) at \"${3:path}\" -> ${4:Item}!")
    );

    let group = labels_after(src, "Backend.todos {\n");
    assert!(group.contains(&"get".to_string()), "{group:?}");
    assert!(!group.contains(&"timeout".to_string()), "{group:?}");
    assert!(!group.contains(&"middleware".to_string()), "{group:?}");

    let after = "type T { id: String }\napi B(base: \"/\") {\n    post make(body: T) at \"t\" -> T!\n    \n}\n";
    let labels = labels_after(after, "-> T!\n    ");
    for word in ["idempotent", "streamed", "invalidates", "pages .cursor"] {
        assert!(labels.contains(&word.to_string()), "{word} in {labels:?}");
    }
}

// ─── What a body may say ──────────────────────────────────────────────────

#[test]
fn an_actions_body_offers_70s_functions_and_the_programs_names() {
    let items = items_after(SERVICE, "NewTodo(title: title))\n");
    let labels: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();
    for name in [
        "http",
        "toast",
        "ask",
        "announce",
        "match",
        "for await",
        "Backend",
        "Todos",
        "NewTodo",
        "localStorage",
        "setTimeout",
        "Math",
    ] {
        assert!(labels.contains(&name), "{name} in {labels:?}");
    }
    assert!(!labels.contains(&"fetch"), "gone in 7.0: {labels:?}");
    assert!(
        !labels.contains(&"Backend.todos"),
        "a group is reached through its service"
    );
    for once in ["navigate", "log"] {
        assert_eq!(
            labels.iter().filter(|l| **l == once).count(),
            1,
            "{once} once: {labels:?}"
        );
    }
}

#[test]
fn an_action_has_its_result_beside_pending() {
    let valid = "store S {\n    action save() { }\n    action go() {\n        save;\n    }\n}\n";
    let labels = labels_mid_edit(valid, &valid.replace("save;", "save."), "save.");
    assert!(labels.contains(&"pending".to_string()), "{labels:?}");
    assert!(labels.contains(&"result".to_string()), "{labels:?}");
}

#[test]
fn a_test_body_offers_the_expect_forms_and_no_head() {
    let labels = labels_after(SERVICE, "    expect \"Milk\"\n");
    for word in [
        "expect",
        "expect not",
        "expect focus",
        "expect called",
        "click",
    ] {
        assert!(labels.contains(&word.to_string()), "{word} in {labels:?}");
    }
    assert!(!labels.contains(&"head".to_string()), "{labels:?}");
}

#[test]
fn the_browsers_objects_have_their_members_after_a_dot() {
    let src = "page P(path: \"/\") {\n    Button(\"Go\") { on click { localStorage. } }\n}\n";
    let labels = labels_after(src, "localStorage.");
    assert!(labels.contains(&"getItem".to_string()), "{labels:?}");
    assert!(labels.contains(&"setItem".to_string()), "{labels:?}");

    let src = "page P(path: \"/\") {\n    Button(\"Go\") { on click { let n = Math. } }\n}\n";
    let labels = labels_after(src, "Math.");
    assert!(labels.contains(&"floor".to_string()), "{labels:?}");

    let src =
        "page P(path: \"/\") {\n    Button(\"Go\") { on click { window.localStorage. } }\n}\n";
    let labels = labels_after(src, "window.localStorage.");
    assert!(labels.contains(&"removeItem".to_string()), "{labels:?}");

    let src = "page P(path: \"/\") {\n    Button(\"Go\") { on click { http. } }\n}\n";
    let labels = labels_after(src, "http.");
    assert_eq!(labels, ["get", "post", "put", "patch", "delete"]);
}

#[test]
fn the_browser_table_is_the_compilers() {
    let table = wf_lsp::browser::table();
    let storage = table.object_of_chain(&["localStorage"]);
    assert_eq!(storage, Some("Storage"));
    // Every global the table types is one the compiler lets a program name.
    for g in &table.globals {
        assert!(
            webfluent::codegen::js::BROWSER_GLOBALS.contains(&g.name.as_str())
                || ["isNaN", "isFinite"].contains(&g.name.as_str()),
            "{} is not one of the compiler's globals",
            g.name
        );
    }
    assert!(table.globals.len() > 20);
}

#[test]
fn the_overlay_functions_are_the_ones_the_checker_takes() {
    for name in wf_lsp::completion::OVERLAY_FUNCTIONS {
        let src = format!(
            "page P(path: \"/\", title: \"P\", description: \"P.\") {{\n    Heading(\"P\").h1\n    Button(\"Go\") {{ on click {{ {name}(\"x\") }} }}\n}}\n"
        );
        let found = wf_lsp::diagnostics::project_diagnostics(&project(&src)).remove(0);
        assert!(
            !found
                .iter()
                .any(|d| d.code == Some(NumberOrString::String("T13".to_string()))),
            "{name}: {found:?}"
        );
    }
}

#[test]
fn a_let_in_an_action_is_given_its_type() {
    let src = "store S {\n    state n = 1\n    action go() {\n        let twice = n * 2\n        n = twice\n    }\n}\n";
    let project = project(src);
    let hints = wf_lsp::inlay::inlay_hints(
        &project,
        0,
        Range::new(Position::new(0, 0), Position::new(10, 0)),
    );
    let labels: Vec<(u32, String)> = hints
        .into_iter()
        .map(|h| match h.label {
            InlayHintLabel::String(s) => (h.position.line, s),
            InlayHintLabel::LabelParts(_) => panic!("a string label"),
        })
        .collect();
    assert!(labels.contains(&(3, ": Number".to_string())), "{labels:?}");
}

// ─── A project on disk ────────────────────────────────────────────────────

fn on_disk(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-lsp-decls-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for sub in ["src/components", "src/translations", "tests"] {
        std::fs::create_dir_all(dir.join(sub)).expect("create the fixture's folder");
    }
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "fixture", "i18n": { "default_locale": "en", "locales": ["en"], "dir": "src/translations" } }"#,
    )
    .expect("write the fixture");
    std::fs::write(
        dir.join("src/translations/en.json"),
        "{\n  \"nav.home\": \"Home\",\n  \"greeting\": \"Hello\"\n}\n",
    )
    .expect("write the fixture");
    std::fs::write(
        dir.join("src/components/Greeting.wf"),
        "theme Brand {\n    color-primary: #0F766E\n}\n\ncomponent Greeting(_ name: String) {\n    Text(t(\"greeting\")) {\n        style { color: $color-primary }\n    }\n}\n",
    )
    .expect("write the fixture");
    std::fs::write(
        dir.join("tests/greeting.wf"),
        "test \"greets\" {\n    Greeting(\"Sam\")\n    expect \"Hello\"\n}\n",
    )
    .expect("write the fixture");
    dir
}

fn definition_in(path: &std::path::Path, needle: &str) -> Option<Location> {
    let uri = Url::from_file_path(path).expect("an absolute path");
    let project = Project::load_with(&uri, &FsVfs, &FileCache::default());
    let ix = project
        .file_index(&uri)
        .expect("the file is in its project");
    let src = std::sync::Arc::clone(&project.files[ix].source);
    let offset = src.find(needle).expect("the needle is in the source");
    let at = project.files[ix].index.offset_to_position(&src, offset);
    match find_definition(&project, ix, at)? {
        GotoDefinitionResponse::Scalar(l) => Some(l),
        _ => panic!("one location expected"),
    }
}

#[test]
fn a_test_file_is_read_with_the_project_it_belongs_to() {
    let dir = on_disk("tests");
    let test = dir.join("tests/greeting.wf");
    let found = definition_in(&test, "Greeting(\"Sam\")").expect("the component");
    assert!(
        found.uri.path().ends_with("src/components/Greeting.wf"),
        "{found:?}"
    );

    let uri = Url::from_file_path(&test).expect("an absolute path");
    let project = Project::load_with(&uri, &FsVfs, &FileCache::default());
    let ix = project.file_index(&uri).unwrap();
    let src = std::sync::Arc::clone(&project.files[ix].source);
    let offset = src.find("    expect").unwrap();
    let at = project.files[ix].index.offset_to_position(&src, offset);
    let labels: Vec<String> = provide_completions(&project, ix, at)
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert!(labels.contains(&"Greeting".to_string()), "{labels:?}");
    let errors: Vec<Diagnostic> = wf_lsp::diagnostics::project_diagnostics(&project)
        .remove(ix)
        .into_iter()
        .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn a_message_key_and_a_design_token_go_to_where_they_are_declared() {
    let dir = on_disk("i18n");
    let component = dir.join("src/components/Greeting.wf");
    let message = definition_in(&component, "greeting\")").expect("the message");
    assert!(
        message.uri.path().ends_with("translations/en.json"),
        "{message:?}"
    );
    assert_eq!(message.range.start.line, 2);

    let token = definition_in(&component, "color-primary }").expect("the token");
    assert!(token.uri.path().ends_with("Greeting.wf"), "{token:?}");
    assert_eq!(token.range.start.line, 1);
}
