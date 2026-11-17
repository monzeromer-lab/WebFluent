# Tag

> Compact chip.

The page: https://webfluent.monzeromer.dev/reference/tag

Compact chip.

```wf
Tag(_ label: String)
```

## Example

```wf
Stack(gap: .sm) {
    Heading("Slow roasting, explained").h3
    Row(gap: .xs) {
        Tag("Cooking")
        Tag("Technique")
        Tag("Long read")
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The tag text |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
