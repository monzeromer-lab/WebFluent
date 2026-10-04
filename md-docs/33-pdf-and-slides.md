# 33. PDF and slides

<!--
route: guide/pdf-and-slides
group: shipping
blurb: The same language, compiled to a paginated PDF document or a slide deck — invoices, reports and talks, laid out like the web.
description: PDF documents and slide decks — the paged engine, CSS layout, fonts, Arabic, headers, page numbers, contents, charts and QR codes.
-->

The same language compiles to paper. `build.output_type` in
`webfluent.app.json` chooses what `wf build` writes:

| Output | `output_type` | Writes |
|---|---|---|
| A website | `spa` (default), with `build.ssg` for static pages | HTML, CSS and JavaScript ([chapter 26](26-static-and-spa.md)) |
| PDF document | `pdf` | One `.pdf`, paginated |
| Slide deck | `slides` | One `.pdf`, a slide per page |
| Custom elements | `elements` | `elements.js` for other frameworks ([chapter 32](32-javascript-interop.md#publishing-your-components-as-custom-elements)) |

To produce one PDF per invoice from data, use
[server rendering](34-server-rendering.md) with the same templates.

## The engine

A PDF is the page, printed. The compiler renders the program to the same
static HTML and CSS a template renders, then lays that out on paper
itself — no browser, no external tool:

- **The page's own stylesheet.** The engine's rules for every built-in, the
  theme's tokens, every `.css` file under `src/` and every `style { }`
  block, through a real CSS cascade: selectors with descendant, child and
  sibling combinators, attribute selectors, `:first-child`/`:nth-child`,
  `:not()`, `:is()` and `:where()`, specificity, inheritance, `var()`,
  `em`, `rem` and `%`. A `Card` in a
  PDF looks like the `Card` on the page, and a class of your own
  (`class: "callout"`) brings its rules with it. `@media print` applies;
  rules only a screen has (`:hover`, a media query on width) do not.
- **Layout.** Block, flex and grid as CSS defines them — `Row`, `Stack`,
  `Grid` and `Column` side by side exactly as on the web — with absolute
  positioning, `gap`, `min-`/`max-` sizes, `aspect-ratio`, inline-blocks on
  a line of text (a badge in a sentence), tables sized by their content,
  and list markers.
- **Text.** Every run shaped (ligatures, kerning, Arabic joining, combining
  marks), set right to left where the language is, broken at Unicode's
  line-break opportunities, aligned or justified. A word too long for its
  line is cut rather than overflowing.
- **Paint.** Backgrounds (colours, linear and radial gradients with any
  number of stops, pictures), borders per side (solid, dashed, dotted) with
  rounded corners, shadows, opacity, `overflow: hidden`, `transform:
  rotate()`, and pictures with their transparency — PNG, JPEG (embedded
  as it is), WebP, GIF and SVG, which stays vector.
- **The file.** Fonts embedded as subsets with the text attached, so a
  reader copies and searches it; compressed; with its title and author;
  an outline of its headings in the reader's sidebar; links that work.

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

A PDF page holds a `Document`; everything inside it flows from page to
page:

```wf
type Line { item: String, qty: Number, price: Number }

const LINES: [Line] = [
    Line(item: "Design", qty: 12, price: 90),
    Line(item: "Build", qty: 40, price: 110),
]

page Invoice(path: "/", title: "Invoice", description: "An invoice.") {
    derived total = LINES.reduce((sum, l) => sum + l.qty * l.price, 0)
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

### Pages

- **`Document(size:, landscape, margin:, title:, author:, subject:,
  keywords:, lang:)`** — the paper (`A4`, `A3`, `A5`, `Letter`, `Legal`,
  `Tabloid`, or two lengths: `"210mm 297mm"`), its margins (one length or
  four, `"20mm 25mm"`), and what the file says about itself. Without them,
  `build.pdf` decides.
- **`Header`** and **`Footer`** are drawn in the top and bottom margins of
  every page, or of those `on:` names — `.all` (the default), `.first`,
  `.rest`, `.odd`, `.even`. Inside them, `page` and `pages` are the page's
  number and the count: `Text("Page {page} of {pages}")`.
- **`Background`** is drawn behind each page's content, the size of the
  whole page, margins and all: a cover's full-bleed gradient, a coloured
  band, a sidebar strip that runs down every page. It takes `on:` too.
- **`Watermark("DRAFT")`** — large, faint, rotated text behind each page;
  its look is the rule `.wf-watermark`, which a stylesheet may change.
- **`PageBreak`** starts a new page; so do `break-before: page` and
  `break-after: page` in a style.

The page's background — the `html` or `body` rule's — fills every page,
edge to edge, so a dark theme prints dark.

### Breaking across pages

Content flows onto as many pages as it needs, and never through the middle
of a line of text:

- Paragraphs break between lines, keeping at least two on each side
  (`orphans`, `widows`).
- Tables break between rows, and **the header row is drawn again** at the
  top of every page the table continues on.
- A grid breaks between its rows; a row of cards moves to the next page
  together.
- Side-by-side columns taller than a page break each on its own, and what
  follows waits for the longer.
- A heading never ends a page: it moves with what follows.
- `break-inside: avoid` keeps a box whole (a totals block, a signature).
- A box that runs onto the next page is painted on both, its border open
  at the break.

### Tables of contents, links and the outline

`TableOfContents(levels: 2)` lists the document's headings with the page
each one lands on, dotted leaders between them, each entry a link to its
heading. The engine lays the document out, learns where every heading
landed, and fills the numbers in. `title:` puts a heading above it.

A `Link` is a link: to a URL, or `to: "#terms"` to the element whose `id:`
is `terms`. Every heading appears in the reader's outline, nested by
level.

### Charts and QR codes

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

`Chart` draws `.bar` (grouped, or `stacked`), `.line`, `.area`, `.pie` and
`.donut` over a list of records, as vector graphics: `x` names each row's
label, `y` its value or a list of values (one series each); `colors`,
`ink` (labels), `grid`, `legend`, `labels` (values on bars and slices),
`unit`, and `width`/`height` in user units — it scales to its box, so
draw a narrow chart narrower for larger text. `QrCode(value, color:,
background:)` encodes a URL or any text. Both work on a web page too, as an
SVG drawn when the page is built — there, what they read must be known at
build time (a literal, a `const`, a `data` file).

### Fonts and languages

The default faces are Liberation Sans, Serif and Mono — the metric twins of
Helvetica, Times and Courier — built into `wf`, so `Helvetica`, `Arial`,
`system-ui`, `sans-serif`, `Times`, `serif`, `Courier` and `monospace` all
resolve without anything installed, with Latin, Greek and Cyrillic.

Any other family is a file: put `.ttf` or `.otf` files under the project's
`fonts/` (or `src/fonts/`, or name them in `pdf.fonts`), and the theme's
`font-family: Manrope, sans-serif` uses them by the family name inside the
file. Variable fonts are drawn at the weight a style asks for; a weight or
an italic a family does not have is synthesised. A font a stylesheet's
`@font-face` names, from a local file, is used too.

A character no named face has is drawn from one that has it — the
project's fonts first, then the machine's. The build says which families
came from the machine, so a document can be made the same everywhere by
copying them into `fonts/`; `"system_fonts": false` forbids it, and a
character no font has is reported, never drawn as `?`.

A document whose `lang` is right to left (`ar`, `he`, `fa`, `ur`) is laid
out right to left — text, flex rows, grids, table columns, list markers —
and a run of the other direction inside it is ordered by the Unicode
bidirectional algorithm.

### What a PDF does not draw

Everything static draws. What only means something in a browser is refused
where it is written (`E109`): controls (`Button`, `Input`, `Form`, …),
navigation (`Router`, `Navbar`, `Tabs`, `Menu`), overlays (`Modal`,
`Dialog`, `Toast`, `Tooltip`), media that plays (`Video`, `Audio`,
`Carousel`), elements a script draws (`Host`, `Element`), handlers,
`resource`, motion.

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
            style { background: linear-gradient(135deg, #1E3A8A, #0F172A); color: #fff }
            Heading("Thank you").h1
        }
    }
}
```

- A deck is laid out by the same engine, a slide to a page: a slide is a
  box the size of the page, its `slides.margin` its padding (which a rule
  of the deck's own overrides), and what runs past its edge is clipped,
  with a note naming the slide.
- A `Presentation` holds slides, a `for` or an `if` over them, and
  components whose body is a slide: a deck writes its section openers,
  headings and footers once, and makes a slide per item of its data. A
  page's `derived` values are worked out before it is drawn, so a slide can
  show a figure computed from a `data` file.
- Kinds: `Slide { }` freeform (a column); `TitleSlide(title, subtitle:)`
  centred; `SectionSlide(label).primary/.success/.danger/.warning/.info`
  full-bleed; `TwoColumn { … }` two equal columns; `ImageSlide(src:,
  caption:)`. Each is styled by the engine's sheet, and a project's rules or
  a `style { }` change it as anywhere.
- `slides.size`: `16:9` (default, 960×540pt), `4:3` (720×540), `A4-landscape`
  (841.89×595.28), or `WIDTHxHEIGHT` in points; `slides.width` and
  `slides.height` override it outright.
- `show_slide_numbers`, `footer_text` and `chrome_color` add the chrome (the
  colour flips between dark and light on the slide's background when
  unset). `default_font`, `default_font_size` (24pt — every size in the
  engine's slide rules is relative to it) and `output_filename` are the
  deck's own.
- Everything a document draws, a slide draws: layout, charts, pictures,
  tables, Arabic.

`wf init my-deck --template slides` scaffolds one.

## Next

[Server rendering](34-server-rendering.md).
