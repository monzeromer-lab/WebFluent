# Slide

> One slide, one PDF page; content is top-aligned and clipped at the bottom margin.

The page: https://webfluent.monzeromer.dev/reference/slide

One slide, one PDF page; content is top-aligned and clipped at the bottom margin.

```wf
Slide { … }
```

## Flags

`.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Presentation {
    Slide {
        Heading("Revenue").h1
        Text("Up 18% on last quarter.")
    }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
