//! WebFluent as a template engine: one set of templates, loaded once and
//! rendered with your own data — an HTML document, a fragment to embed, and
//! a PDF.
//!
//!     cargo run --example invoice
//!
//! In your own project, the engine alone is
//! `webfluent = { version = "4", default-features = false }`.

use serde::Serialize;
use webfluent::{PdfConfig, Template};

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

fn main() -> webfluent::Result<()> {
    // Every .wf under the directory is one template: the components are
    // shared, and each page is a document to render by name. Build it once —
    // it is `Send + Sync`, so a server keeps it in its state.
    let templates = Template::from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/templates"))?;
    println!("pages: {:?}", templates.pages());

    let invoice = Invoice {
        number: 1042,
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
    };

    // A whole document: <title>, the theme's CSS, and the page.
    let html = templates.page("Invoice")?.render_html(&invoice)?;
    // Just the markup, for a page your server already lays out.
    let receipt = templates.page("Receipt")?.render_html_fragment(&invoice)?;
    // The same page, as a PDF on US Letter paper.
    let pdf = templates
        .page("Invoice")?
        .with_pdf(PdfConfig {
            page_size: "Letter".into(),
            ..PdfConfig::default()
        })
        .render_pdf(&invoice)?;

    let out = std::env::temp_dir();
    std::fs::write(out.join("invoice.html"), &html)?;
    std::fs::write(out.join("invoice.pdf"), &pdf)?;
    println!(
        "invoice.html: {} bytes, invoice.pdf: {} bytes, in {}",
        html.len(),
        pdf.len(),
        out.display()
    );
    println!("receipt fragment:\n{receipt}");
    Ok(())
}
