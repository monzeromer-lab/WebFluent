//! A service read from an OpenAPI spec, and `wf api`, which writes its
//! contract as files. Three golden folders under `tests/fixtures/openapi/`
//! — a small spec (the team directory), Petstore in YAML, and a real one
//! (oxn's queue API) — are each current with their spec and each build;
//! `api … from` and the synced files mean the same program; a stale folder
//! fails `check` and `N07`; a file `sync` did not write is never touched.
//! The team directory's tests, against fakes of the generated contract,
//! are `wf test tests/fixtures/openapi/people`.

use std::path::{Path, PathBuf};

use webfluent::api;
use webfluent::build::{BuildOptions, build, check};
use webfluent::vfs::FsVfs;

const FIXTURES: [(&str, &str); 3] = [("people", "Backend"), ("petstore", "Shop"), ("oxn", "Oxn")];

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/openapi")
        .join(name)
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_name() == "build" || entry.file_name() == ".wf-cache" {
            continue;
        }
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

/// A copy of a fixture to change, in a folder of its own.
fn scratch(name: &str, why: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-openapi-{name}-{why}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_tree(&fixture(name), &dir);
    dir
}

fn source(dir: &Path, name: &str) -> webfluent::config::project::ApiSource {
    api::services(dir)
        .unwrap()
        .remove(name)
        .expect("the service is kept")
}

/// Every line `wf check` prints for a project.
fn findings(dir: &Path) -> String {
    match check(&BuildOptions::new(dir, &FsVfs)) {
        Ok(report) => format!("{:?}", report.lines),
        Err(failure) => format!("{:?}\n{failure}", failure.report.lines),
    }
}

/// What `wf types --json` says of a project.
fn types_json(dir: &Path) -> serde_json::Value {
    let (program, files) = webfluent::build::read_project(dir).unwrap();
    let mut json = webfluent::registry::json::declarations_json(&program, &|i| {
        files.get(i).cloned().unwrap_or_default()
    });
    // Where a declaration was written, and so the order files are read in,
    // is the one thing `from` and files differ in.
    strip(&mut json, &["file", "line", "span", "at"]);
    // And every group of every service, as the printer writes it.
    let mut groups: Vec<String> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            webfluent::syntax::parser::Declaration::Api(a) if a.name.contains('.') => {
                Some(webfluent::syntax::print::declaration(d, &program))
            }
            _ => None,
        })
        .collect();
    groups.sort();
    json["groups"] = serde_json::json!(groups);
    json
}

fn strip(v: &mut serde_json::Value, keys: &[&str]) {
    match v {
        serde_json::Value::Object(o) => {
            o.retain(|k, _| !keys.contains(&k.as_str()));
            for x in o.values_mut() {
                strip(x, keys);
            }
        }
        serde_json::Value::Array(a) => {
            for x in a.iter_mut() {
                strip(x, keys);
            }
            a.sort_by_key(|x| x.get("name").and_then(|n| n.as_str()).map(str::to_string));
        }
        _ => {}
    }
}

#[test]
fn each_golden_folder_is_current_with_its_spec_and_builds() {
    for (name, service) in FIXTURES {
        let dir = fixture(name);
        let (_, changes) = api::changes(&FsVfs, &dir, service, &source(&dir, service)).unwrap();
        assert!(
            changes.is_empty(),
            "`{name}` is behind its spec — run `wf api sync` in tests/fixtures/openapi/{name}: {changes:?}"
        );
        let out =
            std::env::temp_dir().join(format!("wf-openapi-build-{name}-{}", std::process::id()));
        let report = build(&BuildOptions {
            output_dir: Some(out.clone()),
            ..BuildOptions::new(&dir, &FsVfs)
        })
        .unwrap_or_else(|failure| {
            panic!(
                "`{name}` does not build: {failure}\n{:?}",
                failure.report.lines
            )
        });
        assert!(
            out.join("index.html").exists(),
            "{name}: {:?}",
            report.lines
        );
        let _ = std::fs::remove_dir_all(&out);
    }
}

#[test]
fn every_generated_file_opens_with_the_marker_and_reads_back_as_written() {
    for (name, service) in FIXTURES {
        let dir = fixture(name);
        let generated = api::generate(&FsVfs, &dir, service, &source(&dir, service)).unwrap();
        assert!(!generated.files.is_empty(), "{name}");
        for f in &generated.files {
            assert!(api::is_generated(&f.text), "{}", f.path);
            assert!(f.text.contains("DO NOT EDIT"), "{}", f.path);
            let again = webfluent::syntax::fmt::format_source(&f.text, &f.path).unwrap();
            assert_eq!(again, f.text, "{} is not as `wf fmt` writes it", f.path);
        }
    }
}

#[test]
fn from_and_sync_build_the_same_program() {
    for (name, service) in FIXTURES {
        let synced = fixture(name);
        let src = source(&synced, service);
        let from = scratch(name, "from");
        // `api Shop from "petstore.yaml"` in place of the files: the service's
        // own file says the same, with `from`, and the generated folder goes.
        let service_wf = from.join(&src.out).join("service.wf");
        let text = std::fs::read_to_string(&service_wf).unwrap();
        let text = text.replacen(
            &format!("api {service}("),
            &format!("api {service} from \"{}\" (", src.spec),
            1,
        );
        assert!(text.contains(" from \""), "{text}");
        std::fs::write(&service_wf, text).unwrap();
        std::fs::remove_dir_all(from.join(&src.out).join("generated")).unwrap();
        std::fs::write(from.join("webfluent.app.json"), r#"{ "name": "from", "build": { "output": "build" }, "env": { "PUBLIC_API": "/api" } }"#).unwrap();
        assert_eq!(
            types_json(&from),
            types_json(&synced),
            "`api {service} from` and `wf api sync` disagree for `{name}`"
        );
        let _ = std::fs::remove_dir_all(&from);
    }
}

#[test]
fn a_spec_that_moved_on_fails_check_and_warns_until_synced() {
    let dir = scratch("petstore", "stale");
    let spec = dir.join("petstore.yaml");
    let text = std::fs::read_to_string(&spec).unwrap();
    std::fs::write(
        &spec,
        text.replace(
            "        tag:\n",
            "        born:\n          type: string\n          format: date\n        tag:\n",
        ),
    )
    .unwrap();
    let src = source(&dir, "Shop");
    let (_, changes) = api::changes(&FsVfs, &dir, "Shop", &src).unwrap();
    assert_eq!(
        changes,
        [api::Change::Changed(
            "src/api/shop/generated/types.wf".into()
        )]
    );

    // `wf check` says so, as `N07`.
    let found = findings(&dir);
    assert_eq!(found.matches("warning[N07]").count(), 1, "{found}");
    assert!(
        found.contains("`wf api sync Shop` would change `src/api/shop/generated/types.wf`"),
        "{found}"
    );

    api::sync(&dir, "Shop", &src).unwrap();
    let (_, changes) = api::changes(&FsVfs, &dir, "Shop", &src).unwrap();
    assert!(changes.is_empty(), "{changes:?}");
    let types = std::fs::read_to_string(dir.join("src/api/shop/generated/types.wf")).unwrap();
    // `Pet` is sent too (`createPets`), so its date is the `String` a program holds.
    assert!(types.contains("born: String?"), "{types}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_tag_that_is_gone_takes_its_file_with_it() {
    let dir = scratch("petstore", "gone");
    let spec = dir.join("petstore.yaml");
    let text = std::fs::read_to_string(&spec).unwrap();
    std::fs::write(
        &spec,
        text.replace("tags: [pets]", "tags: [shop]")
            .replace("tags:\n        - pets", "tags:\n        - shop"),
    )
    .unwrap();
    let (_, changes) = api::sync(&dir, "Shop", &source(&dir, "Shop")).unwrap();
    assert!(
        changes.contains(&api::Change::Added("src/api/shop/generated/shop.wf".into())),
        "{changes:?}"
    );
    assert!(
        changes.contains(&api::Change::Removed(
            "src/api/shop/generated/pets.wf".into()
        )),
        "{changes:?}"
    );
    assert!(!dir.join("src/api/shop/generated/pets.wf").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_sync_did_not_write_is_never_overwritten() {
    let dir = scratch("petstore", "hand");
    let file = dir.join("src/api/shop/generated/pets.wf");
    let text = std::fs::read_to_string(&file).unwrap();
    // A hand edit that took the marker off.
    std::fs::write(&file, text.lines().skip(1).collect::<Vec<_>>().join("\n")).unwrap();
    let e = api::sync(&dir, "Shop", &source(&dir, "Shop"))
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("`src/api/shop/generated/pets.wf`") && e.contains("not written by sync"),
        "{e}"
    );
    assert!(
        !std::fs::read_to_string(&file)
            .unwrap()
            .starts_with("// Code generated"),
        "left as it was"
    );

    // And one beside them, written by hand, is not removed.
    std::fs::write(&file, &text).unwrap();
    std::fs::write(
        dir.join("src/api/shop/generated/mine.wf"),
        "type Mine { a: String }\n",
    )
    .unwrap();
    let e = api::sync(&dir, "Shop", &source(&dir, "Shop"))
        .unwrap_err()
        .to_string();
    assert!(e.contains("mine.wf"), "{e}");
    assert!(dir.join("src/api/shop/generated/mine.wf").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn eject_turns_from_into_a_kept_service() {
    let dir = std::env::temp_dir().join(format!("wf-openapi-eject-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::copy(
        fixture("petstore").join("petstore.yaml"),
        dir.join("petstore.yaml"),
    )
    .unwrap();
    std::fs::copy(
        fixture("petstore").join("src/pages/Pets.wf"),
        dir.join("src/pages/Pets.wf"),
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        "{\n  \"name\": \"eject\",\n  \"build\": { \"output\": \"build\" }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "/// The pet shop.\napi Shop from \"petstore.yaml\" (base: \"/v1\")\n\napp { Router }\n",
    )
    .unwrap();
    let before = types_json(&dir);

    let ejected = api::eject(&dir, "Shop", None).unwrap();
    assert_eq!(ejected.from_file, "src/App.wf");
    assert_eq!(
        std::fs::read_to_string(dir.join("src/App.wf")).unwrap(),
        "app { Router }\n"
    );
    let service = std::fs::read_to_string(dir.join("src/api/shop/service.wf")).unwrap();
    assert!(
        service.contains("/// The pet shop.\napi Shop(base: \"/v1\")"),
        "{service}"
    );
    assert!(dir.join("src/api/shop/generated/pets.wf").exists());
    let config = std::fs::read_to_string(dir.join("webfluent.app.json")).unwrap();
    assert!(config.contains("\"spec\": \"petstore.yaml\""), "{config}");
    assert_eq!(types_json(&dir), before, "the program is the one it was");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn add_records_the_service_once() {
    let dir = std::env::temp_dir().join(format!("wf-openapi-add-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::copy(
        fixture("people").join("openapi.json"),
        dir.join("openapi.json"),
    )
    .unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        "{\n  \"name\": \"add\"\n}\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), "app { Router }\n").unwrap();
    let added = api::add(&dir, "openapi.json", "Backend", None).unwrap();
    assert_eq!(added.service.as_deref(), Some("src/api/backend/service.wf"));
    let service = std::fs::read_to_string(dir.join("src/api/backend/service.wf")).unwrap();
    assert!(
        service.contains("api Backend(base: \"https://api.example.com/v1\")"),
        "{service}"
    );
    assert!(
        service.contains("// middleware [bearer(Session.token)]"),
        "{service}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join(".gitattributes")).unwrap(),
        "src/api/backend/generated/** linguist-generated=true\n"
    );
    let e = api::add(&dir, "openapi.json", "Backend", None)
        .unwrap_err()
        .to_string();
    assert!(e.contains("already kept"), "{e}");
    // A config `wf check` reads without a word about `apis`.
    let found = findings(&dir);
    assert!(!found.contains("E112") && !found.contains("N07"), "{found}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_template_reads_its_spec_as_a_build_does() {
    let dir = scratch("petstore", "template");
    let page = "api Shop from \"petstore.yaml\" (base: \"/v1\")\n\npage Pet(path: \"/\", title: \"Pet\", pet: Pet) {\n    Heading(pet.name).h1\n    Text(pet.tag ?? \"\")\n}\n";
    std::fs::write(dir.join("pet.wf"), page).unwrap();
    let tpl = webfluent::Template::from_file(dir.join("pet.wf")).unwrap();
    let html = tpl
        .render_html_fragment(
            &serde_json::json!({ "pet": { "id": 1, "name": "Rex", "tag": "dog" } }),
        )
        .unwrap();
    assert!(html.contains("Rex") && html.contains("dog"), "{html}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// What the whole pipeline says of `src`, its service blocks placed as a
/// build places them.
fn errors(src: &str) -> Vec<String> {
    let (mut program, out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    webfluent::openapi::expand(&mut program, &|_| None).unwrap();
    let source = src.to_string();
    webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
        program: &program,
        file_of: &|_| "t.wf".to_string(),
        source_of: &|_| Some(source.clone()),
        dir: None,
        config: None,
        declaration_files: &[],
        scripts: &[],
        stylesheets: "",
        incomplete: false,
    })
    .diagnostics
    .into_iter()
    .filter(|d| d.severity == webfluent::diagnostics::Severity::Error)
    .map(|d| format!("[{}] {}", d.code, d.message))
    .collect()
}

const GROUP: &str = r#"
type User { id: String, name: String }
api Backend.users {
    get list(page: Number = 1) at "users" -> [User]!
    get byId(id: String) at "users/{id}" -> User!
    delete remove(id: String) at "users/{id}" -> !
}
api Backend.posts {
    get feed at "posts" -> [String]!
}
"#;

#[test]
fn a_service_block_is_held_to_the_groups_it_names() {
    let ok = format!(
        "api Backend(base: \"/api\") {{\n    invalidations {{\n        users.remove(id) invalidates [users.list, users.byId(id)]\n    }}\n    pagination {{\n        users.list pages .number(param: page)\n    }}\n}}\n{GROUP}"
    );
    assert!(errors(&ok).is_empty(), "{:?}", errors(&ok));

    for (block, want) in [
        (
            "invalidations { users.delete invalidates [users.list] }",
            "[N05] `Backend` names `users.delete`, which no group of it has",
        ),
        (
            "invalidations { users.list invalidates [users.byId] }",
            "[N05] `users.list` is a read: it changes nothing",
        ),
        (
            "invalidations { users.remove invalidates [posts.feed] }",
            "[N05] `users.remove` invalidates `posts.feed`, a read of another group",
        ),
        (
            "invalidations { users.remove(key) invalidates [users.list] }",
            "[N05] `users.remove` takes no parameter `key`",
        ),
        (
            "invalidations { users.remove invalidates [users.lst] }",
            "[N05] `users.remove` invalidates `users.lst`, which no group of `Backend` has",
        ),
        (
            "pagination { users.lst pages .number(param: page) }",
            "[N14] `Backend` pages `users.lst`, which no group of it has",
        ),
        (
            "pagination { users.list pages .number(param: size) }",
            "[N14]",
        ),
    ] {
        let src = format!("api Backend(base: \"/api\") {{\n    {block}\n}}\n{GROUP}");
        let found = errors(&src);
        assert!(
            found.iter().any(|e| e.starts_with(want)),
            "{block}\nwanted {want}\nfound {found:?}"
        );
    }

    // A group says it on its endpoints, or its service in a block.
    let src = format!(
        "api Backend(base: \"/api\") {{ }}\napi Backend.extra {{\n    invalidations {{ users.remove invalidates [users.list] }}\n}}\n{GROUP}"
    );
    let found = errors(&src);
    assert!(
        found
            .iter()
            .any(|e| e.contains("`api Backend.extra` is a group")),
        "{found:?}"
    );
}

#[test]
fn a_status_class_names_every_status_no_other_case_does() {
    let src = r"
enum SaveError {
    conflict(problem: Problem) on 409
    clientError(problem: Problem?) on 4XX
    serverError(problem: Problem?) on 5xx
    http(error: HttpError)
}
";
    assert!(errors(src).is_empty(), "{:?}", errors(src));
    let program = webfluent::parse_source(src, "t.wf").unwrap();
    let printed = webfluent::syntax::print::file_text(&program);
    assert!(
        printed.contains("clientError(problem: Problem?) on 4XX"),
        "{printed}"
    );
    assert!(
        printed.contains("serverError(problem: Problem?) on 5XX"),
        "{printed}"
    );
    let twice = "enum E {\n    a(problem: Problem) on 4XX\n    b(problem: Problem) on 4XX\n    http(error: HttpError)\n}\n";
    assert!(
        errors(twice)
            .iter()
            .any(|e| e.contains("both stand for 4XX")),
        "{:?}",
        errors(twice)
    );
}
