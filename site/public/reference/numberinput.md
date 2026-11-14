# NumberInput

> A number with − and + beside it: the arrow keys step it, PageUp and PageDown by ten steps, and it stays between min: and max:.

The page: https://webfluent.monzeromer.dev/reference/numberinput

A number with − and + beside it: the arrow keys step it, PageUp and PageDown by ten steps, and it stays between min: and max:.

```wf
NumberInput(bind: State, min: Number, max: Number, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The `Number` it holds |
| `min:` | `Number` | The lowest it goes |
| `max:` | `Number` | The highest it goes |
| `step:` | `Number` | How far a press moves it (unset, 1) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Parts

- `NumberInput.NumberInput.Decrease`
- `NumberInput.NumberInput.Increase`

## Example

```wf
state qty = 1
NumberInput(bind: qty, min: 1, max: 99, label: "Quantity")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
