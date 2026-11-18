# Carousel

> Slide track with named controls and dots; optional autoplay.

The page: https://webfluent.monzeromer.dev/reference/carousel

Slide track with named controls and dots; optional autoplay.

```wf
Carousel(label: String, autoplay: Bool, interval: Number) { … }
```

## Example

```wf
Carousel(label: "Featured work", autoplay: true, interval: 6000) {
    Carousel.Slide {
        Image(src: "/og.png", alt: "The sharing card")
    }
    Carousel.Slide {
        Image(src: "/og.png", alt: "The sharing card, its title in close-up") {
            style { object-fit: cover; object-position: 15% 50%; aspect-ratio: 1200 / 630 }
        }
    }
    Carousel.Slide {
        Center {
            style { aspect-ratio: 1200 / 630; background: $color-surface }
            Image(src: "/icon-512.png", alt: "The app icon", width: 160, height: 160)
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The region's accessible name |
| `autoplay:` | `Bool` | Advance by itself every `interval:`, with a pause button; it waits while hovered, focused or in a hidden tab, and never runs under reduced motion; `.autoplay`, or a value that reads state. Written as a flag, `.autoplay`, or `autoplay: true`. |
| `interval:` | `Number` | Milliseconds between slides while it plays, 1 to 2147483647; under 1, 5000 |

## Flags

`.autoplay`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Carousel`.

### Carousel.Slide

One slide of a carousel.

```wf
Carousel.Slide { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [T01](https://webfluent.monzeromer.dev/errors/T01.md): A value of the wrong type
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
