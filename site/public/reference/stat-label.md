# Stat.Label

> Its label.

The page: https://webfluent.monzeromer.dev/reference/stat-label

Its label.

```wf
Stat.Label(_ text: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The label |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Stat("Revenue", value: 48210, format: .currency, change: 0.124)
Stat("Churn", value: 0.031, format: .percent, change: -0.02)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
