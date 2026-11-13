# Avatar

> A picture of a person, or their initials.

The page: https://webfluent.monzeromer.dev/reference/avatar

A picture of a person, or their initials.

```wf
Avatar(src: String, alt: String, initials: String, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Image URL |
| `alt:` | `String` | Text alternative for the image |
| `initials:` | `String` | Letters shown when there is no image |
| `size:` | `.sm .md .lg` | Its size |
| `tone:` | `.primary` | Colour of the initials |

## Flags

`.sm` `.md` `.lg` `.primary` `.dropInside` `.trapFocus`

## Example

```wf
Avatar(src: "/ada.jpg", alt: "Ada Lovelace")
Avatar(initials: "AL").primary.lg
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
