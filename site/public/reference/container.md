# Container

> Centred wrapper with a maximum width and horizontal padding.

The page: https://webfluent.monzeromer.dev/reference/container

Centred wrapper with a maximum width and horizontal padding.

```wf
Container(fluid: Bool) { … }
```

## Example

```wf
Container {
    style { background: $color-surface; border: 1px dashed $color-border; padding: 1rem }
    Heading("Release notes").h3
    Text("Version 7.0 makes the compiler strict: what 6.x accepted and 7.0 refuses is an error with its fix.").muted
    Row(gap: .sm) {
        Badge("Compiler").primary
        Badge("Breaking").warning
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `fluid:` | `Bool` | Full width, no maximum |

## Flags

`.fluid`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
