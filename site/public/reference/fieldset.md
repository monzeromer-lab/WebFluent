# Fieldset

> Controls that belong together — an address, a choice of plan — under a legend that names them.

The page: https://webfluent.monzeromer.dev/reference/fieldset

Controls that belong together — an address, a choice of plan — under a legend that names them.

```wf
Fieldset(legend: String) { … }
```

## Example

```wf
state street = ""
state city = ""
state postcode = ""
Fieldset(legend: "Shipping address") {
    Input(bind: street, label: "Street")
    Row(gap: .md) {
        Input(bind: city, label: "City")
        Input(bind: postcode, label: "Postcode")
    }
}
Text("{street}, {postcode} {city}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `legend:` | `String (required)` | What the group is: drawn as its `<legend>`, which names the group for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Drawn for it

What `Fieldset` draws itself, by its class; not written.

- `Fieldset.Legend` (`.`): Its legend.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
