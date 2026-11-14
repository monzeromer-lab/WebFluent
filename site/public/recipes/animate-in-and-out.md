# Animate an element in and out

> Play an animation when an element appears or leaves: a flag or animate: on the way in, exit: on the way out, with speed, delay and easing.

The page: https://webfluent.monzeromer.dev/recipes/animate-in-and-out

Give an element an enter animation to play when it appears, and an `exit:`
to play before it is removed. Motion attaches to the `if`, `for`, `show`
and `match` that draw and remove elements, so read
[Control flow](https://webfluent.monzeromer.dev/learn/09-control-flow.md) first if those, or how a keyed
`for … by` keeps an item's nodes, are new to you. This page opens four cards
that each enter a different way, and an alert that fades out when you
dismiss it:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Notices(path: "/", title: "Notices", description: "Cards and an alert that animate in and out.") {
    state open = false
    state toast: String? = "Saved"
    Heading("Notices").h1
    Button("Toggle the cards") { on click { open = !open } }
    if open {
        Card.fadeIn { Text("Faded in") }
        Card(animate: .slideUp, speed: .fast) { Text("Slid up, quickly") }
        Card.scaleIn.slow { Text("Grew, slowly") }
        Card(animate: .fadeIn, duration: "600ms", delay: "200ms", easing: "ease-out") { Text("Tuned") }
    }
    if let message = toast {
        Alert(message, animate: .slideDown, exit: .fadeOut).success {
            Button("Dismiss").sm { on click { toast = null } }
        }
    }
    Button("Show the alert") { on click { toast = "Saved again" } }
}
```

## How it works

### Animate an element in

An element plays its enter animation when it appears: on the first paint,
and each time the `if`, `for`, `match` or `show` around it brings it back.
Name the animation with `animate:`, or write it as a flag (`Card.fadeIn`).

The built-in animations are `fadeIn`, `fadeOut`, `slideUp`, `slideDown`,
`slideLeft`, `slideRight`, `scaleIn`, `scaleOut`, `bounce`, `shake`,
`pulse`, `spin`, and `expand` and `collapse`, which open and close a box to
the height of what is inside it ([Open a box to the height of its
content](https://webfluent.monzeromer.dev/recipes/expand-to-content.md)).

Four props set how long it takes and how it is paced:

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

### Animate an element out

`exit:` names the animation to play when the element leaves. The runtime
waits for it to finish before it removes the node: here the alert fades
out after **Dismiss** sets `toast` to `null`.

Any element may carry `exit:`, not only the root of a branch. All the
elements that are leaving animate out together, and the removal waits for
the longest.

A reader who asked for reduced motion sees the cards and the alert appear
and go at once, with no wait before removal
([Respect reduced motion](https://webfluent.monzeromer.dev/recipes/motion-defaults.md#respect-reduced-motion)).

## Variations

### Play an animation when it is scrolled to

To hold an animation until the reader has scrolled to the element, add
`on: .enterView`. It plays once. `on: .mount`, the default, plays it when
the element appears.

```wf
page Story(path: "/", title: "Story", description: "Sections that arrive as you read.") {
    Heading("Our story").h1
    Text("Scroll down.")
    Section(animate: .fadeIn, on: .enterView) { Heading("Further down").h2 }
}
```

### Animate `show` and `match` branches

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
  re-enter: use `replayAnimation`, `animate(…)`
  ([Drive an animation yourself](https://webfluent.monzeromer.dev/recipes/drive-an-animation.md)) or a
  `transition { }` ([Tween a property when state changes](https://webfluent.monzeromer.dev/recipes/tween-on-change.md)).
- **A list item fades out and in again instead of moving.** The `for` has no
  `by`, or its key changes, so the runtime makes a new node. Key the list by
  what identifies the item ([Animate a list](https://webfluent.monzeromer.dev/recipes/animate-a-list.md)).
- **A leaving element disappears at once.** Give it `exit:`. Without one,
  nothing delays the removal.
- **A `shared:` element crossfades instead of moving.** The two pages give
  it different names, or two elements on one page carry the same name
  ([Carry an element across a route change](https://webfluent.monzeromer.dev/recipes/shared-element.md)).
- **Nothing animates at all.** Your system, or the page's root
  (`data-wf-motion="reduce"`), asks for reduced motion.
- **`` E103: Badge has no flag or enum case `Drop` ``, or `` `Card.Drop` is
  not a part of `Card` `` with a `C03`.** Your own keyframes were written
  as a flag. Write `animate: .Drop`; only the built-in animations are flags
  ([Declare your own keyframes](https://webfluent.monzeromer.dev/recipes/keyframes.md)).
