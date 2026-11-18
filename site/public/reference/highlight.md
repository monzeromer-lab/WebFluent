# Highlight

> Text with every match of query: marked (<mark>), as a search's results show what was found; the case is ignored.

The page: https://webfluent.monzeromer.dev/reference/highlight

Text with every match of query: marked (<mark>), as a search's results show what was found; the case is ignored.

```wf
Highlight(_ text: String, query: String)
```

## Example

```wf
state q = "state"
Stack(gap: .sm) {
    Input(bind: q, label: "Search the guide").search
    Highlight("Pages, components and stores hold state; a derived value reads it.", query: q)
    Highlight("A state change repaints only what reads it.", query: q)
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String (positional)` | The text, every match of `query:` in it wrapped in `<mark>` |
| `query:` | `String` | What to mark: every match, its case ignored, after trimming; empty marks nothing |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
