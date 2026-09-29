//! The tools as a library: what `wf init`, `wf test`, `wf migrate`,
//! `wf audit`, `wf docs` and `wf verify` do, called the way the mobile
//! editor calls them, with no terminal and no Chrome.

use std::path::{Path, PathBuf};

use webfluent::init::{Template, create};
use webfluent::testing::{self, Browser, Outcome, Page};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-tools-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build" } }"#,
    )
    .unwrap();
    dir
}

fn build(dir: &Path) {
    let options = webfluent::build::BuildOptions::new(dir, &webfluent::vfs::FsVfs);
    webfluent::build::build(&options)
        .map(|_| ())
        .unwrap_or_else(|f| panic!("{} does not build: {}", dir.display(), f.error));
}

#[test]
fn every_template_makes_a_project_that_builds() {
    for template in Template::ALL {
        let parent = scratch(&format!("init-{}", template.name()));
        let dir = parent.join("site");
        create(&dir, "site", template).unwrap();
        assert!(dir.join("webfluent.app.json").is_file(), "{template:?}");
        assert!(dir.join(".gitignore").is_file(), "{template:?}");
        build(&dir);
        // A second one in the same place is refused.
        assert!(create(&dir, "site", template).is_err());
    }
    assert_eq!(Template::parse("static"), Some(Template::Static));
    assert_eq!(Template::parse("book"), None);
}

#[test]
fn tests_are_collected_rendered_and_held_to_their_snapshots() {
    let dir = scratch("testing");
    std::fs::write(
        dir.join("src/App.wf"),
        "component Greeting(_ name: String) {\n    Card { Text(\"Hello, {name}\") }\n}\npage P(path: \"/\", title: \"T\", description: \"d\") { Heading(\"x\").h1  Greeting(\"world\") }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/greeting.wf"),
        "test \"greets by name\" {\n    Greeting(\"Sam\")\n    expect \"Hello, Sam\"\n}\ntest \"is wrong on purpose\" {\n    Greeting(\"Sam\")\n    expect \"Goodbye\"\n}\n",
    )
    .unwrap();
    let tests = testing::collect(&dir).unwrap();
    assert_eq!(tests.tests.len(), 2);
    assert_eq!(tests.acting(), 0);
    let (file, good) = &tests.tests[0];
    assert_eq!(file, "tests/greeting.wf");
    assert!(good.span.line >= 1);
    // The first run writes the snapshot; the next holds the render to it.
    assert_eq!(tests.run_rendered(file, good, false), Outcome::Written);
    assert_eq!(tests.run_rendered(file, good, false), Outcome::Passed);
    let snapshot = testing::snapshot_path(&tests.snapshots, file, good);
    assert!(snapshot.is_file());
    let (_, bad) = &tests.tests[1];
    match tests.run_rendered(file, bad, false) {
        Outcome::Failed(reasons) => assert!(reasons[0].contains("\"Goodbye\""), "{reasons:?}"),
        other => panic!("{other:?}"),
    }
}

/// A browser that is not one: it answers the scripts a test's steps run,
/// and remembers the site it was asked to open.
struct Fake {
    opened: Option<PathBuf>,
    shown: String,
}

struct FakePage<'a> {
    fake: &'a mut Fake,
}

impl Page for FakePage<'_> {
    fn eval(&mut self, script: &str) -> webfluent::error::Result<serde_json::Value> {
        if script.contains("innerText") {
            return Ok(serde_json::json!(self.fake.shown));
        }
        // A click on "Add": the page now shows 1.
        if script.contains("\"click\"") && script.contains("\"Add\"") {
            self.fake.shown = "Count 1".to_string();
            return Ok(serde_json::json!("ok"));
        }
        Ok(serde_json::json!("none"))
    }

    fn settle(&mut self, _ms: u64) -> webfluent::error::Result<()> {
        Ok(())
    }

    fn errors(&self) -> Vec<String> {
        Vec::new()
    }
}

impl Browser for Fake {
    fn open(&mut self, dir: &Path, _settle_ms: u64) -> webfluent::error::Result<Box<dyn Page + '_>> {
        assert!(dir.join("index.html").is_file() && dir.join("app.js").is_file());
        self.opened = Some(dir.to_path_buf());
        self.shown = "Count 0".to_string();
        Ok(Box::new(FakePage { fake: self }))
    }
}

#[test]
fn a_test_that_acts_is_built_as_a_site_and_played_in_any_browser() {
    let dir = scratch("acting");
    std::fs::write(
        dir.join("src/App.wf"),
        "page P(path: \"/\", title: \"T\", description: \"d\") { Heading(\"x\").h1 }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/count.wf"),
        "test \"adds\" {\n    state n = 0\n    Text(\"Count {n}\")\n    Button(\"Add\") { on click { n = n + 1 } }\n    click \"Add\"\n    expect \"Count 1\"\n}\ntest \"removes\" {\n    Button(\"Add\")\n    click \"Remove\"\n    expect \"gone\"\n}\n",
    )
    .unwrap();
    let tests = testing::collect(&dir).unwrap();
    assert_eq!(tests.acting(), 2);
    let mut fake = Fake { opened: None, shown: String::new() };
    let work = dir.join(".work");
    let (_, adds) = &tests.tests[0];
    assert_eq!(tests.run_acting(&mut fake, adds, &work), Outcome::Passed);
    assert!(fake.opened.is_some());
    let (_, removes) = &tests.tests[1];
    match tests.run_acting(&mut fake, removes, &work) {
        Outcome::Failed(reasons) => assert_eq!(reasons[0], "nothing to click called \"Remove\""),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_migration_is_planned_in_memory_and_then_written() {
    let dir = scratch("migrate");
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/migrate/corpus/dashboard");
    copy(&corpus, &dir);
    let before = std::fs::read_to_string(dir.join("src/App.wf")).unwrap();
    let plan = webfluent::migrate::project::plan(&dir, false).unwrap();
    assert!(plan.changed().count() > 0, "{plan:?}");
    assert_eq!(plan.failed().count(), 0);
    assert!(plan.four.is_some());
    // Nothing is written until it is applied.
    assert_eq!(std::fs::read_to_string(dir.join("src/App.wf")).unwrap(), before);
    let written = webfluent::migrate::project::apply(&plan).unwrap();
    assert!(!written.is_empty());
    // Carried forward, the project has nothing left to migrate.
    let again = webfluent::migrate::project::plan(&dir, false).unwrap();
    assert_eq!(again.changed().count(), 0, "{again:?}");
}

#[test]
fn an_audit_says_what_a_project_trusts() {
    let dir = scratch("audit");
    std::fs::write(
        dir.join("src/App.wf"),
        "store Prefs {\n    persist theme = \"light\"\n}\npage P(path: \"/\", title: \"T\", description: \"d\") {\n    Heading(\"x\").h1\n    Image(src: \"https://images.example.com/a.png\", alt: \"a\")\n}\n",
    )
    .unwrap();
    let report = webfluent::audit::audit_project(&dir).unwrap();
    assert!(
        report.persisted.iter().any(|p| p.name == "theme" && p.owner == "Prefs"),
        "{report:?}"
    );
    assert!(!report.dependencies.is_empty());
    assert!(!report.csp.is_empty());
}

#[test]
fn the_gallery_shows_the_built_ins_and_the_project() {
    let dir = scratch("gallery");
    std::fs::write(
        dir.join("src/App.wf"),
        "component Greeting(_ name: String) {\n    Text(\"Hello, {name}\")\n}\npage P(path: \"/\", title: \"T\", description: \"d\") { Greeting(\"x\") }\n",
    )
    .unwrap();
    let (html, builtins, declared) = webfluent::gallery::for_project(&dir);
    assert!(builtins > 30);
    assert_eq!(declared, Some(1));
    assert!(html.contains("Greeting") && html.contains("Button"));
}

#[test]
fn verify_finds_the_routes_of_a_build_and_what_no_page_drew() {
    let dir = scratch("verify");
    std::fs::write(
        dir.join("src/App.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"Home\").h1 }\npage About(path: \"/about\", title: \"About\", description: \"d\") { Heading(\"About\").h1 }\n",
    )
    .unwrap();
    build(&dir);
    let routes = webfluent::verify::routes(&dir.join("build"), &dir, "").unwrap();
    assert!(routes.contains(&"/".to_string()) && routes.contains(&"/about".to_string()), "{routes:?}");
    let undrawn = webfluent::verify::undrawn(&Default::default());
    assert!(undrawn.iter().any(|n| n == "Button"));
    let mut visit = webfluent::verify::Visit::default();
    webfluent::verify::judge(&mut visit, "/about", Some(100));
    assert_eq!(visit.errors, vec!["the page rendered no text".to_string()]);
    webfluent::verify::read_report(
        &mut visit,
        r#"{"title":"About","fcp":250,"elements":12,"text":5,"drew":["wf-heading"],"images":[]}"#,
    );
    assert_eq!((visit.first_contentful_paint, visit.text), (250, 5));
}

fn copy(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let dest = to.join(path.file_name().unwrap());
        if path.is_dir() {
            std::fs::create_dir_all(&dest).unwrap();
            copy(&path, &dest);
        } else {
            std::fs::write(&dest, std::fs::read(&path).unwrap()).unwrap();
        }
    }
}
