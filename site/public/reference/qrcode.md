# QrCode

> A QR code for a URL or any text, drawn as vector graphics when the page is built.

The page: https://webfluent.monzeromer.dev/reference/qrcode

A QR code for a URL or any text, drawn as vector graphics when the page is built.

```wf
QrCode(_ value: String, color: String, background: String)
```

## Example

```wf
Card.outlined {
    Row(gap: .md, align: .center) {
        QrCode("https://example.com/tickets/QX-4821")
        Stack(gap: .xs) {
            Text("Ticket QX-4821").bold
            Text("Show this at the door.").muted
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `value` | `String (positional)` | What it encodes, a URL or any text, known when the page is built (`E109` on a page otherwise); error correction level M, a 4-module quiet zone |
| `color:` | `String` | The colour of the dark modules, a CSS colour; anything else is the default |
| `background:` | `String` | The colour behind them, a CSS colour; anything else is the default |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.2.0.
