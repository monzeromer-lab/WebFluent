//! The analysis over a project read through a `Vfs`: an editor's unsaved
//! buffers laid over the disk, seen by every file of the project, and
//! reported in the words `wf build` prints for the same text.

use std::path::PathBuf;

use lsp_types::*;
use webfluent::build::{BuildOptions, build};
use webfluent::vfs::{FsVfs, OverlayVfs};
use wf_lsp::completion::provide_completions;
use wf_lsp::diagnostics::project_diagnostics;
use wf_lsp::project::{FileCache, Project};

/// A project on disk: a store `Deploys` with members `live` and `queued`,
/// and a page reading `Deploys.live`.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-lsp-overlay-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::create_dir_all(dir.join("src/stores")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "deploys", "build": { "output": "build" } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/stores/deploys.wf"),
        "store Deploys {\n    state live = 3\n    state queued = []\n}\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/pages/Home.wf"), home("live")).unwrap();
    dir
}

fn home(member: &str) -> String {
    format!(
        "page Home(path: \"/\", title: \"Home\", description: \"Deploys.\") {{\n    use Deploys\n    Heading(\"Deploys\").h1\n    Text(\"{{Deploys.queued.length}} queued\")\n    Text(\"{{Deploys.{member}}} live\")\n}}\n"
    )
}

fn errors(project: &Project, ix: usize) -> Vec<Diagnostic> {
    project_diagnostics(project)
        .remove(ix)
        .into_iter()
        .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
        .collect()
}

#[test]
fn a_store_member_typo_in_a_buffer_is_t06_as_the_build_says_it() {
    let dir = project("t06");
    let page = dir.join("src/pages/Home.wf");
    let uri = Url::from_file_path(&page).unwrap();

    // The disk is clean.
    let clean = Project::load_with(&uri, &FsVfs, &FileCache::default());
    let ix = clean.file_index(&uri).unwrap();
    assert!(errors(&clean, ix).is_empty(), "{:?}", errors(&clean, ix));

    let mut vfs = OverlayVfs::over_disk();
    vfs.set(&page, home("lve"));
    let project = Project::load_with(&uri, &vfs, &FileCache::default());
    let ix = project.file_index(&uri).unwrap();
    let found = errors(&project, ix);
    assert_eq!(found.len(), 1, "{found:?}");
    let t06 = &found[0];
    assert!(t06.message.starts_with("[T06] "), "{}", t06.message);

    // `wf build`, reading the same buffer, says the same at the same place.
    let failure = build(&BuildOptions::new(&dir, &vfs)).unwrap_err();
    let printed = &failure.report.errors[0];
    assert_eq!(
        printed.text,
        "Error: [T06] `Deploys` has no member `lve` at src/pages/Home.wf:5:12\n  Its members are `live`, `queued`"
    );
    assert_eq!(printed.code.as_deref(), Some("T06"));
    assert_eq!(
        t06.message,
        format!(
            "[T06] {}\n{}",
            printed.message,
            printed.hint.as_deref().unwrap()
        )
    );
    assert_eq!(
        t06.range.start,
        Position::new(printed.line as u32 - 1, printed.column as u32 - 1)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_buffer_is_read_afresh_however_little_it_changed() {
    let dir = project("fresh");
    let page = dir.join("src/pages/Home.wf");
    let uri = Url::from_file_path(&page).unwrap();
    let cache = FileCache::default();
    let mut vfs = OverlayVfs::over_disk();
    for typo in ["lve", "liv", "lie"] {
        vfs.set(&page, home(typo));
        let project = Project::load_with(&uri, &vfs, &cache);
        let ix = project.file_index(&uri).unwrap();
        let found = errors(&project, ix);
        assert!(
            found
                .iter()
                .any(|d| d.message.contains(&format!("no member `{typo}`"))),
            "{typo}: {found:?}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_only_the_editor_has_is_part_of_the_project() {
    let dir = project("new");
    let page = dir.join("src/pages/Home.wf");
    let uri = Url::from_file_path(&page).unwrap();
    let mut vfs = OverlayVfs::over_disk();
    // A store not saved yet, in a directory not made yet, and a page that
    // reads it.
    vfs.set(
        dir.join("src/stores/builds/builds.wf"),
        "store Builds {\n    state count = 0\n    action reset() { count = 0 }\n}\n",
    );
    let text = home("live").replace(
        "use Deploys",
        "use Deploys\n    use Builds\n    Text(\"{Builds.count} built\")\n    Button(\"Reset\") { on click { Builds.reset() } }",
    );
    vfs.set(&page, text.clone());
    let project = Project::load_with(&uri, &vfs, &FileCache::default());
    let ix = project.file_index(&uri).unwrap();
    assert!(
        project
            .files
            .iter()
            .any(|f| f.path.ends_with("src/stores/builds/builds.wf")),
        "{:?}",
        project.files.iter().map(|f| &f.path).collect::<Vec<_>>()
    );
    assert!(
        errors(&project, ix).is_empty(),
        "{:?}",
        errors(&project, ix)
    );

    // Its members complete like a saved store's.
    let at = text.find("Builds.reset").unwrap() + "Builds.".len();
    let file = &project.files[ix];
    let position = file.index.offset_to_position(&file.source, at);
    let labels: Vec<String> = provide_completions(&project, ix, position)
        .into_iter()
        .map(|item| item.label)
        .collect();
    assert_eq!(labels, ["count", "reset"]);

    // Deleted, it is gone, and so are its members.
    vfs.delete(dir.join("src/stores/builds"));
    let project = Project::load_with(&uri, &vfs, &FileCache::default());
    let ix = project.file_index(&uri).unwrap();
    assert_eq!(project.files.len(), 2);
    let file = &project.files[ix];
    let position = file.index.offset_to_position(&file.source, at);
    assert!(provide_completions(&project, ix, position).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
