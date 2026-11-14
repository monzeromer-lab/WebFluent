# TwoColumn

> Two equal columns; takes exactly two Container children.

The page: https://webfluent.monzeromer.dev/reference/twocolumn

Two equal columns; takes exactly two Container children.

```wf
TwoColumn { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Presentation {
    TwoColumn {
        Container { Heading("Wins").h3 }
        Container { Heading("Risks").h3 }
    }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
