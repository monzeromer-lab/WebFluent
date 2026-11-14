# AspectRatio

> A box that keeps its shape as its width changes — ratio: 16 / 9 — its child filling it: a picture, a video, a frame.

The page: https://webfluent.monzeromer.dev/reference/aspectratio

A box that keeps its shape as its width changes — ratio: 16 / 9 — its child filling it: a picture, a video, a frame.

```wf
AspectRatio(ratio: Any) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `ratio:` | `Any  (required)` | Its width over its height, `16 / 9` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
AspectRatio(ratio: 16 / 9) {
    Image(src: "/favicon.svg", alt: "The logo, wide")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
