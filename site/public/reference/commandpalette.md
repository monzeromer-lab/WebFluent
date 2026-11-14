# CommandPalette

> A dialog of everything a reader can do or go to, filtered as they type: the arrow keys go through what matches, Enter runs it (on select) and closes. open: is the state that shows it.

The page: https://webfluent.monzeromer.dev/reference/commandpalette

A dialog of everything a reader can do or go to, filtered as they type: the arrow keys go through what matches, Enter runs it (on select) and closes. open: is the state that shows it.

```wf
CommandPalette(open: State, label: String, placeholder: String) { … }
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

- `CommandPalette.CommandPalette.Group`
- `CommandPalette.CommandPalette.Item`
- `CommandPalette.CommandPalette.List`

## Example

```wf
state palette = false
Button("Commands") { on click { palette = true } }
CommandPalette(open: palette) {
    CommandPalette.Group("Pages") {
        CommandPalette.Item("Home", icon: "home") { on select { navigate("/") } }
    }
    CommandPalette.Group("Actions") {
        CommandPalette.Item("Toggle theme", shortcut: "mod+shift+l") { on select { setTheme(if theme == "dark" { "light" } else { "dark" }) } }
    }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
