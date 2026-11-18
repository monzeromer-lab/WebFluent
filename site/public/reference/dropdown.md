# Dropdown

> Button that opens a menu of Dropdown.Items.

The page: https://webfluent.monzeromer.dev/reference/dropdown

Button that opens a menu of Dropdown.Items.

```wf
Dropdown(_ label: String) { … }
```

## Example

```wf
state said = "Nothing chosen"
Dropdown("Actions") {
    Dropdown.Item { Button("Duplicate") { on click { said = "Duplicated" } } }
    Dropdown.Item { Button("Rename") { on click { said = "Renamed" } } }
    Dropdown.Divider
    Dropdown.Item { Button("Archive").danger { on click { said = "Archived" } } }
}
Text(said).muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | The words of the button that opens its menu; the first argument, `Dropdown("Actions")`, or by name |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Dropdown`.

### Dropdown.Item

One entry of the menu.

```wf
Dropdown.Item { … }
```

### Dropdown.Divider

A rule between groups of entries.

```wf
Dropdown.Divider
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
