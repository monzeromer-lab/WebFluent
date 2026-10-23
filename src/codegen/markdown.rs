//! `Markdown(text)`: a small, predictable Markdown, rendered the same way
//! here at build time and by the runtime in the browser (`WF.markdown`),
//! so a hydrated page repaints what the static paint showed.
//!
//! Blocks: `#`–`######` headings, paragraphs (lines run together), fenced
//! code (```` ``` ````), `>` quotes, `-`/`*` and `1.` lists (one level),
//! `---` rules. Inline: `` `code` ``, `**strong**`, `*em*` or `_em_`,
//! `[text](url)`, `![alt](src)`. Every character of the text is escaped
//! first: raw HTML in the text is shown, not run. A link or picture goes
//! through the scheme check every other URL does (`codegen::url`), and one
//! a browser would run is drawn as its words. A backslash before ASCII
//! punctuation draws that character and nothing more, as CommonMark's
//! does: `\*`, `\[`, `\#` and `\\` are a star, a bracket, a hash and a
//! backslash, never emphasis, a link, a heading or an escape.
//!
//! `Markdown(text).inert` is for text somebody else wrote — a language
//! server's documentation, a `///` comment: its entities are decoded
//! before they are escaped, a link is drawn as its words and a picture is
//! left out, so nothing in it can be followed or fetched.

use regex::Regex;
use std::sync::OnceLock;

/// How a text is drawn.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options<'a> {
    /// The site's `base_path`, which a site-relative link or image
    /// (`/docs/x`, `/img.png`) is addressed from, as `Link(to:)` and
    /// `Image(src:)` are.
    pub base: &'a str,
    /// The app's own schemes (`meta.frame`), allowed in a link as they are
    /// everywhere else.
    pub schemes: &'a [String],
    /// `Markdown(text).inert`: text somebody else wrote.
    pub inert: bool,
}

/// The HTML of `text`, with a site-relative link or image addressed from
/// `base`, the site's `base_path`.
pub fn render_with_base(text: &str, base: &str) -> String {
    render_with(
        text,
        Options {
            base,
            ..Options::default()
        },
    )
}

/// The HTML of `text`.
pub fn render(text: &str) -> String {
    render_with(text, Options::default())
}

/// The HTML of `text`, drawn as `options` say.
pub fn render_with(text: &str, options: Options) -> String {
    // The two characters the renderer marks its own places with (a held
    // code span, a line break) cannot arrive in text somebody else wrote.
    let text: std::borrow::Cow<str> = if options.inert && text.contains(['\u{0}', '\u{1}']) {
        text.replace(['\u{0}', '\u{1}'], "\u{FFFD}").into()
    } else {
        text.into()
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("```") {
            // The language takes backslash escapes, as CommonMark's info
            // string does (```` ```c\+\+ ```` is `c++`); the code does not.
            let lang = unescape(rest.trim());
            let lang = lang.as_str();
            let mut raw = String::new();
            i += 1;
            while i < lines.len() && lines[i].trim() != "```" {
                raw.push_str(lines[i]);
                raw.push('\n');
                i += 1;
            }
            i += 1;
            // A fence in a language the highlighter knows is coloured.
            let code = if super::highlight::knows(lang) {
                super::highlight::highlight(&raw, lang)
            } else {
                escape(&raw)
            };
            if lang.is_empty() {
                out.push_str(&format!("<pre><code>{code}</code></pre>\n"));
            } else {
                out.push_str(&format!(
                    "<pre><code class=\"language-{}\">{code}</code></pre>\n",
                    escape(lang)
                ));
            }
            continue;
        }
        if trimmed == "---" || trimmed == "***" {
            out.push_str("<hr>\n");
            i += 1;
            continue;
        }
        if let Some((level, rest)) = heading(trimmed) {
            out.push_str(&format!("<h{level}>{}</h{level}>\n", inline(rest, options)));
            i += 1;
            continue;
        }
        if trimmed.starts_with('>') {
            let mut quoted: Vec<String> = Vec::new();
            while i < lines.len() && lines[i].trim().starts_with('>') {
                quoted.push(lines[i].trim()[1..].trim_start().to_string());
                i += 1;
            }
            out.push_str(&format!(
                "<blockquote>\n{}</blockquote>\n",
                render_with(&quoted.join("\n"), options)
            ));
            continue;
        }
        if let Some(item) = list_item(trimmed) {
            let ordered = item.1;
            let tag = if ordered { "ol" } else { "ul" };
            out.push_str(&format!("<{tag}>\n"));
            while i < lines.len()
                && let Some((text, o)) = list_item(lines[i].trim())
                && o == ordered
            {
                out.push_str(&format!("<li>{}</li>\n", inline(text, options)));
                i += 1;
            }
            out.push_str(&format!("</{tag}>\n"));
            continue;
        }
        // A paragraph runs to the next blank line or block.
        // Its lines run together, as CommonMark's do; a line that ends in
        // two spaces breaks there (`\u{1}` marks it), and so does one that
        // ends in a backslash, which `inline` sees before the newline.
        let mut para: Vec<String> = Vec::new();
        while i < lines.len() {
            let t = lines[i].trim();
            if t.is_empty()
                || t.starts_with("```")
                || t == "---"
                || t == "***"
                || heading(t).is_some()
                || t.starts_with('>')
                || list_item(t).is_some()
            {
                break;
            }
            if lines[i].ends_with("  ") {
                para.push(format!("{t}\u{1}"));
            } else {
                para.push(t.to_string());
            }
            i += 1;
        }
        if let Some(last) = para.last_mut() {
            *last = last.trim_end_matches('\u{1}').to_string();
        }
        let body = inline(&para.join("\n"), options);
        // A paragraph that held nothing but pictures has nothing left to
        // draw once they are left out.
        if options.inert && body.trim_matches([' ', '\t', '\n']).is_empty() {
            continue;
        }
        out.push_str(&format!("<p>{body}</p>\n"));
    }
    out
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &line[level..];
    if !rest.starts_with(' ') {
        return None;
    }
    Some((level, rest.trim()))
}

fn list_item(line: &str) -> Option<(&str, bool)> {
    if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
        return Some((rest.trim(), false));
    }
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && line[digits..].starts_with(". ") {
        return Some((line[digits + 2..].trim(), true));
    }
    None
}

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The named entities `.inert` decodes: those documentation is written
/// with. The runtime has the same table (`MD_NAMED` in `text.js`), and
/// `tests/build_and_runtime_agree.rs` holds the two to each other.
pub const ENTITIES: &[(&str, char)] = &[
    ("amp", '&'),
    ("lt", '<'),
    ("gt", '>'),
    ("quot", '"'),
    ("apos", '\''),
    ("nbsp", '\u{a0}'),
    ("ensp", '\u{2002}'),
    ("emsp", '\u{2003}'),
    ("thinsp", '\u{2009}'),
    ("shy", '\u{ad}'),
    ("zwj", '\u{200d}'),
    ("zwnj", '\u{200c}'),
    ("copy", '©'),
    ("reg", '®'),
    ("trade", '™'),
    ("mdash", '—'),
    ("ndash", '–'),
    ("minus", '−'),
    ("hellip", '…'),
    ("lsquo", '‘'),
    ("rsquo", '’'),
    ("ldquo", '“'),
    ("rdquo", '”'),
    ("laquo", '«'),
    ("raquo", '»'),
    ("bull", '•'),
    ("middot", '·'),
    ("times", '×'),
    ("divide", '÷'),
    ("plusmn", '±'),
    ("deg", '°'),
    ("le", '≤'),
    ("ge", '≥'),
    ("ne", '≠'),
    ("larr", '←'),
    ("rarr", '→'),
    ("uarr", '↑'),
    ("darr", '↓'),
    ("harr", '↔'),
    ("lArr", '⇐'),
    ("rArr", '⇒'),
    ("hArr", '⇔'),
    ("euro", '€'),
    ("pound", '£'),
    ("yen", '¥'),
    ("cent", '¢'),
    ("sect", '§'),
    ("para", '¶'),
];

/// The character an entity's body (`lt`, `#60`, `#x3C`) stands for, or
/// `None` for a name the table does not have. A number may have any
/// number of digits, as in a browser (`&#00060;` is `<`); one that is no
/// character a page should hold — NUL, a surrogate, a control character,
/// one past U+10FFFF — is U+FFFD.
fn entity(body: &str) -> Option<char> {
    let Some(number) = body.strip_prefix('#') else {
        return ENTITIES.iter().find(|(n, _)| *n == body).map(|(_, c)| *c);
    };
    // The pattern let only digits through, so a number that does not
    // parse is one too large for any character.
    let code = match number.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16).unwrap_or(u32::MAX),
        None => number.parse::<u32>().unwrap_or(u32::MAX),
    };
    let refused = code == 0
        || (code < 0x20 && !matches!(code, 0x9 | 0xA | 0xD))
        || (0x7F..=0x9F).contains(&code);
    Some(
        char::from_u32(code)
            .filter(|_| !refused)
            .unwrap_or('\u{FFFD}'),
    )
}

/// `text` with a backslash before each ASCII punctuation character
/// dropped: a fence's language, as CommonMark reads an info string.
fn unescape(text: &str) -> String {
    static ESCAPED: OnceLock<Regex> = OnceLock::new();
    let escaped = ESCAPED.get_or_init(|| Regex::new(r"\\([!-/:-@\[-`{-~])").unwrap());
    escaped.replace_all(text, "$1").to_string()
}

/// One thing [`hold`] set aside: its HTML, and whether it is a character
/// a backslash escaped (rather than a code span).
type Held = (String, bool);

/// `text` escaped, with its code spans and its backslash escapes held
/// aside, each as `\u{0}N\u{0}` where `held[N]` is its HTML, so that
/// nothing found after — a link, emphasis, an entity — can see into a
/// span or be made of an escaped character. One left-to-right pass, as
/// CommonMark reads them: `` \` `` opens no span, and a backslash in a
/// span is a backslash. A backslash before ASCII punctuation is that
/// character and nothing more; before a newline (or the `\u{1}` of a
/// line that ends in two spaces) it breaks the line, and the spaces
/// before it go; before anything else it is a backslash.
fn hold(text: &str) -> (String, Vec<Held>) {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut held: Vec<Held> = Vec::new();
    // Where the run not yet written out starts.
    let mut plain = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation) => {
                out.push_str(&escape(&text[plain..i]));
                held.push((escape(&text[i + 1..i + 2]), true));
                out.push_str(&format!("\u{0}{}\u{0}", held.len() - 1));
                i += 2;
                plain = i;
            }
            b'\\' if matches!(bytes.get(i + 1), Some(b'\n' | b'\x01')) => {
                out.push_str(&escape(&text[plain..i]));
                let kept = out
                    .trim_end_matches(|c: char| c.is_whitespace() && c != '\n')
                    .len();
                out.truncate(kept);
                if bytes[i + 1] == b'\n' {
                    out.push('\u{1}');
                }
                i += 1;
                plain = i;
            }
            b'`' => match bytes[i + 1..].iter().position(|b| *b == b'`') {
                Some(len) if len > 0 => {
                    out.push_str(&escape(&text[plain..i]));
                    let code = escape(&text[i + 1..i + 1 + len]);
                    held.push((format!("<code>{code}</code>"), false));
                    out.push_str(&format!("\u{0}{}\u{0}", held.len() - 1));
                    i += len + 2;
                    plain = i;
                }
                _ => i += 1,
            },
            _ => i += 1,
        }
    }
    out.push_str(&escape(&text[plain..]));
    (out, held)
}

/// `text` with what [`hold`] set aside put back: only the escaped
/// characters when `chars_only`, everything otherwise.
fn restore(text: &str, held: &[Held], chars_only: bool) -> String {
    static MARK: OnceLock<Regex> = OnceLock::new();
    let mark = MARK.get_or_init(|| Regex::new("\u{0}(0|[1-9][0-9]*)\u{0}").unwrap());
    if held.is_empty() {
        return text.to_string();
    }
    mark.replace_all(text, |c: &regex::Captures| {
        match c[1].parse::<usize>().ok().and_then(|n| held.get(n)) {
            Some((html, is_char)) if *is_char || !chars_only => html.clone(),
            _ => c[0].to_string(),
        }
    })
    .to_string()
}

/// The inline spans of a line: code and escaped characters held aside,
/// then images, links, strong and emphasis.
fn inline(text: &str, options: Options) -> String {
    static IMAGE: OnceLock<Regex> = OnceLock::new();
    static LINK: OnceLock<Regex> = OnceLock::new();
    static PICTURE: OnceLock<Regex> = OnceLock::new();
    static WORDS: OnceLock<Regex> = OnceLock::new();
    static STRONG: OnceLock<Regex> = OnceLock::new();
    static EM: OnceLock<Regex> = OnceLock::new();
    static EM2: OnceLock<Regex> = OnceLock::new();
    static ENTITY: OnceLock<Regex> = OnceLock::new();
    let image = IMAGE.get_or_init(|| Regex::new(r"!\[([^\[\]]*)\]\(([^)\s]+)\)").unwrap());
    let link = LINK.get_or_init(|| Regex::new(r"\[([^\[\]]+)\]\(([^)\s]+)\)").unwrap());
    // `.inert`'s: a picture with the space after it, and a link whose
    // words may be empty — what is left of a linked picture.
    let picture = PICTURE.get_or_init(|| Regex::new(r"!\[([^\[\]]*)\]\(([^)\s]+)\) ?").unwrap());
    let words = WORDS.get_or_init(|| Regex::new(r"\[([^\[\]]*)\]\(([^)\s]+)\)").unwrap());
    let strong = STRONG.get_or_init(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());
    let em = EM.get_or_init(|| Regex::new(r"\*([^*]+)\*").unwrap());
    let em2 = EM2.get_or_init(|| Regex::new(r"(^|[^A-Za-z0-9])_([^_]+)_($|[^A-Za-z0-9])").unwrap());
    let entity_re = ENTITY.get_or_init(|| {
        Regex::new(r"&amp;(#[0-9]+|#[xX][0-9a-fA-F]+|[A-Za-z][A-Za-z0-9]{1,31});").unwrap()
    });

    let (held_text, held) = hold(text);
    let mut s = if options.inert {
        let s = picture.replace_all(&held_text, "");
        words.replace_all(&s, "${1}").to_string()
    } else {
        // A site-relative URL is addressed from the base path. The URL is
        // checked as it was escaped, which names the scheme it would
        // unescaped: escaping adds only `&` and `;`, neither of which a
        // scheme may hold. It is checked with its escaped characters
        // as the characters they are (`javascript\:` is `javascript:`),
        // and written with them still held, so none of them is emphasis.
        let base = options.base.trim_end_matches('/');
        let at = |url: &str| {
            if url.starts_with('/') && !url.starts_with("//") {
                base
            } else {
                ""
            }
        };
        let safe = |url: &str| super::url::is_safe_with(url, options.schemes);
        let s = image.replace_all(&held_text, |c: &regex::Captures| {
            let url = restore(&c[2], &held, true);
            if safe(&url) {
                format!("<img src=\"{}{}\" alt=\"{}\">", at(&url), &c[2], &c[1])
            } else {
                c[1].to_string()
            }
        });
        link.replace_all(&s, |c: &regex::Captures| {
            let url = restore(&c[2], &held, true);
            if safe(&url) {
                format!("<a href=\"{}{}\">{}</a>", at(&url), &c[2], &c[1])
            } else {
                c[1].to_string()
            }
        })
        .to_string()
    };
    s = strong.replace_all(&s, "<strong>$1</strong>").to_string();
    s = em.replace_all(&s, "<em>$1</em>").to_string();
    s = em2.replace_all(&s, "$1<em>$2</em>$3").to_string();
    // An entity is decoded where it was escaped, after the spans are found,
    // so what it decodes to is a character and never Markdown: `&#42;` is
    // a star, not emphasis. Code is held aside still, and keeps its
    // entities as written, as CommonMark's does; so is an escaped
    // character, so `\&lt;` is the four characters it says.
    if options.inert {
        s = entity_re
            .replace_all(&s, |c: &regex::Captures| match entity(&c[1]) {
                Some(ch) => escape(&ch.to_string()),
                None => c[0].to_string(),
            })
            .to_string();
    }
    restore(&s, &held, false)
        .replace("\u{1}\n", "<br>\n")
        .replace('\u{1}', "")
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_site_relative_link_takes_the_base_path() {
        let out = super::render_with_base(
            "See [data](/docs/data#paths) and ![i](/i.png), not [x](https://a.b/c).",
            "/WebFluent/",
        );
        assert!(out.contains("href=\"/WebFluent/docs/data#paths\""), "{out}");
        assert!(out.contains("src=\"/WebFluent/i.png\""), "{out}");
        assert!(out.contains("href=\"https://a.b/c\""), "{out}");
        assert_eq!(
            super::render_with_base("[a](/x)", ""),
            super::render("[a](/x)")
        );
        // A protocol-relative URL is another site's, as the runtime has it.
        assert_eq!(
            super::render_with_base("[a](//b.c/d)", "/WebFluent"),
            "<p><a href=\"//b.c/d\">a</a></p>\n"
        );
    }

    use super::*;

    #[test]
    fn blocks_and_inline_spans_render() {
        let md = "# Title\n\nA *word* and **more**, `x < y` and [a link](https://x.y) plus ![alt](/i.png).\\\nSecond line,\nthe same paragraph.\n\n- one\n- two\n\n1. first\n2. second\n\n> quoted *text*\n\n---\n\n```js\nlet a = 1 < 2;\n```\n<script>alert(1)</script>";
        let html = render(md);
        assert_eq!(
            html,
            "<h1>Title</h1>\n<p>A <em>word</em> and <strong>more</strong>, <code>x &lt; y</code> and <a href=\"https://x.y\">a link</a> plus <img src=\"/i.png\" alt=\"alt\">.<br>\nSecond line,\nthe same paragraph.</p>\n<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n<ol>\n<li>first</li>\n<li>second</li>\n</ol>\n<blockquote>\n<p>quoted <em>text</em></p>\n</blockquote>\n<hr>\n<pre><code class=\"language-js\">let a = 1 &lt; 2;\n</code></pre>\n<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n"
        );
    }

    /// The cases the runtime's `WF.markdown` is held to as well
    /// (`tests/js/security.test.mjs`): two renderers of one Markdown, byte
    /// for byte.
    #[test]
    fn the_shared_cases_give_the_shared_answers() {
        let raw = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/markdown-cases.json"
        ))
        .expect("the case table");
        let cases: Vec<(String, serde_json::Value, String)> =
            serde_json::from_str(&raw).expect("the case table");
        for (text, options, expected) in cases {
            let options = Options {
                base: options["base"].as_str().unwrap_or(""),
                inert: options["inert"].as_bool().unwrap_or(false),
                ..Options::default()
            };
            assert_eq!(
                render_with(&text, options),
                expected,
                "{text:?} {options:?}"
            );
        }
    }

    fn inert(text: &str) -> String {
        render_with(
            text,
            Options {
                inert: true,
                ..Options::default()
            },
        )
    }

    #[test]
    fn a_link_or_picture_a_browser_would_run_is_drawn_as_its_words() {
        for (md, html) in [
            ("[x](javascript:alert(1))", "<p>x)</p>\n"),
            ("[x](JavaScript:alert(1))", "<p>x)</p>\n"),
            ("[x](JAVASCRIPT:alert(1))", "<p>x)</p>\n"),
            // A control character a browser strips before it reads the
            // scheme does not hide one.
            ("[x](\u{1f}JavaScript:alert(1))", "<p>x)</p>\n"),
            ("[x](vbscript:msgbox(1))", "<p>x)</p>\n"),
            ("[x](data:text/html;base64,PHNjcmlwdD4=)", "<p>x</p>\n"),
            ("![x](javascript:alert(1))", "<p>x)</p>\n"),
            (
                "![a dot](data:image/png;base64,iVBORw0KGgo=)",
                "<p>a dot</p>\n",
            ),
            ("**[bold](javascript:x)**", "<p><strong>bold</strong></p>\n"),
            // Leading whitespace is no URL at all: the text is drawn as written.
            (
                "[x]( JavaScript:alert(1))",
                "<p>[x]( JavaScript:alert(1))</p>\n",
            ),
        ] {
            assert_eq!(render(md), html, "{md}");
            assert_eq!(render_with_base(md, "/site"), html, "{md}");
        }
        for (md, html) in [
            (
                "[x](https://example.com/a)",
                "<p><a href=\"https://example.com/a\">x</a></p>\n",
            ),
            (
                "[mail](mailto:ada@example.com)",
                "<p><a href=\"mailto:ada@example.com\">mail</a></p>\n",
            ),
            ("[up](../a:b)", "<p><a href=\"../a:b\">up</a></p>\n"),
            (
                "![i](https://x.y/i.png)",
                "<p><img src=\"https://x.y/i.png\" alt=\"i\"></p>\n",
            ),
        ] {
            assert_eq!(render(md), html, "{md}");
        }
        assert_eq!(
            render_with_base("[a](/docs) ![b](/b.png) [c](javascript:x)", "/site/"),
            "<p><a href=\"/site/docs\">a</a> <img src=\"/site/b.png\" alt=\"b\"> c</p>\n"
        );
        // The app's own schemes are allowed here as they are everywhere.
        let schemes = vec!["wfpreview".to_string()];
        let options = Options {
            schemes: &schemes,
            ..Options::default()
        };
        assert_eq!(
            render_with("[p](wfpreview://site/)", options),
            "<p><a href=\"wfpreview://site/\">p</a></p>\n"
        );
        assert_eq!(render("[p](wfpreview://site/)"), "<p>p</p>\n");
    }

    #[test]
    fn plain_markdown_keeps_an_entity_as_written() {
        assert_eq!(
            render("&lt;b&gt; &amp; `&lt;`"),
            "<p>&amp;lt;b&amp;gt; &amp;amp; <code>&amp;lt;</code></p>\n"
        );
    }

    #[test]
    fn inert_text_decodes_its_entities_and_shows_what_they_say() {
        assert_eq!(
            inert(
                "Returns &lt;color&gt; or &#60;b&#62; &amp; &quot;q&quot; &#x27;s&#39; &mdash; &rArr;"
            ),
            "<p>Returns &lt;color&gt; or &lt;b&gt; &amp; &quot;q&quot; 's' — ⇒</p>\n"
        );
        // HTML, written or encoded, is shown and never run.
        for md in [
            "<script>alert(1)</script>",
            "&lt;script&gt;alert(1)&lt;/script&gt;",
            "&#60;script&#62;alert(1)&#60;/script&#62;",
        ] {
            assert_eq!(
                inert(md),
                "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n",
                "{md}"
            );
        }
        // Decoded once: `&amp;lt;` says `&lt;`.
        assert_eq!(inert("&amp;lt;"), "<p>&amp;lt;</p>\n");
        // What an entity decodes to is a character, not Markdown.
        assert_eq!(
            inert("&#42;&#42;x&#42;&#42; &#91;a&#93;(b)"),
            "<p>**x** [a](b)</p>\n"
        );
        // A name the table does not have is kept as written; a number
        // that is no character is U+FFFD, however many digits it has.
        assert_eq!(
            inert("&bogus; &#0; &#xD800; &#x110000; &#1; &#99999999; &#x123456789ABCDEF0;"),
            "<p>&amp;bogus; \u{FFFD} \u{FFFD} \u{FFFD} \u{FFFD} \u{FFFD} \u{FFFD}</p>\n"
        );
        // Leading zeros are digits like any other.
        assert_eq!(inert("&#00000060;&#x000003E;"), "<p>&lt;&gt;</p>\n");
        // The renderer's own markers cannot arrive in the text.
        assert_eq!(
            inert("a\u{0}0\u{0}b `c`"),
            "<p>a\u{FFFD}0\u{FFFD}b <code>c</code></p>\n"
        );
    }

    #[test]
    fn inert_text_keeps_its_entities_in_code_as_written() {
        assert_eq!(
            inert("`&lt;T&gt;` is &lt;T&gt;"),
            "<p><code>&amp;lt;T&amp;gt;</code> is &lt;T&gt;</p>\n"
        );
        assert_eq!(
            inert("```rust\nlet a = &lt;b&gt;; // <c>\n```"),
            "<pre><code class=\"language-rust\">let a = &amp;lt;b&amp;gt;; // &lt;c&gt;\n</code></pre>\n"
        );
    }

    #[test]
    fn inert_text_draws_a_link_as_its_words_and_leaves_a_picture_out() {
        assert_eq!(
            inert(
                "See ![logo](https://x.y/l.png) the [guide](https://x.y/g) or [run](javascript:x)."
            ),
            "<p>See the guide or run.</p>\n"
        );
        assert_eq!(
            inert("[**bold** words](/a)"),
            "<p><strong>bold</strong> words</p>\n"
        );
        // A paragraph of nothing but pictures, linked or not, is left out.
        assert_eq!(
            inert("[![build](https://ci/b.svg)](https://ci) ![x](/x.png)\n\nText"),
            "<p>Text</p>\n"
        );
        let html = inert("![a](/a.png) [b](/b) ![c](javascript:x)");
        assert!(!html.contains("<a") && !html.contains("<img"), "{html}");
    }

    /// Every ASCII punctuation character CommonMark lets a backslash
    /// escape.
    const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

    #[test]
    fn a_backslash_before_punctuation_draws_the_character_and_nothing_more() {
        for ch in PUNCTUATION.chars() {
            let shown = escape(&ch.to_string());
            for md in [format!("\\{ch}"), format!("a \\{ch} b")] {
                let html = render(&md).replace('\n', "");
                assert_eq!(
                    html,
                    format!("<p>{}</p>", md.replace(&format!("\\{ch}"), &shown)),
                    "{md}"
                );
                assert_eq!(inert(&md), render(&md), "{md}");
            }
        }
        let all: String = PUNCTUATION.chars().map(|c| format!("\\{c}")).collect();
        assert_eq!(
            render(&all),
            "<p>!&quot;#$%&amp;'()*+,-./:;&lt;=&gt;?@[\\]^_`{|}~</p>\n"
        );
        // An escaped mark at a line's start makes no block of it.
        assert_eq!(
            render(
                "\\# not a heading\n\\- not a list\n\\* nor this\n1\\. nor this\n\\> not a quote\n\\---\n\\```"
            ),
            "<p># not a heading\n- not a list\n* nor this\n1. nor this\n&gt; not a quote\n---\n```</p>\n"
        );
        // `\\` is one backslash, and the character after it is itself.
        assert_eq!(render("\\\\*a*"), "<p>\\<em>a</em></p>\n");
    }

    #[test]
    fn an_escaped_bracket_makes_no_link_and_an_escaped_star_no_emphasis() {
        assert_eq!(
            render("\\[not a link\\](https://x.y) !\\[nor a picture](/p.png)"),
            "<p>[not a link](https://x.y) ![nor a picture](/p.png)</p>\n"
        );
        assert_eq!(render("[a\\](b)"), "<p>[a](b)</p>\n");
        // Nor does it carry a link's words on into the next link.
        assert_eq!(
            render("[a\\](b) [c](d)"),
            "<p>[a](b) <a href=\"d\">c</a></p>\n"
        );
        // Inside a link's words it is a bracket like any other.
        assert_eq!(render("[a\\]b](/c)"), "<p><a href=\"/c\">a]b</a></p>\n");
        // An escaped `!` leaves a link, not a picture.
        assert_eq!(
            render("\\![a](/i.png)"),
            "<p>!<a href=\"/i.png\">a</a></p>\n"
        );
        assert_eq!(
            render("\\*not em\\* \\*\\*nor strong\\*\\* \\_nor this\\_"),
            "<p>*not em* **nor strong** _nor this_</p>\n"
        );
        assert_eq!(
            render("*a \\* b* and **c \\*\\* d** and _e\\_f_"),
            "<p><em>a * b</em> and <strong>c ** d</strong> and <em>e_f</em></p>\n"
        );
    }

    #[test]
    fn an_escaped_character_in_a_url_is_checked_as_the_character_it_is() {
        // `javascript\:` is `javascript:`, and refused as it is.
        for md in [
            "[x](javascript\\:alert(1))",
            "![x](javascript\\:alert(1))",
            "[x](JavaScript\\:alert\\(1\\))",
        ] {
            let html = render(md);
            assert!(
                !html.contains("<a") && !html.contains("<img"),
                "{md} → {html}"
            );
        }
        assert_eq!(render("[x](javascript\\:alert\\(1\\))"), "<p>x</p>\n");
        // The base path goes where the URL, read through its escapes, is
        // site-relative; an escaped character in one is no emphasis.
        assert_eq!(
            render_with_base(
                "[a](\\/docs) [b](/\\/b.c) [c](/\\*x\\*) [d](/e\\(1\\))",
                "/site"
            ),
            "<p><a href=\"/site/docs\">a</a> <a href=\"//b.c\">b</a> <a href=\"/site/*x*\">c</a> <a href=\"/site/e(1)\">d</a></p>\n"
        );
        // No markup comes through an escaped character, in an attribute
        // or out of one.
        assert_eq!(
            render("\\<b\\> ![a\\\"b](/i.png) [c](/x\\\"onclick=alert(1))"),
            "<p>&lt;b&gt; <img src=\"/i.png\" alt=\"a&quot;b\"> <a href=\"/x&quot;onclick=alert(1\">c</a>)</p>\n"
        );
    }

    #[test]
    fn a_backslash_before_anything_else_or_ending_a_block_is_shown() {
        assert_eq!(
            render("\\a \\n \\é C:\\Users\\me \\ x \\1"),
            "<p>\\a \\n \\é C:\\Users\\me \\ x \\1</p>\n"
        );
        // One that ends a line breaks it; one that ends the paragraph has
        // no line to break, and is shown.
        assert_eq!(render("a\\\nb\\"), "<p>a<br>\nb\\</p>\n");
        assert_eq!(render("a\\"), "<p>a\\</p>\n");
        assert_eq!(render("a\\  "), "<p>a\\</p>\n");
        assert_eq!(render("a \\\nb"), "<p>a<br>\nb</p>\n");
        assert_eq!(render("a\\  \nb"), "<p>a<br>\nb</p>\n");
        // An escaped backslash at the end of a line breaks nothing.
        assert_eq!(render("a\\\\\nb"), "<p>a\\\nb</p>\n");
        assert_eq!(render("a\\\\\\\nb"), "<p>a\\<br>\nb</p>\n");
        assert_eq!(render("a\\\\  \nb"), "<p>a\\<br>\nb</p>\n");
        assert_eq!(
            render("# Title\\\n\n- item\\\n\n> quoted\\"),
            "<h1>Title\\</h1>\n<ul>\n<li>item\\</li>\n</ul>\n<blockquote>\n<p>quoted\\</p>\n</blockquote>\n"
        );
    }

    #[test]
    fn a_backslash_in_code_or_a_fence_is_a_backslash() {
        assert_eq!(
            render("`a\\*b` and `C:\\`"),
            "<p><code>a\\*b</code> and <code>C:\\</code></p>\n"
        );
        // An escaped backtick opens no span; one in a span closes it.
        assert_eq!(render("\\`not code`"), "<p>`not code`</p>\n");
        assert_eq!(render("`a\\`b`"), "<p><code>a\\</code>b`</p>\n");
        // A span over a line's end keeps the backslash before it.
        assert_eq!(render("`a\\\nb`"), "<p><code>a\\\nb</code></p>\n");
        for md in [
            "```\n\\*a\\* \\\\ \\[x\\]\n```",
            "```\n\\*a\\* \\\\ \\[x\\]",
        ] {
            assert_eq!(
                render(md),
                "<pre><code>\\*a\\* \\\\ \\[x\\]\n</code></pre>\n",
                "{md}"
            );
            assert_eq!(inert(md), render(md), "{md}");
        }
        assert_eq!(
            render("```css\nmargin\\-top: 0\n```"),
            "<pre><code class=\"language-css\">margin\\-<span class=\"wf-tok-prop\">top</span>: <span class=\"wf-tok-num\">0</span>\n</code></pre>\n"
        );
        // The fence's language is read through its escapes, as
        // CommonMark's info string is.
        assert_eq!(
            render("```c\\+\\+\nx\\+\n```"),
            "<pre><code class=\"language-c++\">x\\+\n</code></pre>\n"
        );
    }

    #[test]
    fn inert_text_takes_escapes_and_an_escaped_ampersand_begins_no_entity() {
        assert_eq!(
            inert("\\&lt; &lt\\; \\&amp; \\&#60; &lt;"),
            "<p>&amp;lt; &amp;lt; &amp;amp; &amp;#60; &lt;</p>\n"
        );
        assert_eq!(
            inert("\\[a\\](https://x.y) \\![x](/x.png) [y](javascript\\:z)"),
            "<p>[a](https://x.y) !x y</p>\n"
        );
        // Plain Markdown shows the entity as written either way.
        assert_eq!(render("\\&lt; &lt;"), "<p>&amp;lt; &amp;lt;</p>\n");
    }

    /// What vscode-css-languageservice hands an editor for `border-radius`:
    /// its description and syntax put through `textToMarkedString`, which
    /// puts a backslash before `` \ ` * _ { } [ ] ( ) # + - . ! `` and
    /// writes `<` and `>` as entities (the picture's base64 cut short).
    const CSS_HOVER: &str = "Defines the radii of the outer border edge\\.\n\n![Baseline icon](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTgi) _Widely available across major browsers (Baseline since 2015)_\n\nSyntax: &lt;length\\-percentage \\[0,∞\\]&gt;\\{1,4\\} \\[ / &lt;length\\-percentage \\[0,∞\\]&gt;\\{1,4\\} \\]?\n\n[MDN Reference](https://developer.mozilla.org/docs/Web/CSS/Reference/Properties/border-radius)";

    #[test]
    fn a_language_servers_escaped_documentation_reads_as_it_was_meant() {
        assert_eq!(
            inert(CSS_HOVER),
            "<p>Defines the radii of the outer border edge.</p>\n<p><em>Widely available across major browsers (Baseline since 2015)</em></p>\n<p>Syntax: &lt;length-percentage [0,∞]&gt;{1,4} [ / &lt;length-percentage [0,∞]&gt;{1,4} ]?</p>\n<p>MDN Reference</p>\n"
        );
        assert_eq!(
            inert("Syntax: &lt;'margin\\-top'&gt;\\{1,4\\}"),
            "<p>Syntax: &lt;'margin-top'&gt;{1,4}</p>\n"
        );
        assert_eq!(
            inert("Syntax: &lt;single\\-transition&gt;\\#"),
            "<p>Syntax: &lt;single-transition&gt;#</p>\n"
        );
    }

    #[test]
    fn inert_text_is_otherwise_plain_markdown() {
        assert_eq!(inert("_Widely_ used"), "<p><em>Widely</em> used</p>\n");
        assert_eq!(
            inert(
                "# T\n\nA *b* **c** `d`  \nnext\n\n- a &amp; b\n1. one\n\n> q &gt; r\n\n---\n\n```\nx < y\n```"
            ),
            "<h1>T</h1>\n<p>A <em>b</em> <strong>c</strong> <code>d</code><br>\nnext</p>\n<ul>\n<li>a &amp; b</li>\n</ul>\n<ol>\n<li>one</li>\n</ol>\n<blockquote>\n<p>q &gt; r</p>\n</blockquote>\n<hr>\n<pre><code>x &lt; y\n</code></pre>\n"
        );
        // Text with nothing an inert render changes is drawn the same.
        let md = "# Title\n\nA *word* and **more**, `x < y`.\n\n- one\n\n> quoted";
        assert_eq!(inert(md), render(md));
    }
}
