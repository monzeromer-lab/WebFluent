//! The server over real projects in the repository: the documentation site
//! and the fixture projects, each assembled from its files the way `wf build`
//! assembles it.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tower_lsp::lsp_types::*;
use wf_lsp::completion::provide_completions;
use wf_lsp::definition::find_definition;
use wf_lsp::diagnostics::project_diagnostics;
use wf_lsp::hover::provide_hover;
use wf_lsp::project::{FileCache, Project};
use wf_lsp::symbols::{document_symbols, workspace_symbols};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// The project a file belongs to, read from the disk.
fn open(path: &Path) -> (Project, usize) {
    let uri = Url::from_file_path(path).unwrap();
    let project = Project::load(&uri, &|_| None, &FileCache::default());
    let ix = project.file_index(&uri).expect("file is in its project");
    (project, ix)
}

fn position_of(project: &Project, ix: usize, needle: &str) -> Position {
    let file = &project.files[ix];
    let offset = file
        .source
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} in {:?}", file.path));
    file.index.offset_to_position(&file.source, offset)
}

#[test]
fn a_fixture_project_resolves_across_its_files() {
    let root = workspace_root().join("tests/fixtures/dashboard");
    let (project, ix) = open(&root.join("src/pages/Incidents.wf"));

    // Every file under src/ is part of the project, App.wf first.
    assert!(
        project.files.len() >= 6,
        "{:?}",
        project.files.iter().map(|f| &f.path).collect::<Vec<_>>()
    );
    assert!(project.files[0].path.ends_with("App.wf"));

    // The store declared in stores/incidents.wf is known to the page.
    let store = provide_hover(
        &project,
        ix,
        position_of(&project, ix, "IncidentStore.report"),
    )
    .unwrap();
    let HoverContents::Markup(m) = store.contents else {
        panic!()
    };
    assert!(m.value.contains("store"), "{}", m.value);

    // Its member has a definition in the other file.
    let def = find_definition(&project, ix, position_of(&project, ix, "report(draft)")).unwrap();
    let GotoDefinitionResponse::Scalar(loc) = def else {
        panic!()
    };
    assert!(
        loc.uri.path().ends_with("stores/incidents.wf"),
        "{}",
        loc.uri
    );

    // No file reports a component or page it cannot see.
    let diagnostics = project_diagnostics(&project);
    for (file, diags) in project.files.iter().zip(&diagnostics) {
        for d in diags {
            assert!(
                !d.message.contains("unknown component") && !d.message.contains("unknown page"),
                "{}: {}",
                file.path.display(),
                d.message
            );
        }
    }

    // Workspace symbols see every declaration in the project.
    let names: Vec<String> = workspace_symbols(&project, "")
        .into_iter()
        .map(|s| s.name)
        .collect();
    for expected in ["Incidents", "Overview", "Nudge", "IncidentStore"] {
        assert!(names.contains(&expected.to_string()), "{names:?}");
    }
}

#[test]
fn an_open_buffer_overrides_the_disk_and_keeps_its_last_good_parse() {
    let root = workspace_root().join("tests/fixtures/dashboard");
    let path = root.join("src/components/Nudge.wf");
    let uri = Url::from_file_path(&path).unwrap();
    let disk = std::fs::read_to_string(&path).unwrap();
    let parsed = {
        let tokens = webfluent::lexer::Lexer::new(&disk, "").tokenize().unwrap();
        webfluent::parser::Parser::new(tokens, "").parse().unwrap()
    };

    // The buffer renames the component; the parse of the buffer fails half
    // way through an edit.
    let broken = disk.replace("Component Nudge", "Component Nudge (");
    let open = |p: &Path| {
        (p == path).then(|| wf_lsp::project::OpenText {
            text: Arc::from(broken.as_str()),
            last_valid: Some(Arc::new(parsed.clone())),
        })
    };
    let project = Project::load(&uri, &open, &FileCache::default());
    let ix = project.file_index(&uri).unwrap();
    assert!(project.files[ix].parsed.is_err());
    assert!(project.files[ix].stale);
    // The pages that call `Nudge` still see it, from the last good parse.
    let diagnostics = project_diagnostics(&project);
    for (file, diags) in project.files.iter().zip(&diagnostics) {
        if file.path != path {
            assert!(
                diags
                    .iter()
                    .all(|d| !d.message.contains("unknown component `Nudge`")),
                "{:?}",
                diags
            );
        }
    }
    assert_eq!(
        diagnostics[ix].len(),
        1,
        "the broken file reports its parse error: {:?}",
        diagnostics[ix]
    );
}

#[test]
fn the_documentation_site_answers_everywhere_without_panicking() {
    let root = workspace_root().join("site");
    let (project, _) = open(&root.join("src/App.wf"));
    assert!(project.files.len() >= 15);

    let diagnostics = project_diagnostics(&project);
    for (ix, file) in project.files.iter().enumerate() {
        assert!(file.parsed.is_ok(), "{:?}", file.path);
        for d in &diagnostics[ix] {
            assert!(
                !d.message.contains("unknown component"),
                "{}: {}",
                file.path.display(),
                d.message
            );
        }
        let DocumentSymbolResponse::Nested(symbols) = document_symbols(&project, ix, true) else {
            panic!()
        };
        assert!(!symbols.is_empty(), "{:?}", file.path);
        assert!(matches!(
            document_symbols(&project, ix, false),
            DocumentSymbolResponse::Flat(_)
        ));

        // Hover, completion and definition at the start of every line and
        // at a few columns in: none may panic, whatever is there.
        for (line, text) in file.source.lines().enumerate().step_by(3) {
            for character in [0u32, 8, text.encode_utf16().count() as u32] {
                let pos = Position::new(line as u32, character);
                let _ = provide_hover(&project, ix, pos);
                let _ = provide_completions(&project, ix, pos);
                let _ = find_definition(&project, ix, pos);
            }
        }
    }
}

#[test]
fn every_fixture_project_is_clean_of_cross_file_errors() {
    let fixtures = workspace_root().join("tests/fixtures");
    let mut checked = 0;
    for entry in std::fs::read_dir(&fixtures).unwrap().flatten() {
        let app = entry.path().join("src/App.wf");
        if !app.is_file() {
            continue;
        }
        let (project, _) = open(&app);
        for (file, diags) in project.files.iter().zip(project_diagnostics(&project)) {
            for d in diags {
                assert!(
                    d.severity != Some(DiagnosticSeverity::ERROR),
                    "{}: {}",
                    file.path.display(),
                    d.message
                );
            }
        }
        checked += 1;
    }
    assert!(checked >= 3, "fixture projects found: {checked}");
}

#[test]
fn a_file_outside_any_project_is_a_project_of_one() {
    let path = workspace_root().join("tests/fixtures/dashboard/webfluent.app.json");
    // A .wf beside the config but outside src/ is not part of the build.
    let uri = Url::from_file_path(path.with_file_name("scratch.wf")).unwrap();
    let open = |_: &Path| {
        Some(wf_lsp::project::OpenText {
            text: Arc::from("Page P (path: \"/\") { Nudge(label: \"x\") }\n"),
            last_valid: None,
        })
    };
    let project = Project::load(&uri, &open, &FileCache::default());
    assert_eq!(project.files.len(), 1);
    let diags = project_diagnostics(&project).remove(0);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("unknown component `Nudge`")),
        "{diags:?}"
    );
}
