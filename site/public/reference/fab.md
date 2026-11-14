# Fab

> A round button floating at the bottom of the screen for the page's main action; with Fab.Actions in its block it opens them instead (a speed dial).

The page: https://webfluent.monzeromer.dev/reference/fab

A round button floating at the bottom of the screen for the page's main action; with Fab.Actions in its block it opens them instead (a speed dial).

```wf
Fab(icon: String, label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String  (required)` | Its icon |
| `label:` | `String  (required)` | What it does, for a screen reader and its tooltip |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Fab.Fab.Action`
- `Fab.Fab.Trigger`

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
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
