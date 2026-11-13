# Container

> Centred wrapper with a maximum width and horizontal padding.

The page: https://webfluent.monzeromer.dev/reference/container

Centred wrapper with a maximum width and horizontal padding.

```wf
Container(fluid: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `fluid:` | `Bool` | Full width, no maximum |

## Flags

`.fluid` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Container { Heading("Pricing").h1 }
Container.fluid { Text("Edge to edge") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
