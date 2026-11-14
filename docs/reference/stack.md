# Stack

> Vertical flex layout.

The page: https://webfluent.monzeromer.dev/reference/stack

Vertical flex layout.

```wf
Stack(gap: Enum, align: Enum, justify: Enum) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `.xs .sm .md .lg .xl` | Space between children |
| `align:` | `.start .center .end .stretch .baseline` | Cross-axis alignment |
| `justify:` | `.start .center .end .between .around .evenly` | Main-axis distribution |

## Flags

`.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Stack(gap: .sm) {
    Text("First")
    Text("Second")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
