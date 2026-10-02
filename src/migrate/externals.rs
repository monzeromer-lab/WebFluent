//! WebFluent 4 → 4.2: `external` is gone.
//!
//! The parser refuses an `external` now, so this works on the text, through
//! the lexer's tokens: it finds each declaration, takes it out, and puts
//! what it said where 4.2 says it.
//!
//! | Was | Becomes |
//! |---|---|
//! | `external X from "https://…" { … }` | a `meta.scripts` module entry, `as: "X"`, its `integrity:` moved to `meta.integrity`; `X.f(…)` keeps working, typed `Any` |
//! | `external X from "./x.js" { … }` | the file under `src/` as a plain script — its `export`s taken off, the declaration's signatures written into it as JSDoc — and `X.f(…)` rewritten to `f(…)` |
//! | `external element S("tag") { … }` | every `S(…)` rewritten to `Element("tag", …)` |

use std::path::{Path, PathBuf};

use crate::lexer::{Lexer, Token, TokenType};

/// What the step decided, for the CLI to write and report.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Every source, rewritten where it had to be.
    pub files: Vec<(PathBuf, String)>,
    /// The `.wf` files whose text changed.
    pub changed: Vec<PathBuf>,
    /// `meta.scripts` entries to add: `(src, as)`.
    pub modules: Vec<(String, String)>,
    /// `meta.integrity` entries to add.
    pub integrity: Vec<(String, String)>,
    /// A local module made a script: where it was, where it goes, its text.
    pub moved: Vec<(PathBuf, PathBuf, String)>,
    /// What the step could not decide, or the author should know.
    pub notes: Vec<String>,
}

/// `fn name(a: T, b: U) -> R`: the name, each `(param, type)`, the return.
type Signature = (String, Vec<(String, String)>, Option<String>);

/// One `external` declaration, as written.
struct Declared {
    name: String,
    element: bool,
    /// The module specifier, or the element's tag.
    from: String,
    integrity: Option<String>,
    functions: Vec<Signature>,
    /// The byte range to take out, its doc comment and line included.
    range: (usize, usize),
}

/// Migrate every `external` in `files`, a project's sources.
pub fn migrate(project_dir: &Path, files: &[(PathBuf, String)]) -> Outcome {
    let mut out = Outcome::default();
    let mut declared: Vec<Declared> = Vec::new();
    // First every declaration, from every file: a call site may be in a
    // file other than the one that declared what it calls.
    let mut texts: Vec<(PathBuf, String)> = Vec::new();
    for (path, text) in files {
        let found = declarations(text, &path.to_string_lossy());
        let mut text = text.clone();
        for d in found.iter().rev() {
            text.replace_range(d.range.0..d.range.1, "");
        }
        if !found.is_empty() {
            text = text.trim_start_matches('\n').to_string();
        }
        declared.extend(found);
        texts.push((path.clone(), text));
    }
    if declared.is_empty() {
        out.files = files.to_vec();
        return out;
    }

    for d in &declared {
        if d.element {
            continue;
        }
        if d.from.contains("://") {
            out.modules.push((d.from.clone(), d.name.clone()));
            if let Some(hash) = &d.integrity {
                out.integrity.push((d.from.clone(), hash.clone()));
            }
            if !d.functions.is_empty() {
                out.notes.push(format!(
                    "`{}` loads from `{}` through `meta.scripts`; the compiler cannot read a remote file, so `{}.…` is `Any` where its declaration typed it",
                    d.name, d.from, d.name
                ));
            }
        } else {
            local_module(project_dir, d, &mut out);
        }
    }

    // Then the call sites, in every file.
    for (path, text) in texts {
        let rewritten = call_sites(&text, &path.to_string_lossy(), &declared);
        let original = files
            .iter()
            .find(|(p, _)| p == &path)
            .map(|(_, t)| t.as_str());
        if original != Some(rewritten.as_str()) {
            out.changed.push(path.clone());
        }
        out.files.push((path, rewritten));
    }
    out
}

fn tokens(text: &str, file: &str) -> Vec<Token> {
    Lexer::new(text, file).tokenize().unwrap_or_default()
}

fn string_of(t: &Token) -> Option<String> {
    match &t.token_type {
        TokenType::StringLiteral(s) => Some(s.spelling.clone()),
        _ => None,
    }
}

fn ident_of(t: &Token) -> Option<&str> {
    match &t.token_type {
        TokenType::Identifier(w) => Some(w.as_str()),
        _ => None,
    }
}

/// Every top-level `external` in `text`.
fn declarations(text: &str, file: &str) -> Vec<Declared> {
    let toks = tokens(text, file);
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < toks.len() {
        match toks[i].token_type {
            TokenType::OpenBrace => depth += 1,
            TokenType::CloseBrace => depth -= 1,
            _ => {}
        }
        if depth != 0 || ident_of(&toks[i]) != Some("external") {
            i += 1;
            continue;
        }
        let start = i;
        let mut j = i + 1;
        let element = ident_of(&toks[j]) == Some("element");
        if element {
            j += 1;
        }
        let Some(name) = toks.get(j).and_then(ident_of).map(str::to_string) else {
            i += 1;
            continue;
        };
        j += 1;
        let from = if element {
            // `("tag")`
            let tag = toks.get(j + 1).and_then(string_of);
            j += 3;
            tag
        } else {
            // `from "spec"`
            let spec = toks.get(j + 1).and_then(string_of);
            j += 2;
            spec
        };
        let Some(from) = from else {
            i += 1;
            continue;
        };
        // The body, to its matching brace.
        let mut integrity = None;
        let mut functions = Vec::new();
        let mut end = toks.get(j).map(|t| t.end).unwrap_or(text.len());
        if matches!(
            toks.get(j).map(|t| &t.token_type),
            Some(TokenType::OpenBrace)
        ) {
            let mut d = 0;
            let mut k = j;
            while k < toks.len() {
                match toks[k].token_type {
                    TokenType::OpenBrace => d += 1,
                    TokenType::CloseBrace => {
                        d -= 1;
                        if d == 0 {
                            end = toks[k].end;
                            break;
                        }
                    }
                    _ => {}
                }
                if d == 1 {
                    if ident_of(&toks[k]) == Some("integrity") {
                        integrity = toks.get(k + 2).and_then(string_of);
                    }
                    if ident_of(&toks[k]) == Some("fn")
                        && let Some(f) = toks.get(k + 1).and_then(ident_of)
                    {
                        functions.push(signature(text, &toks, k + 1, f));
                    }
                }
                k += 1;
            }
            // Past its closing brace, which the depth count must not see.
            i = k + 1;
        } else {
            i = j;
        }
        // Take its doc comment with it, and the line it stood on.
        let mut from_at = toks[start].offset;
        let line_start = text[..from_at].rfind('\n').map_or(0, |n| n + 1);
        from_at = line_start;
        while let Some(prev) = text[..from_at.saturating_sub(1)]
            .rfind('\n')
            .map(|n| n + 1)
            .or((from_at > 0).then_some(0))
        {
            if text[prev..from_at].trim_start().starts_with("///") {
                from_at = prev;
            } else {
                break;
            }
        }
        let mut to = text[end..].find('\n').map_or(text.len(), |n| end + n + 1);
        // And the blank line that set it apart, so none piles up.
        if text[to..].starts_with('\n') {
            to += 1;
        }
        out.push(Declared {
            name,
            element,
            from,
            integrity,
            functions,
            range: (from_at, to),
        });
    }
    out
}

/// `fn name(a: T, b: U) -> R`, from the token of its name at `at`.
fn signature(text: &str, toks: &[Token], at: usize, name: &str) -> Signature {
    let mut params = Vec::new();
    let mut k = at + 2;
    let mut close = toks.get(at + 1).map(|t| t.end).unwrap_or(0);
    let mut depth = 1;
    let mut current: Option<String> = None;
    let mut ty_from: Option<usize> = None;
    while k < toks.len() {
        match &toks[k].token_type {
            TokenType::OpenParen | TokenType::OpenBracket => depth += 1,
            TokenType::CloseParen | TokenType::CloseBracket if depth == 1 => {
                if let (Some(p), Some(from)) = (current.take(), ty_from.take()) {
                    params.push((p, text[from..toks[k].offset].trim().to_string()));
                }
                close = toks[k].end;
                break;
            }
            TokenType::CloseParen | TokenType::CloseBracket => depth -= 1,
            TokenType::Comma if depth == 1 => {
                if let (Some(p), Some(from)) = (current.take(), ty_from.take()) {
                    params.push((p, text[from..toks[k].offset].trim().to_string()));
                }
            }
            TokenType::Colon if depth == 1 && current.is_none() => {
                current = toks.get(k - 1).and_then(ident_of).map(str::to_string);
                ty_from = Some(toks[k].end);
            }
            _ => {}
        }
        k += 1;
    }
    // `-> T` to the end of the line.
    let rest = &text[close..];
    let line = &rest[..rest.find('\n').unwrap_or(rest.len())];
    let returns = line
        .trim()
        .strip_prefix("->")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    (name.to_string(), params, returns)
}

/// A WebFluent type as JSDoc writes it.
fn jsdoc_type(ty: &str) -> String {
    let ty = ty.trim();
    if let Some(inner) = ty.strip_suffix('?') {
        return format!("?{}", jsdoc_type(inner));
    }
    if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        return format!("{}[]", jsdoc_type(inner));
    }
    match ty {
        "String" => "string",
        "Number" => "number",
        "Bool" => "boolean",
        "Map" => "Object",
        _ => "*",
    }
    .to_string()
}

/// A local module made a plain script under `src/`.
fn local_module(project_dir: &Path, d: &Declared, out: &mut Outcome) {
    // `externals.js` sat at the output root, so the specifier was read from
    // there — which is `public/`.
    let rel = d.from.trim_start_matches("./").trim_start_matches('/');
    let from = project_dir.join("public").join(rel);
    let Ok(text) = std::fs::read_to_string(&from) else {
        out.notes.push(format!(
            "`{}` imported `{}`, which is not in `public/`; put its code in a `.js` file under `src/`",
            d.name, d.from
        ));
        return;
    };
    let scan = crate::project_js::scan::scan(&text);
    if text.lines().any(|l| l.trim_start().starts_with("import ")) {
        out.notes.push(format!(
            "`{}` imports other modules, so it cannot become a plain script by itself; it stays in `public/`, and `{}.…` calls need a script under `src/` that does what it did",
            from.display(),
            d.name
        ));
        return;
    }
    let _ = scan;
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        if trimmed.starts_with("export {") || trimmed.starts_with("export{") {
            // The names it lists are declared in the file already.
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("export default ") {
            out.notes.push(format!(
                "`{}` had an `export default`, which names nothing; it is kept as a plain statement",
                from.display()
            ));
            lines.push(format!("{indent}{rest}"));
            continue;
        }
        let rest = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        // The declaration's signature, as JSDoc above the function it
        // describes, when the file has none of its own there.
        for (name, params, returns) in &d.functions {
            let declares = [
                format!("function {name}("),
                format!("async function {name}("),
                format!("class {name} "),
                format!("class {name}{{"),
                format!("const {name} ="),
            ]
            .iter()
            .any(|p| rest.starts_with(p.as_str()));
            let documented = lines.last().is_some_and(|l| l.trim_end().ends_with("*/"));
            if declares && !documented {
                lines.push(format!("{indent}/**"));
                for (p, ty) in params {
                    lines.push(format!("{indent} * @param {{{}}} {p}", jsdoc_type(ty)));
                }
                if let Some(r) = returns {
                    lines.push(format!("{indent} * @returns {{{}}}", jsdoc_type(r)));
                }
                lines.push(format!("{indent} */"));
            }
        }
        lines.push(format!("{indent}{rest}"));
    }
    let to = project_dir.join("src").join(rel);
    let mut source = lines.join("\n");
    if text.ends_with('\n') {
        source.push('\n');
    }
    out.moved.push((from, to, source));
}

/// Every call site of what the declarations named, rewritten.
fn call_sites(text: &str, file: &str, declared: &[Declared]) -> String {
    let toks = tokens(text, file);
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let Some(word) = ident_of(t) else {
            continue;
        };
        // Not a field: `a.Stripe` is someone else's.
        if i > 0 && matches!(toks[i - 1].token_type, TokenType::Dot) {
            continue;
        }
        let Some(d) = declared.iter().find(|d| d.name == word) else {
            continue;
        };
        let next = toks.get(i + 1).map(|t| &t.token_type);
        if d.element {
            let tag = serde_json::to_string(&d.from).unwrap_or_default();
            match next {
                Some(TokenType::OpenParen) => {
                    let empty = matches!(
                        toks.get(i + 2).map(|t| &t.token_type),
                        Some(TokenType::CloseParen)
                    );
                    let replacement = if empty {
                        format!("Element({tag}")
                    } else {
                        format!("Element({tag}, ")
                    };
                    edits.push((t.offset, toks[i + 1].end, replacement));
                }
                _ => edits.push((t.offset, t.end, format!("Element({tag})"))),
            }
        } else if !d.from.contains("://") && matches!(next, Some(TokenType::Dot)) {
            // `X.f(…)` → `f(…)`: the script's names are global.
            edits.push((t.offset, toks[i + 1].end, String::new()));
        }
    }
    let mut out = text.to_string();
    for (from, to, with) in edits.into_iter().rev() {
        out.replace_range(from..to, &with);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(name: &str, files: &[(&str, &str)], public: &[(&str, &str)]) -> (Outcome, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("wf-migrate-ext-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (p, t) in public {
            let at = dir.join("public").join(p);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, t).unwrap();
        }
        let files: Vec<(PathBuf, String)> = files
            .iter()
            .map(|(p, t)| (dir.join(p), t.to_string()))
            .collect();
        (migrate(&dir, &files), dir)
    }

    const PAGE: &str =
        "page P(path: \"/\", title: \"T\", description: \"D\") {\n    Heading(\"T\").h1\n";

    #[test]
    fn a_remote_module_becomes_a_meta_scripts_entry() {
        let src = format!(
            "/// Charts.\nexternal Chart from \"https://cdn.example.com/chart.js\" {{\n    integrity: \"sha384-abc\"\n    fn Chart(canvas: Any, config: Map) -> Handle\n}}\n\n{PAGE}    Host(tag: \"canvas\", mount: (n) => Chart.Chart(n, {{}}))\n}}\n"
        );
        let (out, _) = run("remote", &[("src/App.wf", &src)], &[]);
        let text = &out.files[0].1;
        assert!(
            !text.contains("external") && !text.contains("/// Charts."),
            "{text}"
        );
        assert!(
            text.contains("Chart.Chart(n, {})"),
            "a remote module keeps its namespace: {text}"
        );
        assert_eq!(
            out.modules,
            [(
                "https://cdn.example.com/chart.js".to_string(),
                "Chart".to_string()
            )]
        );
        assert_eq!(
            out.integrity,
            [(
                "https://cdn.example.com/chart.js".to_string(),
                "sha384-abc".to_string()
            )]
        );
        crate::syntax::parse_source(text, "t.wf").expect("4.2 parses it");
    }

    #[test]
    fn a_local_module_becomes_a_script_with_jsdoc() {
        let src = format!(
            "external Tilt from \"./vendor/tilt.js\" {{\n    fn initTilt(node: Any, max: Number) -> String\n}}\n{PAGE}    Host(mount: (n) => Tilt.initTilt(n, 5))\n}}\n"
        );
        let js = "export function initTilt(node, max) { return \"x\" }\nconst inner = 1\nexport { inner }\n";
        let (out, dir) = run("local", &[("src/App.wf", &src)], &[("vendor/tilt.js", js)]);
        let text = &out.files[0].1;
        assert!(text.contains("mount: (n) => initTilt(n, 5)"), "{text}");
        let (from, to, source) = &out.moved[0];
        assert_eq!(from, &dir.join("public/vendor/tilt.js"));
        assert_eq!(to, &dir.join("src/vendor/tilt.js"));
        assert_eq!(
            source,
            "/**\n * @param {*} node\n * @param {number} max\n * @returns {string}\n */\nfunction initTilt(node, max) { return \"x\" }\nconst inner = 1\n"
        );
        assert!(crate::project_js::scan::scan(source).module_at.is_none());
    }

    #[test]
    fn every_declaration_in_a_file_is_found() {
        let src = format!(
            "external A from \"https://a.example/a.js\" {{ fn a() }}\nexternal B from \"https://b.example/b.js\" {{ fn b() }}\nexternal element C(\"c-el\") {{ prop x: String }}\n{PAGE}    C\n}}\n"
        );
        let (out, _) = run("several", &[("src/App.wf", &src)], &[]);
        assert_eq!(out.modules.len(), 2, "{:?}", out.modules);
        let text = &out.files[0].1;
        assert!(!text.contains("external"), "{text}");
        assert!(text.contains("    Element(\"c-el\")\n"), "{text}");
    }

    #[test]
    fn an_element_is_placed_by_its_tag() {
        let decl = "external element Stripe(\"stripe-pricing-table\") {\n    prop publishableKey: String\n    event ready()\n}\n";
        let page = format!(
            "{PAGE}    state k = \"pk\"\n    Stripe(publishableKey: k) {{ on ready {{ log(1) }} }}\n    Stripe()\n    Stripe\n}}\n"
        );
        let (out, _) = run(
            "element",
            &[("src/Stripe.wf", decl), ("src/App.wf", &page)],
            &[],
        );
        assert_eq!(out.files[0].1, "");
        let text = &out.files[1].1;
        assert!(
            text.contains("Element(\"stripe-pricing-table\", publishableKey: k) { on ready"),
            "{text}"
        );
        assert!(
            text.contains(
                "    Element(\"stripe-pricing-table\")\n    Element(\"stripe-pricing-table\")\n"
            ),
            "{text}"
        );
        crate::syntax::parse_source(text, "t.wf").expect("4.2 parses it");
    }
}
