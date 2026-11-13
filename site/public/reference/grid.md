# Grid

> CSS grid with a fixed number of columns.

The page: https://webfluent.monzeromer.dev/reference/grid

CSS grid with a fixed number of columns.

```wf
Grid(columns: Number, gap: Enum, align: Enum, …) { … }
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

`.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.stacks` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Grid(columns: { base: 1, md: 3 }, gap: .lg) {
    Card { Text("One") }
    Card { Text("Two") }
    Card { Text("Three") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
