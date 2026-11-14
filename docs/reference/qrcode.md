# QrCode

> A QR code for a URL or any text, drawn as vector graphics when the page is built.

The page: https://webfluent.monzeromer.dev/reference/qrcode

A QR code for a URL or any text, drawn as vector graphics when the page is built.

```wf
QrCode(_ value: String, color: String, background: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `value` | `String  (positional)` | What it encodes |
| `color:` | `String` | The colour of the dark modules |
| `background:` | `String` | The colour behind them |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
QrCode("https://example.com/report/q1")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.2.0.
