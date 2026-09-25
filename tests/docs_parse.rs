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

/// Every baseline design token is in the styling chapter's table. A token
/// nobody can find is a token nobody uses, and the table is written by
/// hand — so this is what keeps it from drifting behind `default_tokens`.
#[test]
fn every_design_token_is_documented() {
    let chapter = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("md-docs/12-styling.md"),
    )
    .unwrap();
    let mut missing = Vec::new();
    for name in webfluent::themes::tokens::default_tokens().keys() {
        // A family written as a range (`font-size-xs … font-size-3xl`) or
        // with a slash (`animation-duration-fast/normal/slow`) covers its
        // members: the reader finds them either way.
        let family = name.rsplit_once('-').map(|(head, _)| head).unwrap_or(name);
        if chapter.contains(name.as_str()) || chapter.contains(&format!("{family}-")) {
            continue;
        }
        missing.push(name.clone());
    }
    missing.sort();
    assert!(
        missing.is_empty(),
        "md-docs/12-styling.md does not name: {}",
        missing.join(", ")
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
