# Tag

> Compact chip.

The page: https://webfluent.monzeromer.dev/reference/tag

Compact chip.

```wf
Tag(_ label: String)
```

**Deprecated since 8.0:** write `Chip` instead. It still draws as it always has through every 8.x release and is removed in 9.0; `wf fix --code V13` rewrites it where the rewrite is exact. A `Chip` with no `selected:`, `.removable` or `on click` is the same static label, drawn as a pill at least 24px tall.

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
| `label` | `String (positional)` | The tag's text; unset, its block |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
