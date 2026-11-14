# Blockquote

> Quotation: the text, or its block as the quoted content.

The page: https://webfluent.monzeromer.dev/reference/blockquote

Quotation: the text, or its block as the quoted content.

```wf
Blockquote(_ content: Any) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `Any  (positional)` | The quoted text |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Blockquote { Text("Make it work, make it right, make it fast.") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
