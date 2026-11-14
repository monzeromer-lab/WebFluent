---
chapter: 17
kind: "tutorial"
route: "/learn/17-pdf-and-slides"
from: ["/docs/guide/pdf-and-slides"]
group: "build"
blurb: "Compile the same language to a paginated PDF document or a slide deck — invoices, reports and talks, laid out with CSS as on the web."
description: "Make PDF documents and slide decks: the paged engine, CSS layout, page breaks, fonts, Arabic, headers, page numbers, contents, charts and QR codes."
learned: ["Set `build.output_type` to write a PDF document or a slide deck", "Lay out paper with `Document`, `Header`, `Footer`, `PageBreak` and `break-inside: avoid`", "Number pages with `page` and `pages`, and add a `TableOfContents`, charts and QR codes", "Use your own fonts, and lay out a right-to-left document", "Build a deck from `Presentation` and its slide kinds"]
---
# 17. PDF and slides

This chapter is about compiling a WebFluent project to paper: a paginated
PDF document, such as an invoice or a report, or a slide deck for a talk. You write the pages with the same elements and the same CSS as a
website, and `wf build` lays them out on pages.

In this chapter:

- choosing what the build writes;
- how the paged engine lays out a page;
- making a PDF document: paper, headers, page breaks, contents, charts,
  fonts and right-to-left text;
- making a slide deck.

## Before you start

You need a WebFluent project. To start one already set up for paper, run
one of these commands:

```bash
wf init my-report --template pdf
wf init my-deck --template slides
```

Then run `wf build` in the project's folder. The build prints the page
count and writes one file, `build/<name>.pdf`, named after the project
unless `output_filename` says otherwise:

```text
  PDF: 45586 bytes, 3 page(s)
  Output: ./build/my-report.pdf
Build complete.
```

To produce one PDF per invoice from data, use the same templates with
[server rendering](../recipes/render-a-template.md) instead.

## Choose what the build writes

`build.output_type` in `webfluent.app.json` chooses what `wf build` writes:

| Output | `output_type` | Writes |
|---|---|---|
| A website | `spa` (default), with `build.ssg` for pre-rendered pages; or `static`, which is the same as `ssg` | HTML, CSS and JavaScript ([Static sites and SPAs](../about/static-and-spa.md)) |
| PDF document | `pdf` | One `.pdf`, paginated |
| Slide deck | `slides` | One `.pdf`, one slide per page |
| Custom elements | `elements` | `elements.js` for other frameworks ([JavaScript interop](../recipes/publish-custom-elements.md)) |

## The paged engine

A PDF is the page, printed. The compiler renders the program to the same
static HTML and CSS a template renders, then lays that HTML out on paper
itself, with no browser and no external tool. The engine covers five
areas:

- **The page's own stylesheet.** The engine's rules for every built-in, the
  theme's tokens, every `.css` file under `src/` and every `style { }`
  block go through a real CSS cascade. It supports selectors with
  descendant, child and sibling combinators, attribute selectors,
  `:first-child`/`:nth-child`, `:not()`, `:is()` and `:where()`;
  specificity; inheritance; `var()`; and `em`, `rem` and `%`. A `Card` in a
  PDF looks like the `Card` on the page, and a class of your own
  (`class: "callout"`) brings its rules with it. `@media print` applies;
  rules only a screen has (`:hover`, a media query on width) do not.
- **Layout.** Block, flex and grid work as CSS defines them — `Row`,
  `Stack`, `Grid` and `Column` sit side by side exactly as on the web —
  with absolute positioning, `gap`, `min-`/`max-` sizes, `aspect-ratio`,
  inline-blocks on a line of text (a badge in a sentence), tables sized by
  their content, and list markers.
- **Text.** Every run is shaped (ligatures, kerning, Arabic joining,
  combining marks), set right to left where the language is, broken at
  Unicode's line-break opportunities, and aligned or justified. A word too
  long for its line is cut rather than overflowing.
- **Paint.** Backgrounds (colours, linear and radial gradients with any
  number of stops, pictures), borders per side (solid, dashed, dotted) with
  rounded corners, shadows, opacity, `overflow: hidden`, `transform:
  rotate()`, and pictures with their transparency — PNG, JPEG (embedded
  as it is), WebP, GIF, and SVG, which stays vector.
- **The file.** Fonts are embedded as subsets with the text attached, so a
  reader can copy and search it. The file is compressed, carries its title
  and author, has an outline of its headings in the reader's sidebar, and
  its links work.

## Make a PDF document

To build a PDF, set `output_type` to `pdf`. The `pdf` key sets the paper,
margins (in points) and default font for every document in the project:

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

A PDF page holds a `Document`, and everything inside it flows from page to
page. This invoice has a header on every page but the first, a page number
in the footer, a watermark, a table, and a totals card that is never split:

```wf
type Line { item: String, qty: Number, price: Number }

const LINES: [Line] = [
    Line(item: "Design", qty: 12, price: 90),
    Line(item: "Build", qty: 40, price: 110),
]

page Invoice(path: "/", title: "Invoice", description: "An invoice.") {
    derived total = LINES.map(l => l.qty * l.price).sum()
    Document(size: "A4", title: "Invoice 2026-014", author: "Acme Studio") {
        Header(on: .rest) { Text("Acme Studio · invoice 2026-014").muted.sm.right }
        Footer { Text("Page {page} of {pages}").muted.sm.center }
        Watermark("PAID")

        Row(justify: .between, align: .start) {
            Heading("Invoice").h1
            Stack(gap: .xs) {
                Text("No. 2026-014").bold
                Text("Issued {format("2026-09-01", .date, "long")}").muted
            }
        }
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
        Card {
            style { break-inside: avoid; margin-top: 12pt }
            Text("Total: {format(total, .currency)}").bold.lg
        }
        PageBreak
        Heading("Terms").h2
        Paragraph { Text("Payment within 30 days.") }
    }
}
```

The result is a two-page PDF: the invoice on the first page, ending with
`Total: $5,480.00`, and the terms on the second, under the header.

> [!NOTE]
> **Coming from LaTeX or Typst?** There is no document class and there are
> no TeX boxes: a document is laid out by CSS. `\newpage` is `PageBreak`,
> a `minipage` that must not split is `style { break-inside: avoid }`, and
> `fancyhdr` is a `Header` and a `Footer`. See
> [Coming from LaTeX and Typst](../coming-from/latex.md).

### Set the paper, headers and footers

These elements shape every page of a document:

| Element | What it does |
|---|---|
| `Document(size:, landscape, margin:, title:, author:, subject:, keywords:, lang:)` | The paper (`A4`, `A3`, `A5`, `Letter`, `Legal`, `Tabloid`, or two lengths: `"210mm 297mm"`), its margins (one length or four, `"20mm 25mm"`), and what the file says about itself. What it leaves out, `build.pdf` decides |
| `Header`, `Footer` | Drawn in the top and bottom margins of every page, or of the pages `on:` names: `.all` (the default), `.first`, `.rest`, `.odd`, `.even` |
| `Background` | Drawn behind each page's content, the size of the whole page, margins and all: a cover's full-bleed gradient, a coloured band, a sidebar strip that runs down every page. It takes `on:` too |
| `Watermark("DRAFT")` | Large, faint, rotated text behind each page. Its look is the rule `.wf-watermark`, which a stylesheet may change |
| `PageBreak` | Starts a new page; so do `break-before: page` and `break-after: page` in a style |

`Header` and `Footer` are the same built-ins that draw the `<header>` and
`<footer>` landmarks on a web page ([Elements](06-elements.md#layout-elements)).
Inside a `Document` they become the margins of every page.

Inside a `Header`, `Footer` or `Background`, `page` and `pages` are the
page's number and the page count: `Text("Page {page} of {pages}")`.

The page's background — the `html` or `body` rule's — fills every page,
edge to edge, so a dark theme prints dark.

### Control where pages break

Content flows onto as many pages as it needs, and never breaks through the
middle of a line of text. The engine follows these rules:

- Paragraphs break between lines, keeping at least two on each side
  (`orphans`, `widows`).
- Tables break between rows, and **the header row is drawn again** at the
  top of every page the table continues on.
- A grid breaks between its rows; a row of cards moves to the next page
  together.
- Side-by-side columns taller than a page break each on its own, and what
  follows waits for the longer column.
- A heading never ends a page: it moves with what follows.
- A box that runs onto the next page is painted on both, its border open
  at the break.

To keep a box whole — a totals block, a signature — give it
`break-inside: avoid`, as the invoice's totals card does. To start a new
page, place `PageBreak`.

### Add a table of contents, links and an outline

`TableOfContents(levels: 2)` lists the document's headings with the page
each one lands on, dotted leaders between them, and each entry a link to
its heading. The engine lays the document out, learns where every heading
landed, and fills in the numbers. `title:` puts a heading above the list.

A `Link` is a link in the PDF too: to a URL, or `to: "#terms"` to the
element whose `id:` is `terms`. Every heading appears in the reader's
outline, nested by level.

### Draw charts and QR codes

This report draws a grouped bar chart, a donut chart and a QR code from a
constant list of records:

```wf
const SALES = [
    { month: "Jan", web: 41, store: 22 },
    { month: "Feb", web: 48, store: 25 },
    { month: "Mar", web: 55, store: 21 },
]

page Report(path: "/", title: "Sales", description: "The quarter.") {
    Document {
        Heading("Sales by channel").h1
        Chart(kind: .bar, data: SALES, x: "month", y: ["web", "store"], unit: "k")
        Chart(kind: .donut, data: SALES, x: "month", y: "web", labels: true)
        QrCode("https://example.com/report/q1")
    }
}
```

`Chart` draws a list of records as vector graphics. It takes these props:

| Prop | What it sets |
|---|---|
| `kind` | `.bar` (grouped, or `stacked`), `.line`, `.area`, `.pie` or `.donut` |
| `x` | The field that names each row's label |
| `y` | The field that holds each row's value, or a list of fields (one series each) |
| `colors`, `ink` | The series' colours, and the colour of the labels |
| `grid`, `legend` | Whether to draw grid lines and a legend |
| `labels` | Values written on bars and slices |
| `unit` | A unit after each value |
| `width`, `height` | The size in user units. The chart scales to its box, so draw a narrow chart narrower for larger text |

`QrCode(value, color:, background:)` encodes a URL or any text.

Both work on a web page too, as an SVG drawn when the page is built. There,
what they read must be known at build time — a literal, a `const` or a
`data` file — or the build reports `E109`.

### Use your own fonts

The default faces are Liberation Sans, Serif and Mono — the metric twins of
Helvetica, Times and Courier — built into `wf`. So `Helvetica`, `Arial`,
`system-ui`, `sans-serif`, `Times`, `serif`, `Courier` and `monospace` all
resolve with nothing installed, and cover Latin, Greek and Cyrillic.

To use any other family, add its file:

1. Put `.ttf` or `.otf` files under the project's `fonts/` folder (or
   `src/fonts/`, or name them in `pdf.fonts`).
2. Name the family as the file names it, for example in the theme:
   `font-family: Manrope, sans-serif`.

Variable fonts are drawn at the weight a style asks for. A weight or an
italic a family does not have is synthesised. A font a stylesheet's
`@font-face` names, from a local file, is used too.

A character no named face has is drawn from a face that has it — the
project's fonts first, then the machine's. The build names, in a `note:`
line, each family it took from the machine, so you can make the document
the same everywhere by copying those fonts into `fonts/`. To forbid the
machine's fonts, set `"system_fonts": false`. A character no font has is
reported, never drawn as `?`.

### Lay out a right-to-left document

A document whose `lang` is right to left (`ar`, `he`, `fa`, `ur`) is laid
out right to left — text, flex rows, grids, table columns, list markers.
A run of the other direction inside it is ordered by the Unicode
bidirectional algorithm.

A code or a figure written in Latin characters inside Arabic text — an
invoice number like `2026-0417`, a product code — can come out with its
parts in the wrong order. The bidirectional algorithm reads the hyphen as
part of the right-to-left text around it. To fix it, give the code a run of
its own that says it is left to right: `Text("2026-0417", class: "ltr")`,
with `.ltr { direction: ltr; unicode-bidi: isolate; }` in the project's
CSS. The fix works on a web page and in a PDF alike.

### What a PDF does not draw

Everything static draws. What only means something in a browser is refused
where it is written, with `E109`:

- controls (`Button`, `Input`, `Form`, …);
- navigation (`Router`, `Navbar`, `Tabs`, `Menu`);
- overlays (`Modal`, `Dialog`, `Toast`, `Tooltip`);
- media that plays (`Video`, `Audio`, `Carousel`);
- elements a script draws (`Host`, `Element`, `Portal`);
- handlers, `resource` and motion.

## Make a slide deck

To build a deck, set `output_type` to `slides`. The `slides` key sets the
deck's size and its chrome:

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

A deck is a `Presentation` inside a page. This deck uses every kind of
slide, and restyles the last one with a gradient:

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
        ImageSlide(src: "/chart.png", caption: "Monthly recurring revenue")
        Slide {
            style { background: linear-gradient(135deg, #1E3A8A, #0F172A); color: #fff }
            Heading("Thank you").h1
        }
    }
}
```

The `ImageSlide` reads `public/chart.png`: put a picture of your own there
first. The build warns (`D01`) when the file is missing. The result is a
six-page PDF, one slide per page.

### Write the slides

A deck is laid out by the same engine, one slide to a page. A slide is a
box the size of the page, with `slides.margin` as its padding (a rule of
the deck's own overrides it). What runs past a slide's edge is clipped,
with a note naming the slide.

The slide kinds:

| Element | What it draws |
|---|---|
| `Slide { … }` | A freeform slide, laid out as a column from the top |
| `TitleSlide(title, subtitle:)` | A cover, centred |
| `SectionSlide(label)` | A full-bleed band; `.primary`, `.success`, `.danger`, `.warning` or `.info` |
| `TwoColumn { … }` | Two equal columns |
| `ImageSlide(src:, caption:)` | A picture with an optional caption |

Each kind is styled by the engine's sheet, and a project's rules or a
`style { }` block change it as anywhere. Everything a document draws, a
slide draws: layout, charts, pictures, tables, Arabic.

A `Presentation` holds slides, a `for` or an `if` over them, and components
whose body is a slide. So a deck writes its section openers, headings and
footers once, and makes a slide per item of its data. A page's `derived`
values are worked out before the deck is drawn, so a slide can show a
figure computed from a `data` file.

### Size and frame the deck

These keys under `build.slides` set the deck's size and chrome:

| Key | What it sets |
|---|---|
| `size` | `16:9` (default, 960×540pt), `4:3` (720×540), `A4-landscape` (841.89×595.28), or `WIDTHxHEIGHT` in points |
| `width`, `height` | Override `size` outright |
| `show_slide_numbers` | Draws `n / total` in the bottom right |
| `footer_text` | Text in the bottom left |
| `chrome_color` | The colour of the numbers and footer; when unset, it flips between dark and light on each slide's background |
| `background_color` | Paints every slide |
| `default_font`, `default_font_size` | The deck's font, and its size (24pt). Every size in the engine's slide rules is relative to it |
| `output_filename` | The file the build writes |

## Troubleshooting

| What you see | What to do |
|---|---|
| `E109` on a `Button`, a handler or a `resource` | Remove it from the document or deck: a PDF has nothing to run it |
| `E109` on a `Chart` or `QrCode` in a web page | Give it data known at build time: a literal, a `const` or a `data` file |
| A `note:` names a family taken from the machine | Copy that font into `fonts/` so the document is the same on every machine |
| A note says a slide was clipped | Shorten the slide, or split it in two |
| A totals block or signature is split across two pages | Add `break-inside: avoid` to its `style { }` |
| A Latin code in Arabic text shows its parts out of order | Give it a left-to-right run (see [Lay out a right-to-left document](#lay-out-a-right-to-left-document)) |

## Common mistakes

**A page number outside a header or footer.** `page` and `pages` exist only
inside a `Header`, a `Footer` or a `Background`, which are drawn once per
page:

```wf expect=T13
page Report(path: "/", title: "Report", description: "The quarter.") {
    Document(size: "A4", title: "Q1 report") {
        Heading("Q1 report").h1
        Text("Revenue grew 15% this quarter.")
        Text("Page {page} of {pages}").muted.sm.center
    }
}
```

Move the line into `Footer { Text("Page {page} of {pages}") }`
([T13](../errors/T13.md)).

**A component named like a built-in.** `Footer` is already a built-in: the
document's footer, drawn at the bottom of every page. A component you name
`Footer` takes its place, so it is drawn once, where it is called, and no
page gets a footer:

```wf expect=V11
component Footer(_ note: String) {
    Text(note).muted.sm.center
}

page Invoice(path: "/", title: "Invoice", description: "An invoice.") {
    Document(size: "A4", title: "Invoice 2026-014") {
        Heading("Invoice").h1
        Text("Total: $5,480.00").bold
        Footer("Acme Studio · invoice 2026-014")
    }
}
```

For a line on every page, use the built-in:
`Footer { Text("Acme Studio · invoice 2026-014") }`, and name your own
component for what it is, such as `component InvoiceNote`
([V11](../errors/V11.md)).

**A table with no header row.** Without `Table.Head`, a reader of the file
has no column names, and a table that runs onto a second page has no header
to draw again there:

```wf expect=A10
page Invoice(path: "/", title: "Invoice", description: "An invoice.") {
    Document(size: "A4", title: "Invoice 2026-014") {
        Heading("Invoice").h1
        Table {
            Table.Body {
                Table.Row { Table.Cell("Design")  Table.Cell("$1,080.00") }
                Table.Row { Table.Cell("Build")  Table.Cell("$4,400.00") }
            }
        }
    }
}
```

Add `Table.Head { Table.Row { Table.Cell("Item")  Table.Cell("Amount") } }`
([A10](../errors/A10.md)).

## Next

- [Render a template with data](../recipes/render-a-template.md) — render the same templates
  with data, one PDF per record.
- [Styling](13-styling.md) — the CSS a document and a deck are laid out
  with.
