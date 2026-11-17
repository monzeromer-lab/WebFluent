# Masonry

> Its children in columns: columns, each placed under the last in its column, keeping its own height.

The page: https://webfluent.monzeromer.dev/reference/masonry

Its children in columns: columns, each placed under the last in its column, keeping its own height.

```wf
Masonry(columns: Number) { … }
```

## Example

```wf
Masonry(columns: { base: 1, sm: 2, md: 3 }) {
    Card { Card.Body { Heading("Shipping").h3  Text("Free over $50.") } }
    Card {
        Card.Body {
            Heading("Returns").h3
            Text("Thirty days, no questions. Keep the box and the receipt, and send it back from any post office.")
        }
    }
    Card { Card.Body { Heading("Support").h3  Text("Every day, 8–8.") } }
    Card { Image(src: "/og.png", alt: "The WebFluent card") }
    Card { Card.Body { Heading("Gift cards").h3  Text("Sent by email, any amount.") } }
    Card { Card.Body { Heading("Warranty").h3  Text("Two years on everything we make.") } }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many columns (unset, 3) — or one per breakpoint, `{ base: 1, md: 3 }` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
