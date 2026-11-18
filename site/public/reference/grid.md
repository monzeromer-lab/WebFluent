# Grid

> CSS grid with a fixed number of columns.

The page: https://webfluent.monzeromer.dev/reference/grid

CSS grid with a fixed number of columns.

```wf
Grid(columns: Number, gap: Enum, align: Enum, …) { … }
```

## Example

```wf
Grid(columns: { base: 1, sm: 2, md: 3 }, gap: .md) {
    Card.elevated {
        Card.Body {
            Text("Visitors").muted.sm
            Heading("12,480").h3
            Badge("+8%").success
        }
    }
    Card.elevated {
        Card.Body {
            Text("Sign-ups").muted.sm
            Heading("342").h3
            Badge("+3%").success
        }
    }
    Card.elevated {
        Card.Body {
            Text("Churn").muted.sm
            Heading("1.2%").h3
            Badge("-0.4%").danger
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many equal columns (`repeat(n, 1fr)`), a whole 1–12 as a plain number (`T01` otherwise); any count in a value per breakpoint, `{ base: 1, md: 3 }`; it follows state |
| `gap:` | `one of its cases` | Space between its children, from the theme's spacing scale (`wf-gap--*`, `gap:`); a flag (`.lg`) or a value per breakpoint (`{ base: .sm, lg: .lg }`). Written as a flag, `.xs`, or by name, `gap: .xs`.

- `.xs`: Extra small: `$spacing-xs`, 0.25rem
- `.sm`: Small: `$spacing-sm`, 0.5rem
- `.md` (the default): Medium: `$spacing-md`, 1rem (the default)
- `.lg`: Large: `$spacing-lg`, 1.5rem
- `.xl`: Extra large: `$spacing-xl`, 1.5–2rem, growing with the screen |
| `align:` | `one of its cases` | Where its children sit in their cells, down (`align-items`) (`align-items`); a flag for `.stretch`/`.baseline`, or a value per breakpoint. Written as a flag, `.start`, or by name, `align: .start`.

- `.start`: `align-items: flex-start`
- `.center`: `align-items: center`
- `.end`: `align-items: flex-end`
- `.stretch` (the default): `align-items: stretch`: each child fills that axis
- `.baseline`: `align-items: baseline`: text baselines line up |
| `justify:` | `one of its cases` | `justify-content` on the grid's tracks; with its equal `1fr` columns the tracks fill the width, so it shows no effect; a flag for `.between`/`.around`/`.evenly`. Written as a flag, `.start`, or by name, `justify: .start`.

- `.start` (the default): `justify-content: flex-start`: packed at the start
- `.center`: `justify-content: center`
- `.end`: `justify-content: flex-end`: packed at the end
- `.between`: `justify-content: space-between`: the first and last at the edges
- `.around`: `justify-content: space-around`: equal space around each
- `.evenly`: `justify-content: space-evenly`: equal space everywhere |
| `stacks:` | `Bool` | Two columns below 1024px and one below 768px; without it the columns stay as written at every width. Written as a flag, `.stacks`, or `stacks: true`. |

## Flags

`.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.stacks`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
