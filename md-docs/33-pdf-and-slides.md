# 17. Outputs

One source, several outputs. `build.output_type` in `webfluent.app.json`
chooses what `wf build` writes, and the same file can also be rendered on a
server with data.

| Output | `output_type` | Writes |
|---|---|---|
| Single-page app | `spa` (default) | `index.html` + `app.js` + `styles.css`: the router paints pages in the browser |
| Static site | `spa` with `build.ssg: true` | One pre-rendered HTML file per route, hydrated by the same script ([chapter 16](16-content.md#static-builds)) |
| PDF document | `pdf` | One `.pdf` |
| Slide deck | `slides` | One `.pdf`, a slide per page |
| Server template | — | HTML or PDF from `wf render` or the library, with JSON data |

## SPA

The default: one `index.html` that loads `app.js`, which mounts the `app`
and routes between pages in the browser. `build.split` (on by default)
writes each page as its own `pages/<Name>.js` and `pages/<Name>.css`,
fetched when its route first shows. A host serves `index.html` for every
path (the "SPA fallback"), or you turn on `ssg` and get real files.

`build.base_path` (`"/my-site"` for a GitHub Pages project site) prefixes
every asset and route; `build.minify`, `build.sourcemap`, `build.compress`
(gzip beside each text file) and `build.csp` (a strict
Content-Security-Policy and a `_headers` file) tune the output.

## PDF documents

```json
{
  "build": {
    "output_type": "pdf",
    "pdf": {
      "page_size": "A4",
      "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
      "default_font": "Helvetica",
      "default_font_size": 12,
      "output_filename": "report.pdf"
    }
  }
}
```

A PDF page holds a `Document` with a body of the document components:

```wf
type Line { item: String, qty: Number, price: Number }

const LINES: [Line] = [
    Line(item: "Design", qty: 12, price: 90),
    Line(item: "Build", qty: 40, price: 110),
]

page Invoice(path: "/", title: "Invoice", description: "An invoice.") {
    derived total = LINES.reduce((sum, l) => sum + l.qty * l.price, 0)
    Document(page_size: "A4") {
        Header { Text("Acme Studio").muted.sm.right }
        Footer { Text("Page footer — thank you for your business").muted.sm.center }
        Section {
            Heading("Invoice #2026-014").h1
            Text("Issued {format("2026-09-01", .date, "long")}")
            Spacer
            Table {
                Table.Head {
                    Table.Row { Table.Cell("Item")  Table.Cell("Qty")  Table.Cell("Price")  Table.Cell("Amount") }
                }
                Table.Body {
                    for l in LINES {
                        Table.Row {
                            Table.Cell(l.item)
                            Table.Cell("{l.qty}")
                            Table.Cell(format(l.price, .currency))
                            Table.Cell(format(l.qty * l.price, .currency))
                        }
                    }
                }
            }
            Divider
            Text("Total: {format(total, .currency)}").bold.lg
        }
        PageBreak
        Section {
            Heading("Terms").h2
            Paragraph { Text("Payment within 30 days.") }
        }
    }
}
```

- `Document(page_size:)` — `A4`, `A3`, `A5`, `Letter`, `Legal` — is the
  root; `Header` and `Footer` repeat on every page; `Section` groups with
  spacing; `Paragraph` is a block of text; `PageBreak` starts a new page.
- Typography, layout containers, `Table`, `List`, `Badge`, `Card`, `Image`,
  `Divider`, `Spacer` render; `if` and `for` work over static data (`const`,
  `data`, literals).
- Interactive components (`Button`, `Input`, `Form`, `Modal`, …),
  navigation, handlers, `resource`, `state` that changes, and motion do not
  belong in a PDF; the PDF validator rejects them before codegen.
- A `style { }` on a container honours a subset: `background` (colour or a
  two-stop `linear-gradient`), `padding`, `border`, `border-radius`,
  `box-shadow`, `width`, `height` in `pt` or `%`. Unknown properties warn
  once per component and property.

`wf init my-report --template pdf` scaffolds one.

## Slide decks

```json
{
  "build": {
    "output_type": "slides",
    "slides": {
      "size": "16:9",
      "show_slide_numbers": true,
      "footer_text": "Acme — Q4 review",
      "background_color": "#FFFFFF"
    }
  }
}
```

```wf
page Deck(path: "/", title: "Q4 review", description: "The quarter in slides.") {
    Presentation {
        TitleSlide("Q4 review", subtitle: "Acme Studio, October 2026")
        SectionSlide("Numbers").primary
        Slide {
            Heading("Revenue").h2
            List {
                List.Item { Text("Up 18% on Q3") }
                List.Item { Text("Three new enterprise accounts") }
                List.Item { Text("Churn at 1.2%") }
            }
        }
        TwoColumn {
            Container {
                Heading("What worked").h3
                Text("Self-serve onboarding.")
            }
            Container {
                Heading("What did not").h3
                Text("The March pricing test.")
            }
        }
        ImageSlide(src: "chart.png", caption: "Monthly recurring revenue")
        Slide {
            style { background: linear-gradient(135deg, #1E3A8A, #0F172A) }
            Heading("Thank you").h1
        }
    }
}
```

- `Presentation` holds only slides; a slide is one PDF page and never flows
  onto the next (content past the bottom margin is clipped, with a warning).
- Kinds: `Slide { }` freeform; `TitleSlide(title, subtitle:)`;
  `SectionSlide(label).primary/.success/.danger/.warning/.info` full-bleed;
  `TwoColumn { Container { } Container { } }`; `ImageSlide(src:, caption:)`.
- `slides.size`: `16:9` (default, 960×540pt), `4:3` (720×540), `A4-landscape`
  (841.89×595.28), or `WIDTHxHEIGHT` in points. `slides.width` and
  `slides.height` override it outright; `slides.margin` (60pt) is the space
  round the content, and what runs past it is clipped.
- `show_slide_numbers`, `footer_text` and `chrome_color` add the chrome
  (the colour auto-flips between dark and light on the slide's background
  when unset). `default_font`, `default_font_size` (24pt) and
  `output_filename` are the deck's own.
- Backgrounds: `slides.background_color` deck-wide, a `Slide`'s `style {
  background }`, a `Container`'s; hex or a two-stop `linear-gradient`.
- Inside a slide: `Text`, `Heading` (scaled up), `List`, layout containers,
  `Spacer`, `Divider`, `if`/`for` over literals. Interactive, navigation
  and media components are rejected.

`wf init my-deck --template slides` scaffolds one.

## Custom elements

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

The project as tags any framework can place, rather than a site. `wf build`
writes `elements.js`, `styles.css` and a page listing what it published;
there are no routes, no shell and no pages, because a component is the
whole of what is published. [Chapter 8](08-components.md#publishing-yours)
covers how an attribute becomes a prop and an event reaches the host page.

## Templates: rendering with data on a server

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
`--theme Name` picks one of several `theme` declarations; the data is read
from stdin when `--data` is omitted.

Rust:

```rust
use webfluent::Template;
use serde_json::json;

let tpl = Template::from_file("templates/invoice.wf")?;
let html = tpl.render_html(&json!({ "number": "INV-001", "items": [], "total": 0, "paid": true }))?;
let pdf: Vec<u8> = tpl.render_pdf(&json!({ "number": "INV-001", "items": [], "total": 0, "paid": true }))?;
let themed = tpl.with_theme("Night").with_tokens(&[("color-primary", "#8B5CF6")]).render_html(&json!({}))?;
```

Node (`npm install @aspect/webfluent`):

```js
const { Template } = require("@aspect/webfluent");
const tpl = Template.fromFile("templates/invoice.wf");
res.send(tpl.renderHtml(invoice));
res.type("application/pdf").send(tpl.renderPdf(invoice));
```

A template may declare `type`s and a `theme` like any file; `data` files
are read relative to the template. `format` and `ago` speak the data's
`locale` when it has one, else English.

## Next

[Tooling](18-tooling.md).
