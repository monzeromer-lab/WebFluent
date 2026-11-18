# Column

> A child of the 12-column grid, spanning span columns.

The page: https://webfluent.monzeromer.dev/reference/column

A child of the 12-column grid, spanning span columns.

```wf
Column(span: Number, stacks: Bool) { … }
```

## Example

```wf
Row(gap: .md) {
    Column(span: { base: 12, md: 8 }) {
        Card {
            Card.Body {
                Heading("Article").h3
                Text("The main column takes eight of the twelve on a wide screen.")
            }
        }
    }
    Column(span: { base: 12, md: 4 }) {
        Card.outlined {
            Card.Body {
                Heading("Related").h3
                Link("Layout", to: "/learn")
            }
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `span:` | `Number` | How many of the row's 12 columns it takes, a whole number 1–12 (`T01` otherwise), less its share of the gap; or a value per breakpoint |
| `stacks:` | `Bool` | Full width below 768px; a `span` of 3 or 4 is half the width below 1024px. Written as a flag, `.stacks`, or `stacks: true`. |

## Flags

`.stacks`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [T01](https://webfluent.monzeromer.dev/errors/T01.md): A value of the wrong type
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
