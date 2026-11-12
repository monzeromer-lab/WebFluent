//! A page in a language of its own (`lang:`), and the pages that translate
//! each other (`translates:`): issues #5 and #6. A bilingual site wrote
//! `<html lang="ar" dir="rtl">` on its English pages, and listed `?lang=`
//! addresses as `hreflang` alternates that canonicalised to the bare one.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A project on disk with `pages`, built with `wf build`.
fn project(name: &str, config: &str, pages: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-page-lang-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    std::fs::write(dir.join("src/App.wf"), pages).unwrap();
    dir
}

fn wf(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .arg("-d")
        .arg(dir)
        .output()
        .unwrap()
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join("build").join(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

const PAGES: &str = r#"app { Router }

page Home(path: "/", title: "الرئيسية", description: "الصفحة الأولى.") {
    Heading("مرحباً").h1
    Link("English", to: "/en")
}

page HomeEn(path: "/en", title: "Home", description: "The front page.", lang: "en", translates: Home) {
    Heading("Hello").h1
    Link("العربية", to: "/")
}
"#;

const ARABIC: &str = r#"{ "name": "Site", "build": { "output": "build", "ssg": true, "minify": false },
  "meta": { "lang": "ar", "site_url": "https://site.example" } }"#;

#[test]
fn each_page_is_written_in_its_own_language_with_its_translations_as_alternates() {
    let dir = project("ssg", ARABIC, PAGES);
    let out = wf(&dir, &["build"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let ar = read(&dir, "index.html");
    let en = read(&dir, "en/index.html");
    assert!(ar.contains(r#"<html lang="ar" dir="rtl">"#), "{ar}");
    assert!(en.contains(r#"<html lang="en">"#), "{en}");
    assert!(
        en.contains(r#"<meta property="og:locale" content="en_US">"#),
        "{en}"
    );
    assert!(
        ar.contains(r#"<meta property="og:locale" content="ar_AR">"#),
        "{ar}"
    );

    for html in [&ar, &en] {
        assert!(
            html.contains(r#"<link rel="alternate" hreflang="ar" href="https://site.example/">"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<link rel="alternate" hreflang="en" href="https://site.example/en">"#),
            "{html}"
        );
        assert!(
            html.contains(
                r#"<link rel="alternate" hreflang="x-default" href="https://site.example/">"#
            ),
            "{html}"
        );
        assert!(!html.contains("?lang="), "{html}");
    }
    // Each canonical to itself, so the alternates agree with it.
    assert!(
        en.contains(r#"<link rel="canonical" href="https://site.example/en">"#),
        "{en}"
    );

    // The router puts the page's language on the document when it shows,
    // and the site's back on a page without one.
    let js = read(&dir, "app.js");
    assert!(
        js.contains(r#"lang: "en", dir: "ltr""#),
        "the route says its language"
    );
    assert!(
        js.contains(r#"lang: "ar", dir: "rtl""#),
        "the router knows the site's"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_single_page_app_opens_in_the_entry_pages_language() {
    let dir = project(
        "spa",
        r#"{ "name": "Site", "build": { "output": "build" }, "meta": { "lang": "ar" } }"#,
        &PAGES.replace(
            r#"page HomeEn(path: "/en""#,
            r#"page Spare(path: "/x", title: "X", description: "X.", lang: "fr") { Heading("X").h1 }

page HomeEn(path: "/en""#,
        ),
    );
    let out = wf(&dir, &["build"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let shell = read(&dir, "index.html");
    assert!(shell.contains(r#"<html lang="ar" dir="rtl">"#), "{shell}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_site_with_no_page_of_its_own_language_is_unchanged() {
    let dir = project(
        "plain",
        r#"{ "name": "Site", "build": { "output": "build", "ssg": true, "minify": false }, "meta": { "lang": "ar" } }"#,
        r#"app { Router }
page Home(path: "/", title: "H", description: "D.") { Heading("H").h1 }
page About(path: "/about", title: "A", description: "D.") { Heading("A").h1 }
"#,
    );
    let out = wf(&dir, &["build"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let js = read(&dir, "app.js");
    assert!(
        !js.contains("dir: \""),
        "no page has a language of its own: {js}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_route_parameter_named_lang_is_still_one() {
    let dir = project(
        "param",
        r#"{ "name": "Site", "build": { "output": "build" } }"#,
        r#"page Docs(path: "/:lang/docs", title: "Docs", description: "D.", lang: String) {
    Heading("Docs {lang}").h1
}
"#,
    );
    let out = wf(&dir, &["check"]);
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_language_that_is_not_a_tag_is_refused_where_it_is_written() {
    let dir = project(
        "tag",
        r#"{ "name": "Site", "build": { "output": "build" } }"#,
        r#"page Home(path: "/", title: "H", description: "D.", lang: "English") { Heading("H").h1 }
"#,
    );
    let out = wf(&dir, &["check"]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("is not a language tag"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_translation_of_no_page_or_two_in_one_language_is_r05() {
    for (name, pages, said) in [
        (
            "missing",
            PAGES.replace("translates: Home", "translates: Hom"),
            "no page is named `Hom`",
        ),
        (
            "same",
            PAGES.replace(r#"lang: "en", "#, ""),
            "both in the site's language",
        ),
    ] {
        let dir = project(name, ARABIC, &pages);
        let out = wf(&dir, &["check"]);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!out.status.success(), "{name}: {text}");
        assert!(text.contains("R05"), "{name}: {text}");
        assert!(text.contains(said), "{name}: {text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn the_formatter_keeps_both_options() {
    let dir = project("fmt", ARABIC, PAGES);
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["fmt", "--stdout"])
        .arg(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains(r#"lang: "en""#), "{text}");
    assert!(text.contains("translates: Home"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_markdown_pages_language_is_held_to_a_tag_too() {
    let dir = project("md", ARABIC, PAGES);
    std::fs::write(
        dir.join("src/about.md"),
        "---\npath: /about\ntitle: About\ndescription: About us.\nlang: English\ntranslates: Home\n---\n# About\n",
    )
    .unwrap();
    let out = wf(&dir, &["check"]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("is not a language tag"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}
