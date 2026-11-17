//! WebFluent 6 → 7: `wf migrate` applies every fix 7.0's findings carry —
//! the request forms 6.x had, the types a declaration's readers see — until
//! none is left, and lists what needs a person. `tests/fixtures/migrate7`
//! holds a 6.x project before and what the step writes after; a second run
//! finds nothing; the result builds. And there is no loose mode: `lints`
//! and `// wf-allow` cannot lower or accept the codes 7.0 brought.

use std::path::{Path, PathBuf};

use webfluent::build::{BuildOptions, check};
use webfluent::migrate::seven;
use webfluent::vfs::FsVfs;

fn fixture(side: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/migrate7")
        .join(side)
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

fn scratch(why: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-migrate7-{why}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    for (path, text) in files {
        std::fs::write(dir.join(path), text).unwrap();
    }
    dir
}

/// Every line `wf check` prints.
fn findings(dir: &Path) -> String {
    match check(&BuildOptions::new(dir, &FsVfs)) {
        Ok(report) => format!("{:?}", report.lines),
        Err(failure) => format!("{:?}\n{failure}", failure.report.lines),
    }
}

#[test]
fn a_six_project_is_carried_to_seven_and_builds() {
    let dir = std::env::temp_dir().join(format!("wf-migrate7-fixture-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_tree(&fixture("before"), &dir);

    // What it would do, writing nothing.
    let dry = seven::migrate(&dir, true).unwrap();
    assert!(dry.total() > 0);
    assert_eq!(
        std::fs::read_to_string(dir.join("src/App.wf")).unwrap(),
        std::fs::read_to_string(fixture("before").join("src/App.wf")).unwrap()
    );

    let done = seven::migrate(&dir, false).unwrap();
    for code in ["N08", "N12", "N03", "T01", "T25", "T26", "T29", "T34"] {
        assert!(
            done.by_code.contains_key(code),
            "{code} was not fixed: {:?}",
            done.by_code
        );
    }
    assert!(done.left.is_empty(), "{:?}", done.left);
    assert_eq!(
        std::fs::read_to_string(dir.join("src/App.wf")).unwrap(),
        std::fs::read_to_string(fixture("after").join("src/App.wf")).unwrap(),
        "the step writes what tests/fixtures/migrate7/after holds"
    );

    // Done is done: a second run finds nothing to rewrite.
    let again = seven::migrate(&dir, true).unwrap();
    assert_eq!(again.total(), 0, "{:?}", again.by_code);

    let found = findings(&dir);
    assert!(!found.contains("error["), "{found}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_needs_a_person_is_left_and_listed() {
    let dir = scratch(
        "left",
        &[
            ("webfluent.app.json", r#"{ "name": "left" }"#),
            (
                "src/App.wf",
                "api Legacy(base: \"/x\") {\n    on request(r) { log(r) }\n    get ping at \"ping\" -> String\n}\n\napp { Router }\n\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    resource pong: String = Legacy.ping()\n    Heading(\"H\").h1\n    match pong { .loaded(p) { Text(p) } else { Spinner } }\n}\n",
            ),
        ],
    );
    let done = seven::migrate(&dir, false).unwrap();
    assert_eq!(
        done.by_code.get("N08"),
        Some(&1),
        "the result is said; {:?}",
        done.by_code
    );
    let left: Vec<String> = done
        .left
        .iter()
        .map(|d| format!("{} {}", d.code, d.message))
        .collect();
    assert!(
        left.iter()
            .any(|l| l.starts_with("N08") && l.contains("`on request` hook")),
        "{left:?}"
    );
    let text = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    assert!(text.contains("get ping at \"ping\" -> String!"), "{text}");
    assert!(
        text.contains("on request(r)"),
        "a hook is a person's to rewrite: {text}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn lints_and_allows_cannot_lower_what_seven_brought() {
    let dir = scratch(
        "lints",
        &[
            (
                "webfluent.app.json",
                r#"{ "name": "lints", "lints": { "T29": "warn", "T20": "off", "N08": "off", "T25": "warn" } }"#,
            ),
            (
                "src/App.wf",
                "app { Router }\n\npage Home(path: \"/\", title: \"H\", description: \"d\") {\n    state n = 3\n    state tags = [\"a\", \"b\"]\n    Heading(\"H\").h1\n    // wf-allow(T29)\n    if n { Text(\"x\") }\n    // wf-allow(T20)\n    Text(\"{tags}\")\n}\n",
            ),
        ],
    );
    let found = findings(&dir);
    for code in ["T29", "T20", "N08", "T25"] {
        assert!(
            found.contains(&format!(
                "`lints` lowers `{code}`, an error that ships a broken page"
            )),
            "{code}: {found}"
        );
    }
    assert!(found.contains("error[T29]"), "{found}");
    assert!(found.contains("error[T20]"), "{found}");
    assert!(
        found.contains("`wf-allow(T29)` cannot silence `T29`"),
        "{found}"
    );
    assert!(
        found.contains("`wf-allow(T20)` cannot silence `T20`"),
        "{found}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
