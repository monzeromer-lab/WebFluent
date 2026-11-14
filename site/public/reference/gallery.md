# Gallery

> Pictures in a grid of columns: (unset, as many as fit), each cropped to one shape — ratio:, unset square. In a Lightbox, each opens full screen.

The page: https://webfluent.monzeromer.dev/reference/gallery

Pictures in a grid of columns: (unset, as many as fit), each cropped to one shape — ratio:, unset square. In a Lightbox, each opens full screen.

```wf
Gallery(columns: Number, ratio: Any) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many across; unset, as many as fit |
| `ratio:` | `Any` | Each picture's shape, `4 / 3` (unset, `1`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Gallery(columns: 2, ratio: 4 / 3) {
    Image(src: "/favicon.svg", alt: "The logo")
    Image(src: "/favicon.svg", alt: "The logo again")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
