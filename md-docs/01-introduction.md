# 1. Introduction

<!--
route: guide/introduction
group: start
blurb: What WebFluent is, what it does for you, where it fits and where it does not, and how to read this guide.
description: What WebFluent is, the five ideas behind it, when to use it and when not to, and a map of the guide.
-->

WebFluent is a programming language for websites. You write `.wf` files that
say what each page shows, the state behind it, and what happens when the
reader acts; the compiler, `wf`, turns them into a finished site — HTML, CSS
and a small JavaScript runtime — or into a PDF or a slide deck from the same
source.

```wf
page Home(path: "/", title: "Hello", description: "A first WebFluent page.") {
    state count = 0
    Container {
        Heading("Hello, world").h1
        Text("You have clicked {count} times.")
        Button("Click me").primary { on click { count = count + 1 } }
    }
}
```

That file is a complete, deployable page: routing, reactivity, a styled
button, accessible markup, the `<title>`, the meta description and the Open
Graph tags are all there after `wf build`.

## What you get

- **One binary, no toolchain.** `wf` is the compiler, the dev server, the
  formatter, the test runner and the language server. There is no
  `node_modules`, no bundler and no config to write before the first page.
- **A compiler that checks your work.** Every built-in element and every
  component you declare has a known set of props, flags, events and slots.
  A misspelled prop, a flag the element does not take, a field a record does
  not have or a value that may be `null` read as if it were not is an error
  with a line number and a hint — never a silent no-op in the browser.
- **Fine-grained reactivity.** `state` is a signal; the one text node or
  attribute that reads it updates, and nothing else re-renders.
- **Fifty-odd designed components.** Layout, navigation, tables, forms,
  overlays, media — styled by design tokens a theme changes in a few lines.
- **The web done properly by default.** Pre-rendered HTML per page, one
  stylesheet pruned to what you used, a runtime cut down to what the program
  reaches, image pipelines, sitemaps, canonical links, Open Graph and JSON-LD,
  a Content-Security-Policy the build holds its own output to, and
  accessibility lints on every build.
- **Batteries.** Routing, stores, forms and validation, typed HTTP services,
  WebSockets and WebRTC, offline support with a service worker, i18n with
  automatic right-to-left, animations and dark mode.
- **More than websites.** The same language compiles to PDF documents and
  slide decks, renders templates with JSON on a server, and publishes
  components as custom elements any framework can use.

## The five ideas

1. **A file is a list of declarations.** `page`, `component`, `store`,
   `theme`, `app`, `type`, `enum`, `api`, `external`, `const`, `data`,
   `image`, `animation` and `test` — fourteen kinds, each covered in this
   guide. Pages own their routes; there is no separate route table.
2. **An element is a typed call.** `Button("Save", tone: .primary).lg { on click { save() } }`:
   one positional argument, named props, `.flag`s, then a block. The compiler
   knows every built-in's props and cases, and every component you declare.
3. **State is a signal.** `state count = 0`; anything that reads `count`
   follows it. `derived` values recompute, `effect`s re-run, the DOM updates.
4. **Styles are CSS, written where the element is.**
   `style { padding: $md; &:hover { background: $surface-hover } }` — raw
   values, design tokens with `$`, nested rules as CSS nesting spells them.
5. **The build is the whole site.** `wf build` writes everything a host
   needs, and `wf serve` rebuilds it on every save.

## When to use it — and when not

WebFluent fits well when you want:

- a marketing site, a documentation site, a blog or a portfolio that must be
  fast, accessible and well indexed, with some interactivity;
- a dashboard, an admin panel or an internal tool that talks to an API;
- invoices, reports or slide decks generated from the same data as a site;
- a small team that would rather not maintain a JavaScript build chain.

Reach for something else when:

- you depend on a large ecosystem of framework-specific UI libraries (you
  can still embed any library through [JavaScript interop](32-javascript-interop.md),
  but you write the binding yourself);
- you need server-side rendering on every request with per-user data — a
  WebFluent site is static files plus a browser runtime; your server stays
  your server (though [server rendering](34-server-rendering.md) of templates
  covers emails, invoices and fragments);
- you need a native mobile app.

## How to read this guide

The guide is in six parts:

| Part | Chapters | For |
|---|---|---|
| **Start** | 1–4 | Installing, a first project, a tutorial that ends in a deployed site, and a map from React, Vue and Svelte |
| **The basics** | 5–14 | The language itself, in the order the ideas build on each other |
| **Building** | 15–24 | Styling, forms, data, real-time, offline, motion, i18n, accessibility, security and testing |
| **Shipping** | 25–35 | Structuring a project, static or single-page, content and SEO, media, deploying, environments, performance, JavaScript interop, PDFs and slides, server rendering, and recipes |
| **Reference** | 36–43 | Every component, command, config key, diagnostic, built-in, design token, runtime function and keyword |
| **Help** | 44–48 | Troubleshooting, upgrading, browser support, a glossary and contributing |

- **New to WebFluent:** read [Getting started](02-getting-started.md) and
  work through the [tutorial](03-tutorial.md), then read chapters 5–12 in
  order. Every example in them builds.
- **Coming from a framework:** read [Coming from React, Vue or Svelte](04-coming-from.md)
  first; it maps what you know to what is here.
- **Looking something up:** the [reference](36-components-reference.md)
  chapters are generated from, or checked against, the compiler itself.
- **Something is wrong:** start at [Troubleshooting](44-troubleshooting.md);
  every diagnostic code has an entry in [Diagnostics](39-diagnostics.md).

Every `wf` code block in this guide is parsed, type-checked and compiled by
the test suite, so what you read is what the compiler accepts. A block that
leaves something out says so with a `…`.

## Versions

This guide describes WebFluent 4. Features marked **4.1** are in the next
release; `wf --version` tells you which one you have, and
[Upgrading](45-upgrading.md) lists what changed in each release.

## Next

[Getting started](02-getting-started.md).
