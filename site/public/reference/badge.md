# Badge

> Small status label.

The page: https://webfluent.monzeromer.dev/reference/badge

Small status label.

```wf
Badge(_ label: String, tone: Enum, pill: Bool)
```

## Example

```wf
Row(gap: .sm, align: .center) {
    Heading("Billing").h3
    Badge("Beta").info
    Badge("3 overdue").danger.pill
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | The badge's text; unset, its block |
| `tone:` | `one of its cases` | The colour it is filled with, a tone of the theme, its text in the tone's `color-on-*`. Written as a flag, `.primary`, or by name, `tone: .primary`.

- `.primary`: The theme's primary colour
- `.secondary`: The theme's secondary colour
- `.success`: Green: done, healthy
- `.danger`: Red: failed, destructive
- `.warning`: Amber: needs attention
- `.info`: Blue: a neutral notice |
| `pill:` | `Bool` | Fully rounded ends; the full theme already rounds a badge, so it matters with `theme.builtin: "structural"`. Written as a flag, `.pill`, or `pill: true`. |

## Flags

`.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.pill`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
