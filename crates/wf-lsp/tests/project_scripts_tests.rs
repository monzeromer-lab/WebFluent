//! The project's own scripts and stylesheets, as the editor reads them:
//! a name a `.js` file under `src/` declares is in scope, typed, documented
//! and one jump away; a class a `.css` file defines is offered inside
//! `class: "…"`.

use std::path::PathBuf;
use tower_lsp::lsp_types::*;
use wf_lsp::completion::provide_completions;
use wf_lsp::definition::find_definition;
use wf_lsp::diagnostics::project_diagnostics;
use wf_lsp::hover::provide_hover;
use wf_lsp::project::{FileCache, Project};

/// A project on disk; the file to open in it, and the project.
fn project(name: &str, wf: &str) -> (PathBuf, Project, usize) {
    let dir = std::env::temp_dir().join(format!("wf-lsp-scripts-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/lib")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "s" }"#).unwrap();
    std::fs::write(
        dir.join("src/lib/money.js"),
        "/**\n * Money, in dollars.\n * @param {number} n\n * @returns {string}\n */\nfunction money(n) { return \"$\" + n }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/cards.css"),
        ".feature { padding: 1rem }\n.feature--first::before { content: \"I\" }\n",
    )
    .unwrap();
    let path = dir.join("src/App.wf");
    std::fs::write(&path, wf).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let project = Project::load(&uri, &|_| None, &FileCache::default());
    let ix = project.file_index(&uri).unwrap();
    (dir, project, ix)
}

fn at(project: &Project, ix: usize, needle: &str, plus: usize) -> Position {
    let file = &project.files[ix];
    let offset = file
        .source
        .find(needle)
        .unwrap_or_else(|| panic!("{needle}"))
        + plus;
    file.index.offset_to_position(&file.source, offset)
}

const PAGE: &str = "page Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(\"H\").h1\n    Text(money(12))\n    Card(class: [\"feature\", { \"feature--first\": true }]) { Text(\"x\") }\n    Text(mo)\n}\n";

#[test]
fn a_script_name_is_in_scope_typed_and_one_jump_away() {
    let (dir, project, ix) = project("names", PAGE.replace("    Text(mo)\n", "").as_str());
    let diagnostics = project_diagnostics(&project);
    assert!(
        diagnostics[ix].iter().all(|d| !d.message.contains("T13")),
        "{:?}",
        diagnostics[ix]
    );

    let hover = provide_hover(&project, ix, at(&project, ix, "money(12)", 1)).expect("a hover");
    let HoverContents::Markup(text) = hover.contents else {
        panic!()
    };
    assert!(
        text.value.contains("money(n: Number) -> String"),
        "{}",
        text.value
    );
    assert!(text.value.contains("Money, in dollars."), "{}", text.value);

    let Some(GotoDefinitionResponse::Scalar(loc)) =
        find_definition(&project, ix, at(&project, ix, "money(12)", 1))
    else {
        panic!("a definition")
    };
    assert!(loc.uri.path().ends_with("src/lib/money.js"), "{loc:?}");
    assert_eq!(loc.range.start, Position::new(5, 9));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_script_name_is_offered_with_how_a_call_reads() {
    let (dir, project, ix) = project("complete", PAGE);
    let items = provide_completions(&project, ix, at(&project, ix, "Text(mo)", 7));
    let money = items
        .iter()
        .find(|i| i.label == "money")
        .expect("money is offered");
    assert_eq!(money.kind, Some(CompletionItemKind::FUNCTION));
    assert!(
        money
            .detail
            .as_deref()
            .unwrap_or("")
            .contains("money(n: Number) -> String")
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_class_is_offered_shown_and_found_inside_class() {
    let (dir, project, ix) = project("classes", PAGE);
    // In the string, in a map key.
    for needle in ["\"feature\"", "\"feature--first\""] {
        let items = provide_completions(&project, ix, at(&project, ix, needle, 2));
        let names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(names, ["feature", "feature--first"], "{needle}");
    }
    // Not in any other string.
    assert!(provide_completions(&project, ix, at(&project, ix, "\"H\"", 1)).is_empty());

    let hover =
        provide_hover(&project, ix, at(&project, ix, "\"feature--first\"", 3)).expect("a hover");
    let HoverContents::Markup(text) = hover.contents else {
        panic!()
    };
    assert!(
        text.value.contains(".feature--first::before"),
        "{}",
        text.value
    );

    let Some(GotoDefinitionResponse::Scalar(loc)) =
        find_definition(&project, ix, at(&project, ix, "\"feature--first\"", 3))
    else {
        panic!("a definition")
    };
    assert!(loc.uri.path().ends_with("src/cards.css"), "{loc:?}");
    assert_eq!(loc.range.start, Position::new(1, 1));
    let _ = std::fs::remove_dir_all(dir);
}
