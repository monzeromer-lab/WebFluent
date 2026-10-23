//! The project check runs its phases side by side, on threads of their
//! own. What it finds, and the order it gives it in, must not depend on
//! which thread finishes first: checked again and again, a real project
//! says the same thing every time, in the same order.

use std::path::Path;

use webfluent::build::{BuildOptions, check};
use webfluent::vfs::FsVfs;

/// Every finding of a check of `dir`, as one line each, in order.
fn findings(dir: &Path) -> Vec<String> {
    let report = match check(&BuildOptions::new(dir, &FsVfs)) {
        Ok(report) => report,
        Err(failure) => *failure.report,
    };
    report
        .diagnostics
        .iter()
        .map(|d| {
            format!(
                "{} {}:{}:{} {}",
                d.code, d.file, d.line, d.column, d.message
            )
        })
        .collect()
}

/// A project that finds something in every phase the check runs side by
/// side: the project around it (a picture that is not there, a value from
/// `env` that is not public), the types (a member a store does not have),
/// the vocabulary (a name nothing declares), accessibility (a picture with
/// no words) and what is declared and never read.
fn untidy() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-check-threads-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let root = dir.as_path();
    std::fs::create_dir_all(root.join("src/pages")).unwrap();
    std::fs::create_dir_all(root.join("src/stores")).unwrap();
    std::fs::write(
        root.join("webfluent.app.json"),
        r#"{ "name": "untidy", "build": { "output": "./build" }, "env": { "TOKEN": "x" } }"#,
    )
    .unwrap();
    std::fs::write(root.join("src/stores/Deploys.wf"), "store Deploys {\n    state items: [String] = []\n    state live = 0\n    state never = 1\n}\n").unwrap();
    for (name, path) in [("Home", "/"), ("About", "/about"), ("Pricing", "/pricing")] {
        std::fs::write(
            root.join(format!("src/pages/{name}.wf")),
            format!("page {name}(path: \"{path}\", title: \"{name}\", description: \"{name}.\") {{\n    use Deploys\n    state unread{name} = 2\n    Heading(\"{name}\").h1\n    Text(\"{{Deploys.lve}}\")\n    Text(\"{{env.TOKEN}}\")\n    Image(src: \"/missing-{name}.png\")\n    Row(align: center) {{ Text(\"x\") }}\n}}\n"),
        )
        .unwrap();
    }
    dir
}

#[test]
fn an_untidy_project_checked_again_and_again_says_the_same_in_the_same_order() {
    let dir = untidy();
    let first = findings(&dir);
    let codes: std::collections::BTreeSet<String> = first
        .iter()
        .map(|f| f.split(' ').next().unwrap_or("").to_string())
        .collect();
    assert!(
        codes.len() >= 5,
        "it should find something in each phase: {first:#?}"
    );
    // What the check said when it ran its phases one after another: each
    // finding's code and place.
    let said: Vec<String> = first
        .iter()
        .map(|f| f.splitn(3, ' ').take(2).collect::<Vec<_>>().join(" "))
        .collect();
    assert_eq!(
        said,
        [
            "U01 src/pages/About.wf:3:5",
            "T06 src/pages/About.wf:5:12",
            "E108 src/pages/About.wf:6:10",
            "A01 src/pages/About.wf:7:5",
            "D01 src/pages/About.wf:7:11",
            "T13 src/pages/About.wf:8:16",
            "U01 src/pages/Home.wf:3:5",
            "T06 src/pages/Home.wf:5:12",
            "E108 src/pages/Home.wf:6:10",
            "A01 src/pages/Home.wf:7:5",
            "D01 src/pages/Home.wf:7:11",
            "U01 src/pages/Pricing.wf:3:5",
            "T06 src/pages/Pricing.wf:5:12",
            "E108 src/pages/Pricing.wf:6:10",
            "A01 src/pages/Pricing.wf:7:5",
            "D01 src/pages/Pricing.wf:7:11",
            "U04 src/stores/Deploys.wf:2:5",
            "U04 src/stores/Deploys.wf:3:5",
            "U04 src/stores/Deploys.wf:4:5",
        ],
        "the check found something else than it did one phase at a time"
    );
    for _ in 0..40 {
        assert_eq!(
            findings(&dir),
            first,
            "a check said something else, or in another order"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_project_checked_again_and_again_says_the_same_in_the_same_order() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // The guide's own site, the largest project here, and fixtures that
    // find something: a check with nothing to say would prove little.
    for project in [
        "site",
        "tests/fixtures/dashboard",
        "tests/fixtures/docs",
        "tests/fixtures/deck",
        "examples/tutorials/medium-field-notes",
    ] {
        let dir = root.join(project);
        if !dir.join("webfluent.app.json").exists() {
            continue;
        }
        let first = findings(&dir);
        println!("{project}: {} findings", first.len());
        for _ in 0..12 {
            assert_eq!(
                findings(&dir),
                first,
                "{project}: a check said something else, or in another order"
            );
        }
    }
}
