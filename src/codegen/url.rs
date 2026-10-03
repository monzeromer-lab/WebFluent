//! What may be put where a browser will follow it.
//!
//! `href`, `src`, `to`, `poster` and the rest are the places a string
//! becomes something the browser *runs* or *fetches*. A `javascript:` URL
//! in one of them is script execution written as data — the oldest hole
//! there is, and the one a template language is expected to close.
//!
//! The rule is an allow-list of schemes, not a block-list of the ones
//! anybody has thought of: `http`, `https`, `mailto`, `tel`, `sms`, and
//! anything relative. A `data:` URL is refused everywhere but an image,
//! and even there it is written `Unsafe.dataUrl(…)` so it greps.
//!
//! The runtime has the twin of this (`WF.safeUrl`), and a pair of tests
//! holds the two to the same answers — a literal is checked where it is
//! written, and a value that only exists at run time where it is used.

/// The attributes whose value the browser follows or fetches.
pub const URL_ATTRS: &[&str] = &[
    "src",
    "href",
    "to",
    "poster",
    "action",
    "formaction",
    "cite",
    "ping",
    "srcdoc",
    "data",
    "background",
    "captions",
    "transcript",
];

/// The schemes a URL may name.
const ALLOWED: &[&str] = &["http", "https", "mailto", "tel", "sms", "ftp"];

/// Whether `url` is one a browser may be pointed at.
///
/// A relative URL — no scheme at all — is always allowed; so is one that
/// begins with a splice, since what it holds is checked at run time.
pub fn is_safe(url: &str) -> bool {
    let Some(scheme) = scheme_of(url) else {
        return true;
    };
    ALLOWED.contains(&scheme.as_str())
}

/// The scheme `url` names, lowercased, or `None` where it names none.
///
/// The parse is the browser's, not a regular expression's: everything up
/// to the first `:`, with the control characters and whitespace a browser
/// ignores removed first — `java\0script:`, `java\tscript:` and
/// ` JavaScript:` are all `javascript`.
pub fn scheme_of(url: &str) -> Option<String> {
    let cleaned: String = url
        .chars()
        .filter(|c| !c.is_control() && !c.is_whitespace())
        .collect();
    let at = cleaned.find(':')?;
    let scheme = &cleaned[..at];
    // A path may hold a colon (`/a:b`, `?q=a:b`, `#a:b`), and a scheme may
    // not: it is a letter followed by letters, digits, `+`, `-` and `.`.
    if scheme.is_empty() || !scheme.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    if !scheme
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return None;
    }
    Some(scheme.to_ascii_lowercase())
}

/// `url` where it is one a browser may follow, and nothing where it is not.
///
/// Nothing, rather than a refusal: an `href` that is not there is a piece
/// of text, which is what a link to a hostile URL should be.
pub fn guard(url: &str) -> &str {
    if is_safe(url) { url } else { "" }
}

/// One piece of an address: text the source wrote, or a value spliced in.
pub enum AddressPart {
    Text(String),
    Value(String),
}

/// What `encodeURIComponent` does: every byte but the unreserved ones
/// (`A–Z a–z 0–9 - _ . ! ~ * ' ( )`) as `%XX`.
pub fn encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// An address put together from its pieces, each spliced value encoded
/// where it stands for a value — in a path segment, or a query's value —
/// and written as it is where it opens the address, is a whole query
/// (`?{qs}`), or follows the `#`. The live page does the same
/// (`JsCodegen::encoded_address`), so the static paint links where the page
/// does.
pub fn join_address(parts: &[AddressPart]) -> String {
    let mut out = String::new();
    for part in parts {
        match part {
            AddressPart::Text(t) => out.push_str(t),
            AddressPart::Value(v) => {
                let raw =
                    out.is_empty() || out.contains('#') || out.ends_with('?') || out.ends_with('&');
                if raw {
                    out.push_str(v);
                } else {
                    out.push_str(&encode_component(v));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spliced_value_is_encoded_where_it_stands_for_a_value() {
        let address = |parts: Vec<AddressPart>| join_address(&parts);
        use AddressPart::{Text, Value};
        assert_eq!(
            address(vec![Text("/team/".into()), Value("R&D/Ops".into())]),
            "/team/R%26D%2FOps"
        );
        assert_eq!(
            address(vec![
                Value("/api".into()),
                Text("/rows?q=".into()),
                Value("a b".into())
            ]),
            "/api/rows?q=a%20b"
        );
        assert_eq!(
            address(vec![Text("/s?".into()), Value("x=1&y=2".into())]),
            "/s?x=1&y=2"
        );
        assert_eq!(
            address(vec![Text("/docs#".into()), Value("a b".into())]),
            "/docs#a b"
        );
        assert_eq!(encode_component("ü"), "%C3%BC");
    }

    #[test]
    fn a_scheme_a_browser_would_run_is_not_one_a_page_may_name() {
        for hostile in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "  javascript:alert(1)",
            "java\tscript:alert(1)",
            "java\nscript:alert(1)",
            "java\u{0}script:alert(1)",
            "vbscript:msgbox(1)",
            "data:text/html,<script>alert(1)</script>",
            "blob:https://example.com/x",
            "file:///etc/passwd",
        ] {
            assert!(!is_safe(hostile), "{hostile} is not a URL a page may name");
            assert_eq!(guard(hostile), "");
        }
    }

    #[test]
    fn everything_a_page_actually_links_to_is_allowed() {
        for fine in [
            "/about",
            "about",
            "../up",
            "#section",
            "?q=1",
            "https://example.com/a:b",
            "http://example.com",
            "HTTPS://EXAMPLE.COM",
            "mailto:ada@example.com",
            "tel:+441234567890",
            "sms:+441234567890",
            "//example.com/protocol-relative",
            "/search?q=time:now",
            "/p/{slug}",
        ] {
            assert!(is_safe(fine), "{fine} is an ordinary URL");
            assert_eq!(guard(fine), fine);
        }
    }
}
