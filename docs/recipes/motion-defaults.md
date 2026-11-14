# Set motion defaults, and respect reduced motion

> Set the duration and easing every animation uses by default, and make sure a reader who asked for reduced motion gets none.

The page: https://webfluent.monzeromer.dev/recipes/motion-defaults

Change how long animations take and how they are paced across a whole
project, in one place, and keep motion away from readers who asked for
less. On this page every animation takes the project's defaults, and the
one transition written by hand is held back from those readers:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Welcome(path: "/", title: "Welcome", description: "Motion that follows the project's defaults.") {
    state open = false
    Heading("Welcome").h1.fadeIn
    Button("Details") { on click { open = !open } }
    if open {
        Card(exit: .fadeOut).slideUp { Text("This card takes the default length and easing.") }
    }
    Card {
        style {
            @media (prefers-reduced-motion: no-preference) {
                transition: box-shadow $animation-duration-normal $animation-easing-default
            }
            &:hover { box-shadow: 0 4px 12px rgba(0,0,0,0.12) }
        }
        Text("A hand-written transition, kept from readers who asked for less motion.")
    }
}
```

## How it works

### Set the default duration and easing

An animation whose element names no duration or easing takes the
project's defaults. Set them with `motion` in `webfluent.app.json`:

```json
{ "motion": { "duration": "180ms", "easing": "$ease-standard" } }
```

These set the `animation-duration-normal` and `animation-easing-default`
design tokens, so a theme may set them instead, and a style block reads the
same values — as the hand-written transition above does.

### Respect reduced motion

A reader whose system asks for reduced motion (`prefers-reduced-motion`)
gets none of the declared motion: no animation started, no wait before
removal, no FLIP, no route transition and no `transition { }` tween. A box
that expands is open at once, and a number that counts is shown at its
value. The runtime and the engine's stylesheet do this for you.

> [!WARNING]
> An `animation:` or `transition:` you write by hand in a `style { }` block
> has no reduced-motion path. Wrap it in
> `@media (prefers-reduced-motion: no-preference) { }`, as the page above
> does, or use the declared forms, which have one.

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
