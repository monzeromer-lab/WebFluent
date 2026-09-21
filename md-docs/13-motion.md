# 13. Motion

Motion in WebFluent is declared, not scripted: an element says how it
enters and leaves, a list says how its items stagger, the router says how
pages change, and the runtime plays it — or skips it for a reader who asked
for less motion.

## Enter animations

Every element takes `animate:` (or the animation's name as a flag):

```wf
page Enter(path: "/") {
    state open = false
    Button("Open") { on click { open = !open } }
    if open {
        Card.fadeIn { Text("Faded in") }
        Card(animate: .slideUp, speed: .fast) { Text("Slid up, quickly") }
        Card.scaleIn.slow { Text("Grew, slowly") }
        Card(animate: .fadeIn, duration: "600ms", delay: "200ms", easing: "ease-out") { Text("Tuned") }
    }
}
```

The animations: `fadeIn`, `fadeOut`, `slideUp`, `slideDown`, `slideLeft`,
`slideRight`, `scaleIn`, `scaleOut`, `bounce`, `shake`, `pulse`, `spin`.
Speeds: `.fast` (150ms), `.normal`, `.slow` (500ms); or `duration:`,
`delay:` and `easing:` as raw CSS values.

An animation plays when the element appears — on the first paint and each
time the `if`, `for`, `match` or `show` around it brings it back.

## Exit animations

`exit:` plays when the element leaves, and the runtime waits for it before
removing the node:

```wf
page Exit(path: "/") {
    state toast: String? = "Saved"
    if let t = toast {
        Alert(t, animate: .slideDown, exit: .fadeOut).success {
            Button("Dismiss").sm { on click { toast = null } }
        }
    }
    Button("Show") { on click { toast = "Saved again" } }
}
```

Any element may carry `exit:`, not only the root of a branch; whichever
elements are leaving animate out together, and removal follows the longest.

## Lists

A keyed `for` animates insertions, removals and moves:

```wf
type Item { id: Number, text: String }

page List(path: "/") {
    state items: [Item] = [Item(id: 1, text: "First"), Item(id: 2, text: "Second")]
    state next = 3
    action add() { items.push(Item(id: next, text: "Item {next}"))  next = next + 1 }
    action shuffle() { items = items.reverse() }
    Row(gap: .sm) {
        Button("Add") { on click { add() } }
        Button("Reverse") { on click { shuffle() } }
    }
    for item in items by item.id {
        Card(animate: .fadeIn, exit: .fadeOut, stagger: "40ms") {
            Row(justify: .between) {
                Text(item.text)
                Button("×").sm { on click { items = items.filter(i => i.id != item.id) } }
            }
        }
    }
}
```

- `stagger: "40ms"` delays each item's enter by its index × 40ms.
- An item that moves (a sort, a reverse, an insert above it) slides to its
  new place — a FLIP transform on the node, which `by` keeps alive.
- An item that leaves plays its `exit:` first.

## Custom animations

`animation Name { }` declares keyframes; use the name where a built-in
animation goes:

```wf
animation Pulse {
    from { opacity: 1 }
    50% { opacity: 0.4; transform: scale(0.98) }
    to { opacity: 1 }
}

animation Drop {
    from { transform: translateY(-12px); opacity: 0 }
    to { transform: none; opacity: 1 }
}

page Custom(path: "/") {
    state saving = false
    Card(animate: .Drop) {
        Button("Save") { on click { saving = !saving } }
        if saving { Badge("Saving", animate: .Pulse).info }
    }
}
```

Keyframe steps are `from`, `to` or a percentage; each holds raw CSS. The
declaration is emitted as `@keyframes Name` once, in the shared stylesheet.

## Transitions on state

For a property that changes with state, `transition { }` sets what to
tween and how long ([chapter 12](12-styling.md#transition)):

```wf
page Tween(path: "/") {
    state open = false
    Card {
        style { max-height: {if open { "20rem" } else { "3rem" }}; overflow: hidden }
        transition { max-height: 250ms ease-in-out }
        Button("Toggle") { on click { open = !open } }
        Text("Long content that reveals itself.")
    }
}
```

## Route transitions

```wf
app {
    Navbar {
        Navbar.Brand { Link("Motion", to: "/") }
        Navbar.Links { Link("Home", to: "/")  Link("About", to: "/about") }
    }
    Router(transition: .fade, duration: "200ms")
}

page Home(path: "/") { Heading("Home").h1 }
page About(path: "/about") { Heading("About").h1 }
```

`Router(transition: .fade | .slide | .none, duration:)` animates every
route change: the old page out, the new one in, then focus and scroll
settle and the title is announced. Where the browser has the View
Transitions API the change plays through it — the browser's crossfade, or
the slide the stylesheet defines for `[data-wf-transition="slide"]`, which
your own `::view-transition-*` rules may restyle. Elsewhere the class-based
animation is the fallback.

## `show` and `match`

`show` toggles visibility, so its content animates with `animate:`/`exit:`
the same way; a `match` arm's elements animate as they swap.

```wf
enum Step { one, two, three }

page Wizard(path: "/") {
    state step: Step = .one
    match step {
        .one { Card(animate: .slideLeft, exit: .fadeOut) { Text("Step one")  Button("Next") { on click { step = .two } } } }
        .two { Card(animate: .slideLeft, exit: .fadeOut) { Text("Step two")  Button("Next") { on click { step = .three } } } }
        .three { Card(animate: .scaleIn) { Text("Done") } }
    }
}
```

## Reduced motion

A reader whose system asks for reduced motion gets none of the above: no
enter or exit classes, no wait before removal, no FLIP, no route
transition, no `transition { }` tween. You never check for it; the runtime
and the stylesheet do. An `animation:` or `transition:` you write by hand
in a `style` block has no such path unless you wrap it in
`@media (prefers-reduced-motion: no-preference) { }` — prefer the declared
forms, which have one.

## Next

[Data](14-data.md).
