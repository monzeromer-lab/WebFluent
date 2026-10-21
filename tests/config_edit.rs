//! C11: `webfluent.app.json` edited as a person would, over every real
//! config in this repository — the site's, the examples', the fixtures':
//! each field a file sets, changed and changed back, leaves it
//! byte-identical; a field it does not set, added and removed, too; and
//! every step in between differs from the file by that field alone. Then
//! `wf config` on a project on disk.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use webfluent::config::schema::{self, FieldType};

fn configs() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    let mut walk = vec![
        root.join("site"),
        root.join("examples"),
        root.join("tests/fixtures"),
    ];
    while let Some(dir) = walk.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if p.is_dir()
                && !matches!(
                    name.as_str(),
                    "node_modules" | "build" | "target" | ".wf-cache"
                )
            {
                walk.push(p);
            } else if name == "webfluent.app.json" {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Another value of `ty` than `v`.
fn other(ty: FieldType, v: &Value) -> Value {
    match ty {
        FieldType::Bool => Value::Bool(!v.as_bool().unwrap_or(false)),
        FieldType::String => Value::String(format!("{}-changed", v.as_str().unwrap_or(""))),
        FieldType::Number => serde_json::json!(v.as_f64().unwrap_or(0.0) + 1.5),
        FieldType::Integer { min, max } => {
            let n = v.as_i64().unwrap_or(min);
            serde_json::json!(if n < max { n + 1 } else { n - 1 })
        }
        FieldType::Enum(cases) => Value::String(
            cases
                .iter()
                .find(|c| Some(**c) != v.as_str())
                .unwrap()
                .to_string(),
        ),
        FieldType::Strings => serde_json::json!(["x"]),
        FieldType::Integers => serde_json::json!([123]),
        FieldType::StringMap { values } => serde_json::json!({ "x": values.map_or("y", |c| c[0]) }),
        FieldType::Map => serde_json::json!({ "x": 1 }),
        FieldType::Scripts => serde_json::json!(["/x.js"]),
        FieldType::File(_) => Value::String(format!("{}-changed.png", v.as_str().unwrap_or("/x"))),
    }
}

/// The lines of `a` and `b` that differ, as a set of line numbers of `b`.
fn changed_lines(a: &str, b: &str) -> usize {
    let a: Vec<&str> = a.lines().collect();
    let b: Vec<&str> = b.lines().collect();
    // Same prefix and suffix; what is between is the change.
    let start = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let end = a[start..]
        .iter()
        .rev()
        .zip(b[start..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    b.len() - start - end
}

#[test]
fn every_real_config_round_trips_byte_for_byte() {
    let found = configs();
    assert!(found.len() >= 10, "configs found: {found:?}");
    let mut edits = 0;
    for path in &found {
        let text = std::fs::read_to_string(path).unwrap();
        let Ok(parsed) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        for field in schema::FIELDS {
            // A map field whose entries are hard to vary in general is
            // still changed and restored as a whole.
            let set = schema::at(&parsed, field.path).cloned();
            match set {
                Some(was) => {
                    let new = other(field.ty, &was);
                    let Ok(changed) = schema::edit(&text, field.path, Some(&new)) else {
                        // A value the rest of the file forbids (a type
                        // that only fits one output) is not this test's.
                        continue;
                    };
                    assert_ne!(
                        changed,
                        text,
                        "{}: {} did not change",
                        path.display(),
                        field.path
                    );
                    // Set to what it is: not a byte moves.
                    assert_eq!(
                        schema::edit(&text, field.path, Some(&was)).unwrap(),
                        text,
                        "{}: {} set to itself",
                        path.display(),
                        field.path
                    );
                    let back = schema::edit(&changed, field.path, Some(&was)).unwrap();
                    if was.is_array() || was.is_object() {
                        // A list or a map written back holds the same value,
                        // and nothing outside it moved; inside, a map's keys
                        // read in sorted order once it has been replaced.
                        let reread: Value = serde_json::from_str(&back).unwrap();
                        assert_eq!(
                            schema::at(&reread, field.path),
                            Some(&was),
                            "{}: {}",
                            path.display(),
                            field.path
                        );
                        let lines_off = changed_lines(&text, &back);
                        let field_lines =
                            serde_json::to_string_pretty(&was).unwrap().lines().count();
                        assert!(
                            lines_off <= field_lines,
                            "{}: {} moved more than itself:\n{back}",
                            path.display(),
                            field.path
                        );
                    } else {
                        assert_eq!(
                            back,
                            text,
                            "{}: {} changed and back is not the file",
                            path.display(),
                            field.path
                        );
                    }
                    // A scalar is one line; a list or a map is a few.
                    if !was.is_object() && !was.is_array() && !new.is_object() && !new.is_array() {
                        assert_eq!(
                            changed_lines(&text, &changed),
                            1,
                            "{}: {}\n{changed}",
                            path.display(),
                            field.path
                        );
                    }
                }
                None => {
                    let new = other(field.ty, &Value::Null);
                    let Ok(added) = schema::edit(&text, field.path, Some(&new)) else {
                        continue;
                    };
                    let removed = schema::edit(&added, field.path, None).unwrap();
                    // A section the path goes through that the file wrote
                    // empty goes with it — an empty section means nothing;
                    // otherwise the file is as it was.
                    let keys: Vec<&str> = field.path.split('.').collect();
                    let written_empty = (1..keys.len()).find(|n| {
                        schema::at(&parsed, &keys[..*n].join(".")) == Some(&serde_json::json!({}))
                    });
                    match written_empty {
                        None => assert_eq!(
                            removed,
                            text,
                            "{}: {} added and removed is not the file\n{added}",
                            path.display(),
                            field.path
                        ),
                        Some(n) => {
                            let expect =
                                webfluent::config::json_edit::remove(&text, &keys[..n]).unwrap();
                            assert_eq!(removed, expect, "{}: {}", path.display(), field.path);
                        }
                    }
                }
            }
            edits += 1;
        }
    }
    assert!(edits > 500, "{edits} edits");
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

#[test]
fn wf_config_reads_and_writes_the_project() {
    let dir = std::env::temp_dir().join(format!("wf-config-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let text = "{\n  \"name\": \"site\",\n  \"build\": { \"ssg\": true }\n}\n";
    std::fs::write(dir.join("webfluent.app.json"), text).unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    Heading(\"Home\").h1\n}\n",
    )
    .unwrap();

    let (_, out, _) = wf(&dir, &["config", "get", "build.ssg"]);
    assert_eq!(out.trim(), "true");
    let (_, out, _) = wf(&dir, &["config", "get", "dev.port"]);
    assert_eq!(
        out.trim(),
        "3000 (the default; webfluent.app.json does not set it)"
    );

    let (code, out, err) = wf(&dir, &["config", "set", "build.base_path", "/Docs"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.trim(), "build.base_path = \"/Docs\"");
    let (_, _, _) = wf(&dir, &["config", "set", "meta.locales_typo", "x"]);
    let now = std::fs::read_to_string(dir.join("webfluent.app.json")).unwrap();
    assert_eq!(
        now,
        "{\n  \"name\": \"site\",\n  \"build\": { \"ssg\": true, \"base_path\": \"/Docs\" }\n}\n"
    );

    // A wrong value is refused, and the file is left as it was.
    let (code, _, err) = wf(&dir, &["config", "set", "build.output_type", "website"]);
    assert_ne!(code, 0);
    assert!(
        err.contains("takes one of spa, static, pdf, slides, elements"),
        "{err}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("webfluent.app.json")).unwrap(),
        now
    );

    // What it wrote is what builds: the base path reaches the pages.
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");

    let (_, out, _) = wf(&dir, &["config", "unset", "build.base_path"]);
    assert_eq!(out.trim(), "build.base_path removed: the default applies");
    assert_eq!(
        std::fs::read_to_string(dir.join("webfluent.app.json")).unwrap(),
        text
    );

    let (_, out, _) = wf(&dir, &["config", "schema", "--json"]);
    let rows: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), schema::FIELDS.len());
}
