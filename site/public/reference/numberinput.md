# NumberInput

> A number with − and + beside it: the arrow keys step it, PageUp and PageDown by ten steps, and it stays between min: and max:.

The page: https://webfluent.monzeromer.dev/reference/numberinput

A number with − and + beside it: the arrow keys step it, PageUp and PageDown by ten steps, and it stays between min: and max:.

```wf
NumberInput(bind: State, min: Number, max: Number, …)
```

## Example

```wf
state tickets = 2
NumberInput(bind: tickets, min: 1, max: 10, step: 1, label: "Tickets", hint: "Ten at most per order.")
Text("Total: ${tickets * 45}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | The `Number` state it reads and writes; typing writes `null` when the field is emptied, and a number past `min:` or `max:` is brought back within them when it changes |
| `min:` | `Number` | The lowest value: − is disabled there, and the buttons and PageDown never go below it (a typed value is not held to it) |
| `max:` | `Number` | The highest value: + is disabled there, and the buttons and PageUp never go above it (a typed value is not held to it) |
| `step:` | `Number` | How far −, + and the arrow keys move it; PageUp and PageDown move ten steps |
| `label:` | `String` | Visible label; also the input's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `decrease:` | `String` | The − button's accessible name, in the page's words: `t("qty.less")` |
| `increase:` | `String` | The + button's accessible name, in the page's words |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Drawn for it

What `NumberInput` draws itself, by its class; not written.

- `NumberInput.Decrease` (`.`): Its − button.
- `NumberInput.Increase` (`.`): Its + button.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
