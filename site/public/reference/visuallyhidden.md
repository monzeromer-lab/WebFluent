# VisuallyHidden

> Words for a screen reader only: Link(…) { Text("Docs")  VisuallyHidden("(opens in a new tab)") }.

The page: https://webfluent.monzeromer.dev/reference/visuallyhidden

Words for a screen reader only: Link(…) { Text("Docs")  VisuallyHidden("(opens in a new tab)") }.

```wf
VisuallyHidden(_ text: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The words |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Link(to: "/docs") { Text("Docs")  VisuallyHidden("(the reference)") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
