# 20. Motion

<!--
route: guide/motion
group: building
blurb: Motion is declared, not scripted: how an element enters and leaves, how a list staggers, how pages change.
description: Enter and exit animations, list stagger, keyframes, expanding boxes, counting numbers, shared elements, sequences and route transitions.
-->

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
`slideRight`, `scaleIn`, `scaleOut`, `bounce`, `shake`, `pulse`, `spin`,
and `expand`/`collapse`, which open and close a box to the height of what
is inside it. Speeds: `.fast` (150ms), `.normal`, `.slow` (500ms); or
`duration:` and `delay:` as lengths of time.

`easing:` is how it is paced: a named one — `.standard`, `.spring`,
`.ease`, `.easeIn`, `.easeOut`, `.easeInOut`, `.linear`, `.bouncy`,
`.smooth` — or any CSS timing function written as a string. A named easing
is a design token (`$ease-standard`, `$ease-spring`), so a theme retunes
every animation at once. A spring is not a curve CSS can name: the engine
samples the solver into a `linear()` easing, so it overshoots and settles
the way a spring does.

An animation plays when the element appears — on the first paint and each
time the `if`, `for`, `match` or `show` around it brings it back. An
element that stands still plays it as a stylesheet rule, with no
JavaScript, so a static page animates before its script has loaded;
everything the runtime plays — a branch arriving or leaving, a list
reordering, a route changing — goes through the Web Animations API, where
an animation that is interrupted gives way to the one that replaced it
instead of jumping.

### Playing when it is scrolled to

`on: .enterView` holds the animation until the reader has reached the
element, and plays it once:

```wf
Section(animate: .fadeIn, on: .enterView) { Heading("Further down").h2 }
```

`on: .mount` is the default, and says what already happens.

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

## Opening to the height of the content

CSS cannot animate to `height: auto`, so `.expand` measures what auto
would be and animates to that number, then hands the height back so the
box grows with its content afterwards:

```wf
page Details(path: "/") {
    state open = false
    Button("Details") { on click { open = !open } }
    show open {
        Stack(animate: .expand) { Text("As much text as there is.") }
    }
}
```

`.expand` on the way in is `.collapse` on the way out, so `exit:` needs no
second name.

## A number that counts

`count:` makes a number arrive rather than jump — over the length of time
it names, easing out as it gets there:

```wf
page Revenue(path: "/") {
    state revenue = 0
    Text(format(revenue, .currency), count: "600ms")
    Button("Load") { on click { revenue = 1200 } }
}
```

The first value is written as it is: a page does not open by counting up
from nothing. Every value after it is counted to from the one before, and
each step is formatted the way the first was — a currency counts as a
currency.

## The same element across a route change

`shared:` gives an element a name it keeps from one page to the next, so
the browser carries it rather than crossfading it away and back:

```wf
for post in posts by post.id {
    Link(to: "/p/{post.slug}") { Image(post.cover, alt: post.title, shared: "cover-{post.id}") }
}

page Post(path: "/p/:slug", slug: String) {
    derived post = posts.find(p => p.slug == slug)
    Image(post?.cover ?? "", alt: post?.title ?? "", shared: "cover-{post?.id}")
}
```

The two must carry the same name, and only one element on a page may carry
each name.

## Orchestration

`sequence` starts a group of elements on a clock:

```wf
page Landing(path: "/") {
    sequence {
        step { Heading("Welcome").h1.fadeIn }
        step(after: "120ms") { Text("What we do.").slideUp }
        step(after: "120ms") { Button("Start").primary.slideUp }
    }
}
```

`after:` is measured from the step before it, so the three start at 0ms,
120ms and 240ms. A step with no `after:` starts when the step before it
does, which is how several elements move together. A sequence is
orchestration and nothing else: it writes the clock onto the elements as
`delay:`, and what the rest of the build sees is the program you would
have written by hand.

## Driving one yourself

`animate(element, name, duration)` is a handle: it plays when you make it,
and `play()`, `cancel()` and `finished` are yours.

```wf
Card(ref: panel) {
    on mouseenter { animate(panel, "pulse", "400ms") }
}
```

`replayAnimation(element, name)` plays an animation the element already
carries again, from the start — the one case a flag cannot express, since
a flag plays once on arrival:

```wf
Card.outlined.fadeIn {
    on mouseenter(e) { replayAnimation(e.currentTarget, "fadeIn") }
}
```

## The defaults

`motion` in `webfluent.app.json` sets what an animation does when the
element says nothing:

```json
{ "motion": { "duration": "180ms", "easing": "$ease-standard" } }
```

They are the `animation-duration-normal` and `animation-easing-default`
tokens, so a theme may set them instead, and a style block reads the same
values.

## Transitions on state

For a property that changes with state, `transition { }` sets what to
tween and how long ([chapter 15](15-styling.md#transition)):

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
animation started, no wait before removal, no FLIP, no route transition,
no `transition { }` tween — a box that expands is simply open, and a
number that counts is simply there. You never check for it; the runtime
and the stylesheet do. An `animation:` or `transition:` you write by hand
in a `style` block has no such path unless you wrap it in
`@media (prefers-reduced-motion: no-preference) { }` — prefer the declared
forms, which have one.

## Next

[Internationalisation](21-i18n.md).
