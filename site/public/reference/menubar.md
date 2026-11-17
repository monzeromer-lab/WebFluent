# Menubar

> File, Edit and View, as in a desktop app: one Tab stop, the arrow keys along it; Down or Enter opens a menu, and with one open Left and Right open the one beside it.

The page: https://webfluent.monzeromer.dev/reference/menubar

File, Edit and View, as in a desktop app: one Tab stop, the arrow keys along it; Down or Enter opens a menu, and with one open Left and Right open the one beside it.

```wf
Menubar(label: String) { … }
```

## Example

```wf
state said = "Pick a command"
Menubar(label: "Editor") {
    Menubar.Menu("File") {
        Menu.Item(shortcut: "mod+n") { on click { said = "New file" }  Text("New file") }
        Menu.Item(shortcut: "mod+o") { on click { said = "Open…" }  Text("Open…") }
        Menu.Divider
        Menu.Item(shortcut: "mod+s") { on click { said = "Saved" }  Text("Save") }
    }
    Menubar.Menu("Edit") {
        Menu.Item(shortcut: "mod+z") { on click { said = "Undo" }  Text("Undo") }
        Menu.Item(shortcut: "mod+shift+z") { on click { said = "Redo" }  Text("Redo") }
    }
    Menubar.Menu("View") {
        Menu.Sub(label: "Zoom") {
            Menu.Item(shortcut: "mod+=") { on click { said = "Zoom in" }  Text("Zoom in") }
            Menu.Item(shortcut: "mod+-") { on click { said = "Zoom out" }  Text("Zoom out") }
        }
    }
}
Text(said).muted.sm
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The bar's name for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Menubar`.

### Menubar.Menu

One menu of the bar: its button's words, and its Menu.Items, Menu.Subs and Menu.Dividers.

```wf
Menubar.Menu(_ label: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The button's words |

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
