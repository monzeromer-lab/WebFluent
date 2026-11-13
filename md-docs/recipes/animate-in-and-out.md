---
kind: "recipe"
route: "/recipes/animate-in-and-out"
group: "look"
description: "You should know how `if`, `for`, `show` and `match` draw and remove elements (Control flow), and how a keyed `for … by` keeps an item's nodes."
from: ["/docs/guide/motion#before-you-start", "/docs/guide/motion#animate-an-element-in", "/docs/guide/motion#animate-an-element-out", "/docs/guide/motion#animate-show-and-match-branches", "/docs/guide/motion#troubleshooting"]
---
# Animate an element in and out

## Before you start

You should know how `if`, `for`, `show` and `match` draw and remove
elements ([Control flow](../learn/09-control-flow.md)), and how a keyed `for … by`
keeps an item's nodes. Motion attaches to those: an animation plays when
one of them brings an element in or takes it away.

## Animate an element in

To play an animation when an element appears, give it `animate:`, or write
the animation's name as a flag. This page shows four cards when you press
**Open**, each entering a different way:

```wf
page Enter(path: "/", title: "Enter", description: "Four ways to arrive.") {
    state open = false
    Heading("Enter").h1
    Button("Open") { on click { open = !open } }
    if open {
        Card.fadeIn { Text("Faded in") }
        Card(animate: .slideUp, speed: .fast) { Text("Slid up, quickly") }
        Card.scaleIn.slow { Text("Grew, slowly") }
        Card(animate: .fadeIn, duration: "600ms", delay: "200ms", easing: "ease-out") { Text("Tuned") }
    }
}
```

An animation plays when the element appears: on the first paint, and each
time the `if`, `for`, `match` or `show` around it brings it back.

The built-in animations are `fadeIn`, `fadeOut`, `slideUp`, `slideDown`,
`slideLeft`, `slideRight`, `scaleIn`, `scaleOut`, `bounce`, `shake`,
`pulse`, `spin`, and `expand` and `collapse`, which open and close a box to
the height of what is inside it.

To change how long it takes and how it is paced:

| Prop | What it sets | Values |
|---|---|---|
| `speed:` | A preset length | `.fast` (150ms), `.normal`, `.slow` (500ms); `.fast` and `.slow` are also flags |
| `duration:` | An exact length | a length of time: `"600ms"` |
| `delay:` | A wait before it starts | a length of time: `"200ms"` |
| `easing:` | How it is paced | a named easing, or any CSS timing function as a string |

The named easings are `.standard`, `.spring`, `.ease`, `.easeIn`,
`.easeOut`, `.easeInOut`, `.linear`, `.bouncy` and `.smooth`. Each is a
design token (`$ease-standard`, `$ease-spring`, …), so a theme retunes every
animation at once. A spring is not a curve CSS can name, so the engine
samples the spring into a `linear()` easing: it overshoots and settles the
way a spring does.

An element that stands still plays its enter animation as a stylesheet
rule, with no JavaScript, so a pre-rendered page animates before its script
has loaded. Everything the runtime plays — a branch arriving or leaving, a
list reordering, a route changing — goes through the Web Animations API,
where an animation that is interrupted gives way to the one that replaced
it instead of jumping.

### Play an animation when it is scrolled to

To hold an animation until the reader has scrolled to the element, add
`on: .enterView`. It plays once:

```wf
page Story(path: "/", title: "Story", description: "Sections that arrive as you read.") {
    Heading("Our story").h1
    Text("Scroll down.")
    Section(animate: .fadeIn, on: .enterView) { Heading("Further down").h2 }
}
```

`on: .mount`, the default, plays the animation when the element appears.

## Animate an element out

To play an animation when an element leaves, give it `exit:`. The runtime
waits for the animation to finish before it removes the node. Here the
alert slides in, and fades out when you dismiss it:

```wf
page Exit(path: "/", title: "Exit", description: "An alert that fades away.") {
    state toast: String? = "Saved"
    Heading("Exit").h1
    if let message = toast {
        Alert(message, animate: .slideDown, exit: .fadeOut).success {
            Button("Dismiss").sm { on click { toast = null } }
        }
    }
    Button("Show") { on click { toast = "Saved again" } }
}
```

Any element may carry `exit:`, not only the root of a branch. All the
elements that are leaving animate out together, and the removal waits for
the longest.

## Animate `show` and `match` branches

`show` toggles whether its content is visible, so that content animates
with `animate:` and `exit:` as an `if` branch does. A `match` arm's elements
animate as one arm replaces another. This wizard slides each step in:

```wf
enum Step { one, two, three }

page Wizard(path: "/", title: "Wizard", description: "Three steps, animated.") {
    state step: Step = .one
    Heading("Set up").h1
    match step {
        .one { Card(animate: .slideLeft, exit: .fadeOut) { Text("Step one")  Button("Next") { on click { step = .two } } } }
        .two { Card(animate: .slideLeft, exit: .fadeOut) { Text("Step two")  Button("Next") { on click { step = .three } } } }
        .three { Card(animate: .scaleIn) { Text("Done") } }
    }
}
```

## Troubleshooting

- **An enter animation does not play.** It plays only when the element
  appears. An element that is already on the page and changes does not
  re-enter: use `replayAnimation`, `animate(…)` or a `transition { }`.
- **A list item fades out and in again instead of moving.** The `for` has no
  `by`, or its key changes, so the runtime makes a new node. Key the list by
  what identifies the item.
- **A leaving element disappears at once.** Give it `exit:`. Without one,
  nothing delays the removal.
- **A `shared:` element crossfades instead of moving.** The two pages give
  it different names, or two elements on one page carry the same name.
- **Nothing animates at all.** Your system, or the page's root
  (`data-wf-motion="reduce"`), asks for reduced motion.
- **Your own keyframes are an error as a flag.** Write
  `animate: .Name`; only the built-in animations are flags.
