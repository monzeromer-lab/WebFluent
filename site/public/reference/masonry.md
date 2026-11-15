# Masonry

> Its children in columns: columns, each placed under the last in its column, keeping its own height.

The page: https://webfluent.monzeromer.dev/reference/masonry

Its children in columns: columns, each placed under the last in its column, keeping its own height.

```wf
Masonry(columns: Number) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many columns (unset, 3) — or one per breakpoint, `{ base: 1, md: 3 }` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Masonry(columns: 2) {
    Card { Text("A short note") }
    Card { Text("A longer note, which runs to more than one line of text.") }
    Card { Text("Another") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
