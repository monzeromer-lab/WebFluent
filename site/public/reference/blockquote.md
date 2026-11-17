# Blockquote

> Quotation: the text, or its block as the quoted content.

The page: https://webfluent.monzeromer.dev/reference/blockquote

Quotation: the text, or its block as the quoted content.

```wf
Blockquote(_ content: String or Number or Bool) { … }
```

## Example

```wf
Blockquote {
    Text("We moved the whole docs site over in a weekend, and the compiler caught every broken link.").italic
    Text("Ada Lovelace, platform team").muted.sm
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `String or Number or Bool  (positional)` | The quoted text |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
