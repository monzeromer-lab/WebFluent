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

const DECLARATIONS: &[&str] = &["page", "component", "store", "theme", "app", "type", "enum"];

fn starts_declaration(line: &str) -> bool {
    let word = line
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or("");
    DECLARATIONS.contains(&word)
        && (line.len() == word.len() || !line.as_bytes()[word.len()].is_ascii_alphanumeric())
}

/// The block split into its declarations and the lines outside them.
fn split(block: &str) -> (String, String) {
    let mut declarations = String::new();
    let mut loose = String::new();
    let mut depth = 0i32;
    let mut in_declaration = false;
    // Doc comments go with whatever follows them.
    let mut docs = String::new();
    for line in block.lines() {
        if depth == 0 && line.trim_start().starts_with("///") {
            docs.push_str(line);
            docs.push('\n');
            continue;
        }
        if depth == 0 {
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
        for c in line.split("//").next().unwrap_or("").chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if depth <= 0 {
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
    "animate(",
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
        if block.contains("...") || block.contains('…') {
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
    let failures = check("AGENTS.md");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn the_readme_parses() {
    let failures = check("README.md");
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
            failures.extend(check(&format!("spec/{name}")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
