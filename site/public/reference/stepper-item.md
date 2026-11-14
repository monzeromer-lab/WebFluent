# Stepper.Item

> A step in the list. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/stepper-item

A step in the list. Drawn for it.

```wf
Stepper.Item { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state step = 0
Stepper(bind: step) {
    Stepper.Step("Account") { Text("Your name and email.") }
    Stepper.Step("Billing", done: step > 1) { Text("A card on file.") }
    Stepper.Step("Confirm") { Text("Check and send.") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
