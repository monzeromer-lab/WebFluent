# Checkbox

> Checkbox bound to a boolean state.

The page: https://webfluent.monzeromer.dev/reference/checkbox

Checkbox bound to a boolean state.

```wf
Checkbox(bind: State, checked: Bool, label: String, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `checked:` | `Bool` | Whether it is checked, when its state is not bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.checked` `.disabled` `.dropInside` `.trapFocus`

## Example

```wf
state agree = false
Checkbox(bind: agree, label: "I agree to the terms")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [F04](https://webfluent.monzeromer.dev/errors/F04.md): A checkbox that shows a state and never changes it
- [A04](https://webfluent.monzeromer.dev/errors/A04.md): A control with no label

Since WebFluent 5.0.
