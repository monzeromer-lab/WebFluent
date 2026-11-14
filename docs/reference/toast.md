# Toast

> Temporary notification.

The page: https://webfluent.monzeromer.dev/reference/toast

Temporary notification.

```wf
Toast(_ message: String, tone: Enum)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `message` | `String  (positional)` | The message |
| `tone:` | `.success .danger .warning .info` | Kind of notice |

## Flags

`.success` `.danger` `.warning` `.info`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state saved = false
if saved { Toast("Saved").success }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
