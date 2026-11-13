# Tcell

> Table cell (the original spelling of Table.Cell).

The page: https://webfluent.monzeromer.dev/reference/tcell

Table cell (the original spelling of Table.Cell).

```wf
Tcell(_ content: Any, header: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `Any  (positional)` | The cell's content |
| `header:` | `Bool` | Render as a `<th>` |

## Flags

`.header` `.dropInside` `.trapFocus`

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.0.
