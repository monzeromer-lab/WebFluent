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
//! a browser would run is drawn as its words.
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
            let lang = rest.trim();
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
        // two spaces or a backslash breaks there (`\u{1}` marks it).
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
            if let Some(text) = t.strip_suffix('\\') {
                para.push(format!("{}\u{1}", text.trim_end()));
            } else if lines[i].ends_with("  ") {
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
/// `None` for a name the table does not have. A number that is no
/// character a page may hold — NUL, a surrogate, a control character, one
/// past U+10FFFF — is U+FFFD, as a browser draws it.
fn entity(body: &str) -> Option<char> {
    let Some(number) = body.strip_prefix('#') else {
        return ENTITIES.iter().find(|(n, _)| *n == body).map(|(_, c)| *c);
    };
    let code = match number.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16).ok()?,
        None => number.parse::<u32>().ok()?,
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

/// The inline spans of a line: code held aside, then images, links,
/// strong and emphasis.
fn inline(text: &str, options: Options) -> String {
    static CODE: OnceLock<Regex> = OnceLock::new();
    static IMAGE: OnceLock<Regex> = OnceLock::new();
    static LINK: OnceLock<Regex> = OnceLock::new();
    static PICTURE: OnceLock<Regex> = OnceLock::new();
    static WORDS: OnceLock<Regex> = OnceLock::new();
    static STRONG: OnceLock<Regex> = OnceLock::new();
    static EM: OnceLock<Regex> = OnceLock::new();
    static EM2: OnceLock<Regex> = OnceLock::new();
    static ENTITY: OnceLock<Regex> = OnceLock::new();
    let code = CODE.get_or_init(|| Regex::new(r"`([^`]+)`").unwrap());
    let image = IMAGE.get_or_init(|| Regex::new(r"!\[([^\]]*)\]\(([^)\s]+)\)").unwrap());
    let link = LINK.get_or_init(|| Regex::new(r"\[([^\]]+)\]\(([^)\s]+)\)").unwrap());
    // `.inert`'s: a picture with the space after it, and a link whose
    // words may be empty — what is left of a linked picture.
    let picture = PICTURE.get_or_init(|| Regex::new(r"!\[([^\]]*)\]\(([^)\s]+)\) ?").unwrap());
    let words = WORDS.get_or_init(|| Regex::new(r"\[([^\]]*)\]\(([^)\s]+)\)").unwrap());
    let strong = STRONG.get_or_init(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());
    let em = EM.get_or_init(|| Regex::new(r"\*([^*]+)\*").unwrap());
    let em2 = EM2.get_or_init(|| Regex::new(r"(^|[^A-Za-z0-9])_([^_]+)_($|[^A-Za-z0-9])").unwrap());
    let entity_re = ENTITY.get_or_init(|| {
        Regex::new(r"&amp;(#[0-9]{1,7}|#[xX][0-9a-fA-F]{1,6}|[A-Za-z][A-Za-z0-9]{1,31});").unwrap()
    });

    let escaped = escape(text);
    let mut spans: Vec<String> = Vec::new();
    let held = code.replace_all(&escaped, |c: &regex::Captures| {
        spans.push(format!("<code>{}</code>", &c[1]));
        format!("\u{0}{}\u{0}", spans.len() - 1)
    });
    let mut s = if options.inert {
        let s = picture.replace_all(&held, "");
        words.replace_all(&s, "${1}").to_string()
    } else {
        // A site-relative URL is addressed from the base path. The URL is
        // checked as it was escaped, which names the scheme it would
        // unescaped: escaping adds only `&` and `;`, neither of which a
        // scheme may hold.
        let base = options.base.trim_end_matches('/');
        let at = |url: &str| {
            if url.starts_with('/') && !url.starts_with("//") {
                format!("{base}{url}")
            } else {
                url.to_string()
            }
        };
        let safe = |url: &str| super::url::is_safe_with(url, options.schemes);
        let s = image.replace_all(&held, |c: &regex::Captures| {
            if safe(&c[2]) {
                format!("<img src=\"{}\" alt=\"{}\">", at(&c[2]), &c[1])
            } else {
                c[1].to_string()
            }
        });
        link.replace_all(&s, |c: &regex::Captures| {
            if safe(&c[2]) {
                format!("<a href=\"{}\">{}</a>", at(&c[2]), &c[1])
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
    // entities as written, as CommonMark's does.
    if options.inert {
        s = entity_re
            .replace_all(&s, |c: &regex::Captures| match entity(&c[1]) {
                Some(ch) => escape(&ch.to_string()),
                None => c[0].to_string(),
            })
            .to_string();
    }
    for (i, span) in spans.iter().enumerate() {
        s = s.replace(&format!("\u{0}{i}\u{0}"), span);
    }
    s.replace("\u{1}\n", "<br>\n").replace('\u{1}', "")
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
        // A name the table does not have, and a number that is no
        // character, are what a browser makes of them.
        assert_eq!(
            inert("&bogus; &#0; &#xD800; &#x110000; &#1; &#99999999;"),
            "<p>&amp;bogus; \u{FFFD} \u{FFFD} \u{FFFD} \u{FFFD} &amp;#99999999;</p>\n"
        );
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
