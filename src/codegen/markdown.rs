//! `Markdown(text)`: a small, predictable Markdown, rendered the same way
//! here at build time and by the runtime in the browser (`WF.markdown`),
//! so a hydrated page repaints what the static paint showed.
//!
//! Blocks: `#`–`######` headings, paragraphs (lines run together), fenced
//! code (```` ``` ````), `>` quotes, `-`/`*` and `1.` lists (one level),
//! `---` rules. Inline: `` `code` ``, `**strong**`, `*em*` or `_em_`,
//! `[text](url)`, `![alt](src)`. Every character of the text is escaped
//! first: raw HTML in the text is shown, not run.

use regex::Regex;
use std::sync::OnceLock;

/// The HTML of `text`, with a site-relative link or image (`/docs/x`,
/// `/img.png`) addressed from `base` — the site's `base_path` — as the
/// element `Link(to:)` and `Image(src:)` are.
pub fn render_with_base(text: &str, base: &str) -> String {
    let html = render(text);
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        return html;
    }
    html.replace("href=\"/", &format!("href=\"{base}/"))
        .replace("src=\"/", &format!("src=\"{base}/"))
}

/// The HTML of `text`.
pub fn render(text: &str) -> String {
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
            out.push_str(&format!("<h{level}>{}</h{level}>\n", inline(rest)));
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
                render(&quoted.join("\n"))
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
                out.push_str(&format!("<li>{}</li>\n", inline(text)));
                i += 1;
            }
            out.push_str(&format!("</{tag}>\n"));
            continue;
        }
        // A paragraph runs to the next blank line or block.
        let mut para: Vec<&str> = Vec::new();
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
            para.push(t);
            i += 1;
        }
        out.push_str(&format!("<p>{}</p>\n", inline(&para.join("\n"))));
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

/// The inline spans of a line: code held aside, then images, links,
/// strong and emphasis.
fn inline(text: &str) -> String {
    static CODE: OnceLock<Regex> = OnceLock::new();
    static IMAGE: OnceLock<Regex> = OnceLock::new();
    static LINK: OnceLock<Regex> = OnceLock::new();
    static STRONG: OnceLock<Regex> = OnceLock::new();
    static EM: OnceLock<Regex> = OnceLock::new();
    static EM2: OnceLock<Regex> = OnceLock::new();
    let code = CODE.get_or_init(|| Regex::new(r"`([^`]+)`").unwrap());
    let image = IMAGE.get_or_init(|| Regex::new(r"!\[([^\]]*)\]\(([^)\s]+)\)").unwrap());
    let link = LINK.get_or_init(|| Regex::new(r"\[([^\]]+)\]\(([^)\s]+)\)").unwrap());
    let strong = STRONG.get_or_init(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());
    let em = EM.get_or_init(|| Regex::new(r"\*([^*]+)\*").unwrap());
    let em2 = EM2.get_or_init(|| Regex::new(r"(^|[^A-Za-z0-9])_([^_]+)_($|[^A-Za-z0-9])").unwrap());

    let escaped = escape(text);
    let mut spans: Vec<String> = Vec::new();
    let held = code.replace_all(&escaped, |c: &regex::Captures| {
        spans.push(format!("<code>{}</code>", &c[1]));
        format!("\u{0}{}\u{0}", spans.len() - 1)
    });
    let mut s = image
        .replace_all(&held, "<img src=\"$2\" alt=\"$1\">")
        .to_string();
    s = link.replace_all(&s, "<a href=\"$2\">$1</a>").to_string();
    s = strong.replace_all(&s, "<strong>$1</strong>").to_string();
    s = em.replace_all(&s, "<em>$1</em>").to_string();
    s = em2.replace_all(&s, "$1<em>$2</em>$3").to_string();
    for (i, span) in spans.iter().enumerate() {
        s = s.replace(&format!("\u{0}{i}\u{0}"), span);
    }
    s.replace('\n', "<br>\n")
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
    }

    use super::*;

    #[test]
    fn blocks_and_inline_spans_render() {
        let md = "# Title\n\nA *word* and **more**, `x < y` and [a link](https://x.y) plus ![alt](/i.png).\nSecond line.\n\n- one\n- two\n\n1. first\n2. second\n\n> quoted *text*\n\n---\n\n```js\nlet a = 1 < 2;\n```\n<script>alert(1)</script>";
        let html = render(md);
        assert_eq!(
            html,
            "<h1>Title</h1>\n<p>A <em>word</em> and <strong>more</strong>, <code>x &lt; y</code> and <a href=\"https://x.y\">a link</a> plus <img src=\"/i.png\" alt=\"alt\">.<br>\nSecond line.</p>\n<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n<ol>\n<li>first</li>\n<li>second</li>\n</ol>\n<blockquote>\n<p>quoted <em>text</em></p>\n</blockquote>\n<hr>\n<pre><code class=\"language-js\">let a = 1 &lt; 2;\n</code></pre>\n<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n"
        );
    }
}
