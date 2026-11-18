//! A project's script seen from the script: what it declares is used from
//! `.wf` files no JavaScript tool reads, and the server says where — find
//! references, highlights, a hover, a lens on each name, a faded hint on a
//! name nothing uses, and a rename that reaches the pages.

use lsp_types::*;
use std::path::{Path, PathBuf};
use wf_lsp::project::{FileCache, Project};
use wf_lsp::scripts;

const SCRIPT: &str = "/** @param {number} n @returns {string} */\nfunction money(n) { return \"$\" + helper(n) }\n\n/** @param {number} n @returns {string} */\nfunction helper(n) { return String(n) }\n\n/** @returns {string} */\nfunction forgotten() { return \"\" }\n";

const PAGE: &str = "page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    Text(money(3))\n    Text(\"money\")\n    Button(\"Pay\") { on click { log(money(4)) } }\n}\n";

fn project(name: &str) -> (PathBuf, Project) {
    let dir = std::env::temp_dir().join(format!("wf-lsp-uses-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).expect("the fixture is as written");
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "u" }"#)
        .expect("the fixture is as written");
    std::fs::write(dir.join("src/money.js"), SCRIPT).expect("the fixture is as written");
    std::fs::write(dir.join("src/pages/Home.wf"), PAGE).expect("the fixture is as written");
    let uri = Url::from_file_path(dir.join("src/money.js")).expect("the fixture is as written");
    let project = Project::load_with(&uri, &webfluent::vfs::FsVfs, &FileCache::default());
    (dir, project)
}

fn ix_of(project: &Project, path: &Path) -> usize {
    project
        .files
        .iter()
        .position(|f| f.path == path)
        .expect("the file is in the project")
}

/// Where `needle` first starts in the file, plus `plus` characters.
fn at(project: &Project, ix: usize, needle: &str, plus: u32) -> Position {
    let file = &project.files[ix];
    let offset = file.source.find(needle).expect("the needle is there");
    let mut p = file.index.offset_to_position(&file.source, offset);
    p.character += plus;
    p
}

fn lines(found: &[Location], uri_ends: &str) -> Vec<u32> {
    found
        .iter()
        .filter(|l| l.uri.path().ends_with(uri_ends))
        .map(|l| l.range.start.line)
        .collect()
}

#[test]
fn a_script_s_function_is_found_where_the_pages_call_it() {
    let (dir, project) = project("refs");
    let js = ix_of(&project, &dir.join("src/money.js"));
    assert!(scripts::is_script(&project, js));
    let found = scripts::references(&project, js, at(&project, js, "money(n)", 1), true);
    // The declaration, and the two calls on the page — not the word in a
    // string.
    assert_eq!(lines(&found, "money.js"), vec![1]);
    assert_eq!(lines(&found, "Home.wf"), vec![2, 4]);
    let without = scripts::references(&project, js, at(&project, js, "money(n)", 1), false);
    assert_eq!(without.len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_function_only_another_function_calls_is_used_and_one_nothing_calls_is_faded() {
    let (dir, project) = project("unused");
    let js = ix_of(&project, &dir.join("src/money.js"));
    let unused = scripts::unused(&project, js);
    assert_eq!(unused.len(), 1, "{unused:?}");
    assert!(unused[0].message.contains("`forgotten`"));
    assert_eq!(unused[0].tags, Some(vec![DiagnosticTag::UNNECESSARY]));
    assert_eq!(unused[0].severity, Some(DiagnosticSeverity::HINT));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn each_name_has_a_lens_saying_where_it_is_used() {
    let (dir, project) = project("lens");
    let js = ix_of(&project, &dir.join("src/money.js"));
    let titles: Vec<String> = scripts::lenses(&project, js)
        .into_iter()
        .map(|l| l.command.expect("the fixture is as written").title)
        .collect();
    assert_eq!(
        titles,
        [
            "Used 2 times in WebFluent",
            "Used once in scripts",
            "Not used"
        ]
    );
    let named = scripts::script_names(&project, js);
    assert_eq!(named[0].name, "money");
    assert_eq!(named[0].uses.len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_hover_lists_the_places() {
    let (dir, project) = project("hover");
    let js = ix_of(&project, &dir.join("src/money.js"));
    let hover = scripts::hover(&project, js, at(&project, js, "money(n)", 2))
        .expect("the fixture is as written");
    let HoverContents::Markup(m) = hover.contents else {
        panic!()
    };
    assert!(
        m.value.contains("**money**: Used 2 times in WebFluent"),
        "{}",
        m.value
    );
    assert!(m.value.contains("src/pages/Home.wf:3"), "{}", m.value);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_rename_from_the_script_reaches_the_pages() {
    let (dir, project) = project("rename");
    let js = ix_of(&project, &dir.join("src/money.js"));
    let edit = scripts::rename(&project, js, at(&project, js, "money(n)", 0), "price")
        .expect("the fixture is as written");
    let changes = edit.changes.expect("the fixture is as written");
    let count = |end: &str| {
        changes
            .iter()
            .filter(|(u, _)| u.path().ends_with(end))
            .map(|(_, e)| e.len())
            .sum::<usize>()
    };
    assert_eq!(count("money.js"), 1);
    assert_eq!(count("Home.wf"), 2);
    // From the page, the rename reaches the script too.
    let home = ix_of(&project, &dir.join("src/pages/Home.wf"));
    let edit = wf_lsp::rename::rename(&project, home, at(&project, home, "money(3)", 1), "price")
        .expect("the fixture is as written");
    let changes = edit.changes.expect("the fixture is as written");
    assert!(
        changes.keys().any(|u| u.path().ends_with("money.js")),
        "{changes:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_script_outside_any_project_is_not_read_as_webfluent() {
    let dir = std::env::temp_dir().join(format!("wf-lsp-stray-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the fixture is as written");
    std::fs::write(dir.join("tool.js"), "const x = { a: 1 };\n")
        .expect("the fixture is as written");
    let uri = Url::from_file_path(dir.join("tool.js")).expect("the fixture is as written");
    let project = Project::load_with(&uri, &webfluent::vfs::FsVfs, &FileCache::default());
    assert!(project.files.is_empty());
    assert!(
        wf_lsp::diagnostics::project_diagnostics(&project)
            .iter()
            .all(Vec::is_empty)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
