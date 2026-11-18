# Progress

> Progress bar; a value that reads state moves with it.

The page: https://webfluent.monzeromer.dev/reference/progress

Progress bar; a value that reads state moves with it.

```wf
Progress(value: Number, max: Number)
```

## Example

```wf
state sent = 3
Stack(gap: .sm) {
    Text("Sending invitations: {sent} of 8")
    Progress(value: sent, max: 8)
    Button("Send one more") { on click { sent = if sent < 8 { sent + 1 } else { 0 } } }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | How far along, out of `max:`; unset, the bar shows activity with no amount (indeterminate); a value that reads state follows it |
| `max:` | `Number` | The value at which the bar is full (the native `<progress>`'s `max`); with it unset `value:` is a fraction of 1 |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
