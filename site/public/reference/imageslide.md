# ImageSlide

> Image slide with an optional caption.

The page: https://webfluent.monzeromer.dev/reference/imageslide

Image slide with an optional caption.

```wf
ImageSlide(src: String, caption: String)
```

## Example

```wf
Presentation {
    ImageSlide(src: "/og.png", caption: "The card every shared link shows")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Image path (required) |
| `caption:` | `String` | Caption under the image |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
