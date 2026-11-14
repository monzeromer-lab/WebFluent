# DateRangePicker

> A first and a last day, chosen in two months side by side that a button opens, or by a preset: presets: [.last7, .thisMonth]. bind: holds { start, end }.

The page: https://webfluent.monzeromer.dev/reference/daterangepicker

A first and a last day, chosen in two months side by side that a button opens, or by a preset: presets: [.last7, .thisMonth]. bind: holds { start, end }.

```wf
DateRangePicker(bind: State, label: String, min: Any, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The range: `{ start, end }` |
| `label:` | `String  (required)` | What the range is |
| `min:` | `Any` | The first day that can be chosen |
| `max:` | `Any` | The last day that can be chosen |
| `presets:` | `Any` | Ranges a press takes: `.today`, `.last7`, `.last30`, `.thisMonth`, `.lastMonth` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state stay = { start: null, end: null }
DateRangePicker(bind: stay, label: "Stay", presets: [.last7, .thisMonth])
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
