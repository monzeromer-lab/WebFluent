---
chapter: 39
kind: "reference"
route: "/reference/browser-support"
group: "reference"
blurb: "Which browsers a WebFluent site targets, and what each newer platform feature falls back to where a browser lacks it."
description: "The browsers a WebFluent site targets, and the fallback for each newer platform feature it uses where a browser lacks it."
---
# Browser support

This page lists the browsers a WebFluent site targets, and what each newer
platform feature the runtime uses falls back to where a browser lacks it.

## Supported browsers

A WebFluent site targets the current versions of **Chrome, Edge, Firefox
and Safari**, on desktop and mobile — the browsers that update themselves.

- **The output** is plain HTML, CSS and modern JavaScript (ES2021). There
  is no polyfill and no legacy build.
- **Without JavaScript**, a static build's pages are readable: the HTML the
  build pre-renders for each page carries its text
  ([Static sites and SPAs](../about/static-and-spa.md)). Everything interactive
  needs JavaScript.

> [!TIP]
> `wf verify` loads every page of a build in a headless Chrome and reports
> uncaught errors, failed requests and pages that drew no text
> ([CLI](cli.md)).

## Features with a fallback

Where a browser lacks one of these features, the site still works; the
last column says what changes.

| Feature | Used for | Where it is missing |
|---|---|---|
| View Transitions API | `Router(transition: …)`, `shared:` elements | The class-based transition plays instead; `shared:` does nothing |
| `linear()` easing | `easing: .spring`, `.bouncy` | The browser ignores it and uses its default easing |
| Web Animations API | enter, exit, list moves, `count:` | Present in every current browser; without it, elements appear and leave without motion |
| `IntersectionObserver` | `on: .enterView` | The animation plays at once |
| Background Sync | writes kept offline, sent after the tab closes | The page sends them the next time it is open and online |
| Service workers | `offline` | The site works online as usual |
| WebRTC | `peer` | The peer reads `error`: "WebRTC is not available in this browser" |
| `BroadcastChannel` | `channel` | The channel reads `closed` |
| Network Information API | `network.effectiveType`, `.saveData`, `.downlink` | They read as a fast line: `"4g"`, `false` and `10`. `network.online` works everywhere |
| `Intl` | `format`, `ago`, plural forms | The build's own table for the common locales |
| Container queries | `@container` in a style block | Passed through as written; the rule does not apply |
| `:focus-visible` | focus rings | Focus rings on every focus |

## Dialogs and forms

`Modal` and `Dialog` use the `<dialog>` element and `showModal()`, which
every current browser has. Form validation uses the browser's constraint
validation alongside the language's own rules
([Forms](../learn/14-forms.md)).

## Next

[Glossary](glossary.md).
