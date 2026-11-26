## PDF Output

With `output_type: "pdf"` the build lays the program out on paper with its own engine and writes a PDF.

```json
{
    "build": {
        "output_type": "pdf",
        "pdf": {
            "page_size": "A4",
            "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
            "default_font": "Helvetica",
            "default_font_size": 12,
            "output_filename": "report.pdf",
            "fonts": [],
            "system_fonts": true
        }
    }
}
```

A PDF is the page, printed — by the compiler's own paged engine, no
browser and no external tool. The program is rendered to the static HTML
and CSS a template renders, and that is laid out on paper:

- **The cascade.** The engine's rules for every built-in, the theme's
  tokens, every `.css` file under `src/` and every `style { }` block:
  descendant, child and sibling combinators, attribute selectors,
  `:first-child`/`:nth-child`, `:not()`/`:is()`/`:where()`, specificity,
  inheritance, `var()`, `em`,
  `rem`, `%`, `::before`/`::after` content, CSS counters (`counter-reset`,
  `counter-increment`, `counter(name)`) — so `List.ordered` is numbered. A `Card` in a PDF looks like the
  `Card` on the page; `class: "callout"` brings its rules.
- **Layout.** Block, flex and grid as CSS defines them — `Row`, `Stack`,
  `Grid`, `Column` side by side as on the web — absolute positioning, `gap`,
  `min-`/`max-` sizes, `aspect-ratio`, inline-blocks on a line (a badge in a
  sentence), tables sized by their content, list markers. `px` is 0.75pt;
  `100vh` is the page's height.
- **Text.** Shaped (ligatures, kerning, Arabic joining), bidirectional,
  broken at Unicode's line-break opportunities, aligned or justified.
- **Paint.** Colours, linear and radial gradients with any number of stops,
  background pictures, borders per side (solid, dashed, dotted) with rounded
  corners, shadows, opacity, `overflow: hidden`, `transform: rotate()`;
  PNG, JPEG (embedded as it is), WebP, GIF, and SVG kept vector.
- **The file.** Fonts embedded as subsets with the text attached (copy and
  search work), compressed, with title and author, an outline of the
  headings, and links that work.

### Document elements

| Element | Purpose |
|---|---|
| `Document(size:, landscape, margin:, title:, author:, subject:, keywords:, lang:)` | The root: paper (`A4`, `A3`, `A5`, `Letter`, `Legal`, `Tabloid`, or `"210mm 297mm"`), margins (one length or four), and the file's metadata; `build.pdf` decides what it omits |
| `Header(on:)` · `Footer(on:)` | Drawn in the top/bottom margin of every page, or `on: .first`, `.rest`, `.odd`, `.even` (`.all` is the default) |
| `Background(on:)` | Behind each page's content, the whole page edge to edge: a cover's gradient, a band, a sidebar strip |
| `Watermark("DRAFT")` | Large, faint, rotated text behind each page; styled by `.wf-watermark` |
| `TableOfContents(levels: 2, title:)` | The headings with the page each lands on, dotted leaders, each entry a link |
| `PageBreak` | A new page; so do `break-before: page` / `break-after: page` |
| `Section`, `Paragraph` | Grouping, and a paragraph's spacing |
| `Chart(kind:, data:, x:, y:)` | `.bar` (grouped, or `stacked`), `.line`, `.area`, `.pie`, `.donut` over a list of records, as vector graphics; `colors`, `ink`, `grid`, `legend`, `labels`, `unit`, `width`/`height` |
| `QrCode(value, color:, background:)` | A QR code of a URL or any text |

Inside a `Header`, `Footer` or `Background`, `page` and `pages` are the
page's number and the count: `Text("Page {page} of {pages}")`. The `html`
or `body` background fills every page edge to edge, so a dark theme prints
dark. `Chart` and `QrCode` draw on a web page too, as an SVG made at build
time — there what they read must be known then (a literal, a `const`, a
`data` file), or it is `E109`.

### Page breaks

- Paragraphs break between lines, two at least on each side (`orphans`,
  `widows`).
- Tables break between rows, and the header row is drawn again on every
  page the table continues on.
- A grid breaks between its rows; side-by-side columns taller than a page
  break each on its own.
- A heading never ends a page: it moves with what follows.
- `break-inside: avoid` keeps a box whole (a totals block, a signature),
  and a table's `Table.Body` its rows together.
- A box that runs onto the next page is painted on both, its border open
  at the break.

### Fonts and languages

The default faces are Liberation Sans, Serif and Mono, built into `wf`, so
`Helvetica`, `Arial`, `system-ui`, `sans-serif`, `Times`, `serif`, `Courier`
and `monospace` resolve with nothing installed. Any other family is a file:
`.ttf`/`.otf` under `fonts/`, `src/fonts/`, `public/fonts/` or `pdf.fonts`,
named by the family inside it (variable fonts drawn at the asked weight; a
missing bold or italic synthesised), or a local `@font-face`. A character no
named face has comes from one that has it — the project's fonts, then the
machine's, which the build names in a `note:` so the document can be made
reproducible by copying them into `fonts/`. `system_fonts: false` forbids
the machine's; a character no font has is reported, never drawn as `?`.

A document whose `lang` is `ar`, `he`, `fa` or `ur` is laid out right to
left — text, flex rows, grids, table columns, list markers — with runs of
the other direction ordered by the Unicode bidirectional algorithm.

### PDF example

```wf
const SALES = [
    { month: "Jan", web: 41, store: 22 },
    { month: "Feb", web: 48, store: 25 },
]

page Report(path: "/", title: "Report", description: "The quarter.") {
    Document(size: "A4", title: "Q1 report", author: "Company Inc.") {
        Background(on: .first) {
            Stack { style { height: 100%; background: linear-gradient(160deg, #0B1220, #1E3A8A) } }
        }
        Header(on: .rest) { Text("Company Inc. · Q1").muted.sm.right }
        Footer { Text("Page {page} of {pages}").muted.sm.center }
        Watermark("DRAFT")

        Heading("Q1 Report").h1
        TableOfContents(levels: 2)
        PageBreak

        Heading("Sales").h2
        Row(gap: .lg) {
            Chart(kind: .bar, data: SALES, x: "month", y: ["web", "store"], unit: "k")
            Chart(kind: .donut, data: SALES, x: "month", y: "web", labels: true)
        }
        Table {
            Table.Head { Table.Row { Table.Cell("Region")  Table.Cell("Revenue") } }
            Table.Body { Table.Row { Table.Cell("North America")  Table.Cell("$2.4M") } }
        }
        Card {
            style { break-inside: avoid }
            Text("Revenue grew 15% this quarter.").bold
        }
        QrCode("https://example.com/q1")
    }
}
```

**Refused in a PDF or a deck** (`E109`, where it is written): controls
(`Button`, `Input`, `Select`, `Checkbox`, `Switch`, `Form`, …), navigation
(`Router`, `Navbar`, `Sidebar`, `Tabs`, `Menu`), overlays (`Modal`, `Dialog`,
`AlertDialog`, `Sheet`, `Toast`, `Tooltip`), media that plays (`Video`, `Audio`, `Carousel`),
elements a script draws (`Host`, `Element`, `Portal`), handlers, `resource`, motion.
Everything static draws.

