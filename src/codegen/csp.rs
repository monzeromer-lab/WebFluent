//! Holding the output to the policy it ships with.
//!
//! A `Content-Security-Policy` is a promise about what a page contains. It
//! is easy to write one and then emit something it forbids — an inline
//! `<script>`, a `style="…"` attribute, an `on*` handler, a stylesheet from
//! an origin the policy never named — and the first anyone hears of it is a
//! blank page in production, because the browser enforces the promise and
//! the build did not.
//!
//! This reads the HTML the build just wrote and says whether the policy it
//! carries is one that page satisfies. It is a check on the output, not on
//! the source, so it also catches anything a hand-written file in `public/`
//! brought with it.

/// Whether this program's static paint will carry a `style=` attribute.
///
/// A style value that reads state has nowhere else to go: a literal
/// compiles to a shared class, but a value that changes is the element's
/// own. The policy has to say so before the pages are written, so it is
/// answered from the program rather than from the output — and the
/// verifier reads the output afterwards and says whether the answer held.
pub fn writes_inline_styles(program: &crate::parser::ast::Program) -> bool {
    use crate::parser::ast::Declaration;
    fn element(el: &crate::parser::ast::UIElement) -> bool {
        let own = el.style_block.as_ref().is_some_and(|sb| {
            sb.properties
                .iter()
                .any(|p| crate::codegen::scoped_css::static_declaration(p).is_none())
        });
        // An image's placeholder is a blurred copy or an average colour:
        // one value per picture, painted under it until it arrives.
        let placeholder = matches!(&el.component, crate::parser::ast::ComponentRef::BuiltIn(n) if n == "Image")
            && el
                .args
                .iter()
                .any(|a| matches!(a, crate::parser::ast::Arg::Named(k, _) if k == "placeholder"));
        own || placeholder
            || statements(&el.children)
            || el.slot_fills.iter().any(|f| statements(&f.body))
    }
    fn statements(body: &[crate::parser::ast::Statement]) -> bool {
        use crate::parser::ast::StatementKind as K;
        body.iter().any(|stmt| match &stmt.kind {
            K::UIElement(el) => element(el),
            K::If(i) => {
                statements(&i.then_body)
                    || i.else_if_branches.iter().any(|(_, b)| statements(b))
                    || i.else_body.as_deref().is_some_and(statements)
            }
            K::For(f) => statements(&f.body),
            K::Show(s) => statements(&s.body),
            K::Match(m) => m.arms.iter().any(|a| statements(&a.body)),
            _ => false,
        })
    }
    program.declarations.iter().any(|d| match d {
        Declaration::Page(page) => statements(&page.body),
        Declaration::Component(c) => statements(&c.body),
        Declaration::App(a) => statements(&a.body),
        _ => false,
    })
}

/// One thing in a page the policy beside it forbids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The file it was found in, as the reader would ask for it.
    pub page: String,
    /// The directive it falls under (`script-src`, `style-src`, …).
    pub directive: String,
    /// What was found, in a few words.
    pub what: String,
    /// What to do about it.
    pub hint: String,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} — {} forbids it. {}",
            self.page, self.what, self.directive, self.hint
        )
    }
}

/// What in `html` the `policy` beside it forbids.
pub fn check(page: &str, html: &str, policy: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    let allows = |directive: &str, source: &str| -> bool {
        sources_of(policy, directive).iter().any(|s| s == source)
    };
    let say = |out: &mut Vec<Violation>, directive: &str, what: String, hint: &str| {
        out.push(Violation {
            page: page.to_string(),
            directive: directive.to_string(),
            what,
            hint: hint.to_string(),
        });
    };

    // An inline script: a `<script>` with a body and no `src`.
    for tag in tags(html, "script") {
        if tag.attrs.iter().any(|(k, _)| k == "src") {
            let src = tag.attr("src").unwrap_or_default();
            if let Some(origin) = external_origin(&src)
                && !allows("script-src", &origin)
            {
                say(
                    &mut out,
                    "script-src",
                    format!("a script from {origin}"),
                    "Name the origin in the policy, or serve the file from this site",
                );
            }
        } else if !tag.body.trim().is_empty()
            && is_script(&tag.attr("type").unwrap_or_default())
            && !allows("script-src", "'unsafe-inline'")
        {
            say(
                &mut out,
                "script-src",
                "an inline <script>".to_string(),
                "The compiler writes external files; a hand-written one belongs in `public/`",
            );
        }
    }

    // An inline style, as a tag or as an attribute.
    for tag in tags(html, "style") {
        if !tag.body.trim().is_empty() && !allows("style-src", "'unsafe-inline'") {
            say(
                &mut out,
                "style-src",
                "an inline <style>".to_string(),
                "A `style { }` block compiles to a rule in `styles.css`",
            );
        }
    }
    if html
        .match_indices(" style=\"")
        .any(|(at, _)| inside_tag(html, at))
        && !allows("style-src", "'unsafe-inline'")
    {
        say(
            &mut out,
            "style-src",
            "a style= attribute".to_string(),
            "A literal in a `style { }` block compiles to a class; only a value that reads state is set on the element",
        );
    }

    // An external stylesheet from an origin the policy never named.
    for tag in tags(html, "link") {
        let rel = tag.attr("rel").unwrap_or_default();
        if !rel.split_whitespace().any(|r| r == "stylesheet") {
            continue;
        }
        if let Some(origin) = external_origin(&tag.attr("href").unwrap_or_default())
            && !allows("style-src", &origin)
        {
            say(
                &mut out,
                "style-src",
                format!("a stylesheet from {origin}"),
                "`meta.stylesheets` widens the policy; an origin written into a page by hand does not",
            );
        }
    }

    // An inline handler, which is script in an attribute.
    if let Some(at) = inline_handler(html) {
        say(
            &mut out,
            "script-src",
            format!("an `{at}` attribute"),
            "Write the handler instead: `on click { … }`",
        );
    }

    out
}

/// Whether a `<script>` of this `type` holds script.
///
/// A block whose type is something else — `application/ld+json`, an
/// import map, a template — is **data**: the browser never executes it,
/// and `script-src` does not govern it. Reporting one would be reporting
/// a page for the structured data the compiler wrote into it.
fn is_script(ty: &str) -> bool {
    let ty = ty.trim().to_ascii_lowercase();
    let ty = ty.split(';').next().unwrap_or("").trim();
    matches!(
        ty,
        "" | "module"
            | "text/javascript"
            | "application/javascript"
            | "text/ecmascript"
            | "application/ecmascript"
    )
}

/// The sources a directive names, falling back to `default-src`.
fn sources_of(policy: &str, directive: &str) -> Vec<String> {
    let find = |name: &str| {
        policy.split(';').map(str::trim).find_map(|part| {
            let rest = part.strip_prefix(name)?;
            rest.starts_with(' ')
                .then(|| rest.split_whitespace().map(str::to_string).collect())
        })
    };
    find(directive)
        .or_else(|| find("default-src"))
        .unwrap_or_default()
}

/// The origin of `url` when it is somewhere else, and nothing when it is
/// this site's — a relative path, or one that only names a path.
fn external_origin(url: &str) -> Option<String> {
    let rest = url.strip_prefix("//").map(|r| ("https:", r)).or_else(|| {
        let at = url.find("://")?;
        Some((&url[..at + 1], &url[at + 3..]))
    })?;
    let host = rest.1.split('/').next()?;
    (!host.is_empty()).then(|| format!("{}//{}", rest.0, host))
}

/// The name of the first `on*` attribute in `html`, if it has one.
/// Whether `at` is inside a tag — after a `<` with no `>` since — rather
/// than in text, where the characters of markup arrive escaped.
fn inside_tag(html: &str, at: usize) -> bool {
    let before = &html[..at];
    match (before.rfind('<'), before.rfind('>')) {
        (Some(open), Some(close)) => open > close,
        (Some(_), None) => true,
        _ => false,
    }
}

fn inline_handler(html: &str) -> Option<String> {
    let bytes = html.as_bytes();
    let mut from = 0;
    while let Some(at) = html[from..].find(" on") {
        let start = from + at + 1;
        let end = html[start..]
            .find(|c: char| !c.is_ascii_alphabetic())
            .map(|n| start + n)?;
        // `on…=` and nothing else: ` only` and ` once` are words in text.
        // And inside a tag: a page that shows code (`<button onClick={…}>`
        // in a sample, escaped as text) holds no attribute at all.
        if bytes.get(end) == Some(&b'=') && end > start + 2 && inside_tag(html, start) {
            return Some(html[start..end].to_string());
        }
        from = start;
    }
    None
}

/// One element of `html` with the given tag name: its attributes, and the
/// text between it and its closing tag.
struct Tag {
    attrs: Vec<(String, String)>,
    body: String,
}

impl Tag {
    fn attr(&self, name: &str) -> Option<String> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }
}

fn tags(html: &str, name: &str) -> Vec<Tag> {
    let mut out = Vec::new();
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut from = 0;
    while let Some(at) = html[from..].find(&open) {
        let start = from + at;
        let after = start + open.len();
        // `<script>` and not `<scriptish>`.
        if !matches!(html[after..].chars().next(), Some(c) if c == '>' || c.is_whitespace() || c == '/')
        {
            from = after;
            continue;
        }
        let Some(head_end) = html[start..].find('>').map(|n| start + n) else {
            break;
        };
        let head = &html[after..head_end];
        let body = match html[head_end..].find(&close) {
            Some(n) => html[head_end + 1..head_end + n].to_string(),
            None => String::new(),
        };
        out.push(Tag {
            attrs: attributes(head),
            body,
        });
        from = head_end + 1;
    }
    out
}

/// `name="value"` pairs from the inside of a tag.
fn attributes(head: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let chars: Vec<char> = head.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && !chars[i].is_ascii_alphabetic() {
            i += 1;
        }
        let name_at = i;
        while i < chars.len() && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '-' | '_'))
        {
            i += 1;
        }
        if i == name_at {
            break;
        }
        let name: String = chars[name_at..i].iter().collect();
        if chars.get(i) != Some(&'=') {
            out.push((name.to_ascii_lowercase(), String::new()));
            continue;
        }
        i += 1;
        let quote = chars.get(i).copied();
        let value: String = match quote {
            Some(q @ ('"' | '\'')) => {
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
        out.push((name.to_ascii_lowercase(), value));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self' https://fonts.googleapis.com; object-src 'none'";

    #[test]
    fn markup_shown_as_text_is_not_markup() {
        // A page that shows a JSX sample: its `onClick=` and `style="` are
        // text, escaped, and nothing the policy governs.
        let html =
            "<pre><code>&lt;button onClick={save} style=\"x\"&gt;Save&lt;/button&gt;</code></pre>";
        assert_eq!(check("p", html, POLICY), Vec::new());
        // In a tag it is still an attribute.
        assert!(!check("p", "<button onclick=\"x()\">b</button>", POLICY).is_empty());
    }

    #[test]
    fn what_the_engine_writes_satisfies_the_policy_it_writes() {
        let html = "<html><head>\
            <link rel=\"stylesheet\" href=\"/styles.css\">\
            <link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Inter\">\
            <script src=\"/app.js\" defer></script>\
            </head><body><p class=\"wf-text\">Only text</p></body></html>";
        assert_eq!(check("index.html", html, POLICY), Vec::new());
    }

    #[test]
    fn everything_the_policy_forbids_is_named_before_the_browser_finds_it() {
        let cases: &[(&str, &str)] = &[
            ("<script>alert(1)</script>", "an inline <script>"),
            (
                "<script type=\"module\">import \"x\"</script>",
                "an inline <script>",
            ),
            ("<style>p{color:red}</style>", "an inline <style>"),
            ("<p style=\"color:red\">x</p>", "a style= attribute"),
            (
                "<button onclick=\"go()\">x</button>",
                "an `onclick` attribute",
            ),
            (
                "<script src=\"https://cdn.example.com/a.js\"></script>",
                "a script from https://cdn.example.com",
            ),
            (
                "<link rel=\"stylesheet\" href=\"https://cdn.example.com/a.css\">",
                "a stylesheet from https://cdn.example.com",
            ),
        ];
        for (html, expected) in cases {
            let found = check("index.html", html, POLICY);
            assert!(
                found.iter().any(|v| v.what == *expected),
                "{expected} in {found:?}"
            );
        }
    }

    #[test]
    fn a_data_block_is_not_a_script() {
        // JSON-LD is what the compiler writes for a search result, and a
        // browser never executes it.
        let html = "<script type=\"application/ld+json\">{\"@type\":\"WebSite\"}</script>";
        assert_eq!(check("index.html", html, POLICY), Vec::new());
        let map = "<script type=\"importmap\">{\"imports\":{}}</script>";
        assert_eq!(check("index.html", map, POLICY), Vec::new());
    }

    #[test]
    fn a_policy_that_allows_it_says_nothing() {
        let loose = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'";
        assert_eq!(
            check("i.html", "<script>alert(1)</script>", loose),
            Vec::new()
        );
        assert_eq!(
            check("i.html", "<p style=\"color:red\">x</p>", loose),
            Vec::new()
        );
        // A directive the policy leaves out falls back to `default-src`.
        let only_default = "default-src 'self' https://cdn.example.com";
        assert_eq!(
            check(
                "i.html",
                "<script src=\"https://cdn.example.com/a.js\"></script>",
                only_default
            ),
            Vec::new()
        );
    }
}
