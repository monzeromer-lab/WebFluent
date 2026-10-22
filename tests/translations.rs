//! The translations as a table, both ways, and the plural rules under it:
//! the rules pick what the browser's own `Intl.PluralRules` picks for every
//! locale CLDR has, at every count; every project's tables read into keys
//! whose cells are exactly the files' messages, at their lines, with the
//! missing, the unused and the unknown counted as the build reports them;
//! a message written back changes its line and nothing else; and a `t()`
//! binding reads as the inspector shows it. What a real browser paints,
//! before and after its script, is `tests/browser/plurals.mjs`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use webfluent::i18n::plurals::{self, CATEGORIES};
use webfluent::i18n::table::{
    self, Table, call, call_text, remove_message, set_message, table_via,
};

#[test]
fn the_rules_pick_what_the_browser_picks() {
    // Node's ICU is the browser's; when it carries the same CLDR release,
    // every locale and every count agrees.
    let Ok(out) = std::process::Command::new("node")
        .args(["-p", "process.versions.cldr"])
        .output()
    else {
        eprintln!("no node: skipped");
        return;
    };
    let theirs = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let ours = webfluent::i18n::plurals_data::CLDR_VERSION;
    if !theirs.starts_with(ours) {
        eprintln!("node has CLDR {theirs}, the table {ours}: skipped");
        return;
    }
    let locales: Vec<&str> = webfluent::i18n::plurals_data::RULES
        .iter()
        .map(|(l, _)| *l)
        .filter(|l| *l != "root")
        .collect();
    let mut counts: Vec<f64> = (0..=230).map(f64::from).collect();
    counts.extend([
        1000.0,
        10_000.0,
        100_000.0,
        1_000_000.0,
        2_000_000.0,
        0.5,
        1.5,
        2.5,
        10.1,
        0.25,
    ]);
    let script = format!(
        "const counts = {};\nconst out = {{}};\nfor (const l of {}) {{ try {{ const r = new Intl.PluralRules(l); if (r.resolvedOptions().locale.split('-')[0] !== l.split('-')[0]) continue; out[l] = counts.map((n) => r.select(n)); }} catch (e) {{}} }}\nconsole.log(JSON.stringify(out));",
        serde_json::to_string(&counts).unwrap(),
        serde_json::to_string(&locales).unwrap()
    );
    let out = std::process::Command::new("node")
        .args(["-e", &script])
        .output()
        .unwrap();
    let icu: HashMap<String, Vec<String>> = serde_json::from_slice(&out.stdout).unwrap();
    assert!(icu.len() > 150, "{} locales from ICU", icu.len());
    let mut wrong = Vec::new();
    for (locale, picks) in &icu {
        for (n, want) in counts.iter().zip(picks) {
            let got = plurals::select_number(locale, *n);
            if got != want {
                wrong.push(format!("{locale} {n}: {got}, ICU {want}"));
            }
        }
        let mut forms = plurals::categories(locale);
        forms.sort();
        let mut used: Vec<&str> = picks.iter().map(String::as_str).collect();
        used.sort();
        used.dedup();
        for u in used {
            assert!(
                forms.contains(&u),
                "{locale} picks {u}, which its forms do not list"
            );
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong
            .iter()
            .take(30)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn projects() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = vec![root.join("site")];
    for base in [
        "examples/tutorials",
        "examples/decks",
        "examples/documents",
        "tests/fixtures",
    ] {
        if let Ok(entries) = std::fs::read_dir(root.join(base)) {
            let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            dirs.sort();
            out.extend(
                dirs.into_iter()
                    .filter(|d| d.join("webfluent.app.json").is_file()),
            );
        }
    }
    if let Ok(more) = std::env::var("WF_I18N_PROJECTS") {
        out.extend(more.split(':').filter(|d| !d.is_empty()).map(PathBuf::from));
    }
    out
}

fn read(dir: &Path) -> Option<(Table, webfluent::config::project::I18nConfig)> {
    let config = webfluent::config::ProjectConfig::load(dir).ok()?;
    let i18n = config.i18n.clone()?;
    Some((
        table_via(&webfluent::vfs::FsVfs, dir).unwrap().unwrap(),
        i18n,
    ))
}

#[test]
fn every_table_reads_as_its_files_and_counts_as_the_build_does() {
    let mut tables = 0;
    let mut wrong = Vec::new();
    for dir in projects() {
        let Some((table, i18n)) = read(&dir) else {
            continue;
        };
        tables += 1;
        for locale in &table.locales {
            let path = dir.join(&i18n.dir).join(format!("{}.json", locale.code));
            let Ok(text) = std::fs::read_to_string(&path) else {
                assert!(!locale.exists);
                continue;
            };
            let file: HashMap<String, String> = serde_json::from_str(&text).unwrap();
            // Every message of the file is one cell, its text the file's,
            // at its line.
            let cells: Vec<&table::Cell> = table
                .keys
                .iter()
                .flat_map(|k| k.cells.get(&locale.code).into_iter().flatten())
                .filter(|c| c.text.is_some())
                .collect();
            let mut entries: Vec<&str> = cells.iter().map(|c| c.entry.as_str()).collect();
            entries.sort();
            let mut want: Vec<&str> = file.keys().map(String::as_str).collect();
            want.sort();
            if entries != want {
                wrong.push(format!(
                    "{}: the cells are not the file's messages",
                    path.display()
                ));
            }
            for c in &cells {
                if file.get(&c.entry) != c.text.as_ref() {
                    wrong.push(format!(
                        "{}: {} reads as {:?}",
                        path.display(),
                        c.entry,
                        c.text
                    ));
                }
                let line = text
                    .lines()
                    .nth(c.line.unwrap_or(0).saturating_sub(1) as usize)
                    .unwrap_or("");
                if !line.contains(&serde_json::to_string(&c.entry).unwrap()) {
                    wrong.push(format!(
                        "{}: {} is not on line {:?}",
                        path.display(),
                        c.entry,
                        c.line
                    ));
                }
            }
            // Each locale's forms are its plural rules'.
            assert_eq!(locale.forms, plurals::categories(&locale.code));
            assert_eq!(locale.rtl, table::is_rtl(&locale.code));
            // Set to its own value, each message moves no byte.
            for c in cells.iter().take(40) {
                let same = set_message(&text, &c.entry, c.text.as_deref().unwrap()).unwrap();
                if same != text {
                    wrong.push(format!(
                        "{}: {} set to itself moves bytes",
                        path.display(),
                        c.entry
                    ));
                }
            }
            // Changed, it changes its line alone.
            if let Some(c) = cells.first() {
                let out = set_message(&text, &c.entry, "Changed \"quoted\" {name}").unwrap();
                let changed: Vec<usize> = text
                    .lines()
                    .zip(out.lines())
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .map(|(i, _)| i + 1)
                    .collect();
                assert_eq!(changed, [c.line.unwrap() as usize], "{}", path.display());
                let back: HashMap<String, String> = serde_json::from_str(&out).unwrap();
                assert_eq!(back[&c.entry], "Changed \"quoted\" {name}");
                assert_eq!(back.len(), file.len());
            }
        }
        // What the build says is missing, unknown and mismatched is what
        // the table says.
        let options = webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs);
        let report = match webfluent::build::check(&options) {
            Ok(r) => r,
            Err(f) => *f.report,
        };
        let i01 = report
            .diagnostics
            .iter()
            .filter(|d| d.code == "I01")
            .count();
        if i01 != table.unknown.len() {
            wrong.push(format!(
                "{}: the build reports {i01} I01, the table {} unknown",
                dir.display(),
                table.unknown.len()
            ));
        }
        // An unused key is named nowhere in the sources.
        for k in table.keys.iter().filter(|k| k.unused) {
            let named = walk_sources(&dir)
                .iter()
                .any(|s| s.contains(&format!("\"{}\"", k.key)));
            if named {
                wrong.push(format!(
                    "{}: {} is called unused, but a source names it",
                    dir.display(),
                    k.key
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(tables >= 3, "{tables} tables");
}

fn walk_sources(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.join("src")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                if !p.ends_with("translations") {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "wf" || x == "wfx")
                && let Ok(t) = std::fs::read_to_string(&p)
            {
                out.push(t);
            }
        }
    }
    out
}

fn project(files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "wf-translations-{}-{}",
        std::process::id(),
        files.iter().map(|(_, t)| t.len()).sum::<usize>()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    dir
}

const PAGE: &str = r#"component Title(text: String = t("inbox.title")) {
    Text(text)
}

page Inbox(path: "/", title: "Inbox", description: "The inbox.") {
    state name = ""
    state count = 0
    state status = "open"
    validate name {
        required
    }
    Heading(t("inbox.heading", { name: name })).h1
    Text(t("files", { count: count }))
    Text(t("status.{status}"))
    Text(t("nowhere"))
    Input(bind: name, label: t("inbox.name"))
}
"#;

const EN: &str = r#"{
  "inbox.title": "Inbox",
  "inbox.heading": "Hello, {name}",
  "inbox.name": "Name",
  "files.zero": "No files",
  "files.one": "{count} file",
  "files.other": "{count} files",
  "status.open": "Open",
  "status.closed": "Closed",
  "form.required": "Fill this in",
  "old.banner": "Gone"
}
"#;

const AR: &str = r#"{
  "inbox.title": "الوارد",
  "inbox.heading": "أهلاً {user}",
  "files.zero": "لا ملفات",
  "files.one": "ملف واحد",
  "files.other": "{count} ملف",
  "status.open": "مفتوح"
}
"#;

#[test]
fn a_project_s_table_is_each_thing_it_should_be() {
    let dir = project(&[
        (
            "webfluent.app.json",
            r#"{ "name": "t", "i18n": { "default_locale": "en", "locales": ["en", "ar", "fr"], "dir": "src/translations" } }"#,
        ),
        ("src/pages/Inbox.wf", PAGE),
        ("src/translations/en.json", EN),
        ("src/translations/ar.json", AR),
    ]);
    let t = table_via(&webfluent::vfs::FsVfs, &dir).unwrap().unwrap();
    let key = |k: &str| {
        t.keys
            .iter()
            .find(|x| x.key == k)
            .unwrap_or_else(|| panic!("no key {k}"))
    };
    let loc = |c: &str| t.locales.iter().find(|l| l.code == c).unwrap();
    assert_eq!(t.default_locale, "en");
    assert!(loc("en").default && !loc("ar").default);
    assert!(loc("ar").rtl && !loc("fr").rtl);
    assert!(!loc("fr").exists, "fr.json is not there");
    assert_eq!(
        loc("ar").forms,
        ["zero", "one", "two", "few", "many", "other"]
    );

    // A plural key is one key, with the forms the locale's rules use.
    let files = key("files");
    assert!(files.plural);
    let ar: Vec<(&str, bool, bool)> = files.cells["ar"]
        .iter()
        .map(|c| (c.form.unwrap(), c.text.is_some(), c.needed))
        .collect();
    assert_eq!(
        ar,
        [
            ("zero", true, true),
            ("one", true, true),
            ("two", false, true),
            ("few", false, true),
            ("many", false, true),
            ("other", true, true)
        ]
    );
    let en: Vec<(&str, bool)> = files.cells["en"]
        .iter()
        .map(|c| (c.form.unwrap(), c.needed))
        .collect();
    assert_eq!(
        en,
        [("zero", false), ("one", true), ("other", true)],
        "English's zero is the file's, not its rules'"
    );
    assert_eq!(
        loc("ar").forms_missing,
        1,
        "files lacks ar's two, few and many"
    );
    assert_eq!(files.uses.len(), 1);

    // Missing: per locale, and fr has nothing.
    assert_eq!(key("inbox.name").missing, ["ar", "fr"]);
    assert_eq!(loc("fr").missing, t.keys.len());
    // Placeholders that differ from the default's are flagged.
    let heading = &key("inbox.heading").cells["ar"][0];
    assert!(heading.differs && heading.placeholders == ["user"]);
    assert_eq!(loc("ar").mismatched, 1);
    // Plural forms may spell the number: ar's `one` has no {count}.
    assert!(!files.cells["ar"].iter().any(|c| c.differs));

    // Used, implied, maybe, unused, unknown.
    assert_eq!(
        key("inbox.title").uses.len(),
        1,
        "a prop's default is a use"
    );
    assert_eq!(
        key("form.required").implied_by,
        Some("a validate rule's own message")
    );
    assert!(
        key("status.closed").maybe_used && !key("status.closed").unused,
        "t(\"status.{{status}}\") may be it"
    );
    assert!(key("old.banner").unused);
    assert_eq!(
        t.unknown.iter().map(|u| u.key.as_str()).collect::<Vec<_>>(),
        ["nowhere"]
    );

    // The inspector's binding.
    let program = webfluent::parse_source(PAGE, "Inbox.wf").unwrap();
    let mut calls = Vec::new();
    fn find(e: &webfluent::parser::ast::Expr, out: &mut Vec<webfluent::parser::ast::Expr>) {
        if let webfluent::parser::ast::Expr::FunctionCall(n, _) = e
            && n == "t"
        {
            out.push(e.clone());
        }
        for c in e.children() {
            find(c, out);
        }
    }
    let p = program
        .declarations
        .iter()
        .find_map(|d| match d {
            webfluent::parser::ast::Declaration::Page(p) => Some(p),
            _ => None,
        })
        .unwrap();
    for s in &p.body {
        if let webfluent::parser::ast::StatementKind::UIElement(el) = &s.kind {
            for a in &el.args {
                let (webfluent::parser::ast::Arg::Positional(e)
                | webfluent::parser::ast::Arg::Named(_, e)) = a;
                find(e, &mut calls);
            }
        }
    }
    let heading = call(&calls[0], &t).unwrap();
    assert_eq!(
        (heading.key.as_str(), heading.message.as_deref()),
        ("inbox.heading", Some("Hello, {name}"))
    );
    assert_eq!(heading.args, [("name".to_string(), "name".to_string())]);
    assert_eq!(heading.placeholders, ["name"]);
    let files_call = call(&calls[1], &t).unwrap();
    assert!(files_call.plural && files_call.message.as_deref() == Some("{count} files"));
    let nowhere = calls
        .iter()
        .find_map(|c| call(c, &t).filter(|x| x.key == "nowhere"))
        .unwrap();
    assert!(nowhere.missing);
    assert_eq!(
        call_text("inbox.heading", &[("name".into(), "user.name".into())]),
        "t(\"inbox.heading\", { name: user.name })"
    );
    assert_eq!(call_text("inbox.title", &[]), "t(\"inbox.title\")");

    // The build agrees: I01 for the unknown key, I02 where the call and
    // the message differ, I03 for what a locale lacks.
    let options = webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs);
    let report = match webfluent::build::check(&options) {
        Ok(r) => r,
        Err(f) => *f.report,
    };
    let codes: Vec<&str> = report
        .diagnostics
        .iter()
        .map(|d| d.code)
        .filter(|c| c.starts_with('I'))
        .collect();
    assert_eq!(
        codes.iter().filter(|c| **c == "I01").count(),
        1,
        "{codes:?}"
    );
    let i03 = report
        .diagnostics
        .iter()
        .find(|d| d.code == "I03" && d.message.contains("`ar.json`"))
        .unwrap();
    assert!(
        i03.message.contains("sees the `en` message"),
        "a reader sees the default locale's: {}",
        i03.message
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_key_with_forms_and_a_message_of_its_own_shows_both() {
    let dir = project(&[
        (
            "webfluent.app.json",
            r#"{ "name": "b", "i18n": { "default_locale": "en", "locales": ["en"] } }"#,
        ),
        (
            "src/pages/P.wf",
            "page P(path: \"/\", title: \"P\", description: \"P.\") {\n    Heading(t(\"items\", { count: 2 })).h1\n}\n",
        ),
        (
            "src/translations/en.json",
            r#"{ "items": "Items", "items.one": "One item", "items.other": "{count} items" }"#,
        ),
    ]);
    let t = table_via(&webfluent::vfs::FsVfs, &dir).unwrap().unwrap();
    assert_eq!(t.keys.len(), 1);
    let entries: Vec<&str> = t.keys[0].cells["en"]
        .iter()
        .map(|c| c.entry.as_str())
        .collect();
    assert_eq!(entries, ["items", "items.one", "items.other"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_message_is_written_into_its_file_and_out_again() {
    let added = set_message(EN, "files.many", "{count} files, many").unwrap();
    let back: HashMap<String, String> = serde_json::from_str(&added).unwrap();
    assert_eq!(back["files.many"], "{count} files, many");
    assert_eq!(
        added.lines().count(),
        EN.lines().count() + 1,
        "one line added"
    );
    assert!(
        added.starts_with(&EN[..EN.rfind("\"old.banner\"").unwrap()]),
        "everything before it as it was"
    );
    let removed = remove_message(&added, "files.many").unwrap();
    assert_eq!(removed, EN, "taken out again: the file it was");
    // A locale with no file yet gets one.
    let made = set_message("", "inbox.title", "Boîte").unwrap();
    assert_eq!(made, "{\n  \"inbox.title\": \"Boîte\"\n}\n");
    // An unsaved table is the one read.
    let dir = project(&[
        (
            "webfluent.app.json",
            r#"{ "name": "u", "i18n": { "default_locale": "en", "locales": ["en", "ar"] } }"#,
        ),
        ("src/pages/Inbox.wf", PAGE),
        ("src/translations/en.json", EN),
        ("src/translations/ar.json", AR),
    ]);
    let file = dir.join("src/translations/ar.json");
    let mut vfs = webfluent::vfs::OverlayVfs::over_disk();
    vfs.set(&file, set_message(AR, "inbox.name", "الاسم").unwrap());
    let t = table_via(&vfs, &dir).unwrap().unwrap();
    let name = t.keys.iter().find(|k| k.key == "inbox.name").unwrap();
    assert!(name.missing.is_empty(), "{:?}", name.missing);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), AR);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_build_paints_the_locale_s_form() {
    // An Arabic site pre-renders each count in the form Arabic's rules
    // give it, and exactly none as `zero`.
    let mut page = String::from(
        "page Counts(path: \"/\", title: \"Counts\", description: \"Counts.\") {\n    Heading(\"Counts\").h1\n",
    );
    for n in [0, 1, 2, 5, 11, 100] {
        page.push_str(&format!(
            "    Text(t(\"files\", {{ count: {n} }}), id: \"n{n}\")\n"
        ));
    }
    page.push_str("}\n");
    let ar = r#"{ "files.zero": "zero", "files.one": "one", "files.two": "two", "files.few": "few {count}", "files.many": "many {count}", "files.other": "other {count}" }"#;
    let dir = project(&[
        (
            "webfluent.app.json",
            r#"{ "name": "c", "build": { "ssg": true }, "i18n": { "default_locale": "ar", "locales": ["ar"] } }"#,
        ),
        ("src/pages/Counts.wf", &page),
        ("src/translations/ar.json", ar),
    ]);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    for (n, want) in [
        (0, "zero"),
        (1, "one"),
        (2, "two"),
        (5, "few 5"),
        (11, "many 11"),
        (100, "other 100"),
    ] {
        assert!(
            html.contains(&format!("id=\"n{n}\">{want}<")),
            "{n} paints {want}: {html}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_runtime_writes_right_to_left_for_the_same_languages() {
    let js = include_str!("../src/runtime/modules/i18n.js");
    let line = js
        .lines()
        .find(|l| l.contains("const RTL_LOCALES"))
        .unwrap();
    let list: Vec<String> = line[line.find('[').unwrap()..=line.find(']').unwrap()]
        .trim_matches(['[', ']'])
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .collect();
    assert_eq!(list, table::RTL);
    for (l, rtl) in [
        ("ar-EG", true),
        ("he", true),
        ("ckb", true),
        ("ku", false),
        ("en-GB", false),
        ("ur_PK", true),
    ] {
        assert_eq!(table::is_rtl(l), rtl, "{l}");
        assert_eq!(webfluent::template::is_rtl(l), rtl, "{l}");
    }
    let _ = CATEGORIES;
}

#[test]
fn a_page_is_in_the_default_locale_s_language_and_direction() {
    let page = "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    Heading(t(\"hi\")).h1\n}\n";
    for (ssg, config, lang, rtl) in [
        (
            true,
            r#"{ "name": "a", "build": { "ssg": true }, "i18n": { "default_locale": "ar", "locales": ["ar", "en"] } }"#,
            "ar",
            true,
        ),
        (
            false,
            r#"{ "name": "b", "build": { "ssg": false }, "i18n": { "default_locale": "ar", "locales": ["ar", "en"] } }"#,
            "ar",
            true,
        ),
        (
            true,
            r#"{ "name": "c", "build": { "ssg": true }, "meta": { "lang": "en-GB" }, "i18n": { "default_locale": "en", "locales": ["en", "ar"] } }"#,
            "en-GB",
            false,
        ),
        (
            true,
            r#"{ "name": "d", "build": { "ssg": true }, "meta": { "lang": "he" } }"#,
            "he",
            true,
        ),
    ] {
        let dir = project(&[
            ("webfluent.app.json", config),
            ("src/pages/Home.wf", page),
            ("src/translations/ar.json", r#"{ "hi": "أهلاً" }"#),
            ("src/translations/en.json", r#"{ "hi": "Hello" }"#),
        ]);
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
            .args(["build", "-d"])
            .arg(&dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
        let want = if rtl {
            format!("<html lang=\"{lang}\" dir=\"rtl\">")
        } else {
            format!("<html lang=\"{lang}\">")
        };
        assert!(
            html.contains(&want),
            "{config}: {}",
            &html[..html.find("<head>").unwrap_or(80)]
        );
        let _ = ssg;
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// A test that looks renders as the page does: a `t()` with a `count`
/// takes the default locale's plural form, exactly none is `zero`, and a
/// key combination is written as the build writes it.
#[test]
fn a_test_that_looks_says_what_the_page_says() {
    let ar = r#"{ "files.zero": "لا ملفات", "files.one": "ملف واحد", "files.two": "ملفان", "files.few": "{count} ملفات", "files.many": "{count} ملفًا", "files.other": "{count} ملف" }"#;
    let tests = r#"
test "each count takes Arabic's form" {
    Text(t("files", { count: 0 }))
    Text(t("files", { count: 2 }))
    Text(t("files", { count: 5 }))
    Text(t("files", { count: 11 }))
    Text(t("files", { count: 100 }))
    expect "لا ملفات"
    expect "ملفان"
    expect "5 ملفات"
    expect "11 ملفًا"
    expect "100 ملف"
}

test "a key is written as the build writes it" {
    Text(keyLabel("mod+shift+p"))
    expect "Ctrl+Shift+P"
}
"#;
    let dir = project(&[
        (
            "webfluent.app.json",
            r#"{ "name": "lt", "i18n": { "default_locale": "ar", "locales": ["ar"] } }"#,
        ),
        (
            "src/pages/Home.wf",
            "page Home(path: \"/\", title: \"H\", description: \"H.\") {\n    Heading(\"H\").h1\n}\n",
        ),
        ("src/translations/ar.json", ar),
        ("tests/look.wf", tests),
    ]);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("test")
        .arg(&dir)
        .output()
        .unwrap();
    let said =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{said}");
    assert!(said.contains("2 passed"), "{said}");
    let _ = std::fs::remove_dir_all(&dir);
}
