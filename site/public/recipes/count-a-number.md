# Count a number to its new value

> Make a number count to its new value instead of jumping: give its element count: with a length of time, and each step keeps its format.

The page: https://webfluent.monzeromer.dev/recipes/count-a-number

Make a figure arrive rather than jump — a total, a score, a balance — by
giving its element `count:` with a length of time. Press **Load** and the
revenue counts from `$0.00` to `$1,200.00`:

```wf
page Revenue(path: "/", title: "Revenue", description: "A figure that counts up.") {
    state revenue = 0
    Heading("Revenue").h1
    Text(format(revenue, .currency), count: "600ms")
    Button("Load") { on click { revenue = 1200 } }
}
```

## How it works

- `count: "600ms"` counts the number to each new value over 600ms, easing
  out as it gets there.
- The first value is shown as it is, so a page does not open by counting
  up from nothing. Every later value is counted to from the one before.
- Each step is formatted the way the first was: a currency counts as a
  currency ([`format`](https://webfluent.monzeromer.dev/reference/built-ins.md#format)).

A reader who asked for reduced motion sees the number at its new value at
once ([Respect reduced motion](https://webfluent.monzeromer.dev/recipes/motion-defaults.md#respect-reduced-motion)).
