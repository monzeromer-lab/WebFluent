# Stepper

> Steps taken in order — an account, then billing, then a summary — with each step's state (done, current, an error) and the current step's block shown. bind: holds the current step, from 0; .linear keeps a step out of reach until the ones before it are passed.

The page: https://webfluent.monzeromer.dev/reference/stepper

Steps taken in order — an account, then billing, then a summary — with each step's state (done, current, an error) and the current step's block shown. bind: holds the current step, from 0; .linear keeps a step out of reach until the ones before it are passed.

```wf
Stepper(bind: State, linear: Bool) { … }
```

## Example

```wf
state step = 0
state billed = false
Stepper(bind: step) {
    Stepper.Step("Account", done: step > 0) {
        Text("Your name and email.")
        Button("Next").primary { on click { step = 1 } }
    }
    Stepper.Step("Billing", done: billed) {
        Text("A card, or an invoice each month.")
        Button("Add a card").primary { on click { billed = true  step = 2 } }
    }
    Stepper.Step("Confirm") {
        Text("Check it over, then start.")
        Button("Start the trial").success
    }
}
Text("Step {step + 1} of 3").muted.sm
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | A Number state, the current step counted from 0: pressing a step's button writes it, and only that step's block shows |
| `linear:` | `Bool` | A step after the current one cannot be pressed: its button is disabled. Written as a flag, `.linear`, or `linear: true`. |

## Flags

`.linear`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Stepper`.

### Stepper.Step

One step: its title in the list, its block shown while it is the current one.

```wf
Stepper.Step(_ title: String, done: Bool, error: Bool) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String (positional)` | Its title, on its button in the list and as its panel's name |
| `done:` | `Bool` | Passed: drawn with a check while it is not the current step; `.done`, or `done: value` to follow state. Written as a flag, `.done`, or `done: true`. |
| `error:` | `Bool` | Something in it needs fixing: drawn as an error, before current or done; `.error`, or `error: value` to follow state. Written as a flag, `.error`, or `error: true`. |

Flags: `.done` `.error`

## Drawn for it

What `Stepper` draws itself, by its class; not written.

- `Stepper.List` (`.`): The steps, in order.
- `Stepper.Item` (`.`): A step in the list.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
