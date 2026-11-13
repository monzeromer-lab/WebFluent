# VirtualList

> A scrolling box that draws only the rows in view of the one keyed for inside it — for a tree, search results or a diff of thousands of rows.

The page: https://webfluent.monzeromer.dev/reference/virtuallist

A scrolling box that draws only the rows in view of the one keyed for inside it — for a tree, search results or a diff of thousands of rows.

```wf
VirtualList(rowHeight: Number, estimate: Number, reveal: Any, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `rowHeight:` | `Number` | Every row's height, in px: the fast path. Unset, each row is measured |
| `estimate:` | `Number` | The height a row not yet drawn stands for, in px. Unset, 32 |
| `reveal:` | `Any` | A row's key: when it changes, that row is scrolled into view — the selection, the current match |
| `label:` | `String` | What the list is, for a screen reader |

## Flags

`.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
state rows = (0..10000).map((i) => "row {i}")
state picked = "row 0"
VirtualList(rowHeight: 32, reveal: picked, label: "Rows") {
    style { height: 320px }
    for r in rows by r { Text(r) { style { height: 32px; margin: 0 } } }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [V12](https://webfluent.monzeromer.dev/errors/V12.md): A long list that is not windowed

Since WebFluent 5.9.0.
