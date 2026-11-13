---
kind: "recipe"
route: "/recipes/motion-defaults"
group: "look"
description: "To change what an animation does when the element names no duration or easing, set `motion` in `webfluent.app.json`:"
from: ["/docs/guide/motion#set-the-default-duration-and-easing", "/docs/guide/motion#respect-reduced-motion"]
---
# Set motion defaults, and respect reduced motion

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
