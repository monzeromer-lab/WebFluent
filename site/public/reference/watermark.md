# Watermark

> Large, faint, rotated text behind each PDF page's content.

The page: https://webfluent.monzeromer.dev/reference/watermark

Large, faint, rotated text behind each PDF page's content.

```wf
Watermark(_ text: String, on: Enum)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The text |
| `on:` | `.all .first .rest .odd .even` | Which pages it is drawn on |

## Flags

`.all` `.first` `.rest` `.odd` `.even`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Watermark("DRAFT")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.2.0.
