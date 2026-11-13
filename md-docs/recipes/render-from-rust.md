---
kind: "recipe"
route: "/recipes/render-from-rust"
group: "interop"
description: "The template engine is the `webfluent` crate."
from: ["/docs/guide/server-rendering#render-from-rust"]
---
# Render templates from Rust

The template engine is the `webfluent` crate. To use it without the `wf`
command's own dependencies (its argument parser and dev server), turn off
the default features:

```toml
[dependencies]
webfluent = { version = "6", default-features = false }
serde = { version = "1", features = ["derive"] }
```

Load the templates once, at start-up, and render on every request. This
code loads a directory of templates and renders its `Invoice` page three
ways:

```rust
use serde::Serialize;
use webfluent::{PdfConfig, Template};

#[derive(Serialize)]
struct Customer { name: String }

#[derive(Serialize)]
struct Item { name: String, qty: u32, price: f64 }

#[derive(Serialize)]
struct Invoice { number: u32, customer: Customer, items: Vec<Item>, total: f64, paid: bool }

// Every .wf and .wfx under the directory, as one template: components,
// themes, types and constants shared; each `page` a document.
let templates = Template::from_dir("templates")?;
let invoice: Invoice = load_invoice();

let html = templates.page("Invoice")?.render_html(&invoice)?;          // a whole document
let body = templates.page("Invoice")?.render_html_fragment(&invoice)?; // the markup alone
let pdf: Vec<u8> = templates
    .page("Invoice")?
    .with_pdf(PdfConfig { page_size: "Letter".into(), ..PdfConfig::default() })
    .render_pdf(&invoice)?;
```

To try it, run `cargo run --example invoice` in the WebFluent repository.
The example renders a directory of templates to HTML, a fragment and a PDF.

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
template reads. A `const` and a `data` file are in scope too.

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

Each setting returns the `Template`, so you can chain them before a render:

- `with_theme(name)` picks a theme;
- `with_tokens(&[(name, value)])` sets design tokens over the theme's;
- `with_lang("ar")` sets the language (a right-to-left language also sets
  `dir="rtl"`); without it, the page's own `lang:` is used, else `en`;
- `with_pdf(PdfConfig { .. })` sets page size, margins and fonts;
- `with_slides(SlidesConfig { .. })` sets a deck's options.

### Use it in a server

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

### Limits of one render

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
