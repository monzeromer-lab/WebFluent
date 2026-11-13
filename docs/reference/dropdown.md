# Dropdown

> Button that opens a menu of Dropdown.Items.

The page: https://webfluent.monzeromer.dev/reference/dropdown

Button that opens a menu of Dropdown.Items.

```wf
Dropdown(_ label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Label of the button that opens it |

## Flags

`.dropInside` `.trapFocus`

## Slots

- `children`

## Parts

- `Dropdown.Dropdown.Item`
- `Dropdown.Dropdown.Divider`

## Example

```wf
Dropdown(label: "Actions") {
    Dropdown.Item { Text("Duplicate") }
    Dropdown.Divider
    Dropdown.Item { Text("Archive") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.0.
