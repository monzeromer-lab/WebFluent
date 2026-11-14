# Dropdown.Divider

> A rule between groups of entries.

The page: https://webfluent.monzeromer.dev/reference/dropdown-divider

A rule between groups of entries.

```wf
Dropdown.Divider
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
