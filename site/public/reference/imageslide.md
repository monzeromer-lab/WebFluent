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
| `src:` | `String` | The picture's path, centred on the slide; a slide with none draws an empty image |
| `caption:` | `String` | A line under the picture, which is also its `alt` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
