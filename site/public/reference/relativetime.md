# RelativeTime

> A moment as 3 minutes ago, kept current, with the exact time on hover (<time>).

The page: https://webfluent.monzeromer.dev/reference/relativetime

A moment as 3 minutes ago, kept current, with the exact time on hover (<time>).

```wf
RelativeTime(_ when: Any, datetime: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `when` | `Any  (positional)` | The moment: a `Date`, a `DateTime` or an ISO string |
| `datetime:` | `String` | The moment, as a machine reads it. Written for it |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
RelativeTime("2026-01-01T09:00:00Z")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
