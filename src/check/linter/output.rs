//! What a page writes that its data could turn against it, and numbers a
//! built-in cannot be given.
//!
//! - `V03`: a `Frame(srcdoc:)` drawing HTML from a value. It is sandboxed
//!   (`sandbox=""`) unless it says what it may do, and named either way,
//!   as `Unsafe.Html` is: it is a place where a string becomes a document.
//! - `E107`: a `head { }` tag a value could point somewhere a browser
//!   runs — a `script(src:)` that is not written out, a refresh whose
//!   address comes from a value, an `http-equiv` a value chooses, a
//!   literal address with a scheme a browser runs.
//! - `T01`: a `Carousel(interval:)` no timer keeps, a `Column(span:)` that
//!   is not a whole number of the twelve columns.

use crate::diagnostics::error::A11yWarning;
use crate::syntax::parser::ast::*;

/// Every finding of this module in `program`.
pub fn lint_output(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<A11yWarning> {
    let mut out = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let file = file_of(index);
        match decl {
            Declaration::Page(page) => {
                head_tags(&page.head, &file, &mut out);
                walk(&page.body, &file, &mut out);
            }
            Declaration::Component(c) => walk(&c.body, &file, &mut out),
            Declaration::App(a) => walk(&a.body, &file, &mut out),
            _ => {}
        }
    }
    out
}

fn walk(body: &[Statement], file: &str, out: &mut Vec<A11yWarning>) {
    for stmt in body {
        if let StatementKind::UIElement(el) = &stmt.kind {
            element(el, file, out);
            walk(&el.children, file, out);
            for fill in &el.slot_fills {
                walk(&fill.body, file, out);
            }
        }
        for inner in stmt.kind.bodies() {
            walk(inner, file, out);
        }
    }
}

fn named<'a>(el: &'a UIElement, name: &str) -> Option<&'a Expr> {
    el.args.iter().find_map(|a| match a {
        Arg::Named(k, v) if k == name => Some(v),
        _ => None,
    })
}

/// A number written out, negative ones too.
fn number(e: &Expr) -> Option<f64> {
    match e {
        Expr::NumberLiteral(n) => Some(*n),
        Expr::UnaryOp(UnaryOp::Neg, inner) => number(inner).map(|n| -n),
        _ => None,
    }
}

fn element(el: &UIElement, file: &str, out: &mut Vec<A11yWarning>) {
    let ComponentRef::BuiltIn(name) = &el.component else {
        return;
    };
    let (line, col) = (el.span.line as usize, el.span.col as usize);
    match name.as_str() {
        "Frame" => {
            let Some(doc) = named(el, "srcdoc") else {
                return;
            };
            if matches!(doc, Expr::StringLiteral(_)) {
                return;
            }
            match named(el, "sandbox") {
                None => out.push(A11yWarning::new(
                    "V03",
                    "`Frame(srcdoc:)` draws HTML from a value as a document; it is sandboxed (`sandbox=\"\"`), so nothing in it runs",
                    file,
                    line,
                    col,
                    "Say what it may do with `sandbox:` — `sandbox: \"\"` keeps it as it is now; `allow-scripts` and `allow-same-origin` together undo the sandbox",
                )),
                Some(Expr::StringLiteral(s))
                    if s.split_whitespace()
                        .any(|t| t.eq_ignore_ascii_case("allow-scripts"))
                        && s.split_whitespace()
                            .any(|t| t.eq_ignore_ascii_case("allow-same-origin")) =>
                {
                    out.push(A11yWarning::new(
                        "V03",
                        "`Frame(srcdoc:)` draws HTML from a value with `allow-scripts` and `allow-same-origin`: what it runs runs as this site",
                        file,
                        line,
                        col,
                        "Drop one of the two, or put the HTML through `sanitize(…)` first",
                    ))
                }
                Some(Expr::StringLiteral(_)) => {}
                Some(_) => out.push(A11yWarning::new(
                    "V03",
                    "`Frame(srcdoc:)` draws HTML from a value, sandboxed as a value says",
                    file,
                    line,
                    col,
                    "Write the sandbox out (`sandbox: \"\"`, `sandbox: \"allow-scripts\"`), so a review can read what the frame may do",
                )),
            }
        }
        "Carousel" => {
            if let Some(n) = named(el, "interval").and_then(number)
                && !(1.0..=crate::project::builtin::LONGEST_INTERVAL).contains(&n)
            {
                out.push(A11yWarning::new(
                    "T01",
                    format!(
                        "`interval` of Carousel is {n}, but a number of milliseconds from 1 to 2147483647 is wanted"
                    ),
                    file,
                    line,
                    col,
                    "The time each slide shows, in milliseconds: `interval: 5000`",
                ));
            }
        }
        "Column" => {
            if let Some(n) = named(el, "span").and_then(number)
                && crate::project::builtin::column_span(n).is_none()
            {
                out.push(A11yWarning::new(
                    "T01",
                    format!(
                        "`span` of Column is {n}, but a whole number of the 12 columns, from 1 to 12, is wanted"
                    ),
                    file,
                    line,
                    col,
                    "`span: 6` is half the row",
                ));
            }
        }
        _ => {}
    }
}

/// The address a refresh's `content` names, after `url=`.
fn refresh_target(content: &str) -> Option<&str> {
    let lower = content.to_ascii_lowercase();
    let at = lower.find("url")?;
    let rest = content[at + 3..].trim_start();
    let rest = rest.strip_prefix('=')?.trim();
    Some(rest.trim_matches(|c| c == '\'' || c == '"'))
}

fn head_tags(tags: &[HeadTag], file: &str, out: &mut Vec<A11yWarning>) {
    for tag in tags {
        let (line, col) = (tag.span.line as usize, tag.span.col as usize);
        let attr = |name: &str| {
            tag.attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v)
        };
        let mut refuse = |message: String, hint: &str| {
            out.push(A11yWarning::new("E107", message, file, line, col, hint));
        };
        // A literal address with a scheme a browser runs.
        for (key, value) in &tag.attrs {
            if let Expr::StringLiteral(url) = value
                && crate::project::url::URL_ATTRS.contains(&key.to_ascii_lowercase().as_str())
                && !crate::project::url::is_safe(url)
            {
                refuse(
                    format!(
                        "`head {{ {}({key}:) }}` names the `{}` scheme, which a browser runs",
                        tag.tag,
                        crate::project::url::scheme_of(url).unwrap_or_default()
                    ),
                    "An address here may be relative, or name http, https, mailto, tel, sms or ftp",
                );
            }
        }
        match tag.tag.as_str() {
            "script" => {
                if let Some(src) = attr("src")
                    && !matches!(src, Expr::StringLiteral(_))
                {
                    refuse(
                        "`head { script(src: …) }` loads a script a value chooses: whoever controls the value runs code as this site".to_string(),
                        "Write the address out — `script(src: \"/analytics.js\")` — and keep what varies in the script's own data",
                    );
                }
            }
            "meta" => {
                let Some(equiv) = attr("http-equiv") else {
                    continue;
                };
                let Expr::StringLiteral(equiv) = equiv else {
                    refuse(
                        "`head { meta(http-equiv: …) }` from a value: a value could make it a refresh to anywhere".to_string(),
                        "Write the `http-equiv` out",
                    );
                    continue;
                };
                if !equiv.trim().eq_ignore_ascii_case("refresh") {
                    continue;
                }
                match attr("content") {
                    Some(Expr::StringLiteral(content)) => {
                        if let Some(url) = refresh_target(content)
                            && !crate::project::url::is_safe(url)
                        {
                            refuse(
                                format!(
                                    "`head {{ meta(http-equiv: \"refresh\") }}` sends the reader to the `{}` scheme, which a browser runs",
                                    crate::project::url::scheme_of(url).unwrap_or_default()
                                ),
                                "A refresh may name a relative address, or http or https",
                            );
                        }
                    }
                    Some(_) => refuse(
                        "`head { meta(http-equiv: \"refresh\") }` whose address comes from a value sends the reader wherever the value says".to_string(),
                        "Write the refresh out, or send the reader on with `navigate(…)`, which keeps to the site's routes",
                    ),
                    None => {}
                }
            }
            _ => {}
        }
    }
}
