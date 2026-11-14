# Menu.Item

> One entry of the menu; on click is what choosing it does.

The page: https://webfluent.monzeromer.dev/reference/menu-item

One entry of the menu; on click is what choosing it does.

```wf
Menu.Item(shortcut: String, checked: Bool, disabled: Bool, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `shortcut:` | `String` | The keys that do the same, `mod+b`, shown at its end as this system writes them (`⌘B`, `Ctrl+B`) |
| `checked:` | `Bool` | A check mark: the item turns something on and off |
| `disabled:` | `Bool` | Shown, skipped by the arrow keys, and cannot be chosen |
| `danger:` | `Bool` | What it does cannot be taken back — Delete, Discard: drawn in the danger colour |

## Flags

`.checked` `.disabled` `.danger`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on click`: The element was clicked or activated

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

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

Before WebFluent 5.0.
