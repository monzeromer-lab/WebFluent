//! The project's own words never name an older major version as current.
//!
//! The guide said "This guide describes WebFluent 4", the footer
//! "WebFluent 4 — GPL-3.0" and the server-rendering chapter `version = "4"`
//! a whole major release after 5.0 shipped. This reads every place a reader
//! meets the version — the README, the manifests, the guide, the site, the
//! language reference — and fails on "WebFluent N" where N is the previous
//! major (what goes stale when a new one ships), unless it is an upgrade
//! path ("WebFluent 4 → 5"), and on a `version = "N"` dependency line older
//! than the crate's own major. Older majors are named only where a project
//! that old is being migrated ("a file in the WebFluent 2 grammar"), which
//! is never a claim about the present. History is exempt: the release
//! notes, the changelogs and the upgrading chapter.

use std::fs;
use std::path::{Path, PathBuf};

const EXEMPT: &[&str] = &[
    "RELEASE_NOTES.md",
    "CHANGELOG.md",
    "45-upgrading.md",
    "45-upgrading.wf",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn current_major() -> u32 {
    let manifest = fs::read_to_string(root().join("Cargo.toml")).unwrap();
    let version = manifest
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .expect("the crate's version");
    version.split('.').next().unwrap().parse().unwrap()
}

fn files() -> Vec<PathBuf> {
    let root = root();
    let mut out = vec![
        root.join("README.md"),
        root.join("Cargo.toml"),
        root.join("AGENTS.md"),
        root.join("crates/wf-lsp/Cargo.toml"),
        root.join("bindings/node/README.md"),
        root.join("editors/vscode/README.md"),
    ];
    fn walk(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, exts, out);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| exts.contains(&e))
            {
                out.push(path);
            }
        }
    }
    walk(&root.join("md-docs"), &["md"], &mut out);
    walk(&root.join("site/src"), &["wf", "json"], &mut out);
    out.retain(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        p.exists() && !EXEMPT.contains(&name)
    });
    out
}

/// "WebFluent 4", "WebFluent v4.2" when 5 is current — but not
/// "WebFluent 4 → 5", and not "WebFluent 2", which only a migration names.
fn stale_mentions(line: &str, major: u32) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("WebFluent ") {
        let after = &rest[at + "WebFluent ".len()..];
        let after = after.strip_prefix('v').unwrap_or(after);
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = digits.parse::<u32>() {
            let tail =
                after[digits.len()..].trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
            let is_path = tail.trim_start().starts_with('→') || tail.trim_start().starts_with("->");
            if n + 1 == major && !is_path {
                found.push(format!("WebFluent {digits}"));
            }
        }
        rest = &rest[at + 1..];
    }
    // A dependency line: `webfluent = { version = "4"` or `version = "4"`
    // after `webfluent`.
    if let Some(at) = line.find("webfluent = { version = \"") {
        let v = &line[at + "webfluent = { version = \"".len()..];
        let digits: String = v.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.parse::<u32>().is_ok_and(|n| n < major) {
            found.push(format!("webfluent = {{ version = \"{digits}\" }}"));
        }
    }
    found
}

#[test]
fn no_older_major_version_is_named_as_current() {
    let major = current_major();
    let mut stale = Vec::new();
    for file in files() {
        let text = fs::read_to_string(&file).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            for mention in stale_mentions(line, major) {
                let rel = file
                    .strip_prefix(root())
                    .unwrap_or(&file)
                    .display()
                    .to_string();
                stale.push(format!("{rel}:{}: {mention}", i + 1));
            }
        }
    }
    assert!(
        stale.is_empty(),
        "these name a version older than {major} as current; update them, or make an upgrade path read `WebFluent N → M`:\n  {}",
        stale.join("\n  ")
    );
}

#[test]
fn the_check_tells_an_upgrade_path_from_a_stale_claim() {
    assert_eq!(
        stale_mentions("This guide describes WebFluent 4.", 5),
        ["WebFluent 4"]
    );
    assert!(stale_mentions("**WebFluent 4 → 5** needs no rewrite", 5).is_empty());
    assert!(stale_mentions("WebFluent 4.1 -> 4.2 runs first", 5).is_empty());
    assert!(stale_mentions("WebFluent 5.3 is current", 5).is_empty());
    assert!(stale_mentions("a file in the WebFluent 2 grammar", 5).is_empty());
    assert_eq!(
        stale_mentions(
            r#"webfluent = { version = "4", default-features = false }"#,
            5
        ),
        [r#"webfluent = { version = "4" }"#]
    );
}
