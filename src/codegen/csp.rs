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
pub fn writes_inline_styles(program: &crate::syntax::parser::ast::Program) -> bool {
    use crate::syntax::parser::ast::Declaration;
    fn element(el: &crate::syntax::parser::ast::UIElement) -> bool {
        let own = el.style_block.as_ref().is_some_and(|sb| {
            sb.properties
                .iter()
                .any(|p| crate::codegen::scoped_css::static_declaration(p).is_none())
        });
        // An image's placeholder is a blurred copy or an average colour:
        // one value per picture, painted under it until it arrives.
        let placeholder = matches!(&el.component, crate::syntax::parser::ast::ComponentRef::BuiltIn(n) if n == "Image")
            && el.args.iter().any(
                |a| matches!(a, crate::syntax::parser::ast::Arg::Named(k, _) if k == "placeholder"),
            );
        own || placeholder
            || statements(&el.children)
            || el.slot_fills.iter().any(|f| statements(&f.body))
    }
    fn statements(body: &[crate::syntax::parser::ast::Statement]) -> bool {
        use crate::syntax::parser::ast::StatementKind as K;
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

/// Whether this program frames a page of its own site: a `Frame` whose
/// `src:` is site-relative (`"/preview/"`, `"embed/{id}"`), not another
/// origin's address.
///
/// The pages refuse to be framed by default (`X-Frame-Options: DENY`,
/// `frame-ancestors 'none'`), which would refuse the site's own frame too.
/// Only a project that shows itself inside itself is relaxed, and only to
/// its own origin (`SAMEORIGIN`, `'self'`).
pub fn frames_itself(program: &crate::syntax::parser::ast::Program) -> bool {
    use crate::syntax::parser::ast::{Arg, ComponentRef, Declaration, Expr, StringPart};
    fn site_relative(src: &Expr) -> bool {
        // What the address opens with: a literal, or the literal text an
        // interpolation starts with. A splice up front could be anything,
        // so it does not count.
        let head = match src {
            Expr::StringLiteral(s) => s.as_str(),
            Expr::InterpolatedString(parts) => match parts.first() {
                Some(StringPart::Literal(s)) => s.as_str(),
                _ => return false,
            },
            _ => return false,
        };
        let scheme = head.split_once(':').is_some_and(|(s, _)| {
            !s.is_empty()
                && !s.contains(['/', '?', '#'])
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        });
        !head.starts_with("//") && !scheme
    }
    fn element(el: &crate::syntax::parser::ast::UIElement) -> bool {
        let own = matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Frame")
            && el.args.iter().any(|a| match a {
                Arg::Named(k, v) if k == "src" => site_relative(v),
                Arg::Positional(v) => site_relative(v),
                _ => false,
            });
        own || statements(&el.children) || el.slot_fills.iter().any(|f| statements(&f.body))
    }
    fn statements(body: &[crate::syntax::parser::ast::Statement]) -> bool {
        body.iter().any(|stmt| {
            matches!(&stmt.kind, crate::syntax::parser::ast::StatementKind::UIElement(el) if element(el))
                || stmt.kind.bodies().into_iter().any(statements)
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
///
/// The markup is read as a browser reads it: tag and attribute names in
/// any case, attributes apart by white space or a `/`, values quoted or
/// not, a `>` inside a quoted value no end of the tag, and the text of a
/// `<script>`, a `<style>` or a comment no markup at all.
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

    let elements = elements(html);
    for tag in &elements {
        match tag.name.as_str() {
            // A script: from another origin the policy never named, from an
            // address that is not a place (`data:`, `javascript:`), or
            // written inline.
            "script" => {
                if let Some(src) = tag.attr("src") {
                    match source_of(&src) {
                        Source::Own => {}
                        Source::Scheme(scheme) => say(
                            &mut out,
                            "script-src",
                            format!("a script from {scheme}:"),
                            "A script is a file this site serves, or one at an origin `meta.scripts` names",
                        ),
                        Source::Origin(origin) if !allows("script-src", &origin) => say(
                            &mut out,
                            "script-src",
                            format!("a script from {origin}"),
                            "Name the origin in the policy, or serve the file from this site",
                        ),
                        Source::Origin(_) => {}
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
            "style" => {
                if !tag.body.trim().is_empty() && !allows("style-src", "'unsafe-inline'") {
                    say(
                        &mut out,
                        "style-src",
                        "an inline <style>".to_string(),
                        "A `style { }` block compiles to a rule in `styles.css`",
                    );
                }
            }
            // An external stylesheet from an origin the policy never named.
            "link" => {
                let rel = tag.attr("rel").unwrap_or_default();
                if !rel
                    .split_ascii_whitespace()
                    .any(|r| r.eq_ignore_ascii_case("stylesheet"))
                {
                    continue;
                }
                match source_of(&tag.attr("href").unwrap_or_default()) {
                    Source::Own => {}
                    Source::Scheme(scheme) => say(
                        &mut out,
                        "style-src",
                        format!("a stylesheet from {scheme}:"),
                        "A stylesheet is a file this site serves, or one at an origin `meta.stylesheets` names",
                    ),
                    Source::Origin(origin) if !allows("style-src", &origin) => say(
                        &mut out,
                        "style-src",
                        format!("a stylesheet from {origin}"),
                        "`meta.stylesheets` widens the policy; an origin written into a page by hand does not",
                    ),
                    Source::Origin(_) => {}
                }
            }
            // Every relative address on the page resolves against a
            // `<base>`, so one sends every link and script elsewhere.
            "base" => say(
                &mut out,
                "base-uri",
                "a <base> element".to_string(),
                "A page's addresses are relative to the page; the compiler never writes a <base>",
            ),
            // A refresh to somewhere else is a redirect nothing asked for.
            "meta"
                if tag
                    .attr("http-equiv")
                    .is_some_and(|e| e.trim().eq_ignore_ascii_case("refresh"))
                    && refresh_leaves(&tag.attr("content").unwrap_or_default()) =>
            {
                say(
                    &mut out,
                    "default-src",
                    "a refresh".to_string(),
                    "A refresh may reload the page or name an address on this site; anything else is a redirect",
                )
            }
            _ => {}
        }
    }

    // An inline style or handler, on any element.
    if !allows("style-src", "'unsafe-inline'")
        && elements
            .iter()
            .any(|t| t.attrs.iter().any(|(k, _)| k == "style"))
    {
        say(
            &mut out,
            "style-src",
            "a style= attribute".to_string(),
            "A literal in a `style { }` block compiles to a class; only a value that reads state is set on the element",
        );
    }
    // An inline handler, which is script in an attribute.
    if let Some(handler) = elements.iter().find_map(|t| {
        t.attrs
            .iter()
            .find(|(k, _)| is_handler(k))
            .map(|(k, _)| k.clone())
    }) {
        say(
            &mut out,
            "script-src",
            format!("an `{handler}` attribute"),
            "Write the handler instead: `on click { … }`",
        );
    }

    out
}

/// Whether `name` is an event handler attribute: `on` and a word.
fn is_handler(name: &str) -> bool {
    name.len() > 2 && name.starts_with("on") && name[2..].chars().all(|c| c.is_ascii_alphabetic())
}

/// Where an address loads from, as the policy sees it.
enum Source {
    /// This site: a relative address, or none.
    Own,
    /// An origin elsewhere, `https://cdn.example.com`.
    Origin(String),
    /// A scheme that names no place to fetch from: `data`, `javascript`.
    Scheme(String),
}

fn source_of(url: &str) -> Source {
    match crate::codegen::url::scheme_of(url) {
        Some(scheme) if scheme != "http" && scheme != "https" => Source::Scheme(scheme),
        _ => match external_origin(url.trim()) {
            Some(origin) => Source::Origin(origin),
            None => Source::Own,
        },
    }
}

/// Whether a refresh's `content` sends the reader off this site: an
/// address with a scheme, or one that names another host (`//host`).
fn refresh_leaves(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    let Some(at) = lower.find("url") else {
        return false;
    };
    let rest = content[at + 3..].trim_start();
    let Some(rest) = rest.strip_prefix('=') else {
        return false;
    };
    let url = rest.trim().trim_matches(|c| c == '\'' || c == '"');
    crate::codegen::url::scheme_of(url).is_some()
        || url.starts_with("//")
        || url.starts_with("\\\\")
        || url.starts_with("/\\")
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
/// A page's `head { }` tags that load a script or a stylesheet from an
/// origin `policy` does not name. They are added on the live page, so the
/// read-back of the written HTML never sees them; the browser refuses them
/// all the same.
pub fn head_violations(
    program: &crate::syntax::parser::ast::Program,
    policy: &str,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<crate::diagnostics::Diagnostic> {
    use crate::syntax::parser::ast::{Declaration, Expr};
    let mut out = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let Declaration::Page(page) = decl else {
            continue;
        };
        for tag in &page.head {
            let attr = |name: &str| {
                tag.attrs.iter().find_map(|(k, v)| match v {
                    Expr::StringLiteral(s) if k == name => Some(s.clone()),
                    _ => None,
                })
            };
            let (directive, url, config_key) = match tag.tag.as_str() {
                "script" => ("script-src", attr("src"), "meta.scripts"),
                "link"
                    if attr("rel")
                        .is_some_and(|r| r.split_whitespace().any(|w| w == "stylesheet")) =>
                {
                    ("style-src", attr("href"), "meta.stylesheets")
                }
                _ => continue,
            };
            let Some(origin) = url.as_deref().and_then(external_origin) else {
                continue;
            };
            if sources_of(policy, directive).contains(&origin) {
                continue;
            }
            out.push(
                crate::diagnostics::Diagnostic::coded(
                    "E902",
                    format!(
                        "`head {{ {}(…) }}` loads from {origin}, which the `{directive}` of the policy this build ships forbids",
                        tag.tag
                    ),
                    file_of(index),
                    tag.span.line as usize,
                    tag.span.col as usize,
                )
                .with_hint(format!(
                    "Declare it in `{config_key}` — the policy names exactly the origins the config does — or serve the file from this site"
                )),
            );
        }
    }
    out
}

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

/// One element of the page: its name and attributes, lower-cased, and for
/// a `<script>` or a `<style>` the text up to its closing tag.
struct Tag {
    name: String,
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

/// Every element `html` opens, read as a browser's tokenizer reads the
/// start of one: `<` and a letter, a name, then attributes up to the `>`
/// that is not inside a quoted value. A comment is skipped whole, and the
/// text of a `<script>` or a `<style>` is that element's body, never
/// markup of its own. Text holds no markup: the compiler writes `<` there
/// as `&lt;`.
fn elements(html: &str) -> Vec<Tag> {
    let chars: Vec<char> = html.chars().collect();
    let lower: Vec<char> = chars.iter().map(|c| c.to_ascii_lowercase()).collect();
    let starts = |at: usize, text: &str| {
        text.chars()
            .enumerate()
            .all(|(n, c)| lower.get(at + n) == Some(&c))
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '<' {
            i += 1;
            continue;
        }
        if starts(i, "<!--") {
            i = (i + 4..chars.len().saturating_sub(2))
                .find(|&j| starts(j, "-->"))
                .map_or(chars.len(), |j| j + 3);
            continue;
        }
        if !chars.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic()) {
            i += 1;
            continue;
        }
        let name_at = i + 1;
        let mut j = name_at;
        while j < chars.len() && !(chars[j].is_whitespace() || chars[j] == '/' || chars[j] == '>') {
            j += 1;
        }
        let name: String = lower[name_at..j].iter().collect();
        // The end of the tag: the first `>` outside a quoted value.
        let mut quote: Option<char> = None;
        let mut k = j;
        while k < chars.len() {
            let c = chars[k];
            match quote {
                Some(q) if c == q => quote = None,
                Some(_) => {}
                None if c == '>' => break,
                // A quote opens a value only straight after its `=`.
                None if (c == '"' || c == '\'')
                    && chars[j..k].iter().rev().find(|c| !c.is_whitespace()) == Some(&'=') =>
                {
                    quote = Some(c)
                }
                None => {}
            }
            k += 1;
        }
        let head: String = chars[j..k.min(chars.len())].iter().collect();
        let mut tag = Tag {
            attrs: attributes(&head),
            name,
            body: String::new(),
        };
        i = k + 1;
        // A script's or a style's text runs to its closing tag.
        if matches!(tag.name.as_str(), "script" | "style") {
            let close = format!("</{}", tag.name);
            let end = (i..chars.len())
                .find(|&n| {
                    starts(n, &close)
                        && chars
                            .get(n + close.chars().count())
                            .is_none_or(|c| c.is_whitespace() || *c == '>' || *c == '/')
                })
                .unwrap_or(chars.len());
            tag.body = chars[i.min(end)..end].iter().collect();
            i = end;
        }
        out.push(tag);
    }
    out
}

/// `name="value"` pairs from the inside of a tag, as a browser reads them:
/// apart by white space or `/`, a name lower-cased, white space allowed
/// around the `=`, a value in either quote or none.
fn attributes(head: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let chars: Vec<char> = head.chars().collect();
    let mut i = 0;
    let skip = |i: &mut usize, pred: &dyn Fn(char) -> bool| {
        while *i < chars.len() && pred(chars[*i]) {
            *i += 1;
        }
    };
    while i < chars.len() {
        skip(&mut i, &|c| c.is_whitespace() || c == '/');
        let name_at = i;
        skip(&mut i, &|c| {
            !(c.is_whitespace() || c == '/' || c == '>' || c == '=')
        });
        if i == name_at {
            // A lone `=` with no name: step past it.
            i += 1;
            continue;
        }
        let name: String = chars[name_at..i]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase();
        let mut after = i;
        skip(&mut after, &|c| c.is_whitespace());
        if chars.get(after) != Some(&'=') {
            out.push((name, String::new()));
            continue;
        }
        i = after + 1;
        skip(&mut i, &|c| c.is_whitespace());
        let value: String = match chars.get(i).copied() {
            Some(q @ ('"' | '\'')) => {
                i += 1;
                let from = i;
                skip(&mut i, &|c| c != q);
                let v: String = chars[from..i].iter().collect();
                i += 1;
                v
            }
            _ => {
                let from = i;
                skip(&mut i, &|c| !c.is_whitespace() && c != '>');
                chars[from..i].iter().collect()
            }
        };
        out.push((name, value));
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
