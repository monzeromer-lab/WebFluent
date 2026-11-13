---
kind: "recipe"
route: "/recipes/count-a-number"
group: "look"
description: "To make a number arrive rather than jump, give its element `count:` with a length of time."
from: ["/docs/guide/motion#count-a-number-to-its-new-value"]
---
# Count a number to its new value

To make a number arrive rather than jump, give its element `count:` with a
length of time. The number eases out as it gets there:

```wf
page Revenue(path: "/", title: "Revenue", description: "A figure that counts up.") {
    state revenue = 0
    Heading("Revenue").h1
    Text(format(revenue, .currency), count: "600ms")
    Button("Load") { on click { revenue = 1200 } }
}
```

Press **Load** and the figure counts from `$0.00` to `$1,200.00`. The first
value is shown as it is, so a page does not open by counting up from
nothing. Every later value is counted to from the one before, and each step
is formatted the way the first was: a currency counts as a currency.
