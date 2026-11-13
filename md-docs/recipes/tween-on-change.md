---
kind: "recipe"
route: "/recipes/tween-on-change"
group: "look"
description: "Animate a style property that changes with state, such as a panel's height, with a transition { } block that names the property and its timing."
outputs: ["website", "app"]
from: ["/docs/guide/motion#tween-a-property-when-state-changes"]
---
# Tween a property when state changes

Animate a style property as state changes it — a panel growing taller, a
colour shifting — rather than an element arriving. Add a `transition { }`
block that names the property and how long it takes. Press **Toggle** and
the card grows to show the rest of its text:

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

## How it works

- The `style { }` value of `max-height` reads `open`, so it follows the
  state: `3rem` closed, `20rem` open ([Styling](../learn/13-styling.md)).
- `transition { max-height: 250ms ease-in-out }` tweens that property
  whenever its value changes, over 250ms. Each line names a property, then
  its length and easing; a design token may stand for either.
- An element's block is ordered `style`, then `transition`, then its
  handlers and children.

A reader who asked for reduced motion gets no tween: the card changes
height at once
([Respect reduced motion](motion-defaults.md#respect-reduced-motion)).
