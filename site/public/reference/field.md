# Field

> A label, a hint and an error around a control of your own: the first control in its block is named by the label and described by the hint and the error, and marked invalid while there is one.

The page: https://webfluent.monzeromer.dev/reference/field

A label, a hint and an error around a control of your own: the first control in its block is named by the label and described by the hint and the error, and marked invalid while there is one.

```wf
Field(label: String, hint: String, error: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Field.Field.Label`
- `Field.Field.Hint`
- `Field.Field.Error`

## Example

```wf
state plan = "basic"
Field(label: "Plan", hint: "You can change it later.") {
    Select(bind: plan) { Select.Option("Basic", value: "basic")  Select.Option("Pro", value: "pro") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
