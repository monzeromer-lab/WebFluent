//! WebFluent templates served by Actix Web: an invoice as a web page, and
//! the same invoice as a PDF to download.
//!
//!     cargo run --example actix_server --features actix
//!
//! then open <http://127.0.0.1:3000/invoices/1042> and
//! <http://127.0.0.1:3000/invoices/1042.pdf>. `PORT` picks another port.
//!
//! In your own project:
//! `webfluent = { version = "8.1", default-features = false, features = ["actix"] }`.

use actix_web::{App, HttpServer, error::ErrorNotFound, get, web};
use serde::Serialize;
use std::collections::HashMap;
use webfluent::{Format, Rendered, Template};

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

/// What every handler shares, in the app's data. A `Template` is cheap to
/// clone and safe to share across Actix's worker threads.
struct Site {
    invoice: Template,
    invoices: HashMap<u32, Invoice>,
}

/// `/invoices/1042` is the page; `/invoices/1042.pdf` the PDF. A failed
/// render is a `WebFluentError`, which `?` turns into a `500`.
#[get("/invoices/{file}")]
async fn invoice(site: web::Data<Site>, file: web::Path<String>) -> actix_web::Result<Rendered> {
    let file = file.into_inner();
    let (number, pdf) = match file.strip_suffix(".pdf") {
        Some(number) => (number, true),
        None => (file.as_str(), false),
    };
    let invoice = number
        .parse()
        .ok()
        .and_then(|n: u32| site.invoices.get(&n))
        .ok_or_else(|| ErrorNotFound("No such invoice.\n"))?;
    if pdf {
        // A PDF is laid out on Actix's blocking pool, so it never holds up
        // the requests a worker is answering.
        let pdf = webfluent::actix::render(&site.invoice, invoice, Format::Pdf).await?;
        Ok(pdf.download(&format!("invoice-{number}.pdf")))
    } else {
        // An HTML page takes microseconds: rendered right here.
        Ok(site.invoice.respond(invoice, Format::Html)?)
    }
}

#[get("/")]
async fn index() -> &'static str {
    "Try /invoices/1042 and /invoices/1042.pdf\n"
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parsed and checked once, at start-up: a mistake in a template stops
    // the server here, not on the first request.
    let templates = Template::from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/templates"))?;
    let site = web::Data::new(Site {
        invoice: templates.page("Invoice")?,
        invoices: HashMap::from([(1042, sample(1042))]),
    });

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(site.clone())
            .service(index)
            .service(invoice)
    })
    .bind(("127.0.0.1", port))?;
    println!("Listening on http://{}/invoices/1042", server.addrs()[0]);
    server.run().await?;
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
