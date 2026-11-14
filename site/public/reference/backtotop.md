# BackToTop

> A button that appears once the page has scrolled past after: px, and takes the reader — and the focus — back to the top.

The page: https://webfluent.monzeromer.dev/reference/backtotop

A button that appears once the page has scrolled past after: px, and takes the reader — and the focus — back to the top.

```wf
BackToTop(label: String, after: Number)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | What it says, for a screen reader (unset, `Back to top`) |
| `after:` | `Number` | How far down it appears, in px (unset, 400) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
BackToTop(after: 600)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
