# CheckboxGroup.Option

> One box: its words and its value.

The page: https://webfluent.monzeromer.dev/reference/checkboxgroup-option

One box: its words and its value.

```wf
CheckboxGroup.Option(_ label: String, value: Any)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `value:` | `Any` | What the list holds while it is ticked; unset, its words |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state toppings: [String] = []
CheckboxGroup(bind: toppings, label: "Toppings", all: "Every topping") {
    CheckboxGroup.Option("Olives", value: "olives")
    CheckboxGroup.Option("Onions", value: "onions")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
