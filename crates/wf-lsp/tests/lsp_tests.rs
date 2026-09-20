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
        .find(|d| d.message.contains("[T01]"))
        .expect("a type error");
    assert_eq!(t01.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(t01.range.start.line, 1);
    assert!(t01.message.contains("Convert it"), "{}", t01.message);
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
