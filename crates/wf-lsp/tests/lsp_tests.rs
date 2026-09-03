use tower_lsp::lsp_types::*;
use webfluent::lexer::Lexer;
use webfluent::parser::{Parser, Program};
use wf_lsp::code_actions::provide_code_actions;
use wf_lsp::completion::provide_completions;
use wf_lsp::definition::find_definition;
use wf_lsp::hover::provide_hover;
use wf_lsp::line_index::LineIndex;
use wf_lsp::symbols::build_document_symbols;

fn parse_program(src: &str) -> Program {
    let tokens = Lexer::new(src, "test.wf").tokenize().expect("lex");
    Parser::new(tokens, "test.wf").parse().expect("parse")
}

#[test]
fn test_hover_builtin_component() {
    let src = "Page Home (path: \"/\") {\n  Button(\"Click Me\", primary)\n}\n";
    let prog = parse_program(src);

    // Hover over 'Button' (line 1, col 2)
    let pos = Position::new(1, 4);
    let hover = provide_hover(src, pos, Some(&prog)).expect("hover on Button");

    match &hover.contents {
        HoverContents::Markup(m) => {
            assert_eq!(m.kind, MarkupKind::Markdown);
            assert!(m.value.contains("<button>"));
            assert!(m.value.contains(".wf-btn"));
        }
        _ => panic!("Expected markdown"),
    }
}

#[test]
fn test_hover_slides_component() {
    let src = "Page Deck (path: \"/slides\") {\n  Presentation (title: \"Talk\") {\n    Slide {\n      TwoColumn {\n        Container { Text(\"Left\") }\n        Container { Text(\"Right\") }\n      }\n    }\n  }\n}\n";
    let prog = parse_program(src);

    let pos = Position::new(3, 8); // 'TwoColumn'
    let hover = provide_hover(src, pos, Some(&prog)).expect("hover on TwoColumn");

    match &hover.contents {
        HoverContents::Markup(m) => {
            assert!(m.value.contains("TwoColumn"));
            assert!(m.value.contains("Two-column slide"));
        }
        _ => panic!("Expected markdown"),
    }
}

#[test]
fn test_hover_subcomponent() {
    let src = "Page Home (path: \"/\") {\n  Sidebar {\n    Sidebar.Item(\"Overview\", to: \"/overview\")\n  }\n}\n";
    let prog = parse_program(src);

    let pos = Position::new(2, 14); // 'Sidebar.Item'
    let hover = provide_hover(src, pos, Some(&prog)).expect("hover on Sidebar.Item");

    match &hover.contents {
        HoverContents::Markup(m) => {
            assert!(m.value.contains("Sidebar.Item"));
        }
        _ => panic!("Expected markdown"),
    }
}

#[test]
fn test_hover_canonical_modifier() {
    let src = "Page Home (path: \"/\") {\n  Button(\"Submit\", primary, fadeIn)\n}\n";
    let prog = parse_program(src);

    // Hover over 'primary' (line 1, col 20)
    let pos = Position::new(1, 20);
    let hover = provide_hover(src, pos, Some(&prog)).expect("hover on primary");

    match &hover.contents {
        HoverContents::Markup(m) => {
            assert!(m.value.contains("Color Variant"));
            assert!(m.value.contains(".wf-*--primary"));
        }
        _ => panic!("Expected markdown"),
    }

    // Hover over 'fadeIn'
    let pos_anim = Position::new(1, 30);
    let hover_anim = provide_hover(src, pos_anim, Some(&prog)).expect("hover on fadeIn");

    match &hover_anim.contents {
        HoverContents::Markup(m) => {
            assert!(m.value.contains("Animation"));
            assert!(m.value.contains("0% to 100%"));
        }
        _ => panic!("Expected markdown"),
    }
}

#[test]
fn test_hover_keywords() {
    let src = "Theme Brand {\n  token color-primary: \"#0F766E\"\n}\n";
    let prog = parse_program(src);

    // Hover on 'Theme'
    let pos = Position::new(0, 2);
    let hover = provide_hover(src, pos, Some(&prog)).expect("hover on Theme");

    match &hover.contents {
        HoverContents::Markup(m) => {
            assert!(m.value.contains("Theme"));
            assert!(m.value.contains("design tokens"));
        }
        _ => panic!("Expected markdown"),
    }
}

#[test]
fn test_hover_user_ast_symbols() {
    let src = "Store Counter {\n  state count = 0\n  action increment() { count = count + 1 }\n}\nComponent CardItem (title: String, active: Bool = true) {\n  Text(title)\n}\n";
    let prog = parse_program(src);

    // Hover on Component CardItem
    let pos_comp = Position::new(4, 12);
    let hover_comp = provide_hover(src, pos_comp, Some(&prog)).expect("hover on CardItem");
    match &hover_comp.contents {
        HoverContents::Markup(m) => {
            assert!(m.value.contains("Component CardItem(title: String, active: Bool"));
        }
        _ => panic!("Expected markdown"),
    }

    // Hover on state count
    let pos_state = Position::new(1, 9);
    let hover_state = provide_hover(src, pos_state, Some(&prog)).expect("hover on count");
    match &hover_state.contents {
        HoverContents::Markup(m) => {
            assert!(m.value.contains("count"));
            assert!(m.value.contains("Reactive state signal"));
            assert!(m.value.contains("Counter"));
        }
        _ => panic!("Expected markdown"),
    }
}

#[test]
fn test_completions_inside_braces() {
    let src = "Page Home (path: \"/\") {\n  \n}\n";
    let prog = parse_program(src);

    let pos = Position::new(1, 2);
    let completions = provide_completions(src, pos, Some(&prog));

    // Must include layout and action components and keywords
    assert!(completions.iter().any(|c| c.label == "Button"));
    assert!(completions.iter().any(|c| c.label == "Container"));
    assert!(completions.iter().any(|c| c.label == "state"));
    assert!(completions.iter().any(|c| c.label == "Presentation"));
}

#[test]
fn test_completions_subcomponents() {
    let src = "Page Home (path: \"/\") {\n  Sidebar.\n}\n";

    let pos = Position::new(1, 10);
    let completions = provide_completions(src, pos, None);

    assert!(completions.iter().any(|c| c.label == "Sidebar.Item"));
    assert!(completions.iter().any(|c| c.label == "Sidebar.Header"));
    assert!(completions.iter().any(|c| c.label == "Sidebar.Divider"));
    // Invented/non-existent ones should NOT be present
    assert!(!completions.iter().any(|c| c.label == "Sidebar.Content"));
    assert!(!completions.iter().any(|c| c.label == "Sidebar.Footer"));
}

#[test]
fn test_completions_inside_parens() {
    let src = "Page Home (path: \"/\") {\n  Button(\n}\n";

    let pos = Position::new(1, 9);
    let completions = provide_completions(src, pos, None);

    // Modifiers and named args
    assert!(completions.iter().any(|c| c.label == "primary"));
    assert!(completions.iter().any(|c| c.label == "fadeIn"));
    assert!(completions.iter().any(|c| c.label == "bind:"));
    assert!(completions.iter().any(|c| c.label == "path:"));
}

#[test]
fn test_completions_inside_theme() {
    let src = "Theme Brand {\n  \n}\n";

    let pos = Position::new(1, 2);
    let completions = provide_completions(src, pos, None);

    assert!(completions.iter().any(|c| c.label == "token color-primary"));
    assert!(completions.iter().any(|c| c.label == "token radius-md"));
}

#[test]
fn test_document_symbols() {
    let src = "Theme Brand {\n  token color-primary: \"#000\"\n}\nPage Home (path: \"/\") {\n  state count = 0\n  Button(\"Click\")\n}\n";
    let prog = parse_program(src);
    let index = LineIndex::new(src);
    let uri = Url::parse("file:///test.wf").unwrap();

    let resp = build_document_symbols(&prog, src, &index, &uri, true);
    match resp {
        DocumentSymbolResponse::Nested(symbols) => {
            assert_eq!(symbols.len(), 2);
            assert_eq!(symbols[0].name, "Brand");
            assert_eq!(symbols[0].kind, SymbolKind::NAMESPACE);
            assert!(symbols[0].children.is_some());

            assert_eq!(symbols[1].name, "Home");
            assert_eq!(symbols[1].kind, SymbolKind::CLASS);
            let page_children = symbols[1].children.as_ref().unwrap();
            assert!(page_children.iter().any(|c| c.name == "count" && c.kind == SymbolKind::VARIABLE));
            assert!(page_children.iter().any(|c| c.name == "Button" && c.kind == SymbolKind::FIELD));
        }
        DocumentSymbolResponse::Flat(_) => panic!("Expected nested symbols"),
    }
}

#[test]
fn test_goto_definition() {
    let src = "Component UserBadge (name: String) {\n  Text(name)\n}\nPage Home (path: \"/\") {\n  UserBadge(name: \"Alice\")\n}\n";
    let prog = parse_program(src);
    let index = LineIndex::new(src);
    let uri = Url::parse("file:///test.wf").unwrap();

    // Click on 'UserBadge' at line 4, col 4
    let pos = Position::new(4, 4);
    let def = find_definition(&prog, src, &index, &uri, pos).expect("definition for UserBadge");

    match def {
        GotoDefinitionResponse::Scalar(loc) => {
            assert_eq!(loc.uri, uri);
            assert_eq!(loc.range.start.line, 0); // Component UserBadge starts on line 0
        }
        _ => panic!("Expected scalar location"),
    }
}

#[test]
fn test_code_actions_quickfix() {
    let uri = Url::parse("file:///test.wf").unwrap();
    let diag = Diagnostic {
        range: Range::new(Position::new(1, 10), Position::new(1, 18)),
        message: "'centered' is not a modifier; did you mean `center`?".to_string(),
        severity: Some(DiagnosticSeverity::WARNING),
        ..Default::default()
    };

    let params = CodeActionParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        range: diag.range,
        context: CodeActionContext {
            diagnostics: vec![diag],
            only: None,
            trigger_kind: None,
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };

    let actions = provide_code_actions(&uri, params);
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        CodeActionOrCommand::CodeAction(action) => {
            assert_eq!(action.title, "Change to `center`");
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            let edit = action.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            let text_edits = changes.get(&uri).unwrap();
            assert_eq!(text_edits[0].new_text, "center");
        }
        _ => panic!("Expected code action"),
    }
}
