# Stat

> A figure with its label — revenue, sign-ups — and how it moved: Stat("Revenue", value: total, format: .currency, change: 0.124). count: counts to a new value.

The page: https://webfluent.monzeromer.dev/reference/stat

A figure with its label — revenue, sign-ups — and how it moved: Stat("Revenue", value: total, format: .currency, change: 0.124). count: counts to a new value.

```wf
Stat(_ label: String, value: Any, format: Enum, change: Number) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | What the figure is |
| `value:` | `Any  (required)` | The figure |
| `format:` | `.number .integer .currency .percent .compact` | How the figure is written, as `format` writes it |
| `change:` | `Number` | How it moved, as a fraction: `0.124` is up 12.4% |

## Flags

`.number` `.integer` `.currency` `.percent` `.compact`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Stat.Stat.Label`
- `Stat.Stat.Value`
- `Stat.Stat.Change`

## Example

```wf
Stat("Revenue", value: 48210, format: .currency, change: 0.124)
Stat("Churn", value: 0.031, format: .percent, change: -0.02)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
