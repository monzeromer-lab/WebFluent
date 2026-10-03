//! What the compiler says, and how (`spec/DIAGNOSTICS_PLAN.md`, Part A):
//! one coded diagnostic type, one pipeline that runs every stage over every
//! file, and a renderer a person — and a problem matcher — can read.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A project on disk with these files and this config.
fn project(name: &str, config: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-diag-{name}-{}", std::process::id()));
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

/// `wf <args>` in `dir`: its exit code, standard output and standard error.
fn wf(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .env_remove("NO_COLOR")
        .output()
        .expect("wf runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

const APP: &str = "app { Router }\ntype User { id: String, name: String }\n";

#[test]
fn one_build_reports_every_file_and_every_stage_then_one_line() {
    let dir = project(
        "every-file",
        r#"{ "name": "d" }"#,
        &[
            ("src/App.wf", APP),
            (
                "src/pages/Profile.wf",
                "page Profile(path: \"/\", title: \"P\", description: \"D\") {\n    state user: User = User(id: \"1\", name: \"Ada\")\n    Heading(user.nmae).h1\n    Image(src: \"/a.png\")\n}\n",
            ),
            (
                "src/pages/Broken.wf",
                "page Broken(path: \"/b\", title: \"B\", description: \"D\") {\n    Text(\"a\"\n}\nstore S { state = 1 }\n",
            ),
        ],
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 1, "a build with errors exits 1:\n{err}");
    // Both of the broken file's mistakes, not only the first.
    assert!(
        err.contains(
            "error[E002]: Expected `,` between arguments, got `}`\n --> src/pages/Broken.wf:3:1"
        ),
        "{err}"
    );
    assert!(err.contains("--> src/pages/Broken.wf:4:17"), "{err}");
    // The other file is still checked, through the types and the lints.
    assert!(
        err.contains("error[T05]: `User` has no field `nmae`\n --> src/pages/Profile.wf:3:18\n  |\n3 |     Heading(user.nmae).h1\n  |                  ^^^^"),
        "{err}"
    );
    assert!(err.contains("warning[A01]"), "{err}");
    assert!(
        err.ends_with("error: the build stopped: 3 errors, 1 warning\n"),
        "{err}"
    );
    // Not a terminal: no colour.
    assert!(!err.contains('\x1b'), "{err}");
    // Each finding is said once.
    assert_eq!(err.matches("error[T05]").count(), 1, "{err}");
}

#[test]
fn warnings_alone_let_the_build_finish_and_are_summed_up() {
    let dir = project(
        "warnings",
        r#"{ "name": "d" }"#,
        &[(
            "src/App.wf",
            "app { Router }\npage P(path: \"/\", title: \"P\", description: \"D\") {\n    Heading(\"Hi\").h1\n    Image(src: \"/a.png\")\n}\n",
        )],
    );
    let (code, out, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    assert!(
        err.contains("warning[A01]: Image missing \"alt\" attribute\n --> src/App.wf:4:5"),
        "{err}"
    );
    assert!(err.ends_with("1 warning\n"), "{err}");
    assert!(out.contains("Build complete with 1 warning(s)."), "{out}");
}

#[test]
fn a_build_that_cannot_run_exits_2() {
    let dir = project("no-src", r#"{ "name": "d" }"#, &[]);
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 2, "{err}");
    let dir = project("bad-config", "{ not json", &[("src/App.wf", APP)]);
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 2, "{err}");
}

#[test]
fn a_duplicate_names_where_the_first_one_is() {
    let dir = project(
        "duplicate",
        r#"{ "name": "d" }"#,
        &[
            ("src/App.wf", "app { Router }\nstore Cart { state n = 0 }\n"),
            (
                "src/stores/more.wf",
                "store Cart { state m = 0 }\npage P(path: \"/\", title: \"P\", description: \"D\") { Heading(\"x\").h1 }\n",
            ),
        ],
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 1);
    assert!(err.contains("error[E102]: duplicate store `Cart`"), "{err}");
    assert!(err.contains("--> src/stores/more.wf:1:1"), "{err}");
    assert!(
        err.contains("= note: `Cart` is first declared here at src/App.wf:2:1"),
        "{err}"
    );
}

#[test]
fn the_config_the_scripts_and_the_output_mode_have_places_too() {
    let dir = project(
        "config",
        r#"{
  "name": "d",
  "build": { "minfy": true },
  "meta": { "scripts": [{ "src": "https://cdn.example.com/x.mjs", "module": true }] }
}"#,
        &[
            (
                "src/App.wf",
                "app { Router }\npage P(path: \"/\", title: \"P\", description: \"D\") { Heading(\"x\").h1 }\n",
            ),
            ("src/lib.js", "import x from \"./y.js\";\nfunction f() {}\n"),
        ],
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("warning[E112]: `build.minfy` is not a setting, and nothing reads it — did you mean `minify`?\n --> webfluent.app.json:3:15"), "{err}");
    assert!(err.contains("error[E111]: `meta.scripts` imports `https://cdn.example.com/x.mjs` as a module but names no global for it"), "{err}");
    assert!(err.contains("warning[D05]"), "{err}");
    assert!(
        err.contains("error[E110]: `src/lib.js` is an ES module")
            && err.contains("--> src/lib.js:1:1"),
        "{err}"
    );

    let dir = project(
        "pdf",
        r#"{ "name": "d", "build": { "output_type": "pdf" } }"#,
        &[(
            "src/App.wf",
            "page Report(path: \"/\", title: \"R\") {\n    Document {\n        Heading(\"Q1\").h1\n        Button(\"Click\")\n    }\n}\n",
        )],
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 1, "{err}");
    assert!(
        err.contains("error[E109]: `Button` cannot be drawn in a PDF"),
        "{err}"
    );
    assert!(err.contains("--> src/App.wf:4:9"), "{err}");
}

#[test]
fn a_template_s_errors_are_coded_findings() {
    let err = webfluent::Template::from_str("page P(path: \"/\") { Buton(\"x\") }")
        .err()
        .expect("refused");
    let found = err.diagnostics();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].code, "E101");
    assert!(err.to_string().contains("error[E101]"), "{err}");
}

/// Every finding the guide's programs draw carries a code the registry
/// holds — nothing the compiler says is uncoded.
#[test]
fn every_finding_has_a_registered_code() {
    let chapter = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("md-docs/39-diagnostics.md"),
    )
    .unwrap();
    let mut seen = 0;
    for block in chapter.split("```wf expect ").skip(1) {
        let body = block
            .split_once('\n')
            .unwrap()
            .1
            .split("```")
            .next()
            .unwrap();
        let (program, mut found) = webfluent::syntax::parse_source_recovering(body, "t.wf");
        found.extend(
            webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
                program: &program,
                file_of: &|_| "t.wf".to_string(),
                source_of: &|_| Some(body.to_string()),
                dir: None,
                config: None,
                declaration_files: &[],
                scripts: &[],
                stylesheets: "",
                incomplete: false,
            })
            .diagnostics,
        );
        for d in found {
            seen += 1;
            assert!(
                webfluent::diagnostics::codes::info(d.code).is_some(),
                "uncoded or unregistered: {d}"
            );
        }
    }
    assert!(seen > 60, "only {seen} findings");
}

/// Nothing in the compiler makes a finding without a code: `Diagnostic::new`
/// is for callers outside it.
#[test]
fn the_compiler_never_makes_an_uncoded_finding() {
    fn walk(dir: &Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                if !p.ends_with("diagnostics") {
                    walk(&p, out);
                }
            } else if p.extension().is_some_and(|x| x == "rs")
                && std::fs::read_to_string(&p)
                    .unwrap()
                    .contains("Diagnostic::new(")
            {
                out.push(p.display().to_string());
            }
        }
    }
    let mut found = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("crates"),
        &mut found,
    );
    assert!(found.is_empty(), "uncoded findings in {found:?}");
}
