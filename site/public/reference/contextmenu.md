# ContextMenu

> What it wraps, and a menu a right-click, a long press, the menu key or Shift+F10 inside it opens where it happened.

The page: https://webfluent.monzeromer.dev/reference/contextmenu

What it wraps, and a menu a right-click, a long press, the menu key or Shift+F10 inside it opens where it happened.

```wf
ContextMenu { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `ContextMenu.ContextMenu.Items`

## Example

```wf
state said = ""
ContextMenu {
    Card { Text("Right-click here, or long-press") }
    ContextMenu.Items(label: "File actions") {
        Menu.Item(shortcut: "F2") { on click { said = "rename" }  Text("Rename") }
        Menu.Item.danger { on click { said = "delete" }  Text("Delete") }
    }
}
Text(said)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
