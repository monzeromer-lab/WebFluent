---
chapter: 38
kind: "recipe"
route: "guide/server-rendering"
group: "outputs"
blurb: "Render a .wf template with JSON on a server — an email, an invoice, a report, an HTML fragment — from the CLI, Rust or Node."
description: "Use WebFluent as a template engine: wf render, the Rust API and the Node binding, with HTML, fragment and PDF output, pages, themes and data."
---
# 38. Server rendering

This chapter shows you how to render a `.wf` template with JSON data on a
server — an email, an invoice PDF per order, a fragment for an existing
page — from the command line, from Rust or from Node.

A WebFluent site is static files, and most projects need nothing more.
When HTML or a PDF has to be made on a server for each request or each
record, the same language works as a *template engine*: you hand it a
template and data, and it hands back HTML or PDF bytes. Nothing runs in a
browser afterwards.

## Before you start

You need the `wf` command installed ([Getting started](../learn/01-getting-started.md#install)).
For the Rust API you need a Rust project; for the Node binding, Node.js and
npm.

## Write a template and its data

A template is a `.wf` file with a `page`. The data's top-level keys are in
scope by name, so the template reads `number`, `customer` and `items`
without declaring them. This invoice template shows a table of items and a
badge for whether it is paid:

```wf
page Invoice(path: "/", title: "Invoice", description: "An invoice.") {
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

The data is a JSON object, saved here as `invoice.json`:

```json
{
  "number": "INV-001",
  "customer": { "name": "Acme Corp" },
  "items": [{ "name": "Widget", "qty": 5, "price": 9.99 }],
  "total": 49.95,
  "paid": false,
  "locale": "en"
}
```

Rendered, the page reads `Invoice #INV-001`, `Bill to: Acme Corp`, one table
row (`Widget`, `5`, `$49.95`), the badge `Due` and `Total: $49.95`.

A template uses the static subset of the language: no handlers, stores,
`resource` or motion (see [What a template may use](#what-a-template-may-use)).

## Render from the command line

To render a template, run `wf render` with the template and its data:

```bash
wf render invoice.wf --data invoice.json --format html-fragment
```

The command prints the markup to standard output. Its options:

| Option | What it does |
|---|---|
| `--data FILE` | The JSON data. When it is omitted, the data is read from standard input |
| `--format`, `-f` | `html` (a whole document with its CSS; the default), `html-fragment` (the body only), `pdf` or `slides` |
| `-o FILE`, `--output` | Writes to a file instead of printing |
| `--theme Name` | Picks one of several `theme` declarations |
| `--token NAME=VALUE` | Sets a design token over the theme's; repeat it for more |
| `--lang ar` | Sets the document's language, `<html lang>` (default: the page's `lang:`, else `en`) |
| `--page Name` | Chooses the page to render when the template has several |

The template may also be a directory. Every `.wf` and `.wfx` file under it
renders as one template, with components shared, and `--page Name`
chooses the page. This command renders the `Invoice` page of `templates/`
to a PDF, in Arabic:

```bash
wf render templates/ --page Invoice --lang ar --data invoice.json --format pdf -o invoice.pdf
```

A template that names a component nothing declares, or a flag a component
does not take, is refused, as a build would refuse it.

## Render from Rust

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

## Render from Node

The Node binding is a small wrapper around `wf render`. Install it with
npm:

```bash
npm install webfluent
```

The binding needs `wf` itself installed. It looks for it at `WF_BIN`, then
on `PATH`, then in `~/.webfluent/bin` and `~/.cargo/bin` (and, outside
Windows, `/usr/local/bin` and `/usr/bin`). Its version is the compiler's.

This Express app renders an invoice as HTML and as a PDF:

```js
const { Template } = require("webfluent");

const templates = Template.fromDir("templates");          // or fromFile, fromString
const invoice = templates.page("Invoice");

app.get("/invoices/:id", async (req, res) => {
  const data = await loadInvoice(req.params.id);
  res.send(await invoice.renderHtmlAsync(data));
});
app.get("/invoices/:id.pdf", async (req, res) => {
  res.type("application/pdf").send(await invoice.renderPdfAsync(await loadInvoice(req.params.id)));
});
```

Every render has a synchronous form (`renderHtml`, `renderHtmlFragment`,
`renderPdf`, `renderSlides`) and an `…Async` form that does not block the
event loop. Use the `…Async` forms in a server. `withTheme`, `withTokens`
and `withLang` set what the Rust API sets.

A render runs `wf` once and takes a few milliseconds. The data goes over
standard input, so nothing is written to disk.

## What a template may use

A template uses the static subset of the language, because there is no
browser to run the rest:

| Supported | Not supported |
|---|---|
| Every layout, typography and data-display element | `effect` |
| `for` loops, `if`/`else`, string interpolation | Handlers (`on click`) |
| `format` and `ago` | Navigation |
| `style { }` blocks, flags, themes | Stores |
| `type`s, `const`s, `data` files and components | Animations |
| `state` and `derived` values (see below) | `resource` |

A `state` is its first value, and a `derived` value is what that works out
to. So a page may work out its totals once, in order, and show them
anywhere. A value the data hands over wins over either.

A template may declare `type`s and a `theme` like any file. `data` files
are read relative to the template. `format` and `ago` speak the data's
`locale` key when it has one, and English otherwise.

A template may read private `env` names ([Environments](environments.md)):
it runs on your server, where a secret stays secret.

## Pick an output for the job

| Job | Output |
|---|---|
| An email | `--format html`; inline the styles with your mailer if it needs them |
| Invoices and reports | `--format pdf`, one per record, from a job queue |
| A piece of an existing page — an HTMX response, a CMS block | `--format html-fragment` |

## Troubleshooting

| What you see | What to do |
|---|---|
| The render fails with "no page called …" | Pass one of the page names the message lists to `page(name)` or `--page` |
| A render refuses a component or a flag | Declare the component in the template's files, or use a flag the component takes — the same checks as `wf build` |
| `WebFluent CLI (wf) not found` from Node | Install `wf`, or set `WF_BIN` to the binary's path |
| A `{name}` renders empty | The data has no top-level key `name`; check the JSON's spelling and nesting |
| `format` writes the wrong language | Add a `locale` key to the data |
| The render fails with `E120` or `E121` | The template nests too deeply, or the render does too much; see [Limits of one render](#limits-of-one-render) |

## Next

[JavaScript interop](javascript-interop.md): write plain JavaScript beside your pages and call it by name; load a library from a CDN; hand an element to a script; place custom elements; publish your components.
