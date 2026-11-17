# Fab

> A round button floating at the bottom of the screen for the page's main action; with Fab.Actions in its block it opens them instead (a speed dial).

The page: https://webfluent.monzeromer.dev/reference/fab

A round button floating at the bottom of the screen for the page's main action; with Fab.Actions in its block it opens them instead (a speed dial).

```wf
Fab(icon: String, label: String) { … }
```

## Example

```wf
state said = "Nothing made yet"
Fab(icon: "plus", label: "Create") {
    Fab.Action("Note", icon: "file") { on click { said = "A new note" } }
    Fab.Action("Folder", icon: "folder") { on click { said = "A new folder" } }
    Fab.Action("Upload", icon: "upload") { on click { said = "An upload" } }
}
Text(said).muted
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

Written under its name, inside `Fab`.

### Fab.Action

One more action the button opens: its words and its icon; on click is what it does.

```wf
Fab.Action(_ label: String, icon: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | What it does |
| `icon:` | `String` | Its icon |

## Drawn for it

What `Fab` draws itself, by its class; not written.

- `Fab.Trigger` (`.`): Its button.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
