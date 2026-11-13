---
chapter: 18
kind: "about"
route: "/about/why"
from: ["/docs/guide/introduction"]
group: "about"
blurb: "What WebFluent is, what it does for you, where it fits and where it does not, and how to find your way around this guide."
description: "What WebFluent is, the five ideas behind it, when to use it and when not to, and how the guide is organised by what you need."
---
# Introduction

WebFluent is a programming language for websites, PDFs and slide decks. You
write `.wf` files that say what each page shows, the state behind it, and
what happens when the reader acts. The compiler, `wf`, turns those files
into a finished site — HTML, CSS and a small JavaScript runtime — or into a
PDF or a slide deck from the same source.

The one thing to remember: the compiler knows every element, prop and type
in your program, so most mistakes stop the build with a line number instead
of failing quietly in the browser.

In this chapter:

- [A first page](#a-first-page), and what each line of it does
- [What you get](#what-you-get) from the compiler
- [The five ideas](#the-five-ideas) the language is built on
- [When to use WebFluent](#when-to-use-webfluent), and when to use something else
- [How this guide is organised](#how-this-guide-is-organised), and where to start

## A first page

This file is a complete page with a counter:

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

- `page Home(path: "/", …)` declares a page and the address it answers.
  `title` and `description` become the browser tab's title, the search
  snippet and the link-preview cards.
- `state count = 0` declares a value that can change. Everything that reads
  it updates when it does.
- `Container`, `Heading`, `Text` and `Button` are built-in elements. `.h1`
  and `.primary` are *flags*: short switches that pick a heading level or a
  button's colour.
- `on click { … }` is what happens when the reader clicks the button.

After `wf build`, the page is ready to deploy. Routing, reactivity, a styled
button, accessible markup, the `<title>`, the meta description and the Open
Graph tags are all in the output; you wrote none of them by hand.

## What you get

- **One binary, no toolchain.** `wf` is the compiler, the dev server, the
  formatter, the test runner and the language server. There is no
  `node_modules`, no bundler and no configuration to write before the first
  page.
- **A compiler that checks your work.** Every built-in element and every
  component you declare has a known set of props, flags, events and slots.
  A misspelled prop, a flag the element does not take, a field a record
  does not have, or a value that may be `null` read as if it were not — each
  is an error with a line number and a hint, never a silent no-op in the
  browser.
- **Fine-grained reactivity.** A `state` is a *signal*: a value that tells
  its readers when it changes. Only the text node or attribute that reads it
  updates; nothing else re-renders.
- **More than fifty designed components.** Layout, navigation, tables,
  forms, overlays and media, styled by *design tokens* (named values for
  colours, spacing and fonts) that a theme changes in a few lines.
- **The web done properly by default.** Each page is pre-rendered to HTML.
  You get one stylesheet pruned to what you used, a runtime cut down to what
  the program reaches, an image pipeline, a sitemap, canonical links, Open
  Graph and JSON-LD, a Content-Security-Policy that the build holds its own
  output to, and accessibility checks on every build.
- **Everything a site needs, built in.** Routing, stores, forms and
  validation, typed HTTP services, WebSockets and WebRTC, offline support
  with a service worker, translation with automatic right-to-left layout,
  animation and dark mode.
- **More than websites.** The same language compiles to PDF documents and
  slide decks, renders templates with JSON data on a server, and publishes
  components as custom elements that any framework can use.

## The five ideas

1. **A project is a set of declarations.** Every `.wf` file holds
   declarations of thirteen kinds: `page`, `component`, `store`, `theme`,
   `app`, `type`, `enum`, `api`, `const`, `data`, `image`, `animation` and
   `test`. This guide covers each one. A `.js` or `.css` file under `src/`
   is part of the project too. Each page names its own route, so there is no
   separate route table.
2. **An element is a typed call.** In
   `Button("Save", tone: .primary).lg { on click { save() } }` you see the
   parts in order: one positional argument, named props, `.flag`s, then a
   block. The compiler knows the props and cases of every built-in and of
   every component you declare.
3. **State is a signal.** After `state count = 0`, anything that reads
   `count` follows it: `derived` values recompute, `effect`s run again, and
   the page updates.
4. **Styles are CSS, written on the element.**
   `style { padding: $md; &:hover { background: $surface } }` holds raw CSS
   values, design tokens written with `$`, and nested rules in CSS nesting
   syntax.
5. **The build is the whole site.** `wf build` writes everything a host
   needs, and `wf serve` rebuilds it every time you save.

## When to use WebFluent

WebFluent fits well when you want:

- a marketing site, a documentation site, a blog or a portfolio that must be
  fast, accessible and well indexed, with some interactivity;
- a dashboard, an admin panel or an internal tool that talks to an API;
- invoices, reports or slide decks generated from the same data as a site;
- to work in a small team that would rather not maintain a JavaScript build
  chain.

Use something else when:

- you depend on a large ecosystem of framework-specific UI libraries. You
  can still embed any library through
  [JavaScript interop](../recipes/index.md), but you write the binding
  yourself.
- you need server-side rendering on every request with per-user data. A
  WebFluent site is static files plus a browser runtime, and your server
  stays your server. [Server rendering](../recipes/index.md) of templates
  does cover emails, invoices and HTML fragments.
- you need a native mobile app.

## How this guide is organised

Each page of the guide is one of four kinds, and each kind answers a
different need:

| Kind | Read it when you… | It gives you |
|---|---|---|
| **Tutorial** | are learning, and want to build something | a lesson with a guaranteed working result |
| **How-to guide** | have a goal, such as validating a form | the steps to that one result |
| **Reference** | need a fact, such as a prop's default | a complete, exact description |
| **Explanation** | want to understand how something works and why | the ideas, the reasons and the trade-offs |

The guide is organised by what you are here to do, in eleven sections:

| Section | Kind | What it covers |
|---|---|---|
| **Get started** | explanation, quick start, tutorial | What WebFluent is, installing it, a first site built step by step, and a map from React, Vue and Svelte |
| **Tutorials and recipes** | tutorial, then how-to | Three whole projects — [Todos](../learn/03-tutorial-todos.md), [Field Notes](../recipes/build-a-notebook-site.md) and [Pocket](../recipes/build-a-budget-app.md) — and [recipes](../recipes/index.md) for the pieces most projects need |
| **Concepts** | explanation, each ending in a quick-reference table | The language itself, in the order the ideas build on each other: from [Language basics](../learn/04-language-basics.md) to [Expressions](../reference/expressions.md) |
| **Guides: interface** | how-to | [Styling](../learn/13-styling.md), [motion](../recipes/index.md), [media](../recipes/index.md) and [accessibility](accessibility.md) |
| **Guides: data** | how-to | [Forms](../learn/14-forms.md), [fetching data](../learn/15-data.md), [real-time connections](../recipes/index.md) and [offline](../recipes/index.md) |
| **Guides: content** | how-to | [Content and search](../recipes/index.md) and [translation](../recipes/index.md) |
| **Guides: quality** | how-to | [Testing](../learn/16-testing.md), [security](security.md) and [performance](performance.md) |
| **Guides: shipping** | explanation, then how-to | [Project structure](../reference/project-structure.md), [static or single-page](static-and-spa.md), [deploying](../recipes/index.md), [environments](../recipes/index.md) and [desktop apps](../recipes/index.md) |
| **Other outputs** | how-to | [PDF and slides](../learn/17-pdf-and-slides.md), [server rendering](../recipes/index.md) and [JavaScript interop](../recipes/index.md) |
| **Reference** | reference | Every [component](../reference/components.md), [command](../reference/cli.md), [configuration key](../reference/configuration.md), [diagnostic](../errors/diagnostics.md), [built-in](../reference/built-ins.md), [design token](../reference/design-tokens.md), [runtime function](../reference/runtime-api.md) and [grammar rule](../reference/grammar.md), plus [browser support](../reference/browser-support.md) and a [glossary](../reference/glossary.md) |
| **Help** | troubleshooting and upgrade notes | [Troubleshooting](../errors/troubleshooting.md), [upgrading](upgrading.md) and [contributing](contributing.md) |

## Where to start

- **To learn WebFluent:** read [Getting started](../learn/01-getting-started.md),
  work through the [tutorial](../learn/02-tutorial-shelf.md), then read the *Concepts* section
  in order. Every example in them builds. For more practice, try the
  [three project tutorials](../learn/03-tutorial-todos.md).
- **To understand it coming from a framework:** read
  [Coming from React, Vue or Svelte](../coming-from/react.md) first. It maps what
  you know to what is here.
- **To build a feature:** go to the *Guides* page that names the task.
- **To look something up:** use the reference chapters, starting with the
  [components reference](../reference/components.md). They are generated
  from the compiler, or checked against it.
- **To fix a problem:** start at [Troubleshooting](../errors/troubleshooting.md).
  Every diagnostic code has an entry in [Diagnostics](../errors/diagnostics.md).

The test suite parses, type-checks and compiles every `wf` code block in
this guide, so what you read is what the compiler accepts. A block that
leaves something out says so with a `…`.

## Versions

This guide describes WebFluent 6, the stable line. Run `wf --version` to see
which release you have. [Upgrading](upgrading.md) lists what changed in
each release.

## Next

[Getting started](../learn/01-getting-started.md): install `wf` and run your first
project.
