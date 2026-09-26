# 46. Browser support

<!--
route: guide/browser-support
group: help
blurb: Which browsers a WebFluent site targets, and what each newer platform feature falls back to where it is missing.
description: The browsers a WebFluent site targets, and the fallback for each newer platform feature it uses where a browser lacks it.
-->

A WebFluent site targets the current versions of **Chrome, Edge, Firefox
and Safari**, on desktop and mobile — the browsers that update themselves.
The output is plain HTML, CSS and modern JavaScript (ES2020); there is no
polyfill and no legacy build.

A static build's pages are readable with no JavaScript at all; everything
interactive needs it.

## Features with a fallback

| Feature | Used for | Where it is missing |
|---|---|---|
| View Transitions API | `Router(transition: …)`, `shared:` elements | The class-based transition plays instead; `shared:` does nothing |
| `linear()` easing | `easing: .spring`, `.bouncy` | The browser ignores it and uses its default easing |
| Web Animations API | enter, exit, list moves, `count:` | Everywhere current; without it, elements appear and leave without motion |
| `IntersectionObserver` | `on: .enterView` | The animation plays at once |
| Background Sync | writes kept offline, sent after the tab closes | The page sends them the next time it is open and online |
| Service workers | `offline` | The site works online as usual |
| WebRTC | `peer` | The peer reads `error`: "WebRTC is not available in this browser" |
| `BroadcastChannel` | `channel` | The channel reads `closed` |
| Network Information API | `network.effectiveType`, `.saveData`, `.downlink` | `null`; `network.online` works everywhere |
| `Intl` | `format`, `ago`, plural forms | The build's own table for the common locales |
| Container queries | `@container` in a style block | Passed through as written; the rule does not apply |
| `:focus-visible` | focus rings | Focus rings on every focus |

## Dialogs and forms

`Modal` and `Dialog` use the `<dialog>` element and `showModal()`, which
every current browser has. Validation uses the browser's constraint
validation alongside the language's rules.

## Next

[Glossary](47-glossary.md).
