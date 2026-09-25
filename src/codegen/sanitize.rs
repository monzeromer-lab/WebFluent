//! The build-time twin of the runtime's `sanitize`.
//!
//! `Unsafe.Html(sanitize(body))` has to paint at build time as well as in
//! the browser, or a static site's article body is an empty box until the
//! script runs — which is the whole reason the page was pre-rendered. So
//! the allow-list exists twice, and a pair of tests holds the two to the
//! same answers.
//!
//! It is a filter over the markup, not a parser of a document: every tag
//! is read, an element the list does not name is dropped and its contents
//! kept, an attribute the list does not name is dropped, and a `href` or
//! `src` goes through the same scheme check every other URL does. Anything
//! that is not a well-formed tag — a comment, a processing instruction, a
//! stray `<` — is dropped.

/// The elements markup may keep.
const SAFE_TAGS: &[&str] = &[
    "a",
    "abbr",
    "address",
    "b",
    "blockquote",
    "br",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "dd",
    "del",
    "dfn",
    "div",
    "dl",
    "dt",
    "em",
    "figcaption",
    "figure",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "img",
    "ins",
    "kbd",
    "li",
    "mark",
    "ol",
    "p",
    "pre",
    "q",
    "s",
    "samp",
    "section",
    "small",
    "span",
    "strong",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "time",
    "tr",
    "u",
    "ul",
    "var",
    "wbr",
];

/// The attributes every element may keep.
const GLOBAL_ATTRS: &[&str] = &["class", "id", "title", "lang", "dir", "role"];

/// The attributes particular elements may keep, beyond the global ones.
const TAG_ATTRS: &[(&str, &[&str])] = &[
    ("a", &["href", "target", "rel"]),
    (
        "img",
        &["src", "alt", "width", "height", "loading", "decoding"],
    ),
    ("td", &["colspan", "rowspan", "headers"]),
    ("th", &["colspan", "rowspan", "scope", "headers"]),
    ("ol", &["start", "reversed", "type"]),
    ("li", &["value"]),
    ("time", &["datetime"]),
    ("blockquote", &["cite"]),
    ("q", &["cite"]),
    ("del", &["cite", "datetime"]),
    ("ins", &["cite", "datetime"]),
    ("col", &["span"]),
    ("colgroup", &["span"]),
];

/// The elements that never have a closing tag.
const VOID: &[&str] = &["br", "hr", "img", "wbr", "col"];

/// `html` with everything the allow-list does not name taken out.
pub fn sanitize(html: &str) -> String {
    let chars: Vec<char> = html.chars().collect();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '<' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        // A `<` opens a tag only when a name, a `/` or a `!` follows it
        // immediately — the browser's rule. `1 < 2` is arithmetic.
        let opens = matches!(chars.get(i + 1), Some(c) if c.is_ascii_alphabetic() || *c == '/' || *c == '!' || *c == '?');
        let end = match opens.then(|| tag_end(&chars, i)).flatten() {
            Some(end) => end,
            None => {
                out.push_str("&lt;");
                i += 1;
                continue;
            }
        };
        let inside: String = chars[i + 1..end].iter().collect();
        i = end + 1;
        let closing = inside.starts_with('/');
        let body = inside.trim_start_matches('/');
        // A comment, a doctype, a processing instruction: not an element.
        if body.starts_with('!') || body.starts_with('?') {
            continue;
        }
        let name: String = body
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if name.is_empty() || !SAFE_TAGS.contains(&name.as_str()) {
            // The element goes; what it said stays, so dropping a `<font>`
            // does not drop the sentence inside it. A `<script>`'s body is
            // not a sentence, so it goes with the tag.
            if matches!(name.as_str(), "script" | "style" | "template") && !closing {
                i = skip_to_close(&chars, i, &name);
            }
            continue;
        }
        if closing {
            out.push_str(&format!("</{name}>"));
            continue;
        }
        let allowed: Vec<&str> = GLOBAL_ATTRS
            .iter()
            .copied()
            .chain(
                TAG_ATTRS
                    .iter()
                    .find(|(t, _)| *t == name)
                    .map(|(_, a)| a.iter().copied())
                    .into_iter()
                    .flatten(),
            )
            .collect();
        let rest: String = body.chars().skip(name.chars().count()).collect();
        let mut kept: Vec<(String, String)> = Vec::new();
        for (key, value) in attributes(&rest) {
            if !allowed.contains(&key.as_str()) {
                continue;
            }
            let value = if key == "href" || key == "src" {
                let guarded = crate::codegen::url::guard(&value).to_string();
                if guarded.is_empty() {
                    continue;
                }
                guarded
            } else {
                value
            };
            kept.push((key, value));
        }
        // A link that opens elsewhere does not hand it a handle on this page.
        if name == "a"
            && kept.iter().any(|(k, _)| k == "target")
            && !kept.iter().any(|(k, _)| k == "rel")
        {
            kept.push(("rel".to_string(), "noopener noreferrer".to_string()));
        }
        out.push('<');
        out.push_str(&name);
        for (key, value) in kept {
            out.push_str(&format!(" {key}=\"{}\"", escape_attr(&value)));
        }
        if VOID.contains(&name.as_str()) {
            out.push_str(" />");
        } else {
            out.push('>');
        }
    }
    out
}

/// Where the tag opened at `at` closes, honouring a quoted attribute value
/// that holds a `>`.
fn tag_end(chars: &[char], at: usize) -> Option<usize> {
    let mut i = at + 1;
    let mut quote: Option<char> = None;
    while i < chars.len() {
        let c = chars[i];
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '>' => return Some(i),
            None if c == '<' => return None,
            None => {}
        }
        i += 1;
    }
    None
}

/// Past the closing tag of `name`, or to the end where there is none.
fn skip_to_close(chars: &[char], from: usize, name: &str) -> usize {
    let text: String = chars[from..].iter().collect();
    let close = format!("</{name}");
    match text.to_ascii_lowercase().find(&close) {
        Some(at) => {
            let start = from + text[..at].chars().count();
            tag_end(chars, start).map(|e| e + 1).unwrap_or(chars.len())
        }
        None => chars.len(),
    }
}

/// `name="value"` pairs from the inside of a tag, after its name.
fn attributes(rest: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = rest.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && !chars[i].is_ascii_alphabetic() {
            i += 1;
        }
        let from = i;
        while i < chars.len() && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '-' | '_'))
        {
            i += 1;
        }
        if i == from {
            break;
        }
        let key: String = chars[from..i]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase();
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if chars.get(i) != Some(&'=') {
            out.push((key, String::new()));
            continue;
        }
        i += 1;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        let value = match chars.get(i) {
            Some(&q @ ('"' | '\'')) => {
                i += 1;
                let from = i;
                while i < chars.len() && chars[i] != q {
                    i += 1;
                }
                let v: String = chars[from..i].iter().collect();
                i += 1;
                v
            }
            _ => {
                let from = i;
                while i < chars.len() && !chars[i].is_whitespace() {
                    i += 1;
                }
                chars[from..i].iter().collect()
            }
        };
        out.push((key, unescape(&value)));
    }
    out
}

/// The entities a value may arrive with, so `&amp;#106;avascript:` is not a
/// way past the scheme check.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let chars: Vec<char> = value.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '&' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let rest: String = chars[i..].iter().take(12).collect();
        let Some(end) = rest.find(';') else {
            out.push('&');
            i += 1;
            continue;
        };
        let entity = &rest[1..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "colon" | "#58" => Some(':'),
            e if e.starts_with("#x") || e.starts_with("#X") => u32::from_str_radix(&e[2..], 16)
                .ok()
                .and_then(char::from_u32),
            e if e.starts_with('#') => e[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match decoded {
            Some(c) => {
                out.push(c);
                i += end + 1;
            }
            None => {
                out.push('&');
                i += 1;
            }
        }
    }
    out
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_page_may_keep_it_keeps() {
        let html = "<p class=\"lead\">A <strong>word</strong> and a \
                    <a href=\"/about\" title=\"About\">link</a>.</p>\
                    <ul><li>one</li><li>two</li></ul>\
                    <img src=\"/a.png\" alt=\"A\" width=\"10\" height=\"10\">";
        let out = sanitize(html);
        assert!(out.contains("<strong>word</strong>"), "{out}");
        assert!(
            out.contains("<a href=\"/about\" title=\"About\">link</a>"),
            "{out}"
        );
        assert!(out.contains("<li>one</li>"), "{out}");
        assert!(out.contains("<img src=\"/a.png\" alt=\"A\""), "{out}");
    }

    #[test]
    fn what_it_may_not_keep_is_gone_and_the_words_stay() {
        let cases: &[(&str, &str, &str)] = &[
            (
                "<script>alert(1)</script>",
                "alert",
                "a script's body goes with it",
            ),
            (
                "<p onclick=\"go()\">x</p>",
                "onclick",
                "a handler is not an attribute",
            ),
            (
                "<a href=\"javascript:alert(1)\">x</a>",
                "javascript",
                "nor is a scheme the browser runs",
            ),
            (
                "<a href=\"java&#115;cript:alert(1)\">x</a>",
                "cript:alert",
                "written as an entity either",
            ),
            ("<iframe src=\"//evil\"></iframe>", "iframe", "no frames"),
            ("<style>p{x:y}</style>", "p{x:y}", "no stylesheet"),
            ("<img src=x onerror=alert(1)>", "onerror", "not on an image"),
            (
                "<svg><animate onbegin=alert(1)></svg>",
                "onbegin",
                "nor in an svg",
            ),
            ("<form action=\"/x\"><input></form>", "input", "no form"),
        ];
        for (html, forbidden, why) in cases {
            let out = sanitize(html);
            assert!(!out.contains(forbidden), "{why}: {out}");
        }
        // An element that goes keeps what it said.
        assert!(sanitize("<font color=red>hello</font>").contains("hello"));
        assert!(sanitize("<div><marquee>text</marquee></div>").contains("text"));
    }

    #[test]
    fn a_link_that_opens_elsewhere_carries_noopener() {
        let out = sanitize("<a href=\"https://example.com\" target=\"_blank\">x</a>");
        assert!(out.contains("rel=\"noopener noreferrer\""), "{out}");
    }

    /// The cases the runtime's `sanitize` is held to as well: two
    /// implementations of one allow-list, byte for byte.
    #[test]
    fn the_shared_cases_give_the_shared_answers() {
        let raw = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/sanitize-cases.json"
        ))
        .expect("the case table");
        let cases: Vec<(String, String)> = serde_json::from_str(&raw).expect("the case table");
        for (html, expected) in cases {
            assert_eq!(sanitize(&html), expected, "sanitize({html:?})");
        }
    }

    #[test]
    fn text_that_only_looks_like_markup_is_text() {
        assert_eq!(sanitize("1 < 2 and 3 > 2"), "1 &lt; 2 and 3 > 2");
        assert_eq!(sanitize("<!-- a comment -->"), "");
    }
}
