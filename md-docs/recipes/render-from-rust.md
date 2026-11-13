---
kind: "recipe"
route: "/recipes/render-from-rust"
group: "interop"
description: "Render WebFluent templates from a Rust program or server with the webfluent crate: load them once, then render HTML, fragments, PDFs and slides."
outputs: ["website", "pdf", "slides"]
from: ["/docs/guide/server-rendering#render-from-rust"]
---
# Render templates from Rust

The template engine is the `webfluent` crate. Load the templates once, at
start-up, and render them with your own data as HTML, a fragment of a
page, a PDF or slides. You need a Rust project, and a template like the
one in [Render a template with data](render-a-template.md). This one is
`templates/invoice.wf`:

```wf
page Invoice(path: "/", title: "Invoice {number}", description: "An invoice.") {
    Heading("Invoice #{number}").h1
    Text("Bill to: {customer.name}")
    Table {
        Table.Head { Table.Row { Table.Cell("Item")  Table.Cell("Qty")  Table.Cell("Amount") } }
        Table.Body {
            for item in items {
                Table.Row {
                    Table.Cell(item.name)
                    Table.Cell("{item.qty}")
                    Table.Cell(format(item.price * item.qty, .currency))
                }
            }
        }
    }
    if paid { Badge("Paid").success } else { Badge("Due").warning }
    Text("Total: {format(total, .currency)}").bold.lg
}
```

To use the engine without the `wf` command's own dependencies (its
argument parser and dev server), turn off the default features in
`Cargo.toml`:

```toml
[dependencies]
webfluent = { version = "6", default-features = false }
serde = { version = "1", features = ["derive"] }
```

This program, `src/main.rs`, renders the `Invoice` page three ways and
writes the results beside the project:

```rust
use serde::Serialize;
use webfluent::{PdfConfig, Template};

#[derive(Serialize)]
struct Customer { name: String }

#[derive(Serialize)]
struct Item { name: String, qty: u32, price: f64 }

#[derive(Serialize)]
struct Invoice { number: String, customer: Customer, items: Vec<Item>, total: f64, paid: bool }

fn main() -> webfluent::Result<()> {
    // Every .wf and .wfx under the directory, as one template: components,
    // themes, types and constants shared; each `page` a document.
    let templates = Template::from_dir("templates")?;
    let page = templates.page("Invoice")?;

    let invoice = Invoice {
        number: "INV-001".into(),
        customer: Customer { name: "Acme Corp".into() },
        items: vec![Item { name: "Widget".into(), qty: 5, price: 9.99 }],
        total: 49.95,
        paid: false,
    };

    let html = page.render_html(&invoice)?;          // a whole document
    let body = page.render_html_fragment(&invoice)?; // the markup alone
    let pdf: Vec<u8> = page
        .clone()
        .with_pdf(PdfConfig { page_size: "Letter".into(), ..PdfConfig::default() })
        .render_pdf(&invoice)?;

    std::fs::write("invoice.html", html)?;
    std::fs::write("invoice.pdf", pdf)?;
    println!("{body}");
    Ok(())
}
```

`cargo run` prints the fragment and writes `invoice.html` and
`invoice.pdf`. To try a fuller example, run `cargo run --example invoice`
in the WebFluent repository: it renders a directory of templates to HTML,
a fragment and a PDF.

## How it works

### Load templates

Each loader parses and checks the template once. An error names the file
and the line.

| Loader | Reads |
|---|---|
| `Template::from_str(source)` | One template from a string |
| `Template::from_file(path)` | One file; a `.wfx` file is read in its indented layout |
| `Template::from_files(&[..])` | Several files, as one template |
| `Template::from_dir(dir)` | Every `.wf` and `.wfx` under a directory, as one template |
| `Template::from_sources(&[(name, source), ..])` | Templates embedded in the binary with `include_str!` |

`pages()` lists the template's pages, and `page(name)` picks one. Without
`page(name)`, every page of the template renders. A name the template does
not have is an error that lists the pages it does have:

```text
no page called `Receipt`; the template has `Invoice`
```

### Pass data

The data is anything `serde` serializes: your own `#[derive(Serialize)]`
structs, or a `serde_json::Value`. Its top-level fields are the names the
template reads, so `Invoice`'s `number`, `customer` and `items` are what
the template splices. A `const` and a `data` file are in scope too.

### Choose the output

| Method | Returns |
|---|---|
| `render_html` | A whole document: `<html lang>`, a `<title>` from the page's (its `{…}` filled from the data), and the CSS in a `<style>` block |
| `render_html_parts` | The CSS and the markup separately, for a server that links its stylesheet and keeps a strict Content-Security-Policy |
| `render_html_fragment` | The markup alone |
| `render_pdf`, `render_slides` | PDF bytes, laid out by the paged engine of [PDF and slides](../learn/17-pdf-and-slides.md) |
| `render_pdf_report`, `render_slides_report` | A `PdfReport`: the bytes, the page count, each page's text in reading order, and the notes (a font taken from the machine, a character no font had) |

A `PdfReport`'s page text lets a test hold a document to what it says. A
template loaded from files reads its fonts from `fonts/` beside it, and its
pictures from there too.

### Configure a render

Each setting returns the `Template`, so you can chain them before a render,
as the program does with `with_pdf`:

- `with_theme(name)` picks a theme;
- `with_tokens(&[(name, value)])` sets design tokens over the theme's;
- `with_lang("ar")` sets the language (a right-to-left language also sets
  `dir="rtl"`); without it, the page's own `lang:` is used, else `en`;
- `with_pdf(PdfConfig { .. })` sets page size, margins and fonts;
- `with_slides(SlidesConfig { .. })` sets a deck's options.

A setting takes the `Template` by value, so the program clones `page`
before it sets the paper size, and keeps `page` for other renders.

## Use it in a server

A `Template` is `Send + Sync` and cheap to clone, because the parsed
program is shared. Keep it in your framework's state — an `Arc`, or a clone
per handler — and render from any thread. A render takes tens of
microseconds.

> [!NOTE]
> Every value the data supplies is escaped as text. A URL from the data
> that a browser would run — `javascript:`, `data:` — is dropped from
> `href` and `src`, as the browser build drops it. An `Image`'s `src` keeps
> a picture carried in its address (`data:image/png;base64,…`), as the
> browser build keeps it.
>
> A value the data puts in a `style { }` block is one value, as the live
> page sets one property at a time: a piece with a `;`, a brace or a
> backslash, or one that fetches something (`url(…)`, `image-set(…)`),
> leaves its declaration out. What the template writes around a splice —
> `background: url({img})` — is the template's own. A `class:` from the
> data is escaped inside its attribute.

## Limits of one render

A render is bounded, so one request cannot take a server with it:

| Limit | What is past it |
|---|---|
| 100 levels of nesting — elements, and the components they place, counted together | `E120` |
| 10,000,000 steps — elements drawn, loop items, units of working a value out | `E121` |
| 256 MB of markup | `E121` |
| About 500,000 list items or 32 MB of text made by one expression | `E121` |
| `>` quotes in `Markdown` nested 32 deep | The deeper `>` is shown as text |
| A precision past 100 digits (`toFixed`, `format(…, .decimal, n)`) | `toFixed` shows nothing, as the browser throws; `format` writes 100 |

Each error is a `WebFluentError::Diagnostics` naming the code and the
element where the render stopped.

## Troubleshooting

| What you see | What to do |
|---|---|
| `no page called …` | Pass `page(name)` one of the names the message lists |
| `WebFluentError::Diagnostics` when the template loads | The template does not check: the error names the file, the line and the code, as `wf check` would |
| A `{name}` renders empty | The data has no top-level field `name`; check the struct's field names, or `#[serde(rename)]` |
