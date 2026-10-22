//! The agent's change set and context.
//!
//! A change set is built before anyone reads it and the tree is untouched
//! until it is accepted — whole, by file, or by hunk; the files it may
//! touch are its to refuse; a private `env` value is never read through
//! it, and survives a write that shows it withheld. Its hunks, over every
//! `.wf` file of the repository, give the new text when all are accepted,
//! the old when none are, and what `patch` makes of its unified diff. The
//! context names everything it gives and withholds every private value,
//! and its guide's anchors are the site's.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use webfluent::agent::changeset::{ChangeSet, FileKind, WITHHELD};
use webfluent::agent::context::{Depth, anchor, chapters, context_via, guide};
use webfluent::agent::diff::{self, LineKind};
use webfluent::vfs::{FsVfs, OverlayVfs};

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            if e.file_name() != "build" {
                copy(&p, &to.join(e.file_name()));
            }
        } else {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// Every file under `dir` and its bytes: what "untouched" is checked by.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                if e.file_name() != "build" && e.file_name() != ".wf-cache" {
                    stack.push(p);
                }
            } else {
                out.insert(p.clone(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

/// A copy of a fixture of its own for each test, which run at once.
fn fixture(name: &str, test: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!("wf-agent-{name}-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy(&root.join("tests/fixtures").join(name), &dir);
    dir
}

#[test]
fn a_change_set_is_built_before_it_is_read_and_the_tree_is_untouched() {
    let dir = fixture("dashboard", "t1");
    let before = snapshot(&dir);
    let mut set = ChangeSet::new(Arc::new(FsVfs), &dir);

    // Read two files, change one, add a component, delete a test.
    let page = set.read("src/pages/Overview.wf").unwrap();
    let _ = set.read("src/theme.wf").unwrap();
    let changed = page.replacen("Heading(", "Text(\"Welcome back\")\n    Heading(", 1);
    assert_ne!(changed, page, "the fixture's page has a heading");
    set.write("src/pages/Overview.wf", &changed).unwrap();
    set.write(
        "src/components/Empty.wf",
        "/// Nothing here yet.\ncomponent Empty {\n    Text(\"Nothing yet\").muted\n}\n",
    )
    .unwrap();
    set.delete("tests/incidents.wf").unwrap();

    let files = set.files();
    let kinds: Vec<(&str, FileKind)> = files.iter().map(|f| (f.path.as_str(), f.kind)).collect();
    assert_eq!(
        kinds,
        [
            ("src/components/Empty.wf", FileKind::Added),
            ("src/pages/Overview.wf", FileKind::Modified),
            ("tests/incidents.wf", FileKind::Deleted),
        ]
    );
    assert!(
        files[2].outside_src && !files[0].outside_src,
        "a write outside src/ is marked for a yes"
    );
    assert_eq!(
        set.left_alone(),
        ["src/theme.wf"],
        "what it read and left alone is listed"
    );

    let checked = set.check();
    assert!(checked.builds, "{:#?}", checked.owned);
    let heading = set.heading(&checked);
    assert!(heading.starts_with("3 files · +"), "{heading}");
    assert!(heading.contains(" · builds · "), "{heading}");
    assert_eq!(
        snapshot(&dir),
        before,
        "nothing is written until it is accepted"
    );

    // What accepting writes: the whole set, a file, a hunk.
    let all = set.accept_all();
    assert_eq!(all.len(), 3);
    assert_eq!(
        set.accepted("tests/incidents.wf", None),
        Some(None),
        "a delete"
    );
    assert_eq!(
        set.accepted("src/pages/Overview.wf", None),
        Some(Some(changed.clone()))
    );
    let hunks = &files[1].hunks;
    assert_eq!(
        set.accepted("src/pages/Overview.wf", Some(&[])),
        Some(Some(page.clone())),
        "no hunk is the file as it was"
    );
    let ids: Vec<usize> = hunks.iter().map(|h| h.id).collect();
    assert_eq!(
        set.accepted("src/pages/Overview.wf", Some(&ids)),
        Some(Some(changed.clone()))
    );
    assert_eq!(
        set.accepted("src/App.wf", None),
        None,
        "a file it does not change"
    );

    // A set that breaks the build says so, and owns only what it broke.
    set.write(
        "src/components/Broken.wf",
        "component Broken {\n    Text(missingName)\n}\n",
    )
    .unwrap();
    let broken = set.check();
    assert!(!broken.builds);
    let errors: Vec<_> = broken
        .owned
        .iter()
        .filter(|d| d.severity == webfluent::diagnostics::Severity::Error)
        .collect();
    assert!(
        !errors.is_empty() && errors.iter().all(|d| d.file.contains("Broken")),
        "{:#?}",
        broken.owned
    );
    // Everything it owns is about a file it wrote: `Empty` is placed nowhere.
    assert!(
        broken
            .owned
            .iter()
            .all(|d| d.file.contains("Broken") || d.file.contains("Empty")),
        "{:#?}",
        broken.owned
    );
    assert!(set.heading(&broken).contains("does not build"));
    // Rejecting the file takes it out.
    assert!(set.revert("src/components/Broken.wf"));
    assert!(set.check().builds);
    // Writing a file back as it was is no change.
    set.write("src/pages/Overview.wf", &page).unwrap();
    assert!(
        !set.files()
            .iter()
            .any(|f| f.path == "src/pages/Overview.wf")
    );
    assert_eq!(snapshot(&dir), before);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_set_over_the_editor_s_buffers_sees_them_and_fixes_count() {
    let dir = fixture("dashboard", "t2");
    // The editor holds an unsaved break; the agent's set fixes it.
    let file = dir.join("src/pages/Settings.wf");
    let saved = std::fs::read_to_string(&file).unwrap();
    let broken = format!("{saved}\ncomponent Undone {{\n    Text(undefinedThing)\n}}\n");
    let mut buffers = OverlayVfs::over_disk();
    buffers.set(&file, broken.clone());
    let mut set = ChangeSet::new(Arc::new(buffers), &dir);
    assert_eq!(
        set.read("src/pages/Settings.wf").unwrap(),
        broken,
        "the set reads the editor's buffer"
    );
    set.write("src/pages/Settings.wf", &saved).unwrap();
    let checked = set.check();
    assert!(checked.builds);
    assert!(
        checked.before > checked.after && checked.fixed >= 1,
        "{checked:?}"
    );
    assert!(checked.owned.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_a_set_may_touch_is_its_to_refuse() {
    let dir = fixture("dashboard", "t3");
    std::fs::write(dir.join(".env"), "SECRET=1\n").unwrap();
    let mut set = ChangeSet::new(Arc::new(FsVfs), &dir);
    for path in [
        "../outside.wf",
        "/etc/passwd",
        ".git/config",
        "build/index.html",
        ".env",
        "src/.env.local",
        "node_modules/x.js",
        "",
    ] {
        assert!(set.write(path, "x").is_err(), "{path} written");
        assert!(set.read(path).is_err(), "{path} read");
    }
    assert!(set.delete("webfluent.app.json").is_err());
    assert!(set.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_private_value_is_never_read_and_survives_a_write() {
    let dir = fixture("dashboard", "t4");
    let config_path = dir.join("webfluent.app.json");
    let mut config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    config["env"] = serde_json::json!({ "STRIPE_SECRET": "sk_live_do_not_leak", "PUBLIC_REGION": "eu", "API_BASE": "https://api.example.com" });
    config["public_env"] = serde_json::json!(["API_BASE"]);
    let text = serde_json::to_string_pretty(&config).unwrap() + "\n";
    std::fs::write(&config_path, &text).unwrap();

    let mut set = ChangeSet::new(Arc::new(FsVfs), &dir);
    let read = set.read("webfluent.app.json").unwrap();
    assert!(!read.contains("sk_live_do_not_leak"), "{read}");
    assert!(
        read.contains(WITHHELD) && read.contains("\"eu\"") && read.contains("api.example.com"),
        "the public ones are shown: {read}"
    );

    // The agent edits the config it was shown; the secret is kept.
    let edited = read.replace("\"eu\"", "\"us\"");
    set.write("webfluent.app.json", &edited).unwrap();
    let written = set.accepted("webfluent.app.json", None).unwrap().unwrap();
    assert!(
        written.contains("sk_live_do_not_leak") && written.contains("\"us\""),
        "{written}"
    );
    assert!(!written.contains(WITHHELD));
    // Leaving the secret out of its write does not remove it either.
    let mut without: serde_json::Value = serde_json::from_str(&edited).unwrap();
    without["env"]
        .as_object_mut()
        .unwrap()
        .remove("STRIPE_SECRET");
    set.write(
        "webfluent.app.json",
        &serde_json::to_string_pretty(&without).unwrap(),
    )
    .unwrap();
    assert!(
        set.accepted("webfluent.app.json", None)
            .unwrap()
            .unwrap()
            .contains("sk_live_do_not_leak")
    );

    // The context withholds it too, everywhere, and says so.
    let context = context_via(
        &FsVfs,
        &dir,
        Some(("webfluent.app.json", None)),
        guide(),
        Depth::Index,
    )
    .unwrap();
    let json = serde_json::to_string(&context).unwrap();
    assert!(
        !json.contains("sk_live_do_not_leak"),
        "the context carries the secret"
    );
    assert!(!context.to_markdown().contains("sk_live_do_not_leak"));
    assert!(json.contains("PUBLIC_REGION") && json.contains("\"eu\""));
    assert!(
        context
            .withheld
            .iter()
            .any(|w| w.contains("STRIPE_SECRET") && !w.contains("sk_live")),
        "{:?}",
        context.withheld
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_context_names_what_it_gives() {
    let dir = fixture("dashboard", "t5");
    std::fs::write(dir.join(".env.local"), "X=1\n").unwrap();
    let page = std::fs::read_to_string(dir.join("src/pages/Overview.wf")).unwrap();
    let at = page.find("Heading").unwrap();
    let context = context_via(
        &FsVfs,
        &dir,
        Some(("src/pages/Overview.wf", Some((at, at + 7)))),
        guide(),
        Depth::Index,
    )
    .unwrap();
    assert_eq!(context.project, "dashboard");
    assert!(context.files.contains(&"src/pages/Overview.wf".to_string()));
    assert!(
        !context
            .files
            .iter()
            .any(|f| f.starts_with("build/") || f.contains(".env")),
        "{:?}",
        context.files
    );
    let open = context.open.as_ref().unwrap();
    assert_eq!(open.selected.as_deref(), Some("Heading"));
    assert!(
        context
            .components
            .components
            .iter()
            .any(|c| c.name == "Nudge"),
        "the project's own components"
    );
    assert!(
        context
            .components
            .components
            .iter()
            .any(|c| c.name == "Alert")
    );
    assert!(!context.tokens.is_empty());
    let numbered = guide()
        .iter()
        .filter(|(f, _)| {
            f.split_once('-')
                .is_some_and(|(n, _)| n.parse::<u32>().is_ok())
        })
        .count();
    assert_eq!(
        context.guide.len(),
        numbered,
        "every chapter, not the README"
    );
    assert!(
        context
            .guide
            .iter()
            .all(|c| c.text.is_none() && !c.title.is_empty())
    );
    let full = context_via(&FsVfs, &dir, None, guide(), Depth::Full).unwrap();
    assert!(full.guide.iter().all(|c| c.text.is_some()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_guide_s_anchors_are_the_site_s() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let chapters = chapters(guide(), Depth::Index);
    assert!(chapters.len() >= 49, "{} chapters", chapters.len());
    let anchors = regex::Regex::new(r#"Anchor\(id: "([^"]+)""#).unwrap();
    let mut checked = 0;
    for ch in &chapters {
        let Some(route) = &ch.route else { continue };
        let slug = route.trim_start_matches("guide/");
        let Some(page) = std::fs::read_dir(root.join("site/src/pages/guide"))
            .unwrap()
            .flatten()
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .trim_end_matches(".wf")
                    .split_once('-')
                    .is_some_and(|(_, s)| s == slug)
            })
        else {
            continue;
        };
        let text = std::fs::read_to_string(page.path()).unwrap();
        let ids: Vec<String> = anchors
            .captures_iter(&text)
            .map(|c| c[1].to_string())
            .collect();
        for s in ch.sections.iter().filter(|s| s.level == 2) {
            assert!(
                ids.contains(&s.anchor),
                "{} › {}: the site has no #{} ({ids:?})",
                ch.number,
                s.title,
                s.anchor
            );
            checked += 1;
        }
    }
    assert!(checked > 200, "{checked} sections");
    assert_eq!(
        anchor("When your rule and a built-in's disagree"),
        "when-your-rule-and-a-built-in-s-disagree"
    );
    assert_eq!(anchor("`style { }`"), "style");
    let forms = chapters
        .iter()
        .find(|c| c.route.as_deref() == Some("guide/forms"))
        .unwrap();
    assert_eq!(
        forms.cite(Some("what-a-value-must-be-validate")),
        format!(
            "{} {} › What a value must be: validate",
            forms.number, forms.title
        )
    );
    assert_eq!(
        forms.cite(None),
        format!("{} {}", forms.number, forms.title)
    );
}

#[test]
fn every_file_s_hunks_apply_as_patch_applies_them() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    let mut stack = vec![root.join("examples"), root.join("tests/fixtures")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                if e.file_name() != "build" {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "wf") {
                files.push(p);
            }
        }
    }
    files.sort();
    let have_patch = std::process::Command::new("patch")
        .arg("--version")
        .output()
        .is_ok();
    let scratch = std::env::temp_dir().join(format!("wf-agent-patch-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut checked = 0;
    for (n, path) in files.iter().enumerate() {
        let old = std::fs::read_to_string(path).unwrap();
        // A deterministic edit of every seventh line: one changed, one
        // removed, one added, in turn.
        let mut new = String::new();
        for (i, line) in old.split_inclusive('\n').enumerate() {
            match (i + n) % 21 {
                0 => new.push_str(&format!("    // changed {i}\n")),
                7 => {}
                14 => {
                    new.push_str(line);
                    new.push_str("    // added\n");
                }
                _ => new.push_str(line),
            }
        }
        let hunks = diff::hunks(&old, &new);
        if hunks.is_empty() {
            assert_eq!(old, new);
            continue;
        }
        let ids: Vec<usize> = hunks.iter().map(|h| h.id).collect();
        assert_eq!(diff::apply(&old, &hunks, &ids), new, "{}", path.display());
        assert_eq!(diff::apply(&old, &hunks, &[]), old, "{}", path.display());
        for h in &hunks {
            assert!(
                h.lines.iter().any(|l| l.kind != LineKind::Context),
                "a hunk changes something"
            );
        }
        if have_patch && n % 5 == 0 {
            let target = scratch.join("f.wf");
            std::fs::write(&target, &old).unwrap();
            let out = std::process::Command::new("patch")
                .arg("-s")
                .arg(&target)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write as _;
                    child
                        .stdin
                        .take()
                        .unwrap()
                        .write_all(diff::unified("f.wf", &hunks).as_bytes())?;
                    child.wait_with_output()
                })
                .unwrap();
            assert!(
                out.status.success(),
                "{}: {}",
                path.display(),
                String::from_utf8_lossy(&out.stderr)
            );
            assert_eq!(
                std::fs::read_to_string(&target).unwrap(),
                new,
                "patch reads the diff as we do: {}",
                path.display()
            );
        }
        checked += 1;
    }
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(checked > 100, "{checked} files");
}
