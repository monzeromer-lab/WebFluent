# Lightbox

> Every picture in its block opens, on a press or Enter, full screen: the arrow keys and a swipe go between them, a double press or + zooms, Escape closes and the focus goes back. A picture's data-full: is the file it opens, else its largest size.

The page: https://webfluent.monzeromer.dev/reference/lightbox

Every picture in its block opens, on a press or Enter, full screen: the arrow keys and a swipe go between them, a double press or + zooms, Escape closes and the focus goes back. A picture's data-full: is the file it opens, else its largest size.

```wf
Lightbox(label: String, previous: String, next: String, …) { … }
```

## Example

```wf
Lightbox(label: "Launch week", previous: "Previous photo", next: "Next photo", close: "Close", zoom: "Zoom", of: "of") {
    Gallery(columns: 3, ratio: 4 / 3) {
        Image(src: "/og.png", alt: "The launch card, as it was shared")
        Image(src: "/icon-512.png", alt: "The app icon on a white tile")
        Image(src: "/og.png", alt: "The launch card, cropped to its title") {
            style { object-position: 20% 50% }
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | What the pictures are: the accessible name of the full-screen dialog they open in |
| `previous:` | `String` | The previous button's accessible name, in your words |
| `next:` | `String` | The next button's accessible name, in your words |
| `close:` | `String` | The close button's accessible name, in your words |
| `zoom:` | `String` | The zoom button's accessible name, in your words |
| `of:` | `String` | The word between the two numbers of the counter, `2 of 5` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
