# CommandPalette

> A dialog of everything a reader can do or go to, filtered as they type: the arrow keys go through what matches, Enter runs it (on select) and closes. open: is the state that shows it.

The page: https://webfluent.monzeromer.dev/reference/commandpalette

A dialog of everything a reader can do or go to, filtered as they type: the arrow keys go through what matches, Enter runs it (on select) and closes. open: is the state that shows it.

```wf
CommandPalette(open: State, label: String, placeholder: String) { … }
```

## Example

```wf
state palette = false
state said = "Nothing run yet"
Button("Search commands…") { on click { palette = true } }
Text(said).muted.sm
CommandPalette(open: palette, label: "Commands", placeholder: "Type a command or a page") {
    CommandPalette.Group("Pages") {
        CommandPalette.Item("Reference", icon: "list") { on select { navigate("/reference") } }
        CommandPalette.Item("Recipes", icon: "book") { on select { navigate("/recipes") } }
    }
    CommandPalette.Group("Actions") {
        CommandPalette.Item("New file", icon: "file-plus", shortcut: "mod+n") { on select { said = "New file" } }
        CommandPalette.Item("Copy link", icon: "copy", shortcut: "mod+shift+c") { on select { said = "Link copied" } }
        CommandPalette.Item("Switch theme", icon: "moon") { on select { said = "Theme switched" } }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `open:` | `State  (required)` | The state that shows it, written back when it closes |
| `label:` | `String` | Its name and the field's; unset, `Commands` |
| `placeholder:` | `String` | Shown in the field; unset, `Type a command or search…` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `CommandPalette`.

### CommandPalette.Group

Items under a heading.

```wf
CommandPalette.Group(_ title: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | Its heading |

### CommandPalette.Item

One thing to do or go to; on select is what it does.

```wf
CommandPalette.Item(_ label: String, icon: String, shortcut: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words, which the typing matches |
| `icon:` | `String` | An icon before the words |
| `shortcut:` | `String` | Its keys, shown at its end as this system writes them |

Events: `select`

## Drawn for it

What `CommandPalette` draws itself, by its class; not written.

- `CommandPalette.List` (`.`): What matches.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
