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
fn hover_on_a_builtin_says_what_the_reference_says() {
    let src =
        "Page Home (path: \"/\") {\n    Column(span: 6) { Stack(gap: md) { Spacer(sm) } }\n}\n";
    let column = hover_text(src, "Column").unwrap();
    assert!(column.contains("12-column grid"), "{column}");
    assert!(column.contains("`span:`"), "{column}");
    let stack = hover_text(src, "Stack").unwrap();
    assert!(stack.contains("Vertical flex"), "{stack}");
    let spacer = hover_text(src, "Spacer").unwrap();
    assert!(spacer.contains("Vertical space"), "{spacer}");
    assert!(spacer.contains("<div>"), "{spacer}");
}

#[test]
fn hover_on_a_modifier_names_the_element_it_modifies() {
    let src = "Page Home (path: \"/\") {\n    Button(\"Save\", primary, large)\n}\n";
    let primary = hover_text(src, "primary").unwrap();
    assert!(primary.contains("Color modifier on `Button`"), "{primary}");
    assert!(primary.contains("`Button` lists it"), "{primary}");
}

#[test]
fn hover_on_a_prop_that_shadows_a_modifier_word_is_the_prop() {
    let src = "Component Chip (text: String) {\n    Text(text, bold)\n}\n";
    let text = hover_text(src, "text, bold").unwrap();
    assert!(text.contains("prop"), "{text}");
    assert!(!text.contains("Input type"), "{text}");
}

#[test]
fn hover_inside_a_string_or_comment_is_nothing() {
    let src = "Page Home (path: \"/\") {\n    // a Button in a comment\n    Text(\"Button\")\n}\n";
    assert!(hover_text(src, "Button in a").is_none());
    assert!(hover_text(src, "Button\")").is_none());
}

#[test]
fn hover_on_a_name_resolves_in_the_enclosing_declaration_not_the_first() {
    let src = "Page A (path: \"/a\") {\n    state count = 1\n}\nPage B (path: \"/b\") {\n    derived count = total * 2\n    Text(\"{count}\")\n}\n";
    let b_count = hover_text(src, "count = total").unwrap();
    assert!(b_count.contains("derived"), "{b_count}");
    assert!(b_count.contains("derived count = total * 2"), "{b_count}");
}

#[test]
fn hover_on_a_store_member_finds_the_store() {
    let src = "Store CartStore {\n    state items = []\n    action clear() { items = [] }\n}\nPage Shop (path: \"/\") {\n    use CartStore\n    Button(\"Clear\") { CartStore.clear() }\n}\n";
    let clear = hover_text(src, "clear() }").unwrap();
    assert!(clear.contains("action"), "{clear}");
    assert!(clear.contains("Member of store `CartStore`"), "{clear}");
    let store = hover_text(src, "CartStore.clear").unwrap();
    assert!(store.contains("store"), "{store}");
    assert!(store.contains("`items` — state"), "{store}");
}

#[test]
fn hover_on_a_named_argument_explains_it_for_that_component() {
    let src = "Page Home (path: \"/\") {\n    Input(text, bind: name, aria-label: \"Name\")\n}\n";
    let bind = hover_text(src, "bind:").unwrap();
    assert!(bind.contains("argument of `Input`"), "{bind}");
    let aria = hover_text(src, "aria-label").unwrap();
    assert!(aria.contains("attribute on `Input`"), "{aria}");
}

#[test]
fn hover_on_an_event_and_a_pseudo_state() {
    let src = "Page Home (path: \"/\") {\n    Button(\"x\") {\n        on:click { go() }\n        style { hover { background: \"red\" } }\n    }\n}\n";
    let click = hover_text(src, "on:click").unwrap();
    assert!(click.contains("event handler"), "{click}");
    let hover = hover_text(src, "hover {").unwrap();
    assert!(hover.contains("pseudo-state"), "{hover}");
}

#[test]
fn hover_on_a_user_component_shows_its_props_and_a_route_target_its_page() {
    let src = "Component UserCard (name: String, active?: Bool) {\n    Text(name)\n}\nPage Home (path: \"/\", title: \"Home\") {\n    UserCard(name: \"x\")\n}\nApp {\n    Router { Route(path: \"/\", page: Home) }\n}\n";
    let card = hover_text(src, "UserCard(name").unwrap();
    assert!(
        card.contains("Component UserCard (name: String, active?: Bool)"),
        "{card}"
    );
    let page = hover_text(src, "page: Home").map(|_| ()).is_some();
    let home = hover_text(src, "Home)").unwrap();
    assert!(page && home.contains("page at `/`"), "{home}");
}

#[test]
fn hover_with_arabic_text_on_the_line_lands_on_the_right_word() {
    let src = "Page Welcome (path: \"/\") {\n    Text(\"مرحباً بك في WebFluent! 🚀\", muted)\n    Button(\"ابدأ الآن\", primary)\n}\n";
    let muted = hover_text(src, "muted").unwrap();
    assert!(muted.contains("Typography modifier on `Text`"), "{muted}");
    let primary = hover_text(src, "primary").unwrap();
    assert!(primary.contains("on `Button`"), "{primary}");
    // The hover's own range covers exactly the word.
    let project = project(src);
    let h = provide_hover(&project, 0, at(src, "primary")).unwrap();
    let r = h.range.unwrap();
    assert_eq!(r.end.character - r.start.character, "primary".len() as u32);
}

// ─── Completion ───────────────────────────────────────────────────────────

#[test]
fn completion_inside_an_element_offers_its_own_arguments_first() {
    let src = "Page Home (path: \"/\") {\n    Slider(bind: v, )\n}\n";
    let items = labels_after(src, "bind: v, ");
    assert!(items.contains(&"min:".to_string()), "{items:?}");
    assert!(items.contains(&"step:".to_string()), "{items:?}");
    assert!(
        !items.contains(&"src:".to_string()),
        "Slider takes no src: {items:?}"
    );
    assert!(items.contains(&"v".to_string()) || !items.contains(&"Card".to_string()));
}

#[test]
fn completion_inside_a_user_component_call_offers_its_props() {
    let src = "Component UserCard (name: String, role: String) {\n    Text(name)\n}\nPage Home (path: \"/\") {\n    UserCard()\n}\n";
    let items = labels_after(src, "UserCard(");
    assert_eq!(items, vec!["name:", "role:"]);
}

#[test]
fn completion_after_a_dot_offers_sub_components_or_store_members() {
    let valid = "Store CartStore {\n    state items = []\n    action clear() { items = [] }\n}\nPage Home (path: \"/\") {\n    use CartStore\n    Card { }\n    Button(\"x\") { }\n}\n";
    let broken = valid
        .replace("Card { }", "Card { Card. }")
        .replace("Button(\"x\") { }", "Button(\"x\") { CartStore. }");
    let card = labels_mid_edit(valid, &broken, "Card. ");
    assert_eq!(card, vec!["Header", "Body", "Footer"]);
    let store = labels_mid_edit(valid, &broken, "CartStore. ");
    assert_eq!(store, vec!["items", "clear"]);
}

#[test]
fn completion_in_a_body_offers_components_keywords_and_scope() {
    let src =
        "Page Home (path: \"/\") {\n    state count = 0\n    Container {\n        \n    }\n}\n";
    let items = labels_after(src, "Container {\n        ");
    assert!(items.contains(&"Button".to_string()));
    assert!(items.contains(&"if".to_string()));
    assert!(items.contains(&"count".to_string()));
    assert!(
        !items.contains(&"Page".to_string()),
        "declarations are not statements: {items:?}"
    );
}

#[test]
fn completion_in_a_style_block_offers_css_and_tokens() {
    let src = "Page Home (path: \"/\") {\n    Card {\n        style {\n            \n            color: \n        }\n    }\n}\n";
    let props = labels_after(src, "style {\n            ");
    assert!(props.contains(&"border-radius".to_string()), "{props:?}");
    assert!(props.contains(&"hover".to_string()), "{props:?}");
    assert!(props.contains(&"@media".to_string()), "{props:?}");
    assert!(!props.contains(&"Button".to_string()), "{props:?}");
    let values = labels_after(src, "color: ");
    assert!(
        values.contains(&"var(--color-primary)".to_string()),
        "{values:?}"
    );
}

#[test]
fn completion_in_a_fetch_body_offers_the_missing_blocks() {
    let src = "Page Home (path: \"/\") {\n    fetch users from \"/api\" {\n        loading { Spinner() }\n        \n    }\n}\n";
    let items = labels_after(src, "Spinner() }\n        ");
    assert_eq!(items, vec!["error", "success"]);
}

#[test]
fn completion_offers_nothing_inside_strings_and_comments() {
    let src = "Page Home (path: \"/\") {\n    // \n    Text(\"\")\n}\n";
    assert!(labels_after(src, "// ").is_empty());
    assert!(labels_after(src, "Text(\"").is_empty());
}

#[test]
fn completion_at_top_level_and_in_a_theme() {
    let src = "Theme Brand {\n    \n}\n\n";
    let top = labels_after(src, "}\n\n");
    assert!(top.contains(&"Page".to_string()));
    let theme = labels_after(src, "Brand {\n    ");
    assert!(
        theme.contains(&"token color-primary".to_string()),
        "{theme:?}"
    );
}

#[test]
fn completion_keeps_working_while_the_file_does_not_parse() {
    // The unclosed parenthesis means the file does not parse; the last good
    // parse still supplies the scope, and the element's arguments come from
    // the text.
    let src = "Page Home (path: \"/\") {\n    state count = 0\n    Button(\"x\", \n}\n";
    let project = project(src);
    assert!(project.files[0].parsed.is_err());
    let pos = project.files[0]
        .index
        .offset_to_position(src, src.find("\"x\", ").unwrap() + 5);
    let items: Vec<String> = provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert!(items.contains(&"primary".to_string()), "{items:?}");
}

#[test]
fn completion_after_on_colon_offers_events() {
    let src = "Page Home (path: \"/\") {\n    Button(\"x\") { on: }\n}\n";
    let items = labels_after(src, "on:");
    assert!(items.contains(&"click".to_string()), "{items:?}");
    assert!(items.contains(&"submit".to_string()), "{items:?}");
}

// ─── Definition ───────────────────────────────────────────────────────────

#[test]
fn definition_of_a_name_is_the_one_in_scope() {
    let src = "Page A (path: \"/a\") {\n    state count = 1\n}\nPage B (path: \"/b\") {\n    state count = 2\n    Text(\"{count}\")\n    Button(\"+\") { count = count + 1 }\n}\n";
    let project = project(src);
    let def = find_definition(&project, 0, at(src, "count + 1")).unwrap();
    let GotoDefinitionResponse::Scalar(loc) = def else {
        panic!()
    };
    assert_eq!(loc.range.start.line, 4, "B's count, not A's");
}

#[test]
fn definition_of_a_component_call_and_a_store_member() {
    let src = "Component Nudge (label: String) {\n    Text(label)\n}\nStore S {\n    state n = 0\n    action bump() { n = n + 1 }\n}\nPage Home (path: \"/\") {\n    use S\n    Nudge(label: \"x\")\n    Button(\"b\") { S.bump() }\n}\n";
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
}

// ─── Diagnostics ──────────────────────────────────────────────────────────

#[test]
fn diagnostics_point_at_the_word_even_after_non_ascii_text() {
    let src = "Page Home (path: \"/\", title: \"x\", description: \"y\") {\n    Heading(\"أهلاً\", h1)\n    Text(\"مرحباً بك\", centred)\n}\n";
    let project = project(src);
    let diagnostics = project_diagnostics(&project).remove(0);
    let v01 = diagnostics
        .iter()
        .find(|d| d.message.contains("centred"))
        .expect("V01 for `centred`");
    let start = project.files[0]
        .index
        .offset_to_position(src, src.find("centred").unwrap());
    assert_eq!(v01.range.start, start, "{v01:?}");
    assert_eq!(v01.range.end.character, start.character + 7);
}

#[test]
fn a_parse_error_is_one_error_at_its_position() {
    let src = "Page Home (path: \"/\") {\n    Text(\"x\"\n}\n";
    let project = project(src);
    let diagnostics = project_diagnostics(&project).remove(0);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostics[0].range.start.line, 2);
}

// ─── Symbols ──────────────────────────────────────────────────────────────

#[test]
fn document_symbols_are_nested_under_declarations() {
    let src = "Store CartStore {\n    state items = []\n    action clear() { items = [] }\n}\n";
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
