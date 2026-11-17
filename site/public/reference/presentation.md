# Presentation

> Root of a slide deck; its children must be slides.

The page: https://webfluent.monzeromer.dev/reference/presentation

Root of a slide deck; its children must be slides.

```wf
Presentation { … }
```

## Example

```wf
Presentation {
    TitleSlide("Q3 review", subtitle: "Northwind Ltd · October 2026")
    SectionSlide("The numbers").primary
    Slide {
        Heading("Highlights").h1
        List {
            List.Item { Text("Revenue up 15%") }
            List.Item { Text("Three new enterprise customers") }
        }
    }
    TwoColumn {
        Container { Heading("Wins").h2  Text("Self-serve plan launched") }
        Container { Heading("Risks").h2  Text("Hiring behind plan") }
    }
    ImageSlide(src: "/og.png", caption: "The launch card")
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
