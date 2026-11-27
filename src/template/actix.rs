//! Templates in an [Actix Web](https://docs.rs/actix-web) server: the
//! `actix` feature.
//!
//! A [`Template`] is cheap to clone and safe to share, so it goes in the
//! app's data as it is. A [`Rendered`] is a `Responder`, and a
//! [`WebFluentError`] a `ResponseError` — a `500` — so a handler returns
//! `Result<Rendered, WebFluentError>` and uses `?`.
//!
//! ```rust,no_run
//! use actix_web::{App, HttpServer, get, web};
//! use serde::Serialize;
//! use webfluent::{Format, Rendered, Template, WebFluentError};
//!
//! #[derive(Serialize)]
//! struct Invoice { number: String, total: f64 }
//!
//! #[get("/invoices/{number}")]
//! async fn invoice(
//!     tpl: web::Data<Template>,
//!     number: web::Path<String>,
//! ) -> Result<Rendered, WebFluentError> {
//!     let number = number.into_inner();
//!     let data = Invoice { number: number.clone(), total: 12.5 };
//!     // A PDF is laid out on the blocking pool, not on the workers.
//!     let pdf = webfluent::actix::render(&tpl, &data, Format::Pdf).await?;
//!     Ok(pdf.inline(&format!("invoice-{number}.pdf")))
//! }
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let tpl = web::Data::new(Template::from_dir("templates")?.page("Invoice")?);
//! HttpServer::new(move || App::new().app_data(tpl.clone()).service(invoice))
//!     .bind(("127.0.0.1", 3000))?
//!     .run()
//!     .await?;
//! # Ok(()) }
//! ```

use super::{Format, Rendered, Template};
use crate::diagnostics::error::{Result, WebFluentError};
use actix_web::http::StatusCode;
use actix_web::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, HeaderValue};
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, body::BoxBody};
use serde::Serialize;

impl Responder for Rendered {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut response = HttpResponse::Ok();
        response.insert_header((CONTENT_TYPE, HeaderValue::from_static(self.content_type())));
        if let Some(d) = self
            .content_disposition()
            .and_then(|d| HeaderValue::from_str(d).ok())
        {
            response.insert_header((CONTENT_DISPOSITION, d));
        }
        response.body(self.into_body())
    }
}

impl ResponseError for WebFluentError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// A `500` in plain text: what went wrong in a debug build, that the
    /// page could not be drawn in a release one.
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(super::respond::failure_text(self))
    }
}

/// [`Template::respond`] on Actix's blocking pool, so a PDF being laid out
/// never holds up the requests a worker is serving.
///
/// The data is read once, here; the render then owns it.
///
/// # Errors
///
/// As [`Template::respond`], and an [`IoError`](WebFluentError::IoError)
/// when the data cannot be read as JSON or the render was stopped.
pub async fn render<T: Serialize + ?Sized>(
    template: &Template,
    data: &T,
    format: Format,
) -> Result<Rendered> {
    let data = serde_json::to_value(data)
        .map_err(|e| WebFluentError::io_with("the data could not be read as JSON", e))?;
    let template = template.clone();
    actix_web::web::block(move || template.respond(&data, format))
        .await
        .map_err(|e| WebFluentError::io_with("the render did not finish", e))?
}
