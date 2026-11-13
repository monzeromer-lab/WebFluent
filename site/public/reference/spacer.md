# Spacer

> Vertical space; md when no size is given.

The page: https://webfluent.monzeromer.dev/reference/spacer

Vertical space; md when no size is given.

```wf
Spacer(size: Enum)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `size:` | `.xs .sm .md .lg .xl` | How much space |

## Flags

`.xs` `.sm` `.md` `.lg` `.xl` `.dropInside` `.trapFocus`

## Example

```wf
Text("Above")
Spacer.lg
Text("Below")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
