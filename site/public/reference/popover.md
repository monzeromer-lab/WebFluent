# Popover

> A layer at a control: drawn above everything, placed against its anchor:, closed by Escape or a press outside, which write false back to visible: and fire on close.

The page: https://webfluent.monzeromer.dev/reference/popover

A layer at a control: drawn above everything, placed against its anchor:, closed by Escape or a press outside, which write false back to visible: and fire on close.

```wf
Popover(visible: State, anchor: Any, at: Any, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `anchor:` | `Any` | What it is placed against: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }` |
| `at:` | `Any` | A point to place it at, `{ x: e.clientX, y: e.clientY }`: where the pointer asked for a context menu |
| `placement:` | `.bottom .bottomStart .bottomEnd .top .topStart .topEnd .left .leftStart .leftEnd .right .rightStart .rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room. Unset, `.bottomStart` |
| `offset:` | `Number` | How far from the anchor, in px |
| `label:` | `String` | What it is, for a screen reader: its accessible name |
| `arrow:` | `Bool` | A point toward the anchor |

## Flags

`.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` `.arrow` `.dropInside` `.trapFocus`

## Events

- `close`

## Slots

- `children`

## Example

```wf
state open = false
Button("Branches", ref: branches) { on click { open = !open } }
Popover(visible: open, anchor: branches, placement: .bottomStart, label: "Branches").arrow {
    on close(reason) { log(reason) }
    Text("main")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.9.0.
