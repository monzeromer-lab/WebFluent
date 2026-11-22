# Radio

> One choice of a group; every radio bound to the same state is one group.

The page: https://webfluent.monzeromer.dev/reference/radio

One choice of a group; every radio bound to the same state is one group.

```wf
Radio(bind: State, value: String or Number or Bool or Bound, checked: Bool, …)
```

## Example

```wf
state shipping = "standard"
Stack(gap: .sm) {
    Radio(bind: shipping, value: "standard", label: "Standard — 5 to 7 days, free")
    Radio(bind: shipping, value: "express", label: "Express — 2 days, $9")
    Radio(bind: shipping, value: "overnight", label: "Overnight — $24")
    Text("Chosen: {shipping}").muted
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The state shared by a group of radios: it holds the chosen radio's `value:`, and radios bound to one state are one group for the arrow keys |
| `value:` | `String or Number or Bool or Bound` | What `bind:` holds when this one is chosen, and what a form sends for it; a bound radio without one is never checked; a case (`.food`) when `bind:` holds an enum (`Bound`) |
| `checked:` | `Bool` | Whether it is chosen, when no state is bound: shown, not written back. Written as a flag, `.checked`, or `checked: true`. |
| `label:` | `String` | The words after the button, which a press on also chooses; also its accessible name |
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
- [A04](https://webfluent.monzeromer.dev/errors/A04.md): A control with no label

Before WebFluent 5.0.
