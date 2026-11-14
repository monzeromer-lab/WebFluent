# Carousel

> Slide track with named controls and dots; optional autoplay.

The page: https://webfluent.monzeromer.dev/reference/carousel

Slide track with named controls and dots; optional autoplay.

```wf
Carousel(label: String, autoplay: Bool, interval: Number) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Accessible name of the region |
| `autoplay:` | `Bool` | Advance automatically |
| `interval:` | `Number` | Milliseconds between slides |

## Flags

`.autoplay`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Carousel.Carousel.Slide`

## Example

```wf
Carousel(label: "Recent work", autoplay: true, interval: 5000) {
    Carousel.Slide { Image(src: "/one.jpg", alt: "A bridge at dusk") }
    Carousel.Slide { Image(src: "/two.jpg", alt: "The harbour") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [T01](https://webfluent.monzeromer.dev/errors/T01.md): A value of the wrong type
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
