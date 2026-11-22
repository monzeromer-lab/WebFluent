# Calendar

> A month as a grid of days, chosen with a press or the keyboard — the arrows by day and week, PageUp and PageDown by month, Home and End to the week's ends — one day, a range, or several. bind: holds a Date, a { start, end } with .range, a list with .multiple.

The page: https://webfluent.monzeromer.dev/reference/calendar

A month as a grid of days, chosen with a press or the keyboard — the arrows by day and week, PageUp and PageDown by month, Home and End to the week's ends — one day, a range, or several. bind: holds a Date, a { start, end } with .range, a list with .multiple.

```wf
Calendar(bind: State, mode: Enum, min: Date, …)
```

## Example

```wf
state delivery: Date? = @2026-11-17
Calendar(bind: delivery, min: @2026-11-17, disabled: d => d.weekday() >= 6, label: "Delivery day").single
if let d = delivery {
    Text("Delivered on {d:.date(long)}").muted
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | The state it reads and writes, by `mode:`: a `Date?`, a `{ start, end }` of `Date`s (`end` null until the second day), or a `[Date]` kept in order |
| `mode:` | `one of its cases` | How many days it holds. Written as a flag, `.single`, or by name, `mode: .single`.

- `.single` (the default): One day: `bind:` is a `Date`
- `.range`: A first and a last day: `{ start, end }`; a third press starts again
- `.multiple`: Any number of days: a list, a press adding or removing one |
| `min:` | `Date` | The first day that can be chosen; days before it are shown but cannot be chosen |
| `max:` | `Date` | The last day that can be chosen; days after it are shown but cannot be chosen |
| `disabled:` | `[Date] or (Date) -> Bool` | Days that cannot be chosen: a list of them, or a test of a day, `d => d.weekday() >= 6` |
| `label:` | `String` | Its name for a screen reader, read once when it is drawn |
| `previous:` | `String` | The previous month button's accessible name, in the page's words |
| `next:` | `String` | The next month button's accessible name, in the page's words: `t("calendar.next")` |

## Flags

`.single` `.range` `.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
