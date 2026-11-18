# VisuallyHidden

> Words for a screen reader only: Link(…) { Text("Docs")  VisuallyHidden("(opens in a new tab)") }.

The page: https://webfluent.monzeromer.dev/reference/visuallyhidden

Words for a screen reader only: Link(…) { Text("Docs")  VisuallyHidden("(opens in a new tab)") }.

```wf
VisuallyHidden(_ text: String) { … }
```

## Example

```wf
Row(gap: .md, align: .center) {
    Link(to: "/reference") {
        Text("Read more")
        VisuallyHidden("about the reference")
    }
    Row {
        Text("★★★★☆", aria-hidden: "true")
        VisuallyHidden("Rated 4 out of 5")
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String (positional)` | The words a screen reader reads and the page does not show; unset, its block |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
