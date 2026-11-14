# ToggleGroup.Item

> One choice: its words, and the value bind: holds when it is chosen.

The page: https://webfluent.monzeromer.dev/reference/togglegroup-item

One choice: its words, and the value bind: holds when it is chosen.

```wf
ToggleGroup.Item(_ label: String, value: Any, icon: String, disabled: Bool)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `value:` | `Any  (required)` | What `bind:` holds when it is chosen |
| `icon:` | `String` | An icon before the words |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.disabled`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state view = "grid"
ToggleGroup(bind: view, label: "View") {
    ToggleGroup.Item("Grid", value: "grid", icon: "grid")
    ToggleGroup.Item("List", value: "list", icon: "list")
}
state styles: [String] = []
ToggleGroup(bind: styles, label: "Text style").multiple {
    ToggleGroup.Item("Bold", value: "b")
    ToggleGroup.Item("Italic", value: "i")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
