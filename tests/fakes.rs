//! Fakes: a group's endpoints answered in place of the network, held to the
//! group's contract (`N06`); a test that names them (`fakes: [ … ]`) and
//! what it expects was called; `wf verify --fakes`, which builds a site
//! with them; and a read's value in `wf render` data. A test played in
//! Chrome is `wf test tests/fixtures/fakes`; the runtime's half is
//! `tests/js/fakes.test.mjs`.

use std::path::{Path, PathBuf};

use webfluent::testing::{self, Browser, Page};

const CONTRACT: &str = r#"type User { id: String, name: String }
type NewUser { name: String }

enum CreateUserError {
    unprocessableContent(problem: Problem) on 422
    http(error: HttpError)
}

api Backend(base: "/api") { }

api Backend.users {
    get list(page: Number = 1) at "users" -> [User]!
    post create(body: NewUser) at "users" -> User ! CreateUserError
    invalidates [list]
    delete remove(id: String) at "users/{id}" -> !
}
"#;

const PAGE: &str = r#"
app { Router }

page Users(path: "/", title: "Users", description: "d") {
    resource users = Backend.users.list()
    Heading("Users").h1
    match users {
        .loaded(list) { for u in list by u.id { Text(u.name) } }
        else { Spinner }
    }
}
"#;

const FAKE: &str = r#"fake OneUser for Backend.users {
    list(page) {
        .ok([User(id: "{page}", name: "Ada")])
    }
    create(body) {
        if body.name == "" { .err(.unprocessableContent(Problem(title: "A name is needed"))) } else { .ok(User(id: "2", name: body.name)) }
    }
    remove(id) {
        .err(.status(500, null))
    }
}
"#;

fn errors(src: &str) -> Vec<String> {
    let (program, out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
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

fn with_fake(fake: &str) -> Vec<String> {
    errors(&format!("{CONTRACT}{PAGE}{fake}"))
}

#[test]
fn a_fake_that_keeps_the_contract_checks() {
    assert_eq!(with_fake(FAKE), Vec::<String>::new());
    // Any of an endpoint's parameters, or none of them; `after`, which is
    // also the language's, is the endpoint's own name for it.
    let paged = "type Page { items: [User], next: String? }\napi Feed(base: \"/f\") {\n    get feed(after: String?) at \"feed\" -> Page!\n}\nfake F for Feed {\n    feed(after) {\n        .ok(Page(items: [], next: after))\n    }\n}\n";
    assert_eq!(
        errors(&format!("{CONTRACT}{PAGE}{paged}")),
        Vec::<String>::new()
    );
}

#[test]
fn a_fake_is_held_to_its_groups_contract() {
    let cases = [
        (
            FAKE.replace("for Backend.users", "for Backend.usres"),
            "`fake OneUser` answers for `Backend.usres`, which is not a service or a group of one",
        ),
        (
            FAKE.replace("    list(page) {", "    lists(page) {"),
            "`Backend.users` has no endpoint `lists` for `fake OneUser` to answer",
        ),
        (
            FAKE.replace("    list(page) {", "    list(pages) {"),
            "`Backend.users.list` has no parameter `pages`",
        ),
        (
            FAKE.replace(
                "    remove(id) {",
                "    list() {\n        .ok([])\n    }\n    remove(id) {",
            ),
            "`fake OneUser` answers `list` twice",
        ),
        (
            FAKE.replace(".err(.status(500, null))", "5"),
            "`remove` of `fake OneUser` answers `Number`, but `Backend.users.remove` answers `!`",
        ),
    ];
    for (src, want) in cases {
        let found = with_fake(&src);
        assert!(
            found
                .iter()
                .any(|d| d.starts_with("[N06]") && d.contains(want)),
            "{want}\n{found:?}"
        );
    }
    // A value inside an answer that is wrong for it is a value of the
    // wrong type, where it is written.
    let found = with_fake(&FAKE.replace("name: \"Ada\"", "name: 3"));
    assert!(found.iter().any(|d| d.starts_with("[T01]")), "{found:?}");
}

#[test]
fn fakes_tests_and_called_endpoints_print_back_as_written() {
    let src = format!(
        "{FAKE}\ntest \"adds\"(data: {{ n: 1 }}, fakes: [OneUser]) {{\n    Users\n    expect \"Ada\"\n    expect called Backend.users.create\n    expect not called Backend.users.remove\n}}\n"
    );
    let program = webfluent::syntax::parse_source(&src, "t.wf").expect("parses");
    let printed: Vec<String> = program
        .declarations
        .iter()
        .map(|d| webfluent::syntax::print::declaration(d, &program))
        .collect();
    assert_eq!(printed.join("\n\n"), src.trim_end());
    let webfluent::syntax::parser::ast::Declaration::Test(test) = &program.declarations[1] else {
        panic!("a test");
    };
    assert_eq!(test.fakes, ["OneUser"]);
    // A test that names fakes is played in a browser, where a call is made.
    assert!(test.acts());
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-fakes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build" } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), format!("{CONTRACT}{PAGE}")).unwrap();
    dir
}

fn faults(dir: &Path) -> Vec<String> {
    match testing::collect(dir) {
        Err(webfluent::WebFluentError::Diagnostics(d)) => d
            .iter()
            .map(|d| format!("[{}] {}:{} {}", d.code, d.file, d.line, d.message))
            .collect(),
        Err(other) => panic!("{other}"),
        Ok(_) => Vec::new(),
    }
}

#[test]
fn a_test_file_s_fakes_and_what_its_tests_name_are_checked() {
    let dir = scratch("names");
    std::fs::write(
        dir.join("tests/users.wf"),
        format!("{FAKE}\ntest \"lists\"(fakes: [OneUser]) {{\n    Users\n    expect \"Ada\"\n}}\n"),
    )
    .unwrap();
    assert_eq!(faults(&dir), Vec::<String>::new());

    let broken = FAKE.replace("    list(page) {", "    list(pages) {");
    std::fs::write(
        dir.join("tests/users.wf"),
        format!("{broken}\ntest \"lists\"(fakes: [OneUsr, OneUser]) {{\n    Users\n    expect \"Ada\"\n    expect called Backend.users.nope\n}}\n"),
    )
    .unwrap();
    let found = faults(&dir);
    let want = [
        "[N06] tests/users.wf:2 `Backend.users.list` has no parameter `pages`",
        "[N06] tests/users.wf:13 test \"lists\" names the fake `OneUsr`, and nothing declares it",
        "[N06] tests/users.wf:13 test \"lists\" expects a call of `Backend.users.nope`, which no `api` declares",
    ];
    for w in want {
        assert!(found.iter().any(|f| f == w), "{w}\n{found:#?}");
    }

    // Two fakes answering one endpoint in one test.
    let other = FAKE.replace("fake OneUser", "fake Another");
    std::fs::write(
        dir.join("tests/users.wf"),
        format!("{FAKE}\n{other}\ntest \"lists\"(fakes: [OneUser, Another]) {{\n    Users\n    expect \"Ada\"\n}}\n"),
    )
    .unwrap();
    let found = faults(&dir);
    assert!(
        found
            .iter()
            .any(|f| f.contains("`OneUser` and `Another` both answer `Backend.users.list`")),
        "{found:#?}"
    );
}

/// What a test's page was built as.
#[derive(Default)]
struct Looker {
    app: String,
}

struct Shown;

impl Page for Shown {
    fn eval(&mut self, script: &str) -> webfluent::diagnostics::error::Result<serde_json::Value> {
        if script.contains("__wfCalls") {
            return Ok(serde_json::json!(true));
        }
        Ok(serde_json::json!("Users Ada"))
    }

    fn settle(&mut self, _ms: u64) -> webfluent::diagnostics::error::Result<()> {
        Ok(())
    }

    fn errors(&self) -> Vec<String> {
        Vec::new()
    }
}

impl Browser for Looker {
    fn open(
        &mut self,
        dir: &Path,
        _settle_ms: u64,
    ) -> webfluent::diagnostics::error::Result<Box<dyn Page + '_>> {
        self.app = std::fs::read_to_string(dir.join("app.js")).unwrap();
        Ok(Box::new(Shown))
    }
}

#[test]
fn a_test_s_page_carries_its_fakes_and_a_build_carries_none() {
    let dir = scratch("page");
    let other = FAKE.replace("fake OneUser", "fake NoUsers");
    std::fs::write(
        dir.join("tests/users.wf"),
        format!("{FAKE}\n{other}\ntest \"lists\"(fakes: [OneUser]) {{\n    Users\n    expect \"Ada\"\n    expect called Backend.users.list\n}}\ntest \"plain\"(fakes: []) {{\n    Users\n    click \"Users\"\n    expect \"Users\"\n}}\n"),
    )
    .unwrap();
    let tests = testing::collect(&dir).unwrap();
    let mut browser = Looker::default();
    let work = dir.join(".work");
    let (_, named) = &tests.tests[0];
    assert_eq!(
        tests.run_acting(&mut browser, named, &work),
        testing::Outcome::Passed
    );
    assert!(browser.app.contains("window.__wfFakes = {};"), "the table");
    assert!(
        browser
            .app
            .contains("window.__wfFakes[\"Backend.users.list\"]")
    );
    assert_eq!(
        browser.app.matches("window.__wfFakes[\"").count(),
        3,
        "the fake the test names, each of its answers, and no other fake"
    );
    // A parameter the call leaves out is its default, as the server takes it.
    let table = browser
        .app
        .lines()
        .find(|l| l.contains("window.__wfFakes[\"Backend.users.list\"]"))
        .unwrap_or_default()
        .to_string();
    assert!(
        table.contains("$args[\"page\"] === undefined ? 1 : $args[\"page\"]"),
        "{table}"
    );

    // A test that names none still answers nothing from the network.
    let (_, plain) = &tests.tests[1];
    let _ = tests.run_acting(&mut browser, plain, &work);
    assert!(browser.app.contains("window.__wfFakes = {};"));
    assert!(!browser.app.contains("window.__wfFakes[\""));

    // A build leaves fakes out, wherever they are declared.
    std::fs::write(dir.join("src/Fakes.wf"), FAKE).unwrap();
    let options = webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs);
    webfluent::build::build(&options).unwrap_or_else(|f| panic!("{}", f.error));
    let app = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert!(!app.contains("__wfFakes = {}"), "no table");
    assert!(!app.contains("Ada"), "no answer");
}

#[test]
fn a_build_with_a_file_of_fakes_answers_with_them() {
    let dir = scratch("verify");
    std::fs::write(dir.join("tests/fakes.wf"), FAKE).unwrap();
    let mut options = webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs);
    options.output_dir = Some(dir.join("faked"));
    options.fakes = Some("tests/fakes.wf");
    webfluent::build::build(&options).unwrap_or_else(|f| panic!("{}", f.error));
    let app = std::fs::read_to_string(dir.join("faked/app.js")).unwrap();
    assert!(app.contains("window.__wfFakes[\"Backend.users.create\"]"));

    // One that breaks the contract stops it, at its place in that file.
    std::fs::write(
        dir.join("tests/fakes.wf"),
        FAKE.replace("    list(page) {", "    lists(page) {"),
    )
    .unwrap();
    let failure = webfluent::build::build(&options).expect_err("refused");
    let said = failure
        .report
        .lines
        .iter()
        .map(|l| l.text.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        said.contains("error[N06]") && said.contains("tests/fakes.wf:2:5"),
        "{said}"
    );
}

#[test]
fn a_read_in_render_data_paints_what_it_holds() {
    let src = "type User { id: String, name: String }\npage Users(path: \"/\", title: \"Users\") {\n    resource users: [User] = http.get(\"/api/users\")\n    match users {\n        .loading { Text(\"Loading\") }\n        .failed(e) { Text(\"Failed: {e.message}\") }\n        .loaded(list) { for u in list by u.id { Text(u.name) } }\n    }\n    Text(\"{users.value?.length ?? 0} shown, loaded {users.isLoaded}\")\n}\n";
    let tpl = webfluent::Template::from_str(src).expect("checks");
    let html = |data: serde_json::Value| tpl.render_html_fragment(&data).expect("renders");
    let loaded = html(serde_json::json!({ "users": { "loaded": [{ "id": "1", "name": "Ada" }] } }));
    assert!(
        loaded.contains(">Ada<") && loaded.contains("1 shown, loaded true"),
        "{loaded}"
    );
    let failed = html(serde_json::json!({ "users": { "failed": { "message": "down" } } }));
    assert!(failed.contains("Failed: down"), "{failed}");
    let none = html(serde_json::json!({}));
    assert!(
        none.contains("Loading") && none.contains("0 shown"),
        "{none}"
    );
    // What `loaded` gives is held to the read's type.
    let refused = tpl
        .render_html_fragment(&serde_json::json!({ "users": { "loaded": [{ "id": "1" }] } }))
        .expect_err("refused")
        .to_string();
    assert!(
        refused.contains("users.loaded[0].name is missing"),
        "{refused}"
    );
}

#[test]
fn a_test_names_a_fake_any_test_file_declares() {
    let dir = scratch("elsewhere");
    // A file of fakes, with what its fakes read beside them.
    let fake = FAKE.replace("name: \"Ada\"", "name: WHO");
    std::fs::write(
        dir.join("tests/fakes.wf"),
        format!("const WHO = \"Ada\"\n{fake}"),
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/users.wf"),
        "test \"lists\"(fakes: [OneUser]) {\n    Users\n    expect \"Ada\"\n}\n",
    )
    .unwrap();
    assert_eq!(faults(&dir), Vec::<String>::new());
    let tests = testing::collect(&dir).unwrap();
    let mut browser = Looker::default();
    let (_, test) = &tests.tests[0];
    assert_eq!(
        tests.run_acting(&mut browser, test, &dir.join(".work")),
        testing::Outcome::Passed
    );
    assert!(
        browser
            .app
            .contains("window.__wfFakes[\"Backend.users.list\"]"),
        "the other file's fake answers"
    );
    assert!(
        browser.app.contains("\"Ada\""),
        "with the constant beside it"
    );

    // One file named runs its tests alone, and still reaches the fake.
    let only = testing::collect(&dir.join("tests/users.wf")).unwrap();
    assert_eq!(only.tests.len(), 1);
    assert_eq!(only.tests[0].0, "tests/users.wf");

    // A fake of one name in two other files cannot be told apart.
    std::fs::write(
        dir.join("tests/more.wf"),
        format!("const WHO = \"Bo\"\n{fake}"),
    )
    .unwrap();
    let found = faults(&dir);
    assert!(
        found.iter().any(|f| f.contains("[N06]")
            && f.contains("`OneUser`")
            && f.contains("tests/fakes.wf")
            && f.contains("tests/more.wf")),
        "{found:#?}"
    );
    // One beside the test wins over another file's.
    std::fs::write(
        dir.join("tests/users.wf"),
        format!(
            "{}\ntest \"lists\"(fakes: [OneUser]) {{\n    Users\n    expect \"Cy\"\n}}\n",
            FAKE.replace("name: \"Ada\"", "name: \"Cy\"")
        ),
    )
    .unwrap();
    assert_eq!(faults(&dir), Vec::<String>::new());
}

#[test]
fn a_file_of_fakes_brings_its_constants_and_types_to_the_build() {
    let dir = scratch("verify-consts");
    std::fs::write(
        dir.join("tests/fakes.wf"),
        format!(
            "type Seed {{ name: String }}\nconst SEED = Seed(name: \"Ada\")\n{}",
            FAKE.replace("name: \"Ada\"", "name: SEED.name")
        ),
    )
    .unwrap();
    let mut options = webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs);
    options.output_dir = Some(dir.join("faked"));
    options.fakes = Some("tests/fakes.wf");
    webfluent::build::build(&options).unwrap_or_else(|f| {
        let said: Vec<String> = f.report.lines.iter().map(|l| l.text.clone()).collect();
        panic!("{}\n{}", f.error, said.join("\n"))
    });
    let app = std::fs::read_to_string(dir.join("faked/app.js")).unwrap();
    assert!(app.contains("window.__wfFakes[\"Backend.users.list\"]"));
    assert!(
        app.contains("Ada"),
        "the constant the fake reads is built in"
    );
}

#[test]
fn inspect_opens_a_route_with_a_file_of_fakes_answering() {
    let dir = scratch("inspect");
    std::fs::write(dir.join("tests/fakes.wf"), FAKE).unwrap();
    let run = |args: &[&str]| {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
            .args(args)
            .current_dir(&dir)
            .output()
            .expect("wf runs");
        (
            out.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
        )
    };
    // A URL is a server already running: there is nothing to build fakes into.
    let (ok, out) = run(&[
        "inspect",
        "http://127.0.0.1:9/",
        "--fakes",
        "tests/fakes.wf",
    ]);
    assert!(!ok && out.contains("--fakes"), "{out}");

    let (ok, out) = run(&["inspect", "/", "--fakes", "tests/fakes.wf"]);
    if out.contains("no Chrome or Chromium") {
        eprintln!("\n  SKIPPED: no Chrome or Chromium for `wf inspect`.\n");
        return;
    }
    assert!(ok, "{out}");
    assert!(out.contains("\"Ada\""), "the fake's user is listed: {out}");
}
