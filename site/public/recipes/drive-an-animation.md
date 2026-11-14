# Drive an animation yourself

> Play an animation from a handler with animate(element, name, duration) and a ref: handle, or play an element's own again with replayAnimation.

The page: https://webfluent.monzeromer.dev/recipes/drive-an-animation

Play an animation when something happens, not only when an element
arrives: a card that pulses as the pointer reaches it. Call
`animate(element, name, duration)` from a handler, with a `ref:` handle on
the element:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Hover(path: "/", title: "Hover", description: "A card that pulses on hover.") {
    Heading("Hover").h1
    Card(ref: panel) {
        on mouseenter { animate(panel, "pulse", "400ms") }
        Text("Point at me.")
    }
}
```

## How it works

- `ref: panel` gives the page a handle on the card's element.
- `animate(panel, "pulse", "400ms")` plays the built-in `pulse` animation
  on it for 400ms, each time the pointer enters the card.
- `animate` returns a handle that plays as soon as it is made, and its
  `play()`, `cancel()` and `finished` are yours.

## Variations

### Replay an element's own animation

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
