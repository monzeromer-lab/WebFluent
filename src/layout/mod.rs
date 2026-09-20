//! Between the two layouts of the one grammar: `.wf`, whose blocks are
//! braced, and `.wfx`, whose blocks are indented.
//!
//! The conversion is token-level, not a printer: a `.wf` file loses the
//! braces that open a block on one line and close it on a line of their
//! own, and keeps its indentation, which already says what the braces
//! said; a `.wfx` file gains a ` {` at the end of every line that opens a
//! block and a `}` on a line of its own where the block ends. Braces
//! written on one line (`on click { save() }`), map literals and anything
//! inside parentheses are left as they are, because both layouts read them
//! the same way. Each result is lexed back and held to the token stream of
//! the original, so a file whose indentation does not follow its braces is
//! refused rather than changed in meaning.

use crate::error::{Diagnostic, Result, WebFluentError};
use crate::lexer::v2::{Layout, LexerV2};
use crate::lexer::{Token, TokenType};

/// The `.wfx` spelling of a `.wf` source.
pub fn to_offside(source: &str, file: &str) -> Result<String> {
    let tokens = LexerV2::with_layout(source, file, Layout::Braces).tokenize()?;
    let removed = block_braces(&tokens, source);
    // Delete the braces, right to left, so earlier offsets hold. An empty
    // block keeps its braces, closed on the line that opened it.
    let mut text = source.to_string();
    let mut edits: Vec<(usize, bool)> = removed
        .iter()
        .flat_map(|(o, c, empty)| [(*o, *empty), (*c, false)])
        .collect();
    edits.sort_unstable();
    for (at, keep_open) in edits.into_iter().rev() {
        if keep_open {
            text.replace_range(at..at + 1, "{ }");
            continue;
        }
        // The space before an opening brace goes with it (`Row {` → `Row`),
        // as does the one after a closing brace (`} else` → `else`).
        let mut start = at;
        let mut end = at + 1;
        if &text[at..at + 1] == "{" {
            while start > 0 && text.as_bytes()[start - 1] == b' ' {
                start -= 1;
            }
        } else {
            while end < text.len() && text.as_bytes()[end] == b' ' {
                end += 1;
            }
        }
        text.replace_range(start..end, "");
    }
    // A line that held only a closing brace is gone with it.
    let mut lines: Vec<&str> = text.lines().collect();
    let was_brace_line: Vec<bool> = source
        .lines()
        .map(|l| l.trim() == "}" || l.trim().starts_with("} "))
        .collect();
    let kept: Vec<String> = lines
        .iter_mut()
        .enumerate()
        .filter_map(|(i, l)| {
            let trimmed = l.trim_end();
            if trimmed.trim().is_empty() && was_brace_line.get(i).copied().unwrap_or(false) {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();
    let mut out = kept.join("\n");
    if source.ends_with('\n') {
        out.push('\n');
    }
    verify(&tokens, &out, file, Layout::Offside)?;
    Ok(out)
}

/// The `.wf` spelling of a `.wfx` source.
pub fn to_braces(source: &str, file: &str) -> Result<String> {
    let tokens = LexerV2::with_layout(source, file, Layout::Offside).tokenize()?;
    // Every synthetic brace, with the indentation of the line that opened
    // its block; innermost blocks close first at a shared offset.
    let mut insertions: Vec<(usize, String)> = Vec::new();
    let mut open: Vec<String> = Vec::new();
    for token in &tokens {
        let synthetic = token.offset == token.end;
        match &token.token_type {
            TokenType::OpenBrace if synthetic => {
                open.push(indent_of_line(source, token.offset));
                insertions.push((token.offset, " {".to_string()));
            }
            TokenType::CloseBrace if synthetic => {
                let indent = open.pop().unwrap_or_default();
                insertions.push((token.offset, format!("\n{indent}}}")));
            }
            _ => {}
        }
    }
    let mut text = String::with_capacity(source.len() + insertions.len() * 4);
    let mut at = 0;
    for (offset, what) in insertions {
        text.push_str(&source[at..offset]);
        text.push_str(&what);
        at = offset;
    }
    text.push_str(&source[at..]);
    // `}` and an `else` at the same indent share a line, as they are
    // written; blank lines between them go.
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut i = 0;
    while i + 1 < lines.len() {
        let indent = lines[i][..lines[i].len() - lines[i].trim_start().len()].to_string();
        let mut j = i + 1;
        while j < lines.len() && lines[j].trim().is_empty() {
            j += 1;
        }
        let next = lines
            .get(j)
            .map(|l| l.trim_start().to_string())
            .unwrap_or_default();
        if lines[i].trim() == "}"
            && (next == "else" || next.starts_with("else "))
            && lines[j].len() - next.len() == indent.len()
            && lines[j].starts_with(&indent)
        {
            lines[i] = format!("{indent}}} {next}");
            lines.drain(i + 1..=j);
        }
        i += 1;
    }
    let mut out = lines.join("\n");
    if source.ends_with('\n') {
        out.push('\n');
    }
    verify(&tokens, &out, file, Layout::Braces)?;
    Ok(out)
}

/// The braces a `.wfx` file does without: those that open a block at the
/// end of a line and close it on a line of their own, outside parentheses,
/// map literals and expressions. Each pair as `(open offset, close offset,
/// empty)` — an empty block has nothing to indent.
fn block_braces(tokens: &[Token], source: &str) -> Vec<(usize, usize, bool)> {
    let mut pairs = Vec::new();
    // Each open brace: its index, and whether it is a block.
    let mut stack: Vec<(usize, bool)> = Vec::new();
    let mut parens = 0usize;
    for (i, token) in tokens.iter().enumerate() {
        match &token.token_type {
            TokenType::OpenParen | TokenType::OpenBracket => parens += 1,
            TokenType::CloseParen | TokenType::CloseBracket => parens = parens.saturating_sub(1),
            TokenType::OpenBrace => {
                let inside_expression = parens > 0
                    || stack.last().is_some_and(|(_, block)| !block)
                    || i.checked_sub(1)
                        .is_some_and(|p| expression_position(&tokens[p].token_type));
                let ends_its_line = tokens.get(i + 1).is_some_and(|n| n.line > token.line);
                stack.push((i, !inside_expression && ends_its_line));
            }
            TokenType::CloseBrace => {
                if let Some((open, block)) = stack.pop()
                    && block
                    && first_on_its_line(source, token.offset)
                {
                    pairs.push((tokens[open].offset, token.offset, open + 1 == i));
                }
            }
            _ => {}
        }
    }
    pairs
}

/// Whether a `{` after this token is a value — a map literal, a lambda's
/// body — rather than a block.
fn expression_position(kind: &TokenType) -> bool {
    matches!(
        kind,
        TokenType::Equals
            | TokenType::Colon
            | TokenType::OpenParen
            | TokenType::Comma
            | TokenType::OpenBracket
            | TokenType::Arrow
            | TokenType::NullCoalesce
            | TokenType::Or
            | TokenType::And
            | TokenType::DoubleEquals
            | TokenType::NotEquals
            | TokenType::StrictNotEqual
            | TokenType::LessThan
            | TokenType::GreaterThan
            | TokenType::LessEquals
            | TokenType::GreaterEquals
            | TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::Percent
            | TokenType::Not
            | TokenType::OpenBrace
    ) || matches!(kind, TokenType::Identifier(w) if w == "return")
}

fn first_on_its_line(source: &str, offset: usize) -> bool {
    let line_start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
    source[line_start..offset].trim().is_empty()
}

fn indent_of_line(source: &str, offset: usize) -> String {
    let line_start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
    source[line_start..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

/// The converted text, lexed in its layout, must be the same token stream.
fn verify(original: &[Token], converted: &str, file: &str, layout: Layout) -> Result<()> {
    let again = LexerV2::with_layout(converted, file, layout).tokenize()?;
    let kinds = |ts: &[Token]| ts.iter().map(|t| t.token_type.clone()).collect::<Vec<_>>();
    let (a, b) = (kinds(original), kinds(&again));
    if a == b {
        return Ok(());
    }
    let at = a
        .iter()
        .zip(&b)
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()));
    let token = original.get(at).or(original.last());
    let (line, column) = token.map_or((1, 1), |t| (t.line, t.column));
    Err(WebFluentError::ParseError(
        Diagnostic::new(
            "The file's indentation does not follow its blocks, so its layout cannot be changed without changing what it says",
            file,
            line,
            column,
        )
        .with_hint("Indent each block's lines deeper than the line that opens it, and nothing else"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WF: &str = "page Home(path: \"/\") {\n    state open = true\n    state user = {\n        name: \"a\"\n    }\n    Row(gap: .sm) {\n        style {\n            padding: 6px 0\n            &:hover {\n                background: $surface-hover\n            }\n        }\n        on click { open = !open }\n        Text(\"a\").bold\n    }\n    if open {\n        Spinner\n    } else if user.name == \"\" {\n        Text(\"c\")\n    } else {\n        Text(\"b\")\n    }\n    derived label = if open { \"x\" } else { \"y\" }\n    Router\n}\n\nenum Tone { calm, loud }\n\napp {\n    Router\n}\n";

    const WFX: &str = "page Home(path: \"/\")\n    state open = true\n    state user = {\n        name: \"a\"\n    }\n    Row(gap: .sm)\n        style\n            padding: 6px 0\n            &:hover\n                background: $surface-hover\n        on click { open = !open }\n        Text(\"a\").bold\n    if open\n        Spinner\n    else if user.name == \"\"\n        Text(\"c\")\n    else\n        Text(\"b\")\n    derived label = if open { \"x\" } else { \"y\" }\n    Router\n\nenum Tone { calm, loud }\n\napp\n    Router\n";

    #[test]
    fn braces_become_indentation_and_back() {
        assert_eq!(to_offside(WF, "t.wf").unwrap(), WFX);
        assert_eq!(to_braces(WFX, "t.wfx").unwrap(), WF);
    }

    #[test]
    fn an_empty_block_keeps_its_braces_on_one_line() {
        assert_eq!(
            to_offside("app {\n    Navbar.Links {\n    }\n    Router\n}\n", "t.wf").unwrap(),
            "app\n    Navbar.Links { }\n    Router\n"
        );
    }

    #[test]
    fn a_file_whose_indentation_lies_is_refused() {
        let err = to_offside("page P(path: \"/\") {\nText(\"a\")\n}\n", "t.wf")
            .unwrap_err()
            .to_string();
        assert!(err.contains("indentation does not follow"), "{err}");
    }
}
