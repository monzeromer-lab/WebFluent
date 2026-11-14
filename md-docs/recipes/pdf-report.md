---
kind: "recipe"
route: "/recipes/pdf-report"
group: "content"
description: "Build a project into a PDF report: a page wrapped in Document, with a running footer, a chart drawn from data and a table that breaks across pages."
outputs: ["pdf"]
---
# Make a PDF report

A project can build to a PDF instead of a site: the same pages, laid out on
paper by the compiler, with no browser involved. This report shows a
quarter's sales as a chart, a table and a total, with a page number in
every footer:

**Needs:** `"output_type": "pdf"` in `webfluent.app.json`, as below.
`wf init q1 -t pdf` makes a project with it; replace its
`src/pages/Report.wf` with the page.

```json
{
  "name": "Q1 report",
  "build": {
    "output_type": "pdf",
    "pdf": { "page_size": "A4", "output_filename": "q1-report.pdf" }
  }
}
```

```wf
const MONTHS = [
    { month: "Jan", web: 41, store: 22 },
    { month: "Feb", web: 48, store: 25 },
    { month: "Mar", web: 55, store: 24 },
]

page Report(path: "/", title: "Q1 report", description: "Sales for the first quarter.") {
    Document(size: "A4", title: "Q1 report", author: "Acme Inc.") {
        Header(on: .rest) { Text("Acme Inc. · Q1 report").muted.sm.right }
        Footer { Text("Page {page} of {pages}").muted.sm.center }

        Heading("Q1 report").h1
        Text("Web sales grew every month; the shop held steady.")

        Heading("Sales by month").h2
        Chart(kind: .bar, data: MONTHS, x: "month", y: ["web", "store"], unit: "k", legend: true)

        Table(caption: "Sales by month, in thousands") {
            Table.Head {
                Table.Row { Table.Cell("Month")  Table.Cell("Web")  Table.Cell("Shop") }
            }
            Table.Body {
                for m in MONTHS {
                    Table.Row { Table.Cell(m.month)  Table.Cell("{m.web}k")  Table.Cell("{m.store}k") }
                }
            }
        }

        Card {
            style { break-inside: avoid }
            Text("Total: {MONTHS.map(m => m.web + m.store).sum()}k").bold
        }
    }
}
```

`wf build` writes `build/q1-report.pdf`: a page with the heading, the bar
chart, the table and the total, and "Page 1 of 1" at the foot.

## How it works

- `"output_type": "pdf"` makes `wf build` write one PDF instead of a site.
  Every page of the project is laid out in turn; a report is usually one.
- `Document` is the root of the paper: its size, and the file's title and
  author, which a PDF reader shows in the document's properties.
- `Footer` is drawn in the bottom margin of every page, and inside it
  `page` and `pages` are the page's number and the count. `Header(on: .rest)`
  is drawn on every page but the first, which already has the title.
- `Chart` draws the list of records as vector graphics: `x:` names the
  field along the bottom, `y:` the fields drawn as bars, one colour each.
- A `Table` that runs past the foot of a page breaks between rows, and its
  header row is drawn again at the top of the next page.
- `break-inside: avoid` keeps the total's `Card` on one page.
- The data is a `const` here. A JSON file read at build time
  (`data months = "months.json"`) works the same way, so the report can be
  built again whenever the numbers change.

The look comes from the same CSS a web page gets: a theme, a `style { }`
block and the project's `.css` files style the PDF too.

## Variations

- **A cover page**: put the title in a `Background(on: .first) { … }` with a
  gradient, then `PageBreak` before the content.
- **A table of contents**: `TableOfContents(levels: 2)` lists the headings
  with the page each lands on, each a link.
- **A draft mark**: `Watermark("DRAFT")` draws large, faint text behind
  every page.
- **One PDF per customer**: render a template with each customer's data
  instead of building the project
  ([Render a template with data](render-a-template.md)).
- **A slide deck**: `"output_type": "slides"` and a `Presentation { Slide { … } }`
  page make one PDF page per slide.

See [PDF and slides](../learn/17-pdf-and-slides.md) for everything a
document can hold.
