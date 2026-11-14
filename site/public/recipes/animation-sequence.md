# Run a sequence of animations

> Start a group of animations one after another on a clock: wrap the elements in sequence { } and each in a step, with after: from the step before.

The page: https://webfluent.monzeromer.dev/recipes/animation-sequence

Bring a page's opening elements in one after another — a heading, then a
line of text, then a button — by wrapping them in `sequence` and putting
each in a `step`:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Landing(path: "/", title: "Welcome", description: "A page that introduces itself.") {
    sequence {
        step { Heading("Welcome").h1.fadeIn }
        step(after: "120ms") { Text("What we do.").slideUp }
        step(after: "120ms") { Button("Start").primary.slideUp }
    }
}
```

The heading fades in as the page opens, the line beneath it slides up a
moment later, and the button slides up last, so the page arrives in order
rather than all at once.

## How it works

- `after:` is measured from the step before it, so the three start at 0ms,
  120ms and 240ms.
- A step with no `after:` starts when the step before it does, which is how
  several elements move together.
- A `sequence` does nothing but timing: it writes the clock onto its
  elements as `delay:` and leaves nothing of itself behind. What the rest
  of the build sees is the program you would have written by hand, with
  `delay: "120ms"` and `delay: "240ms"` on the second and third elements.
