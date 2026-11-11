# PDF documents, slide decks and server rendering

The same language lays out paper. `wf build` with
`"output_type": "pdf"` or `"slides"` renders the program to the static HTML
and CSS a page would get, and lays that out with WebFluent's own paged engine.
There is no browser and no external tool. Fonts are embedded, links work, and
the headings become the outline.

Start one with `wf init report -t pdf` or `wf init deck -t slides`.

## How a good document is written

- **Data in, layout out.**
  - The figures come from a `data` file (`data inv = "invoice.json"`) or a
    `const`, and totals are `derived`. Nothing is typed twice.
  - From a server, the same template renders with JSON through `wf render`
    or the Rust and Node APIs.
- **Look in CSS.** Give elements classes, and keep the rules in a `.css`
  file beside the page. A PDF honours the cascade, flex, grid, `gap`,
  borders, gradients, `var()` and the theme's tokens, so style a document as
  you would a page.
- **Paper's own rules.**
  - `break-inside: avoid` keeps a block whole (totals, a signature).
  - `PageBreak` starts a new page.
  - Headings never end a page.
  - Tables repeat their header row on every page they run onto.
- **Pinned fonts.** Put the `.ttf`/`.otf` files in `fonts/` and set
  `"system_fonts": false`. The build then fails rather than borrow a font from
  the machine, so the file is the same everywhere.
- **Nothing that runs.**
  - A `Button`, `Input`, `Form`, `Modal`, `Tabs`, a handler, a `resource` or
    motion in a document is `E109`.
  - `Chart` and `QrCode` draw as vectors, and on a web page they need data
    known at build time.

## A document

```wf
/// Revenue by month, in thousands.
const SALES = [
    { month: "Jan", web: 41, store: 22 },
    { month: "Feb", web: 48, store: 25 },
    { month: "Mar", web: 53, store: 24 }
]

page Report(path: "/", title: "Q1 report", description: "The first quarter, in figures.") {
    derived total = SALES.map(s => s.web + s.store).sum()

    Document(size: "A4", title: "Q1 report", author: "Company Inc.", lang: "en") {
        Background(on: .first) {
            Stack(class: "cover-band") { }
        }
        Header(on: .rest) { Text("Company Inc. · Q1 report").muted.sm.right }
        Footer { Text("Page {page} of {pages}").muted.sm.center }

        Heading("Q1 report").h1
        Text("Revenue of {total}k across web and stores.").lg
        TableOfContents(levels: 2)
        PageBreak

        Heading("Sales").h2
        Chart(kind: .bar, data: SALES, x: "month", y: ["web", "store"], unit: "k", legend: true)
        Table(caption: "Revenue by month, in thousands") {
            Table.Head {
                Table.Row { Table.Cell("Month")  Table.Cell("Web")  Table.Cell("Stores") }
            }
            Table.Body {
                for s in SALES by s.month {
                    Table.Row { Table.Cell(s.month)  Table.Cell("{s.web}")  Table.Cell("{s.store}") }
                }
            }
        }
        Card(class: "totals") {
            Text("Total: {total}k").bold
        }
        QrCode("https://example.com/q1")
    }
}
```

```css
/* src/report.css */
.cover-band { height: 100%; background: linear-gradient(160deg, #0B1220, #1E3A8A); }
.totals { break-inside: avoid; margin-block-start: var(--spacing-lg); }
```

The configuration:

```json
{
  "build": {
    "output_type": "pdf",
    "pdf": {
      "page_size": "A4",
      "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
      "default_font": "Helvetica",
      "default_font_size": 11,
      "output_filename": "q1-report.pdf",
      "system_fonts": true
    }
  }
}
```

The document's elements:

| Element | What it does |
|---|---|
| `Document(size:, landscape, margin:, title:, author:, lang:)` | The root: paper, margins and metadata |
| `Header(on:)` · `Footer(on:)` | Drawn in the margins, on `.all` pages (default), `.first`, `.rest`, `.odd` or `.even`; `page` and `pages` are in scope inside |
| `Background(on:)` | Behind a page's content, edge to edge |
| `Watermark("DRAFT")` | Large, faint, rotated text |
| `TableOfContents(levels:, title:)` | The headings, with page numbers and links |
| `PageBreak` | A new page |
| `Chart(kind:, data:, x:, y:)` | `.bar`, `.line`, `.area`, `.pie`, `.donut`, as vectors |
| `QrCode(value)` | A QR code |

A document in Arabic (`lang: "ar"`) is laid out right to left, with the text
shaped and joined. Give a Latin code inside it its own LTR run
(`class: "ltr"`).

## A deck

One `Slide` is one page. A deck is a `Presentation` inside a page.
- **Components for repeated slides.** Write the repeated slides as components
  whose body is a slide, and make a slide per item of data with `for`.
- **Overflow.** What runs past a slide's edge is clipped, with a note naming
  the slide. Shorten it or split it.

```wf
/// One point of the talk, numbered.
component Point(_ label: String, n: Number, of: Number) {
    Slide {
        Heading(label).h1
        Text("{n} of {of}").muted
    }
}

page Deck(path: "/deck", title: "Why WebFluent", description: "Three points, one deck.") {
    Presentation {
        TitleSlide("Why WebFluent", subtitle: "Three points")
        SectionSlide("The points").primary
        for p, i in ["Fast", "Small", "Typed"] {
            Point(p, n: i + 1, of: 3)
        }
        TwoColumn {
            Container { Heading("Wins").h3  Text("One language, three outputs.") }
            Container { Heading("Risks").h3  Text("A new language to learn.") }
        }
        Slide {
            style { background: linear-gradient(135deg, #1E3A8A, #0F172A); color: #fff }
            Heading("Thank you").h1
        }
    }
}
```

`build.slides` sets the size (`"16:9"`, `"4:3"`, `"A4-landscape"`,
`"WIDTHxHEIGHT"`), `margin`, `show_slide_numbers`, `footer_text`,
`background_color` and `default_font_size`. Every size in the engine's slide
rules is relative to that last one. `"margin": 0` hands the spacing to your
CSS.

## Server rendering

A template is a `.wf` page whose values come from JSON data at render time:

```bash
wf render invoice.wf --data invoice.json -f pdf -o invoice.pdf
wf render templates/ --page Receipt --lang ar --data data.json
```

- **Libraries.** From Rust, `webfluent::Template::from_dir("templates")?`,
  then `.page("Receipt")?.render_pdf(&data)?`. A `Template` is
  `Send + Sync`: build it once at start-up. From Node, `require('webfluent')`.
- **What a template may use.** Layout, typography, data display, `for`, `if`,
  splices, styles, themes and components all work. `effect`, handlers,
  stores, `resource` and navigation don't: nothing runs after the render.
- **Escaping.** Values from the data are escaped, and a URL from the data
  that a browser would run is dropped.
