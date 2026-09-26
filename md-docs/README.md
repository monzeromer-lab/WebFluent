# The WebFluent Guide

WebFluent is a language for websites. A file describes what a page shows —
its elements, the state behind them, what happens when the reader acts — and
the compiler turns it into a static site, a single-page app, a PDF or a slide
deck, with the accessibility, search and performance work done for you.

This guide is the documentation: it takes you from installing `wf` to a
deployed application, and it is the reference once you are there. The same
chapters are published at
[monzeromer-lab.github.io/WebFluent](https://monzeromer-lab.github.io/WebFluent).

Every `wf` code block in it is parsed, type-checked and compiled by the test
suite, so what you read is what the compiler accepts.

## Where to start

- **New to WebFluent:** [Introduction](01-introduction.md), then
  [Getting started](02-getting-started.md) and the [tutorial](03-tutorial.md).
- **Coming from a framework:** [Coming from React, Vue or Svelte](04-coming-from.md).
- **Looking something up:** the Reference part — [components](36-components-reference.md),
  [CLI](37-cli.md), [configuration](38-configuration.md), [diagnostics](39-diagnostics.md).
- **Something is wrong:** [Troubleshooting](44-troubleshooting.md).

## Contents

<!-- contents -->
| # | Chapter | What it covers |
|---|---|---|
| | **Start** | |
| 1 | [Introduction](01-introduction.md) | What WebFluent is, what it does for you, where it fits and where it does not, and how to read this guide. |
| 2 | [Getting started](02-getting-started.md) | Install the compiler and your editor's support, make a project, and see what a build writes. |
| 3 | [Tutorial](03-tutorial.md) | Build a small, complete site — data, components, a store, a form, a theme, a test — and put it on the web. |
| 4 | [Coming from React, Vue or Svelte](04-coming-from.md) | What you already know, mapped to what is here — and the few places WebFluent works differently. |
| | **The basics** | |
| 5 | [Language basics](05-language-basics.md) | A file is a list of declarations. This chapter covers all fourteen kinds, naming, comments, bodies, strings and the two layouts. |
| 6 | [Pages and routing](06-pages-and-routing.md) | A page is a route and what it shows. Pages own their paths; the app shell frames them; a layout wraps them. |
| 7 | [Elements](07-elements.md) | Everything on screen is an element: a call with one positional value, named props, flags, and a block. |
| 8 | [State and reactivity](08-state-and-reactivity.md) | Reactivity here is fine-grained: a state is a signal, and only the thing that read it updates. |
| 9 | [Events](09-events.md) | Handlers on elements, keyboard shortcuts, the events components fire, and the overlays a page opens. |
| 10 | [Control flow](10-control-flow.md) | Five statements decide what shows: if, if let, for, show and match. Each creates and removes elements as its condition changes. |
| 11 | [Components](11-components.md) | A component is a reusable element you declare: typed props, events, slots, parts and a body, checked like a built-in. |
| 12 | [Stores](12-stores.md) | A store is state shared by every page and component that uses it, built the first time something reads it. |
| 13 | [Types](13-types.md) | A gradual, structural type checker that changes no output and names every mistake with a hint. |
| 14 | [Expressions](14-expressions.md) | The whole expression language: literals, interpolation, operators, values that choose, calls, methods, formatting and await. |
| | **Building** | |
| 15 | [Styling](15-styling.md) | Every built-in ships styled; a theme changes the whole site; a style block is real CSS scoped to its element. |
| 16 | [Forms and validation](16-forms.md) | Controls bound to state, forms with a handle, validation rules that show their own messages, files, and what the server said. |
| 17 | [Data](17-data.md) | Getting data onto a page: a service described once, a resource a page shows, a fetch an action makes, and files read at build time. |
| 18 | [Real-time](18-realtime.md) | Sockets, server-sent events, a channel every tab hears, a direct line to another reader's page — and the network itself as a value. |
| 19 | [Offline](19-offline.md) | A service worker the build writes for you: the site keeps working with no network, writes wait for the connection, and a new version is offered, not forced. |
| 20 | [Motion](20-motion.md) | Motion is declared, not scripted: how an element enters and leaves, how a list staggers, how pages change. |
| 21 | [Internationalisation](21-i18n.md) | One JSON file per locale, t() to read it, and a switch that flips the whole site — direction included. |
| 22 | [Accessibility](22-accessibility.md) | What the built-ins do for readers who use a keyboard, a screen reader or voice control, what the compiler checks on every build, and what is still yours to do. |
| 23 | [Security](23-security.md) | What the compiler guarantees, what it cannot, and the decisions left to you: markup, secrets, storage, headers and a deployment checklist. |
| 24 | [Testing](24-testing.md) | Tests written in WebFluent: render a component and read what it shows, or click through it in a real browser — then check every page in one. |
| | **Shipping** | |
| 25 | [Project structure](25-project-structure.md) | Where files go, how the compiler reads them, how names work across a project, and how a project grows without getting lost. |
| 26 | [Static or single-page](26-static-and-spa.md) | The same source builds as pre-rendered pages or as one app shell. What each gives you, what the static paint knows, and how dynamic routes are rendered. |
| 27 | [Content](27-content.md) | Markdown, pages written as .md files, code on the page, and what the build writes for search engines and link previews. |
| 28 | [Media](28-media.md) | Pictures the build resizes for every screen, video with captions, audio with a transcript, icons and carousels. |
| 29 | [Deploying](29-deploying.md) | The build is a folder of static files. Here is how to put it on GitHub Pages, Netlify, Vercel, Cloudflare Pages, a server of your own, or a container. |
| 30 | [Environments](30-environments.md) | Values fixed per build — an API address, a feature switch, a public key — from the config, a .env file or the shell, and which of them a page may read. |
| 31 | [Performance](31-performance.md) | What a build already does to be fast, how to see what it weighs, budgets that warn, and the few things that make a site slow. |
| 32 | [JavaScript interop](32-javascript-interop.md) | Use a JavaScript library — a chart, a map, an editor — with a typed declaration and a node with a lifetime; place other people's custom elements; publish your components for any framework. |
| 33 | [PDF and slides](33-pdf-and-slides.md) | The same language, compiled to a paginated PDF document or a slide deck — invoices, reports and talks. |
| 34 | [Server rendering](34-server-rendering.md) | Render a .wf template with JSON on a server — an email, an invoice, a report, an HTML fragment — from the CLI, Rust or Node. |
| 35 | [Cookbook](35-recipes.md) | Three complete applications you can paste into a fresh project, and recipes for the things every site needs. |
| | **Reference** | |
| 36 | [Components reference](36-components-reference.md) | Every built-in with its props, cases, flags, events, slots and parts — generated from the compiler's own registry. |
| 37 | [Command line and editors](37-cli.md) | Every wf command and flag, the dev server, and the language server in each editor. |
| 38 | [Configuration](38-configuration.md) | Every key of webfluent.app.json, its default, and what it changes. |
| 39 | [Diagnostics](39-diagnostics.md) | Every error and warning the compiler reports — what it means, a program that draws it, the message, and the fix. |
| 40 | [Built-ins](40-built-ins.md) | Every function, browser value, method and global the language gives a program, in one place. |
| 41 | [Design tokens](41-design-tokens.md) | Every token of the baseline theme, with its value: the colours, type, spacing, radii, shadows, motion and breakpoints every built-in is drawn with. |
| 42 | [Runtime API](42-runtime-api.md) | window.WF — the runtime every compiled page calls, usable from a script of your own, the console or a library's callback. |
| 43 | [Grammar](43-grammar.md) | Every keyword and where it may appear, the shapes of declarations, statements and expressions, operators and precedence, and the literals. |
| | **Help** | |
| 44 | [Troubleshooting](44-troubleshooting.md) | The problems people meet, what causes each, and the fix — from installing to deploying, plus the questions that come up most. |
| 45 | [Upgrading](45-upgrading.md) | What changed in each release, what to do when you upgrade, and how wf migrate carries an older project forward. |
| 46 | [Browser support](46-browser-support.md) | Which browsers a WebFluent site targets, and what each newer platform feature falls back to where it is missing. |
| 47 | [Glossary](47-glossary.md) | The words this guide uses, each in a sentence, with where to read more. |
| 48 | [Contributing](48-contributing.md) | How the compiler is put together, how to build and test it, and how to change the language, a built-in or this guide. |
<!-- /contents -->
