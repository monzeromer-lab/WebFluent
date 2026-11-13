# Menu

> A menu above everything: opened by its trigger: button, or by visible: at an anchor: or a point at: — a context menu. Escape, a press outside or a choice close it, write false back to visible: and fire on close.

The page: https://webfluent.monzeromer.dev/reference/menu

A menu above everything: opened by its trigger: button, or by visible: at an anchor: or a point at: — a context menu. Escape, a press outside or a choice close it, write false back to visible: and fire on close.

```wf
Menu(trigger: String, label: String, visible: State, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `trigger:` | `String` | Label of the button that opens the menu |
| `label:` | `String` | Same as `trigger` |
| `visible:` | `State` | State flag that opens and closes it |
| `anchor:` | `Any` | What it is placed against: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }` |
| `at:` | `Any` | A point to place it at, `{ x: e.clientX, y: e.clientY }`: where the pointer asked for a context menu |
| `placement:` | `.bottom .bottomStart .bottomEnd .top .topStart .topEnd .left .leftStart .leftEnd .right .rightStart .rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room. Unset, `.bottomStart` |
| `offset:` | `Number` | How far from the anchor, in px |

## Flags

`.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` `.dropInside` `.trapFocus`

## Events

- `close`

## Slots

- `children`

## Parts

- `Menu.Menu.Item`
- `Menu.Menu.Divider`
- `Menu.Menu.Sub`

## Example

```wf
Menu(trigger: "Options") {
    Menu.Item(shortcut: "F2") { Text("Rename") }
    Menu.Sub(label: "Move to") { Menu.Item { Text("Archive") } }
    Menu.Divider
    Menu.Item(disabled: true) { Text("Delete") }
}
state asked = false
state point = { x: 0, y: 0 }
Menu(visible: asked, at: point) { Menu.Item { Text("Copy") } }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
