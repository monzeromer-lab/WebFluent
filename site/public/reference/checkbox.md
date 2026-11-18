# Checkbox

> Checkbox bound to a boolean state.

The page: https://webfluent.monzeromer.dev/reference/checkbox

Checkbox bound to a boolean state.

```wf
Checkbox(bind: State, checked: Bool, label: String, …)
```

## Example

```wf
state terms = false
state newsletter = true
Stack(gap: .sm) {
    Checkbox(bind: terms, label: "I accept the terms of service")
    Checkbox(bind: newsletter, label: "Send me the monthly newsletter")
    Text(if terms { "Ready to sign up." } else { "Accept the terms to go on." }).muted
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `Bool` state it reads and writes: true while ticked |
| `checked:` | `Bool` | Whether it is ticked, when no state is bound: shown, not written back — handle `on change` to keep it (`F04` otherwise). Written as a flag, `.checked`, or `checked: true`. |
| `label:` | `String` | The words after the box, which a press on also ticks; also its accessible name |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it. Written as a flag, `.disabled`, or `disabled: true`. |

## Flags

`.checked` `.disabled`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on change`: The value was committed

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [F04](https://webfluent.monzeromer.dev/errors/F04.md): A checkbox that shows a state and never changes it
- [A04](https://webfluent.monzeromer.dev/errors/A04.md): A control with no label

Before WebFluent 5.0.
