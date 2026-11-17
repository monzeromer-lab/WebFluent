# Splitter

> A handle that sets the size of a region beside it: dragged, between min: and max:; double-clicked or Enter, back to reset:; the arrow keys move it.

The page: https://webfluent.monzeromer.dev/reference/splitter

A handle that sets the size of a region beside it: dragged, between min: and max:; double-clicked or Enter, back to reset:; the arrow keys move it.

```wf
Splitter(bind: State, min: Number, max: Number, …)
```

## Example

```wf
state side = 220
Text("Sidebar: {side}px").muted.sm
Row {
    style { height: 180px; border: 1px solid $color-border; border-radius: $radius-md; overflow: hidden }
    Stack(gap: .xs) {
        style { width: {side}px; padding: 0.75rem; background: $color-surface }
        Text("Files").bold
        Text("App.wf")
        Text("Home.wf")
    }
    Splitter(bind: side, min: 140, max: 400, reset: 220, label: "Sidebar width")
    Stack {
        style { flex: 1; padding: 0.75rem }
        Text("Drag the handle, or focus it and use the arrow keys. Double-click puts it back.")
    }
}
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

`.horizontal` `.reverse`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.9.0.
