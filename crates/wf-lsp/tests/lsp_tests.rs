//! Behaviour of the server's features on small in-memory documents.
//!
//! Each test names the mistake it guards against: the ones a user notices as
//! "the language server is not accurate".

use tower_lsp::lsp_types::*;
use wf_lsp::completion::provide_completions;
use wf_lsp::definition::find_definition;
use wf_lsp::diagnostics::project_diagnostics;
use wf_lsp::hover::provide_hover;
use wf_lsp::project::Project;
use wf_lsp::symbols::document_symbols;

fn project(src: &str) -> Project {
    Project::single(Url::parse("file:///test.wf").unwrap(), src)
}

/// The position of `needle` in `src`, as the editor would send it.
fn at(src: &str, needle: &str) -> Position {
    let offset = src
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in source"));
    let file = project(src);
    file.files[0].index.offset_to_position(src, offset)
}

fn hover_text(src: &str, needle: &str) -> Option<String> {
    let project = project(src);
    provide_hover(&project, 0, at(src, needle)).map(|h| match h.contents {
        HoverContents::Markup(m) => m.value,
        _ => panic!("markdown expected"),
    })
}

fn labels_after(src: &str, needle: &str) -> Vec<String> {
    let project = project(src);
    let offset = src.find(needle).unwrap() + needle.len();
    let pos = project.files[0].index.offset_to_position(src, offset);
    provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| c.label)
        .collect()
}

/// A buffer mid-edit: `broken` is the text, `valid` the last parse of it that
/// succeeded — what the server holds while the user types.
fn mid_edit(valid: &str, broken: &str) -> Project {
    let mut project = project(valid);
    project.files[0].source = broken.into();
    project.files[0].index = wf_lsp::line_index::LineIndex::new(broken);
    project.files[0].parsed = Project::single(Url::parse("file:///x.wf").unwrap(), broken)
        .files
        .remove(0)
        .parsed;
    project.files[0].stale = project.files[0].parsed.is_err();
    project
}

fn labels_mid_edit(valid: &str, broken: &str, needle: &str) -> Vec<String> {
    let project = mid_edit(valid, broken);
    let offset = broken.find(needle).unwrap() + needle.len();
    let pos = project.files[0].index.offset_to_position(broken, offset);
    provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| c.label)
        .collect()
}

// ─── Hover ────────────────────────────────────────────────────────────────

#[test]
fn hover_on_a_builtin_says_what_the_registry_says() {
    let src =
        "page Home(path: \"/\") {\n    Column(span: 6) { Stack(gap: .md) { Spacer.sm } }\n}\n";
    let column = hover_text(src, "Column").unwrap();
    assert!(column.contains("12-column grid"), "{column}");
    assert!(column.contains("`span: …`"), "{column}");
    let stack = hover_text(src, "Stack").unwrap();
    assert!(stack.contains("Vertical flex"), "{stack}");
    let spacer = hover_text(src, "Spacer").unwrap();
    assert!(spacer.contains("Vertical space"), "{spacer}");
    assert!(spacer.contains("<div>"), "{spacer}");
    assert!(spacer.contains("`.sm`"), "{spacer}");
}

#[test]
fn hover_on_a_flag_names_the_prop_it_sets() {
    let src = "page Home(path: \"/\") {\n    Button(\"Save\").primary.lg\n}\n";
    let primary = hover_text(src, "primary").unwrap();
    assert!(
        primary.contains("`tone: .primary` on `Button`"),
        "{primary}"
    );
    let lg = hover_text(src, "lg\n").unwrap();
    assert!(lg.contains("`size: .lg` on `Button`"), "{lg}");
    assert!(lg.contains("`.sm`"), "the other cases are listed: {lg}");
}

#[test]
fn hover_on_a_prop_that_shares_a_flag_word_is_the_prop() {
    let src = "component Chip(text: String) {\n    Text(text).bold\n}\n";
    let text = hover_text(src, "text).bold").unwrap();
    assert!(text.contains("prop"), "{text}");
    assert!(!text.contains("Input"), "{text}");
}

#[test]
fn hover_inside_a_string_or_comment_is_nothing() {
    let src = "page Home(path: \"/\") {\n    // a Button in a comment\n    Text(\"Button\")\n}\n";
    assert!(hover_text(src, "Button in a").is_none());
    assert!(hover_text(src, "Button\")").is_none());
}

#[test]
fn hover_on_a_name_resolves_in_the_enclosing_declaration_not_the_first() {
    let src = "page A(path: \"/a\") {\n    state count = 1\n}\npage B(path: \"/b\") {\n    derived count = total * 2\n    Text(\"{count}\")\n}\n";
    let b_count = hover_text(src, "count = total").unwrap();
    assert!(b_count.contains("derived"), "{b_count}");
    assert!(b_count.contains("derived count = total * 2"), "{b_count}");
}

#[test]
fn hover_on_a_store_member_finds_the_store() {
    let src = "store CartStore {\n    state items = []\n    action clear() { items = [] }\n}\npage Shop(path: \"/\") {\n    use CartStore\n    Button(\"Clear\") { on click { CartStore.clear() } }\n}\n";
    let clear = hover_text(src, "clear() }").unwrap();
    assert!(clear.contains("action"), "{clear}");
    assert!(clear.contains("Member of store `CartStore`"), "{clear}");
    let store = hover_text(src, "CartStore.clear").unwrap();
    assert!(store.contains("store"), "{store}");
    assert!(store.contains("`items` — state"), "{store}");
}

#[test]
fn hover_on_a_prop_explains_it_for_that_component() {
    let src = "page Home(path: \"/\") {\n    Input(bind: name, aria-label: \"Name\", tone: .primary).text\n}\n";
    let bind = hover_text(src, "bind:").unwrap();
    assert!(bind.contains("of `Input`"), "{bind}");
    assert!(bind.contains("state variable"), "{bind}");
    let aria = hover_text(src, "aria-label").unwrap();
    assert!(aria.contains("attribute on `Input`"), "{aria}");
    let tone = hover_text(src, "tone:").unwrap();
    assert!(tone.contains("no prop called `tone`"), "{tone}");
}

#[test]
fn hover_on_a_case_a_token_an_event_and_a_selector() {
    let src = "theme T { line: #ccc }\npage Home(path: \"/\") {\n    Badge(\"x\", tone: .success)\n    Button(\"x\") {\n        style { border: 1px solid $line; padding: $md; &:hover { background: red } }\n        on click { go() }\n    }\n}\n";
    let case = hover_text(src, "success)").unwrap();
    assert!(case.contains("`tone` of `Badge`"), "{case}");
    assert!(case.contains("Green"), "{case}");
    let token = hover_text(src, "line; padding").unwrap();
    assert!(token.contains("design token of theme `T`"), "{token}");
    let short = hover_text(src, "md; &").unwrap();
    assert!(short.contains("short token name"), "{short}");
    assert!(short.contains("`$spacing-md`"), "{short}");
    let click = hover_text(src, "click {").unwrap();
    assert!(click.contains("event handler"), "{click}");
    let hover = hover_text(src, "hover {").unwrap();
    assert!(hover.contains("nested rule"), "{hover}");
    assert!(hover.contains("pointer"), "{hover}");
}

#[test]
fn hover_on_a_user_component_shows_its_props_events_and_slots() {
    let src = "component UserCard(_ name: String, active: Bool = true) {\n    event pick(id: String)\n    slot trailing\n    Text(name)\n    trailing\n}\npage Home(path: \"/\", title: \"Home\", layout: Shell) {\n    UserCard(\"x\") { on pick(id) { } }\n}\ncomponent Shell { slot  children }\n";
    let card = hover_text(src, "UserCard(\"x\")").unwrap();
    assert!(
        card.contains("component UserCard(_ name: String, active: Bool = …)"),
        "{card}"
    );
    assert!(card.contains("Events: `pick`"), "{card}");
    assert!(card.contains("Slots: `trailing`"), "{card}");
    let pick = hover_text(src, "pick(id) {").unwrap();
    assert!(pick.contains("event of `UserCard`"), "{pick}");
    assert!(pick.contains("event pick(id: String)"), "{pick}");
    let layout = hover_text(src, "Shell) {").unwrap();
    assert!(layout.contains("component Shell"), "{layout}");
}

#[test]
fn hover_with_arabic_text_on_the_line_lands_on_the_right_word() {
    let src = "page Welcome(path: \"/\") {\n    Text(\"مرحباً بك في WebFluent! 🚀\").muted\n    Button(\"ابدأ الآن\").primary\n}\n";
    let muted = hover_text(src, "muted").unwrap();
    assert!(muted.contains("`muted: true` on `Text`"), "{muted}");
    let primary = hover_text(src, "primary").unwrap();
    assert!(primary.contains("on `Button`"), "{primary}");
    // The hover's own range covers exactly the word.
    let project = project(src);
    let h = provide_hover(&project, 0, at(src, "primary")).unwrap();
    let r = h.range.unwrap();
    assert_eq!(r.end.character - r.start.character, "primary".len() as u32);
}

#[test]
fn hover_shows_the_type_the_checker_infers() {
    let src = "type Todo { id: String, title: String }\nstore S {\n    state items: [Todo] = []\n    derived count = items.length\n    action add(t: Todo) { items = items.concat([t]) }\n}\npage Home(path: \"/\", id: String) {\n    use S\n    state draft = \"\"\n    for todo in S.items by todo.id { Text(todo.title) }\n    derived c = S.count\n}\n";
    let draft = hover_text(src, "draft = ").unwrap();
    assert!(draft.contains("Type `String`"), "{draft}");
    let todo = hover_text(src, "todo.title").unwrap();
    assert!(todo.contains("Type `Todo`"), "{todo}");
    let count = hover_text(src, "count\n").unwrap();
    assert!(count.contains("Type `Number`"), "{count}");
    let add = hover_text(src, "add(t").unwrap();
    assert!(add.contains("Type `action(Todo)`"), "{add}");
}

// ─── Completion ───────────────────────────────────────────────────────────

#[test]
fn completion_after_a_typed_value_offers_its_fields_or_methods() {
    let valid = "type Todo { id: String, title: String, done: Bool }\npage Home(path: \"/\") {\n    state todos: [Todo] = []\n    state name = \"\"\n    for t in todos by t.id { Text(t.title) }\n}\n";
    let broken = valid.replace("Text(t.title)", "Text(t.)");
    let fields = labels_mid_edit(valid, &broken, "Text(t.");
    assert_eq!(fields, vec!["id", "title", "done"]);
    let broken = valid.replace("Text(t.title)", "Text(todos.)");
    let methods = labels_mid_edit(valid, &broken, "Text(todos.");
    assert!(methods.contains(&"length".to_string()), "{methods:?}");
    assert!(methods.contains(&"filter".to_string()), "{methods:?}");
    let broken = valid.replace("Text(t.title)", "Text(name.)");
    let methods = labels_mid_edit(valid, &broken, "Text(name.");
    assert!(methods.contains(&"toUpperCase".to_string()), "{methods:?}");
}

#[test]
fn a_type_error_is_a_diagnostic_with_its_hint() {
    let src = "page Home(path: \"/\") {\n    state n: Number = \"x\"\n    Text(\"{n}\")\n}\n";
    let project = project(src);
    let diagnostics = project_diagnostics(&project).remove(0);
    let t01 = diagnostics
        .iter()
        .find(|d| d.code == Some(NumberOrString::String("T01".into())))
        .expect("a type error");
    assert_eq!(t01.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(t01.range.start.line, 1);
    assert!(t01.message.contains("Convert it"), "{}", t01.message);
    // The code is data with a link to its entry, and the range is the
    // value's whole span, not the first word of the line.
    assert!(
        t01.code_description.as_ref().is_some_and(|c| c
            .href
            .as_str()
            .ends_with("diagnostics#t01-a-value-of-the-wrong-type")),
        "{t01:?}"
    );
    assert!(
        t01.range.end.character > t01.range.start.character + 1,
        "{t01:?}"
    );
}

#[test]
fn completion_inside_an_element_offers_its_own_props_first() {
    let src = "page Home(path: \"/\") {\n    Slider(bind: v, )\n}\n";
    let items = labels_after(src, "bind: v, ");
    assert!(items.contains(&"min:".to_string()), "{items:?}");
    assert!(items.contains(&"step:".to_string()), "{items:?}");
    assert!(
        !items.contains(&"src:".to_string()),
        "Slider takes no src: {items:?}"
    );
    assert!(
        !items.contains(&"bind:".to_string()),
        "a prop already written is not offered again: {items:?}"
    );
    assert!(items.contains(&"v".to_string()) || !items.contains(&"Card".to_string()));
}

#[test]
fn completion_inside_a_user_component_call_offers_its_props() {
    let src = "component UserCard(name: String, role: String) {\n    Text(name)\n}\npage Home(path: \"/\") {\n    UserCard()\n}\n";
    let items = labels_after(src, "    UserCard(");
    assert_eq!(items, vec!["name:", "role:"]);
}

#[test]
fn completion_after_a_dot_offers_parts_flags_cases_or_store_members() {
    let valid = "store CartStore {\n    state items = []\n    action clear() { items = [] }\n}\npage Home(path: \"/\") {\n    use CartStore\n    Card { }\n    Button(\"x\") { }\n    Badge(\"x\", tone: .info)\n}\n";
    let broken = valid
        .replace("Card { }", "Card. { }")
        .replace("Button(\"x\") { }", "Button(\"x\"). { CartStore. }")
        .replace("tone: .info", "tone: .");
    let card = labels_mid_edit(valid, &broken, "Card. ");
    assert!(
        card.starts_with(
            &[
                "Header".to_string(),
                "Body".to_string(),
                "Footer".to_string()
            ][..]
        ),
        "{card:?}"
    );
    assert!(card.contains(&"elevated".to_string()), "{card:?}");
    let button = labels_mid_edit(valid, &broken, "Button(\"x\").");
    assert!(button.contains(&"primary".to_string()), "{button:?}");
    assert!(button.contains(&"lg".to_string()), "{button:?}");
    assert!(
        button.contains(&"fadeIn".to_string()),
        "the universal flags too: {button:?}"
    );
    let store = labels_mid_edit(valid, &broken, "CartStore. ");
    assert_eq!(store, vec!["items", "clear"]);
    let cases = labels_mid_edit(valid, &broken, "tone: .");
    assert_eq!(
        cases,
        vec![
            "primary",
            "secondary",
            "success",
            "danger",
            "warning",
            "info"
        ]
    );
}

#[test]
fn completion_in_a_body_offers_components_keywords_and_scope() {
    let src =
        "page Home(path: \"/\") {\n    state count = 0\n    Container {\n        \n    }\n}\n";
    let items = labels_after(src, "Container {\n        ");
    assert!(items.contains(&"Button".to_string()));
    assert!(items.contains(&"if".to_string()));
    assert!(items.contains(&"match".to_string()));
    assert!(items.contains(&"count".to_string()));
    assert!(
        !items.contains(&"page".to_string()),
        "declarations are not statements: {items:?}"
    );
}

#[test]
fn completion_in_a_style_block_offers_css_selectors_and_tokens() {
    let src = "theme T { line: #ccc }\npage Home(path: \"/\") {\n    Card {\n        style {\n            \n            color: \n        }\n    }\n}\n";
    let props = labels_after(src, "style {\n            ");
    assert!(props.contains(&"border-radius".to_string()), "{props:?}");
    assert!(props.contains(&"&:hover".to_string()), "{props:?}");
    assert!(props.contains(&"@media".to_string()), "{props:?}");
    assert!(!props.contains(&"Button".to_string()), "{props:?}");
    let values = labels_after(src, "color: ");
    assert!(values.contains(&"$color-primary".to_string()), "{values:?}");
    assert!(
        values.contains(&"$line".to_string()),
        "the theme's own: {values:?}"
    );
    assert!(
        values.contains(&"$primary".to_string()),
        "the group's short name: {values:?}"
    );
}

#[test]
fn completion_in_a_match_body_offers_the_missing_arms() {
    let src = "page Home(path: \"/\") {\n    resource users = fetch(\"/api\")\n    match users {\n        loading { Spinner }\n        \n    }\n}\n";
    let items = labels_after(src, "Spinner }\n        ");
    assert_eq!(items, vec!["error", "ready", "else"]);
}

#[test]
fn completion_offers_nothing_inside_strings_and_comments() {
    let src = "page Home(path: \"/\") {\n    // \n    Text(\"\")\n}\n";
    assert!(labels_after(src, "// ").is_empty());
    assert!(labels_after(src, "Text(\"").is_empty());
}

#[test]
fn completion_at_top_level_and_in_a_theme() {
    let src = "theme Brand {\n    \n}\n\n";
    let top = labels_after(src, "}\n\n");
    assert!(top.contains(&"page".to_string()), "{top:?}");
    assert!(top.contains(&"enum".to_string()), "{top:?}");
    let theme = labels_after(src, "Brand {\n    ");
    assert!(theme.contains(&"color-primary".to_string()), "{theme:?}");
}

#[test]
fn completion_keeps_working_while_the_file_does_not_parse() {
    // The unclosed parenthesis means the file does not parse; the last good
    // parse still supplies the scope, and the element's props come from
    // the text.
    let src = "page Home(path: \"/\") {\n    state count = 0\n    Button(\"x\", \n}\n";
    let project = project(src);
    assert!(project.files[0].parsed.is_err());
    let pos = project.files[0]
        .index
        .offset_to_position(src, src.find("\"x\", ").unwrap() + 5);
    let items: Vec<String> = provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert!(items.contains(&"to:".to_string()), "{items:?}");
}

#[test]
fn completion_after_on_offers_events_and_after_emit_the_declared_ones() {
    let valid = "component PickRow(_ label: String) {\n    event pick(id: String)\n    Button(label) { on click { emit pick(1) } }\n}\npage Home(path: \"/\") {\n    PickRow(\"x\") { on pick(id) { } }\n}\n";
    let broken = valid.replace(
        "PickRow(\"x\") { on pick(id) { } }",
        "PickRow(\"x\") { on  }",
    );
    let items = labels_mid_edit(valid, &broken, "PickRow(\"x\") { on ");
    assert!(items.contains(&"click".to_string()), "{items:?}");
    assert!(items.contains(&"submit".to_string()), "{items:?}");
    assert_eq!(
        items[0], "pick",
        "the component's own event comes first: {items:?}"
    );
    let broken = valid.replace("emit pick(1)", "emit ");
    let emitted = labels_mid_edit(valid, &broken, "emit ");
    assert_eq!(emitted, vec!["pick"]);
}

#[test]
fn completion_after_layout_offers_components_with_a_default_slot() {
    let valid = "component Shell { slot  children }\ncomponent Chip { Text(\"x\") }\npage Home(path: \"/\", layout: Shell) { }\n";
    let broken = valid.replace("layout: Shell", "layout: ");
    let items = labels_mid_edit(valid, &broken, "layout: ");
    assert_eq!(items, vec!["Shell"]);
}

// ─── Definition ───────────────────────────────────────────────────────────

#[test]
fn definition_of_a_name_is_the_one_in_scope() {
    let src = "page A(path: \"/a\") {\n    state count = 1\n}\npage B(path: \"/b\") {\n    state count = 2\n    Text(\"{count}\")\n    Button(\"+\") { on click { count = count + 1 } }\n}\n";
    let project = project(src);
    let def = find_definition(&project, 0, at(src, "count + 1")).unwrap();
    let GotoDefinitionResponse::Scalar(loc) = def else {
        panic!()
    };
    assert_eq!(loc.range.start.line, 4, "B's count, not A's");
}

#[test]
fn definition_of_a_component_call_a_store_member_and_a_layout() {
    let src = "component Nudge(label: String) {\n    Text(label)\n}\nstore S {\n    state n = 0\n    action bump() { n = n + 1 }\n}\npage Home(path: \"/\", layout: Nudge) {\n    use S\n    Nudge(label: \"x\")\n    Button(\"b\") { on click { S.bump() } }\n}\n";
    let project = project(src);
    let GotoDefinitionResponse::Scalar(component) =
        find_definition(&project, 0, at(src, "Nudge(label")).unwrap()
    else {
        panic!()
    };
    assert_eq!(component.range.start.line, 0);
    let GotoDefinitionResponse::Scalar(member) =
        find_definition(&project, 0, at(src, "bump() }")).unwrap()
    else {
        panic!()
    };
    assert_eq!(member.range.start.line, 5);
    let GotoDefinitionResponse::Scalar(store) =
        find_definition(&project, 0, at(src, "S\n    Nudge")).unwrap()
    else {
        panic!()
    };
    assert_eq!(store.range.start.line, 3);
    let GotoDefinitionResponse::Scalar(layout) =
        find_definition(&project, 0, at(src, "Nudge) {")).unwrap()
    else {
        panic!()
    };
    assert_eq!(layout.range.start.line, 0);
}

// ─── Diagnostics ──────────────────────────────────────────────────────────

#[test]
fn diagnostics_point_at_the_word_even_after_non_ascii_text() {
    let src = "page Home(path: \"/\", title: \"x\", description: \"y\") {\n    Heading(\"أهلاً\").h1\n    Text(\"مرحباً بك\").centred\n}\n";
    let project = project(src);
    let diagnostics = project_diagnostics(&project).remove(0);
    let unknown = diagnostics
        .iter()
        .find(|d| d.message.contains("centred"))
        .expect("an error for `centred`");
    let start = project.files[0]
        .index
        .offset_to_position(src, src.find(".centred").unwrap());
    assert_eq!(unknown.range.start, start, "{unknown:?}");
    assert_eq!(unknown.severity, Some(DiagnosticSeverity::ERROR));
}

#[test]
fn a_parse_error_is_one_error_at_its_position() {
    let src = "page Home(path: \"/\") {\n    Text(\"x\"\n}\n";
    let project = project(src);
    let diagnostics = project_diagnostics(&project).remove(0);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostics[0].range.start.line, 2);
}

// ─── Symbols ──────────────────────────────────────────────────────────────

#[test]
fn document_symbols_are_nested_under_declarations() {
    let src = "store CartStore {\n    state items = []\n    action clear() { items = [] }\n}\n";
    let project = project(src);
    let DocumentSymbolResponse::Nested(symbols) = document_symbols(&project, 0, true) else {
        panic!()
    };
    assert_eq!(symbols[0].name, "CartStore");
    let children = symbols[0].children.as_ref().unwrap();
    assert_eq!(
        children.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        vec!["items", "clear()"]
    );
}

#[test]
fn an_indented_file_is_read_by_its_layout() {
    // The same page, blocks by indentation: no parse error, the same
    // diagnostics, hover and completion as its braced spelling.
    let src = "page Home(path: \"/\", title: \"x\", description: \"y\")\n    state count = 0\n    Heading(\"Hi\").h1\n    Button(\"+1\").primary\n        on click\n            count = count + 1\n    Text(\"{count}\").centred\n";
    let project = Project::single(Url::parse("file:///test.wfx").unwrap(), src);
    let diagnostics = project_diagnostics(&project).remove(0);
    assert!(
        diagnostics.iter().all(|d| !d.message.contains("Expected")),
        "{diagnostics:?}"
    );
    let unknown = diagnostics
        .iter()
        .find(|d| d.message.contains("centred"))
        .expect("an error for `centred`");
    let start = project.files[0]
        .index
        .offset_to_position(src, src.find(".centred").unwrap());
    assert_eq!(unknown.range.start, start, "{unknown:?}");
    let pos = project.files[0]
        .index
        .offset_to_position(src, src.find(".primary").unwrap() + 1);
    let hover = provide_hover(&project, 0, pos).map(|h| match h.contents {
        HoverContents::Markup(m) => m.value,
        _ => panic!("markdown expected"),
    });
    assert!(
        hover.as_deref().is_some_and(|h| h.contains("tone")),
        "{hover:?}"
    );
}

// ─── Rename and extract ──────────────────────────────────

fn apply(src: &str, edit: &WorkspaceEdit) -> String {
    let project = project(src);
    let file = &project.files[0];
    let mut edits: Vec<&TextEdit> = edit.changes.as_ref().unwrap().values().flatten().collect();
    edits.sort_by_key(|e| (e.range.start.line, e.range.start.character));
    let mut out = src.to_string();
    for e in edits.into_iter().rev() {
        let s = file.index.position_to_offset(src, e.range.start).unwrap();
        let t = file.index.position_to_offset(src, e.range.end).unwrap();
        out.replace_range(s..t, &e.new_text);
    }
    out
}

#[test]
fn rename_changes_a_component_everywhere_it_is_named() {
    let src = "component Shell { children }\ncomponent Row2 { Shell { Text(\"x\") } }\npage P(path: \"/\", layout: Shell) { Shell { Text(\"y\") }  Text(\"Shell\") }";
    let project = project(src);
    let edit =
        wf_lsp::rename::rename(&project, 0, at(src, "Shell { Text(\"x\")"), "Frame").unwrap();
    assert_eq!(
        apply(src, &edit),
        "component Frame { children }\ncomponent Row2 { Frame { Text(\"x\") } }\npage P(path: \"/\", layout: Frame) { Frame { Text(\"y\") }  Text(\"Shell\") }",
    );
}

#[test]
fn rename_changes_a_state_and_its_reads_inside_interpolations_but_not_another_declarations() {
    let src = "page P(path: \"/\") { state count = 0\n state row = { count: 2 }\n Button(\"+\") { on click { count = count + 1 } }\n Text(\"{count} items {row.count}\")\n Text(row.count)\n Text(\"count\") }\ncomponent C { state count = 5\n Text(\"{count}\") }";
    let project = project(src);
    let edit = wf_lsp::rename::rename(&project, 0, at(src, "count = count + 1"), "total").unwrap();
    assert_eq!(
        apply(src, &edit),
        "page P(path: \"/\") { state total = 0\n state row = { count: 2 }\n Button(\"+\") { on click { total = total + 1 } }\n Text(\"{total} items {row.count}\")\n Text(row.count)\n Text(\"count\") }\ncomponent C { state count = 5\n Text(\"{count}\") }",
    );
}

#[test]
fn rename_changes_a_store_member_at_its_declaration_and_every_dotted_read() {
    let src = "store Cart { state items = []\n derived total = items.length\n action add(x: Any) { items.push(x) } }\npage P(path: \"/\") { use Cart\n Text(\"{Cart.total}\")\n Button(\"a\") { on click { Cart.add(1) } } }";
    let project = project(src);
    let edit = wf_lsp::rename::rename(&project, 0, at(src, "total}"), "count").unwrap();
    let out = apply(src, &edit);
    assert!(
        out.contains("derived count = items.length") && out.contains("{Cart.count}"),
        "{out}"
    );
    let refused = wf_lsp::rename::rename(&project, 0, at(src, "Button(\"a\")"), "X");
    assert!(refused.is_err(), "a built-in is not renamed");
    assert!(
        wf_lsp::rename::rename(&project, 0, at(src, "total}"), "Total").is_err(),
        "a value keeps its case"
    );
    assert!(wf_lsp::rename::prepare_rename(&project, 0, at(src, "total}")).is_some());
    assert!(wf_lsp::rename::prepare_rename(&project, 0, at(src, "Button(\"a\")")).is_none());
}

#[test]
fn extract_component_lifts_the_selection_with_what_it_reads_as_props() {
    let src = "type Todo { id: String, title: String }\npage P(path: \"/\") {\n    state todos: [Todo] = []\n    state open = true\n    action save() { log(1) }\n    use Cart\n    Card {\n        Text(\"head\")\n    }\n    for t in todos {\n        Row(gap: .sm) {\n            Text(t.title)\n            if open { Badge(\"{Cart.total}\") }\n        }\n    }\n}\nstore Cart { state total = 0 }\n";
    let project = project(src);
    let file = &project.files[0];
    let start = src.find("for t in todos").unwrap();
    let end = src.find("\n}\nstore").unwrap();
    let range = Range {
        start: file.index.offset_to_position(src, start),
        end: file.index.offset_to_position(src, end),
    };
    let actions = wf_lsp::code_actions::extract_component_action(&project, 0, range);
    assert_eq!(actions.len(), 1);
    let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
        panic!()
    };
    let out = apply(src, action.edit.as_ref().unwrap());
    assert!(
        out.contains("    Extracted(open: open, todos: todos)\n}"),
        "{out}"
    );
    assert!(out.contains("\n\ncomponent Extracted(open: Bool, todos: [Todo]) {\n    use Cart\n    for t in todos {\n        Row(gap: .sm) {\n            Text(t.title)\n            if open { Badge(\"{Cart.total}\") }\n        }\n    }\n}\n"), "{out}");
    // A selection that writes outer state, or calls an action, is not offered.
    let src2 = "page P(path: \"/\") {\n    state n = 0\n    action save() { log(1) }\n    Button(\"x\") { on click { n = n + 1 } }\n    Button(\"y\") { on click { save() } }\n}\n";
    let project2 = Project::single(Url::parse("file:///test.wf").unwrap(), src2);
    let file2 = &project2.files[0];
    for needle in ["Button(\"x\")", "Button(\"y\")"] {
        let s = src2.find(needle).unwrap();
        let e = s + src2[s..].find('\n').unwrap();
        let range = Range {
            start: file2.index.offset_to_position(src2, s),
            end: file2.index.offset_to_position(src2, e),
        };
        assert!(
            wf_lsp::code_actions::extract_component_action(&project2, 0, range).is_empty(),
            "{needle}"
        );
    }
}

#[test]
fn a_misspelled_field_offers_the_field_it_meant() {
    let src = "type User { id: String, name: String }\npage Home(path: \"/\") {\n    state u = User(id: \"1\", name: \"Ada\")\n    Text(u.nmae)\n}\n";
    let project = project(src);
    let diagnostics = project_diagnostics(&project).remove(0);
    let t05 = diagnostics
        .iter()
        .find(|d| d.code == Some(NumberOrString::String("T05".into())))
        .expect("a missing field");
    let uri = Url::parse("file:///p/src/App.wf").unwrap();
    let actions = wf_lsp::code_actions::provide_code_actions(
        &uri,
        CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: t05.range,
            context: CodeActionContext {
                diagnostics: vec![t05.clone()],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        },
    );
    let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
        panic!("{actions:?}");
    };
    assert_eq!(action.title, "Change to `name`");
    let edits = &action.edit.as_ref().unwrap().changes.as_ref().unwrap()[&uri];
    // `.nmae` on line 4 becomes `.name`.
    assert_eq!(edits[0].new_text, ".name");
    assert_eq!(edits[0].range.start, Position::new(3, 10));
    assert_eq!(edits[0].range.end, Position::new(3, 15));
}

/// A project on disk with these files, loaded the way the server loads the
/// one an open document belongs to.
fn disk_project(name: &str, files: &[(&str, &str)], open: &str) -> (Project, usize) {
    let root = std::env::temp_dir().join(format!("wf-lsp-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("webfluent.app.json"), r#"{ "name": "t" }"#).unwrap();
    for (path, text) in files {
        let at = root.join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    let uri = Url::from_file_path(root.join(open)).unwrap();
    let project = Project::load(&uri, &|_| None, &wf_lsp::project::FileCache::default());
    let ix = project
        .file_index(&uri)
        .expect("the open file is in its project");
    (project, ix)
}

#[test]
fn a_markdown_page_is_a_page_the_editor_knows() {
    let app = "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(\"H\").h1\n    Link(\"About\", to: \"/about\")\n    Link(\"Gone\", to: \"/gone\")\n}\n";
    let (project, ix) = disk_project(
        "md",
        &[
            ("src/App.wf", app),
            (
                "src/about.md",
                "---\ntitle: About\ndescription: Who we are.\n---\n# About\n\nWe make things.\n",
            ),
        ],
        "src/App.wf",
    );
    let found = project_diagnostics(&project);
    let r01: Vec<u32> = found[ix]
        .iter()
        .filter(|d| d.code == Some(NumberOrString::String("R01".into())))
        .map(|d| d.range.start.line + 1)
        .collect();
    // `/about` is the Markdown page's route; `/gone` is nobody's.
    assert_eq!(r01, vec![5], "{:#?}", found[ix]);

    // A broken page's mistake is its front matter's, not a WebFluent parse
    // of its Markdown.
    let (project, _) = disk_project(
        "md-broken",
        &[
            ("src/App.wf", app),
            ("src/broken.md", "---\ntitle: Broken\n# never closed\n"),
        ],
        "src/App.wf",
    );
    let found = project_diagnostics(&project);
    let broken = project
        .files
        .iter()
        .position(|f| f.path.ends_with("broken.md"))
        .unwrap();
    let codes: Vec<String> = found[broken]
        .iter()
        .map(|d| match &d.code {
            Some(NumberOrString::String(c)) => c.clone(),
            _ => String::new(),
        })
        .collect();
    assert_eq!(codes, vec!["E002"], "{:#?}", found[broken]);
    assert!(
        found[broken][0].message.contains("front matter"),
        "{}",
        found[broken][0].message
    );
}

const DECLS: &str = "const LIMIT = 3\ndata posts = \"p.json\"\nimage hero = \"h.jpg\"\ntype Post { id: String }\nenum Tone { calm, loud }\nanimation Wobble { from { opacity: 0 } to { opacity: 1 } }\napi Backend(base: \"/api\") {\n    get users() -> [Post]\n}\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    state t: Tone = .calm\n    state p: Post? = Post(id: \"1\")\n    resource r = Backend.users()\n    Heading(\"{LIMIT} {posts.length}\").h1\n    Image(hero, alt: \"a\")\n    Card(animate: .Wobble) { Text(\"a\") }\n    match t {\n        .calm { Text(\"c\") }\n        .loud { Text(\"l\") }\n    }\n}\n";

/// The line (1-based) go-to-definition lands on for the `nth` occurrence of
/// `needle`.
fn definition_line(src: &str, needle: &str, nth: usize) -> Option<u32> {
    let project = project(src);
    let offset = src.match_indices(needle).nth(nth).unwrap().0 + 1;
    let pos = project.files[0].index.offset_to_position(src, offset);
    match find_definition(&project, 0, pos)? {
        GotoDefinitionResponse::Scalar(l) => Some(l.range.start.line + 1),
        _ => None,
    }
}

#[test]
fn every_kind_of_declaration_has_a_definition() {
    // A use, and the line its declaration is on.
    for (needle, nth, line) in [
        ("LIMIT", 1, 1),   // in a string's splice
        ("posts", 1, 2),   // in a splice, before `.length`
        ("hero", 1, 3),    // an image, as an argument
        ("Post?", 0, 4),   // a type in an annotation
        ("Post(id", 0, 4), // a record built
        ("Tone =", 0, 5),  // an enum in an annotation
        ("calm", 1, 5),    // a case, as a value
        ("calm", 2, 5),    // a case, as a match arm
        ("Wobble", 1, 6),  // an animation, played
        ("Backend", 1, 7), // a service
        ("users", 1, 8),   // its endpoint
    ] {
        assert_eq!(
            definition_line(DECLS, needle, nth),
            Some(line),
            "{needle} #{nth}"
        );
    }
}

#[test]
fn hover_and_definition_read_a_string_s_splices_as_code() {
    let src = "page P(path: \"/\") {\n    state count = 0\n    Text(\"count: {count}\")\n}\n";
    // The splice's `count` is the state; the text's `count` is text.
    assert_eq!(definition_line(src, "count", 2), Some(2));
    assert_eq!(definition_line(src, "count", 1), None);
    let project = project(src);
    let offset = src.match_indices("count").nth(2).unwrap().0 + 1;
    let pos = project.files[0].index.offset_to_position(src, offset);
    assert!(provide_hover(&project, 0, pos).is_some());
    // A raw string has no splices.
    let raw = "page P(path: \"/\") {\n    state count = 0\n    Text(#\"{count}\"#)\n}\n";
    assert_eq!(definition_line(raw, "count", 1), None);
}

/// `src` with the name at the `nth` `needle` renamed to `to`.
fn renamed(src: &str, needle: &str, nth: usize, to: &str) -> String {
    let project = project(src);
    let offset = src.match_indices(needle).nth(nth).unwrap().0 + 1;
    let pos = project.files[0].index.offset_to_position(src, offset);
    assert!(
        wf_lsp::rename::prepare_rename(&project, 0, pos).is_some(),
        "{needle} cannot be renamed"
    );
    apply(src, &wf_lsp::rename::rename(&project, 0, pos, to).unwrap())
}

#[test]
fn every_kind_of_declaration_renames_everywhere_it_is_named() {
    let out = renamed(DECLS, "LIMIT", 1, "MAX");
    assert!(
        out.starts_with("const MAX = 3") && out.contains("{MAX} {posts.length}"),
        "{out}"
    );
    let out = renamed(DECLS, "Post?", 0, "Article");
    assert!(
        out.contains("type Article {")
            && out.contains("-> [Article]")
            && out.contains("state p: Article? = Article(id"),
        "{out}"
    );
    let out = renamed(DECLS, "calm", 1, "quiet");
    assert!(
        out.contains("enum Tone { quiet, loud }")
            && out.contains("state t: Tone = .quiet")
            && out.contains(".quiet { Text"),
        "{out}"
    );
    let out = renamed(DECLS, "users", 1, "people");
    assert!(
        out.contains("get people() ->") && out.contains("Backend.people()"),
        "{out}"
    );
    let out = renamed(DECLS, "Backend", 1, "Server");
    assert!(
        out.contains("api Server(base") && out.contains("Server.users()"),
        "{out}"
    );
    let out = renamed(DECLS, "Wobble", 1, "Shake");
    assert!(
        out.contains("animation Shake {") && out.contains("animate: .Shake"),
        "{out}"
    );
    let out = renamed(DECLS, "hero", 1, "banner");
    assert!(
        out.contains("image banner =") && out.contains("Image(banner,"),
        "{out}"
    );
}

#[test]
fn a_record_s_field_has_a_definition_in_the_type_that_declares_it() {
    let src = "type User { id: String, name: String }\ntype Admin = User { role: String }\npage P(path: \"/\") {\n    state a: Admin? = null\n    state u: User = User(id: \"1\", name: \"Ada\")\n    Text(u.name)\n    Text(a?.role ?? \"\")\n    Text(a?.id ?? \"\")\n}\n";
    assert_eq!(definition_line(src, "name)", 0), Some(1));
    assert_eq!(definition_line(src, "role ??", 0), Some(2));
    // A field `Admin` inherits is `User`'s.
    assert_eq!(definition_line(src, "id ??", 0), Some(1));
}

#[test]
fn a_field_renames_where_it_is_read_and_where_a_record_is_built() {
    let src = "type User { id: String, name: String }\npage P(path: \"/\") {\n    state u: User = User(id: \"1\", name: \"Ada\")\n    state row = { name: \"x\" }\n    Text(\"{u.name}\")\n    Text(u.name)\n    Text(row.name)\n}\n";
    let out = renamed(src, "name)", 0, "title");
    assert_eq!(
        out,
        "type User { id: String, title: String }\npage P(path: \"/\") {\n    state u: User = User(id: \"1\", title: \"Ada\")\n    state row = { name: \"x\" }\n    Text(\"{u.title}\")\n    Text(u.title)\n    Text(row.name)\n}\n"
    );
}

// ─── Completion: what the compiler knows ──────────────────────────────────

/// Completion at `‸` in `broken`, the server holding `valid` as the last
/// parse that succeeded — as it does while the user types. Labels come back
/// without a prop's `:` or a token's `$`.
fn offered(valid: &str, broken: &str) -> Vec<String> {
    let at = broken.find('‸').expect("a ‸ marks the cursor");
    let text = broken.replacen('‸', "", 1);
    let project = mid_edit(valid, &text);
    let pos = project.files[0].index.offset_to_position(&text, at);
    provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| {
            c.label
                .trim_end_matches(':')
                .trim_start_matches('$')
                .to_string()
        })
        .collect()
}

#[track_caller]
fn offers(labels: &[String], want: &[&str]) {
    let missing: Vec<&&str> = want
        .iter()
        .filter(|w| !labels.iter().any(|l| l == *w))
        .collect();
    assert!(missing.is_empty(), "missing {missing:?} in {labels:?}");
}

const HEAD: &str =
    "page Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(\"H\").h1\n";

/// `body` inside a page, as the last good parse and as the buffer.
fn in_page(valid_body: &str, broken_body: &str) -> Vec<String> {
    offered(
        &format!("{HEAD}{valid_body}\n}}\n"),
        &format!("{HEAD}{broken_body}\n}}\n"),
    )
}

#[test]
fn every_keyword_the_parser_reads_is_documented() {
    use wf_lsp::reference::KEYWORDS;
    let documented = |w: &str| KEYWORDS.iter().any(|k| k.name == w);
    for word in webfluent::parser::v2::DECLARATION_WORDS
        .iter()
        .filter(|w| **w != "external")
    {
        assert!(
            documented(word),
            "`{word}` declares something and has no entry"
        );
    }
    for word in webfluent::parser::v2::BODY_KEYWORDS
        .iter()
        .chain(webfluent::parser::v2::IMPERATIVE_KEYWORDS)
    {
        assert!(
            documented(word),
            "`{word}` begins a statement and has no entry"
        );
    }
    for name in webfluent::sema::types::built_in_functions()
        .iter()
        .chain(webfluent::codegen::js::BROWSER_VALUES)
    {
        assert!(
            wf_lsp::reference::builtin(name).is_some(),
            "`{name}` is the language's and has no entry"
        );
    }
}

#[test]
fn keywords_are_offered_where_they_may_be_written() {
    offers(
        &offered("", "‸\n"),
        &[
            "page",
            "component",
            "store",
            "app",
            "theme",
            "animation",
            "type",
            "enum",
            "const",
            "data",
            "image",
            "api",
            "test",
        ],
    );
    let body = in_page("    Text(\"a\")", "    ‸");
    offers(
        &body,
        &[
            "state", "persist", "validate", "resource", "socket", "stream", "channel", "peer",
            "every", "after", "head", "sequence", "match", "show", "for", "if",
        ],
    );
    assert!(
        !body.contains(&"slot".to_string()) && !body.contains(&"let".to_string()),
        "{body:?}"
    );
    let comp = offered(
        "component C {\n    Text(\"a\")\n}\n",
        "component C {\n    ‸\n}\n",
    );
    offers(&comp, &["slot", "part", "event", "emit", "children"]);
    assert!(!comp.contains(&"head".to_string()));
    let action = in_page(
        "    action a() {\n        log(1)\n    }",
        "    action a() {\n        ‸\n    }",
    );
    offers(
        &action,
        &[
            "let", "return", "try", "if", "for", "format", "uuid", "navigate",
        ],
    );
    let test = offered(
        "test \"t\" {\n    Text(\"a\")\n    expect \"a\"\n}\n",
        "test \"t\" {\n    Text(\"a\")\n    ‸\n}\n",
    );
    offers(&test, &["expect", "click", "type", "press", "state"]);
}

#[test]
fn the_language_s_functions_and_values_and_the_program_s_constants_are_offered_in_expressions() {
    let src = "const LIMIT = 3\ndata posts = \"p.json\"\nimage hero = \"h.jpg\"\n";
    let labels = offered(
        &format!("{src}{HEAD}    Text(\"a\")\n}}\n"),
        &format!("{src}{HEAD}    Text(‸)\n}}\n"),
    );
    offers(
        &labels,
        &[
            "format", "ago", "t", "uuid", "viewport", "query", "now", "network", "LIMIT", "posts",
            "hero",
        ],
    );
}

#[test]
fn after_a_dot_a_value_offers_what_its_type_has() {
    let pairs: &[(&str, &str, &str, &[&str])] = &[
        (
            "state d: Date = @2026-03-14",
            "d.year()",
            "d.‸",
            &["plus", "year", "isBefore", "startOfMonth", "until"],
        ),
        (
            "state m: Money = €12.99",
            "m.amount",
            "m.‸",
            &["amount", "currency", "plus", "times"],
        ),
        (
            "state u: Url = \"https://a.com\"",
            "u.host()",
            "u.‸",
            &["host", "path", "with"],
        ),
        (
            "state xs = [1, 2]",
            "xs.length",
            "xs.‸",
            &[
                "length", "map", "sortBy", "groupBy", "unique", "take", "first", "sum",
            ],
        ),
        (
            "state s = \"a\"",
            "s.length",
            "s.‸",
            &[
                "capitalize",
                "truncate",
                "dedent",
                "lines",
                "words",
                "toLowerCase",
            ],
        ),
        (
            "state w = 3.seconds",
            "w",
            "3.‸",
            &["ms", "seconds", "minutes", "hours", "days", "weeks"],
        ),
        (
            "state n = 3",
            "format(n, .currency)",
            "format(n, .‸)",
            &[
                "number", "currency", "percent", "compact", "date", "relative",
            ],
        ),
        (
            "state x = 0",
            "viewport.md",
            "viewport.‸",
            &["md", "width", "sm", "xl"],
        ),
        (
            "state x = 0",
            "network.online",
            "network.‸",
            &["online", "saveData", "queued"],
        ),
    ];
    for (decl, valid, broken, want) in pairs {
        let labels = in_page(
            &format!("    {decl}\n    Text({valid})"),
            &format!("    {decl}\n    Text({broken})"),
        );
        offers(&labels, want);
    }
}

#[test]
fn after_a_dot_a_declaration_or_a_handle_offers_its_members() {
    let api = "api Backend(base: \"/api\") {\n    get users() -> [Map]\n    get user(id: String) at \"users/:id\" -> Map\n}\n";
    let v = format!("{api}{HEAD}    resource r = Backend.users()\n}}\n");
    offers(
        &offered(&v, &v.replace("Backend.users()", "Backend.‸")),
        &["users", "user", "invalidate"],
    );
    offers(
        &offered(&v, &v.replace("Backend.users()", "Backend.users.‸")),
        &["invalidate", "prefetch", "pending", "url"],
    );
    let e = "enum Tone { calm, loud }\n";
    let v = format!(
        "{e}{HEAD}    state t: Tone = .calm\n    Text(if t == .calm {{ \"a\" }} else {{ \"b\" }})\n}}\n"
    );
    offers(
        &offered(&v, &v.replacen("Tone = .calm", "Tone = .‸", 1)),
        &["calm", "loud"],
    );
    offers(
        &offered(&v, &v.replace("t == .calm", "t == .‸")),
        &["calm", "loud"],
    );
    let im = "image hero = \"h.jpg\"\n";
    let v = format!("{im}{HEAD}    Text(hero.src)\n}}\n");
    offers(
        &offered(&v, &v.replace("hero.src", "hero.‸")),
        &["src", "width", "height", "color", "srcset"],
    );
    for (decl, valid, broken, want) in [
        (
            "socket chat = ws(\"wss://x\")",
            "chat.state",
            "chat.‸",
            &["send", "messages", "state", "close", "last"][..],
        ),
        (
            "channel c = broadcast(\"x\")",
            "c.post(1)",
            "c.‸",
            &["post", "messages"][..],
        ),
        (
            "resource r = fetch(\"/x\")",
            "r.data",
            "r.‸",
            &["reload", "state", "data", "error", "loadMore"][..],
        ),
        (
            "action save() { await fetch(\"/x\") }",
            "save.pending",
            "save.‸",
            &["pending"][..],
        ),
        ("state p: Map? = null", "p?.x", "p?.‸", &[][..]),
    ] {
        offers(
            &in_page(
                &format!("    {decl}\n    Text({valid})"),
                &format!("    {decl}\n    Text({broken})"),
            ),
            want,
        );
    }
    let form = "    Form(bind: f) { Button(\"x\", disabled: !f.valid) }";
    offers(
        &in_page(form, &form.replace("f.valid", "f.‸")),
        &["valid", "errors", "reset", "pending", "touched"],
    );
    let el = "    Input(ref: box, label: \"L\")\n    Button(\"x\") { on click { box.focus() } }";
    offers(
        &in_page(el, &el.replace("box.focus()", "box.‸")),
        &["focus", "blur", "value"],
    );
}

#[test]
fn a_type_is_offered_wherever_one_is_written() {
    let decls = "type Post { id: String }\nenum Tone { calm, loud }\n";
    let types = &[
        "Post", "Tone", "String", "Number", "Bool", "Date", "Money", "Url", "Duration",
    ];
    let v = format!("{decls}{HEAD}    state p: Post? = null\n}}\n");
    offers(
        &offered(&v, &v.replace("state p: Post?", "state p: ‸")),
        types,
    );
    let v = format!("{decls}component C(title: String) {{ Text(title) }}\n");
    offers(&offered(&v, &v.replace("title: String", "title: ‸")), types);
    let v = format!("{decls}type Row {{ id: String }}\n");
    offers(
        &offered(
            &v,
            &v.replace("type Row { id: String }", "type Row { id: ‸ }"),
        ),
        types,
    );
    let v = format!("{decls}api B(base: \"/x\") {{\n    get posts() -> [Post]\n}}\n");
    offers(&offered(&v, &v.replace("-> [Post]", "-> ‸")), types);
    offers(&offered(&v, &v.replace("-> [Post]", "-> [‸")), types);
}

#[test]
fn a_block_offers_what_it_may_hold() {
    let persist = "    persist n = 0 {\n        sync: false\n    }";
    offers(
        &in_page(persist, &persist.replace("sync: false", "‸")),
        &["in", "version", "sync", "key", "migrate"],
    );
    let validate = "    state e = \"\"\n    validate e {\n        required\n    }\n    Input(bind: e, label: \"E\")";
    offers(
        &in_page(validate, &validate.replace("required", "‸")),
        &[
            "required",
            "email",
            "minLength",
            "pattern",
            "matches",
            "custom",
            "async",
        ],
    );
    let api = "api B(base: \"/x\") {\n    get a() -> Map\n}\n";
    offers(
        &offered(api, &api.replace("get a() -> Map", "‸")),
        &[
            "timeout",
            "retry",
            "credentials",
            "cache",
            "headers",
            "get",
            "post",
            "delete",
            "on",
        ],
    );
    let grid = "    Grid(columns: { base: 1 }) { Text(\"a\") }";
    offers(
        &in_page(grid, &grid.replace("{ base: 1 }", "{ ‸ }")),
        &["base", "sm", "md", "lg", "xl"],
    );
    offers(
        &in_page(grid, &grid.replace("{ base: 1 }", "{ base: 1, ‸ }")),
        &["md"],
    );
}

#[test]
fn a_string_or_a_comment_that_names_something_offers_it() {
    let key = "    on key(\"Escape\") { log(1) }";
    offers(
        &in_page(key, &key.replace("\"Escape\"", "\"‸\"")),
        &["Escape", "Enter", "ArrowDown", "ctrl+"],
    );
    let host = "    Host(tag: \"canvas\")";
    offers(
        &in_page(host, &host.replace("\"canvas\"", "\"‸\"")),
        &["div", "canvas", "svg", "table"],
    );
    let allow = "    // wf-allow(U01)\n    state n = 0";
    let labels = in_page(allow, &allow.replace("(U01)", "(‸"));
    offers(&labels, &["U01", "A01", "R01"]);
    assert!(
        !labels.contains(&"T05".to_string()),
        "an error no allow may silence is not offered"
    );
    let splice =
        "    state count = 0\n    state d: Date = @2026-01-01\n    Text(\"{count} {d.year()}\")";
    offers(
        &in_page(splice, &splice.replace("{count}", "{cou‸}")),
        &["count", "format"],
    );
    offers(
        &in_page(splice, &splice.replace("d.year()", "d.‸")),
        &["year", "plus"],
    );
}

#[test]
fn a_project_s_messages_and_public_env_names_are_offered() {
    let app = "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(t(\"hello\")).h1\n    Text(env.PUBLIC_NAME)\n}\n";
    let root = std::env::temp_dir().join(format!("wf-lsp-msgs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/translations")).unwrap();
    std::fs::write(
        root.join("webfluent.app.json"),
        r#"{ "name": "t", "env": { "PUBLIC_NAME": "x", "SECRET": "y" }, "i18n": { "default_locale": "en", "locales": ["en"], "dir": "src/translations" } }"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/translations/en.json"),
        r#"{ "hello": "Hello", "nav.home": "Home" }"#,
    )
    .unwrap();
    std::fs::write(root.join("src/App.wf"), app).unwrap();
    let uri = Url::from_file_path(root.join("src/App.wf")).unwrap();
    let complete = |broken: &str| -> Vec<String> {
        let at = broken.find('‸').unwrap();
        let text = broken.replacen('‸', "", 1);
        let project = Project::load(
            &uri,
            &|_| {
                Some(wf_lsp::project::OpenText {
                    text: text.as_str().into(),
                    last_valid: Some(std::sync::Arc::new(
                        webfluent::parse_source(app, "src/App.wf").unwrap(),
                    )),
                })
            },
            &wf_lsp::project::FileCache::default(),
        );
        let ix = project.file_index(&uri).unwrap();
        let pos = project.files[ix].index.offset_to_position(&text, at);
        provide_completions(&project, ix, pos)
            .into_iter()
            .map(|c| c.label)
            .collect()
    };
    offers(
        &complete(&app.replace("t(\"hello\")", "t(\"‸\")")),
        &["hello", "nav.home"],
    );
    let env = complete(&app.replace("env.PUBLIC_NAME", "env.‸"));
    offers(&env, &["PUBLIC_NAME"]);
    assert!(
        !env.contains(&"SECRET".to_string()),
        "a private name is never offered: {env:?}"
    );
}

#[test]
fn a_theme_s_own_tokens_are_offered_after_a_dollar() {
    let theme = "theme Brand {\n    viz-1: #ff0000\n    color-primary: #000000\n}\n";
    let v = format!("{theme}{HEAD}    Card {{ style {{ color: $viz-1 }} }}\n}}\n");
    offers(
        &offered(&v, &v.replace("$viz-1", "$‸")),
        &["viz-1", "color-primary", "surface"],
    );
}

// ─── Hover: what each thing is ────────────────────────────────────────────

#[test]
fn hover_on_host_names_its_lifetime_props_and_its_tags() {
    let src = format!("{HEAD}    Host(tag: \"canvas\", mount: (n) => n)\n}}\n");
    let host = hover_text(&src, "Host").unwrap();
    for prop in [
        "`mount:`",
        "`update:`",
        "`cleanup:`",
        "`ref:`",
        "`class:`",
        "`shared:`",
    ] {
        assert!(host.contains(prop), "{prop} in:\n{host}");
    }
    let tag = hover_text(&src, "tag:").unwrap();
    assert!(tag.contains("`canvas`") && tag.contains("`table`"), "{tag}");
}

#[test]
fn hover_on_a_function_or_a_value_the_language_gives() {
    let src = format!(
        "{HEAD}    state d = @2026-01-01\n    Text(\"{{ago(d)}} {{viewport.md}}\")\n    Text(format(3, .currency))\n}}\n"
    );
    let format = hover_text(&src, "format").unwrap();
    assert!(
        format.contains("format(value, .style, option)") && format.contains("locale"),
        "{format}"
    );
    let viewport = hover_text(&src, "viewport").unwrap();
    assert!(
        viewport.contains("`.md`") && viewport.contains("`.width`"),
        "{viewport}"
    );
    assert!(hover_text(&src, "ago").unwrap().contains("ago"));
}

#[test]
fn hover_on_a_member_says_what_it_is_and_what_it_gives() {
    let src = "type User { id: String, /// Shown everywhere.\nname: String }\napi Backend(base: \"/api\") {\n    /// Everyone.\n    get users(page: Number = 1) -> [User]\n}\nenum Tone { calm, loud, failed(reason: String) }\nimage hero = \"h.jpg\"\npage P(path: \"/\") {\n    state u: User = User(id: \"1\", name: \"Ada\")\n    state d: Date = @2026-01-01\n    state t: Tone = .loud\n    resource r = Backend.users(page: 1)\n    Text(u.name)\n    Text(d.plus(days: 1))\n    Image(hero, alt: \"a\")\n}\n";
    let endpoint = hover_text(src, "users(page: 1)").unwrap();
    assert!(
        endpoint.contains("endpoint of `Backend`")
            && endpoint.contains("-> [User]")
            && endpoint.contains("Everyone."),
        "{endpoint}"
    );
    let field = hover_text(src, "name)").unwrap();
    assert!(
        field.contains("field of `User`") && field.contains("`String`"),
        "{field}"
    );
    let method = hover_text(src, "plus(").unwrap();
    assert!(
        method.contains("method of a `Date`") && method.contains("Gives `Date`"),
        "{method}"
    );
    let case = hover_text(src, "loud\n").unwrap();
    assert!(
        case.contains("case of `Tone`") && case.contains("`.failed`"),
        "{case}"
    );
    let image = hover_text(src, "hero, alt").unwrap();
    assert!(
        image.contains("image from `h.jpg`") && image.contains("`.width`"),
        "{image}"
    );
}

#[test]
fn hover_on_every_keyword_explains_it() {
    for (src, word) in [
        (
            format!(
                "{HEAD}    state e = \"\"\n    validate e {{ required }}\n    Input(bind: e, label: \"E\")\n}}\n"
            ),
            "validate",
        ),
        (
            format!("{HEAD}    socket s = ws(\"wss://x\")\n}}\n"),
            "socket",
        ),
        (format!("{HEAD}    every(1000) {{ log(1) }}\n}}\n"), "every"),
        (
            format!("{HEAD}    head {{ meta(name: \"a\", content: \"b\") }}\n}}\n"),
            "head",
        ),
        ("data posts = \"p.json\"\n".to_string(), "data"),
        (
            "api B(base: \"/x\") {\n    get a() -> Map\n}\n".to_string(),
            "api",
        ),
        (
            "test \"t\" {\n    Text(\"a\")\n    expect \"a\"\n}\n".to_string(),
            "test",
        ),
    ] {
        let doc = hover_text(&src, word).unwrap_or_else(|| panic!("no hover on `{word}`"));
        assert!(doc.contains(&format!("**{word}**")), "{word}: {doc}");
    }
}

// ─── Formatting, references, highlights ───────────────────────────────────

#[test]
fn formatting_is_what_wf_fmt_writes() {
    let src = "page P(path: \"/\"){\n  Heading(\"H\").h1\n\n\n  Text(\"a\")\n}";
    let project = project(src);
    let edits = wf_lsp::formatting::format_document(&project, 0).unwrap();
    let want = webfluent::fmt::format_source(src, "test.wf").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, want);
    assert_eq!(edits[0].range.start, Position::new(0, 0));
    // Formatted already: nothing to do.
    let done = project_of(&want);
    assert_eq!(
        wf_lsp::formatting::format_document(&done, 0).unwrap(),
        Vec::new()
    );
    // A file that does not parse is left as it is.
    assert!(
        wf_lsp::formatting::format_document(&project_of("page P(path: \"/\") { Text(\"a\" }"), 0)
            .is_none()
    );
    // An indented file is formatted as one.
    let wfx = Project::single(
        Url::parse("file:///t.wfx").unwrap(),
        "page P(path: \"/\")\n  Text(\"a\")\n",
    );
    let edits = wf_lsp::formatting::format_document(&wfx, 0).unwrap();
    assert_eq!(edits[0].new_text, "page P(path: \"/\")\n    Text(\"a\")\n");
}

fn project_of(src: &str) -> Project {
    project(src)
}

#[test]
fn references_and_highlights_are_every_place_a_name_is_named() {
    let src = "const LIMIT = 3\npage P(path: \"/\") {\n    state count = 0\n    Text(\"{count} of {LIMIT}\")\n    Button(\"+\") { on click { count = count + 1 } }\n    Text(\"count\")\n}\ncomponent C {\n    state count = 1\n    Text(\"{count}\")\n}\n";
    let project = project(src);
    let lines = |locations: Vec<Location>| -> Vec<(u32, u32)> {
        locations
            .iter()
            .map(|l| (l.range.start.line + 1, l.range.start.character))
            .collect()
    };
    let at = |needle: &str, nth: usize| {
        let offset = src.match_indices(needle).nth(nth).unwrap().0 + 1;
        project.files[0].index.offset_to_position(src, offset)
    };
    // The page's `count`: its declaration, the splice, both sides of the
    // assignment — not the text "count", not the component's own.
    let all = lines(wf_lsp::rename::references(
        &project,
        0,
        at("count", 1),
        true,
    ));
    assert_eq!(all, vec![(3, 10), (4, 11), (5, 29), (5, 37)], "{all:?}");
    let uses = lines(wf_lsp::rename::references(
        &project,
        0,
        at("count", 1),
        false,
    ));
    assert_eq!(uses, vec![(4, 11), (5, 29), (5, 37)]);
    // A constant, from a use inside a splice.
    let limit = lines(wf_lsp::rename::references(
        &project,
        0,
        at("LIMIT", 1),
        true,
    ));
    assert_eq!(limit, vec![(1, 6), (4, 22)]);
    let marks = wf_lsp::rename::highlights(&project, 0, at("count", 1));
    assert_eq!(marks.len(), 4);
    // Text names nothing.
    assert!(wf_lsp::rename::references(&project, 0, at("count", 4), true).is_empty());
}

// ─── Signature help ───────────────────────────────────────────────────────

/// The signature shown at `‸` in `broken` (`valid` the last good parse), and
/// the label of the argument it marks.
fn signature_at(valid: &str, broken: &str) -> Option<(String, Option<String>)> {
    let at = broken.find('‸').unwrap();
    let text = broken.replacen('‸', "", 1);
    let project = mid_edit(valid, &text);
    let pos = project.files[0].index.offset_to_position(&text, at);
    let help = wf_lsp::signature::signature_help(&project, 0, pos)?;
    let sig = help.signatures.into_iter().next()?;
    let active = help.active_parameter.and_then(|i| {
        sig.parameters
            .as_ref()?
            .get(i as usize)
            .map(|p| match &p.label {
                ParameterLabel::Simple(s) => s.clone(),
                _ => String::new(),
            })
    });
    Some((sig.label, active))
}

#[test]
fn signature_help_shows_what_a_call_takes_and_where_the_cursor_is() {
    let decls = "component UserCard(_ name: String, role: String, active: Bool = true) { Text(name) }\napi Backend(base: \"/api\") {\n    get user(id: String) at \"users/:id\" -> Map\n}\nstore Cart {\n    state n = 0\n    action add(item: Map, qty: Number) { n = n + qty }\n}\n";
    let page = |valid: &str, broken: &str| {
        signature_at(
            &format!("{decls}{HEAD}    use Cart\n{valid}\n}}\n"),
            &format!("{decls}{HEAD}    use Cart\n{broken}\n}}\n"),
        )
    };
    let (label, active) = page(
        "    UserCard(\"Ada\", role: \"Dev\")",
        "    UserCard(\"Ada\", role: ‸)",
    )
    .unwrap();
    assert_eq!(
        label,
        "UserCard(_ name: String, role: String, active: Bool = …)"
    );
    assert_eq!(active.as_deref(), Some("role: String"));
    let (_, active) = page("    UserCard(\"Ada\")", "    UserCard(‸)").unwrap();
    assert_eq!(active.as_deref(), Some("_ name: String"));
    let (label, _) = page(
        "    Row(gap: .md) { Text(\"a\") }",
        "    Row(gap: ‸) { Text(\"a\") }",
    )
    .unwrap();
    assert!(
        label.starts_with("Row(") && label.contains("gap: ") && label.contains(".md"),
        "{label}"
    );
    let (label, active) = page(
        "    Button(\"x\") { on click { Backend.user(id: \"1\") } }",
        "    Button(\"x\") { on click { Backend.user(‸) } }",
    )
    .unwrap();
    assert_eq!(label, "Backend.user(id: String) -> Map");
    assert_eq!(active.as_deref(), Some("id: String"));
    let (label, active) = page(
        "    Button(\"x\") { on click { Cart.add({}, 1) } }",
        "    Button(\"x\") { on click { Cart.add({}, ‸) } }",
    )
    .unwrap();
    assert_eq!(label, "Cart.add(item: Map, qty: Number)");
    assert_eq!(active.as_deref(), Some("qty: Number"));
    let (label, active) = page("    Text(format(3, .currency))", "    Text(format(3, ‸))").unwrap();
    assert_eq!(label, "format(value, .style, option)");
    assert_eq!(active.as_deref(), Some(".style"));
    // Outside every parenthesis: none.
    assert!(page("    Text(\"a\")", "    ‸Text(\"a\")").is_none());
}
