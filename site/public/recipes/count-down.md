# Count down

> Count down once a second with an every timer that changes a state, and show the time left as a progress bar and text.

The page: https://webfluent.monzeromer.dev/recipes/count-down

A countdown is a number that drops once a second until it reaches zero. To
build one, change a state in an `every` timer:

```wf
page Countdown(path: "/", title: "Countdown", description: "Ten seconds, counted down.") {
    state left = 10
    every(1000) { if left > 0 { left = left - 1 } }
    Heading("Countdown").h1
    Progress(value: 10 - left, max: 10)
    Text(if left == 0 { "Done" } else { "{left}s" })
}
```

## How it works

- `left` is a [`state`](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#state) holding
  the seconds that remain.
- [`every(1000)`](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#timers) runs its
  block every 1000 milliseconds. The `if` stops the count at zero.
- The `Progress` bar and the `Text` both read `left`, so they repaint on
  every tick. The text says **Done** at zero.

The timer stops when the page leaves, so a reader who navigates away
leaves no interval running.
