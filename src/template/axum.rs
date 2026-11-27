//! Templates in an [axum](https://docs.rs/axum) server: the `axum` feature.
//!
//! A [`Template`] is cheap to clone and safe to share, so it is the
//! router's state as it is. A [`Rendered`] is an `IntoResponse`, and so is
//! a [`WebFluentError`] — a `500` — so a handler returns
//! `Result<Rendered, WebFluentError>` and uses `?`.
//!
//! ```rust,no_run
//! use axum::{Router, extract::{Path, State}, routing::get};
//! use serde::Serialize;
//! use webfluent::{Format, Rendered, Template, WebFluentError};
//!
//! #[derive(Serialize)]
//! struct Invoice { number: String, total: f64 }
//!
//! async fn invoice(
//!     State(tpl): State<Template>,
//!     Path(number): Path<String>,
//! ) -> Result<Rendered, WebFluentError> {
//!     let data = Invoice { number: number.clone(), total: 12.5 };
//!     // A PDF is laid out on the blocking pool, not on the async workers.
//!     let pdf = webfluent::axum::render(&tpl, &data, Format::Pdf).await?;
//!     Ok(pdf.inline(&format!("invoice-{number}.pdf")))
//! }
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let tpl = Template::from_dir("templates")?.page("Invoice")?;
//! let app = Router::new().route("/invoices/{number}", get(invoice)).with_state(tpl);
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
//! axum::serve(listener, app).await?;
//! # Ok(()) }
//! ```

use super::{Format, Rendered, Template};
use crate::diagnostics::error::{Result, WebFluentError};
use axum_core::body::Body;
use axum_core::response::{IntoResponse, Response};
use http::{HeaderValue, StatusCode, header};
use serde::Serialize;

impl IntoResponse for Rendered {
    fn into_response(self) -> Response {
        let content_type = HeaderValue::from_static(self.content_type());
        let disposition = self
            .content_disposition()
            .and_then(|d| HeaderValue::from_str(d).ok());
        let mut response = Response::new(Body::from(self.into_body()));
        let headers = response.headers_mut();
        headers.insert(header::CONTENT_TYPE, content_type);
        if let Some(d) = disposition {
            headers.insert(header::CONTENT_DISPOSITION, d);
        }
        response
    }
}

impl IntoResponse for WebFluentError {
    /// A `500` in plain text: what went wrong in a debug build, that the
    /// page could not be drawn in a release one.
    fn into_response(self) -> Response {
        let mut response = Response::new(Body::from(super::respond::failure_text(&self)));
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response
    }
}

/// [`Template::respond`] on tokio's blocking pool, so a PDF being laid out
/// never holds up the requests the async workers are serving.
///
/// The data is read once, here; the render then owns it.
///
/// # Errors
///
/// As [`Template::respond`], and an [`IoError`](WebFluentError::IoError)
/// when the data cannot be read as JSON or the render was stopped (a
/// panic, a runtime shutting down).
pub async fn render<T: Serialize + ?Sized>(
    template: &Template,
    data: &T,
    format: Format,
) -> Result<Rendered> {
    let data = serde_json::to_value(data)
        .map_err(|e| WebFluentError::io_with("the data could not be read as JSON", e))?;
    let template = template.clone();
    tokio::task::spawn_blocking(move || template.respond(&data, format))
        .await
        .map_err(|e| WebFluentError::io_with("the render did not finish", e))?
}
