# Tcell

> Table cell (the original spelling of Table.Cell).

The page: https://webfluent.monzeromer.dev/reference/tcell

Table cell (the original spelling of Table.Cell).

```wf
Tcell(_ content: Any, header: Bool) { … }
```

Use [`Table.Cell`](https://webfluent.monzeromer.dev/reference/table-cell.md) in new code: it is the current spelling, and its page has the example.

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `Any  (positional)` | The cell's content |
| `header:` | `Bool` | Render as a `<th>` |

## Flags

`.header`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
