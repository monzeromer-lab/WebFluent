---
kind: "recipe"
route: "/recipes/keyframes"
group: "motion"
description: "Declare keyframes of your own with animation Name { } and play them with animate: .Name or exit: .Name, wherever a built-in animation goes."
outputs: ["website", "app"]
from: ["/docs/guide/motion#declare-your-own-keyframes"]
---
# Declare your own keyframes

When none of the built-in animations moves the way you want, declare
keyframes with `animation Name { }` and use the name where a built-in
animation goes. Here the card drops into place and the badge pulses while
it is shown:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

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

## How it works

- Each step of an `animation` is `from`, `to` or a percentage, and holds
  raw CSS.
- The declaration is written once to the shared stylesheet as
  `@keyframes Name`.
- Your own animation is always written `animate: .Name` (or `exit:
  .Name`). The flags belong to the built-in animations, so a flag of your
  own is an error: `Badge("New").Drop` is `` E103: Badge has no flag or
  enum case `Drop` ``, and `Card.Drop { … }` reads as a part, `` E103:
  `Card.Drop` is not a part of `Card` `` with a `C03` beside it.
- It plays wherever a built-in one would — on an element that stands still,
  from the stylesheet, before any script has loaded — and, like a built-in
  one, not at all for a reader who asked for reduced motion.

## Variations

### Name the keyframes in a style

You can also name your keyframes in a style, as CSS does, to set how often
they repeat. A hand-written `animation:` has no reduced-motion path, so
wrap it in `@media (prefers-reduced-motion: no-preference) { }`
([Respect reduced motion](motion-defaults.md#respect-reduced-motion)):

```wf
animation Pulse {
    from { opacity: 1 }
    50% { opacity: 0.4 }
    to { opacity: 1 }
}

page Live(path: "/", title: "Live", description: "A badge that pulses for as long as it is shown.") {
    Heading("Live").h1
    Badge("Recording") {
        style {
            @media (prefers-reduced-motion: no-preference) {
                animation: Pulse 1s infinite
            }
        }
    }
}
```
