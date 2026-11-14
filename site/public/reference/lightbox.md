# Lightbox

> Every picture in its block opens, on a press or Enter, full screen: the arrow keys and a swipe go between them, a double press or + zooms, Escape closes and the focus goes back. A picture's data-full: is the file it opens, else its largest size.

The page: https://webfluent.monzeromer.dev/reference/lightbox

Every picture in its block opens, on a press or Enter, full screen: the arrow keys and a swipe go between them, a double press or + zooms, Escape closes and the focus goes back. A picture's data-full: is the file it opens, else its largest size.

```wf
Lightbox(label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | What the pictures are, for a screen reader (unset, `Pictures`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Lightbox(label: "Trip") {
    Image(src: "/favicon.svg", alt: "The logo")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
