# Carousel.Slide

> One slide of a carousel.

The page: https://webfluent.monzeromer.dev/reference/carousel-slide

One slide of a carousel.

```wf
Carousel.Slide { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Carousel(label: "Recent work", autoplay: true, interval: 5000) {
    Carousel.Slide { Image(src: "/one.jpg", alt: "A bridge at dusk") }
    Carousel.Slide { Image(src: "/two.jpg", alt: "The harbour") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
