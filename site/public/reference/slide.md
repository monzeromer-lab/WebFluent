# Slide

> One slide, one PDF page; content is top-aligned and clipped at the bottom margin.

The page: https://webfluent.monzeromer.dev/reference/slide

One slide, one PDF page; content is top-aligned and clipped at the bottom margin.

```wf
Slide { … }
```

## Example

```wf
Presentation {
    Slide {
        style { background: linear-gradient(135deg, #1E3A8A, #0F172A); color: #ffffff }
        Heading("Revenue").h1
        Text("Up 15% on last quarter, to €2.4M.").lg
        Text("Source: finance, unaudited").sm
    }
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
