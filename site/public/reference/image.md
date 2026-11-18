# Image

> Image; alt is required for accessibility (rule A01). Given an image the program declares, it becomes a <picture> with every width the build wrote.

The page: https://webfluent.monzeromer.dev/reference/image

Image; alt is required for accessibility (rule A01). Given an image the program declares, it becomes a <picture> with every width the build wrote.

```wf
Image(_ source: Picture or String, src: String, sizes: String, placeholder: Enum, …)
```

## Example

```wf
Stack(gap: .md) {
    Image(src: "/og.png", alt: "WebFluent's sharing card: the name over a dark field",
          sizes: "(max-width: 768px) 100vw, 640px", placeholder: .blur).rounded
    Row(gap: .md, align: .center) {
        Image(src: "/icon-512.png", alt: "The app icon", width: 64, height: 64).circle
        Text("Pictures from public/ are resized to WebP at every width a page asks for.").muted.sm
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `source` | `Picture or String (positional)` | An `image` the program declares (a `<picture>` at every width the build wrote), or a URL |
| `src:` | `String` | The picture's address. A literal path of `public/` to a PNG, JPEG, GIF or WebP is processed by the build like an `image`; an SVG, a URL or a `data:image/…` is used as written |
| `sizes:` | `String` | How wide it will be shown, so the browser picks one of the widths the build wrote: `(max-width: 768px) 100vw, 1200px`; for a processed picture |
| `placeholder:` | `one of its cases` | What fills the box of a processed picture while it loads, cleared when it lands. Written as a flag, `.blur`, or by name, `placeholder: .blur`.

- `.blur`: A 16-pixel-wide blurred copy, inlined in the page
- `.color`: The picture's average colour
- `.none` (the default): Nothing (the default) |
| `alt:` | `String` | Text alternative; `""` for a purely decorative image (`A01` when left out) |
| `width:` | `Number` | The `<img>`'s `width` in px, so its box is reserved before it loads; ignored on a processed picture, which carries the file's real size |
| `height:` | `Number` | The `<img>`'s `height` in px, so its box is reserved before it loads; ignored on a processed picture, which carries the file's real size |
| `loading:` | `one of its cases` | When the browser fetches it. Written as a flag, `.lazy`, or by name, `loading: .lazy`.

- `.lazy`: When it nears the viewport
- `.eager`: At once |
| `shape:` | `one of its cases` | Its corners. Written as a flag, `.rounded`, or by name, `shape: .rounded`.

- `.rounded`: Rounded corners (`radius-md`)
- `.circle`: Clipped to a circle (an ellipse, unless it is square) |

## Flags

`.blur` `.color` `.none` `.lazy` `.eager` `.rounded` `.circle`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on load`: An image, video or frame finished loading
- `on error`: An image, video or script failed to load

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [R04](https://webfluent.monzeromer.dev/errors/R04.md): A relative URL on a nested route
- [D01](https://webfluent.monzeromer.dev/errors/D01.md): A file the project does not have
- [A01](https://webfluent.monzeromer.dev/errors/A01.md): An image with no alt text

Before WebFluent 5.0.
