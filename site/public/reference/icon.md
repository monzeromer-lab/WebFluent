# Icon

> One of the built-in SVG icons, rendered inline.

The page: https://webfluent.monzeromer.dev/reference/icon

One of the built-in SVG icons, rendered inline.

```wf
Icon(_ name: String, size: Enum, tone: Enum, muted: Bool)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `name` | `String  (positional)` | The icon name |
| `size:` | `.sm .md .lg` | Its size |
| `tone:` | `.primary .secondary .success .danger .warning .info` | Colour |
| `muted:` | `Bool` | Dimmed, for secondary text |

## Flags

`.sm` `.md` `.lg` `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.muted` `.dropInside` `.trapFocus`

## Example

```wf
Icon("search")
Icon("check").lg.success
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [V08](https://webfluent.monzeromer.dev/errors/V08.md): An icon the runtime does not draw

Since WebFluent 5.0.
