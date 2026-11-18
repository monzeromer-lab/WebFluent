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
| `bind:` | `State` | A Number of px it reads and writes: the size of the region beside it, which your own `style { width: {side}px }` applies; without it the handle does nothing |
| `min:` | `Number` | The smallest size it writes, in px; Home goes there |
| `max:` | `Number` | The largest size it writes, in px; End goes there |
| `reset:` | `Number` | The size a double-click or Enter writes, taken as written (not held to `min:`): `0` can mean the region's default share. Unset, neither does anything |
| `label:` | `String` | The handle's accessible name (`aria-label`): what it resizes, for a screen reader |
| `horizontal:` | `Bool` | A bar between two regions one above the other: it moves up and down, and the arrow keys are Up and Down. Written as a flag, `.horizontal`, or `horizontal: true`. |
| `reverse:` | `Bool` | It sizes the region after it, not before — a pane on the right, a panel below — so a drag toward the start grows it. Written as a flag, `.reverse`, or `reverse: true`. |

## Flags

`.horizontal` `.reverse`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.9.0.
