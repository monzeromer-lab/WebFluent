//! Every component a call can name, described in one vocabulary — and the
//! description is the checker's own: for every built-in, and every
//! component of the site and the fixtures (and of `WF_COMPONENTS_PROJECTS`,
//! Halyard say), a call written from the table alone — each required prop
//! given a value of the kind the table says, then each flag and each case
//! it marks as a flag on its own — draws nothing the checker says about a
//! prop, a flag or a type.

use std::path::{Path, PathBuf};

use webfluent::registry::schema::{self, Catalog, Component, Prop};
use webfluent::vfs::OverlayVfs;

/// A value of the kind a prop takes, as source; `None` where the table
/// alone cannot make one (a record, a state, a declaration).
fn value(p: &Prop) -> Option<String> {
    Some(match p.kind {
        "text" | "any" | "secret" => "\"x\"".into(),
        "number" => {
            let r = p.range.as_ref();
            let n = r.and_then(|r| r.min).unwrap_or(1.0);
            webfluent::print::number(n)
        }
        "bool" => "true".into(),
        "enum" => format!(".{}", p.cases.first()?.name),
        "path" => "\"/\"".into(),
        "file" => match p.file? {
            "image" | "icon" => "\"/probe.png\"".into(),
            "video" => "\"/probe.mp4\"".into(),
            "audio" => "\"/probe.mp3\"".into(),
            "captions" => "\"/probe.vtt\"".into(),
            _ => "\"/probe.pdf\"".into(),
        },
        "list" => "[]".into(),
        "map" => "{}".into(),
        "date" => "@2026-01-01".into(),
        "time" => "@09:30".into(),
        "datetime" => "@2026-01-01T09:30Z".into(),
        "duration" => "1.seconds".into(),
        "money" => "€1".into(),
        "url" => "\"https://example.com\"".into(),
        "email" => "\"a@example.com\"".into(),
        "color" => "#ffffff".into(),
        "uuid" => "\"3f2b5e2a-0000-4000-8000-000000000000\"".into(),
        _ => return None,
    })
}

/// The calls the table says are good: the required props, then each flag
/// on its own. `None` when a required prop has no value to give.
fn calls(c: &Component) -> Option<Vec<String>> {
    let mut args = Vec::new();
    for p in &c.props {
        if !(p.required || p.positional) {
            continue;
        }
        match value(p) {
            Some(v) if p.positional => args.insert(0, v),
            Some(v) => args.push(format!("{}: {v}", p.name)),
            None if p.required => return None,
            None => {}
        }
    }
    let base = if args.is_empty() {
        c.name.clone()
    } else {
        format!("{}({})", c.name, args.join(", "))
    };
    let mut out = vec![base.clone()];
    // Every prop, named, with a value of its kind.
    for p in c.props.iter().filter(|p| !p.required && !p.positional) {
        if let Some(v) = value(p) {
            let sep = if args.is_empty() { "" } else { ", " };
            out.push(format!(
                "{}({}{sep}{}: {v})",
                c.name,
                args.join(", "),
                p.name
            ));
        }
    }
    for p in &c.props {
        if p.flag {
            out.push(format!("{base}.{}", p.name));
        }
        for case in p.cases.iter().filter(|c| c.flag) {
            out.push(format!("{base}.{}", case.name));
        }
    }
    // A part is placed in its owner.
    if let Some(owner) = &c.owner {
        out = out
            .into_iter()
            .map(|call| format!("{owner} {{ {call} }}"))
            .collect();
    }
    Some(out)
}

const ABOUT_PROPS: &[&str] = &["C02", "C05", "C06", "E103", "E113", "T01"];

/// The probe's findings about props, flags and types, for every call the
/// table says is good in the project at `dir`.
fn probe(
    dir: &Path,
    catalog: &Catalog,
    which: &dyn Fn(&Component) -> bool,
) -> (usize, Vec<String>) {
    let mut lines = Vec::new();
    for c in catalog.components.iter().filter(|c| which(c)) {
        if let Some(cs) = calls(c) {
            lines.extend(cs);
        }
    }
    let page = format!(
        "page ZzProbe(path: \"/zz-probe\", title: \"Probe\", description: \"Probe.\") {{\n    Heading(\"Probe\").h1\n{}\n}}\n",
        lines
            .iter()
            .map(|l| format!("    {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let probe_file = dir.join("src/pages/ZzProbe.wf");
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(&probe_file, page.clone());
    let options = webfluent::build::BuildOptions::new(dir, &vfs);
    let report = match webfluent::build::check(&options) {
        Ok(r) => r,
        Err(f) => *f.report,
    };
    let found: Vec<String> = report
        .diagnostics
        .iter()
        .filter(|d| d.file.ends_with("ZzProbe.wf"))
        .filter(|d| ABOUT_PROPS.contains(&d.code))
        .map(|d| {
            let line = page
                .lines()
                .nth(d.line.saturating_sub(1))
                .unwrap_or("")
                .trim()
                .to_string();
            format!("{}: {} — {}", d.code, d.message, line)
        })
        .collect();
    (lines.len(), found)
}

#[test]
fn every_built_in_is_described_as_the_checker_reads_it() {
    let dir = std::env::temp_dir().join(format!("wf-components-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "probe" }"#).unwrap();
    let catalog = schema::catalog(None);
    assert_eq!(
        catalog.components.len(),
        webfluent::registry::COMPONENTS.len()
    );
    // The ones that only a deck, a document or the app holds are probed
    // where they belong; here, a page's.
    let (calls, found) = probe(&dir, &catalog, &|c| {
        !matches!(c.group.as_str(), "Slides" | "PDF")
            && !matches!(c.name.as_str(), "Router" | "Route")
    });
    let _ = std::fs::remove_dir_all(&dir);
    assert!(calls > 400, "{calls} calls");
    assert!(
        found.is_empty(),
        "{} findings about what the table says is good:\n{}",
        found.len(),
        found.join("\n")
    );
}

#[test]
fn every_project_component_is_described_as_the_checker_reads_it() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut projects: Vec<PathBuf> = vec![root.join("site")];
    for fixture in ["marketing", "dashboard", "layers", "storage", "keys"] {
        projects.push(root.join("tests/fixtures").join(fixture));
    }
    for tutorial in ["easy-todos", "medium-field-notes", "hard-pocket"] {
        projects.push(root.join("examples/tutorials").join(tutorial));
    }
    if let Ok(more) = std::env::var("WF_COMPONENTS_PROJECTS") {
        projects.extend(more.split(':').filter(|d| !d.is_empty()).map(PathBuf::from));
    }
    let mut total = 0;
    let mut wrong = Vec::new();
    for dir in projects
        .iter()
        .filter(|d| d.join("webfluent.app.json").is_file())
    {
        let catalog = schema::of_project_via(&webfluent::vfs::FsVfs, dir);
        let (calls, found) = probe(dir, &catalog, &|c| c.source == "project");
        eprintln!(
            "{}: {calls} calls, {} components",
            dir.display(),
            catalog
                .components
                .iter()
                .filter(|c| c.source == "project")
                .count()
        );
        total += calls;
        wrong.extend(found.into_iter().map(|f| format!("{}: {f}", dir.display())));
    }
    assert!(total > 50, "{total} calls");
    assert!(
        wrong.is_empty(),
        "{} findings:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

const PROJECT: &str = r#"enum Tone {
    calm
    loud
}

enum Size {
    sm
    lg
}

enum Level {
    sm
    md
}

type Todo {
    title: String
}

/// A card.
component NoteCard(
    /// What it says.
    _ title: String,
    tone: Tone = .calm,
    size: Size?,
    level: Level = .md,
    score: Number(0..=10) = 5,
    share: Number(min: 0, below: 1) = 0,
    due: Date?,
    accent: Color = #0F766E,
    price: Money?,
    items: [Todo] = [],
    todo: Todo?,
    open: Bool = false,
    note: String
) {
    /// When it opens.
    event opened(id: String)
    slot row(item: Todo)
    part Header(_ text: String) {
        Heading(text).h3
    }
    Stack {
        children
    }
}

component Plain {
    Text("x")
}
"#;

#[test]
fn a_project_component_says_what_each_prop_takes() {
    let program = webfluent::syntax::parse_source(PROJECT, "t.wf").unwrap();
    let file_of = |_| "src/Card.wf".to_string();
    let catalog = schema::catalog(Some((&program, &file_of)));
    let card = catalog.get("NoteCard").unwrap();
    assert_eq!(card.source, "project");
    assert_eq!(card.group, "This project");
    assert_eq!(card.summary.as_deref(), Some("A card."));
    assert_eq!(
        (card.file.as_deref(), card.line),
        (Some("src/Card.wf"), Some(21))
    );
    let prop = |n: &str| card.props.iter().find(|p| p.name == n).unwrap();
    // The positional one first, and documented.
    assert_eq!(card.props[0].name, "title");
    assert!(card.props[0].positional && card.props[0].required);
    assert_eq!(card.props[0].summary.as_deref(), Some("What it says."));
    // An enum: its cases, its default, which are flags.
    let tone = prop("tone");
    assert_eq!(
        (tone.kind, tone.declared.as_deref()),
        ("enum", Some("Tone"))
    );
    assert_eq!(
        tone.cases
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["calm", "loud"]
    );
    assert!(tone.cases[0].default && !tone.cases[1].default);
    assert!(tone.cases.iter().all(|c| c.flag));
    assert_eq!(tone.default.as_deref(), Some(".calm"));
    assert!(!tone.required);
    // `.sm` is a case of two enum props, so it is a flag of neither.
    assert!(!prop("size").cases[0].flag && !prop("level").cases[0].flag);
    assert!(prop("size").cases[1].flag && prop("level").cases[1].flag);
    assert!(!prop("size").required, "optional");
    // A range where the type says one.
    let score = prop("score").range.clone().unwrap();
    assert_eq!(
        (score.min, score.max, score.inclusive),
        (Some(0.0), Some(10.0), true)
    );
    let share = prop("share").range.clone().unwrap();
    assert_eq!(
        (share.min, share.max, share.inclusive),
        (Some(0.0), Some(1.0), false)
    );
    // The language's own types, a list of records, a record.
    assert_eq!(prop("due").kind, "date");
    assert_eq!(
        (prop("accent").kind, prop("accent").default.as_deref()),
        ("color", Some("#0F766E"))
    );
    assert_eq!(prop("price").kind, "money");
    let items = prop("items");
    assert_eq!(
        (items.kind, items.item, items.declared.as_deref()),
        ("list", Some("record"), Some("Todo"))
    );
    assert_eq!(
        (prop("todo").kind, prop("todo").ty.as_str()),
        ("record", "Todo?")
    );
    // A Bool is a flag; a String with no default is required.
    assert!(prop("open").flag && prop("open").kind == "bool");
    assert!(prop("note").required);
    // Its events, slots, parts and children.
    assert_eq!(card.events[0].name, "opened");
    assert_eq!(card.events[0].params[0].ty, "String");
    assert_eq!(card.events[0].summary.as_deref(), Some("When it opens."));
    let slots: Vec<&str> = card.slots.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(slots, ["row", "children"]);
    assert_eq!(card.slots[0].params[0].ty, "Todo");
    assert_eq!(card.children, "elements");
    assert_eq!(card.parts, ["NoteCard.Header"]);
    let header = catalog.get("NoteCard.Header").unwrap();
    assert_eq!(header.owner.as_deref(), Some("NoteCard"));
    assert_eq!(catalog.get("Plain").unwrap().children, "none");
    assert_eq!(card.attributes, ["global", "aria", "data"]);
    // Where a drop is legal.
    assert!(catalog.may_hold("NoteCard", "Text").is_ok());
    assert!(catalog.may_hold("NoteCard", "NoteCard.Header").is_ok());
    assert!(
        catalog
            .may_hold("Stack", "NoteCard.Header")
            .unwrap_err()
            .contains("only inside `NoteCard`")
    );
    assert!(
        catalog
            .may_hold("Plain", "Text")
            .unwrap_err()
            .contains("takes no children")
    );
    assert!(
        catalog
            .may_hold("Text", "Badge")
            .unwrap_err()
            .contains("takes no children")
    );
    assert!(
        catalog.may_hold("NoteCard", "Card.Footer").is_err(),
        "a built-in part"
    );
}

#[test]
fn a_built_in_says_what_each_prop_takes() {
    let catalog = schema::catalog(None);
    let button = catalog.get("Button").unwrap();
    assert_eq!(button.source, "builtin");
    assert_eq!(button.props[0].name, "label");
    assert!(button.props[0].positional);
    let tone = button.props.iter().find(|p| p.name == "tone").unwrap();
    assert_eq!(tone.kind, "enum");
    assert!(tone.cases.iter().any(|c| c.name == "primary" && c.flag));
    assert_eq!(button.tag, Some("button"));
    let image = catalog.get("Image").unwrap();
    let src = image.props.iter().find(|p| p.name == "src").unwrap();
    assert_eq!((src.kind, src.file), ("file", Some("image")));
    assert_eq!(image.children, "none");
    let card = catalog.get("Card").unwrap();
    assert!(card.parts.iter().any(|p| p == "Card.Header"));
    assert_eq!(
        catalog.get("Card.Header").unwrap().owner.as_deref(),
        Some("Card")
    );
    assert!(
        catalog
            .universal
            .iter()
            .any(|p| p.name == "animate" && p.kind == "enum")
    );
    assert!(catalog.dom_events.iter().any(|e| e.name == "click"));
    // `wf components --json` is this, with the project's after.
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["components", "--json", "-d"])
        .arg(std::env::temp_dir().join("wf-no-project-here"))
        .output()
        .unwrap();
    let said: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(said, serde_json::to_value(&catalog).unwrap());
}
