## Animation

Animation is set with props and flags on any element; the runtime plays enter, exit and reorder animations.

Every element takes the universal motion props: an enter animation as a
flag or `animate:`, `exit:`, `delay:`, `duration:`, `speed:` (`.fast`,
`.normal`, `.slow` — the flags `.fast`, `.normal` and `.slow` are this prop),
`stagger:` inside a list, `easing:`, `on:` (when the animation plays —
`.mount` or `.enterView`), `shared:` (a name it keeps across a route
change) and `count:` (a number that counts to its new value). Beside
them, every element also takes `class:`, `ref:`, and `mount:`/`update:`/
`cleanup:` (see [JavaScript interop](#javascript-interop)).

An element that stands still plays its enter animation as a stylesheet
rule, so a static page animates before its script has loaded. Everything
the runtime plays — a branch arriving or leaving, a list reordering, a
route changing, a replay — goes through the Web Animations API, where an
animation that is interrupted gives way to the one that replaced it.

```wf
// Mount animation (plays once on appear)
Card.elevated.fadeIn { Text("…") }
Heading("Title").h1.slideUp.slow

// Enter and exit: on the root of an if, for or show, the branch plays
// them; anywhere else the element plays its own exit when it leaves
if showPanel {
    Card(exit: .fadeOut).scaleIn { Text("Animated panel") }
}

// List with stagger; a keyed list moves items instead of rebuilding them,
// and with an enter or exit, an item whose order changed slides to its place
for item in items by item.id {
    Text(item.name, exit: .fadeOut, stagger: "50ms").slideUp
}

// Replay on hover (via event)
Card.outlined.fadeIn {
    on mouseenter(event) { replayAnimation(event.currentTarget, "fadeIn") }
}

// Plays when the reader has scrolled to it, and once
Section(animate: .fadeIn, on: .enterView) { Text("Further down") }

// A box that opens to the height of its content, which CSS cannot do
show details { Stack(animate: .expand) { Text("As much text as there is") } }

// A number that arrives rather than jumps, formatted the whole way
Text(format(revenue, .currency), count: "600ms")

// The same element carried from one page to the next
Image(post.cover, alt: post.title, shared: "cover-{post.id}")

// Orchestration: `after:` is measured from the step before it, so these
// start at 0ms, 120ms and 240ms. It writes the clock onto the elements
// as `delay:` and leaves nothing of itself behind.
sequence {
    step { Heading("Welcome").h1.fadeIn }
    step(after: "120ms") { Text("What we do.").slideUp }
    step(after: "120ms") { Button("Start").primary.slideUp }
}

// A handle, for a page that drives one itself: it plays when it is made
Card(ref: panel) { on mouseenter { animate(panel, "pulse", "400ms") } }

// Transition block (declarative, per-property; tokens allowed)
Button("Hover") {
    transition {
        background: 200ms ease
        transform: $animation-duration-fast spring
    }
}

// CSS transition property (in style block)
Button("Hover") {
    style {
        transition: all 200ms ease
    }
}
```

A route change animates too: `Router(transition: .fade | .slide, duration:
"200ms")` in `app`. Where the browser has the View Transitions API the
change plays through it — the browser's own crossfade, or the slide the
sheet defines for `[data-wf-transition="slide"]`, which a project's own
`::view-transition-*` rules may restyle — and the class-based animation is
the fallback. A reader who asked for less motion
(`prefers-reduced-motion`) gets none: nothing started, no wait, a box that
expands simply open and a number that counts simply there. What is playing
its exit is `inert` while it does: a press or a Tab lands on nothing that
is leaving.

`easing:` takes a named easing — `.standard`, `.spring`, `.ease`,
`.easeIn`, `.easeOut`, `.easeInOut`, `.linear`, `.bouncy`, `.smooth` — or
any CSS timing function as a string. Each name is a design token
(`$ease-standard`, `$ease-spring`), so a theme retunes every animation at
once; a spring is sampled into the `linear()` easing the platform runs, so
it overshoots and settles as a spring does. `motion` in the config sets
what an animation does when the element says nothing:

```json
{ "motion": { "duration": "180ms", "easing": "$ease-standard" } }
```

