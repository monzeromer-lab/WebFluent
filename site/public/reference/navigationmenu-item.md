# NavigationMenu.Item

> An entry that opens a panel: its words are its button, its block the panel.

The page: https://webfluent.monzeromer.dev/reference/navigationmenu-item

An entry that opens a panel: its words are its button, its block the panel.

```wf
NavigationMenu.Item(_ label: String) { … }
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
NavigationMenu(label: "Main") {
    NavigationMenu.Item("Products") {
        Link("Editor", to: "/")
        Link("Compiler", to: "/")
    }
    NavigationMenu.Link("Pricing", to: "/")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
