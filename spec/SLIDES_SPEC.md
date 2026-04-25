# WebFluent Slides — Specification

PDF slide-deck output. One `Slide` = exactly one PDF page; no flow pagination across slides.

## Output target

Set in `webfluent.app.json`:

```json
{
  "build": {
    "output_type": "slides",
    "slides": { "size": "16:9" }
  }
}
```

Sibling of `"spa"`, `"static"`, and `"pdf"`. Produces a single `.pdf` file in the build output directory.

## Source structure

A slide deck must live inside a `Page` body, wrapped in a `Presentation { ... }` block:

```
Page Deck (path: "/", title: "My Deck") {
    Presentation {
        TitleSlide("Welcome", "Subtitle here")
        Slide { Heading("Topic", h2)  Text("Body") }
        TwoColumn { Container { ... } Container { ... } }
        ImageSlide(src: "logo.png", caption: "Fig 1")
        SectionSlide("Wrap-up", primary)
    }
}
```

`Presentation` may only contain slide elements. Slide elements may not appear outside a `Presentation` (validation error).

## Slide kinds

| Kind | Signature | Renders |
|------|-----------|---------|
| `Slide { ... }` | freeform body | top-aligned content with full margin, like a page |
| `TitleSlide(title, subtitle?)` | two positional strings | both lines vertical-centered, title 56pt bold, subtitle 28pt grey |
| `SectionSlide(label, [color])` | label + optional color modifier | full-bleed colored background, white label centered, 48pt bold |
| `TwoColumn { Container {..} Container {..} }` | exactly two `Container` children | content split 50/50 with a 24pt gutter |
| `ImageSlide(src: String, caption?: String)` | required `src`, optional `caption` | image placeholder filling 85%×90% of content area, optional caption below |

`SectionSlide` color modifiers: `primary`, `success`, `danger`, `warning`, `info`. Defaults to `primary`.

`ImageSlide` currently renders a placeholder rectangle with `[Image]` label — same as the `Image` component in PDF mode. Real image embedding is a v2 feature.

## Page sizing

`slides.size` accepts:
- `"16:9"` → 960×540pt (default)
- `"4:3"` → 720×540pt
- `"A4-landscape"` → 841.89×595.28pt
- `"WIDTHxHEIGHT"` (e.g. `"800x600"`) → explicit dimensions in points

Or override with `slides.width` and `slides.height` (in points), which take precedence over `size`.

## Slide chrome (page numbers + footer)

Both opt-in via config:

```json
{
  "slides": {
    "show_slide_numbers": true,
    "footer_text": "Confidential — Q4 2026"
  }
}
```

- `show_slide_numbers`: emits `n / total` in the bottom-right of each slide.
- `footer_text`: emits the text in the bottom-left of each slide.

Both render in 11pt grey, baselined at `margin × 0.4` from the bottom edge.

## Body components

Inside a freeform `Slide` (or inside `TwoColumn` containers), the following components are rendered:

- `Text`, `Heading` (h1–h6) — auto-scaled ~2× larger than the PDF defaults
- `List` (with `ordered` modifier for numbered lists) — bullet `•` marker
- `Container`, `Stack`, `Column`, `Grid`, `Section` — recurse into children
- `Spacer` — vertical gap; modifiers: `small`/`sm`, `medium`/`md`, `large`/`lg`, `xl`
- `Divider` — thin grey horizontal line
- `if`/`for` — control flow (only static iteration over `ListLiteral`)

Any component not in this list recursively emits its children (graceful degradation).

## Overflow handling

If a slide's content runs past the bottom margin, **rendering is clipped** at that point and a stderr warning is emitted:

```
warning[slides]: slide 3 content overflows; truncated
```

The build does not fail. Auto-fit / scale-to-fit is a v2 feature.

## Rejected components

The slides validator (`slides_validation.rs`) rejects:

1. **All interactive components** — same list PDF rejects (`Button`, `Input`, `Select`, `Checkbox`, `Radio`, `Switch`, `Slider`, `DatePicker`, `FileUpload`, `Form`, `Dropdown`, `Modal`, `Dialog`, `Toast`, `Spinner`, `Skeleton`, `Tooltip`, etc.) plus navigation (`Router`, `Route`, `Navbar`, `Sidebar`, `Menu`, `Tabs`, `TabPage`, `Breadcrumb`, `Link`) and media (`Video`, `Carousel`).
2. **PDF document components** (`Document`, `Paragraph`, `PageBreak`, `Header`, `Footer`) — these belong to the PDF document model, not slides. Use `slides.footer_text` config for slide footers.
3. **Slide nesting** — slide elements may not be nested inside other slide elements.
4. **Slide elements outside a Presentation** — must be wrapped.
5. **Event handlers** (`on:click`, etc.) — not supported.
6. **`navigate`, `fetch`, `animate`** statements — web-only.

Validation runs before codegen; any error fails the build.

## Library API

Available via the `Template` API:

```rust
use webfluent::Template;
use serde_json::json;

let tpl = Template::from_file("deck.wf")?;
let pdf_bytes: Vec<u8> = tpl.render_slides(&json!({}))?;
```

Uses `SlidesConfig::default()` (16:9, no chrome). For custom config, use the CLI or instantiate `SlidesCodegen` directly.

## CLI

```bash
wf init my-deck --template slides   # scaffold a starter deck
wf build                             # compile to PDF in ./build/
```

## Limitations (v1)

- Image embedding is a placeholder (mirrors PDF backend).
- Speaker notes are not supported.
- HTML/reveal.js-style web slides are out of scope.
- Per-slide page sizes are not supported (whole deck shares one size).
- Text scaling/auto-fit on overflow is not supported.
