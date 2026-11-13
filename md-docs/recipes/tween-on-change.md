---
kind: "recipe"
route: "/recipes/tween-on-change"
group: "look"
description: "To animate a style property that changes with state, rather than an element arriving, add a `transition { }` block that names the property and how…"
from: ["/docs/guide/motion#tween-a-property-when-state-changes"]
---
# Tween a property when state changes

To animate a style property that changes with state, rather than an element
arriving, add a `transition { }` block that names the property and how long
it takes ([Styling](../learn/13-styling.md)):

```wf
page Tween(path: "/", title: "Tween", description: "A panel that grows.") {
    state open = false
    Heading("Tween").h1
    Card {
        style { max-height: {if open { "20rem" } else { "3rem" }}; overflow: hidden }
        transition { max-height: 250ms ease-in-out }
        Button("Toggle") { on click { open = !open } }
        Text("Long content that reveals itself.")
    }
}
```
