# CommandPalette.Item

> One thing to do or go to; on select is what it does.

The page: https://webfluent.monzeromer.dev/reference/commandpalette-item

One thing to do or go to; on select is what it does.

```wf
CommandPalette.Item(_ label: String, icon: String, shortcut: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words, which the typing matches |
| `icon:` | `String` | An icon before the words |
| `shortcut:` | `String` | Its keys, shown at its end as this system writes them |

## Events

- `select`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
