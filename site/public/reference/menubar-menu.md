# Menubar.Menu

> One menu of the bar: its button's words, and its Menu.Items, Menu.Subs and Menu.Dividers.

The page: https://webfluent.monzeromer.dev/reference/menubar-menu

One menu of the bar: its button's words, and its Menu.Items, Menu.Subs and Menu.Dividers.

```wf
Menubar.Menu(_ label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The button's words |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
