# TwoColumn

> Two equal columns; takes exactly two Container children.

The page: https://webfluent.monzeromer.dev/reference/twocolumn

Two equal columns; takes exactly two Container children.

```wf
TwoColumn { … }
```

## Example

```wf
Presentation {
    TwoColumn {
        Container {
            Heading("Before").h2
            Text("A build took four minutes.")
        }
        Container {
            Heading("After").h2
            Text("It takes forty seconds.")
        }
    }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
