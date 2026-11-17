# Text

> A paragraph of text. {…} in the string reads state and stays live.

The page: https://webfluent.monzeromer.dev/reference/text

A paragraph of text. {…} in the string reads state and stays live.

```wf
Text(_ content: String or Number or Bool, tone: Enum, size: Enum, align: Enum, …)
```

## Example

```wf
Stack(gap: .xs) {
    Text("Billing").uppercase.sm.muted
    Text("Your plan renews on 1 December.").lg
    Text("Payment failed: the card was declined.").danger.bold
    Text("Updated 3 minutes ago").muted.sm.italic
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `String or Number or Bool  (positional)` | The text |
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

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
