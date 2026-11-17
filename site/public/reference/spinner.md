# Spinner

> Loading indicator, announced as a status.

The page: https://webfluent.monzeromer.dev/reference/spinner

Loading indicator, announced as a status.

```wf
Spinner(size: Enum)
```

## Example

```wf
Row(gap: .lg, align: .center) {
    Spinner.sm
    Spinner
    Spinner.lg
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `size:` | `.sm .md .lg` | Its size |

## Flags

`.sm` `.md` `.lg`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
