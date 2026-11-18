# Avatar

> A picture of a person, or their initials.

The page: https://webfluent.monzeromer.dev/reference/avatar

A picture of a person, or their initials.

```wf
Avatar(src: String, alt: String, initials: String, …)
```

## Example

```wf
Row(gap: .md, align: .center) {
    Avatar(src: "/icon-512.png", alt: "Ada Lovelace").lg
    Avatar(initials: "GH").primary
    Avatar(initials: "AT").sm
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The picture's address, cropped to the circle; a `data:image/…` address is taken. With it, `initials:` is not shown |
| `alt:` | `String` | What the picture shows, its `alt`; empty for a picture the name beside it already says |
| `initials:` | `String` | The letters shown when there is no `src:` |
| `size:` | `one of its cases` | How large the circle is. Written as a flag, `.sm`, or by name, `size: .sm`.

- `.sm`: 32px
- `.md` (the default): 40px, the default
- `.lg`: 56px |
| `tone:` | `one of its cases` | The colour behind the initials. Written as a flag, `.primary`, or by name, `tone: .primary`.

- `.primary`: The theme's primary colour, the initials in `color-on-primary` |

## Flags

`.sm` `.md` `.lg` `.primary`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
