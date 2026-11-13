---
kind: "recipe"
route: "/recipes/count-down"
group: "interaction"
description: "To count down once a second, change a state in an `every` timer:"
from: ["/docs/guide/recipes#count-down"]
---
# Count down

To count down once a second, change a state in an `every` timer:

```wf
page Countdown(path: "/", title: "Countdown", description: "Ten seconds, counted down.") {
    state left = 10
    every(1000) { if left > 0 { left = left - 1 } }
    Heading("Countdown").h1
    Progress(value: 10 - left, max: 10)
    Text(if left == 0 { "Done" } else { "{left}s" })
}
```

The timer stops when the page leaves.
