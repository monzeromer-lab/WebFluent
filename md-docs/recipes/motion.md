---
chapter: 20
kind: "recipe"
route: "guide/motion"
group: "ui"
blurb: "Declare how an element enters and leaves, how a list staggers and how pages change — the runtime plays it, or skips it for readers who asked for less."
description: "Enter and exit animations, list stagger, keyframes, expanding boxes, counting numbers, shared elements, sequences and route transitions."
---
# 20. Motion

This chapter shows you how to animate a page by declaring the motion, not by
scripting it. An element says how it enters and leaves, a list says how its
items stagger, the router says how pages change, and the runtime plays it
all. A reader who asked their system for less motion gets none, and you
never have to check for that yourself.

In this chapter you will:

- animate an element in and out, and when it is scrolled to;
- animate a list as items arrive, leave and move;
- declare your own keyframes;
- open a box to the height of its content, and count a number up;
- carry an element across a route change, and animate the route change;
- run a sequence of animations, or drive one yourself;
- check what a reader who wants reduced motion sees.

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

## Animate a list

To animate the items of a list as they are added, removed and moved, use a
keyed `for … by` and give the item's element `animate:`, `exit:` and
`stagger:`:

```wf
type Item { id: Number, text: String }

page Tasks(path: "/", title: "Tasks", description: "A list that animates.") {
    state items: [Item] = [Item(id: 1, text: "First"), Item(id: 2, text: "Second")]
    state next = 3
    action add() { items.push(Item(id: next, text: "Item {next}"))  next = next + 1 }
    action shuffle() { items = items.reverse() }
    Heading("Tasks").h1
    Row(gap: .sm) {
        Button("Add") { on click { add() } }
        Button("Reverse") { on click { shuffle() } }
    }
    for item in items by item.id {
        Card(animate: .fadeIn, exit: .fadeOut, stagger: "40ms") {
            Row(justify: .between) {
                Text(item.text)
                Button("Remove").sm { on click { items = items.filter(i => i.id != item.id) } }
            }
        }
    }
}
```

What each change does:

- **An item arrives.** It plays its `animate:`. `stagger: "40ms"` delays
  each item's enter by its index × 40ms, so a list fills in one item after
  another.
- **An item moves** — a sort, a reverse, an insert above it. It slides to
  its new place. This is a *FLIP* transform: the runtime measures where the
  node was and where it is, and animates between them. It works because
  `by` keeps the item's node alive.
- **An item leaves.** It plays its `exit:` first.

## Declare your own keyframes

To animate in a way the built-ins do not, declare keyframes with
`animation Name { }` and use the name where a built-in animation goes:

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

page Custom(path: "/", title: "Custom", description: "Keyframes of your own.") {
    state saving = false
    Heading("Custom").h1
    Card(animate: .Drop) {
        Button("Save") { on click { saving = !saving } }
        if saving { Badge("Saving", animate: .Pulse).info }
    }
}
```

The rules for keyframes:

- A step is `from`, `to` or a percentage, and holds raw CSS.
- The declaration is written once to the shared stylesheet as
  `@keyframes Name`.
- Your own animation is always written `animate: .Name` (or `exit:
  .Name`). The flags belong to the built-in animations.
- It plays wherever a built-in one would — on an element that stands still,
  from the stylesheet, before any script has loaded.

You can also name your keyframes in a style, as CSS does:
`animation: Pulse 1s infinite`.

## Open a box to the height of its content

CSS cannot animate to `height: auto`. To open a box to whatever height its
content needs, use `.expand`: it measures the height `auto` would give,
animates to that number, then hands the height back so the box keeps
growing with its content:

```wf
page Details(path: "/", title: "Details", description: "A box that opens smoothly.") {
    state open = false
    Heading("Details").h1
    Button("Details") { on click { open = !open } }
    show open {
        Stack(animate: .expand) { Text("As much text as there is.") }
    }
}
```

`.expand` on the way in is `.collapse` on the way out, so the box needs no
`exit:`.

## Count a number to its new value

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

## Carry an element across a route change

To have the browser carry one element from one page to the next — a cover
picture that grows into the article's header — give it the same `shared:`
name on both pages. The browser moves it instead of fading it out and back
in:

```wf
type Post { id: String, slug: String, title: String, cover: String }

const posts: [Post] = [
    Post(id: "1", slug: "slow-roasting", title: "Slow roasting", cover: "/roast.jpg"),
]

page Blog(path: "/", title: "Blog", description: "Every post.") {
    Heading("Blog").h1
    for post in posts by post.id {
        Link(to: "/p/{post.slug}") {
            Image(src: post.cover, alt: post.title, shared: "cover-{post.id}")
            Text(post.title)
        }
    }
}

page Article(path: "/p/:slug", title: "Post", description: "One post.", slug: String) {
    if let post = posts.find(p => p.slug == slug) {
        Heading(post.title).h1
        Image(src: post.cover, alt: post.title, shared: "cover-{post.id}")
    } else {
        Heading("Not found").h1
    }
}
```

Both elements must carry the same name, and only one element on a page may
carry each name — which is why the name here includes the post's `id`.

## Animate route changes

To animate every change of page, give the `Router` a `transition:` and a
`duration:`:

```wf
app {
    Navbar {
        Navbar.Brand { Link("Motion", to: "/") }
        Navbar.Links { Link("Home", to: "/")  Link("About", to: "/about") }
    }
    Router(transition: .fade, duration: "200ms")
}

page Home(path: "/", title: "Home", description: "The front page.") { Heading("Home").h1 }
page About(path: "/about", title: "About", description: "Who we are.") { Heading("About").h1 }
```

`transition:` is `.fade`, `.slide` or `.none` (the next page replaces the
last at once). On each route change the old page plays out and the new one
in; then focus and scroll settle and the new title is announced.

Where the browser has the View Transitions API, the change plays through
it: the browser's own crossfade, or the slide the engine's stylesheet
defines for `[data-wf-transition="slide"]`. Your own `::view-transition-*`
rules in a project stylesheet may restyle either. In other browsers, a
class-based animation plays instead.

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

## Tween a property when state changes

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

## Run a sequence of animations

To start a group of elements one after another on a clock, wrap them in
`sequence` and put each in a `step`:

```wf
page Landing(path: "/", title: "Welcome", description: "A page that introduces itself.") {
    sequence {
        step { Heading("Welcome").h1.fadeIn }
        step(after: "120ms") { Text("What we do.").slideUp }
        step(after: "120ms") { Button("Start").primary.slideUp }
    }
}
```

`after:` is measured from the step before it, so the three start at 0ms,
120ms and 240ms. A step with no `after:` starts when the step before it
does, which is how several elements move together.

A `sequence` does nothing but timing: it writes the clock onto its elements
as `delay:` and leaves nothing of itself behind. What the rest of the build
sees is the program you would have written by hand.

## Drive an animation yourself

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

## Set the default duration and easing

To change what an animation does when the element names no duration or
easing, set `motion` in `webfluent.app.json`:

```json
{ "motion": { "duration": "180ms", "easing": "$ease-standard" } }
```

These set the `animation-duration-normal` and `animation-easing-default`
design tokens, so a theme may set them instead, and a style block reads the
same values.

## Respect reduced motion

A reader whose system asks for reduced motion (`prefers-reduced-motion`)
gets none of the motion in this chapter: no animation started, no wait
before removal, no FLIP, no route transition and no `transition { }` tween.
A box that expands is open at once, and a number that counts is shown at its
value. The runtime and the engine's stylesheet do this for you.

> [!WARNING]
> An `animation:` or `transition:` you write by hand in a `style { }` block
> has no reduced-motion path. Wrap it in
> `@media (prefers-reduced-motion: no-preference) { }`, or use the declared
> forms in this chapter, which have one.

### Preview reduced motion

To see a page as that reader does without changing your system's setting,
set `data-wf-motion="reduce"` on the document's root element, for example
from the browser's console:

```js
document.documentElement.setAttribute("data-wf-motion", "reduce")
```

The stylesheet and the runtime honour the attribute exactly as they honour
the reader's setting, and removing it plays everything again. An editor's
preview sets it when you ask it for reduced motion.

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

## Next

[Media](media.md): add pictures the build resizes for every screen, video with captions, audio with a transcript, icons, carousels and frames.
