# DateRangePicker

> A first and a last day, chosen in two months side by side that a button opens, or by a preset: presets: [.last7, .thisMonth]. bind: holds { start, end }.

The page: https://webfluent.monzeromer.dev/reference/daterangepicker

A first and a last day, chosen in two months side by side that a button opens, or by a preset: presets: [.last7, .thisMonth]. bind: holds { start, end }.

```wf
DateRangePicker(bind: State, label: String, min: Date, …)
```

## Example

```wf
state period = { start: @2026-11-01, end: @2026-11-17 }
DateRangePicker(bind: period, label: "Report period", presets: [.today, .last7, .last30, .thisMonth, .lastMonth])
Text("{period.start:.date(medium)} to {period.end:.date(medium)}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | The state it reads and writes: a `{ start, end }` of `Date`s, `end` null until the second day; the panel closes once both are chosen |
| `label:` | `String (required)` | What the range is: the button's words (`Stay: 3 Mar 2026 – 9 Mar 2026` once chosen) and the panel's name |
| `min:` | `Date` | The first day that can be chosen in either month |
| `max:` | `Date` | The last day that can be chosen in either month |
| `presets:` | `[RangePreset]` | Buttons above the months, each a range a press takes: `.today`, `.last7`, `.last30`, `.thisMonth`, `.lastMonth`, in the order listed |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
