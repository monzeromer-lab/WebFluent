//! `wf fmt`: the canonical spelling of a source file, arrived at without a
//! printer.
//!
//! A printer would drop every comment, since the tree holds none, and
//! would have to know the raw text of every style value. So the formatter
//! works on the lines of the file and the token stream over them: each
//! line is indented to the depth of the brackets open before it (four
//! spaces a level, a line that closes one a level out), trailing blanks
//! go, tabs become spaces, runs of blank lines fold to one, and a brace
//! that opens a block sits one space after what it opens (`Row{` → `Row {`,
//! `}else` → `} else`). A line that continues a token — a value carried
//! over by a trailing comma, a string with a newline in it — is left as it
//! was. The result is lexed again and held to the original's tokens, so
//! formatting never changes what a file says.
//!
//! A `.wfx` file is formatted through its braced spelling and written back
//! indented, which normalises its layout the same way.

use crate::error::{Diagnostic, Result, WebFluentError};
use crate::layout::{to_braces, to_offside};
use crate::lexer::v2::{Layout, LexerV2};
use crate::lexer::{Token, TokenType};

const INDENT: &str = "    ";

/// The canonical spelling of `source`, which is `file`'s text.
pub fn format_source(source: &str, file: &str) -> Result<String> {
    // Only a file the compiler reads is formatted: a WebFluent 2 file is
    // refused with the migration hint, a broken one with its error.
    crate::syntax::parse_source(source, file)?;
    if Layout::of_file(file) == Layout::Offside {
        let braced = to_braces(source, file)?;
        let formatted = format_braced(&braced, file)?;
        return to_offside(&formatted, file);
    }
    format_braced(source, file)
}

fn format_braced(source: &str, file: &str) -> Result<String> {
    let tokens = LexerV2::with_layout(source, file, Layout::Braces).tokenize()?;
    let lines: Vec<&str> = source.lines().collect();

    // The depth each line starts at, and whether the line begins inside a
    // token that started on an earlier line.
    let mut depth_at: Vec<usize> = vec![0; lines.len() + 1];
    let mut inside_token: Vec<bool> = vec![false; lines.len() + 1];
    let mut closes_first: Vec<bool> = vec![false; lines.len() + 1];
    let mut depth = 0usize;
    let mut line_of_last = 0usize;
    let mut first_on_line: Vec<bool> = vec![true; lines.len() + 1];
    for token in &tokens {
        if matches!(token.token_type, TokenType::EOF) {
            break;
        }
        let line = token.line;
        // Lines between the last token and this one start at the running depth.
        for l in (line_of_last + 1)..=line {
            depth_at[l.min(lines.len())] = depth;
        }
        if first_on_line[line.min(lines.len())] {
            first_on_line[line.min(lines.len())] = false;
            if is_closer(&token.token_type) {
                closes_first[line.min(lines.len())] = true;
            }
        }
        match &token.token_type {
            TokenType::OpenBrace | TokenType::OpenParen | TokenType::OpenBracket => depth += 1,
            t if is_closer(t) => depth = depth.saturating_sub(1),
            _ => {}
        }
        // A token that runs past its line: the lines it covers are its own.
        let end_line = line
            + source[token.offset..token.end.min(source.len())]
                .matches('\n')
                .count();
        for l in (line + 1)..=end_line {
            if l <= lines.len() {
                inside_token[l] = true;
                depth_at[l] = depth;
            }
        }
        line_of_last = end_line.max(line);
    }
    for d in depth_at.iter_mut().skip(line_of_last + 1) {
        *d = depth;
    }

    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut blank_run = 0usize;
    for (i, raw) in lines.iter().enumerate() {
        let n = i + 1;
        let trimmed_end = raw.trim_end();
        if trimmed_end.trim().is_empty() {
            blank_run += 1;
            if blank_run == 1 && !out.is_empty() {
                out.push(String::new());
            }
            continue;
        }
        blank_run = 0;
        if inside_token[n] {
            out.push(trimmed_end.to_string());
            continue;
        }
        let body = trimmed_end.trim_start();
        let mut level = depth_at[n];
        if closes_first[n] {
            level = level.saturating_sub(1);
        }
        let mut line = String::new();
        for _ in 0..level {
            line.push_str(INDENT);
        }
        line.push_str(&space_braces(body, &tokens, n, source));
        out.push(line);
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    let mut text = out.join("\n");
    text.push('\n');
    verify(&tokens, &text, file)?;
    Ok(text)
}

fn is_closer(t: &TokenType) -> bool {
    matches!(
        t,
        TokenType::CloseBrace | TokenType::CloseParen | TokenType::CloseBracket
    )
}

/// One space before a `{` that follows a word or `)`, and one after a `}`
/// that a word follows, on line `n` — outside strings and style values,
/// which the tokens tell apart.
fn space_braces(body: &str, tokens: &[Token], n: usize, source: &str) -> String {
    // The braces on this line that the lexer saw as tokens, by column.
    let line_start = source
        .lines()
        .take(n - 1)
        .map(|l| l.len() + 1)
        .sum::<usize>();
    let bytes = source.as_bytes();
    let tight = |at: usize| at > 0 && !matches!(bytes[at - 1], b' ' | b'\t' | b'\n');
    let mut fixes: Vec<(usize, bool)> = Vec::new();
    let mut prev: Option<&Token> = None;
    for token in tokens {
        if matches!(token.token_type, TokenType::EOF) {
            break;
        }
        if token.line == n {
            match token.token_type {
                TokenType::OpenBrace => {
                    if let Some(p) = prev
                        && p.line == n
                        && tight(token.offset)
                        && !matches!(
                            p.token_type,
                            TokenType::OpenBrace | TokenType::OpenParen | TokenType::OpenBracket
                        )
                    {
                        fixes.push((token.offset - line_start, true));
                    }
                }
                TokenType::CloseBrace => {
                    if let Some(p) = prev
                        && p.line == n
                        && tight(token.offset)
                        && !matches!(p.token_type, TokenType::OpenBrace)
                    {
                        // `x}` → `x }` for a block closed on its line.
                        fixes.push((token.offset - line_start, true));
                    }
                }
                _ => {
                    if let Some(p) = prev
                        && p.line == n
                        && matches!(p.token_type, TokenType::CloseBrace)
                        && tight(token.offset)
                        && !is_closer(&token.token_type)
                        && !matches!(
                            token.token_type,
                            TokenType::Comma | TokenType::Dot | TokenType::Semicolon
                        )
                    {
                        fixes.push((token.offset - line_start, true));
                    }
                }
            }
            prev = Some(token);
        } else if token.line > n {
            break;
        }
    }
    if fixes.is_empty() {
        return body.to_string();
    }
    // Columns are of the original line; the body was trimmed at the start.
    let lead = source
        .lines()
        .nth(n - 1)
        .map(|l| l.len() - l.trim_start().len())
        .unwrap_or(0);
    let mut text = body.to_string();
    fixes.sort_unstable();
    for (col, _) in fixes.into_iter().rev() {
        let at = col.saturating_sub(lead);
        if at <= text.len() && text.is_char_boundary(at) {
            text.insert(at, ' ');
        }
    }
    text.trim_end().to_string()
}

fn verify(original: &[Token], formatted: &str, file: &str) -> Result<()> {
    let again = LexerV2::with_layout(formatted, file, Layout::Braces).tokenize()?;
    let kinds = |ts: &[Token]| ts.iter().map(|t| t.token_type.clone()).collect::<Vec<_>>();
    if kinds(original) == kinds(&again) {
        return Ok(());
    }
    Err(WebFluentError::ParseError(
        Diagnostic::new(
            "Formatting would change what the file says, so it was left alone",
            file,
            1,
            1,
        )
        .with_hint("This is a formatter fault: please report the file"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_are_indented_to_their_depth_and_braces_spaced() {
        let src = "page Home(path: \"/\"){\nstate open = true\n  // a comment\n    Row(gap: .sm){\n      style {\n    padding: 6px 0\n  &:hover {\n background: $surface-hover\n }\n      }\n on click{ open = !open }\n Text(\"a\").bold   \n    }\n\n\n\nif open {\nSpinner\n}else {\nText(\"b\")\n}\n}\n";
        let out = format_source(src, "t.wf").unwrap();
        assert_eq!(
            out,
            "page Home(path: \"/\") {\n    state open = true\n    // a comment\n    Row(gap: .sm) {\n        style {\n            padding: 6px 0\n            &:hover {\n                background: $surface-hover\n            }\n        }\n        on click { open = !open }\n        Text(\"a\").bold\n    }\n\n    if open {\n        Spinner\n    } else {\n        Text(\"b\")\n    }\n}\n"
        );
    }

    #[test]
    fn a_file_without_a_final_newline_gains_one_and_nothing_else() {
        let src = "page P(path: \"/\") {\n    Text(\"a\")\n}";
        assert_eq!(format_source(src, "t.wf").unwrap(), format!("{src}\n"));
    }

    #[test]
    fn a_formatted_file_is_a_fixed_point() {
        let src = "page Home(path: \"/\") {\n    state user = {\n        name: \"a\",\n        tags: [\n            1,\n            2\n        ]\n    }\n    Card(\n        title: \"x\",\n        open: true\n    ) {\n        Text(\"b\")\n    }\n}\n";
        let once = format_source(src, "t.wf").unwrap();
        assert_eq!(once, src);
        assert_eq!(format_source(&once, "t.wf").unwrap(), once);
    }

    #[test]
    fn a_value_carried_over_a_line_is_left_as_written() {
        let src = "page P(path: \"/\") {\n    Card {\n        style {\n            box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1),\n                        0 2px 8px rgba(0, 0, 0, 0.2)\n        }\n    }\n}\n";
        assert_eq!(format_source(src, "t.wf").unwrap(), src);
    }

    #[test]
    fn an_indented_file_is_normalised_through_its_braced_spelling() {
        let src =
            "page Home(path: \"/\")\n  state open = true\n  Row(gap: .sm)\n      Text(\"a\")\n";
        assert_eq!(
            format_source(src, "t.wfx").unwrap(),
            "page Home(path: \"/\")\n    state open = true\n    Row(gap: .sm)\n        Text(\"a\")\n"
        );
    }
}
