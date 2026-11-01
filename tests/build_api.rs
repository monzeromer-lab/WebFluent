//! `webfluent::build`: the pipeline `wf build` runs, called as a library —
//! what an editor does to build the project its buffers describe before
//! they are saved.

use std::path::PathBuf;
use std::process::Command;

use webfluent::WebFluentError;
use webfluent::build::{BuildOptions, Stream, build};
use webfluent::vfs::{FsVfs, OverlayVfs};

/// A project on disk: a store with members `live` and `queued`, and a page
/// reading one of them.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-build-api-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::create_dir_all(dir.join("src/stores")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "deploys", "build": { "output": "build", "ssg": true } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/stores/deploys.wf"),
        "store Deploys {\n    state live = 3\n    state queued = []\n}\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/pages/Home.wf"), home("on disk", "live")).unwrap();
    dir
}

fn home(words: &str, member: &str) -> String {
    format!(
        "page Home(path: \"/\", title: \"Home\", description: \"Deploys.\") {{\n    use Deploys\n    Heading(\"Deploys\").h1\n    Text(\"{words}\")\n    Text(\"{{Deploys.{member}}} live\")\n    Text(\"{{Deploys.queued.length}} queued\")\n}}\n"
    )
}

#[test]
fn an_unsaved_buffer_is_built_instead_of_the_file() {
    let dir = project("buffer");
    let out = dir.join("elsewhere");
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(dir.join("src/pages/Home.wf"), home("unsaved", "live"));
    // A page that exists only in the editor so far.
    vfs.set(
        dir.join("src/pages/Post.wf"),
        "page Post(path: \"/posts/:slug\", slug: String, title: \"Post\", description: \"A post.\") {\n    Heading(\"Post\").h1\n}\n",
    );

    let report = build(&BuildOptions {
        output_dir: Some(out.clone()),
        ..BuildOptions::new(&dir, &vfs)
    })
    .unwrap_or_else(|failure| panic!("{failure}\n{:?}", failure.report.lines));

    let html = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("unsaved"), "{html}");
    assert!(!html.contains("on disk"), "{html}");
    // The disk is as it was, and nothing was written where the config says.
    assert!(
        std::fs::read_to_string(dir.join("src/pages/Home.wf"))
            .unwrap()
            .contains("on disk")
    );
    assert!(!dir.join("src/pages/Post.wf").exists());
    assert!(!dir.join("build").exists());

    assert_eq!(report.output_dir, out);
    assert_eq!((report.pages, report.components, report.stores), (2, 0, 1));
    let routes: Vec<(&str, &str, &str, Vec<String>)> = report
        .routes
        .iter()
        .map(|r| {
            (
                r.path.as_str(),
                r.page.as_str(),
                r.file.as_str(),
                r.params.clone(),
            )
        })
        .collect();
    assert_eq!(
        routes,
        [
            ("/", "Home", "src/pages/Home.wf", vec![]),
            (
                "/posts/:slug",
                "Post",
                "src/pages/Post.wf",
                vec!["slug".to_string()]
            ),
        ]
    );
    assert_eq!(report.errors().count(), 0);
    assert_eq!(report.lines[0].text, "Building deploys...");
    assert_eq!(report.lines.last().unwrap().text, "Build complete.");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_finding_carries_its_code_its_place_and_what_the_cli_prints() {
    let dir = project("t06");
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(dir.join("src/pages/Home.wf"), home("typo", "lve"));
    let failure = build(&BuildOptions::new(&dir, &vfs)).unwrap_err();

    let error = failure.report.errors().next().expect("an error");
    assert_eq!(error.code, "T06");
    assert_eq!(error.message, "`Deploys` has no member `lve`");
    assert_eq!(error.file, "src/pages/Home.wf");
    assert_eq!((error.line, error.column), (5, 12));
    assert_eq!(
        error.hint.as_deref(),
        Some("Did you mean `live`? Its members are `live`, `queued`")
    );
    // Rendered on stderr, at its place, and the error the build ends with
    // is the findings themselves.
    assert!(
        failure
            .report
            .lines
            .iter()
            .any(|l| l.stream == Stream::Stderr
                && l.text.contains("T06")
                && l.text.contains("src/pages/Home.wf:5:12"))
    );
    assert!(matches!(&failure.error, WebFluentError::Diagnostics(found) if found.contains(error)));

    // `wf build` on the same text says the same, line for line.
    std::fs::write(dir.join("src/pages/Home.wf"), home("typo", "lve")).unwrap();
    let cli = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!cli.status.success());
    let printed = |stream: Stream| {
        failure
            .report
            .lines
            .iter()
            .filter(|l| l.stream == stream)
            .map(|l| format!("{}\n", l.text))
            .collect::<String>()
    };
    assert_eq!(
        String::from_utf8_lossy(&cli.stdout),
        printed(Stream::Stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&cli.stderr),
        printed(Stream::Stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_typo_in_a_derived_value_is_one_error() {
    // `derived live = Deploys.lve` was found twice at the same place — as
    // the derived value's own expression and as what it declares — and
    // the build said "2 error(s)" for one typo, printing it twice.
    let dir = project("derived");
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(
        dir.join("src/pages/Home.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"Deploys.\") {\n    use Deploys\n    derived live = Deploys.lve\n    Text(\"{live} live\")\n}\n",
    );
    let failure = build(&BuildOptions::new(&dir, &vfs)).unwrap_err();
    let t06: Vec<_> = failure
        .report
        .errors()
        .filter(|e| e.code == "T06")
        .collect();
    assert_eq!(t06.len(), 1, "{:?}", failure.report.diagnostics);
    assert_eq!((t06[0].line, t06[0].column), (3, 20));
    let printed = failure
        .report
        .lines
        .iter()
        .filter(|l| l.stream == Stream::Stderr && l.text.contains("Home.wf:3:20"))
        .count();
    assert_eq!(printed, 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_parse_error_is_a_finding_with_a_place() {
    let dir = project("parse");
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(
        dir.join("src/pages/Home.wf"),
        "page Home(path: \"/\") {\n    Heading(\"Oops\".h1\n}\n",
    );
    let failure = build(&BuildOptions::new(&dir, &vfs)).unwrap_err();
    let error = failure.report.errors().next().expect("an error");
    assert_eq!(error.file, "src/pages/Home.wf");
    assert_eq!(error.line, 3);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_report_is_what_wf_build_prints() {
    let dir = project("lines");
    let copy = dir.with_extension("cli");
    let _ = std::fs::remove_dir_all(&copy);
    std::fs::create_dir_all(copy.join("src/pages")).unwrap();
    std::fs::create_dir_all(copy.join("src/stores")).unwrap();
    for file in [
        "webfluent.app.json",
        "src/pages/Home.wf",
        "src/stores/deploys.wf",
    ] {
        std::fs::copy(dir.join(file), copy.join(file)).unwrap();
    }
    // Two warnings: a key nothing reads, and an image with no alt.
    let config = r#"{ "name": "deploys", "colour": "red", "build": { "output": "build" } }"#;
    let page = home("hi", "live").replace("Heading(", "Image(src: \"/a.png\")\n    Heading(");
    for root in [&dir, &copy] {
        std::fs::write(root.join("webfluent.app.json"), config).unwrap();
        std::fs::write(root.join("src/pages/Home.wf"), &page).unwrap();
    }

    let report = build(&BuildOptions::new(&dir, &FsVfs)).unwrap();
    let cli = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&copy)
        .output()
        .unwrap();
    assert!(cli.status.success());
    let printed = |stream: Stream| {
        report
            .lines
            .iter()
            .filter(|l| l.stream == stream)
            .map(|l| format!("{}\n", l.text))
            .collect::<String>()
    };
    assert_eq!(
        String::from_utf8_lossy(&cli.stdout),
        printed(Stream::Stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&cli.stderr),
        printed(Stream::Stderr)
    );

    let warnings: Vec<(&str, &str)> = report
        .warnings()
        .map(|w| (w.code, w.file.as_str()))
        .collect();
    assert!(
        warnings.contains(&("A01", "src/pages/Home.wf")),
        "{warnings:?}"
    );
    assert!(
        warnings
            .iter()
            .any(|(_, file)| *file == "webfluent.app.json"),
        "{warnings:?}"
    );
    // Every build writes the same output, read from the same text.
    for file in ["index.html", "app.js", "styles.css"] {
        assert_eq!(
            std::fs::read(dir.join("build").join(file)).unwrap(),
            std::fs::read(copy.join("build").join(file)).unwrap(),
            "{file}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn lines_reach_the_caller_as_they_are_printed() {
    let dir = project("live");
    let seen = std::sync::Mutex::new(Vec::new());
    let on_line = |line: &webfluent::build::BuildLine| seen.lock().unwrap().push(line.clone());
    let report = build(&BuildOptions {
        on_line: Some(&on_line),
        ..BuildOptions::new(&dir, &FsVfs)
    })
    .unwrap();
    assert_eq!(*seen.lock().unwrap(), report.lines);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_project_translated_by_its_own_words() {
    let dir = std::env::temp_dir().join(format!("wf-build-api-literals-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::create_dir_all(dir.join("src/translations")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "words", "build": { "output": "build", "ssg": true }, "i18n": { "default_locale": "en", "locales": ["en", "ar"], "literals": true } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/translations/en.json"), "{}\n").unwrap();
    std::fs::write(
        dir.join("src/translations/ar.json"),
        "{ \"Problems\": \"المشاكل\", \"{n} missing\": \"{n} مفقودة\" }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/pages/Home.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"Words.\") {\n    state n = 3\n    Heading(\"Problems\").h1\n    Text(\"{n} missing\")\n    Icon(\"warning\")\n}\n",
    )
    .unwrap();
    let vfs = OverlayVfs::over_disk();
    let report = build(&BuildOptions::new(&dir, &vfs))
        .unwrap_or_else(|f| panic!("{f}\n{:?}", f.report.lines));
    assert_eq!(report.errors().count(), 0, "{:?}", report.lines);
    let i0: Vec<&str> = report
        .lines
        .iter()
        .map(|l| l.text.as_str())
        .filter(|t| t.contains("[I0"))
        .collect();
    assert!(
        i0.is_empty(),
        "the words are the default's messages: {i0:?}"
    );
    // The page reads the words as written in the default locale, and asks
    // the table for them in the others.
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(
        html.contains("Problems") && html.contains("3 missing"),
        "{html}"
    );
    let js: String = std::fs::read_dir(dir.join("build"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "js"))
        .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
        .chain(
            std::fs::read_dir(dir.join("build/pages"))
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default()),
        )
        .collect();
    assert!(
        js.contains("\"Problems\"") && js.contains("\"{n} missing\""),
        "the keys are asked for"
    );
    assert!(js.contains("المشاكل"), "the table is in the build");
    // The table counts them as used, and what the other locale is missing.
    let t = webfluent::i18n::table::table_via(&vfs, &dir)
        .unwrap()
        .unwrap();
    assert!(t.keys.iter().any(|k| k.key == "Problems" && !k.unused));
    let ar = t.locales.iter().find(|l| l.code == "ar").unwrap();
    assert_eq!(ar.missing, 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_message_with_a_line_break_or_a_quote_is_written_as_a_string() {
    let dir = std::env::temp_dir().join(format!("wf-build-api-breaks-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::create_dir_all(dir.join("src/translations")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "breaks", "build": { "output": "build" }, "i18n": { "default_locale": "en", "locales": ["en", "ar"] } }"#).unwrap();
    std::fs::write(
        dir.join("src/translations/en.json"),
        "{ \"two\\nlines\": \"one\\ntwo \\\"quoted\\\" \\\\ end\" }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/translations/ar.json"),
        "{ \"two\\nlines\": \"واحد\\nاثنان\" }\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/pages/Home.wf"), "page Home(path: \"/\", title: \"Home\", description: \"Breaks.\") {\n    Text(t(\"two\\nlines\"))\n}\n").unwrap();
    let vfs = OverlayVfs::over_disk();
    let report = build(&BuildOptions::new(&dir, &vfs))
        .unwrap_or_else(|f| panic!("{f}\n{:?}", f.report.lines));
    assert_eq!(report.errors().count(), 0, "{:?}", report.lines);
    let _ = std::fs::remove_dir_all(&dir);
}
