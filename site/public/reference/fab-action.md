# Fab.Action

> One more action the button opens: its words and its icon; on click is what it does.

The page: https://webfluent.monzeromer.dev/reference/fab-action

One more action the button opens: its words and its icon; on click is what it does.

```wf
Fab.Action(_ label: String, icon: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | What it does |
| `icon:` | `String` | Its icon |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state said = ""
Fab(icon: "plus", label: "Create") {
    Fab.Action("Note", icon: "file") { on click { said = "note" } }
    Fab.Action("Folder", icon: "folder") { on click { said = "folder" } }
}
Text(said)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
