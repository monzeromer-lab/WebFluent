//! A render as an HTTP response: its bytes, the media type they are, and
//! whether a browser shows them or saves them. Every web framework can send
//! one; the `axum` and `actix` features make it a response those two take
//! as it is (see [`crate::axum`] and [`crate::actix`]).

use super::Template;
use crate::diagnostics::error::Result;
use serde::Serialize;

/// What a render writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Format {
    /// A whole HTML document: `<title>`, `<html lang>` and the stylesheet.
    Html,
    /// The page's markup alone, for a page of your own to place.
    Fragment,
    /// A PDF document.
    Pdf,
    /// A PDF slide deck.
    Slides,
}

impl Format {
    /// The media type of what this format writes.
    #[must_use]
    pub fn content_type(self) -> &'static str {
        match self {
            Format::Html | Format::Fragment => "text/html; charset=utf-8",
            Format::Pdf | Format::Slides => "application/pdf",
        }
    }
}

/// A rendered page, ready to send: its bytes and the headers that say what
/// they are.
///
/// Made by [`Template::respond`]. With the `axum` feature it is an
/// `IntoResponse`, with `actix` a `Responder`; with neither, its parts are
/// here for any framework.
///
/// ```rust
/// use webfluent::{Format, Template};
/// use serde_json::json;
///
/// let tpl = Template::from_str(r#"page Hi(path: "/") { Heading("Hi, {name}").h1 }"#).unwrap();
/// let page = tpl.respond(&json!({ "name": "Ada" }), Format::Html).unwrap();
/// assert_eq!(page.content_type(), "text/html; charset=utf-8");
/// assert!(String::from_utf8_lossy(page.body()).contains("Hi, Ada"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    body: Vec<u8>,
    format: Format,
    disposition: Option<String>,
}

impl Rendered {
    /// A response of these bytes, in this format.
    #[must_use]
    pub fn new(body: Vec<u8>, format: Format) -> Self {
        Self {
            body,
            format,
            disposition: None,
        }
    }

    /// The bytes.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// The bytes, taken.
    #[must_use]
    pub fn into_body(self) -> Vec<u8> {
        self.body
    }

    /// What was rendered.
    #[must_use]
    pub fn format(&self) -> Format {
        self.format
    }

    /// The `Content-Type` header's value.
    #[must_use]
    pub fn content_type(&self) -> &'static str {
        self.format.content_type()
    }

    /// The `Content-Disposition` header's value, when
    /// [`download`](Self::download) or [`inline`](Self::inline) set one.
    #[must_use]
    pub fn content_disposition(&self) -> Option<&str> {
        self.disposition.as_deref()
    }

    /// Saved rather than shown: the browser offers to save it as
    /// `filename` (`invoice-1042.pdf`).
    #[must_use]
    pub fn download(self, filename: &str) -> Self {
        self.disposed("attachment", filename)
    }

    /// Shown in the browser, and saved as `filename` when the reader saves
    /// it.
    #[must_use]
    pub fn inline(self, filename: &str) -> Self {
        self.disposed("inline", filename)
    }

    fn disposed(mut self, kind: &str, filename: &str) -> Self {
        self.disposition = Some(disposition(kind, filename));
        self
    }
}

/// `attachment; filename="…"`, the name kept to what a header may carry,
/// with the whole name in `filename*` (RFC 6266, RFC 8187) when it is not
/// plain ASCII.
fn disposition(kind: &str, filename: &str) -> String {
    // A path is not a name: only what follows its last separator.
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let plain: String = name
        .chars()
        .map(|c| {
            if (c.is_ascii_graphic() && c != '"' && c != '\\') || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut out = format!("{kind}; filename=\"{plain}\"");
    if plain != name {
        out.push_str("; filename*=UTF-8''");
        for b in name.bytes() {
            if b.is_ascii_alphanumeric() || b"!#$&+-.^_`|~".contains(&b) {
                out.push(b as char);
            } else {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

impl Template {
    /// Renders `data` as `format` and wraps it as a response: the bytes
    /// and the media type they are.
    ///
    /// This runs on the calling thread. A PDF takes tens of milliseconds or
    /// more, so an async handler hands it to the runtime's blocking pool —
    /// what `webfluent::axum::render` and `webfluent::actix::render` do.
    ///
    /// # Errors
    ///
    /// As [`render_html`](Template::render_html) and
    /// [`render_pdf`](Template::render_pdf): data that is not what the page
    /// takes, or a page the engine cannot lay out.
    pub fn respond<T: Serialize + ?Sized>(&self, data: &T, format: Format) -> Result<Rendered> {
        let body = match format {
            Format::Html => self.render_html(data)?.into_bytes(),
            Format::Fragment => self.render_html_fragment(data)?.into_bytes(),
            Format::Pdf => self.render_pdf(data)?,
            Format::Slides => self.render_slides(data)?,
        };
        Ok(Rendered::new(body, format))
    }
}

/// What a failed render sends: a `500`, and — in a debug build — what
/// went wrong, so a developer sees `$.items[0].price is missing` in the
/// browser; a release build says only that the page could not be drawn,
/// since the message names the data's shape.
#[cfg(any(feature = "axum", feature = "actix"))]
pub(crate) fn failure_text(error: &crate::diagnostics::error::WebFluentError) -> String {
    if cfg!(debug_assertions) {
        format!("The page could not be rendered: {error}\n")
    } else {
        "The page could not be rendered.\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_download_name_is_kept_to_what_a_header_may_carry() {
        assert_eq!(
            disposition("attachment", "invoice-1042.pdf"),
            "attachment; filename=\"invoice-1042.pdf\""
        );
        assert_eq!(
            disposition("inline", "../../etc/a \"b\".pdf"),
            "inline; filename=\"a _b_.pdf\"; filename*=UTF-8''a%20%22b%22.pdf"
        );
        assert_eq!(
            disposition("attachment", "فاتورة.pdf"),
            "attachment; filename=\"______.pdf\"; filename*=UTF-8''%D9%81%D8%A7%D8%AA%D9%88%D8%B1%D8%A9.pdf"
        );
        // No line break reaches the header.
        assert!(!disposition("attachment", "a\r\nSet-Cookie: x.pdf").contains('\n'));
    }

    #[test]
    fn each_format_says_what_it_is() {
        let tpl = Template::from_str(r#"page P(path: "/") { Document { Text("Hi") } }"#).unwrap();
        let data = serde_json::json!({});
        let html = tpl.respond(&data, Format::Html).unwrap();
        assert_eq!(html.content_type(), "text/html; charset=utf-8");
        assert!(html.body().starts_with(b"<!DOCTYPE html>"));
        let pdf = tpl.respond(&data, Format::Pdf).unwrap().download("hi.pdf");
        assert_eq!(pdf.content_type(), "application/pdf");
        assert!(pdf.body().starts_with(b"%PDF"));
        assert_eq!(
            pdf.content_disposition(),
            Some("attachment; filename=\"hi.pdf\"")
        );
    }
}
