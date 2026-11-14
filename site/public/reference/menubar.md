# Menubar

> File, Edit and View, as in a desktop app: one Tab stop, the arrow keys along it; Down or Enter opens a menu, and with one open Left and Right open the one beside it.

The page: https://webfluent.monzeromer.dev/reference/menubar

File, Edit and View, as in a desktop app: one Tab stop, the arrow keys along it; Down or Enter opens a menu, and with one open Left and Right open the one beside it.

```wf
Menubar(label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The bar's name for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Menubar.Menubar.Menu`

## Example

```wf
state said = ""
Menubar(label: "Editor") {
    Menubar.Menu("File") {
        Menu.Item(shortcut: "mod+n") { on click { said = "new" }  Text("New") }
        Menu.Item { on click { said = "open" }  Text("Open…") }
    }
    Menubar.Menu("Edit") {
        Menu.Item(shortcut: "mod+z") { on click { said = "undo" }  Text("Undo") }
    }
}
Text(said)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
