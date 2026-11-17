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
| `columns:` | `Number` | Number of equal columns |
| `gap:` | `.xs .sm .md .lg .xl` | Space between children |
| `align:` | `.start .center .end .stretch .baseline` | Cross-axis alignment |
| `justify:` | `.start .center .end .between .around .evenly` | Main-axis distribution |
| `stacks:` | `Bool` | Reflow on a narrow screen: a Row becomes a column, a Grid one column |

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
