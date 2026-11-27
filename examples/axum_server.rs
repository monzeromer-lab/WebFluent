//! WebFluent templates served by axum: an invoice as a web page, and the
//! same invoice as a PDF to download.
//!
//!     cargo run --example axum_server --features axum
//!
//! then open <http://127.0.0.1:3000/invoices/1042> and
//! <http://127.0.0.1:3000/invoices/1042.pdf>. `PORT` picks another port.
//!
//! In your own project:
//! `webfluent = { version = "8.1", default-features = false, features = ["axum"] }`.

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use webfluent::{Format, Template};

#[derive(Serialize)]
struct Customer {
    name: String,
}

#[derive(Serialize)]
struct Line {
    label: String,
    amount: f64,
}

#[derive(Serialize)]
struct Invoice {
    number: u32,
    customer: Customer,
    lines: Vec<Line>,
    total: f64,
    paid: bool,
}

/// What every handler shares. A `Template` is cheap to clone: the parsed
/// program inside it is shared, so this is the router's state as it is.
#[derive(Clone)]
struct App {
    invoice: Template,
    invoices: Arc<HashMap<u32, Invoice>>,
}

/// `/invoices/1042` is the page; `/invoices/1042.pdf` the PDF.
async fn invoice(State(app): State<App>, Path(file): Path<String>) -> Response {
    let (number, pdf) = match file.strip_suffix(".pdf") {
        Some(number) => (number, true),
        None => (file.as_str(), false),
    };
    let Some(invoice) = number.parse().ok().and_then(|n: u32| app.invoices.get(&n)) else {
        return (StatusCode::NOT_FOUND, "No such invoice.\n").into_response();
    };
    if pdf {
        // A PDF is laid out on tokio's blocking pool, so it never holds up
        // the requests the async workers are answering.
        webfluent::axum::render(&app.invoice, invoice, Format::Pdf)
            .await
            .map(|pdf| pdf.download(&format!("invoice-{number}.pdf")))
            .into_response()
    } else {
        // An HTML page takes microseconds: rendered right here.
        app.invoice.respond(invoice, Format::Html).into_response()
    }
}

async fn index() -> &'static str {
    "Try /invoices/1042 and /invoices/1042.pdf\n"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parsed and checked once, at start-up: a mistake in a template stops
    // the server here, not on the first request.
    let templates = Template::from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/templates"))?;
    let app = App {
        invoice: templates.page("Invoice")?,
        invoices: Arc::new(HashMap::from([(1042, sample(1042))])),
    };

    let router = Router::new()
        .route("/", get(index))
        .route("/invoices/{file}", get(invoice))
        .with_state(app);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    println!(
        "Listening on http://{}/invoices/1042",
        listener.local_addr()?
    );
    axum::serve(listener, router).await?;
    Ok(())
}

fn sample(number: u32) -> Invoice {
    Invoice {
        number,
        customer: Customer {
            name: "Acme Corp".into(),
        },
        lines: vec![
            Line {
                label: "Design".into(),
                amount: 1200.0,
            },
            Line {
                label: "Hosting, one year".into(),
                amount: 240.0,
            },
        ],
        total: 1440.0,
        paid: false,
    }
}
