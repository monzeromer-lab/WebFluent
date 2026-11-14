# Column

> A child of the 12-column grid, spanning span columns.

The page: https://webfluent.monzeromer.dev/reference/column

A child of the 12-column grid, spanning span columns.

```wf
Column(span: Number, stacks: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `span:` | `Number` | How many of the 12 grid columns to span |
| `stacks:` | `Bool` | Reflow on a narrow screen: a Row becomes a column, a Grid one column |

## Flags

`.stacks`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Row {
    Column(span: 8) { Text("Main") }
    Column(span: 4) { Text("Aside") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [T01](https://webfluent.monzeromer.dev/errors/T01.md): A value of the wrong type
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
