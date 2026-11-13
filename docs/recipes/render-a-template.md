# Render a template with data

> Render a .wf template with JSON data to HTML, an HTML fragment, a PDF or slides with wf render, for invoices, emails and server responses.

The page: https://webfluent.monzeromer.dev/recipes/render-a-template

A template is a `.wf` file with a `page`, rendered once with the data you
hand it: an invoice, an email, a block of a page your server already
draws. `wf render` turns it into HTML, a fragment, a PDF or slides. You
need the `wf` command installed
([Getting started](https://webfluent.monzeromer.dev/learn/01-getting-started.md#install)).

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

The data is a JSON object, saved here as `invoice.json` beside the
template, `invoice.wf`:

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

This command renders the template with the data and prints the markup to
standard output:

```bash
wf render invoice.wf --data invoice.json --format html-fragment
```

Rendered, the page reads `Invoice #INV-001`, `Bill to: Acme Corp`, one table
row (`Widget`, `5`, `$49.95`), the badge `Due` and `Total: $49.95`.

## How it works

The data's top-level keys are in scope by name, so the template reads
`number`, `customer` and `items` without declaring them. A template uses
the static subset of the language: no handlers, stores, `resource` or
motion (see [What a template may use](#what-a-template-may-use)).

`wf render` takes these options:

| Option | What it does |
|---|---|
| `--data FILE` | The JSON data. When it is omitted, the data is read from standard input |
| `--format`, `-f` | `html` (a whole document with its CSS; the default), `html-fragment` (the body only), `pdf` or `slides` |
| `-o FILE`, `--output` | Writes to a file instead of printing |
| `--theme Name` | Picks one of several `theme` declarations |
| `--token NAME=VALUE` | Sets a design token over the theme's; repeat it for more |
| `--lang ar` | Sets the document's language, `<html lang>` (default: the page's `lang:`, else `en`) |
| `--page Name` | Chooses the page to render when the template has several |

A template that names a component nothing declares, or a flag a component
does not take, is refused, as a build would refuse it.

## Variations

The template may also be a directory. Every `.wf` and `.wfx` file under it
renders as one template, with components shared, and `--page Name`
chooses the page. This command renders the `Invoice` page of `templates/`
to a PDF, in Arabic:

```bash
wf render templates/ --page Invoice --lang ar --data invoice.json --format pdf -o invoice.pdf
```

To render from a program instead of the command line, use the Rust crate
([Render templates from Rust](https://webfluent.monzeromer.dev/recipes/render-from-rust.md)) or the Node binding
([Render templates from Node](https://webfluent.monzeromer.dev/recipes/render-from-node.md)).

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

A template may read private `env` names
([Environments and secrets](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md)): it runs on your
server, where a secret stays secret.

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
| The render fails with `E120` or `E121` | The template nests too deeply, or the render does too much; see [Limits of one render](https://webfluent.monzeromer.dev/recipes/render-from-rust.md#limits-of-one-render) |
