# Divider

> Horizontal rule.

The page: https://webfluent.monzeromer.dev/reference/divider

Horizontal rule.

```wf
Divider
```

## Example

```wf
Card {
    Card.Body {
        Heading("Billing").h3
        Text("Pro plan, billed yearly.")
        Divider
        Row(justify: .between) {
            Text("Next invoice").muted
            Text("$240.00").bold
        }
    }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
