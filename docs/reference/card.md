# Card

> Card surface with optional header, body and footer sections.

The page: https://webfluent.monzeromer.dev/reference/card

Card surface with optional header, body and footer sections.

```wf
Card(surface: Enum) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `surface:` | `.flat .elevated .outlined` | Surface treatment |

## Flags

`.flat` `.elevated` `.outlined` `.dropInside` `.trapFocus`

## Slots

- `children`

## Parts

- `Card.Card.Header`
- `Card.Card.Body`
- `Card.Card.Footer`

## Example

```wf
Card.elevated {
    Card.Header { Heading("Pro").h3 }
    Card.Body { Text("Everything in Free, and more.") }
    Card.Footer { Button("Choose Pro").primary }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
