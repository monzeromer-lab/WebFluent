# Field.Label

> Its label. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/field-label

Its label. Drawn for it.

```wf
Field.Label(_ text: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The label |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state plan = "basic"
Field(label: "Plan", hint: "You can change it later.") {
    Select(bind: plan) { Select.Option("Basic", value: "basic")  Select.Option("Pro", value: "pro") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
