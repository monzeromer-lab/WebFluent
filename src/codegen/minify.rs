//! Minification of the bundle and the stylesheet.
//!
//! `build.minify` has been `true` in every project's config since `wf init`
//! first wrote one, and nothing read it: the bundle shipped with its comments
//! and its indentation. This is the reader.
//!
//! The JavaScript minifier removes comments and whitespace, and nothing else:
//! it does not rename, reorder or rewrite, so a stack trace still names the
//! functions the compiler wrote. It is a scanner, not a parser — enough to
//! know where a string, a template literal, a regular expression or a comment
//! starts and ends, which is all that whitespace removal needs to get right.
//! A newline is kept wherever dropping it could change the program under
//! automatic semicolon insertion (`x = {}` then `foo()` on the next line), so
//! the output runs exactly as the input did.
//!
//! The CSS minifier removes comments and whitespace outside strings, and the
//! last semicolon of each block.

/// Minify JavaScript source.
pub fn minify_js(source: &str) -> String {
    let mut out = String::with_capacity(source.len() / 2);
    let bytes = source.as_bytes();
    let mut i = 0;
    // The last significant character written, for whitespace and regex decisions.
    let mut last: Option<char> = None;
    // Whether the last token was a word (identifier, keyword, number), which
    // decides whether a `/` starts a regular expression or divides — unless
    // the word is a keyword an expression can follow, after which it is a
    // regular expression.
    let mut last_was_word = false;
    let mut last_word = String::new();
    // A pending newline or space, written only if what follows needs it.
    let mut pending_newline = false;
    let mut pending_space = false;

    while i < bytes.len() {
        let c = bytes[i] as char;

        // Comments.
        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let end = source[i + 2..]
                .find("*/")
                .map(|e| i + 2 + e + 2)
                .unwrap_or(bytes.len());
            i = end;
            continue;
        }

        // Whitespace is deferred until the next token says what it needs.
        if c == '\n' || c == '\r' {
            pending_newline = true;
            i += 1;
            continue;
        }
        if c == ' ' || c == '\t' {
            pending_space = true;
            i += 1;
            continue;
        }

        // A token starts here. Decide what separates it from the last one.
        let next_is_word = is_word_char(c);
        if pending_newline {
            // Safe to drop after an opener or a separator, and before a closer.
            let droppable_after = matches!(last, Some(';' | '{' | ',' | '(' | '[') | None);
            let droppable_before = matches!(c, '}' | ')' | ']');
            if !(droppable_after || droppable_before) {
                out.push('\n');
            }
        } else if pending_space {
            let last_char = last.unwrap_or(' ');
            let both_words = is_word_char(last_char) && next_is_word;
            // `a + +b` and `a - -b` must keep their space; so must `a in b`.
            let same_sign = matches!((last_char, c), ('+', '+') | ('-', '-'));
            if both_words || same_sign {
                out.push(' ');
            }
        }
        pending_newline = false;
        pending_space = false;

        match c {
            '"' | '\'' => {
                let end = string_end(bytes, i);
                out.push_str(&source[i..end]);
                i = end;
                last = Some(c);
                last_was_word = true; // a string ends a value, as a word does
                last_word.clear();
            }
            '`' => {
                let end = template_end(bytes, i);
                out.push_str(&source[i..end]);
                i = end;
                last = Some('`');
                last_was_word = true;
            }
            '/' if (!last_was_word || is_expression_keyword(&last_word))
                && !matches!(last, Some(')' | ']')) =>
            {
                let end = regex_end(bytes, i);
                out.push_str(&source[i..end]);
                i = end;
                last = Some('/');
                last_was_word = true;
            }
            _ if next_is_word => {
                let start = i;
                while i < bytes.len() && is_word_byte(bytes[i]) {
                    i += 1;
                }
                out.push_str(&source[start..i]);
                last = source[start..i].chars().last();
                last_was_word = true;
                last_word.clear();
                last_word.push_str(&source[start..i]);
            }
            _ => {
                out.push(c);
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
        }
    }
    out
}

/// A keyword after which a `/` begins a regular expression, not a division.
fn is_expression_keyword(word: &str) -> bool {
    matches!(
        word,
        "return"
            | "typeof"
            | "case"
            | "in"
            | "of"
            | "delete"
            | "void"
            | "throw"
            | "new"
            | "instanceof"
    )
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$' || !c.is_ascii()
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$' || b >= 0x80
}

/// One past the closing quote of the string starting at `start`.
fn string_end(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b if b == quote => return i + 1,
            b'\n' => return i, // unterminated; leave the rest alone
            _ => i += 1,
        }
    }
    bytes.len()
}

/// One past the closing backtick of the template literal starting at `start`.
fn template_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'`' => return i + 1,
            b'$' if i + 1 < bytes.len() && bytes[i + 1] == b'{' => {
                i = expression_end(bytes, i + 2);
            }
            _ => i += 1,
        }
    }
    bytes.len()
}

/// One past the `}` that closes a `${` expression opened just before `start`.
fn expression_end(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => i = string_end(bytes, i),
            b'`' => i = template_end(bytes, i),
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => i += 1,
        }
    }
    bytes.len()
}

/// One past the end of the regular expression literal starting at `start`,
/// flags included.
fn regex_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    let mut in_class = false;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'[' => {
                in_class = true;
                i += 1;
            }
            b']' => {
                in_class = false;
                i += 1;
            }
            b'/' if !in_class => {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
                return i;
            }
            b'\n' => return i,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Minify a stylesheet.
pub fn minify_css(source: &str) -> String {
    let mut out = String::with_capacity(source.len() / 2);
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut pending_space = false;
    // Inside `calc(100% + 1px)` a `+` is arithmetic and its spaces are
    // significant; outside parentheses it is a combinator.
    let mut parens = 0usize;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let end = source[i + 2..]
                .find("*/")
                .map(|e| i + 2 + e + 2)
                .unwrap_or(bytes.len());
            i = end;
            continue;
        }
        if c.is_ascii_whitespace() {
            pending_space = true;
            i += 1;
            continue;
        }
        if pending_space {
            let last = out.chars().last().unwrap_or('{');
            // No space is needed next to a structural character, or around a
            // combinator; anywhere else it separates two words of a value.
            let structural_last = matches!(last, '{' | '}' | ';' | ':' | ',' | '(')
                || (parens == 0 && matches!(last, '>' | '+' | '~'));
            let structural_next = matches!(c, '{' | '}' | ';' | ',' | ')')
                || (parens == 0 && matches!(c, '>' | '+' | '~'));
            if !structural_last && !structural_next {
                out.push(' ');
            }
            pending_space = false;
        }
        match c {
            '"' | '\'' => {
                let end = string_end(bytes, i);
                out.push_str(&source[i..end]);
                i = end;
            }
            '(' => {
                parens += 1;
                out.push(c);
                i += 1;
            }
            ')' => {
                parens = parens.saturating_sub(1);
                out.push(c);
                i += 1;
            }
            '}' => {
                // The last declaration's semicolon is optional.
                if out.ends_with(';') {
                    out.pop();
                }
                out.push('}');
                i += 1;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_and_indentation_go_and_the_program_survives() {
        let src = "// header\nfunction f(a, b) {\n    // inner\n    const x = a + b; /* sum */\n    return x * 2;\n}\n";
        assert_eq!(minify_js(src), "function f(a,b){const x=a+b;return x*2;}");
    }

    #[test]
    fn strings_templates_and_regexes_are_kept_verbatim() {
        let src = "const s = \"a // not a comment\";\nconst t = `x ${ f({a: 1}) } // y`;\nconst r = path.replace(/\\/$/, \"\");\nconst d = a / b / c;\n";
        let out = minify_js(src);
        assert!(out.contains("\"a // not a comment\""), "{out}");
        assert!(out.contains("`x ${ f({a: 1}) } // y`"), "{out}");
        assert!(out.contains("replace(/\\/$/,\"\")"), "{out}");
        assert!(out.contains("d=a/b/c"), "{out}");
        assert_eq!(minify_js("return /a\"b/.test(s);"), "return/a\"b/.test(s);");
    }

    #[test]
    fn newlines_that_asi_relies_on_are_kept() {
        let src = "const x = {}\nfoo()\nlet y = 1\n(a)\n";
        let out = minify_js(src);
        assert_eq!(out, "const x={}\nfoo()\nlet y=1\n(a)");
    }

    #[test]
    fn newlines_after_a_semicolon_or_brace_go() {
        let src = "a();\nb();\nif (x) {\n  c();\n}\nd();\n";
        assert_eq!(minify_js(src), "a();b();if(x){c();}\nd();");
    }

    #[test]
    fn a_space_between_words_and_between_signs_stays() {
        assert_eq!(minify_js("return a in b;"), "return a in b;");
        assert_eq!(minify_js("x = a + +b - -c;"), "x=a+ +b- -c;");
        assert_eq!(minify_js("const  e = new  Foo ( ) ;"), "const e=new Foo();");
    }

    #[test]
    fn css_loses_comments_whitespace_and_trailing_semicolons() {
        let src = "/* c */\n.a > .b, .c {\n  color: red;\n  content: \" { \";\n}\n@media (max-width: 600px) { .a { display: none; } }\n";
        assert_eq!(
            minify_css(src),
            ".a>.b,.c{color:red;content:\" { \"}@media (max-width:600px){.a{display:none}}"
        );
    }

    #[test]
    fn css_values_keep_their_inner_spaces() {
        assert_eq!(
            minify_css(".a { font: 400 14px/1.2 var(--font-sans); margin: 0 auto; }"),
            ".a{font:400 14px/1.2 var(--font-sans);margin:0 auto}"
        );
        assert_eq!(
            minify_css(".a { width: calc(100% - 2 * var(--x)); left: calc(50% + 4px); }"),
            ".a{width:calc(100% - 2 * var(--x));left:calc(50% + 4px)}"
        );
        assert_eq!(minify_css(".a + .b > .c ~ .d { x: 1 }"), ".a+.b>.c~.d{x:1}");
    }
}
