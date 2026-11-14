# Fieldset.Legend

> Its legend. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/fieldset-legend

Its legend. Drawn for it.

```wf
Fieldset.Legend(_ text: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The legend |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state street = ""
state city = ""
Fieldset(legend: "Address") {
    Input(bind: street, label: "Street")
    Input(bind: city, label: "City")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
