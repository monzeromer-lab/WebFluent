# VirtualList

> A scrolling box that draws only the rows in view of the one keyed for inside it — for a tree, search results or a diff of thousands of rows.

The page: https://webfluent.monzeromer.dev/reference/virtuallist

A scrolling box that draws only the rows in view of the one keyed for inside it — for a tree, search results or a diff of thousands of rows.

```wf
VirtualList(rowHeight: Number, estimate: Number, reveal: String or Number, …) { … }
```

## Example

```wf
state rows = (0..500).map(i => "Commit #{i + 1}")
state picked = "Commit #250"
Text("Picked: {picked}").muted
VirtualList(rowHeight: 36, reveal: picked, label: "Commits") {
    style { height: 260px; border: 1px solid $color-border; border-radius: $radius-md }
    for r in rows by r {
        Row(justify: .between, align: .center) {
            style { height: 36px; padding: 0 0.75rem }
            Text(r)
            Button("Pick").sm { on click { picked = r } }
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `rowHeight:` | `Number` | Every row's height, in px: the fast path. Unset, each row is measured once drawn |
| `estimate:` | `Number` | The height a row not yet drawn stands for while there is no `rowHeight`, in px |
| `reveal:` | `String or Number` | A row's key, as its `for … by` gives it: when it changes, that row is scrolled into view — the selection, the current match |
| `label:` | `String` | The scrolling box's accessible name (`aria-label`): what the list is, for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [V12](https://webfluent.monzeromer.dev/errors/V12.md): A long list that is not windowed

Since WebFluent 5.9.0.
