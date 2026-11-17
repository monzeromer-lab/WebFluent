# ContextMenu

> What it wraps, and a menu a right-click, a long press, the menu key or Shift+F10 inside it opens where it happened.

The page: https://webfluent.monzeromer.dev/reference/contextmenu

What it wraps, and a menu a right-click, a long press, the menu key or Shift+F10 inside it opens where it happened.

```wf
ContextMenu { … }
```

## Example

```wf
state said = "Right-click the file, or focus it and press Shift+F10"
ContextMenu {
    Card.outlined {
        Row(gap: .sm, align: .center) {
            Icon("file")
            Text("quarterly-report.pdf")
        }
    }
    ContextMenu.Items(label: "File actions") {
        Menu.Item(shortcut: "mod+o") { on click { said = "Opened" }  Text("Open") }
        Menu.Item(shortcut: "F2") { on click { said = "Renamed" }  Text("Rename") }
        Menu.Divider
        Menu.Item.danger { on click { said = "Deleted" }  Text("Delete") }
    }
}
Text(said).muted.sm
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `ContextMenu`.

### ContextMenu.Items

The menu: Menu.Items, Menu.Subs and Menu.Dividers.

```wf
ContextMenu.Items(label: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The menu's name for a screen reader |

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
