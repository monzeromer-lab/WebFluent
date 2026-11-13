---
kind: "recipe"
route: "/recipes/drive-an-animation"
group: "look"
description: "To play an animation from a handler, call `animate(element, name, duration)` with a `ref:` handle."
from: ["/docs/guide/motion#drive-an-animation-yourself"]
---
# Drive an animation yourself

To play an animation from a handler, call `animate(element, name,
duration)` with a `ref:` handle. It returns a handle that plays as soon as
it is made, and its `play()`, `cancel()` and `finished` are yours:

```wf
page Hover(path: "/", title: "Hover", description: "A card that pulses on hover.") {
    Heading("Hover").h1
    Card(ref: panel) {
        on mouseenter { animate(panel, "pulse", "400ms") }
        Text("Point at me.")
    }
}
```

A flag plays once, when the element arrives. To play an animation the
element already carries again, from the start, call
`replayAnimation(element, name)`:

```wf
page Replay(path: "/", title: "Replay", description: "A card that fades in again on hover.") {
    Heading("Replay").h1
    Card.outlined.fadeIn {
        on mouseenter(e) { replayAnimation(e.currentTarget, "fadeIn") }
        Text("Point at me.")
    }
}
```
