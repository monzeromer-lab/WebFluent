# Card

> Card surface with optional header, body and footer sections.

The page: https://webfluent.monzeromer.dev/reference/card

Card surface with optional header, body and footer sections.

```wf
Card(surface: Enum) { … }
```

## Example

```wf
Card.elevated {
    Card.Header {
        Row(justify: .between, align: .center) {
            Heading("Pro").h3
            Badge("Popular").primary.pill
        }
    }
    Card.Body {
        Text("Everything in Free, and more.").muted
        List {
            List.Item { Text("Unlimited projects") }
            List.Item { Text("Priority support") }
        }
    }
    Card.Footer { Button("Choose Pro").primary.full }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `surface:` | `one of its cases` | How it stands off the page. Written as a flag, `.flat`, or by name, `surface: .flat`.

- `.flat`: No border and no shadow
- `.elevated`: No border, raised with a shadow
- `.outlined`: A border on a transparent background, no shadow |

## Flags

`.flat` `.elevated` `.outlined`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Card`.

### Card.Header

The card's heading area.

```wf
Card.Header { … }
```

### Card.Body

The card's content area.

```wf
Card.Body { … }
```

### Card.Footer

The card's action area.

```wf
Card.Footer { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
