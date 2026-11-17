# AspectRatio

> A box that keeps its shape as its width changes — ratio: 16 / 9 — its child filling it: a picture, a video, a frame.

The page: https://webfluent.monzeromer.dev/reference/aspectratio

A box that keeps its shape as its width changes — ratio: 16 / 9 — its child filling it: a picture, a video, a frame.

```wf
AspectRatio(ratio: Number) { … }
```

## Example

```wf
Grid(columns: 2, gap: .md) {
    AspectRatio(ratio: 16 / 9) {
        Image(src: "/og.png", alt: "The WebFluent card, in 16:9")
    }
    AspectRatio(ratio: 1) {
        Image(src: "/icon-512.png", alt: "The WebFluent icon, square")
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `ratio:` | `Number  (required)` | Its width over its height, `16 / 9` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
