# Menu.Divider

> A rule between groups of entries.

The page: https://webfluent.monzeromer.dev/reference/menu-divider

A rule between groups of entries.

```wf
Menu.Divider
```

## Flags

`.dropInside` `.trapFocus`

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
