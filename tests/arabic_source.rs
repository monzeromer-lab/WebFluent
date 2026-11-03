//! Arabic in a project's source: names written in Arabic, marks and all,
//! and the direction controls that would make code read one way and run
//! another (Trojan Source), which the check points out.

use std::path::Path;
use webfluent::build::{BuildOptions, check};
use webfluent::vfs::FsVfs;

fn project(name: &str, page: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-arabic-source-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "arabic", "build": { "output": "./build" } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/pages/Home.wf"), page).unwrap();
    dir
}

/// Each finding as `code line:col`.
fn findings(dir: &Path) -> Vec<String> {
    let report = match check(&BuildOptions::new(dir, &FsVfs)) {
        Ok(report) => report,
        Err(failure) => *failure.report,
    };
    report
        .diagnostics
        .iter()
        .map(|d| format!("{} {}:{}", d.code, d.line, d.column))
        .collect()
}

#[test]
fn a_name_in_arabic_keeps_the_marks_of_its_letters() {
    let dir = project(
        "names",
        "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    state عددٌ = 3\n    Heading(\"Home\").h1\n    Text(\"{عددٌ}\")\n}\n",
    );
    let found = findings(&dir);
    assert!(
        found
            .iter()
            .all(|f| !f.starts_with('E') && !f.starts_with("T")),
        "{found:?}"
    );
}

#[test]
fn a_name_is_one_name_however_its_hamza_was_typed() {
    // Declared with أ precomposed, read with ا and a combining hamza above.
    let dir = project(
        "nfc",
        "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    state أسماء = [\"a\"]\n    Heading(\"Home\").h1\n    Text(\"{\u{0627}\u{0654}سماء.length}\")\n}\n",
    );
    let found = findings(&dir);
    assert!(
        found
            .iter()
            .all(|f| !f.starts_with('E') && !f.starts_with("T")),
        "{found:?}"
    );
}

#[test]
fn a_hidden_direction_control_is_pointed_out_where_it_is() {
    let dir = project(
        "trojan",
        "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    state role = \"user\"\n    Heading(\"Home\").h1\n    Text(\"admin\u{202E} \u{2066}// check\u{2069}\")\n}\n",
    );
    let found: Vec<String> = findings(&dir)
        .into_iter()
        .filter(|f| f.starts_with("E119"))
        .collect();
    assert_eq!(found, ["E119 4:16", "E119 4:18", "E119 4:27"]);
}

#[test]
fn a_store_its_state_and_its_action_named_in_arabic_build() {
    let dir = project(
        "build",
        "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    use المهام\n    Heading(\"Home\").h1\n    Text(\"{المهام.عدد}\")\n    Button(\"Add\") { on click { المهام.أضف(\"c\") } }\n}\n",
    );
    std::fs::create_dir_all(dir.join("src/stores")).unwrap();
    std::fs::write(
        dir.join("src/stores/tasks.wf"),
        "store المهام {\n    state قائمة = [\"a\", \"b\"]\n    derived عدد = قائمة.length\n    action أضف(اسم: String) {\n        قائمة = قائمة.concat([اسم])\n    }\n}\n",
    )
    .unwrap();
    let report = webfluent::build::build(&BuildOptions::new(&dir, &FsVfs))
        .unwrap_or_else(|f| panic!("{:?}", f.report.diagnostics));
    assert!(report.diagnostics.iter().all(|d| !d.code.starts_with('E')));
    let app = std::fs::read_to_string(dir.join("build/pages/Home.js")).unwrap();
    assert!(app.contains("أضف"), "{app}");
}
