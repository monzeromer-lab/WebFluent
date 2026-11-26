## Slides Output (PDF Slide Decks)

With `output_type: "slides"` the build writes a PDF slide deck, one `Slide` per page.

```json
{
    "build": {
        "output_type": "slides",
        "slides": {
            "size": "16:9",
            "default_font": "Helvetica",
            "default_font_size": 24,
            "margin": 60,
            "show_slide_numbers": true,
            "footer_text": "My Deck",
            "background_color": "#1A1A19",
            "chrome_color": null,
            "output_filename": "deck.pdf",
            "fonts": [],
            "system_fonts": true
        }
    }
}
```

The same engine, **one `Slide` = one page**: a slide is a box the size of
the page with `slides.margin` as its padding (a deck's own rule overrides
it). A deck is a `Presentation { }` inside a page; slide elements outside
one are a compile error. A `Presentation` holds slides, a `for` or an `if`
over them, and components whose body is a slide — so a deck writes its
openers and footers once and makes a slide per item of its data:

```wf
component Point(_ label: String, n: Number) {
    Slide { Heading(label).h1  Text("{n} of 3").muted }
}
page D(path: "/", title: "Deck") {
    Presentation {
        TitleSlide("Three points")
        for p, i in ["Fast", "Small", "Typed"] { Point(p, n: i + 1) }
    }
}
```

| Element | Purpose |
|---|---|
| `Presentation { ... }` | The deck — its children are slides |
| `Slide { ... }` | Freeform, a column from the top |
| `TitleSlide("Title", subtitle: "…")` | Cover, centred |
| `SectionSlide("Label").primary` | Full-bleed band; `.primary`, `.success`, `.danger`, `.warning`, `.info` |
| `TwoColumn { Container { … } Container { … } }` | Two equal columns |
| `ImageSlide(src: "…", caption: "…")` | A picture with an optional caption |

Each is styled by the engine's sheet (every size relative to
`default_font_size`), and a project's rules or a `style { }` change it as
anywhere: `Slide { style { background: linear-gradient(135deg, #1E3A8A,
#0F172A); color: #fff } }`. Everything a document draws, a slide draws —
layout, charts, QR codes, pictures, tables, Arabic.

- `slides.size`: `"16:9"` (960×540pt, default), `"4:3"` (720×540),
  `"A4-landscape"` (841.89×595.28), or `"WIDTHxHEIGHT"` in points;
  `slides.width` + `slides.height` override it.
- `show_slide_numbers` (`n / total`, bottom right) and `footer_text` (bottom
  left) are the chrome; `chrome_color` colours it, flipping between dark and
  light on the slide's background when `null`. `background_color` paints
  every slide.
- What runs past a slide's edge is **clipped**, with a note naming the
  slide; the build does not fail.

```wf
page Deck(path: "/", title: "Q1 Review", description: "The quarter.") {
    Presentation {
        TitleSlide("Q1 Review", subtitle: "Company Inc. — March 2026")
        SectionSlide("Numbers").primary
        Slide {
            Heading("Highlights").h1
            List {
                List.Item { Text("Launched 3 new products") }
                List.Item { Text("Revenue grew 15%") }
            }
        }
        TwoColumn {
            Container { Heading("Wins").h3  Text("New enterprise deals") }
            Container { Heading("Risks").h3  Text("Supply chain delays") }
        }
        ImageSlide(src: "chart.png", caption: "Q1 revenue by region")
    }
}
```

