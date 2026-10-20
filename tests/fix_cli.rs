//! `wf fix`, on a project on disk: every finding with a decided fix is
//! fixed, the files say so, `wf check` no longer reports them, and what a
//! person must decide is left and said. With `--code`, one family; with
//! `--dry-run`, nothing is written.

use std::path::{Path, PathBuf};
use std::process::Command;

fn project(name: &str, config: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-fix-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    dir
}

fn wf(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("wf runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

const CONFIG: &str = r#"{ "name": "f", "i18n": { "default_locale": "en", "locales": ["en", "fr"], "dir": "src/translations" } }"#;

/// A project with one finding of each of several families, each with a
/// decided fix — and one the fix cannot decide.
fn broken(name: &str) -> PathBuf {
    project(
        name,
        CONFIG,
        &[
            (
                "src/App.wf",
                "app {\n    Navbar { Text(\"Site\") }\n}\n\nenum Tone { calm, loud }\n\npage Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    state enabled = false\n    state w = 40\n    state tone: Tone = .clam\n    persist items: [String] = [] {\n        version: 3\n        migrate 2 -> 3 { old }\n    }\n    Heading(\"Home\").h1\n    Image(src: \"/hro.png\", alt: \"\")\n    Checkbox(checked: enabled, label: \"On\")\n    Container {\n        style { width: {w} }\n        Text(\"{tone} {items.length}\")\n    }\n    Link(\"About\", to: \"/abuot\")\n    Button(\"FR\") { on click { setLocale(\"fe\") } }\n}\n\npage About(path: \"/about\", title: \"About\", description: \"About.\") {\n    Heading(\"About\").h1\n    Text(t(\"missing.key\"))\n}\n",
            ),
            ("src/translations/en.json", r#"{ "x": "x" }"#),
            ("src/translations/fr.json", r#"{ "x": "x" }"#),
            ("public/hero.png", "png"),
        ],
    )
}

#[test]
fn every_finding_with_a_fix_is_fixed_and_the_rest_is_said() {
    let dir = broken("all");
    let (_, before, _) = wf(&dir, &["check", "--format", "json"]);
    for code in [
        "R03", "T02", "P01", "D01", "F04", "V07", "R01", "I04", "I01",
    ] {
        assert!(
            before.contains(&format!("\"{code}\"")),
            "{code} before: {before}"
        );
    }
    let (code, out, err) = wf(&dir, &["fix"]);
    assert_eq!(code, 0, "{out}\n{err}");
    assert!(out.starts_with("Fixed "), "{out}");
    let app = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    for (what, want) in [
        ("R03", "    Navbar { Text(\"Site\") }\n    Router\n}"),
        ("T02", "state tone: Tone = .calm"),
        (
            "P01",
            "        migrate 2 -> 3 { old }\n        migrate 1 -> 2 { old }\n    }",
        ),
        ("D01", "Image(src: \"/hero.png\""),
        ("F04", "Checkbox(bind: enabled"),
        ("V07", "width: {w}px"),
        ("R01", "to: \"/about\""),
        ("I04", "setLocale(\"fr\")"),
    ] {
        assert!(app.contains(want), "{what}: {app}");
    }
    let (_, after, _) = wf(&dir, &["check", "--format", "json"]);
    for code in ["R03", "T02", "P01", "D01", "F04", "V07", "R01", "I04"] {
        assert!(
            !after.contains(&format!("\"{code}\"")),
            "{code} after: {after}\n{app}"
        );
    }
    // A message no translation has is a sentence only a person writes.
    assert!(after.contains("\"I01\""), "{after}");
    assert!(out.contains("1 left, which a person decides"), "{out}");
    // What is left is written by a person; then the program builds.
    for locale in ["en", "fr"] {
        std::fs::write(
            dir.join(format!("src/translations/{locale}.json")),
            r#"{ "x": "x", "missing.key": "Found" }"#,
        )
        .unwrap();
    }
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}\n{app}");
}

#[test]
fn one_family_at_a_time() {
    let dir = broken("one");
    let (code, out, _) = wf(&dir, &["fix", "--code", "V07"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("Fixed 1 finding in 1 file: 1 V07"), "{out}");
    let app = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    assert!(app.contains("width: {w}px"));
    assert!(app.contains(".clam"), "another family is left alone");
}

#[test]
fn a_dry_run_writes_nothing() {
    let dir = broken("dry");
    let was = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    let (code, out, _) = wf(&dir, &["fix", "--dry-run"]);
    assert_eq!(code, 0);
    assert!(out.contains("would change src/App.wf"), "{out}");
    assert!(out.contains("Would fix "), "{out}");
    assert_eq!(
        std::fs::read_to_string(dir.join("src/App.wf")).unwrap(),
        was
    );
}

#[test]
fn a_new_shape_raises_its_version_and_brings_the_old_one_forward() {
    let dir = project(
        "d04",
        r#"{ "name": "f" }"#,
        &[(
            "src/App.wf",
            "app { Router }\npage Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    persist items: [String] = []\n    Heading(\"Home\").h1\n    Text(\"{items.length}\")\n}\n",
        )],
    );
    // A build remembers the shape; the shape then changes.
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    let app = dir.join("src/App.wf");
    let changed = std::fs::read_to_string(&app).unwrap().replace(
        "persist items: [String] = []",
        "persist items: [Number] = []",
    );
    std::fs::write(&app, &changed).unwrap();
    let (_, before, _) = wf(&dir, &["check", "--format", "json"]);
    assert!(before.contains("\"D04\""), "{before}");
    let (code, out, _) = wf(&dir, &["fix", "--code", "D04"]);
    assert_eq!(code, 0, "{out}");
    let fixed = std::fs::read_to_string(&app).unwrap();
    assert!(
        fixed.contains("persist items: [Number] = [] {\n        version: 2\n        migrate 1 -> 2 { old }\n    }"),
        "{fixed}"
    );
    let (_, after, _) = wf(&dir, &["check", "--format", "json"]);
    assert!(
        !after.contains("\"D04\"") && !after.contains("\"P01\""),
        "{after}\n{fixed}"
    );
}

#[test]
fn nothing_to_fix_says_so() {
    let dir = project(
        "clean",
        r#"{ "name": "f" }"#,
        &[(
            "src/App.wf",
            "app { Router }\npage Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    Heading(\"Home\").h1\n}\n",
        )],
    );
    let (code, out, _) = wf(&dir, &["fix"]);
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "Nothing to fix.");
}
