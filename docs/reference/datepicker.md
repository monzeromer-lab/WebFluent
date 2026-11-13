# DatePicker

> Date input.

The page: https://webfluent.monzeromer.dev/reference/datepicker

Date input.

```wf
DatePicker(bind: State, label: String, min: String, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `min:` | `String` | Earliest date, `YYYY-MM-DD` |
| `max:` | `String` | Latest date, `YYYY-MM-DD` |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.disabled` `.dropInside` `.trapFocus`

## Example

```wf
state start: Date? = null
DatePicker(bind: start, label: "Start date", min: "2026-01-01")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
