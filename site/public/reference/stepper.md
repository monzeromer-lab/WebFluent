# Stepper

> Steps taken in order — an account, then billing, then a summary — with each step's state (done, current, an error) and the current step's block shown. bind: holds the current step, from 0; .linear keeps a step out of reach until the ones before it are passed.

The page: https://webfluent.monzeromer.dev/reference/stepper

Steps taken in order — an account, then billing, then a summary — with each step's state (done, current, an error) and the current step's block shown. bind: holds the current step, from 0; .linear keeps a step out of reach until the ones before it are passed.

```wf
Stepper(bind: State, linear: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The current step, from 0 |
| `linear:` | `Bool` | A step ahead of the current one cannot be pressed |

## Flags

`.linear`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Stepper.Stepper.Step`
- `Stepper.Stepper.List`
- `Stepper.Stepper.Item`

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
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
