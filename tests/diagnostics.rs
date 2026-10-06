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
            ("public/a.png", "png"),
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
        &[
            (
                "src/App.wf",
                "app { Router }\npage P(path: \"/\", title: \"P\", description: \"D\") {\n    Heading(\"Hi\").h1\n    Image(src: \"/a.png\")\n}\n",
            ),
            ("public/a.png", "png"),
        ],
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

#[test]
fn lints_lower_or_raise_what_may_be_and_refuse_what_may_not() {
    let dir = project(
        "lints",
        r#"{
  "name": "d",
  "lints": { "A01": "off", "R01": "warn", "U": "error", "T05": "off", "Q99": "off", "S01": "loud" }
}"#,
        &[(
            "src/App.wf",
            "app { Router }\ntype U2 { name: String }\npage P(path: \"/\", title: \"P\", description: \"D\") {\n    state u = U2(name: \"a\")\n    state unused = 1\n    Heading(u.nmae).h1\n    Image(src: \"/a.png\")\n    Link(\"x\", to: \"/nowhere\")\n}\n",
        )],
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 1, "{err}");
    assert!(!err.contains("[A01]"), "a warning turned off: {err}");
    assert!(
        err.contains("warning[R01]"),
        "a lowerable error made a warning: {err}"
    );
    assert!(err.contains("error[U01]"), "a family made errors: {err}");
    assert!(err.contains("error[T05]"), "a T error stays one: {err}");
    assert!(
        err.contains("`lints` lowers `T05`, an error that ships a broken page"),
        "{err}"
    );
    assert!(
        err.contains("`lints` names `Q99`, which is no code or family"),
        "{err}"
    );
    assert!(err.contains("`lints.S01` is `loud`"), "{err}");
}

#[test]
fn a_project_s_files_translations_and_stored_shapes_are_held_to_its_program() {
    let dir = project(
        "project-level",
        r#"{ "name": "d", "i18n": { "default_locale": "en", "locales": ["en", "ar"], "dir": "src/translations" } }"#,
        &[
            (
                "src/App.wf",
                "app { Router }\npage P(path: \"/\", title: \"P\", description: \"D\") {\n    persist items = []\n    Heading(t(\"title\")).h1\n    Text(t(\"greet\", { nme: \"Ada\" }))\n    Text(t(\"missing.key\"))\n    Image(src: \"/hero.png\", alt: \"\")\n    Image(src: \"/here.png\", alt: \"\")\n    Button(\"FR\") { on click { setLocale(\"fr\") } }\n    Text(\"{items.length}\")\n}\n",
            ),
            (
                "src/translations/en.json",
                r#"{ "title": "Hi", "greet": "Hello, {name}!", "only.en": "x" }"#,
            ),
            (
                "src/translations/ar.json",
                r#"{ "title": "مرحبا", "greet": "أهلاً، {name}!" }"#,
            ),
            ("public/here.png", "png"),
        ],
    );
    let (_, _, err) = wf(&dir, &["build"]);
    for code in ["D01", "I01", "I02", "I03", "I04"] {
        assert!(err.contains(&format!("[{code}]")), "{code}: {err}");
    }
    assert_eq!(
        err.matches("[D01]").count(),
        1,
        "only the missing file: {err}"
    );
    assert_eq!(
        err.matches("[I02]").count(),
        2,
        "an unused `nme` and an unpassed `{{name}}`: {err}"
    );

    // D04: the shape a persisted value had at the last build is remembered.
    let app = dir.join("src/App.wf");
    let fixed = std::fs::read_to_string(&app)
        .unwrap()
        .replace("    Text(t(\"missing.key\"))\n", "")
        .replace("    Text(t(\"greet\", { nme: \"Ada\" }))\n", "")
        .replace("    Image(src: \"/hero.png\", alt: \"\")\n", "")
        .replace(
            "    Button(\"FR\") { on click { setLocale(\"fr\") } }\n",
            "",
        );
    std::fs::write(&app, &fixed).unwrap();
    std::fs::write(
        dir.join("src/translations/ar.json"),
        r#"{ "title": "مرحبا", "greet": "أهلاً، {name}!", "only.en": "x" }"#,
    )
    .unwrap();
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    std::fs::write(
        &app,
        fixed.replace("persist items = []", "persist items = { open: [] }"),
    )
    .unwrap();
    let (_, _, err) = wf(&dir, &["build"]);
    assert!(err.contains("warning[D04]"), "{err}");
}

const PAGE: &str = "page Home(path: \"/\", title: \"H\", description: \"D\") {\n";

#[test]
fn check_writes_nothing_and_speaks_each_format() {
    let dir = project(
        "check",
        r#"{ "name": "c" }"#,
        &[(
            "src/App.wf",
            &format!(
                "{APP}{PAGE}    state u = User(id: \"1\", name: \"Ada\")\n    Heading(u.nmae).h1\n}}\n"
            ),
        )],
    );
    let (code, out, err) = wf(&dir, &["check"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("error[T05]"), "{err}");
    assert!(
        out.is_empty(),
        "the human findings go to standard error: {out}"
    );
    assert!(!dir.join("build").exists(), "`wf check` wrote output");

    // JSON: the document alone on standard output, with the struct's fields.
    let (code, out, _) = wf(&dir, &["check", "--format", "json"]);
    assert_eq!(code, 1);
    let v: serde_json::Value = serde_json::from_str(&out).expect("standard output is JSON");
    assert_eq!(v["errors"], 1);
    assert_eq!(v["diagnostics"][0]["code"], "T05");
    assert_eq!(v["diagnostics"][0]["fixes"][0]["title"], "Change to `name`");

    let (_, out, _) = wf(&dir, &["check", "--format", "sarif"]);
    let v: serde_json::Value = serde_json::from_str(&out).expect("standard output is SARIF");
    assert_eq!(v["version"], "2.1.0");
    assert_eq!(v["runs"][0]["results"][0]["ruleId"], "T05");

    let (_, out, err) = wf(&dir, &["check", "--format", "github"]);
    assert!(
        out.starts_with("::error file=src/App.wf,line=5,col=15"),
        "{out}"
    );
    assert!(
        err.contains("error[T05]"),
        "the log keeps the rendering: {err}"
    );

    // A build in JSON keeps its progress off standard output, too.
    let (code, out, err) = wf(&dir, &["build", "--format", "json"]);
    assert_eq!(code, 1, "{err}");
    serde_json::from_str::<serde_json::Value>(&out).expect("a build's standard output is JSON");
    assert!(err.contains("Building c"), "{err}");

    let (code, _, err) = wf(&dir, &["check", "--format", "yaml"]);
    assert_eq!(code, 2, "{err}");
}

#[test]
fn a_clean_check_says_so_and_deny_warnings_fails_on_a_warning() {
    let dir = project(
        "deny",
        r#"{ "name": "d" }"#,
        &[("src/App.wf", &format!("{PAGE}    Heading(\"Hi\").h1\n}}\n"))],
    );
    let (code, out, _) = wf(&dir, &["check"]);
    assert_eq!(code, 0);
    assert!(out.contains("No problems in d."), "{out}");

    let dir = project(
        "deny-w",
        r#"{ "name": "d" }"#,
        &[(
            "src/App.wf",
            &format!("{PAGE}    state unused = 1\n    Heading(\"Hi\").h1\n}}\n"),
        )],
    );
    let (code, _, err) = wf(&dir, &["check"]);
    assert_eq!(code, 0, "a warning alone passes: {err}");
    let (code, _, err) = wf(&dir, &["check", "--deny-warnings"]);
    assert_eq!(code, 1, "{err}");
    assert!(
        err.contains("warning[U01]") && err.contains("--deny-warnings"),
        "{err}"
    );
    let (code, _, err) = wf(&dir, &["build", "--deny-warnings"]);
    assert_eq!(code, 1, "{err}");
    assert!(
        !dir.join("build/index.html").exists(),
        "a denied build wrote its pages"
    );
}

#[test]
fn explain_shows_a_code_s_entry() {
    let dir = std::env::temp_dir();
    let (code, out, _) = wf(&dir, &["explain", "t05"]);
    assert_eq!(code, 0);
    assert!(
        out.starts_with("T05 — A field or method that does not exist (error)"),
        "{out}"
    );
    assert!(
        out.contains("In the guide: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t05-"),
        "{out}"
    );
    let (code, _, err) = wf(&dir, &["explain", "T5"]);
    assert_eq!(code, 2);
    assert!(err.contains("is no code"), "{err}");
    let (code, out, _) = wf(&dir, &["explain"]);
    assert_eq!(code, 0);
    assert!(out.contains("U07") && out.contains("E101"), "{out}");
}

#[test]
fn an_allow_silences_one_finding_and_a_stale_one_is_reported() {
    let dir = project(
        "allow",
        r#"{ "name": "a" }"#,
        &[(
            "src/App.wf",
            &format!(
                "{PAGE}    // wf-allow(U01)\n    state kept = 1\n    state other = 2 // wf-allow(U)\n    // wf-allow(U02)\n    state stale = 3\n    Heading(\"Hi {{stale}}\").h1\n    // wf-allow(T05)\n    Text(\"x\")\n}}\n"
            ),
        )],
    );
    let (_, out, _) = wf(&dir, &["check", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let found: Vec<(String, u64)> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_string(),
                d["line"].as_u64().unwrap(),
            )
        })
        .collect();
    // `kept` and `other` are allowed; the U02 allow covers a state that is
    // read, and T05 is an error no allow may silence.
    assert_eq!(
        found,
        vec![("U07".to_string(), 5), ("U07".to_string(), 8)],
        "{out}"
    );
}

/// `wf check --format json` in `dir`: every finding.
fn findings(dir: &Path) -> Vec<serde_json::Value> {
    let (_, out, err) = wf(dir, &["check", "--format", "json"]);
    let v: serde_json::Value =
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}\n{err}"));
    v["diagnostics"].as_array().unwrap().clone()
}

#[test]
fn each_fix_applied_makes_its_finding_go() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "T05",
            "type User { id: String, name: String }\nPAGE    state u = User(id: \"1\", name: \"Ada\")\n    Heading(u.nmae).h1\n}\n",
            "Heading(u.name)",
        ),
        (
            "T13",
            "PAGE    state count = 0\n    Heading(\"Hi\").h1\n    Button(\"+\") { on click { cuont = 1 } }\n    Text(\"{count}\")\n}\n",
            "count = 1",
        ),
        (
            "T16",
            "PAGE    state xs = [\"a\"]\n    Heading(xs.joined(\", \")).h1\n}\n",
            "xs.join(",
        ),
        (
            "T06",
            "store Cart {\n    state total = 0\n}\nPAGE    use Cart\n    Heading(\"{Cart.totl}\").h1\n}\n",
            "Cart.total",
        ),
        (
            "T04",
            "type User { id: String, name: String }\nPAGE    state u: User? = null\n    Heading(\"Hi\").h1\n    Text(u.name)\n}\n",
            "u?.name",
        ),
        (
            "E101",
            "PAGE    Heading(\"Hi\").h1\n    Buton(\"Save\")\n}\n",
            "Button(\"Save\")",
        ),
        (
            "E103",
            "PAGE    Heading(\"Hi\").h1\n    Button(\"Save\").primry\n}\n",
            ".primary",
        ),
        (
            "U01",
            "PAGE    state unused = 1\n    Heading(\"Hi\").h1\n}\n",
            "state _unused",
        ),
        (
            "A01",
            "PAGE    Heading(\"Hi\").h1\n    Image(src: \"https://example.com/a.png\")\n}\n",
            "alt: \"\"",
        ),
        (
            "C01",
            "component Card2(title: String, count: Number) { Text(\"{title} {count}\") }\nPAGE    Heading(\"Hi\").h1\n    Card2(count: 1)\n}\n",
            "Card2(count: 1, title: \"\")",
        ),
        (
            "T15",
            "enum Tone { calm, loud, quiet }\nPAGE    state t: Tone = .calm\n    Heading(\"Hi\").h1\n    match t {\n        .calm { Text(\"c\") }\n    }\n}\n",
            ".quiet { }",
        ),
        (
            "V01",
            "PAGE    Heading(\"Hi\").h1\n    Button(outline) { on click { log(1) } }\n}\n",
            "Button.outlined {",
        ),
    ];
    for (code, src, expect) in cases {
        let src = src.replace("PAGE", PAGE);
        let dir = project(
            &format!("fix-{}", code.to_lowercase()),
            r#"{ "name": "f" }"#,
            &[("src/App.wf", &src)],
        );
        let before = findings(&dir);
        let finding = before
            .iter()
            .find(|d| d["code"] == *code)
            .unwrap_or_else(|| panic!("no {code} in {before:#?}"));
        let fix: webfluent::diagnostics::Fix = webfluent::diagnostics::Fix {
            title: finding["fixes"][0]["title"]
                .as_str()
                .unwrap_or_else(|| panic!("{code} offers no fix: {finding:#}"))
                .to_string(),
            edits: finding["fixes"][0]["edits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| webfluent::diagnostics::Edit {
                    file: e["file"].as_str().unwrap().to_string(),
                    line: e["line"].as_u64().unwrap() as usize,
                    column: e["column"].as_u64().unwrap() as usize,
                    end_line: e["end_line"].as_u64().unwrap() as usize,
                    end_column: e["end_column"].as_u64().unwrap() as usize,
                    text: e["text"].as_str().unwrap().to_string(),
                })
                .collect(),
        };
        let fixed = webfluent::diagnostics::fixes::apply(&src, &fix);
        assert!(
            fixed.contains(expect),
            "{code}: `{}` gave\n{fixed}",
            fix.title
        );
        std::fs::write(dir.join("src/App.wf"), &fixed).unwrap();
        let after = findings(&dir);
        assert!(
            !after.iter().any(|d| d["code"] == *code),
            "{code} is still reported after `{}`:\n{fixed}\n{after:#?}",
            fix.title
        );
    }
}

#[test]
fn a_build_keeps_what_each_persisted_value_starts_as_and_the_last_build_s() {
    let page = |initial: &str| {
        format!(
            "{PAGE}    persist items = {initial} {{ version: 2 }}\n    Heading(\"{{items.length}}\").h1\n}}\n"
        )
    };
    let dir = project(
        "persist-values",
        r#"{ "name": "p" }"#,
        &[("src/App.wf", &page("[\"a\"]"))],
    );
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(dir.join(name)).unwrap()).unwrap()
    };
    assert_eq!(
        read(".wf-cache/persist-values.json")["wf:Home.items"],
        serde_json::json!({ "wf:v": 2, "wf:d": ["a"] })
    );
    assert!(!dir.join(".wf-cache/persist-values.previous.json").exists());
    std::fs::write(dir.join("src/App.wf"), page("[\"b\", \"c\"]")).unwrap();
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        read(".wf-cache/persist-values.previous.json")["wf:Home.items"]["wf:d"],
        serde_json::json!(["a"])
    );
    assert_eq!(
        read(".wf-cache/persist-values.json")["wf:Home.items"]["wf:d"],
        serde_json::json!(["b", "c"])
    );
}

/// The codes the guide shows no example of (a project-level or output-level
/// finding needs more than one file): each through the whole pipeline.
#[test]
fn the_codes_with_no_guide_example_are_reported_where_they_are() {
    // E003: a `.wfx` line that lines up with no block.
    let dir = project(
        "e003",
        r#"{ "name": "x" }"#,
        &[(
            "src/App.wfx",
            "page Home(path: \"/\", title: \"H\", description: \"D\")\n    Heading(\"Hi\").h1\n      Text(\"a\")\n  Text(\"b\")\n",
        )],
    );
    let found = findings(&dir);
    assert!(
        found
            .iter()
            .any(|d| d["code"] == "E003" && d["file"] == "src/App.wfx"),
        "{found:#?}"
    );

    // E108: a page that reads an `env` name nobody said is public.
    let dir = project(
        "e108",
        r#"{ "name": "x", "env": { "API_KEY": "sk" } }"#,
        &[(
            "src/App.wf",
            &format!("{PAGE}    Heading(env.API_KEY).h1\n}}\n"),
        )],
    );
    let found = findings(&dir);
    let e108 = found
        .iter()
        .find(|d| d["code"] == "E108")
        .unwrap_or_else(|| panic!("{found:#?}"));
    assert_eq!(
        (e108["line"].as_u64(), e108["severity"].as_str()),
        (Some(2), Some("error"))
    );

    // A13: a theme whose text cannot be read on its background.
    let dir = project(
        "a13",
        r#"{ "name": "x" }"#,
        &[(
            "src/App.wf",
            &format!(
                "theme Pale {{\n    color-text: #EEEEEE\n    color-background: #FFFFFF\n}}\n{PAGE}    Heading(\"Hi\").h1\n}}\n"
            ),
        )],
    );
    let found = findings(&dir);
    assert!(found.iter().any(|d| d["code"] == "A13"), "{found:#?}");

    // E902: a page asks for a script from an origin its policy never named.
    let dir = project(
        "e902",
        r#"{ "name": "x", "build": { "csp": true } }"#,
        &[(
            "src/App.wf",
            &format!(
                "{PAGE}    head {{ script(src: \"https://cdn.example.com/x.js\") }}\n    Heading(\"Hi\").h1\n}}\n"
            ),
        )],
    );
    let (code, out, err) = wf(&dir, &["build", "--format", "json"]);
    assert_eq!(code, 1, "{err}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        v["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "E902"),
        "{out}"
    );
}

#[test]
fn a_file_that_does_not_parse_fails_a_check_and_the_rest_is_still_read() {
    let dir = project(
        "check-parse",
        r#"{ "name": "p" }"#,
        &[
            (
                "src/App.wf",
                &format!("{PAGE}    Heading(\"Hi\").h1\n    Text(\"a\"\n}}\n"),
            ),
            (
                "src/Other.wf",
                "page Other(path: \"/o\", title: \"O\", description: \"D\") {\n    Heading(\"O\").h1\n    Image(src: \"https://example.com/a.png\")\n}\n",
            ),
        ],
    );
    let (code, out, _) = wf(&dir, &["check", "--format", "json"]);
    assert_eq!(code, 1);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let codes: Vec<&str> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap())
        .collect();
    assert!(
        codes.contains(&"E002") && codes.contains(&"A01"),
        "{codes:?}"
    );
}

#[test]
fn a_github_build_annotates_and_a_denied_json_check_still_writes_its_document() {
    let dir = project(
        "github-build",
        r#"{ "name": "g" }"#,
        &[(
            "src/App.wf",
            &format!("{PAGE}    state unused = 1\n    Heading(\"Hi\").h1\n}}\n"),
        )],
    );
    let (code, out, err) = wf(&dir, &["build", "--format", "github"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("::warning file=src/App.wf,line=2,"), "{out}");
    assert!(out.contains("title=U01::"), "{out}");
    let (code, out, _) = wf(&dir, &["check", "--format", "json", "--deny-warnings"]);
    assert_eq!(code, 1);
    let v: serde_json::Value =
        serde_json::from_str(&out).expect("one JSON document, even when denied");
    assert_eq!(v["warnings"], 1);
}

#[test]
fn an_allow_works_in_the_indented_layout() {
    let dir = project(
        "allow-wfx",
        r#"{ "name": "a" }"#,
        &[(
            "src/App.wfx",
            "page Home(path: \"/\", title: \"H\", description: \"D\")\n    // wf-allow(U01)\n    state kept = 1\n    state other = 2\n    Heading(\"Hi\").h1\n",
        )],
    );
    let found: Vec<(String, u64)> = findings(&dir)
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_string(),
                d["line"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(found, vec![("U01".to_string(), 4)]);
}

#[test]
fn more_fixes_applied_make_their_findings_go() {
    let cases: &[(&str, &str, &str)] = &[
        // The arms of a match expression are values.
        (
            "T15",
            "enum Tone { calm, loud }\nPAGE    state t: Tone = .calm\n    derived label = match t { .calm { \"c\" } }\n    Heading(label).h1\n}\n",
            ".loud { null }",
        ),
        // A case's payload is named by its fields.
        (
            "T15",
            "enum Status { idle, failed(reason: String) }\nPAGE    state s: Status = .idle\n    Heading(\"Hi\").h1\n    match s {\n        .idle { Text(\"i\") }\n    }\n}\n",
            ".failed(reason) { }",
        ),
        // A call with no parentheses gets them.
        (
            "C01",
            "component Card2(title: String) { Text(title) }\nPAGE    Heading(\"Hi\").h1\n    Card2\n}\n",
            "Card2(title: \"\")",
        ),
        // An enum prop is filled with its first case.
        (
            "C01",
            "enum Tone { calm, loud }\ncomponent Chip(tone: Tone) { Text(\"chip\") }\nPAGE    Heading(\"Hi\").h1\n    Chip()\n}\n",
            "Chip(tone: .calm)",
        ),
    ];
    for (i, (code, src, expect)) in cases.iter().enumerate() {
        let src = src.replace("PAGE", PAGE);
        let dir = project(
            &format!("more-fix-{i}"),
            r#"{ "name": "f" }"#,
            &[("src/App.wf", &src)],
        );
        let before = findings(&dir);
        let finding = before
            .iter()
            .find(|d| d["code"] == *code)
            .unwrap_or_else(|| panic!("no {code} in {before:#?}"));
        let fix = webfluent::diagnostics::Fix {
            title: finding["fixes"][0]["title"]
                .as_str()
                .unwrap_or_else(|| panic!("{code} offers no fix: {finding:#}"))
                .to_string(),
            edits: serde_json::from_value::<Vec<serde_json::Value>>(
                finding["fixes"][0]["edits"].clone(),
            )
            .unwrap()
            .iter()
            .map(|e| webfluent::diagnostics::Edit {
                file: e["file"].as_str().unwrap().to_string(),
                line: e["line"].as_u64().unwrap() as usize,
                column: e["column"].as_u64().unwrap() as usize,
                end_line: e["end_line"].as_u64().unwrap() as usize,
                end_column: e["end_column"].as_u64().unwrap() as usize,
                text: e["text"].as_str().unwrap().to_string(),
            })
            .collect(),
        };
        let fixed = webfluent::diagnostics::fixes::apply(&src, &fix);
        assert!(
            fixed.contains(expect),
            "{code}: `{}` gave\n{fixed}",
            fix.title
        );
        std::fs::write(dir.join("src/App.wf"), &fixed).unwrap();
        let after = findings(&dir);
        assert!(
            !after.iter().any(|d| d["code"] == *code),
            "{code} is still reported after `{}`:\n{fixed}\n{after:#?}",
            fix.title
        );
    }
}

#[test]
fn a_head_script_whose_origin_the_config_declares_is_allowed() {
    let app = format!(
        "{PAGE}    head {{\n        script(src: \"https://cdn.example.com/x.js\")\n        link(rel: \"stylesheet\", href: \"https://fonts.example.com/a.css\")\n    }}\n    Heading(\"Hi\").h1\n}}\n"
    );
    let dir = project(
        "head-declared",
        r#"{ "name": "x", "build": { "csp": true }, "meta": { "scripts": ["https://cdn.example.com/lib.js"], "stylesheets": ["https://fonts.example.com/b.css"], "integrity": { "https://cdn.example.com/lib.js": "sha384-x" } } }"#,
        &[("src/App.wf", &app)],
    );
    let found = findings(&dir);
    assert!(!found.iter().any(|d| d["code"] == "E902"), "{found:#?}");
    // Without the stylesheet's origin declared, the `link` is refused.
    let dir = project(
        "head-undeclared-css",
        r#"{ "name": "x", "build": { "csp": true }, "meta": { "scripts": ["https://cdn.example.com/lib.js"], "integrity": { "https://cdn.example.com/lib.js": "sha384-x" } } }"#,
        &[("src/App.wf", &app)],
    );
    let found = findings(&dir);
    let refused: Vec<u64> = found
        .iter()
        .filter(|d| d["code"] == "E902")
        .map(|d| d["line"].as_u64().unwrap())
        .collect();
    assert_eq!(refused, vec![4], "{found:#?}");
}
