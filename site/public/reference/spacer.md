# Spacer

> Vertical space; md when no size is given.

The page: https://webfluent.monzeromer.dev/reference/spacer

Vertical space; md when no size is given.

```wf
Spacer(size: Enum)
```

## Example

```wf
Heading("Before you deploy").h3
Text("Run the checks, then the tests.")
Spacer.lg
Button("Deploy").primary
Spacer.sm
Text("Last deployed 3 hours ago.").muted.sm
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `size:` | `.xs .sm .md .lg .xl` | How much space |

## Flags

`.xs` `.sm` `.md` `.lg` `.xl`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
