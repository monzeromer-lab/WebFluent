# ContextMenu.Items

> The menu: Menu.Items, Menu.Subs and Menu.Dividers.

The page: https://webfluent.monzeromer.dev/reference/contextmenu-items

The menu: Menu.Items, Menu.Subs and Menu.Dividers.

```wf
ContextMenu.Items(label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The menu's name for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
