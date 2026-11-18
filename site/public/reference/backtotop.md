# BackToTop

> A button that appears once the page has scrolled past after: px, and takes the reader — and the focus — back to the top.

The page: https://webfluent.monzeromer.dev/reference/backtotop

A button that appears once the page has scrolled past after: px, and takes the reader — and the focus — back to the top.

```wf
BackToTop(label: String, after: Number)
```

## Example

```wf
Stack(gap: .md) {
    for n in 1..=12 {
        Text("Chapter {n}: what the page says, long enough to scroll past.")
    }
}
BackToTop(after: 600, label: "Back to the top")
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Its accessible name and tooltip (`aria-label`, `title`); the button shows only an upward chevron |
| `after:` | `Number` | How far the window must scroll down before it appears, in px; `0` counts as 400 |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
