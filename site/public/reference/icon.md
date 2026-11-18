# Icon

> One of the built-in SVG icons, rendered inline.

The page: https://webfluent.monzeromer.dev/reference/icon

One of the built-in SVG icons, rendered inline.

```wf
Icon(_ name: String, size: Enum, tone: Enum, muted: Bool)
```

## Example

```wf
Row(gap: .lg, align: .center) {
    Row(gap: .xs, align: .center) { Icon("check").success  Text("Deployed") }
    Row(gap: .xs, align: .center) { Icon("warning").warning  Text("2 warnings") }
    Row(gap: .xs, align: .center) { Icon("alert-circle").danger  Text("1 failed") }
    Row(gap: .xs, align: .center) { Icon("clock").muted  Text("Queued") }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `name` | `String (positional)` | One of the 83 built-in icon names (`home`, `search`, `chevron-down` …), drawn inline as SVG; another is `V08` and draws nothing |
| `size:` | `one of its cases` | How large it is, against the text around it. Written as a flag, `.sm`, or by name, `size: .sm`.

- `.sm`: `font-size-sm` (0.875rem)
- `.md` (the default): The text's own size, 1em (the default)
- `.lg`: `font-size-xl` |
| `tone:` | `one of its cases` | Its colour, a tone of the theme. Written as a flag, `.primary`, or by name, `tone: .primary`.

- `.primary`: The theme's primary colour
- `.secondary`: The theme's secondary colour
- `.success`: Green: done, healthy
- `.danger`: Red: destructive, failed
- `.warning`: Amber: needs attention
- `.info`: Blue: a neutral notice |
| `muted:` | `Bool` | In `color-text-muted`, for something secondary. Written as a flag, `.muted`, or `muted: true`. |

## Flags

`.sm` `.md` `.lg` `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.muted`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [V08](https://webfluent.monzeromer.dev/errors/V08.md): An icon the runtime does not draw

Before WebFluent 5.0.
