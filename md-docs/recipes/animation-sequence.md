---
kind: "recipe"
route: "/recipes/animation-sequence"
group: "look"
description: "To start a group of elements one after another on a clock, wrap them in `sequence` and put each in a `step`:"
from: ["/docs/guide/motion#run-a-sequence-of-animations"]
---
# Run a sequence of animations

To start a group of elements one after another on a clock, wrap them in
`sequence` and put each in a `step`:

```wf
page Landing(path: "/", title: "Welcome", description: "A page that introduces itself.") {
    sequence {
        step { Heading("Welcome").h1.fadeIn }
        step(after: "120ms") { Text("What we do.").slideUp }
        step(after: "120ms") { Button("Start").primary.slideUp }
    }
}
```

`after:` is measured from the step before it, so the three start at 0ms,
120ms and 240ms. A step with no `after:` starts when the step before it
does, which is how several elements move together.

A `sequence` does nothing but timing: it writes the clock onto its elements
as `delay:` and leaves nothing of itself behind. What the rest of the build
sees is the program you would have written by hand.
