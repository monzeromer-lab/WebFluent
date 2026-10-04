# Slide decks

Three decks, each a WebFluent project whose `build.output_type` is
`slides`. `wf build -d examples/decks/<name>` writes the PDF to its
`build/`; the fonts are the shared ones in `examples/documents/fonts`.

| Deck | Slides | What it shows |
|---|---|---|
| [`webfluent`](webfluent) | 16 | WebFluent itself, in the docs site's design system: code consoles, a grammar table, a measured bar chart, thumbnails of the reference PDFs, QR codes |
| [`halyard-review`](halyard-review) | 16 | Halyard's quarterly review from the report's data: every number worked out from `review.json`, charts, tables with status pills, one slide per incident from a `for` |
| [`arabic-workshop`](arabic-workshop) | 8 | A workshop in Arabic, laid out right to left: mirrored rows and grids, a type scale, translations, a chart with Arabic labels |

A slide is a component like any other: each deck writes its section
openers, headings and footers once and places them on every slide, and a
`Presentation` may hold a `for` or an `if` over slides.
