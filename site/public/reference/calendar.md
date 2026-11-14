# Calendar

> A month as a grid of days, chosen with a press or the keyboard — the arrows by day and week, PageUp and PageDown by month, Home and End to the week's ends — one day, a range, or several. bind: holds a Date, a { start, end } with .range, a list with .multiple.

The page: https://webfluent.monzeromer.dev/reference/calendar

A month as a grid of days, chosen with a press or the keyboard — the arrows by day and week, PageUp and PageDown by month, Home and End to the week's ends — one day, a range, or several. bind: holds a Date, a { start, end } with .range, a list with .multiple.

```wf
Calendar(bind: State, mode: Enum, min: Any, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The chosen day, range or days |
| `mode:` | `.single .range .multiple` | How many days it holds |
| `min:` | `Any` | The first day that can be chosen |
| `max:` | `Any` | The last day that can be chosen |
| `disabled:` | `Any` | Days that cannot be chosen: a list of them, or `d => d.weekday() >= 6` |
| `label:` | `String` | Its name for a screen reader; unset, `Calendar` |

## Flags

`.single` `.range` `.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state day: Date? = null
Calendar(bind: day, min: @2026-01-01, disabled: d => d.weekday() >= 6, label: "Delivery day")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
