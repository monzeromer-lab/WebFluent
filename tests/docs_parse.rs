//! Every `wf` code block in the documentation is written in the grammar the
//! compiler reads. A block's declarations are parsed as a file; what it shows
//! at the top level besides them — usage of a component, a fragment of a
//! page — is parsed inside a page, and a block of statements inside an
//! action. A block that abbreviates with `...` or `…` is only checked for the
//! old grammar's spellings.

use std::path::Path;

use webfluent::parse_source;

/// The `wf`- and `wfx`-tagged fences of a Markdown file, with the line
/// each starts on and whether it is indented.
fn wf_blocks(markdown: &str) -> Vec<(usize, String, bool)> {
    let mut blocks = Vec::new();
    let mut current: Option<(usize, String, bool)> = None;
    for (ix, line) in markdown.lines().enumerate() {
        match &mut current {
            Some((_, body, _)) => {
                if line.trim_end() == "```" {
                    let (start, body, wfx) = current.take().unwrap();
                    blocks.push((start + 1, body, wfx));
                } else {
                    body.push_str(line);
                    body.push('\n');
                }
            }
            None => match line.trim_end() {
                "```wf" => current = Some((ix + 1, String::new(), false)),
                "```wfx" => current = Some((ix + 1, String::new(), true)),
                _ => {}
            },
        }
    }
    blocks
}

const DECLARATIONS: &[&str] = &[
    "api",
    "external",
    "page",
    "component",
    "store",
    "theme",
    "app",
    "type",
    "enum",
    "const",
    "animation",
    "test",
    "data",
    "image",
];

fn starts_declaration(line: &str) -> bool {
    let word = line
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or("");
    DECLARATIONS.contains(&word)
        && (line.len() == word.len() || !line.as_bytes()[word.len()].is_ascii_alphanumeric())
}

/// A line with its comment and its string literals taken out, so the
/// brackets left in it are the ones that open and close blocks.
///
/// `open` says a `"""` block string was still open when the line began,
/// and what comes back says whether it still is.
fn code_of(line: &str, open: bool) -> (String, bool) {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut block = open;
    let mut i = 0;
    while i < chars.len() {
        if block {
            if chars[i] == '"' && chars.get(i + 1) == Some(&'"') && chars.get(i + 2) == Some(&'"') {
                block = false;
                i += 3;
                continue;
            }
            i += 1;
            continue;
        }
        // A comment ends the line.
        if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        // `#"…"#`, however many hashes.
        if chars[i] == '#' {
            let mut hashes = 0;
            let mut j = i;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                j += 1;
                while j < chars.len() {
                    if chars[j] == '"' && (1..=hashes).all(|n| chars.get(j + n) == Some(&'#')) {
                        j += hashes + 1;
                        break;
                    }
                    j += 1;
                }
                i = j;
                continue;
            }
        }
        if chars[i] == '"' {
            if chars.get(i + 1) == Some(&'"') && chars.get(i + 2) == Some(&'"') {
                block = true;
                i += 3;
                continue;
            }
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    (out, block)
}

/// The block split into its declarations and the lines outside them.
fn split(block: &str) -> (String, String) {
    let mut declarations = String::new();
    let mut loose = String::new();
    let mut depth = 0i32;
    let mut in_declaration = false;
    let mut in_block_string = false;
    // Doc comments go with whatever follows them.
    let mut docs = String::new();
    for line in block.lines() {
        if depth == 0 && !in_block_string && line.trim_start().starts_with("///") {
            docs.push_str(line);
            docs.push('\n');
            continue;
        }
        if depth == 0 && !in_block_string {
            in_declaration = starts_declaration(line);
        }
        let into = if in_declaration {
            &mut declarations
        } else {
            &mut loose
        };
        into.push_str(&std::mem::take(&mut docs));
        into.push_str(line);
        into.push('\n');
        // A declaration runs to its closing bracket of any kind — a `const`
        // list or a call may span lines as well as a block — and a block
        // string runs to its closing delimiter, brackets and all.
        let (code, still_open) = code_of(line, in_block_string);
        in_block_string = still_open;
        for c in code.chars() {
            match c {
                '{' | '[' | '(' => depth += 1,
                '}' | ']' | ')' => depth -= 1,
                _ => {}
            }
        }
        if depth <= 0 && !in_block_string {
            depth = 0;
            in_declaration = false;
        }
    }
    (declarations, loose)
}

/// Spellings of the grammar WebFluent 2 had, which no block may contain:
/// at the start of a line, or anywhere in it.
const OLD_LEADING: &[&str] = &[
    "Page ",
    "Component ",
    "Store ",
    "Theme ",
    "App ",
    "App{",
    "Route(",
    "fetch ",
    "token ",
];
const OLD_ANYWHERE: &[&str] = &[
    "on:click",
    "on:submit",
    "on:change",
    "on:input",
    "TabPage(",
    "Tcell(",
    "Thead {",
    "Trow {",
    "Tbody {",
    ", h1)",
    ", h2)",
    ", h3)",
    ", primary)",
    ", primary,",
    // The original grammar's clause, `if open, animate(fadeIn, fast) {`.
    // `animate(node, "pulse")` is the handle the runtime gives back, and
    // is current — so the comma is what tells them apart.
    ", animate(",
    ", large)",
    ", small)",
];

fn old_spellings(block: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    for line in block.lines() {
        let code = line.split("//").next().unwrap_or("").trim_start();
        for old in OLD_LEADING {
            if code.starts_with(old) && !found.contains(old) {
                found.push(*old);
            }
        }
        for old in OLD_ANYWHERE {
            if code.contains(old) && !found.contains(old) {
                found.push(*old);
            }
        }
    }
    found
}

fn indent(text: &str, by: &str) -> String {
    text.lines()
        .map(|l| {
            if l.is_empty() {
                String::from("\n")
            } else {
                format!("{by}{l}\n")
            }
        })
        .collect()
}

fn check(path: &str) -> Vec<String> {
    check_until(path, None)
}

/// A page may only read an `env` name that says anyone may: one beginning
/// `PUBLIC_`, or one the config's `public_env` lists. A guide cannot show
/// the second — the block carries no config — so every `env.NAME` it
/// writes must be a `PUBLIC_` one, or a reader copying it gets a compile
/// error the page they copied it from did not have.
fn public_env(path: &str, markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (line, block, _) in wf_blocks(markdown) {
        for (n, _) in block.match_indices("env.") {
            let rest = &block[n + 4..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            // `env.NAME` in prose about the rule itself, and a method call
            // on something a program called `env`, are not reads.
            if name.is_empty() || name == "NAME" || name == "X" || name.starts_with("PUBLIC_") {
                continue;
            }
            out.push(format!(
                "{path}:{line}: `env.{name}` is a name no page may read —                  every `env` a guide shows must begin `PUBLIC_`"
            ));
        }
    }
    out
}

/// [`check`], and every block that is whole declarations is also held to
/// the semantic and type checks: a guide's examples must build, not only
/// parse. A block that names something another block declares is written
/// with a `…` and skipped.
/// Whether a block leaves something out — `…`, or `...` that is not a
/// spread (`[...a, b]`, `{ ...m }`).
fn abbreviates(block: &str) -> bool {
    if block.contains('…') {
        return true;
    }
    block.match_indices("...").any(|(i, _)| {
        !block[i + 3..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

fn check_strictly(path: &str) -> Vec<String> {
    let mut failures = check(path);
    let markdown =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    failures.extend(public_env(path, &markdown));
    for (line, block, wfx) in wf_blocks(&markdown) {
        if abbreviates(&block) {
            continue;
        }
        let block = if wfx {
            match webfluent::layout::to_braces(&block, &format!("{path}.wfx")) {
                Ok(b) => b,
                Err(_) => continue,
            }
        } else {
            block
        };
        let (declarations, loose) = split(&block);
        if !loose.trim().is_empty() || declarations.trim().is_empty() {
            continue;
        }
        let Ok(program) = parse_source(&declarations, path) else {
            continue;
        };
        let file_of = |_: usize| format!("{path}:{line}");
        let mut findings = webfluent::sema::check(&program, &file_of);
        let typed = webfluent::sema::types::check(&program, &file_of);
        findings.errors.extend(typed.findings.errors);
        for e in &findings.errors {
            failures.push(format!("{path}:{line}: {e}"));
        }
        // A guide shows the idiomatic spelling, so its warnings are
        // failures too.
        for w in &findings.warnings {
            failures.push(format!("{path}:{line}: {w}"));
        }
        let semantic = webfluent::linter::validate_semantics_in(&program, &file_of);
        for e in &semantic {
            failures.push(format!("{path}:{line}: {e}"));
        }
        // The back end too. Parsing and type-checking say nothing about the
        // JavaScript a block turns into: a store's `remove` action once
        // compiled to a list's `splice`, which passed every check above and
        // threw on the first click.
        for finding in emitted_js_problems(&program) {
            failures.push(format!("{path}:{line}: {finding}"));
        }
    }
    failures
}

/// What the JavaScript for `program` gets wrong, read back from the emission.
///
/// The check that matters here is the one the type checker cannot make: a
/// store compiles to an object of the state and actions it declares, so every
/// `Store.member(…)` the bundle calls has to be one of them.
fn emitted_js_problems(program: &webfluent::parser::ast::Program) -> Vec<String> {
    use webfluent::parser::ast::{Declaration, StatementKind};

    let js = webfluent::codegen::JsCodegen::new().generate(program);
    let mut problems = Vec::new();

    for declaration in &program.declarations {
        let Declaration::Store(store) = declaration else {
            continue;
        };
        let members: Vec<&str> = store
            .body
            .iter()
            .filter_map(|statement| match &statement.kind {
                StatementKind::State(s) => Some(s.name.as_str()),
                StatementKind::Derived(d) => Some(d.name.as_str()),
                StatementKind::Action(a) => Some(a.name.as_str()),
                StatementKind::Resource(r) => Some(r.name.as_str()),
                _ => None,
            })
            .collect();

        let needle = format!("{}.", store.name);
        for (at, _) in js.match_indices(&needle) {
            // A call, not a read: `Todos.remove(` but not `Todos.items`.
            let rest = &js[at + needle.len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() || !rest[name.len()..].starts_with('(') {
                continue;
            }
            if !members.contains(&name.as_str()) {
                problems.push(format!(
                    "the bundle calls `{}.{name}(…)`, which the store does not declare",
                    store.name
                ));
            }
        }
    }
    problems.sort();
    problems.dedup();
    problems
}

/// Check `path` up to the first line that starts with `stop`, when given:
/// release notes keep the older grammar in their older sections.
fn check_until(path: &str, stop: Option<&str>) -> Vec<String> {
    let markdown =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    let markdown = match stop.and_then(|stop| markdown.lines().position(|l| l.starts_with(stop))) {
        Some(at) => markdown.lines().take(at).collect::<Vec<_>>().join("\n"),
        None => markdown,
    };
    let mut failures = Vec::new();
    for (line, block, wfx) in wf_blocks(&markdown) {
        for old in old_spellings(&block) {
            failures.push(format!("{path}:{line}: the block still spells `{old}`"));
        }
        if abbreviates(&block) {
            continue;
        }
        // An indented block is checked as its braced spelling, which the
        // converter holds to the same tokens.
        let block = if wfx {
            match webfluent::layout::to_braces(&block, &format!("{path}.wfx")) {
                Ok(braced) => braced,
                Err(e) => {
                    failures.push(format!("{path}:{line}: {e}\n{block}"));
                    continue;
                }
            }
        } else {
            block
        };
        let (declarations, loose) = split(&block);
        let loose = loose.trim_matches('\n');
        let as_render = format!(
            "{declarations}page Doc(path: \"/\") {{\n{}}}\n",
            indent(loose, "    ")
        );
        if parse_source(&as_render, path).is_ok() {
            continue;
        }
        let as_action = format!(
            "{declarations}page Doc(path: \"/\") {{\n    action doc() {{\n{}    }}\n}}\n",
            indent(loose, "        ")
        );
        if parse_source(&as_action, path).is_ok() {
            continue;
        }
        let error = parse_source(&as_render, path).unwrap_err();
        failures.push(format!("{path}:{line}: {error}\n{block}"));
    }
    failures
}

#[test]
fn the_agents_guide_parses() {
    let failures = check_strictly("AGENTS.md");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn the_readme_parses() {
    let failures = check_strictly("README.md");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn the_current_release_notes_parse() {
    let failures = check_until("RELEASE_NOTES.md", Some("# WebFluent v2.2"));
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn the_specs_parse() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
    let mut failures = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".md") {
            // `SYNTAX_V2.md` is the design record of the 3.0 grammar, not a
            // guide: its blocks quote one another's declarations, so they
            // are held to the grammar and not to the checker.
            if name == "SYNTAX_V2.md" {
                failures.extend(check(&format!("spec/{name}")));
            } else {
                failures.extend(check_strictly(&format!("spec/{name}")));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The developer guide under `md-docs/`: every block is real WebFluent 3.
#[test]
fn the_guide_parses() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("md-docs");
    let mut failures = Vec::new();
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".md"))
        .collect();
    names.sort();
    for name in names {
        failures.extend(check_strictly(&format!("md-docs/{name}")));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The design-tokens chapter lists every baseline token with its value,
/// between `<!-- tokens -->` and `<!-- /tokens -->`, written from
/// `default_tokens` itself — a hand-kept table fell behind it twice. Run
/// with `WF_WRITE_TOKENS=1` to rewrite it after the baseline changes.
#[test]
fn the_design_tokens_chapter_is_the_baseline() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("md-docs/41-design-tokens.md");
    let chapter = std::fs::read_to_string(&path).unwrap();
    let tokens = webfluent::themes::tokens::default_tokens();
    let groups: &[(&str, &[&str])] = &[
        ("Colour", &["color-"]),
        ("Font", &["font-", "line-height-"]),
        ("Spacing", &["spacing-"]),
        ("Radius", &["radius-"]),
        ("Shadow", &["shadow-"]),
        ("Motion", &["transition-", "animation-"]),
        ("Easing", &["ease-"]),
        ("Breakpoints", &["screen-"]),
        ("Code", &["syntax-"]),
        ("Terminal", &["term-"]),
    ];
    let mut names: Vec<&String> = tokens.keys().collect();
    names.sort();
    let mut table = String::new();
    let mut placed: Vec<&String> = Vec::new();
    for (group, prefixes) in groups {
        let members: Vec<&&String> = names
            .iter()
            .filter(|n| prefixes.iter().any(|p| n.starts_with(p)))
            .collect();
        if members.is_empty() {
            continue;
        }
        table.push_str(&format!("### {group}\n\n| Token | Baseline |\n|---|---|\n"));
        for n in members {
            table.push_str(&format!(
                "| `{n}` | `{}` |\n",
                tokens[*n].replace('|', "\\|")
            ));
            placed.push(n);
        }
        table.push('\n');
    }
    let rest: Vec<&&String> = names.iter().filter(|n| !placed.contains(n)).collect();
    if !rest.is_empty() {
        table.push_str("### Layout\n\n| Token | Baseline |\n|---|---|\n");
        for n in rest {
            table.push_str(&format!(
                "| `{n}` | `{}` |\n",
                tokens[*n].replace('|', "\\|")
            ));
        }
        table.push('\n');
    }
    table.push_str(&format!("{} tokens in all.\n", names.len()));
    let (open, close) = ("<!-- tokens -->\n", "<!-- /tokens -->");
    let start = chapter.find(open).expect("the tokens marker") + open.len();
    let end = chapter.find(close).expect("the closing marker");
    let want = format!("\n{table}\n");
    if std::env::var("WF_WRITE_TOKENS").is_ok() {
        let written = format!("{}{want}{}", &chapter[..start], &chapter[end..]);
        std::fs::write(&path, written).unwrap();
        return;
    }
    assert_eq!(
        &chapter[start..end],
        want,
        "md-docs/41-design-tokens.md is behind the baseline; run with WF_WRITE_TOKENS=1"
    );
}

/// The guide's pages are generated from `md-docs/` by
/// `scripts/site-from-guide.py`, but the sidebar that links them is written
/// by hand in `site/src/components/DocSidebar.wf`. They drifted: chapter 19,
/// Security, was added to the guide and never to the sidebar, so the page
/// built, deployed and answered its URL while being reachable from no
/// navigation at all — and the two chapters after it were left a number
/// short, numbering the components reference 19 and the cookbook 20.
#[test]
fn the_sidebar_lists_every_chapter_of_the_guide() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // The chapters, from the directory the guide actually is.
    let mut chapters: Vec<(String, String)> = std::fs::read_dir(root.join("md-docs"))
        .expect("md-docs")
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let (num, rest) = name.split_once('-')?;
            if num.len() != 2 || !num.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            Some((num.to_string(), rest.trim_end_matches(".md").to_string()))
        })
        .collect();
    chapters.sort();
    assert!(
        chapters.len() > 15,
        "expected the whole guide, got {chapters:?}"
    );

    let sidebar = std::fs::read_to_string(root.join("site/src/components/DocSidebar.wf"))
        .expect("DocSidebar.wf");

    for (num, stem) in &chapters {
        // Every chapter is numbered in the rail, and its number is the one
        // the file carries.
        assert!(
            sidebar.contains(&format!("Text(\"{num}\", class: \"num\")")),
            "chapter {num} ({stem}) is not in the sidebar"
        );
        assert!(
            sidebar.contains(&format!("t(\"ch.{num}\")")),
            "chapter {num} ({stem}) has no title in the sidebar"
        );
    }

    // And nothing beyond them: a number the guide does not have means the
    // rail was renumbered without the guide.
    let highest: u32 = chapters.last().unwrap().0.parse().unwrap();
    for over in (highest + 1)..=(highest + 3) {
        assert!(
            !sidebar.contains(&format!("t(\"ch.{over:02}\")")),
            "the sidebar names chapter {over:02}, which the guide does not have"
        );
    }

    // Both locales have to carry every title, or the rail is blank in one.
    for locale in ["en", "ar"] {
        let table =
            std::fs::read_to_string(root.join(format!("site/src/translations/{locale}.json")))
                .expect("translations");
        for (num, stem) in &chapters {
            assert!(
                table.contains(&format!("\"ch.{num}\"")),
                "{locale}.json has no title for chapter {num} ({stem})"
            );
        }
    }
}

/// The guide's pages on the site are generated from `md-docs/` by
/// `scripts/site-from-guide.py`, and nothing held the committed pages to what
/// the script would write. They fell behind: the documentation audit added
/// five rows to the design-token table in `md-docs/12-styling.md`, and the
/// published Styling chapter never got them.
///
/// This runs the script into a scratch directory, formatted by the binary
/// this test run just built, and compares.
#[test]
fn the_sites_guide_pages_are_current_with_the_guide() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipped: the generator needs python3");
        return;
    }
    let out = std::env::temp_dir().join(format!("wf-guide-fresh-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let run = std::process::Command::new("python3")
        .arg(root.join("scripts/site-from-guide.py"))
        .env("WF_GUIDE_OUT", &out)
        .env("WF_BIN", env!("CARGO_BIN_EXE_wf"))
        .output()
        .expect("run the generator");
    assert!(
        run.status.success(),
        "the generator failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let committed = root.join("site/src/pages/guide");
    let names = |dir: &Path| {
        let mut v: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".wf"))
            .collect();
        v.sort();
        v
    };
    assert_eq!(
        names(&committed),
        names(&out),
        "the site's guide pages are not the set md-docs/ generates"
    );
    let mut stale = Vec::new();
    for name in names(&out) {
        let want = std::fs::read_to_string(out.join(&name)).unwrap();
        let have = std::fs::read_to_string(committed.join(&name)).unwrap();
        if want != have {
            stale.push(name);
        }
    }
    // The sidebar is written from the chapters' metadata too.
    let want = std::fs::read_to_string(out.join("sidebar/DocSidebar.wf")).unwrap_or_default();
    let have = std::fs::read_to_string(root.join("site/src/components/DocSidebar.wf")).unwrap();
    if want != have {
        stale.push("site/src/components/DocSidebar.wf".to_string());
    }
    let _ = std::fs::remove_dir_all(&out);
    assert!(
        stale.is_empty(),
        "these pages are behind md-docs/; run `python3 scripts/site-from-guide.py`: {stale:?}"
    );
}

// ─── The editor grammar's corpus ─────────────────────────────────────────
//
// The Tree-sitter grammar that Zed highlights with is written by hand beside
// the compiler's parser, and nothing held the one to the other: `just
// grammar-test` parsed every `.wf` file in the repository, and no file in
// the repository used `api`, `socket`, `validate`, `image`, `try`, a date
// literal or most of what 4.0 added — so the grammar marked all of it an
// error in every reader's editor and the check still passed. The guide's
// blocks do use all of it, and this file already holds every one of them to
// the compiler. So the blocks the compiler accepts are written out as the
// grammar's corpus, one file per document, and the grammar is held to them.

/// The documents whose blocks the compiler is held to, as above.
fn corpus_documents() -> Vec<(String, Option<&'static str>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut docs = vec![
        ("AGENTS.md".to_string(), None),
        ("README.md".to_string(), None),
        ("RELEASE_NOTES.md".to_string(), Some("# WebFluent v2.2")),
    ];
    for dir in ["spec", "md-docs"] {
        let mut names: Vec<String> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".md"))
            .collect();
        names.sort();
        for name in names {
            docs.push((format!("{dir}/{name}"), None));
        }
    }
    docs
}

/// The corpus files for one document: its blocks as the compiler accepts
/// them, braced (for `.wf`), and its indented blocks that are whole
/// declarations as written (for `.wfx`).
fn corpus_for(path: &str, stop: Option<&str>) -> (String, String) {
    let markdown =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    let markdown = match stop.and_then(|stop| markdown.lines().position(|l| l.starts_with(stop))) {
        Some(at) => markdown.lines().take(at).collect::<Vec<_>>().join("\n"),
        None => markdown,
    };
    let mut wf = String::new();
    let mut wfx = String::new();
    for (_, block, indented) in wf_blocks(&markdown) {
        if abbreviates(&block) {
            continue;
        }
        let braced = if indented {
            match webfluent::layout::to_braces(&block, &format!("{path}.wfx")) {
                Ok(b) => b,
                Err(_) => continue,
            }
        } else {
            block.clone()
        };
        let (declarations, loose) = split(&braced);
        let loose = loose.trim_matches('\n');
        let as_render = format!(
            "{declarations}page Doc(path: \"/\") {{\n{}}}\n",
            indent(loose, "    ")
        );
        let as_action = format!(
            "{declarations}page Doc(path: \"/\") {{\n    action doc() {{\n{}    }}\n}}\n",
            indent(loose, "        ")
        );
        let accepted = if loose.trim().is_empty() && parse_source(&declarations, path).is_ok() {
            declarations.clone()
        } else if parse_source(&as_render, path).is_ok() {
            as_render
        } else if parse_source(&as_action, path).is_ok() {
            as_action
        } else {
            continue;
        };
        // By document, not line: a line number would move with every edit
        // above the block, and the corpus would go stale with no code changed.
        wf.push_str(&format!("// {path}\n{accepted}\n"));
        if indented && loose.trim().is_empty() {
            wfx.push_str(&format!("// {path}\n{}\n", block.trim_end()));
        }
    }
    (wf, wfx)
}

fn corpus_name(path: &str) -> String {
    path.trim_end_matches(".md").replace('/', "--")
}

/// The committed corpus is what the guide's blocks are now. Run with
/// `WF_WRITE_GUIDE_CORPUS=1` to rewrite it after the guide changes.
#[test]
fn the_editor_grammar_corpus_is_current_with_the_guide() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wf_dir = root.join("editors/tree-sitter-webfluent/test/guide");
    let wfx_dir = root.join("editors/tree-sitter-webfluentx/test/guide");
    let write = std::env::var_os("WF_WRITE_GUIDE_CORPUS").is_some();
    if write {
        for dir in [&wf_dir, &wfx_dir] {
            let _ = std::fs::remove_dir_all(dir);
            std::fs::create_dir_all(dir).unwrap();
        }
    }
    let mut stale = Vec::new();
    let mut expected_files = Vec::new();
    for (path, stop) in corpus_documents() {
        let (wf, wfx) = corpus_for(&path, stop);
        for (dir, ext, text) in [(&wf_dir, "wf", wf), (&wfx_dir, "wfx", wfx)] {
            if text.trim().is_empty() {
                continue;
            }
            let file = dir.join(format!("{}.{ext}", corpus_name(&path)));
            expected_files.push(file.clone());
            if write {
                std::fs::write(&file, &text).unwrap();
            } else if std::fs::read_to_string(&file).ok().as_deref() != Some(text.as_str()) {
                stale.push(file.strip_prefix(root).unwrap().display().to_string());
            }
        }
    }
    // And nothing left over from a document that is gone.
    for dir in [&wf_dir, &wfx_dir] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                if !expected_files.contains(&e.path()) {
                    stale.push(e.path().strip_prefix(root).unwrap().display().to_string());
                }
            }
        }
    }
    assert!(
        stale.is_empty(),
        "the editor grammar's corpus is behind the guide; rewrite it with \
         `WF_WRITE_GUIDE_CORPUS=1 cargo test --test docs_parse the_editor_grammar_corpus`: {stale:?}"
    );
}

/// The documentation site's search and its components reference are written
/// by `scripts/site-data.py`, from the guide and from `wf registry --json`,
/// and nothing held the committed files to what it writes — so a section
/// added to the guide was missing from the site's search until someone
/// remembered to rerun it. This runs it into a scratch directory, with the
/// binary this test run just built, and compares.
#[test]
fn the_sites_search_and_registry_are_current() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipped: the generator needs python3");
        return;
    }
    let out = std::env::temp_dir().join(format!("wf-site-data-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).unwrap();
    let run = std::process::Command::new("python3")
        .arg(root.join("scripts/site-data.py"))
        .env("WF_SITE_DATA_OUT", &out)
        .env("WF_BIN", env!("CARGO_BIN_EXE_wf"))
        .output()
        .expect("run the generator");
    assert!(
        run.status.success(),
        "the generator failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let mut stale = Vec::new();
    for name in ["search-index.json", "registry.json"] {
        let want = std::fs::read_to_string(out.join(name)).unwrap();
        let have = std::fs::read_to_string(root.join("site/src").join(name)).unwrap_or_default();
        if want != have {
            stale.push(name);
        }
    }
    let _ = std::fs::remove_dir_all(&out);
    assert!(
        stale.is_empty(),
        "these are behind the guide or the registry; run `python3 scripts/site-data.py`: {stale:?}"
    );
}

// ─── The guide, held to the compiler ────────────────────────────────────
//
// What the guide says exists is what the compiler has: every example that
// claims a diagnostic draws it, every link lands, every declaration, keyword,
// built-in, browser value, function, config key, command and code is named
// somewhere a reader will find it, and the defaults it prints are the
// compiler's own.

/// The guide's chapters, in order: `(file name, text)`.
fn chapters() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("md-docs");
    let mut out: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() > 3 && n[..2].chars().all(|c| c.is_ascii_digit()) && n.ends_with(".md"))
        .map(|n| {
            let text = std::fs::read_to_string(dir.join(&n)).unwrap();
            (n, text)
        })
        .collect();
    out.sort();
    out
}

fn chapter(stem: &str) -> String {
    chapters()
        .into_iter()
        .find(|(n, _)| n.contains(stem))
        .unwrap_or_else(|| panic!("no chapter named like {stem}"))
        .1
}

/// Every finding the build would print for `src`, as text.
fn every_finding(src: &str) -> Vec<String> {
    let Ok(program) = parse_source(src, "example.wf") else {
        return vec![format!(
            "does not parse: {}",
            parse_source(src, "example.wf").unwrap_err()
        )];
    };
    let file_of = |_: usize| "example.wf".to_string();
    let mut out = Vec::new();
    let findings = webfluent::sema::check(&program, &file_of);
    out.extend(findings.errors.iter().map(|d| d.to_string()));
    out.extend(findings.warnings.iter().map(|d| d.to_string()));
    let typed = webfluent::sema::types::check(&program, &file_of);
    out.extend(typed.findings.errors.iter().map(|d| d.to_string()));
    out.extend(typed.findings.warnings.iter().map(|d| d.to_string()));
    out.extend(
        webfluent::linter::validate_semantics_in(&program, &file_of)
            .iter()
            .map(|d| d.to_string()),
    );
    let program = webfluent::sema::lower(program);
    out.extend(
        webfluent::linter::lint_accessibility_in(&program, &file_of)
            .iter()
            .map(|w| w.to_string()),
    );
    if let Ok(tokens) = webfluent::themes::resolve_tokens(&program, &Default::default()) {
        out.extend(
            webfluent::linter::lint_contrast_in(&program, &tokens, &file_of)
                .iter()
                .map(|w| w.to_string()),
        );
    }
    out.extend(
        webfluent::linter::lint_unused_in(&program, &file_of)
            .iter()
            .map(|w| w.to_string()),
    );
    out.extend(
        webfluent::linter::lint_vocabulary_with(&program, "", &file_of)
            .iter()
            .map(|w| w.to_string()),
    );
    out
}

/// A ```` ```wf expect T04 ```` block is a program that draws `T04` on
/// purpose. Every one of them draws exactly the code it is filed under —
/// and every code the compiler has is filed somewhere.
#[test]
fn every_diagnostic_example_draws_its_code() {
    let mut failures = Vec::new();
    let mut shown: Vec<String> = Vec::new();
    for (name, text) in chapters() {
        let mut lines = text.lines().enumerate();
        while let Some((at, line)) = lines.next() {
            let Some(code) = line.trim_end().strip_prefix("```wf expect ") else {
                continue;
            };
            let mut body = String::new();
            for (_, l) in lines.by_ref() {
                if l.trim_end() == "```" {
                    break;
                }
                body.push_str(l);
                body.push('\n');
            }
            let found = every_finding(&body);
            if !found.iter().any(|f| f.contains(&format!("[{code}]"))) {
                failures.push(format!(
                    "md-docs/{name}:{}: does not draw {code}; it draws {found:?}",
                    at + 1
                ));
            }
            shown.push(code.to_string());
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));

    // Every code in the compiler has an entry in the diagnostics chapter.
    let diagnostics = chapter("-diagnostics");
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut codes: Vec<String> = Vec::new();
    fn walk(dir: &Path, codes: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, codes);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let text = std::fs::read_to_string(&p).unwrap();
                let bytes = text.as_bytes();
                for i in 0..bytes.len().saturating_sub(4) {
                    if bytes[i] == b'"'
                        && b"ATSPUV".contains(&bytes[i + 1])
                        && bytes[i + 2].is_ascii_digit()
                        && bytes[i + 3].is_ascii_digit()
                        && bytes[i + 4] == b'"'
                    {
                        let code = text[i + 1..i + 4].to_string();
                        if !codes.contains(&code) {
                            codes.push(code);
                        }
                    }
                }
            }
        }
    }
    walk(&src_dir, &mut codes);
    codes.sort();
    let missing: Vec<&String> = codes
        .iter()
        .filter(|c| !diagnostics.contains(&format!("### {c} ")))
        .collect();
    assert!(
        missing.is_empty(),
        "the diagnostics chapter has no entry for {missing:?}"
    );
    assert!(codes.len() > 35, "found only {codes:?}");
}

/// The anchor a heading gets on the site and on GitHub, as the generator
/// writes it.
fn slug(text: &str) -> String {
    let text = text.replace('`', "").to_lowercase();
    let mut out = String::new();
    let mut dash = false;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Every link from one chapter to another lands on a chapter that exists,
/// and on a heading it has; every link to a file of the repository finds it.
#[test]
fn every_link_in_the_guide_lands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let all = chapters();
    let anchors = |text: &str| -> Vec<String> {
        text.lines()
            .filter_map(|l| l.strip_prefix("## ").or_else(|| l.strip_prefix("### ")))
            .map(|h| slug(h.trim_start_matches('#').trim()))
            .collect()
    };
    let mut failures = Vec::new();
    let mut docs = all.clone();
    docs.push((
        "README.md".into(),
        std::fs::read_to_string(root.join("md-docs/README.md")).unwrap(),
    ));
    for (name, text) in &docs {
        // Links in code — a Markdown sample, a URL in a snippet — are not
        // the guide's own.
        let mut prose = String::new();
        let mut fenced = false;
        for line in text.lines() {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
                continue;
            }
            if !fenced {
                let mut inline = false;
                for c in line.chars() {
                    if c == '`' {
                        inline = !inline;
                    } else if !inline {
                        prose.push(c);
                    }
                }
                prose.push('\n');
            }
        }
        let mut rest = prose.as_str();
        while let Some(at) = rest.find("](") {
            let after = &rest[at + 2..];
            let Some(end) = after.find(')') else { break };
            let target = &after[..end];
            rest = &after[end..];
            if target.starts_with("http") || target.starts_with("mailto:") || target.contains(' ') {
                continue;
            }
            let (file, anchor) = target.split_once('#').unwrap_or((target, ""));
            if file.is_empty() {
                if !anchor.is_empty() && !anchors(text).contains(&anchor.to_string()) {
                    failures.push(format!("{name}: #{anchor} is not a heading of its own"));
                }
                continue;
            }
            if let Some(repo_path) = file.strip_prefix("../") {
                if !root.join(repo_path).exists() {
                    failures.push(format!("{name}: {file} does not exist"));
                }
                continue;
            }
            match all.iter().find(|(n, _)| n == file) {
                None => failures.push(format!("{name}: {file} is not a chapter")),
                Some((_, target_text)) => {
                    if !anchor.is_empty() && !anchors(target_text).contains(&anchor.to_string()) {
                        failures.push(format!("{name}: {file}#{anchor} is not a heading there"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A link that says "chapter 12" goes to chapter 12: the numbers moved when
/// the guide was reorganised, and a label that kept the old one sent the
/// reader looking in the wrong place.
#[test]
fn a_link_that_names_a_chapter_number_goes_to_that_chapter() {
    let mut failures = Vec::new();
    for (name, text) in chapters() {
        let mut rest = text.as_str();
        while let Some(at) = rest.find("[chapter ").or_else(|| rest.find("[Chapter ")) {
            let after = &rest[at + 9..];
            let number: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Some(close) = after.find("](") {
                let target = &after[close + 2..];
                if target.len() > 2 && target[..2].chars().all(|c| c.is_ascii_digit()) {
                    let file: usize = target[..2].parse().unwrap();
                    if number.parse::<usize>().ok() != Some(file) {
                        failures.push(format!(
                            "{name}: \"chapter {number}\" links to chapter {file}"
                        ));
                    }
                }
            }
            rest = &after[1..];
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every chapter says where it lives on the site and what it is, and says
/// it in a length a search result shows whole.
#[test]
fn every_chapter_says_where_it_lives() {
    let groups = [
        "start",
        "basics",
        "building",
        "shipping",
        "reference",
        "help",
    ];
    let mut routes: Vec<String> = Vec::new();
    let mut failures = Vec::new();
    for (name, text) in chapters() {
        let meta = |key: &str| {
            text.lines()
                .take(10)
                .find_map(|l| l.strip_prefix(&format!("{key}: ")).map(str::to_string))
        };
        let (Some(route), Some(group), Some(blurb), Some(description)) = (
            meta("route"),
            meta("group"),
            meta("blurb"),
            meta("description"),
        ) else {
            failures.push(format!("{name}: its metadata comment is incomplete"));
            continue;
        };
        if !groups.contains(&group.as_str()) {
            failures.push(format!("{name}: `{group}` is not a part of the guide"));
        }
        if description.chars().count() > 158 {
            failures.push(format!(
                "{name}: its description is {} characters; a search result shows about 160",
                description.chars().count()
            ));
        }
        if blurb.is_empty() {
            failures.push(format!("{name}: an empty blurb"));
        }
        if routes.contains(&route) {
            failures.push(format!("{name}: the route {route} is taken"));
        }
        routes.push(route);
        let number: usize = name[..2].parse().unwrap();
        if !text.starts_with(&format!("# {number}. ")) {
            failures.push(format!("{name}: its title is not numbered {number}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The configuration chapter's defaults are the compiler's: the config a
/// project that names only itself is loaded as, written out.
#[test]
fn the_configuration_chapter_prints_the_real_defaults() {
    let text = chapter("-configuration");
    let start = text.find("<!-- defaults -->").expect("the defaults marker");
    let end = text.find("<!-- /defaults -->").expect("its end");
    let block = &text[start..end];
    let json = &block[block.find("```json").unwrap() + 7..block.rfind("```").unwrap()];
    let printed: serde_json::Value = serde_json::from_str(json).expect("the block is JSON");
    let mut real =
        serde_json::to_value(webfluent::config::ProjectConfig::default_config("my-site")).unwrap();
    // Absent unless written: a project without them has none.
    for key in ["i18n", "offline"] {
        real.as_object_mut().unwrap().remove(key);
    }
    assert_eq!(
        printed, real,
        "md-docs/38-configuration.md prints defaults the compiler does not have"
    );
}

/// Everything a program can write is named in the guide, where a reader
/// looks for it: every declaration, every keyword, every browser value and
/// built-in function, every config key, every command and flag, every icon.
#[test]
fn the_guide_names_everything_the_language_has() {
    let mut missing = Vec::new();
    let mut need = |chapter_stem: &str, text: &str, words: &[&str]| {
        for w in words {
            if !text.contains(&format!("`{w}")) {
                missing.push(format!("{chapter_stem} does not name `{w}`"));
            }
        }
    };
    // Declarations, in the chapter that lists them.
    need(
        "language basics",
        &chapter("-language-basics"),
        DECLARATIONS,
    );
    // Every keyword a statement starts with, in the grammar.
    need(
        "grammar",
        &chapter("-grammar"),
        &[
            "state", "persist", "derived", "effect", "cleanup", "action", "use", "resource",
            "validate", "socket", "stream", "channel", "peer", "every", "after", "on key", "head",
            "if", "if let", "for", "show", "match", "sequence", "step", "slot", "event", "part",
            "children", "let", "return", "try", "await", "emit", "expect", "click", "type",
            "press",
        ],
    );
    // Browser values and helpers, in the built-ins reference.
    let built_ins = chapter("-built-ins");
    let values: Vec<&str> = webfluent::codegen::js::BROWSER_VALUES.to_vec();
    need("built-ins", &built_ins, &values);
    let helpers: Vec<&str> = webfluent::codegen::js::LIST_AND_STRING_HELPERS.to_vec();
    need("built-ins", &built_ins, &helpers);
    need(
        "built-ins",
        &built_ins,
        &[
            "log",
            "navigate",
            "format",
            "ago",
            "t(",
            "setLocale",
            "setTheme",
            "uuid",
            "sanitize",
            "fetch",
            "optimistic",
            "beacon",
            "animate",
            "replayAnimation",
            "every",
            "ws(",
            "sse(",
            "broadcast(",
            "rtc(",
        ],
    );
    // Every key of the config, in the configuration reference.
    let config_text = chapter("-configuration");
    let real =
        serde_json::to_value(webfluent::config::ProjectConfig::default_config("my-site")).unwrap();
    fn keys(v: &serde_json::Value, out: &mut Vec<String>) {
        if let Some(map) = v.as_object() {
            for (k, v) in map {
                out.push(k.clone());
                keys(v, out);
            }
        }
    }
    let mut config_keys = Vec::new();
    keys(&real, &mut config_keys);
    for extra in [
        "precache",
        "fallback",
        "cache",
        "sync",
        "default_locale",
        "locales",
        "dir",
    ] {
        config_keys.push(extra.to_string());
    }
    let config_refs: Vec<&str> = config_keys.iter().map(String::as_str).collect();
    need("configuration", &config_text, &config_refs);
    // Every command and its flags, in the CLI reference.
    let cli = chapter("-cli");
    let help = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&help.stdout).to_string();
    let commands: Vec<String> = help
        .lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| l.starts_with("  "))
        .filter_map(|l| l.split_whitespace().next().map(str::to_string))
        .filter(|c| c != "help")
        .collect();
    assert!(commands.len() > 10, "{help}");
    for command in &commands {
        if !cli.contains(&format!("wf {command}")) {
            missing.push(format!("the CLI reference does not name `wf {command}`"));
        }
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
            .args([command.as_str(), "--help"])
            .output()
            .unwrap();
        for flag in String::from_utf8_lossy(&out.stdout).split_whitespace() {
            let flag = flag.trim_end_matches(',');
            if flag.starts_with("--") && flag != "--help" && !cli.contains(flag) {
                missing.push(format!(
                    "the CLI reference does not name `wf {command} {flag}`"
                ));
            }
        }
    }
    // Every icon, in the media chapter.
    let media = chapter("-media");
    for icon in webfluent::registry::ICONS {
        if !media.contains(&format!("`{icon}`")) {
            missing.push(format!("the media chapter does not name the icon `{icon}`"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}
