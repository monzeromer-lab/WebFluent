# 33. PDF and slides

<!--
route: guide/pdf-and-slides
group: shipping
blurb: The same language, compiled to a paginated PDF document or a slide deck — invoices, reports and talks.
description: PDF documents and slide decks: configuration, the document and slide elements, what renders, styling, and templates for them.
-->

The same language compiles to paper. `build.output_type` in
`webfluent.app.json` chooses what `wf build` writes:

| Output | `output_type` | Writes |
|---|---|---|
| A website | `spa` (default), with `build.ssg` for static pages | HTML, CSS and JavaScript ([chapter 26](26-static-and-spa.md)) |
| PDF document | `pdf` | One `.pdf`, paginated |
| Slide deck | `slides` | One `.pdf`, a slide per page |
| Custom elements | `elements` | `elements.js` for other frameworks ([chapter 32](32-javascript-interop.md#publishing-your-components-as-custom-elements)) |

The PDF is written by the compiler itself — PDF 1.7, the standard PDF fonts,
no browser and no external tool — so a document builds anywhere `wf` runs.
To produce one PDF per invoice from data, use
[server rendering](34-server-rendering.md) with the same templates.

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

## Next

[Server rendering](34-server-rendering.md).
