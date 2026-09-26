# 34. Server rendering

<!--
route: guide/server-rendering
group: shipping
blurb: Render a .wf template with JSON on a server — an email, an invoice, a report, an HTML fragment — from the CLI, Rust or Node.
description: WebFluent as a template engine: wf render, the Rust API, the Node binding, HTML, fragment and PDF output, themes and the data context.
-->

A WebFluent site is static files. When HTML has to be made on a server — an
email, an invoice PDF per order, a fragment for an existing page — the same
language renders a template with JSON data, from the command line, from Rust
or from Node.

## A template and its data

A `.wf` file can be rendered with JSON data — an invoice, an email, a
report — from the CLI or from Rust and Node. The data's top-level keys
are in scope by name; the static subset of the language applies (no
`state`, handlers, stores, `resource`, motion).

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

CLI:

```bash
wf render invoice.wf --data invoice.json --format html-fragment
```

`--format` is `html` (a whole document with its CSS), `html-fragment`
(the body only), `pdf` or `slides`; `-o file` writes instead of printing;
`--theme Name` picks one of several `theme` declarations; `--token
NAME=VALUE` sets a design token over the theme's; the data is read from
stdin when `--data` is omitted. A template that names a component nothing
declares, or a flag a component does not take, is refused, as a build
would refuse it.

Rust:

```rust
use webfluent::Template;
use serde_json::json;

let tpl = Template::from_file("templates/invoice.wf")?;
let html = tpl.render_html(&json!({ "number": "INV-001", "items": [], "total": 0, "paid": true }))?;
let pdf: Vec<u8> = tpl.render_pdf(&json!({ "number": "INV-001", "items": [], "total": 0, "paid": true }))?;
let themed = tpl.with_theme("Night").with_tokens(&[("color-primary", "#8B5CF6")]).render_html(&json!({}))?;
```

Node — `npm install webfluent`, a small wrapper around `wf render`. It
needs `wf` itself installed: it looks on `PATH`, in `~/.webfluent/bin` and
`~/.cargo/bin`, or at `WF_BIN`. Its version is the compiler's.

```js
const { Template } = require("webfluent");
const tpl = Template.fromFile("templates/invoice.wf");
res.send(tpl.renderHtml(invoice));
res.type("application/pdf").send(tpl.renderPdf(invoice));
```

A template may declare `type`s and a `theme` like any file; `data` files
are read relative to the template. `format` and `ago` speak the data's
`locale` when it has one, else English.

## What a template may use

The static subset of the language: every layout, typography and
data-display element, `for` loops, `if`/`else`, string
interpolation, `format` and `ago`, `style { }` blocks, flags, themes,
`type`s, `const`s and components. Not: `state` that changes, `derived`,
`effect`, handlers, navigation, stores, animations, `resource` — there is no
browser to run them in.

A template may read private `env` names ([Environments](30-environments.md)):
it runs on your server, where a secret stays secret.

## Uses

- **Email**: render with `--format html`; inline the styles with your mailer
  if it needs them.
- **Invoices and reports**: `--format pdf`, one per record, from a job queue.
- **Fragments**: `--format html-fragment` for a piece of an existing page —
  an HTMX response, a CMS block.

## Next

[Cookbook](35-recipes.md).
