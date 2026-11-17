# Gallery

> Pictures in a grid of columns: (unset, as many as fit), each cropped to one shape — ratio:, unset square. In a Lightbox, each opens full screen.

The page: https://webfluent.monzeromer.dev/reference/gallery

Pictures in a grid of columns: (unset, as many as fit), each cropped to one shape — ratio:, unset square. In a Lightbox, each opens full screen.

```wf
Gallery(columns: Number, ratio: Number) { … }
```

## Example

```wf
Gallery(columns: { base: 2, md: 4 }, ratio: 1) {
    Image(src: "/og.png", alt: "The sharing card")
    Image(src: "/icon-512.png", alt: "The app icon")
    Image(src: "/og.png", alt: "The sharing card, its left edge") {
        style { object-position: 0% 50% }
    }
    Image(src: "/og.png", alt: "The sharing card, its right edge") {
        style { object-position: 100% 50% }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many across — or one per breakpoint, `{ base: 2, md: 4 }`; unset, as many as fit |
| `ratio:` | `Number` | Each picture's shape, `4 / 3` (unset, `1`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
