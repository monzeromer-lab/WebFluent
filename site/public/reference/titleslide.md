# TitleSlide

> Cover slide: title and subtitle, vertically centred.

The page: https://webfluent.monzeromer.dev/reference/titleslide

Cover slide: title and subtitle, vertically centred.

```wf
TitleSlide(_ title: String, subtitle: String)
```

## Example

```wf
Presentation {
    TitleSlide("Shipping WebFluent 7", subtitle: "Platform team · 17 November 2026")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | The title |
| `subtitle:` | `String` | The subtitle |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
