//! End-to-end: real projects, through the real `wf` binary, asserted on disk.
//!
//! The unit and conformance suites compile fragments through library entry
//! points. This one runs the compiler the way a user does — `wf build` in a
//! project directory — and then reads the `build/` directory it produced. That
//! is the only way to catch the things that live between the parts: config
//! handling, multi-file programs, route-to-file mapping, asset copying, the
//! linter's verdict on ordinary code, and every output mode the CLI supports.
//!
//! Fixtures live in `tests/fixtures/`, one directory per project, written the
//! way somebody would actually write them — including hand-authored design.
//! Each is copied to `target/e2e/<name>` and built there, so the fixtures stay
//! clean and the outputs stay inspectable after a failure.

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, LazyLock, Mutex};

use common::VOID_ELEMENTS;

/// A fixture project and what its build must produce.
struct Site {
    /// Directory name under `tests/fixtures/`.
    name: &'static str,
    /// Files the build must write, relative to the output directory.
    outputs: &'static [&'static str],
    /// Text that must appear somewhere in the built HTML (or, for PDF and slide
    /// builds, that is checked against the page stream instead).
    contains: &'static [&'static str],
}

const SITES: &[Site] = &[
    Site {
        name: "gallery",
        outputs: &["index.html", "app.js", "styles.css"],
        contains: &["Component Gallery"],
    },
    Site {
        name: "marketing",
        outputs: &["index.html", "app.js", "styles.css"],
        contains: &["Ledger"],
    },
    Site {
        name: "dashboard",
        outputs: &["index.html", "app.js", "styles.css"],
        contains: &["Ops Console"],
    },
    Site {
        name: "docs",
        outputs: &[
            "index.html",
            "handbook/index.html",
            "contact/index.html",
            "app.js",
            "styles.css",
        ],
        contains: &["The team handbook"],
    },
    Site {
        name: "bespoke",
        outputs: &["index.html", "app.js", "styles.css"],
        contains: &["Atelier"],
    },
    Site {
        name: "invoice",
        outputs: &["invoice.pdf"],
        contains: &[],
    },
    Site {
        name: "deck",
        outputs: &["deck.pdf"],
        contains: &[],
    },
];

// ─── Harness ────────────────────────────────────────────────────────────

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir(name: &str) -> PathBuf {
    repo_root().join("tests/fixtures").join(name)
}

/// Where a fixture is built. Under `target/`, so it is gitignored and survives
/// the test run for inspection.
fn build_root(name: &str) -> PathBuf {
    repo_root().join("target/e2e").join(name)
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create output dir");
    for entry in std::fs::read_dir(from).expect("read fixture dir") {
        let entry = entry.expect("dir entry");
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).expect("copy fixture file");
        }
    }
}

/// The result of one `wf build`.
struct Built {
    dir: PathBuf,
    stdout: String,
    stderr: String,
    ok: bool,
}

impl Built {
    fn out(&self, rel: &str) -> PathBuf {
        self.dir.join("build").join(rel)
    }
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.out(rel))
            .unwrap_or_else(|e| panic!("reading {}: {e}", self.out(rel).display()))
    }
    fn bytes(&self, rel: &str) -> Vec<u8> {
        std::fs::read(self.out(rel))
            .unwrap_or_else(|e| panic!("reading {}: {e}", self.out(rel).display()))
    }
    /// Every HTML file the build produced.
    fn html_files(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        collect_html(&self.dir.join("build"), &self.dir.join("build"), &mut out);
        out
    }
}

fn collect_html(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_html(&p, root, out);
        } else if p.extension().is_some_and(|e| e == "html") {
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .to_string();
            if let Ok(s) = std::fs::read_to_string(&p) {
                out.push((rel, s));
            }
        }
    }
}

/// Each site is built once and shared. Cargo runs these tests in parallel
/// threads of one process, so without the cache several of them would be
/// deleting and repopulating the same directory while the others read it.
static BUILDS: LazyLock<Mutex<HashMap<&'static str, Arc<Built>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Copy a fixture to `target/e2e` and run the real binary in it.
fn build_site(name: &'static str) -> Arc<Built> {
    let mut cache = BUILDS.lock().expect("build cache");
    if let Some(built) = cache.get(name) {
        return Arc::clone(built);
    }

    let dir = build_root(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_tree(&fixture_dir(name), &dir);

    let output = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("build")
        .current_dir(&dir)
        .output()
        .expect("running `wf build` — is the binary built?");

    let built = Arc::new(Built {
        dir,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        ok: output.status.success(),
    });
    cache.insert(name, Arc::clone(&built));
    built
}

// ─── The builds themselves ──────────────────────────────────────────────

/// Every fixture must build, and write the files its output mode promises.
#[test]
fn every_site_builds_and_writes_its_outputs() {
    let mut failures = Vec::new();
    for site in SITES {
        let built = build_site(site.name);
        if !built.ok {
            failures.push(format!(
                "{}: build failed\n  stdout: {}\n  stderr: {}",
                site.name,
                built.stdout.trim(),
                built.stderr.trim()
            ));
            continue;
        }
        for out in site.outputs {
            if !built.out(out).exists() {
                failures.push(format!("{}: build did not write {}", site.name, out));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "build failures:\n{}",
        failures.join("\n")
    );
}

/// Ordinary, idiomatic source must not produce accessibility warnings. A linter
/// that cries wolf on correct code is a linter people learn to ignore — and the
/// engine's own `wf init` scaffolds used to trip it.
#[test]
fn a_correctly_written_site_produces_no_lint_warnings() {
    let mut failures = Vec::new();
    for site in SITES {
        let built = build_site(site.name);
        let noise: Vec<&str> = built
            .stderr
            .lines()
            .filter(|l| l.contains("Warning ["))
            .collect();
        if !noise.is_empty() {
            failures.push(format!("{}:\n    {}", site.name, noise.join("\n    ")));
        }
    }
    assert!(
        failures.is_empty(),
        "idiomatic source drew accessibility warnings:\n{}",
        failures.join("\n")
    );
}

/// The scaffolds `wf init` writes must build clean. If the tool's own starting
/// point warns, every new project starts with noise its author did not cause.
#[test]
fn every_init_template_builds_clean() {
    let root = repo_root().join("target/e2e/_init");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create init root");

    let mut failures = Vec::new();
    for template in ["spa", "static", "pdf", "slides"] {
        let name = format!("scaffold_{template}");
        let init = Command::new(env!("CARGO_BIN_EXE_wf"))
            .args(["init", &name, "-t", template])
            .current_dir(&root)
            .output()
            .expect("running `wf init`");
        if !init.status.success() {
            failures.push(format!(
                "{template}: init failed: {}",
                String::from_utf8_lossy(&init.stderr).trim()
            ));
            continue;
        }

        let build = Command::new(env!("CARGO_BIN_EXE_wf"))
            .arg("build")
            .current_dir(root.join(&name))
            .output()
            .expect("running `wf build`");
        let stderr = String::from_utf8_lossy(&build.stderr).to_string();
        let stdout = String::from_utf8_lossy(&build.stdout).to_string();

        if !build.status.success() {
            failures.push(format!("{template}: build failed: {}", stderr.trim()));
        }
        let warnings: Vec<&str> = stderr
            .lines()
            .chain(stdout.lines())
            .filter(|l| l.contains("Warning ["))
            .collect();
        if !warnings.is_empty() {
            failures.push(format!("{template}:\n    {}", warnings.join("\n    ")));
        }
    }
    assert!(
        failures.is_empty(),
        "`wf init` scaffolds do not build clean:\n{}",
        failures.join("\n")
    );
}

// ─── What the HTML actually says ────────────────────────────────────────

/// The built pages must carry the content their source declared, and the shell
/// the config asked for.
#[test]
fn built_pages_carry_their_content_and_document_shell() {
    let mut failures = Vec::new();
    for site in SITES {
        if site.contains.is_empty() {
            continue;
        }
        let built = build_site(site.name);
        let all: String = built
            .html_files()
            .into_iter()
            .map(|(_, s)| s)
            .collect::<Vec<_>>()
            .join("\n");
        let js = built.read("app.js");
        let haystack = format!("{all}\n{js}");

        for needle in site.contains {
            if !haystack.contains(needle) {
                failures.push(format!("{}: output never mentions {:?}", site.name, needle));
            }
        }
        for (path, html) in built.html_files() {
            for required in ["<!DOCTYPE html>", "<html", "<head>", "<body>", "</html>"] {
                if !html.contains(required) {
                    failures.push(format!("{}/{}: missing {}", site.name, path, required));
                }
            }
            if !html.contains("styles.css") {
                failures.push(format!(
                    "{}/{}: never links the stylesheet",
                    site.name, path
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "document shell failures:\n{}",
        failures.join("\n")
    );
}

/// Every route the source declares must exist as a served file in a static
/// build. A route with no file is a 404 in production.
#[test]
fn static_builds_emit_a_file_per_route() {
    let built = build_site("docs");
    for route in ["index.html", "handbook/index.html", "contact/index.html"] {
        assert!(
            built.out(route).exists(),
            "the docs site declares a route with no file: {route}"
        );
    }

    // A deployment base path must reach the emitted links, or every link on the
    // deployed site points at the wrong origin.
    let home = built.read("index.html");
    assert!(
        home.contains("/handbook"),
        "base_path never reached the links:\n{home}"
    );
}

/// A static build must paint its content, not defer everything to JavaScript.
/// This is the entire promise of SSG: view-source shows the page.
#[test]
fn static_builds_paint_their_content_without_javascript() {
    let built = build_site("docs");
    let home = built.read("index.html");

    let body = home
        .split_once("<div id=\"app\">")
        .and_then(|(_, rest)| rest.split_once("</div>\n    <script"))
        .map(|(b, _)| b)
        .unwrap_or(&home);

    for needle in ["The team handbook", "Start here", "How we decide"] {
        assert!(
            body.contains(needle),
            "SSG page did not paint {needle:?} — it is JS-only:\n{body}"
        );
    }

    // i18n text must be resolved at build time, not left as a key.
    assert!(
        !body.contains("home.title"),
        "an untranslated i18n key reached the static paint:\n{body}"
    );
}

// ─── Output hygiene, on real builds ─────────────────────────────────────

/// Nothing invisible or non-textual may reach a shipped page. This is the same
/// invariant the unit suite asserts, re-checked on real, whole-project output —
/// the zero-width spaces the static renderers used to emit were only ever
/// visible here.
#[test]
fn shipped_html_contains_no_invisible_junk() {
    let forbidden = [
        ('\u{200B}', "ZERO WIDTH SPACE"),
        ('\u{FEFF}', "BOM"),
        ('\u{FFFE}', "U+FFFE"),
        ('\u{FFFF}', "U+FFFF"),
    ];

    let mut failures = Vec::new();
    for site in SITES {
        let built = build_site(site.name);
        for (path, html) in built.html_files() {
            for (ch, label) in forbidden {
                let n = html.matches(ch).count();
                if n > 0 {
                    failures.push(format!("{}/{}: {} × {}", site.name, path, n, label));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "junk in shipped HTML:\n{}",
        failures.join("\n")
    );
}

/// Shipped HTML must be well formed: tags balanced, void elements left void.
#[test]
fn shipped_html_is_well_formed() {
    let mut failures = Vec::new();
    for site in SITES {
        let built = build_site(site.name);
        for (path, html) in built.html_files() {
            if let Err(e) = check_balanced(&html) {
                failures.push(format!("{}/{}: {}", site.name, path, e));
            }
            for void in VOID_ELEMENTS {
                let close = format!("</{void}>");
                if html.contains(&close) {
                    failures.push(format!("{}/{}: closing {}", site.name, path, close));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "malformed HTML:\n{}",
        failures.join("\n")
    );
}

fn check_balanced(html: &str) -> Result<(), String> {
    let mut stack: Vec<String> = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find('<') {
        let after = &rest[i + 1..];
        if after.starts_with('!') {
            rest = after;
            continue;
        }
        let Some(end) = after.find('>') else { break };
        let inner = &after[..end];
        if let Some(name) = inner.strip_prefix('/') {
            let name = name.trim().to_ascii_lowercase();
            match stack.pop() {
                Some(open) if open == name => {}
                Some(open) => return Err(format!("</{name}> closed while <{open}> was open")),
                None => return Err(format!("</{name}> with nothing open")),
            }
        } else {
            let name = inner
                .split(char::is_whitespace)
                .next()
                .unwrap_or("")
                .trim_end_matches('/')
                .to_ascii_lowercase();
            if !name.is_empty() && !VOID_ELEMENTS.contains(&name.as_str()) && !inner.ends_with('/')
            {
                stack.push(name);
            }
        }
        rest = after;
    }
    if stack.is_empty() {
        Ok(())
    } else {
        Err(format!("never closed: {stack:?}"))
    }
}

/// The bundle must be loadable JavaScript. A stray brace from the codegen takes
/// the whole site down with a syntax error before a single line runs.
#[test]
fn every_bundle_is_syntactically_valid_javascript() {
    let mut failures = Vec::new();
    for site in SITES {
        let built = build_site(site.name);
        if !built.out("app.js").exists() {
            continue;
        }
        let js = built.read("app.js");
        if let Err(e) = check_js_balance(&js) {
            failures.push(format!("{}: {}", site.name, e));
        }
        for marker in ["undefined,", "[object Object]", "NaN"] {
            if js.contains(marker) {
                failures.push(format!("{}: bundle contains {:?}", site.name, marker));
            }
        }
    }
    assert!(failures.is_empty(), "bad bundles:\n{}", failures.join("\n"));
}

fn check_js_balance(js: &str) -> Result<(), String> {
    let (mut curly, mut paren, mut bracket) = (0i64, 0i64, 0i64);
    let mut in_str: Option<u8> = None;
    let mut escape = false;
    let mut line_comment = false;
    let mut prev = 0u8;

    for &b in js.as_bytes() {
        if line_comment {
            if b == b'\n' {
                line_comment = false;
            }
            prev = b;
            continue;
        }
        match in_str {
            Some(q) => {
                if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == q {
                    in_str = None;
                }
            }
            None => match b {
                b'/' if prev == b'/' => line_comment = true,
                b'"' | b'\'' | b'`' => in_str = Some(b),
                b'{' => curly += 1,
                b'}' => curly -= 1,
                b'(' => paren += 1,
                b')' => paren -= 1,
                b'[' => bracket += 1,
                b']' => bracket -= 1,
                _ => {}
            },
        }
        prev = b;
    }

    if in_str.is_some() {
        return Err("ends inside an unterminated string".into());
    }
    if curly != 0 || paren != 0 || bracket != 0 {
        return Err(format!(
            "unbalanced: braces {curly:+}, parens {paren:+}, brackets {bracket:+}"
        ));
    }
    Ok(())
}

// ─── Author-supplied design ─────────────────────────────────────────────

/// Hand-authored `style { }` blocks must survive into the shipped output. This
/// is the whole of a custom design: if the engine drops the author's styling,
/// the site renders as the default theme and nothing tells them why.
#[test]
fn hand_authored_design_reaches_the_output() {
    // Declarations written in the fixtures, which must appear in the build.
    let cases: &[(&str, &[&str])] = &[
        (
            "marketing",
            &[
                "3.25rem",
                "linear-gradient(135deg, #0F766E, #134E4A)",
                "rgba(255, 255, 255, 0.9)",
            ],
        ),
        (
            "bespoke",
            &["0.3em", "68rem", "'Editorial New', Georgia, serif"],
        ),
        ("dashboard", &["240px", "#0B1120"]),
    ];

    let mut failures = Vec::new();
    for (site, decls) in cases.iter().copied() {
        let built = build_site(site);
        let js = built.read("app.js");
        let html: String = built.html_files().into_iter().map(|(_, s)| s).collect();
        let haystack = format!("{js}{html}");
        for decl in decls.iter().copied() {
            if !haystack.contains(decl) {
                failures.push(format!(
                    "{site}: the author's `{decl}` never reached the build"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "dropped author styling:\n{}",
        failures.join("\n")
    );
}

/// A `Theme` declared in the source must reach the stylesheet, or a designed
/// site silently renders as the baseline one.
#[test]
fn a_declared_theme_reaches_the_stylesheet() {
    let built = build_site("marketing");
    let css = built.read("styles.css");
    for (token, value) in [
        ("--color-primary", "#0F766E"),
        ("--color-secondary", "#134E4A"),
        ("--radius-md", "14px"),
    ] {
        assert!(
            css.contains(value),
            "`Theme Ledger` sets {token} to {value}, which never reached styles.css"
        );
    }
}

/// A theme names only its differences; everything else keeps the baseline. A
/// theme that blanked the rest would leave `var(--…)` references dangling.
#[test]
fn a_theme_leaves_the_tokens_it_does_not_name_alone() {
    let css = build_site("marketing").read("styles.css");
    assert!(
        css.contains("--spacing-md: 1rem"),
        "an unnamed token lost its baseline value"
    );
    let declared = css.matches("  --").count();
    assert!(
        declared > 50,
        "only {declared} tokens reached :root — the baseline was not layered under the theme"
    );
}

/// Structural mode ships no design of the engine's, so the author's theme is the
/// entire palette. It must survive there.
#[test]
fn a_theme_applies_in_structural_mode_too() {
    let css = build_site("bespoke").read("styles.css");
    for value in ["#FAF9F6", "'Editorial New', Georgia, serif"] {
        assert!(
            css.contains(value),
            "structural mode dropped the author's `{value}` — it is the whole design there"
        );
    }
}

/// Naming a palette the engine no longer ships must fail loudly and say where it
/// went. Silently falling back to the baseline would ship the wrong design.
#[test]
fn a_removed_palette_name_fails_the_build_with_a_migration_hint() {
    let root = repo_root().join("target/e2e/_migration");
    let _ = std::fs::remove_dir_all(&root);
    copy_tree(&fixture_dir("gallery"), &root);

    // Put the project back the way it was before the palettes were removed, and
    // take away the theme file that replaced them.
    std::fs::write(
        root.join("webfluent.app.json"),
        r#"{ "name": "legacy", "theme": { "name": "brutalist" } }"#,
    )
    .expect("write config");

    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("build")
        .current_dir(&root)
        .output()
        .expect("running `wf build`");

    assert!(!out.status.success(), "the build should have refused");

    let message = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        message.contains("brutalist"),
        "the error must name the theme:\n{message}"
    );
    assert!(
        message.contains("examples/themes/brutalist.wf"),
        "the error must say where it went:\n{message}"
    );
}

/// A project that declares no theme still gets a complete token set, so nothing
/// in the stylesheet references a custom property that was never defined.
#[test]
fn a_project_with_no_theme_still_gets_every_token() {
    let css = build_site("gallery").read("styles.css");
    for token in [
        "--color-primary",
        "--spacing-md",
        "--radius-md",
        "--shadow-md",
        "--font-family",
    ] {
        assert!(css.contains(token), "the baseline is missing {token}");
    }

    // Every `var(--x)` the sheet uses must be defined in the same sheet.
    let mut undefined: Vec<String> = Vec::new();
    let mut rest = css.as_str();
    while let Some(i) = rest.find("var(--") {
        let after = &rest[i + 6..];
        let end = after
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
            .unwrap_or(after.len());
        let name = &after[..end];
        if !css.contains(&format!("--{name}:")) && !undefined.iter().any(|u| u == name) {
            undefined.push(name.to_string());
        }
        rest = after;
    }
    assert!(
        undefined.is_empty(),
        "the stylesheet references custom properties it never defines: {undefined:?}"
    );
}

/// Structural mode ships layout and mechanics without the engine's opinions. A
/// build that asked for no baseline design must not receive one.
#[test]
fn structural_mode_ships_no_baseline_design() {
    let bespoke = build_site("bespoke").read("styles.css");
    let styled = build_site("marketing").read("styles.css");

    // Layout and mechanics survive.
    for mechanism in [".wf-row", ".wf-container", ".wf-modal"] {
        assert!(
            bespoke.contains(mechanism),
            "structural mode dropped the layout mechanism {mechanism}"
        );
    }

    // The baseline look does not.
    assert!(
        bespoke.len() < styled.len(),
        "structural stylesheet ({} bytes) is not smaller than the full one ({} bytes)",
        bespoke.len(),
        styled.len()
    );
}

// ─── Documents ──────────────────────────────────────────────────────────

/// A PDF build must produce a real PDF: the header, a page tree, and the trailer
/// a reader needs to open it.
#[test]
fn pdf_and_slide_builds_produce_readable_documents() {
    for (site, file, min_pages) in [
        ("invoice", "invoice.pdf", 2usize),
        ("deck", "deck.pdf", 5usize),
    ] {
        let built = build_site(site);
        let bytes = built.bytes(file);

        assert!(
            bytes.starts_with(b"%PDF-"),
            "{site}: {file} does not start with the PDF header"
        );
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("/Type /Catalog") || text.contains("/Type/Catalog"),
            "{site}: no document catalog"
        );
        assert!(
            text.contains("trailer") || text.contains("startxref"),
            "{site}: no trailer — readers will reject it"
        );

        let pages = text
            .matches("/Type /Page")
            .count()
            .max(text.matches("/Type/Page").count());
        assert!(
            pages >= min_pages,
            "{site}: expected at least {min_pages} pages, the document declares {pages}"
        );
        assert!(
            bytes.len() > 1000,
            "{site}: {file} is {} bytes — too small to hold its content",
            bytes.len()
        );
    }
}

/// The deck must contain one page per slide, and the invoice must honour its
/// explicit page break.
#[test]
fn document_pagination_follows_the_source() {
    let deck = build_site("deck");
    assert!(
        deck.stdout.contains("5 slide(s)"),
        "the deck declares five slides; the build reported: {}",
        deck.stdout.trim()
    );

    let invoice = build_site("invoice");
    assert!(
        invoice.stdout.contains("2 page(s)"),
        "the invoice has one PageBreak, so two pages; the build reported: {}",
        invoice.stdout.trim()
    );
}

// ─── What reaches the static paint ──────────────────────────────────────

/// A list over seeded data must be in the HTML, not left to JavaScript.
///
/// The static renderer used to emit `<!--wf-for-->` for every loop, so a static
/// site's most substantial content — the rows — reached neither a crawler nor
/// the first paint, which is exactly what Largest Contentful Paint measures.
#[test]
fn seeded_lists_and_conditionals_paint_statically() {
    let root = repo_root().join("target/e2e/_static");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/pages")).expect("mkdir");
    std::fs::create_dir_all(root.join("src/stores")).expect("mkdir");

    std::fs::write(
        root.join("webfluent.app.json"),
        r#"{ "name": "s", "build": { "output": "build", "ssg": true } }"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/stores/posts.wf"),
        "Store PostStore {\n  state posts = [\n    { title: \"Slow roasting\", tag: \"coffee\" },\n    { title: \"Closed Mondays\", tag: \"shop\" }\n  ]\n  state featured = true\n}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/pages/Home.wf"),
        "Page Home (path: \"/\", title: \"Journal\") {\n\
         \x20 use PostStore\n\
         \x20 Container {\n\
         \x20   Heading(\"Journal\", h1)\n\
         \x20   if PostStore.featured { Text(\"Featured\") } else { Text(\"Nothing\") }\n\
         \x20   List { for post in PostStore.posts { Card { Heading(post.title, h2) Badge(post.tag) } } }\n\
         \x20 }\n\
         }\n",
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("build")
        .current_dir(&root)
        .output()
        .expect("wf build");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let html = std::fs::read_to_string(root.join("build/index.html")).expect("index.html");

    for row in ["Slow roasting", "coffee", "Closed Mondays", "shop"] {
        assert!(
            html.contains(row),
            "the static paint is missing {row:?}:\n{html}"
        );
    }
    assert!(
        !html.contains("<!--wf-for-->"),
        "a resolvable list was left as a placeholder:\n{html}"
    );
    // The resolved condition takes one branch and only one.
    assert!(html.contains("Featured"), "the true branch was not painted");
    assert!(!html.contains("Nothing"), "both branches were painted");
    assert!(
        !html.contains("<!--wf-if-->"),
        "a resolvable condition stayed a placeholder"
    );
}

/// What the compiler cannot know stays a placeholder rather than becoming a
/// guess. A fetched list has no build-time value, and inventing one would ship
/// markup the running page immediately contradicts.
#[test]
fn unresolvable_lists_still_defer_to_the_client() {
    let root = repo_root().join("target/e2e/_dynamic");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/pages")).expect("mkdir");

    std::fs::write(
        root.join("webfluent.app.json"),
        r#"{ "name": "d", "build": { "output": "build", "ssg": true } }"#,
    )
    .unwrap();
    std::fs::write(
        root.join("src/pages/Home.wf"),
        "Page Home (path: \"/\", title: \"D\") {\n\
         \x20 state rows = []\n\
         \x20 Container {\n\
         \x20   Heading(\"D\", h1)\n\
         \x20   for row in rows.filter(r => r.live) { Text(row.name) }\n\
         \x20 }\n\
         }\n",
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("build")
        .current_dir(&root)
        .output()
        .expect("wf build");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let html = std::fs::read_to_string(root.join("build/index.html")).expect("index.html");
    assert!(
        html.contains("<!--wf-for-->"),
        "a list the compiler cannot resolve must be left to the client:\n{html}"
    );
}

/// The document shell carries the structure a keyboard and screen-reader user
/// needs, and loads its bundle without blocking the parser.
#[test]
fn the_document_shell_is_navigable_and_non_blocking() {
    for site in ["gallery", "docs"] {
        let built = build_site(site);
        for (path, html) in built.html_files() {
            assert!(
                html.contains("class=\"wf-skip-link\""),
                "{site}/{path} has no skip link"
            );
            assert!(
                html.contains("<main id=\"wf-main\">"),
                "{site}/{path} has no main landmark"
            );
            assert!(
                html.contains("app.js\" defer"),
                "{site}/{path} loads its bundle without defer"
            );
        }
    }
}

// ─── Search and sharing ─────────────────────────────────────────────────

/// A static build must carry what a search result and a link preview are made
/// from. None of this existed: a page shipped a title and, if the project set
/// one, a description — no canonical, no card, no structured data.
#[test]
fn static_pages_carry_their_search_and_sharing_tags() {
    let built = build_site("docs");
    let mut failures = Vec::new();

    for (path, html) in built.html_files() {
        let head = html.split_once("</head>").map(|(h, _)| h).unwrap_or(&html);
        for required in [
            "<link rel=\"canonical\"",
            "property=\"og:title\"",
            "property=\"og:url\"",
            "property=\"og:site_name\"",
            "name=\"twitter:card\"",
            "application/ld+json",
        ] {
            if !head.contains(required) {
                failures.push(format!("docs/{path}: missing {required}"));
            }
        }
        // The canonical must be absolute; a relative one is worse than none.
        if let Some(after) = head.split_once("<link rel=\"canonical\" href=\"") {
            let url = after.1.split('"').next().unwrap_or("");
            if !url.starts_with("https://") {
                failures.push(format!("docs/{path}: canonical {url:?} is not absolute"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "missing SEO tags:\n{}",
        failures.join("\n")
    );
}

/// The structured data must be valid JSON, and must survive a title containing
/// the characters that would otherwise close its own `<script>` element.
#[test]
fn structured_data_is_valid_json_on_every_page() {
    let built = build_site("docs");
    for (path, html) in built.html_files() {
        let body = html
            .split_once("application/ld+json\">")
            .and_then(|(_, r)| r.split_once("</script>"))
            .unwrap_or_else(|| panic!("docs/{path}: no structured data"))
            .0;
        serde_json::from_str::<serde_json::Value>(body).unwrap_or_else(|e| {
            panic!("docs/{path}: structured data is not valid JSON: {e}\n{body}")
        });
    }
}

/// Every route the build emits must be in the sitemap, and the sitemap must not
/// name a route the build did not emit.
#[test]
fn the_sitemap_matches_the_routes_that_were_built() {
    let built = build_site("docs");
    let xml = built.read("sitemap.xml");
    let robots = built.read("robots.txt");

    // The fixture deploys under `/handbook`, so every URL carries that prefix — a
    // sitemap of origin-relative paths would point a crawler at the wrong path on
    // the right host, which is the failure that is hardest to notice.
    for route in ["/", "/handbook", "/contact"] {
        let expected = if route == "/" {
            "https://handbook.example/handbook/".to_string()
        } else {
            format!("https://handbook.example/handbook{route}")
        };
        assert!(
            xml.contains(&expected),
            "the sitemap is missing {expected}:\n{xml}"
        );
    }
    assert!(
        robots.contains("Sitemap: https://handbook.example/handbook/sitemap.xml"),
        "robots.txt does not point at the sitemap:\n{robots}"
    );
    // Google ignores both; emitting them implies a control nobody has.
    assert!(!xml.contains("priority"));
    assert!(!xml.contains("changefreq"));
}

/// A project with no `site_url` gets no absolute tags rather than wrong ones —
/// and must still build.
#[test]
fn a_site_without_a_url_omits_what_it_cannot_know() {
    let built = build_site("gallery");
    assert!(built.ok, "the build must not require a site_url");
    let html = built.read("index.html");
    assert!(
        !html.contains("rel=\"canonical\""),
        "a canonical was emitted with no origin to base it on"
    );
    assert!(
        !built.out("sitemap.xml").exists(),
        "a sitemap of relative URLs is worse than none"
    );
}
