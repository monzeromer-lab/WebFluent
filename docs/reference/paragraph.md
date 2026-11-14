# Paragraph

> Block of text with paragraph spacing.

The page: https://webfluent.monzeromer.dev/reference/paragraph

Block of text with paragraph spacing.

```wf
Paragraph(_ content: Any, tone: Enum, size: Enum, align: Enum, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `Any  (positional)` | The text |
| `tone:` | `.primary .secondary .success .danger .warning .info` | Colour |
| `size:` | `.sm .md .lg` | Its size |
| `align:` | `.left .center .right` | Text alignment |
| `transform:` | `.uppercase .lowercase` | Letter case |
| `kind:` | `.heading .subtitle` | Typographic role |
| `bold:` | `Bool` | Bold |
| `italic:` | `Bool` | Italic |
| `underline:` | `Bool` | Underlined |
| `muted:` | `Bool` | Dimmed, for secondary text |

## Flags

`.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.left` `.center` `.right` `.uppercase` `.lowercase` `.heading` `.subtitle` `.bold` `.italic` `.underline` `.muted`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Paragraph("A block of prose, with the spacing a paragraph has.")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
