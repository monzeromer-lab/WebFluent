# Paged output — the plan

> Approved 2026-10-04, against 5.1.1. Decisions are recorded at the end.
> Author: Monzer Omer · Date: 2026-10-04

A PDF written in WebFluent should be able to look like anything a designer
draws: a cover with a full-bleed gradient, a letterhead, three figures side
by side, a table that runs for nine pages with its header on every one,
Arabic set right to left in its own typeface, a chart, a QR code. Today it
can draw a flowing column of headings, paragraphs and simple tables, and
nothing else.

---

## What was measured

Three documents were built and every page rasterised: the `wf init -t pdf`
template, and a probe using side-by-side layout, components, data,
conditions, styles, tokens, non-Latin text and a 60-row table — through
`wf build` and through `wf render`.

**Wrong output.**
- `{page}` / `{pages}` in a `Header` or `Footer` — the documented way to
  number pages — is `T13` since 5.0: a splice is an expression.
- `wf build` hands the writer the raw program; `wf render` resolves it
  first. Through the build an `if` always draws its first branch and never
  its `else`, a `for` over data draws nothing (over a literal it prints
  `{n}`), a component draws nothing, a `const` or `data` value never arrives.
- The first page has no header: it starts before `Header` is read.
- Leftovers of the WebFluent 2 spelling are dead code: `justify: .between`
  never matches (it looks for a bare word), `.sm`/`.lg` on text are ignored
  (it looks for `small`/`large`), `font-size: 20px` is ignored (the text path
  matches a bare number). `$tokens`, themes, `.italic`, `.underline` do nothing.
- The linter is a denylist: `Markdown`, `Icon`, `Avatar`, `Unsafe.Html` and
  others compile and draw nothing.
- Through the template engine every image is a grey box (no asset root); a
  transparent PNG loses its transparency.
- The page total is a search-and-replace of `###` across every stream:
  body text containing `###` is overwritten, and 999 pages is the limit.
- `Link` is refused, though a PDF has links.

**Missing.** Anything beyond WinAnsi prints `?` (Arabic, CJK, Polish, ✓); no
font can be embedded; no metadata, outline, compression, tagging or PDF/A;
table headers do not repeat and every column is the same width. The writer
has no unit tests, and the deck writer is a second copy of it (1,550 lines).

**Why complex designs cannot be built.** The writer is a print head: each
element draws itself at a moving baseline and moves it down. Nothing is
measured before it is drawn, so `Row` and `Grid` stack, a box's background
cannot fit its content, text cannot be inline, and nothing can be layered.

---

## The engine

One pipeline, `src/paged/`, for `wf build`, `wf render`, the Rust and Node
template APIs, and the slide deck:

```
static HTML + CSS ──► DOM + cascade ──► box tree ──► layout ──► paginate ──► paint ──► write
```

1. **Render.** The template engine renders the program to the static HTML
   and CSS it already produces for `wf render` — for the build too: every
   component expanded, `if`/`for`/`match` decided, `const` and `data` read,
   `style { }` blocks compiled to rules, the theme's tokens as custom
   properties. A PDF is that page, printed: what the web draws, the PDF
   draws, by construction rather than by a second mapping of every element.
2. **Cascade and box tree.** The HTML is parsed into a DOM and styled by
   the same sheets the web build ships — the engine's structural rules, the
   component rules when the theme includes them, the project's own `.css`,
   `@media print` — through a CSS cascade (selectors with descendant,
   child and sibling combinators, attribute selectors, specificity,
   inheritance, `var()`, `em`/`rem`/`%`). Rules that only mean something on a
   screen (`:hover`, screen-size media queries) are skipped. Each element
   becomes a box; runs of inline content become inline formatting contexts
   whose inline-blocks (a badge in a sentence) sit on the line.
3. **Layout.** [`taffy`] lays out block, flex and grid (and absolute
   positioning) on a strip as wide as the page's content and as tall as it
   needs; text leaves are measured by the text engine.
4. **Paginate.** The strip is cut into pages. Break opportunities lie
   between the children of a breakable box, between the lines of a
   paragraph, between table rows and grid rows. An unbreakable unit that
   would straddle a page moves to the next, and everything after it moves
   with it. Repeating table headers, `break-inside: avoid`,
   `break-before/after: page`, a heading kept with what follows, two-line
   orphans and widows. A box that spans pages paints a slice on each.
5. **Paint.** A display list: filled and stroked shapes, glyph runs,
   images, vector graphics, clips, opacity, links, destinations.
6. **Write.** [`krilla`] — the PDF library Typst's export is built on:
   compressed streams, embedded font subsets with `ToUnicode` maps so text
   copies and searches, images with their alpha, JPEGs passed through,
   gradients, opacity, clipping, metadata, an outline from the headings,
   link annotations, and (later) tagged PDF and PDF/A.

A slide is a fixed page that does not flow: the same tree, layout and
paint, clipped at its edge with a warning. `codegen/pdf.rs` and
`codegen/slides.rs` go.

### Text

- **Fonts.** Liberation Sans, Serif and Mono (metric twins of Helvetica,
  Times and Courier, SIL OFL) ship inside `wf` as the zero-config default,
  for every generic and standard name — so Latin, Greek and Cyrillic work
  with nothing installed, and a document looks the same on every machine. A
  `.ttf`/`.otf` under the project's `fonts/` (or listed in `pdf.fonts`) is
  registered under its family name, so `font-family: Manrope` in a theme or
  style just works. The system's fonts are the last resort for a character
  no named font has. Generic names map to the standard fonts.
- **Shaping and direction.** [`rustybuzz`] shapes every embedded run
  (ligatures, Arabic joining, marks); [`unicode-bidi`] orders mixed
  directions; [`unicode-linebreak`] finds break opportunities.
- **Fallback.** A run is split where the chosen font lacks a glyph and
  continued in one that has it. A character no font has is a finding at the
  `Text` that holds it, never a silent `?`.

### What a design can say

Everything is the language as it is: elements, flags, props, CSS in
`style { }`. The CSS subset the engine honours:

| Area | Properties |
|---|---|
| Box | `width` `height` `min-*` `max-*` `padding*` `margin*` `gap` `flex` `flex-grow` `flex-shrink` `flex-basis` `align-self` |
| Decoration | `background` (colour, linear and radial gradients with any stops) `border*` (per side; solid, dashed, dotted) `border-radius` (per corner) `box-shadow` `opacity` `overflow: hidden` |
| Text | `color` `font-family` `font-size` `font-weight` `font-style` `line-height` `letter-spacing` `text-align` (incl. `justify`) `text-transform` `text-decoration` `white-space` `direction` |
| Position | `position: absolute` with `top` `right` `bottom` `left`; `transform: rotate()` |
| Paging | `break-before` `break-after` `break-inside` |

Units: `pt`, `px` (¾ pt, as CSS), `mm`, `cm`, `in`, `em`, `%`. A property
the engine does not honour is a warning at the property, in the editor.

New and changed elements:

| Element | |
|---|---|
| `Document(size:, landscape, margin:, title:, author:, subject:, keywords:, lang:)` | The paper, the margins, the metadata |
| `Header(on:)` · `Footer(on:)` | `.all` (default), `.first`, `.rest`, `.odd`, `.even`; `page` and `pages` are names inside them |
| `Background(on:)` | Drawn behind each page's content, page-sized: a cover, a band, a full-bleed image |
| `Watermark("DRAFT")` | Large, rotated, faint text behind each page |
| `Table(widths:, repeatHeader:)` | Columns sized by content or as given; the header repeats (default); rows split across pages |
| `Link` | An external link, or `to: "#id"` to an element's `id:` |
| `Image` | Real size and ratio, `fit:`, SVG drawn as vectors |
| `Icon` | The built-in icons, as vectors |
| `Markdown` | Rich text |
| `Chart(kind:, data:, …)` | Bar, line, area, pie, donut — vector |
| `QrCode(value)` | A QR code, vector |
| `TableOfContents` | The headings with the page each lands on |

The linter becomes an allowlist read from the registry: an element either
draws in a paged output or is `E109`.

---

## Phases

1. **One pipeline.** The resolver for the build; `page`/`pages`; the
   allowlist; the new crate dependencies; the `src/paged/` skeleton —
   style, box tree, text with the standard fonts, taffy layout, a basic
   paginator, the writer. The old writer is replaced, not patched.
2. **Text.** Embedded fonts, subsetting, shaping, bidi, fallback, inline
   runs, justification, the findings.
3. **Pages.** Header/footer/background variants, watermarks, repeating
   table headers and split rows, breaks, keep-with-next, orphans and
   widows, links and destinations, outline, metadata, table of contents.
4. **Graphics.** Gradients, per-side borders, shadows, opacity, clipping,
   rotation, images with alpha, SVG, icons, charts, QR codes.
5. **Slides** on the engine; the old writers deleted.
6. **Reference documents** — the acceptance tests:
   - *Halyard* — a quarterly platform report in the Halyard design: the
     Fluant Ink tokens, Manrope / Fraunces / JetBrains Mono, a dark cover,
     region grid, latency charts, a deployments table across pages, a
     config listing, callouts, table of contents.
   - *Invoice* — letterhead, two addresses side by side, line items across
     pages, a totals box that never splits, a QR code, a PAID watermark.
   - *Résumé (Arabic)* — right to left, Arabic shaping, a sidebar with its
     own background, icons, a Latin and CJK line.
   Each renders through `wf build` and `wf render`, is held to a layout
   snapshot (every box's page and position as text) and checked by eye.
7. **Documentation**, the editor (registry, hover, completion for the new
   props), release.

---

## Decisions

- **Native engine on focused pure-Rust crates** (`taffy`, `krilla`,
  `krilla-svg`, `ttf-parser`, `rustybuzz`, `unicode-bidi`,
  `unicode-linebreak`, `fontdb`, `qrcode`) — over compiling to Typst (a
  second layout model) or printing with Chrome (a browser on every server).
  No C, one layout model for the web and the page.
- **The PDF is the template engine's HTML and CSS, laid out by the engine**
  — not a second mapping of every element to drawing code. Parity with the
  web comes from sharing the source of truth.
- **`px` is CSS's** (¾ pt), so a style written for the web is the same size
  on paper. A bare number is points, as before.
- **The default fonts are bundled** (Liberation, ~2 MB compressed), not the
  non-embedded standard 14: krilla embeds every font, PDF/A requires it, and
  a bundled family makes non-ASCII Latin work everywhere.

[`taffy`]: https://crates.io/crates/taffy
[`krilla`]: https://crates.io/crates/krilla
[`rustybuzz`]: https://crates.io/crates/rustybuzz
[`unicode-bidi`]: https://crates.io/crates/unicode-bidi
[`unicode-linebreak`]: https://crates.io/crates/unicode-linebreak
