# Image

> Image; alt is required for accessibility (rule A01). Given an image the program declares, it becomes a <picture> with every width the build wrote.

The page: https://webfluent.monzeromer.dev/reference/image

Image; alt is required for accessibility (rule A01). Given an image the program declares, it becomes a <picture> with every width the build wrote.

```wf
Image(_ source: Any, src: String, sizes: String, placeholder: Enum, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `source` | `Any  (positional)` | An `image` the program declares, or a URL |
| `src:` | `String` | Image URL |
| `sizes:` | `String` | How wide it will be shown, so the browser picks a width: `(max-width: 768px) 100vw, 1200px` |
| `placeholder:` | `.blur .color .none` | What fills the box while the image loads |
| `alt:` | `String` | Text alternative; empty for a purely decorative image |
| `width:` | `Number` | Intrinsic width |
| `height:` | `Number` | Intrinsic height |
| `loading:` | `.lazy .eager` | Unset, the first image on a page is eager at high priority and the rest lazy |
| `shape:` | `.rounded .circle` | Shape |

## Flags

`.blur` `.color` `.none` `.lazy` `.eager` `.rounded` `.circle` `.dropInside` `.trapFocus`

## Example

```wf
Image(src: "/team.jpg", alt: "The team at the launch", width: 1200, height: 630)
Image(src: "/flourish.svg", alt: "")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [R04](https://webfluent.monzeromer.dev/errors/R04.md): A relative URL on a nested route
- [D01](https://webfluent.monzeromer.dev/errors/D01.md): A file the project does not have
- [A01](https://webfluent.monzeromer.dev/errors/A01.md): An image with no alt text

Since WebFluent 5.0.
