//! Syntax colouring for `Code(…, language: "wf")` and a Markdown fence:
//! the code as HTML with a `<span class="wf-tok-…">` around each token
//! kind, rendered the same way here at build time and by the runtime in
//! the browser (`WF.highlight`), so a hydrated block repaints what the
//! static paint showed.
//!
//! The kinds, and the design token each takes its colour from:
//! `kw` (`--syntax-keyword`), `name` (`--syntax-function`: a capitalised
//! name — an element, a type, a case), `str` (`--syntax-string`), `num`
//! (`--syntax-number`: a number with its unit, a colour), `prop` (a name
//! followed by `:`), `tok` (`$token`), `cmt` (`--syntax-comment`),
//! `prompt` (a shell line's `$`). Everything else is the block's own
//! colour. Languages: `wf`/`wfx`, `json`, `bash`/`sh`/`shell`, `css`;
//! any other is escaped and left plain.

use super::markdown::escape;

const WF_KEYWORDS: &[&str] = &[
    "page",
    "component",
    "store",
    "theme",
    "app",
    "type",
    "enum",
    "const",
    "data",
    "animation",
    "test",
    "state",
    "persist",
    "derived",
    "effect",
    "action",
    "use",
    "resource",
    "event",
    "slot",
    "part",
    "if",
    "else",
    "for",
    "in",
    "by",
    "show",
    "match",
    "let",
    "return",
    "await",
    "try",
    "catch",
    "emit",
    "navigate",
    "log",
    "on",
    "style",
    "transition",
    "children",
    "every",
    "after",
    "from",
    "to",
    "cleanup",
    "expect",
    "not",
    "true",
    "false",
    "null",
    "loading",
    "error",
    "ready",
];

/// Whether `lang` is one the highlighter colours.
pub fn knows(lang: &str) -> bool {
    matches!(
        lang,
        "wf" | "wfx" | "json" | "bash" | "sh" | "shell" | "css"
    )
}

/// `code` as HTML, coloured as `lang`.
pub fn highlight(code: &str, lang: &str) -> String {
    match lang {
        "wf" | "wfx" => code_like(code, true),
        "css" => code_like(code, false),
        "json" => json(code),
        "bash" | "sh" | "shell" => shell(code),
        _ => escape(code),
    }
}

fn span(kind: &str, text: &str, out: &mut String) {
    out.push_str("<span class=\"wf-tok-");
    out.push_str(kind);
    out.push_str("\">");
    out.push_str(&escape(text));
    out.push_str("</span>");
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

/// WebFluent and CSS: comments, strings, `$tokens`, numbers with units,
/// `name:` props, keywords and capitalised names.
fn code_like(code: &str, keywords: bool) -> String {
    let chars: Vec<char> = code.chars().collect();
    let mut out = String::new();
    let mut plain = String::new();
    let mut i = 0;
    let flush = |plain: &mut String, out: &mut String| {
        if !plain.is_empty() {
            out.push_str(&escape(plain));
            plain.clear();
        }
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            flush(&mut plain, &mut out);
            let start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            span("cmt", &chars[start..i].iter().collect::<String>(), &mut out);
            continue;
        }
        // `#"…"#`: a raw string, however many hashes it was written with.
        if keywords
            && c == '#'
            && let Some(end) = raw_string_end(&chars, i)
        {
            flush(&mut plain, &mut out);
            span("str", &chars[i..end].iter().collect::<String>(), &mut out);
            i = end;
            continue;
        }
        if c == '"' {
            flush(&mut plain, &mut out);
            let start = i;
            // `"""…"""`: a block string, over as many lines as it likes.
            if keywords && chars.get(i + 1) == Some(&'"') && chars.get(i + 2) == Some(&'"') {
                i += 3;
                while i + 2 < chars.len()
                    && !(chars[i] == '"' && chars[i + 1] == '"' && chars[i + 2] == '"')
                {
                    i += 1;
                }
                i = (i + 3).min(chars.len());
                span("str", &chars[start..i].iter().collect::<String>(), &mut out);
                continue;
            }
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i = (i + 1).min(chars.len());
            span("str", &chars[start..i].iter().collect::<String>(), &mut out);
            continue;
        }
        if c == '$' && chars.get(i + 1).is_some_and(|c| is_word(*c)) {
            flush(&mut plain, &mut out);
            let start = i;
            i += 1;
            while i < chars.len() && is_word(chars[i]) {
                i += 1;
            }
            span("tok", &chars[start..i].iter().collect::<String>(), &mut out);
            continue;
        }
        if c == '#' && chars.get(i + 1).is_some_and(|c| c.is_ascii_hexdigit()) {
            flush(&mut plain, &mut out);
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_ascii_hexdigit() {
                i += 1;
            }
            span("num", &chars[start..i].iter().collect::<String>(), &mut out);
            continue;
        }
        if c.is_ascii_digit() && !(i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_'))
        {
            flush(&mut plain, &mut out);
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '%') {
                i += 1;
            }
            span("num", &chars[start..i].iter().collect::<String>(), &mut out);
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || (!keywords && chars[i] == '-'))
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            let after_dot = start > 0 && chars[start - 1] == '.';
            let kind = if chars.get(j) == Some(&':') && chars.get(j + 1) != Some(&':') && !after_dot
            {
                Some("prop")
            } else if keywords && !after_dot && WF_KEYWORDS.contains(&word.as_str()) {
                Some("kw")
            } else if keywords && word.chars().next().is_some_and(char::is_uppercase) {
                Some("name")
            } else {
                None
            };
            match kind {
                Some(kind) => {
                    flush(&mut plain, &mut out);
                    span(kind, &word, &mut out);
                }
                None => plain.push_str(&word),
            }
            continue;
        }
        plain.push(c);
        i += 1;
    }
    flush(&mut plain, &mut out);
    out
}

/// The index just past the closing `"#…` of the raw string opening at
/// `at`, when one opens there.
fn raw_string_end(chars: &[char], at: usize) -> Option<usize> {
    let mut hashes = 0;
    let mut i = at;
    while chars.get(i) == Some(&'#') {
        hashes += 1;
        i += 1;
    }
    if chars.get(i) != Some(&'"') {
        return None;
    }
    i += 1;
    while i < chars.len() {
        if chars[i] == '"' && (1..=hashes).all(|n| chars.get(i + n) == Some(&'#')) {
            return Some(i + hashes + 1);
        }
        i += 1;
    }
    Some(chars.len())
}

/// JSON: a string before `:` is a key, any other a string; numbers,
/// `true`/`false`/`null`.
fn json(code: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    let mut out = String::new();
    let mut plain = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            if !plain.is_empty() {
                out.push_str(&escape(&plain));
                plain.clear();
            }
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i = (i + 1).min(chars.len());
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            let kind = if chars.get(j) == Some(&':') {
                "prop"
            } else {
                "str"
            };
            span(kind, &chars[start..i].iter().collect::<String>(), &mut out);
            continue;
        }
        if c.is_ascii_digit() || (c == '-' && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit()))
        {
            if !plain.is_empty() {
                out.push_str(&escape(&plain));
                plain.clear();
            }
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | 'e' | 'E' | '-' | '+'))
            {
                i += 1;
            }
            span("num", &chars[start..i].iter().collect::<String>(), &mut out);
            continue;
        }
        if c.is_ascii_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if matches!(word.as_str(), "true" | "false" | "null") {
                if !plain.is_empty() {
                    out.push_str(&escape(&plain));
                    plain.clear();
                }
                span("kw", &word, &mut out);
            } else {
                plain.push_str(&word);
            }
            continue;
        }
        plain.push(c);
        i += 1;
    }
    if !plain.is_empty() {
        out.push_str(&escape(&plain));
    }
    out
}

/// A shell transcript: a leading `$ ` is the prompt, a `#` line a comment.
fn shell(code: &str) -> String {
    let mut out = String::new();
    for (n, line) in code.split('\n').enumerate() {
        if n > 0 {
            out.push('\n');
        }
        if let Some(rest) = line.strip_prefix("$ ") {
            span("prompt", "$ ", &mut out);
            out.push_str(&escape(rest));
        } else if line.trim_start().starts_with('#') {
            span("cmt", line, &mut out);
        } else {
            out.push_str(&escape(line));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webfluent_is_coloured_by_kind() {
        let out = highlight(
            "page Home(path: \"/\") { // hi\n    state n = 0\n    Button(\"Go\", tone: .primary).lg { style { padding: $md; color: #FF0 } }\n}",
            "wf",
        );
        assert_eq!(
            out,
            "<span class=\"wf-tok-kw\">page</span> <span class=\"wf-tok-name\">Home</span>(<span class=\"wf-tok-prop\">path</span>: <span class=\"wf-tok-str\">&quot;/&quot;</span>) { <span class=\"wf-tok-cmt\">// hi</span>\n    <span class=\"wf-tok-kw\">state</span> n = <span class=\"wf-tok-num\">0</span>\n    <span class=\"wf-tok-name\">Button</span>(<span class=\"wf-tok-str\">&quot;Go&quot;</span>, <span class=\"wf-tok-prop\">tone</span>: .primary).lg { <span class=\"wf-tok-kw\">style</span> { <span class=\"wf-tok-prop\">padding</span>: <span class=\"wf-tok-tok\">$md</span>; <span class=\"wf-tok-prop\">color</span>: <span class=\"wf-tok-num\">#FF0</span> } }\n}"
        );
    }

    #[test]
    fn the_raw_and_block_forms_are_one_string_each() {
        assert_eq!(
            highlight("const S = #\"a \"b\" c\"#", "wf"),
            "<span class=\"wf-tok-kw\">const</span> <span class=\"wf-tok-name\">S</span> = <span class=\"wf-tok-str\">#&quot;a &quot;b&quot; c&quot;#</span>"
        );
        assert_eq!(
            highlight("x = \"\"\"\n  a \"b\"\n  \"\"\"", "wf"),
            "x = <span class=\"wf-tok-str\">&quot;&quot;&quot;\n  a &quot;b&quot;\n  &quot;&quot;&quot;</span>"
        );
        // A `#` colour in CSS is still a colour, not a raw string.
        assert_eq!(
            highlight("color: #FF0", "css"),
            "<span class=\"wf-tok-prop\">color</span>: <span class=\"wf-tok-num\">#FF0</span>"
        );
    }

    #[test]
    fn json_and_shell_and_the_unknown() {
        assert_eq!(
            highlight("{ \"a\": [1, true], \"b\": \"x\" }", "json"),
            "{ <span class=\"wf-tok-prop\">&quot;a&quot;</span>: [<span class=\"wf-tok-num\">1</span>, <span class=\"wf-tok-kw\">true</span>], <span class=\"wf-tok-prop\">&quot;b&quot;</span>: <span class=\"wf-tok-str\">&quot;x&quot;</span> }"
        );
        assert_eq!(
            highlight("$ wf build\n# then\nwf serve", "bash"),
            "<span class=\"wf-tok-prompt\">$ </span>wf build\n<span class=\"wf-tok-cmt\"># then</span>\nwf serve"
        );
        assert_eq!(highlight("a < b", "python"), "a &lt; b");
        // A property read after a dot is not a prop, a keyword or a flag.
        assert_eq!(highlight("t.title", "wf"), "t.title");
    }
}
