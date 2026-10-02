//! The classes the project's own stylesheets define, where a `class:` names
//! one.
//!
//! `Card(class: "feature")` picks up a rule from a `.css` file under `src/`.
//! Inside the string — or a key of a `class:` map, or an entry of its list —
//! the editor offers the classes those sheets define, shows the rule on
//! hover, and goes to it.

use tower_lsp::lsp_types::*;
use webfluent::lexer::{Token, TokenType};

use crate::project::Project;

/// One class a stylesheet defines: the first rule that names it.
pub struct Defined {
    pub name: String,
    /// Index into `project.stylesheet_files`.
    pub sheet: usize,
    /// Byte offset of the name (after its dot) in the sheet.
    pub at: usize,
    /// The rule it appears in, as written.
    pub rule: String,
}

/// Whether `offset` is inside a string that is a `class:` value: the string
/// itself, a key of its map, or an entry of its list.
pub fn in_class_value(tokens: &[Token], offset: usize) -> bool {
    let Some(ix) = tokens.iter().position(|t| {
        matches!(t.token_type, TokenType::StringLiteral(_)) && t.offset < offset && offset < t.end
    }) else {
        return false;
    };
    let mut depth = 0i32;
    for i in (0..ix).rev() {
        match &tokens[i].token_type {
            TokenType::CloseBracket | TokenType::CloseBrace | TokenType::CloseParen => depth += 1,
            TokenType::OpenBracket | TokenType::OpenBrace if depth == 0 => {}
            TokenType::OpenBracket | TokenType::OpenBrace => depth -= 1,
            TokenType::OpenParen if depth == 0 => return false,
            TokenType::OpenParen => depth -= 1,
            TokenType::Colon if depth == 0 => {
                return matches!(
                    i.checked_sub(1).map(|p| &tokens[p].token_type),
                    Some(TokenType::Identifier(k)) if k == "class"
                );
            }
            _ => {}
        }
    }
    false
}

/// The class name around `offset` in `source`: letters, digits, `-`, `_`.
pub fn class_word_at(source: &str, offset: usize) -> Option<&str> {
    let is_part = |c: char| c.is_alphanumeric() || c == '-' || c == '_';
    let start = source[..offset]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| is_part(c))
        .last()
        .map_or(offset, |(i, _)| i);
    let end = source[offset..]
        .char_indices()
        .find(|&(_, c)| !is_part(c))
        .map_or(source.len(), |(i, _)| offset + i);
    (start < end).then(|| &source[start..end])
}

/// Every class the project's stylesheets define, first definition first.
pub fn defined(project: &Project) -> Vec<Defined> {
    let mut out: Vec<Defined> = Vec::new();
    for (sheet, (_, text)) in project.stylesheet_files.iter().enumerate() {
        for (name, at, rule) in selectors(text) {
            if !out.iter().any(|d| d.name == name) {
                out.push(Defined {
                    name,
                    sheet,
                    at,
                    rule,
                });
            }
        }
    }
    out
}

/// `(class, offset, rule)` for every `.class` in a selector of `css`. A
/// selector is the text before a `{`; what follows `:` inside a block is a
/// value, where `.5em` is a number, not a class.
fn selectors(css: &str) -> Vec<(String, usize, String)> {
    let bytes = css.as_bytes();
    let mut out = Vec::new();
    let mut segment_start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = css[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |e| i + 2 + e + 2);
                segment_start = i;
                continue;
            }
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
            }
            b'{' => {
                let prelude = &css[segment_start..i];
                if !prelude.trim_start().starts_with('@') {
                    let end = rule_end(css, i);
                    let rule = css[segment_start..end].trim().to_string();
                    for (at, name) in classes_in(prelude) {
                        out.push((name, segment_start + at, rule.clone()));
                    }
                }
                segment_start = i + 1;
            }
            b'}' | b';' => segment_start = i + 1,
            _ => {}
        }
        i += 1;
    }
    out
}

/// The offset just past the `}` that closes the block opened at `open`.
fn rule_end(css: &str, open: usize) -> usize {
    let mut depth = 0;
    for (i, c) in css[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + i + 1;
                }
            }
            _ => {}
        }
    }
    css.len()
}

/// `(offset of the name, name)` for each `.name` in a selector.
fn classes_in(selector: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let chars: Vec<(usize, char)> = selector.char_indices().collect();
    for (k, &(at, c)) in chars.iter().enumerate() {
        if c != '.' {
            continue;
        }
        let starts = chars
            .get(k + 1)
            .is_some_and(|&(_, n)| n.is_alphabetic() || n == '_' || n == '-');
        if !starts {
            continue;
        }
        let name: String = chars[k + 1..]
            .iter()
            .map(|&(_, c)| c)
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        out.push((at + 1, name));
    }
    out
}

/// What the editor offers inside a `class:` string.
pub fn completions(project: &Project) -> Vec<CompletionItem> {
    defined(project)
        .into_iter()
        .map(|d| {
            let sheet = &project.stylesheet_files[d.sheet].0;
            let file = project
                .root
                .as_ref()
                .and_then(|root| sheet.strip_prefix(root).ok())
                .unwrap_or(sheet)
                .to_string_lossy()
                .to_string();
            CompletionItem {
                label: d.name.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(file),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```css\n{}\n```", clip(&d.rule)),
                })),
                ..Default::default()
            }
        })
        .collect()
}

/// The rule that defines `name`, for a hover.
pub fn hover(project: &Project, name: &str) -> Option<String> {
    let d = defined(project).into_iter().find(|d| d.name == name)?;
    Some(format!("```css\n{}\n```", clip(&d.rule)))
}

/// Where `name` is defined.
pub fn location(project: &Project, name: &str) -> Option<Location> {
    let d = defined(project).into_iter().find(|d| d.name == name)?;
    let (path, text) = &project.stylesheet_files[d.sheet];
    let before = &text[..d.at];
    let line = before.matches('\n').count() as u32;
    let col = before[before.rfind('\n').map_or(0, |i| i + 1)..]
        .encode_utf16()
        .count() as u32;
    let end = col + d.name.encode_utf16().count() as u32;
    Some(Location {
        uri: Url::from_file_path(path).ok()?,
        range: Range::new(Position::new(line, col), Position::new(line, end)),
    })
}

/// A rule shown in a hover: its first lines, enough to recognise it.
fn clip(rule: &str) -> String {
    let lines: Vec<&str> = rule.lines().collect();
    if lines.len() <= 12 {
        rule.to_string()
    } else {
        format!("{}\n  …", lines[..12].join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectors_not_values() {
        let css = "/* .not-me */\n.feature, .card > .title:hover { margin: .5em; content: \".no\" }\n@media (min-width: 40em) { .wide { gap: 1rem } }\n.a { &.nested-b { color: red } }";
        let names: Vec<String> = selectors(css).into_iter().map(|(n, _, _)| n).collect();
        assert_eq!(names, ["feature", "card", "title", "wide", "a", "nested-b"]);
    }
}
