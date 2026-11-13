# Presentation

> Root of a slide deck; its children must be slides.

The page: https://webfluent.monzeromer.dev/reference/presentation

Root of a slide deck; its children must be slides.

```wf
Presentation { … }
```

## Flags

`.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Presentation {
    TitleSlide("Q1 review", subtitle: "March 2026")
    Slide { Heading("Highlights").h1 }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
