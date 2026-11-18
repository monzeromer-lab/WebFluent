# DatePicker

> A date field: the reader picks a day, and the bound state holds a Date.

The page: https://webfluent.monzeromer.dev/reference/datepicker

A date field: the reader picks a day, and the bound state holds a Date.

```wf
DatePicker(bind: State, label: String, min: String, …)
```

## Example

```wf
state start: Date? = @2026-11-17
DatePicker(bind: start, label: "Start date", min: "2026-01-01", max: "2026-12-31")
if let d = start {
    Text("Starts {d:.date(long)}").muted
} else {
    Text("No date chosen").muted
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `Date?` state it reads and writes: the day chosen, `null` when the field is emptied |
| `label:` | `String` | The label drawn above the field; also its accessible name |
| `min:` | `String` | The earliest day the picker offers, `YYYY-MM-DD` |
| `max:` | `String` | The latest day the picker offers, `YYYY-MM-DD` |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it. Written as a flag, `.disabled`, or `disabled: true`. |

## Flags

`.disabled`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on change`: The value was committed

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
