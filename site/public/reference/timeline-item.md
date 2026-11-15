# Timeline.Item

> One event: its title, at: its time, and its block.

The page: https://webfluent.monzeromer.dev/reference/timeline-item

One event: its title, at: its time, and its block.

```wf
Timeline.Item(_ title: String, at: Date or DateTime or String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | What happened |
| `at:` | `Date or DateTime or String` | When: a `Date`, a `DateTime` or an ISO string, written as a date |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Timeline {
    Timeline.Item("Released", at: "2026-11-14") { Text("Version 6.1") }
    Timeline.Item("Planned") { Text("Version 7") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
