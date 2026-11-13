# Splitter

> A handle that sets the size of a region beside it: dragged, between min: and max:; double-clicked or Enter, back to reset:; the arrow keys move it.

The page: https://webfluent.monzeromer.dev/reference/splitter

A handle that sets the size of a region beside it: dragged, between min: and max:; double-clicked or Enter, back to reset:; the arrow keys move it.

```wf
Splitter(bind: State, min: Number, max: Number, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `min:` | `Number` | The smallest size, in px |
| `max:` | `Number` | The largest size, in px |
| `reset:` | `Number` | The size a double-click or Enter goes back to, taken as written: `0` can mean the region's default share |
| `label:` | `String` | What it resizes, for a screen reader |
| `horizontal:` | `Bool` | Between two regions one above the other: it moves up and down |
| `reverse:` | `Bool` | On the start side of what it sizes — a pane on the right, a panel below — so a drag toward the start grows it |

## Flags

`.horizontal` `.reverse` `.dropInside` `.trapFocus`

## Example

```wf
state side = 240
Row {
    Container { style { width: {side}px } Text("Files") }
    Splitter(bind: side, min: 160, max: 480, reset: 240, label: "Sidebar width")
    Container { Text("Editor") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.9.0.
