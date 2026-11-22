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
    // The fence that opened the block: a block of four ticks holds lines
    // of three.
    let mut ticks = "```";
    for (ix, line) in markdown.lines().enumerate() {
        if let Some((_, body, _)) = &mut current {
            if line.trim_end() == ticks {
                let (start, body, wfx) = current.take().unwrap();
                blocks.push((start + 1, body, wfx));
            } else {
                body.push_str(line);
                body.push('\n');
            }
            continue;
        }
        // A sample: a `wf` or `wfx` block that is neither meant to fail nor
        // marked not to be checked.
        ticks = if line.starts_with("````") {
            "````"
        } else {
            "```"
        };
        match crate::common::fence(line) {
            Some(Ok(f)) if f.is_sample() && f.lang == "wf" => {
                current = Some((ix + 1, String::new(), false));
            }
            Some(Ok(f)) if f.is_sample() && f.lang == "wfx" => {
                current = Some((ix + 1, String::new(), true));
            }
            _ => {}
        }
    }
    blocks
}

const DECLARATIONS: &[&str] = &[
    "api",
    "middleware",
    "fake",
    "example",
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
    // A run of a tutorial project's file, named on its first line
    // (`// src/pages/Home.wf`): `tests/tutorials.rs` holds it to the file,
    // which the project's own build checks whole.
    if block
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("// "))
        .is_some_and(|path| {
            (path.starts_with("src/") || path.starts_with("tests/")) && !path.contains(' ')
        })
    {
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
    let markdown =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    check_strictly_text(path, &markdown)
}

/// [`check_strictly`] of a text: a page of the site, written as Markdown
/// (`common::doc_pages`), named `path` in what it reports.
fn check_strictly_text(path: &str, markdown: &str) -> Vec<String> {
    let mut failures = check_text(path, markdown, None);
    let markdown = markdown.to_string();
    failures.extend(public_env(path, &markdown));
    let scripts = chapter_scripts(&markdown);
    for (line, block, wfx) in wf_blocks(&markdown) {
        if abbreviates(&block) {
            continue;
        }
        let block = if wfx {
            match webfluent::syntax::layout::to_braces(&block, &format!("{path}.wfx")) {
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
        let Ok(mut program) = parse_source(&declarations, path) else {
            continue;
        };
        // What the chapter's own scripts declare is in scope, as it would be
        // in the project the chapter describes.
        program.declarations.extend(scripts.iter().cloned());
        let file_of = |_: usize| format!("{path}:{line}");
        let mut findings = webfluent::sema::check(&program, &file_of);
        let typed = webfluent::sema::types::check(&program, &file_of);
        findings.errors.extend(typed.findings.errors);
        // A template reads its data's keys by name; everything else must
        // declare what it reads.
        // A template reads the data it is rendered with: its names are the
        // data's, which no strict check can see.
        let template = [
            "render-a-template",
            "render-from-rust",
            "render-from-node",
            "TEMPLATE_ENGINE",
        ]
        .iter()
        .any(|t| path.contains(t));
        if !template {
            findings.errors.extend(typed.unresolved);
        }
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

/// The scripts a chapter shows: every ```` ```js ```` block whose first line
/// is `// src/<file>.js`, read as the build reads a script under `src/`.
fn chapter_scripts(markdown: &str) -> Vec<webfluent::syntax::parser::ast::Declaration> {
    let mut out = Vec::new();
    let mut lines = markdown.lines();
    while let Some(line) = lines.next() {
        if line.trim_end() != "```js" {
            continue;
        }
        let body: Vec<&str> = lines
            .by_ref()
            .take_while(|l| l.trim_end() != "```")
            .collect();
        let Some(path) = body.first().and_then(|l| l.strip_prefix("// src/")) else {
            continue;
        };
        let scan = webfluent::project_js::scan::scan(&body.join("\n"));
        assert!(
            scan.problems.is_empty() && scan.module_at.is_none(),
            "src/{path}: {scan:?}"
        );
        // A page shows a script, then the script fixed: the last one shown
        // for a file is the file.
        let path = format!("src/{}", path.trim());
        out.retain(|d| {
            !matches!(d, webfluent::syntax::parser::ast::Declaration::Script(s) if s.path == path)
        });
        // Its `@typedef`s are records the chapter's pages name.
        for def in &scan.typedefs {
            out.push(webfluent::syntax::parser::ast::Declaration::Type(
                webfluent::project_js::typedef_decl(def),
            ));
        }
        out.push(webfluent::syntax::parser::ast::Declaration::Script(
            webfluent::syntax::parser::ast::ScriptDecl {
                path,
                names: scan.names,
            },
        ));
    }
    out
}

/// What the JavaScript for `program` gets wrong, read back from the emission.
///
/// The check that matters here is the one the type checker cannot make: a
/// store compiles to an object of the state and actions it declares, so every
/// `Store.member(…)` the bundle calls has to be one of them.
fn emitted_js_problems(program: &webfluent::syntax::parser::ast::Program) -> Vec<String> {
    use webfluent::syntax::parser::ast::{Declaration, StatementKind};

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
    check_text(path, &markdown, stop)
}

fn check_text(path: &str, markdown: &str, stop: Option<&str>) -> Vec<String> {
    let markdown = match stop.and_then(|stop| markdown.lines().position(|l| l.starts_with(stop))) {
        Some(at) => markdown.lines().take(at).collect::<Vec<_>>().join("\n"),
        None => markdown.to_string(),
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
            match webfluent::syntax::layout::to_braces(&block, &format!("{path}.wfx")) {
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

/// The guide — the site's pages and each code's page: every block is real
/// WebFluent, and builds.
#[test]
fn the_guide_parses() {
    let mut failures = Vec::new();
    for page in crate::common::doc_pages() {
        failures.extend(check_strictly_text(&label(&page), &page.text));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// What a page is called in a finding: its address and the file it is
/// written in. A line number after it counts lines of the page written as
/// Markdown, where its code blocks are fences.
fn label(page: &crate::common::DocPage) -> String {
    format!("/{} ({})", page.name, page.file)
}

/// The page of the documentation at `name` (`reference/config`, `errors`).
fn page(name: &str) -> String {
    crate::common::doc_pages()
        .into_iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("no page at /{name}"))
        .text
}

/// Every route a page of the site declares (`path: "/learn/07-…"`), but a
/// redirect's.
fn declared_routes() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let decl = regex::Regex::new(r#"(?m)^page \w+\(path: "([^"]*)"([^\n]*)"#).unwrap();
    let mut out = Vec::new();
    for path in crate::common::wf_files(&root.join("site/src/pages")) {
        let text = std::fs::read_to_string(&path).unwrap();
        for c in decl.captures_iter(&text) {
            if !c[2].contains("redirect:") {
                out.push(c[1].to_string());
            }
        }
    }
    out
}

/// The book's sidebar, `site/src/components/Guide.wf`, is written by hand
/// beside the chapters it lists, and the two drifted once: chapter 19,
/// Security, was added to the guide and never to the sidebar, so the page
/// built, deployed and answered its URL while being reachable from no
/// navigation at all — and the two chapters after it were left a number
/// short. So every chapter of the book is a row of the sidebar, at its
/// address and under its title, and nothing else is.
#[test]
fn the_sidebar_lists_every_chapter_of_the_guide() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let chapters: Vec<(u32, String, String)> = crate::common::guide_chapters()
        .into_iter()
        .filter(|(_, name, _)| name.starts_with("learn/"))
        .collect();
    assert!(
        chapters.len() == 17,
        "expected the whole book, got {:?}",
        chapters.iter().map(|c| &c.1).collect::<Vec<_>>()
    );
    let sidebar =
        std::fs::read_to_string(root.join("site/src/components/Guide.wf")).expect("Guide.wf");
    let (english, arabic) = sidebar.split_once("const GUIDE_AR").expect("GUIDE_AR");
    let rows = |text: &str| -> Vec<(u32, String, String, String)> {
        let row = regex::Regex::new(
            r#"Chapter\(n: (\d+), title: "([^"]*)", key: "ch\.(\d+)", to: "([^"]*)""#,
        )
        .unwrap();
        row.captures_iter(text)
            .map(|c| {
                (
                    c[1].parse().unwrap(),
                    c[2].to_string(),
                    c[3].to_string(),
                    c[4].to_string(),
                )
            })
            .collect()
    };
    let en = rows(english);
    let ar = rows(arabic);
    let mut failures = Vec::new();
    let tables: Vec<(&str, serde_json::Value)> = ["en", "ar"]
        .iter()
        .map(|l| {
            let text =
                std::fs::read_to_string(root.join(format!("site/src/translations/{l}.json")))
                    .expect("translations");
            (*l, serde_json::from_str(&text).unwrap())
        })
        .collect();
    for (n, name, text) in &chapters {
        let title = crate::common::front_matter(text, "title").unwrap_or_default();
        let key = format!("{n:02}");
        match en.iter().find(|r| r.0 == *n) {
            None => failures.push(format!("chapter {n} (/{name}) is not in the sidebar")),
            Some(r) => {
                if r.1 != title || r.2 != key || r.3 != format!("/{name}") {
                    failures.push(format!(
                        "the sidebar's row {r:?} is not chapter {n}, {title:?} at /{name}"
                    ));
                }
            }
        }
        // The Arabic book: each chapter at its translation when it has one.
        let translated = crate::common::doc_pages()
            .into_iter()
            .any(|p| p.name == format!("ar/{name}"));
        let want = if translated {
            format!("/ar/{name}")
        } else {
            format!("/{name}")
        };
        if !ar.iter().any(|r| r.0 == *n && r.3 == want) {
            failures.push(format!("GUIDE_AR has no row for chapter {n} at {want}"));
        }
        // Both locales carry every title, or the rail is blank in one; the
        // English one is the page's.
        for (locale, table) in &tables {
            match table[format!("ch.{key}")].as_str() {
                None => failures.push(format!(
                    "{locale}.json has no title for chapter {n} (/{name})"
                )),
                Some(t) if *locale == "en" && t != title => {
                    failures.push(format!(
                        "en.json names chapter {n} {t:?}; the page says {title:?}"
                    ));
                }
                Some(_) => {}
            }
        }
    }
    // And nothing beyond them.
    for r in en.iter().chain(&ar) {
        if !chapters.iter().any(|(n, _, _)| *n == r.0) {
            failures.push(format!(
                "the sidebar names chapter {}, which the book does not have",
                r.0
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The lists written by hand beside the pages they list hold every one of
/// them, and nothing else: every recipe is in `RECIPE_INDEX` (the recipes'
/// index and sidebar) under its own title; every chapter of the book and
/// essay is on its part's contents page (`/learn`, `/about`); a chapter's
/// previous and next pages are pages; every page a chapter used to be at
/// sends the reader on to one; and the chapter numbers are each a file's
/// once.
#[test]
fn the_hand_written_lists_hold_every_page() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pages = crate::common::doc_pages();
    let routes = declared_routes();
    let mut failures = Vec::new();
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();

    let index = read("site/src/pages/recipes/Index.wf");
    let entry = regex::Regex::new(r#"IndexEntry\(label: "([^"]*)", to: "([^"]*)""#).unwrap();
    let listed: Vec<(String, String)> = entry
        .captures_iter(&index)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect();
    for p in pages.iter().filter(|p| p.name.starts_with("recipes/")) {
        let title = crate::common::front_matter(&p.text, "title").unwrap_or_default();
        if !listed
            .iter()
            .any(|(l, to)| *to == format!("/{}", p.name) && *l == title)
        {
            failures.push(format!(
                "RECIPE_INDEX does not list /{} as {title:?}",
                p.name
            ));
        }
    }
    for (label, to) in &listed {
        if !routes.contains(to) {
            failures.push(format!(
                "RECIPE_INDEX lists {label:?} at {to}, which is no recipe"
            ));
        }
    }

    for (part, file) in [
        ("learn", "site/src/pages/guide/Learn.wf"),
        ("about", "site/src/pages/guide/About.wf"),
    ] {
        let text = read(file);
        for p in pages.iter().filter(|p| {
            p.name.starts_with(&format!("{part}/"))
                && crate::common::front_matter(&p.text, "chapter").is_some()
        }) {
            if !text.contains(&format!("to: \"/{}\"", p.name)) {
                failures.push(format!("{file} does not link /{}", p.name));
            }
        }
    }

    // A chapter's neighbours, as its shell links them.
    let to = regex::Regex::new(r#"(?:prevTo|nextTo|other): "([^"]*)""#).unwrap();
    let mut numbers: Vec<(u32, String)> = Vec::new();
    for path in crate::common::wf_files(&root.join("site/src/pages/guide")) {
        let text = std::fs::read_to_string(&path).unwrap();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        for c in to.captures_iter(&text) {
            if !routes.contains(&c[1].to_string()) {
                failures.push(format!("{file}: links {}, which is no page", &c[1]));
            }
        }
        if !file.ends_with("-ar.wf")
            && let Some(n) = file.get(..2).and_then(|n| n.parse::<u32>().ok())
        {
            if let Some((_, other)) = numbers.iter().find(|(m, _)| *m == n) {
                failures.push(format!("{file} and {other} are both chapter {n}"));
            }
            numbers.push((n, file));
        }
    }

    // Each old address goes to a page.
    let moved = read("site/src/old-links-pages.js");
    let pair = regex::Regex::new(r#""(/[^"]*)": "(/[^"]*)""#).unwrap();
    let redirects = read("site/src/pages/guide/Moved.wf");
    for c in pair.captures_iter(&moved) {
        if !routes.contains(&c[2].to_string()) {
            failures.push(format!(
                "old-links-pages.js sends {} to {}, which is no page",
                &c[1], &c[2]
            ));
        }
        if !redirects.contains(&format!("path: \"{}\"", &c[1])) {
            failures.push(format!("{} has no redirect in pages/guide/Moved.wf", &c[1]));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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

/// The documents whose blocks the compiler is held to, as above: each
/// one's name, its Markdown, and where to stop reading it.
fn corpus_documents() -> Vec<(String, String, Option<&'static str>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut docs: Vec<(String, String, Option<&'static str>)> = [
        ("AGENTS.md", None),
        ("README.md", None),
        ("RELEASE_NOTES.md", Some("# WebFluent v2.2")),
    ]
    .into_iter()
    .map(|(p, stop)| {
        (
            p.to_string(),
            std::fs::read_to_string(root.join(p)).unwrap(),
            stop,
        )
    })
    .collect();
    // The site's pages by address (`site/learn/07-state-and-reactivity`),
    // each code's page by its code (`errors/T05`). A built-in's page gives
    // its example, the one `scripts/docsite/examples.py` names first; the
    // variants below it are held to the compiler, not to the grammar.
    for page in crate::common::doc_pages() {
        let name = if page.name.starts_with("errors/") {
            page.name.clone()
        } else {
            format!("site/{}", page.name)
        };
        let text = if page.file.ends_with("/Builtins.wf") {
            let mut blocks = page.text.split("\n```wf\n");
            let _ = blocks.next();
            match blocks.next().and_then(|b| b.split_once("\n```")) {
                Some((code, _)) => format!("```wf\n{code}\n```\n"),
                None => continue,
            }
        } else {
            page.text
        };
        docs.push((name, text, None));
    }
    docs
}

/// The corpus files for one document: its blocks as the compiler accepts
/// them, braced (for `.wf`), and its indented blocks that are whole
/// declarations as written (for `.wfx`).
fn corpus_for(path: &str, markdown: &str, stop: Option<&str>) -> (String, String) {
    let markdown = match stop.and_then(|stop| markdown.lines().position(|l| l.starts_with(stop))) {
        Some(at) => markdown.lines().take(at).collect::<Vec<_>>().join("\n"),
        None => markdown.to_string(),
    };
    let mut wf = String::new();
    let mut wfx = String::new();
    for (_, block, indented) in wf_blocks(&markdown) {
        if abbreviates(&block) {
            continue;
        }
        let braced = if indented {
            match webfluent::syntax::layout::to_braces(&block, &format!("{path}.wfx")) {
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
            declarations
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

/// The grammar's corpus is what the guide's blocks are now. The grammars
/// are the `editors/tree-sitter-webfluent` submodule; without it checked
/// out there is nothing to compare, and CI checks it out. After the guide
/// changes, run with `WF_WRITE_GUIDE_CORPUS=1`, then commit the corpus in
/// the submodule and bump it here.
#[test]
fn the_editor_grammar_corpus_is_current_with_the_guide() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let grammars = root.join("editors/tree-sitter-webfluent");
    if !grammars.join("tree-sitter.json").exists() {
        eprintln!(
            "skipped: the grammar submodule is not checked out \
             (git submodule update --init editors/tree-sitter-webfluent)"
        );
        return;
    }
    let wf_dir = grammars.join("webfluent/test/guide");
    let wfx_dir = grammars.join("webfluentx/test/guide");
    let write = std::env::var_os("WF_WRITE_GUIDE_CORPUS").is_some();
    if write {
        for dir in [&wf_dir, &wfx_dir] {
            let _ = std::fs::remove_dir_all(dir);
            std::fs::create_dir_all(dir).unwrap();
        }
    }
    let mut stale = Vec::new();
    let mut expected_files = Vec::new();
    for (path, text, stop) in corpus_documents() {
        let (wf, wfx) = corpus_for(&path, &text, stop);
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
         `WF_WRITE_GUIDE_CORPUS=1 cargo test --test it -- docs_parse::the_editor_grammar_corpus` \
         and commit it in the submodule: {stale:?}"
    );
}

/// The documentation site's search and its reference pages are written by
/// `scripts/docsite/data.py` and `scripts/docsite/reference.py`, from the
/// site's pages and from `wf registry/commands/config schema/theme --json` (with
/// the data beside them for tools), and
/// nothing held the committed files to what they write — so a section added
/// to the guide was missing from the site's search until someone remembered
/// to rerun it. This runs both into a scratch directory, with the binary
/// this test run just built, and compares.
#[test]
fn the_sites_search_and_reference_data_are_current() {
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
    for (script, var) in [
        ("data.py", "WF_SITE_DATA_OUT"),
        ("reference.py", "WF_REFERENCE_OUT"),
    ] {
        let run = std::process::Command::new("python3")
            .arg(root.join("scripts/docsite").join(script))
            .env(var, &out)
            .env("WF_BIN", env!("CARGO_BIN_EXE_wf"))
            .output()
            .expect("run the generator");
        assert!(
            run.status.success(),
            "{script} failed: {}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
    let mut stale = Vec::new();
    for (name, committed) in [
        ("search-index.json", "site/public/search-index.json"),
        ("components.json", "site/src/data/components.json"),
        ("commands.json", "site/src/data/commands.json"),
        ("config.json", "site/src/data/config.json"),
        ("tokens.json", "site/src/data/tokens.json"),
        ("Builtins.wf", "site/src/pages/reference/Builtins.wf"),
        ("Cli.wf", "site/src/pages/reference/Cli.wf"),
        (
            "FromTheCompiler.wf",
            "site/src/pages/reference/FromTheCompiler.wf",
        ),
    ] {
        let want = std::fs::read_to_string(out.join(name)).unwrap();
        let have = std::fs::read_to_string(root.join(committed)).unwrap_or_default();
        if want != have {
            stale.push(name);
        }
    }
    let _ = std::fs::remove_dir_all(&out);
    assert!(
        stale.is_empty(),
        "these are behind the guide or the compiler; run `cargo xtask site`: {stale:?}"
    );
}

// ─── The guide, held to the compiler ────────────────────────────────────
//
// What the guide says exists is what the compiler has: every example that
// claims a diagnostic draws it, every link lands, every declaration, keyword,
// built-in, browser value, function, config key, command and code is named
// somewhere a reader will find it, and the defaults it prints are the
// compiler's own.

/// The guide's chapters, in chapter order: `(name, text)`.
fn chapters() -> Vec<(String, String)> {
    crate::common::guide_chapters()
        .into_iter()
        .map(|(_, path, text)| (path, text))
        .collect()
}

/// The page of the guide whose name holds `stem`: a chapter or a recipe.
fn chapter(stem: &str) -> String {
    crate::common::guide_pages()
        .into_iter()
        .find(|(n, _)| n.contains(stem))
        .unwrap_or_else(|| panic!("no chapter named like {stem}"))
        .1
}

/// Every finding the build would report for `src`, through the build's own
/// pipeline, as if it were saved as `file`: `src/App.wfx` reads it in the
/// indented layout.
/// The ```` ```js ```` blocks shown above line `at` of a page, each whose
/// first line is `// src/<file>.js`: the scripts an example below them has,
/// the last one shown for a file winning — a page shows a script, then the
/// script fixed.
fn scripts_above(text: &str, at: usize) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut lines = text.lines().take(at);
    while let Some(line) = lines.next() {
        if line.trim_end() != "```js" {
            continue;
        }
        let body: Vec<&str> = lines
            .by_ref()
            .take_while(|l| l.trim_end() != "```")
            .collect();
        if let Some(path) = body.first().and_then(|l| l.strip_prefix("// src/")) {
            let path = format!("src/{}", path.trim());
            out.retain(|(p, _)| *p != path);
            out.push((path, body.join("\n")));
        }
    }
    out
}

/// [`findings_in`], with the project's scripts beside the example.
fn findings_with(
    src: &str,
    file: &str,
    scripts: &[(String, String)],
) -> Vec<webfluent::diagnostics::Diagnostic> {
    let (mut program, mut out) = webfluent::syntax::parse_source_recovering(src, file);
    for (path, body) in scripts {
        let scan = webfluent::project_js::scan::scan(body);
        for def in &scan.typedefs {
            program
                .declarations
                .push(webfluent::syntax::parser::ast::Declaration::Type(
                    webfluent::project_js::typedef_decl(def),
                ));
        }
        program
            .declarations
            .push(webfluent::syntax::parser::ast::Declaration::Script(
                webfluent::syntax::parser::ast::ScriptDecl {
                    path: path.clone(),
                    names: scan.names,
                },
            ));
    }
    let incomplete = !out.is_empty();
    let source = src.to_string();
    let checked =
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| file.to_string(),
            source_of: &|_| Some(source.clone()),
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete,
        });
    out.extend(checked.diagnostics);
    out
}

/// Each example's ```` ```text ```` block is what the compiler prints for
/// it — rendered by the build's own renderer — so the guide cannot show a
/// message the compiler no longer writes. `WF_WRITE_DIAGNOSTICS=1` writes
/// them.
#[test]
fn every_diagnostic_example_shows_what_the_build_prints() {
    let write = std::env::var_os("WF_WRITE_DIAGNOSTICS").is_some();
    let mut stale = Vec::new();
    for c in webfluent::diagnostics::codes::CODES {
        // The file as it is written: what the writer rewrites.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(webfluent::diagnostics::codes::EXPLAIN_DIR)
            .join(format!("{}.md", c.code));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        stale.extend(shows_what_the_build_prints(&path, &text, write));
    }
    assert!(
        stale.is_empty(),
        "src/diagnostics/explain/ shows output the compiler no longer prints for {stale:?}; \
         rewrite it with `WF_WRITE_DIAGNOSTICS=1 cargo test --test it -- docs_parse::every_diagnostic_example_shows`"
    );
}

/// One page's `expect=` examples, held to what the build prints for them:
/// the codes whose shown output is stale, after writing the page afresh
/// when `write` says to.
fn shows_what_the_build_prints(path: &Path, text: &str, write: bool) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut stale = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        out.push(lines[i].to_string());
        let Some((code, file)) = crate::common::fence(lines[i])
            .and_then(Result::ok)
            .and_then(|f| f.expect.map(|c| (c, example_file(&f.lang))))
        else {
            i += 1;
            continue;
        };
        let mut body = String::new();
        i += 1;
        while i < lines.len() && lines[i].trim_end() != "```" {
            body.push_str(lines[i]);
            body.push('\n');
            out.push(lines[i].to_string());
            i += 1;
        }
        out.push("```".to_string());
        i += 1;
        let scripts = scripts_above(text, i);
        let Some(finding) = findings_with(&body, file, &scripts)
            .into_iter()
            .find(|d| d.code == code)
        else {
            continue;
        };
        let want = webfluent::diagnostics::render::diagnostic(&finding, Some(&body), false);
        // The block that follows, if there is one: a blank line, then ```text.
        let has_block = lines.get(i).is_some_and(|l| l.is_empty())
            && lines.get(i + 1).is_some_and(|l| l.trim_end() == "```text");
        let mut shown = String::new();
        if has_block {
            let mut j = i + 2;
            while j < lines.len() && lines[j].trim_end() != "```" {
                shown.push_str(lines[j]);
                shown.push('\n');
                j += 1;
            }
            i = j + 1;
        }
        if shown.trim_end() != want.trim_end() {
            stale.push(code);
        }
        out.push(String::new());
        out.push("```text".to_string());
        out.extend(want.lines().map(str::to_string));
        out.push("```".to_string());
    }
    if write && !stale.is_empty() {
        std::fs::write(path, format!("{}\n", out.join("\n"))).unwrap();
        return Vec::new();
    }
    stale
}

/// A ```` ```wf expect=T04 ```` block is a program that draws `T04` on
/// purpose. Every one of them draws exactly the code it is filed under —
/// and every code the compiler has is filed somewhere.
#[test]
fn every_diagnostic_example_draws_its_code() {
    fn walk(dir: &Path, named: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, named);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let text = std::fs::read_to_string(&p).unwrap();
                for piece in text.split('"').skip(1).step_by(2) {
                    let b = piece.as_bytes();
                    let looks = (piece.len() == 3 || piece.len() == 4)
                        && b"ETCRFXIDASPUV".contains(&b[0])
                        && b[1..].iter().all(u8::is_ascii_digit);
                    if looks && !named.contains(&piece.to_string()) {
                        named.push(piece.to_string());
                    }
                }
            }
        }
    }

    let mut failures = Vec::new();
    let mut shown: Vec<String> = Vec::new();
    for (name, text) in crate::common::guide_pages() {
        let mut lines = text.lines().enumerate();
        while let Some((at, line)) = lines.next() {
            let Some((code, file)) = crate::common::fence(line)
                .and_then(Result::ok)
                .and_then(|f| f.expect.map(|c| (c, example_file(&f.lang))))
            else {
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
            let found: Vec<String> = findings_with(&body, file, &scripts_above(&text, at))
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            if !found.iter().any(|f| f.contains(&format!("[{code}]"))) {
                failures.push(format!(
                    "/{name}:{}: does not draw {code}; it draws {found:?}",
                    at + 1
                ));
            }
            shown.push(code);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));

    // Every code the registry holds has a page (its explanation,
    // `src/diagnostics/explain/CODE.md`) and is listed on the diagnostics
    // chapter, `/errors`, under its family; every code the compiler's
    // source names is registered.
    let diagnostics = page("errors");
    let pages = crate::common::guide_pages();
    let registered: Vec<&str> = webfluent::diagnostics::codes::CODES
        .iter()
        .map(|c| c.code)
        .collect();
    let missing: Vec<&&str> = registered
        .iter()
        .filter(|c| {
            !pages.iter().any(|(n, _)| *n == format!("errors/{c}"))
                || !diagnostics.contains(&format!("- [{c}](/errors/{c})"))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "these codes have no page in src/diagnostics/explain/, or are not listed on /errors: {missing:?}"
    );
    let undocumented_examples: Vec<&&str> = registered
        .iter()
        .filter(|c| !shown.iter().any(|s| s == **c))
        .filter(|c| !EXAMPLE_EXEMPT.contains(c))
        .collect();
    assert!(
        undocumented_examples.is_empty(),
        "these codes have no ```wf expect=…``` example in the guide: {undocumented_examples:?}"
    );

    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut named: Vec<String> = Vec::new();
    walk(&src_dir, &mut named);
    let unregistered: Vec<&String> = named
        .iter()
        .filter(|c| !registered.contains(&c.as_str()))
        .collect();
    assert!(
        unregistered.is_empty(),
        "the source names codes the registry does not hold: {unregistered:?}"
    );
    assert!(registered.len() > 50, "found only {registered:?}");
}

/// Codes no single program in the guide can draw: a file the formatter
/// refuses, the compiler's own faults, and what only a project around the
/// program — its config, its scripts, its output mode — can show.
const EXAMPLE_EXEMPT: &[&str] = &[
    "N07", "E005", "E108", "E109", "E110", "E111", "E112", "E115", "E119", "E901", "E902", "D05",
    "D06", "D01", "D04", "I01", "I02", "I03", "I04", "I05", "A13", "V02",
    // E120 needs a hundred levels of nesting to show, and E121 is a
    // template render's, which no check draws.
    "E120", "E121",
    // T33 is a data file's, a render's or a test run's: data from outside
    // the program, which one file checked on its own has none of.
    "T33",
];

/// Where a link written on the guide page `from` goes, as an address on the
/// site (`/learn/07-state-and-reactivity`, with its `#anchor`), or `None`
/// when it leaves the site. The site's pages link by address; a code's page
/// (its explanation, which `wf explain` prints too) links its neighbours by
/// file — `T15.md`, `diagnostics.md#…`, `../recipes/x.md` — which the error
/// pages' writer turns into the same addresses.
fn link_route(from: &str, link: &str) -> Option<String> {
    if link.starts_with("http") || link.starts_with("mailto:") {
        return None;
    }
    if link.starts_with('/') || link.starts_with('#') || !from.starts_with("errors/") {
        return Some(link.to_string());
    }
    let (file, anchor) = link.split_once('#').map_or((link, ""), |(f, a)| (f, a));
    let anchor = if anchor.is_empty() {
        String::new()
    } else {
        format!("#{anchor}")
    };
    let file = file.trim_end_matches(".md");
    let route = if file == "diagnostics" {
        "/errors".to_string()
    } else if let Some(rest) = file.strip_prefix("../") {
        format!("/{}", rest.trim_end_matches("/index"))
    } else {
        format!("/errors/{file}")
    };
    Some(format!("{route}{anchor}"))
}

/// The links a page's prose makes, outside its code.
fn prose_links(text: &str) -> Vec<String> {
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
    let mut out = Vec::new();
    let mut rest = prose.as_str();
    while let Some(at) = rest.find("](") {
        let after = &rest[at + 2..];
        let Some(end) = after.find(')') else { break };
        let target = &after[..end];
        rest = &after[end..];
        if !target.contains(' ') && !target.is_empty() {
            out.push(target.to_string());
        }
    }
    out
}

/// Every link from one page of the guide to another lands on a page the
/// site has, and on a heading or a section that page has.
#[test]
fn every_link_in_the_guide_lands() {
    let pages = crate::common::doc_pages();
    let routes = declared_routes();
    let mut failures = Vec::new();
    for page in &pages {
        for target in prose_links(&page.text) {
            let Some(route) = link_route(&page.name, &target) else {
                continue;
            };
            let (path, anchor) = route
                .split_once('#')
                .map_or((route.as_str(), ""), |(p, a)| (p, a));
            let path = path.trim_end_matches('/');
            if path.is_empty() && route.starts_with('#') {
                if !page.anchors.contains(&anchor.to_string()) {
                    failures.push(format!(
                        "/{}: #{anchor} is not a heading of its own",
                        page.name
                    ));
                }
                continue;
            }
            let path = if path.is_empty() { "/" } else { path };
            // A file the site serves (`/llms.txt`, a picture) is not a page.
            if path.contains('.') {
                continue;
            }
            match pages.iter().find(|p| format!("/{}", p.name) == path) {
                Some(target_page) => {
                    if !anchor.is_empty() && !target_page.anchors.contains(&anchor.to_string()) {
                        failures.push(format!("/{}: {route} is not a heading there", page.name));
                    }
                }
                None if routes.iter().any(|r| r == path) => {}
                None => failures.push(format!(
                    "/{}: {target} is not a page of the site",
                    page.name
                )),
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
    let numbered = crate::common::guide_chapters();
    let mut failures = Vec::new();
    for (name, text) in chapters() {
        let mut rest = text.as_str();
        while let Some(at) = rest.find("[chapter ").or_else(|| rest.find("[Chapter ")) {
            let after = &rest[at + 9..];
            let number: String = after.chars().take_while(char::is_ascii_digit).collect();
            if let Some(close) = after.find("](") {
                let target = &after[close + 2..];
                let target = &target[..target.find([')', '#']).unwrap_or(target.len())];
                let to = numbered
                    .iter()
                    .find(|(_, n, _)| format!("/{n}") == target)
                    .map(|(n, _, _)| *n as usize);
                if let Some(file) = to
                    && number.parse::<usize>().ok() != Some(file)
                {
                    failures.push(format!(
                        "/{name}: \"chapter {number}\" links to chapter {file}"
                    ));
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
    let groups = ["tour", "language", "build", "about", "reference", "help"];
    let mut failures = Vec::new();
    for (name, text) in chapters() {
        let meta = |key: &str| crate::common::front_matter(&text, key);
        let (Some(group), Some(blurb), Some(description)) =
            (meta("group"), meta("blurb"), meta("description"))
        else {
            failures.push(format!(
                "/{name}: its shell names no part, blurb or description"
            ));
            continue;
        };
        if !groups.contains(&group.as_str()) {
            failures.push(format!("/{name}: `{group}` is not a part of the guide"));
        }
        if description.chars().count() > 158 {
            failures.push(format!(
                "/{name}: its description is {} characters; a search result shows about 160",
                description.chars().count()
            ));
        }
        if blurb.is_empty() {
            failures.push(format!("/{name}: an empty blurb"));
        }
        // A page lives in the part of the site its kind is.
        let kind = meta("kind").unwrap_or_default();
        let folder = match kind.as_str() {
            "tutorial" => "learn",
            "explanation" => "about",
            "reference" => "reference",
            "error" => "errors",
            other => {
                failures.push(format!("/{name}: `kind: .{other}` is not a chapter's"));
                continue;
            }
        };
        if name != folder && !name.starts_with(&format!("{folder}/")) {
            failures.push(format!("/{name}: a .{kind} page belongs under /{folder}/"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The configuration chapter is the site's only listing of the settings:
/// every key `wf config schema` has is in one of its tables, under its own
/// name or its section's (`build.pdf` lists `page_size`).
#[test]
fn the_configuration_chapter_lists_every_setting() {
    let text = page("reference/config");
    let mut section = String::new();
    let mut listed: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(h) = line
            .strip_prefix("## ")
            .or_else(|| line.strip_prefix("### "))
        {
            // A section's own heading lists it: `## \`lints\``.
            section = h.trim().trim_matches('`').to_string();
            listed.push(section.clone());
            continue;
        }
        // A row may name several keys: `| \`width\`, \`height\` | …`.
        if let Some(first) = line.strip_prefix('|').and_then(|r| r.split('|').next()) {
            for key in first.split(',').map(|k| k.trim().trim_matches('`')) {
                if !key.is_empty() && !key.contains(' ') {
                    listed.push(key.to_string());
                    listed.push(format!("{section}.{key}"));
                }
            }
        }
    }
    let missing: Vec<&str> = webfluent::config::schema::FIELDS
        .iter()
        .map(|f| f.path)
        .filter(|path| {
            let mut parents = vec![*path];
            // `build.pdf.margins.top` is listed as `margins` under `build.pdf`.
            let mut at = *path;
            while let Some((head, _)) = at.rsplit_once('.') {
                parents.push(head);
                at = head;
            }
            !parents.iter().any(|p| listed.iter().any(|l| l == p))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "/reference/config lists no row for these settings: {missing:?}"
    );
}

/// The configuration chapter's defaults are the compiler's: the config a
/// project that names only itself is loaded as, written out.
#[test]
fn the_configuration_chapter_prints_the_real_defaults() {
    let text = page("reference/config");
    // The JSON block of its section "The defaults".
    let start = text
        .find("\n## The defaults\n")
        .expect("the defaults section");
    let block = &text[start..];
    let open = block.find("```json\n").expect("the defaults' JSON") + 8;
    let json = &block[open..open + block[open..].find("\n```").unwrap()];
    let printed: serde_json::Value = serde_json::from_str(json).expect("the block is JSON");
    let mut real =
        serde_json::to_value(webfluent::config::ProjectConfig::default_config("my-site")).unwrap();
    // Absent unless written: a project without them has none.
    for key in ["i18n", "offline"] {
        real.as_object_mut().unwrap().remove(key);
    }
    assert_eq!(
        printed, real,
        "/reference/config prints defaults the compiler does not have"
    );
}

/// Everything a program can write is named in the guide, where a reader
/// looks for it: every declaration, every keyword, every browser value and
/// built-in function, every config key, every command and flag, every icon.
#[test]
fn the_guide_names_everything_the_language_has() {
    fn keys(v: &serde_json::Value, out: &mut Vec<String>) {
        if let Some(map) = v.as_object() {
            for (k, v) in map {
                out.push(k.clone());
                keys(v, out);
            }
        }
    }

    let mut missing = Vec::new();
    let mut need = |chapter_stem: &str, text: &str, words: &[&str]| {
        for w in words {
            // Named in code, or as a heading of its own (`## dev`).
            let heading = text
                .lines()
                .any(|l| l.trim_start_matches('#').trim() == *w && l.starts_with("##"));
            if !text.contains(&format!("`{w}")) && !heading {
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
        &page("reference/grammar"),
        &[
            "state", "persist", "derived", "effect", "cleanup", "action", "use", "resource",
            "validate", "socket", "stream", "channel", "peer", "every", "after", "on key", "head",
            "if", "if let", "for", "show", "match", "sequence", "step", "slot", "event", "part",
            "children", "let", "return", "try", "await", "emit", "expect", "click", "type",
            "press",
        ],
    );
    // Browser values and helpers, in the built-ins reference.
    let built_ins = page("reference/built-ins");
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
    let config_text = page("reference/config");
    let real =
        serde_json::to_value(webfluent::config::ProjectConfig::default_config("my-site")).unwrap();
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
    // Every command, with words of its own on its page (`Cli.wf`'s part
    // `// ── words: build ──`, written by hand), and every flag, on the
    // command's page (from `wf commands`).
    let cli = page("reference/cli");
    let cli_pages = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("site/src/pages/reference/Cli.wf"),
    )
    .unwrap();
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
        let words = cli_pages
            .split_once(&format!("// ── words: {command} ──"))
            .and_then(|(_, rest)| rest.split_once("// ── end of the words ──"))
            .map(|(part, _)| part.lines().skip(1).any(|l| !l.trim().is_empty()));
        if words != Some(true) {
            missing.push(format!("Cli.wf has no words of its own for `wf {command}`"));
        }
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
            .args([command.as_str(), "--help"])
            .output()
            .unwrap();
        for flag in String::from_utf8_lossy(&out.stdout).split_whitespace() {
            let flag = flag.trim_end_matches(',');
            if flag.starts_with("--") && flag != "--help" && !cli_pages.contains(flag) {
                missing.push(format!(
                    "the CLI reference does not name `wf {command} {flag}`"
                ));
            }
        }
    }
    // Every icon, in the media chapter.
    let media = chapter("/show-an-icon");
    for icon in webfluent::registry::ICONS {
        if !media.contains(&format!("`{icon}`")) {
            missing.push(format!("the icon recipe does not name the icon `{icon}`"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

/// The reference's tables teach by inline code, which no fence check reads, and
/// an agent is given them. `Row(gap: md)` read as a name, `md`, and is `T13`.
#[test]
fn the_references_layout_examples_compile() {
    let reference = include_str!("../AGENTS.md");
    let mut checked = 0;
    for line in reference.lines() {
        let row = ["| `Row` |", "| `Grid` |", "| `Stack` |"]
            .iter()
            .any(|h| line.starts_with(h));
        if !row {
            continue;
        }
        let example = line
            .split('`')
            .nth(3)
            .expect("an example")
            .replace("{ ... }", "{ }");
        let dir = std::env::temp_dir().join(format!("wf-layout-{}-{checked}", std::process::id()));
        std::fs::create_dir_all(dir.join("src/pages")).unwrap();
        std::fs::write(dir.join("webfluent.app.json"), r#"{"name":"t"}"#).unwrap();
        let page = format!(
            "page A(path: \"/\", title: \"t\", description: \"d\") {{\n    Heading(\"x\").h1\n    {example}\n}}\n"
        );
        std::fs::write(dir.join("src/pages/A.wf"), &page).unwrap();
        let vfs = webfluent::vfs::OverlayVfs::over_disk();
        let options = webfluent::build::BuildOptions::new(&dir, &vfs);
        let report = match webfluent::build::check(&options) {
            Ok(r) => r,
            Err(f) => *f.report,
        };
        let errors: Vec<String> = report
            .errors()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(errors.is_empty(), "{example}: {errors:?}");
        checked += 1;
    }
    assert_eq!(checked, 3, "the three layout rows");
}

/// The skill any agent installs to write WebFluent
/// (`skills/writing-webfluent/`): its examples build, its reference is this
/// version's, and every command and diagnostic code it names is one `wf` has.
/// After a change to `AGENTS.md`, write the reference's section files again:
/// `WF_BLESS_SKILL=1 cargo test --test it the_writing_webfluent_skill`.
#[test]
fn the_writing_webfluent_skill_is_true_of_this_version() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = "skills/writing-webfluent/SKILL.md";
    let failures = check_strictly(path);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    let skill = std::fs::read_to_string(root.join(path)).unwrap();
    let agents = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();

    // The reference, a file a section, as `wf init` writes it: the same
    // function, so the skill an agent installs from the repository and the
    // one a new project gets are one.
    let skill_dir = root.join("skills/writing-webfluent");
    let sections = webfluent::init::reference_files(&agents);
    if std::env::var_os("WF_BLESS_SKILL").is_some() {
        let _ = std::fs::remove_dir_all(skill_dir.join("references/language"));
        for (rel, text) in &sections {
            let p = skill_dir.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
    }
    let on_disk: std::collections::BTreeSet<String> = walk(&skill_dir.join("references/language"))
        .iter()
        .map(|p| {
            format!(
                "references/language/{}",
                p.file_name().unwrap().to_string_lossy()
            )
        })
        .collect();
    let wanted: std::collections::BTreeSet<String> = sections
        .iter()
        .map(|(rel, _)| rel.clone())
        .filter(|rel| rel.starts_with("references/language/"))
        .collect();
    assert_eq!(
        on_disk, wanted,
        "the reference's sections aren't this AGENTS.md's: run with WF_BLESS_SKILL=1"
    );
    for (rel, text) in &sections {
        let have = std::fs::read_to_string(skill_dir.join(rel)).unwrap_or_default();
        assert!(
            &have == text,
            "{rel} isn't this AGENTS.md's: run with WF_BLESS_SKILL=1"
        );
    }
    // Each section is one an agent can afford to read.
    for (rel, text) in &sections {
        assert!(
            text.len() < 40_000,
            "{rel} is {} bytes: split it",
            text.len()
        );
    }

    // The standard's frontmatter: the name is the folder's, the description
    // under 1,024 characters, and the version this one.
    let front = skill
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .map(|(f, _)| f)
        .expect("frontmatter");
    let field = |name: &str| {
        front
            .lines()
            .find_map(|l| l.trim_start().strip_prefix(&format!("{name}: ")))
            .unwrap_or_default()
    };
    assert_eq!(field("name"), "writing-webfluent");
    let description = field("description");
    assert!(
        !description.is_empty() && description.chars().count() <= 1024,
        "{description}"
    );
    assert_eq!(
        field("version").trim_matches('"'),
        webfluent::VERSION,
        "metadata.version"
    );
    assert!(
        skill.contains(&format!("WebFluent {}", webfluent::VERSION)),
        "the body names this version"
    );

    // Every `wf <command>` it names is one the reference's CLI lists.
    let commands: std::collections::BTreeSet<&str> = agents
        .lines()
        .filter_map(|l| l.strip_prefix("wf "))
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    let mut named = Vec::new();
    for (i, _) in skill.match_indices("`wf ") {
        let word: String = skill[i + 4..]
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || *c == '-')
            .collect();
        if !word.is_empty() && !word.starts_with('-') {
            named.push(word);
        }
    }
    assert!(named.len() > 10, "{named:?}");
    for word in &named {
        assert!(
            commands.contains(word.as_str()),
            "`wf {word}` isn't a command: {commands:?}"
        );
    }

    // Every diagnostic code it names is the compiler's.
    let mut codes = 0;
    for (i, _) in skill.match_indices('`') {
        let rest = &skill[i + 1..];
        let Some(end) = rest.find('`') else { continue };
        let code = &rest[..end];
        let looks = code.len() >= 3
            && code.len() <= 4
            && code.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && code.chars().skip(1).all(|c| c.is_ascii_digit());
        if looks {
            assert!(
                webfluent::diagnostics::codes::info(code).is_some(),
                "`{code}` isn't a code `wf explain` knows"
            );
            codes += 1;
        }
    }
    assert!(codes > 5, "{codes} codes");
}

/// `wf init` gives every new project the skill, in `.agents/skills/` and
/// `.claude/skills/`: every file of `skills/writing-webfluent/` as it is,
/// the reference's section files included. A reference added to the skill
/// and not to `init` would be missing from every project made after it.
#[test]
fn every_new_project_has_the_skill() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let skill = root.join("skills/writing-webfluent");
    let mut wanted = Vec::new();
    for entry in walk(&skill) {
        let rel = entry
            .strip_prefix(&skill)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        wanted.push(rel);
    }
    wanted.sort();
    for template in webfluent::init::Template::ALL {
        let dir = std::env::temp_dir().join(format!(
            "wf-skill-{}-{}",
            std::process::id(),
            template.name()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        webfluent::init::create(&dir, "skilled", template).unwrap();
        for folder in webfluent::init::SKILL_ROOTS {
            let installed = dir.join(folder);
            let mut found: Vec<String> = walk(&installed)
                .iter()
                .map(|p| {
                    p.strip_prefix(&installed)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/")
                })
                .collect();
            found.sort();
            assert_eq!(found, wanted, "{} of a {} project", folder, template.name());
            for rel in &wanted {
                let text = std::fs::read_to_string(installed.join(rel)).unwrap();
                let source = std::fs::read_to_string(skill.join(rel)).unwrap();
                assert!(text == source, "{folder}/{rel} isn't the repository's");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// The skill's other references — code quality, recipes, styling, testing,
/// diagnostics, documents, desktop apps — are what an agent copies from.
/// Each is held as the skill is: every block parses, and the commands and
/// codes it names are `wf`'s. More than that, a reference's blocks are one
/// project's files: a later block uses the `type` an earlier one declared,
/// so every block of whole declarations in a file (and the names its
/// `// src/*.js` scripts declare) is checked together, warnings included.
#[test]
fn the_skills_references_build_and_name_only_what_wf_has() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("skills/writing-webfluent/references");
    let agents = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
    let commands: std::collections::BTreeSet<&str> = agents
        .lines()
        .filter_map(|l| l.strip_prefix("wf "))
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        // The reference's index, and its sections (a folder), are AGENTS.md's.
        .filter(|p| p.file_name().is_some_and(|n| n != "language.md"))
        .collect();
    files.sort();
    assert!(files.len() >= 7, "{files:?}");
    let mut failures = Vec::new();
    for file in &files {
        let rel = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        failures.extend(check(&rel));
        let markdown = std::fs::read_to_string(file).unwrap();
        failures.extend(public_env(&rel, &markdown));

        // Every `wf <command>` it names is one `wf` has.
        for (i, _) in markdown.match_indices("`wf ") {
            // A fence's opening line (```` ```wf no-check ````) names no command.
            if markdown[..i].ends_with("``") {
                continue;
            }
            let word: String = markdown[i + 4..]
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                .collect();
            if !word.is_empty() && !word.starts_with('-') && !commands.contains(word.as_str()) {
                failures.push(format!("{rel}: `wf {word}` isn't a command"));
            }
        }
        // Every diagnostic code it names is the compiler's.
        for (i, _) in markdown.match_indices('`') {
            let rest = &markdown[i + 1..];
            let Some(end) = rest.find('`') else { continue };
            let code = &rest[..end];
            let looks = code.len() >= 3
                && code.len() <= 4
                && code.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && code.chars().skip(1).all(|c| c.is_ascii_digit());
            if looks && webfluent::diagnostics::codes::info(code).is_none() {
                failures.push(format!("{rel}: `{code}` isn't a code `wf explain` knows"));
            }
        }

        // The file's declarations, as one program.
        let mut source = String::new();
        for (_, block, wfx) in wf_blocks(&markdown) {
            if abbreviates(&block) {
                continue;
            }
            let block = if wfx {
                match webfluent::syntax::layout::to_braces(&block, &format!("{rel}.wfx")) {
                    Ok(b) => b,
                    Err(_) => continue,
                }
            } else {
                block
            };
            let (declarations, loose) = split(&block);
            if loose.trim().is_empty() && !declarations.trim().is_empty() {
                source.push_str(&declarations);
                source.push('\n');
            }
        }
        if source.trim().is_empty() {
            continue;
        }
        let mut program = match parse_source(&source, &rel) {
            Ok(p) => p,
            Err(e) => {
                failures.push(format!("{rel}: its declarations don't parse together: {e}"));
                continue;
            }
        };
        program.declarations.extend(chapter_scripts(&markdown));
        let file_of = |_: usize| rel.clone();
        let mut findings = webfluent::sema::check(&program, &file_of);
        let typed = webfluent::sema::types::check(&program, &file_of);
        findings.errors.extend(typed.findings.errors);
        findings.errors.extend(typed.unresolved);
        for e in &findings.errors {
            failures.push(format!("{rel}: {e}"));
        }
        for w in &findings.warnings {
            failures.push(format!("{rel}: {w}"));
        }
        for e in webfluent::linter::validate_semantics_in(&program, &file_of) {
            failures.push(format!("{rel}: {e}"));
        }
        for problem in emitted_js_problems(&program) {
            failures.push(format!("{rel}: {problem}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// Every code fence in the guide says only what a fence can say
/// (`common::Fence`): a misspelt `expct=T05` would quietly make a block that
/// must fail into a sample, and a `break=` on a block no reader edits would
/// do nothing.
#[test]
fn every_fence_in_the_guide_says_what_a_fence_can() {
    let codes: Vec<&str> = webfluent::diagnostics::codes::CODES
        .iter()
        .map(|c| c.code)
        .collect();
    let mut failures = Vec::new();
    for (name, text) in crate::common::guide_pages() {
        // The ticks of the fence that is open: a block inside a longer
        // fence is the outer block's text, not a fence of the guide's.
        let mut open: Option<usize> = None;
        for (ix, line) in text.lines().enumerate() {
            let ticks = line.chars().take_while(|c| *c == '`').count();
            if ticks < 3 {
                continue;
            }
            if let Some(n) = open {
                if ticks >= n && line[ticks..].trim().is_empty() {
                    open = None;
                }
                continue;
            }
            open = Some(ticks);
            match crate::common::fence(line) {
                None => {}
                Some(Err(e)) => failures.push(format!("/{name}:{}: {e}", ix + 1)),
                Some(Ok(f)) => {
                    if let Some(code) = &f.expect
                        && !codes.contains(&code.as_str())
                    {
                        failures.push(format!("/{name}:{}: {code} is no code", ix + 1));
                    }
                    if f.swap.is_some() && !f.live {
                        failures.push(format!(
                            "/{name}:{}: `break=` needs `live`: nothing edits this block",
                            ix + 1
                        ));
                    }
                    if f.fixed && f.expect.is_some() {
                        failures.push(format!(
                            "/{name}:{}: a block is `fixed` or `expect=`, not both",
                            ix + 1
                        ));
                    }
                    // An example of a code may be in the indented layout
                    // (E003 is one); a page or a live block is `.wf`.
                    let example_lang = matches!(f.lang.as_str(), "wf" | "wfx");
                    if ((f.page || f.live) && f.lang != "wf")
                        || ((f.expect.is_some() || f.fixed) && !example_lang)
                    {
                        failures.push(format!(
                            "/{name}:{}: only a `wf` block is a page, live or expected to fail",
                            ix + 1
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A ```` ```wf fixed ```` block is the `expect=` block before it, put
/// right: it draws nothing at all — not the code it fixes, and no other —
/// and every error page that shows a broken program shows it fixed.
#[test]
fn every_fixed_example_draws_nothing() {
    let mut failures = Vec::new();
    for (name, text) in crate::common::guide_pages() {
        let mut expected: Option<String> = None;
        let mut fixed_one = false;
        let mut lines = text.lines().enumerate();
        while let Some((at, line)) = lines.next() {
            let Some(Ok(f)) = crate::common::fence(line) else {
                continue;
            };
            if f.expect.is_none() && !f.fixed {
                continue;
            }
            let mut body = String::new();
            for (_, l) in lines.by_ref() {
                if l.trim_end() == "```" {
                    break;
                }
                body.push_str(l);
                body.push('\n');
            }
            if let Some(code) = f.expect {
                expected = Some(code);
                continue;
            }
            let Some(code) = &expected else {
                failures.push(format!(
                    "/{name}:{}: a `fixed` block with no `expect=` block before it",
                    at + 1
                ));
                continue;
            };
            fixed_one = true;
            let found: Vec<String> =
                findings_with(&body, example_file(&f.lang), &scripts_above(&text, at))
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
            if !found.is_empty() {
                failures.push(format!(
                    "/{name}:{}: the fix for {code} still draws {found:?}",
                    at + 1
                ));
            }
        }
        if name.starts_with("errors/") && expected.is_some() && !fixed_one {
            failures.push(format!(
                "/{name}: shows a broken program and no ```wf fixed``` one"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The reference's "Codes this can raise" comes from
/// `scripts/docsite/builtin_codes.json`. Every code it lists is one the
/// compiler has, every built-in it names is one the registry has, and the
/// code's page names each built-in it is listed under — so a code the
/// compiler drops, or a built-in renamed, fails here rather than linking
/// to nothing.
#[test]
fn the_codes_each_builtin_can_raise_are_codes_the_compiler_has() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let table: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("scripts/docsite/builtin_codes.json")).unwrap(),
    )
    .unwrap();
    let codes: Vec<&str> = webfluent::diagnostics::codes::CODES
        .iter()
        .map(|c| c.code)
        .collect();
    let builtins: Vec<String> = webfluent::registry::components()
        .flat_map(|c| {
            std::iter::once(c.name.to_string()).chain(
                webfluent::registry::parts_of(c.name).map(|p| format!("{}.{}", c.name, p.name)),
            )
        })
        .collect();
    let mut failures = Vec::new();
    for rule in ["every", "parts", "named_only"] {
        for code in table[rule].as_array().unwrap() {
            let code = code.as_str().unwrap();
            if !codes.contains(&code) {
                failures.push(format!("`{rule}` lists {code}, which is no code"));
            }
        }
    }
    let by_code = table["codes"].as_object().unwrap();
    assert!(by_code.len() > 20, "found only {by_code:?}");
    for (code, names) in by_code {
        if !codes.contains(&code.as_str()) {
            failures.push(format!("{code} is no code"));
            continue;
        }
        let page = webfluent::diagnostics::codes::explain_source(code).unwrap_or_default();
        for name in names.as_array().unwrap() {
            let name = name.as_str().unwrap();
            if !builtins.iter().any(|b| b == name) {
                failures.push(format!("{code} lists `{name}`, which is no built-in"));
            }
            let named = page.match_indices(name).any(|(at, _)| {
                let before = page[..at].chars().next_back();
                let after = page[at + name.len()..].chars().next();
                !before.is_some_and(|c| c.is_alphanumeric() || c == '.')
                    && !after.is_some_and(char::is_alphanumeric)
            });
            if !named {
                failures.push(format!(
                    "{code} lists `{name}`, and src/diagnostics/explain/{code}.md does not name it"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The book is written to one set of rules ("How we write", in the
/// contributing chapter), and these are the ones a reader would miss most:
///
/// - every chapter ends with `## Common mistakes`: two or more programs
///   that draw the finding a reader is most likely to meet next
///   (`wf expect=CODE`), each linked to its error page;
/// - the chapters readers most often get wrong — state, components, stores
///   and data — close on a quiz of two to four questions;
/// - every chapter says what its reader learned (`learned:`);
/// - the book and the essays say nothing is "simple", "easy" or "obvious".
#[test]
fn the_book_is_written_to_its_rules() {
    let quizzed = [7, 10, 11, 15];
    let banned = [
        "simply",
        "just",
        "easy",
        "easily",
        "obviously",
        "powerful",
        "seamless",
    ];
    let link_target = regex::Regex::new(r"\]\([^)]*\)").unwrap();
    let code_span = regex::Regex::new(r"`[^`]*`").unwrap();
    let words = regex::Regex::new(r"(?i)\b[a-z]+\b").unwrap();
    let mut failures = Vec::new();
    for (n, path, text) in crate::common::guide_chapters() {
        let learn = path.starts_with("learn/");
        if !learn && !path.starts_with("about/") {
            continue;
        }
        // The prose, without its code or where its links go: a program may
        // say `just`, and a link may name `easy-todos`.
        let mut prose = String::new();
        let mut fenced = false;
        for line in text.lines() {
            if line.starts_with("```") {
                fenced = !fenced;
            } else if !fenced {
                let line = link_target.replace_all(line, "]");
                prose.push_str(&code_span.replace_all(&line, ""));
                prose.push('\n');
            }
        }
        for w in words.find_iter(&prose) {
            let w = w.as_str().to_lowercase();
            if banned.contains(&w.as_str()) {
                failures.push(format!("{path}: says `{w}`"));
            }
        }
        if !learn {
            continue;
        }
        if crate::common::front_matter(&text, "learned").is_none_or(|l| !l.starts_with('[')) {
            failures.push(format!("{path}: no `learned:` list in its front matter"));
        }
        let body = text.split_once("\n---\n").map_or(text.as_str(), |(_, b)| b);
        let sections: Vec<&str> = body.split("\n## ").skip(1).collect();
        let last = sections
            .iter()
            .rev()
            .find(|s| !s.starts_with("Next") && !s.starts_with("Quiz"));
        match last {
            Some(s) if s.starts_with("Common mistakes") => {
                let programs = s.matches("```wf expect=").count();
                let links = s.matches("errors/").count();
                if programs < 2 {
                    failures.push(format!(
                        "{path}: its mistakes show {programs} program(s), not two or more"
                    ));
                }
                if links < programs {
                    failures.push(format!("{path}: a mistake does not link its error page"));
                }
            }
            _ => failures.push(format!("{path}: does not end with `## Common mistakes`")),
        }
        let quizzes = body.matches("```quiz").count();
        if quizzed.contains(&n) && !(2..=4).contains(&quizzes) {
            failures.push(format!(
                "{path}: {quizzes} quiz question(s), not two to four"
            ));
        }
        if !quizzed.contains(&n) && quizzes > 0 {
            failures.push(format!("{path}: a quiz in a chapter that has none"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every page the site publishes has a Markdown twin at its address plus
/// `.md` (`/` is `/index.md`), for an agent and for *Copy page*; and
/// `llms.txt` lists them. The pages are the built sitemap's.
#[test]
fn every_published_page_has_a_markdown_twin() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sitemap = built_sitemap();
    let llms = std::fs::read_to_string(root.join("site/public/llms.txt")).expect("llms.txt");
    let loc = regex::Regex::new(r"<loc>https?://[^/<]+(/[^<]*)</loc>").unwrap();
    let mut failures = Vec::new();
    let mut pages = 0;
    for m in loc.captures_iter(&sitemap) {
        let route = m[1].trim_end_matches('/');
        let twin = if route.is_empty() {
            "index.md".to_string()
        } else {
            format!("{}.md", route.trim_start_matches('/'))
        };
        pages += 1;
        if !root.join("site/public").join(&twin).is_file() {
            failures.push(format!("{route}/: no twin at site/public/{twin}"));
        } else if !route.starts_with("/ar/") && !llms.contains(&format!("/{twin})")) {
            failures.push(format!("{route}/: llms.txt does not list /{twin}"));
        }
    }
    assert!(pages > 300, "{pages} pages in the sitemap");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The search finds the WebFluent way under another framework's word:
/// `useState`, `v-for`, `onClick` and `\newpage` each lead to the section
/// that teaches it, and a code is a row of its own, so `T05` jumps to it.
#[test]
fn the_search_knows_other_frameworks_words() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rows: Vec<serde_json::Value> = serde_json::from_str(
        &std::fs::read_to_string(root.join("site/public/search-index.json")).unwrap(),
    )
    .unwrap();
    let led_to = |word: &str| -> Vec<String> {
        rows.iter()
            .filter(|r| {
                r["also"]
                    .as_str()
                    .is_some_and(|a| a.split('|').any(|w| w == word))
            })
            .map(|r| r["route"].as_str().unwrap().to_string())
            .collect()
    };
    for (word, route) in [
        ("usestate", "/learn/07-state-and-reactivity#state"),
        ("computed", "/learn/07-state-and-reactivity#derived"),
        ("v-for", "/learn/09-control-flow#for"),
        ("onclick", "/learn/08-events#handlers"),
        ("classname", "/learn/13-styling#add-a-class"),
        ("usequery", "/learn/15-data#show-a-request-with-resource"),
        (
            "\\newpage",
            "/learn/17-pdf-and-slides#control-where-pages-break",
        ),
    ] {
        assert_eq!(led_to(word), [route], "`{word}`");
    }
    let t05: Vec<_> = rows
        .iter()
        .filter(|r| {
            r["section"]
                .as_str()
                .is_some_and(|s| s.starts_with("T05 —"))
        })
        .collect();
    assert_eq!(t05.len(), 1);
    assert_eq!(t05[0]["route"], "/errors/T05");
    assert!(
        rows.iter().all(|r| r["kind"].is_string()),
        "a row with no kind"
    );
}

/// What an agent reads before the site — `AGENTS.md` (which `wf init`
/// writes into every project), the README and the skill — says the same as
/// the site and the compiler, where a script can tell:
///
/// - every page of the site they send a reader to is a page the site
///   publishes, not a redirect left behind by a move;
/// - `AGENTS.md` names every built-in the registry has (but the legacy
///   spellings), and its icon list is the runtime's, with the right count.
#[test]
fn the_agent_files_agree_with_the_site_and_the_compiler() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sitemap = built_sitemap();
    let loc = regex::Regex::new(r"<loc>https?://[^/<]+(/[^<]*)</loc>").unwrap();
    let live: Vec<String> = loc
        .captures_iter(&sitemap)
        .map(|c| c[1].trim_end_matches('/').to_string())
        .collect();
    let mut files = vec![root.join("AGENTS.md"), root.join("README.md")];
    let mut stack = vec![root.join("skills")];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "md") {
                files.push(p);
            }
        }
    }
    let site =
        regex::Regex::new(r"https://webfluent\.monzeromer\.dev(/[A-Za-z0-9/_.-]*)?").unwrap();
    let path = regex::Regex::new(r"`(/docs/[a-z0-9/-]+)`").unwrap();
    let mut failures = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let name = file.strip_prefix(root).unwrap().display().to_string();
        let routes = site
            .captures_iter(&text)
            .map(|c| c.get(1).map_or("", |m| m.as_str()).to_string())
            .chain(path.captures_iter(&text).map(|c| c[1].to_string()));
        for route in routes {
            let route = route.trim_end_matches(['/', '.']).to_string();
            let file_like =
                route.ends_with(".md") || route.ends_with(".txt") || route.ends_with(".json");
            if !route.is_empty() && !file_like && !live.contains(&route) {
                failures.push(format!("{name}: {route} is no page the site publishes"));
            }
        }
    }

    let agents = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
    for c in webfluent::registry::components() {
        let legacy =
            c.summary.contains("original spelling") || c.summary.contains("original grammar");
        let named = regex::Regex::new(&format!(r"\b{}\b", regex::escape(c.name))).unwrap();
        if !legacy && !named.is_match(&agents) {
            failures.push(format!("AGENTS.md: the built-in `{}` is not named", c.name));
        }
    }
    let icons = webfluent::registry::ICONS;
    let listed: Vec<&str> = agents
        .split_once("Available icons:")
        .and_then(|(_, rest)| rest.split_once("Usage:"))
        .map(|(list, _)| list)
        .unwrap_or_default()
        .split('`')
        .skip(1)
        .step_by(2)
        .collect();
    let mut want: Vec<&str> = icons.to_vec();
    let mut have = listed.clone();
    want.sort_unstable();
    have.sort_unstable();
    if want != have {
        failures.push(format!(
            "AGENTS.md: the icon list is not the runtime's ({} listed, {} drawn)",
            have.len(),
            want.len()
        ));
    }
    let count = format!("{} built-in SVG icons", icons.len());
    if agents.matches("built-in SVG icons").count() != agents.matches(&count).count() {
        failures.push(format!("AGENTS.md: an icon count other than `{count}`"));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The site's sitemap, as a build writes it. The built site is not kept in
/// the repository (a workflow builds and deploys it), so the site is built
/// here, once, into the target folder.
fn built_sitemap() -> String {
    static SITEMAP: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SITEMAP
        .get_or_init(|| {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"));
            let out = root.join("target/docs-parse-site");
            let site = root.join("site");
            let options = webfluent::build::BuildOptions {
                output_dir: Some(out.clone()),
                ..webfluent::build::BuildOptions::new(&site, &webfluent::vfs::FsVfs)
            };
            webfluent::build::build(&options).expect("the site builds");
            std::fs::read_to_string(out.join("sitemap.xml")).expect("the built sitemap")
        })
        .clone()
}

/// Where an example is saved to be checked: a ```` ```wfx ```` block is an
/// indented-layout file.
fn example_file(lang: &str) -> &'static str {
    if lang == "wfx" {
        "src/App.wfx"
    } else {
        "src/App.wf"
    }
}
