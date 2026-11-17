# Stat

> A figure with its label — revenue, sign-ups — and how it moved: Stat("Revenue", value: total, format: .currency, change: 0.124). count: counts to a new value.

The page: https://webfluent.monzeromer.dev/reference/stat

A figure with its label — revenue, sign-ups — and how it moved: Stat("Revenue", value: total, format: .currency, change: 0.124). count: counts to a new value.

```wf
Stat(_ label: String, value: Number or Money or String, format: Enum, change: Number, …) { … }
```

## Example

```wf
Grid(columns: { base: 1, md: 3 }, gap: .md) {
    Stat("Revenue", value: 48210, format: .currency, change: 0.124, trend: [31, 40, 28, 44, 52, 48])
    Stat("Active users", value: 18400, format: .compact, change: 0.031, trend: [15, 16, 16, 17, 18, 18])
    Stat("Churn", value: 0.021, format: .percent, change: -0.004)
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | What the figure is |
| `value:` | `Number or Money or String  (required)` | The figure |
| `format:` | `.number .integer .currency .percent .compact` | How the figure is written, as `format` writes it |
| `change:` | `Number` | How it moved, as a fraction: `0.124` is up 12.4% |
| `trend:` | `[Number]` | Its recent values, drawn as a sparkline under it: `[31, 40, 28, 51]` |

## Flags

`.number` `.integer` `.currency` `.percent` `.compact`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Stat`.

### Stat.Label

Its label.

```wf
Stat.Label(_ text: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The label |

### Stat.Value

Its figure.

```wf
Stat.Value(_ text: String or Number)
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String or Number  (positional)` | The figure |

### Stat.Change

How it moved, with its direction.

```wf
Stat.Change(_ text: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The change |

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
