//! The build reads back the JavaScript it wrote.
//!
//! A program the compiler accepts should never reach the browser as a
//! script the browser refuses — but nothing used to check, so a code
//! generator bug shipped as a `SyntaxError` that stopped the whole page
//! before it drew. Two `state a` in one page compiled to two `const _a`, and
//! the build said "complete".
//!
//! This is a scanner, as the minifier is, and shares its idea of where a
//! string, a template literal, a regular expression and a comment end. It
//! checks what a scanner can be sure of: that every string, template,
//! regular expression and comment is closed, that every bracket closes the
//! one it should, and that no block declares one name twice with `const`,
//! `let` or `class` — the `Identifier has already been declared` that a
//! clash of generated names produces. A fault it finds is the compiler's,
//! never the author's, and is reported as one.

use crate::codegen::minify::{
    is_expression_keyword, is_word_byte, regex_end, string_end, template_end,
};
use std::collections::HashMap;

/// Where the JavaScript is wrong, and how.
#[derive(Debug, Clone, PartialEq)]
pub struct JsFault {
    /// 1-based line of the fault.
    pub line: usize,
    pub message: String,
}

/// One `{ … }` block: the names it declares, and how deep in parentheses
/// it opened — a `for (const x of …)` head declares into the loop, not into
/// the block around it.
struct Block {
    names: HashMap<String, (&'static str, usize)>,
    parens: usize,
}

/// Check `source`; `Ok` when nothing is wrong that a scanner can see.
pub fn check_js(source: &str) -> Result<(), JsFault> {
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut line = 1;
    let mut brackets: Vec<(u8, usize)> = Vec::new();
    let mut blocks = vec![Block {
        names: HashMap::new(),
        parens: 0,
    }];
    let mut parens = 0usize;
    let mut last: Option<u8> = None;
    let mut last_was_word = false;
    let mut last_word = String::new();
    let lines_in = |from: usize, to: usize| bytes[from..to].iter().filter(|&&b| b == b'\n').count();
    let fault = |line: usize, message: String| Err(JsFault { line, message });

    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'\n' => {
                line += 1;
                i += 1;
            }
            b' ' | b'\t' | b'\r' => i += 1,
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let Some(end) = source[i + 2..].find("*/") else {
                    return fault(line, "a comment that never closes".into());
                };
                let end = i + 2 + end + 2;
                line += lines_in(i, end);
                i = end;
            }
            b'"' | b'\'' => {
                let end = string_end(bytes, i);
                if end > bytes.len() || end < i + 2 || bytes[end - 1] != c {
                    return fault(line, "a string that never closes".into());
                }
                i = end;
                last = Some(c);
                last_was_word = true;
                last_word.clear();
            }
            b'`' => {
                let end = template_end(bytes, i);
                if end > bytes.len() || bytes[end - 1] != b'`' || end == i + 1 {
                    return fault(line, "a template literal that never closes".into());
                }
                line += lines_in(i, end);
                i = end;
                last = Some(c);
                last_was_word = true;
                last_word.clear();
            }
            b'/' if (!last_was_word || is_expression_keyword(&last_word))
                && !matches!(last, Some(b')' | b']')) =>
            {
                let end = regex_end(bytes, i).min(bytes.len());
                // Closed when, flags aside, it ends in its own second `/`.
                let body = &bytes[i + 1..end];
                let flags = body
                    .iter()
                    .rev()
                    .take_while(|b| b.is_ascii_alphabetic())
                    .count();
                if body[..body.len() - flags].last() != Some(&b'/') {
                    return fault(line, "a regular expression that never closes".into());
                }
                i = end;
                last = Some(b'/');
                last_was_word = true;
                last_word.clear();
            }
            b'(' | b'[' | b'{' => {
                brackets.push((c, line));
                if c == b'(' {
                    parens += 1;
                } else if c == b'{' {
                    blocks.push(Block {
                        names: HashMap::new(),
                        parens,
                    });
                }
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
            b')' | b']' | b'}' => {
                let want = match c {
                    b')' => b'(',
                    b']' => b'[',
                    _ => b'{',
                };
                match brackets.pop() {
                    Some((open, _)) if open == want => {}
                    Some((open, at)) => {
                        return fault(
                            line,
                            format!(
                                "`{}` closes the `{}` opened at line {at}",
                                c as char, open as char
                            ),
                        );
                    }
                    None => return fault(line, format!("`{}` closes nothing", c as char)),
                }
                if c == b')' {
                    parens = parens.saturating_sub(1);
                } else if c == b'}' && blocks.len() > 1 {
                    blocks.pop();
                }
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
            _ if is_word_byte(c) => {
                let start = i;
                while i < bytes.len() && is_word_byte(bytes[i]) {
                    i += 1;
                }
                let word = &source[start..i];
                // Property access (`x.class`) is not a declaration.
                let after_dot = last == Some(b'.');
                let kind = match word {
                    "const" => Some("const"),
                    "let" => Some("let"),
                    "class" => Some("class"),
                    "function" => Some("function"),
                    _ => None,
                };
                if let Some(kind) = kind.filter(|_| !after_dot) {
                    let block = blocks.last_mut().expect("the top block is never popped");
                    if block.parens == parens {
                        let mut j = i;
                        while j < bytes.len() && matches!(bytes[j], b' ' | b'\t') {
                            j += 1;
                        }
                        let name_start = j;
                        while j < bytes.len() && is_word_byte(bytes[j]) {
                            j += 1;
                        }
                        let name = &source[name_start..j];
                        let is_name = !name.is_empty() && !name.as_bytes()[0].is_ascii_digit();
                        if is_name {
                            // Two functions of one name are allowed; any
                            // other pair in one block is not.
                            if let Some(&(first, at)) = block.names.get(name)
                                && !(first == "function" && kind == "function")
                            {
                                return fault(
                                    line,
                                    format!(
                                        "`{name}` is declared twice in one block (`{first}` at line {at}, `{kind}` here)"
                                    ),
                                );
                            }
                            block.names.insert(name.to_string(), (kind, line));
                        }
                    }
                }
                last = word.bytes().last();
                last_was_word = true;
                last_word.clear();
                last_word.push_str(word);
            }
            _ => {
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
        }
    }
    if let Some((open, at)) = brackets.pop() {
        return fault(
            at,
            format!("the `{}` opened here never closes", open as char),
        );
    }
    Ok(())
}

/// The line of `source` a fault points at, trimmed, for a message.
pub fn line_of(source: &str, line: usize) -> String {
    let text = source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim();
    if text.chars().count() > 120 {
        format!("{}…", text.chars().take(120).collect::<String>())
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fault(src: &str) -> String {
        check_js(src).expect_err("a fault").message
    }

    #[test]
    fn what_the_compiler_writes_passes() {
        let ok = r#"
            "use strict";
            const WF = (() => { const a = /[}{]/g; return { a }; })();
            function Page_Home(params) {
              const _a = WF.signal(1);
              for (const x of [1]) { const y = x; }
              for (const x of [2]) { const y = x; }
              const t = `a ${ { b: 1 }.b } ${"}"}`;
              if (x) { let q = 1; } else { let q = 2; }
              return a / b / c;
            }
            function Page_Home(params) {}
            x.class = 1; obj.const = 2;
        "#;
        assert_eq!(check_js(ok), Ok(()));
    }

    #[test]
    fn a_name_declared_twice_in_one_block_is_found() {
        let msg = fault("function P() {\n  const _a = 1;\n  const _a = 2;\n}");
        assert!(msg.contains("`_a` is declared twice"), "{msg}");
        assert_eq!(check_js("const K = 1;\nlet K = 2;").unwrap_err().line, 2);
        assert!(fault("function f() {}\nconst f = 1;").contains("`f`"));
    }

    #[test]
    fn what_never_closes_is_found() {
        assert!(fault("const a = \"x;\n").contains("string"));
        assert!(fault("const a = `x;").contains("template"));
        assert!(fault("f(1;\n").contains("never closes"));
        assert!(fault("f(1]").contains("closes the `(`"));
        assert!(fault("}").contains("closes nothing"));
        assert!(fault("/* x").contains("comment"));
    }
}
