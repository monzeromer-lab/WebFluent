# The WebFluent Guide

WebFluent is a language for websites. A file describes what a page shows —
its elements, the state behind them, what happens when the reader acts — and
the compiler turns it into a static site, a single-page app, a PDF or a slide
deck, with the accessibility, search and performance work done for you.

This guide is the documentation: it takes you from installing `wf` to a
deployed application, and it is the reference once you are there. The same
chapters are published at
[webfluent.monzeromer.dev](https://webfluent.monzeromer.dev).

Every `wf` code block in it is parsed, type-checked and compiled by the test
suite, so what you read is what the compiler accepts.

## Where to start

- **New to WebFluent:** [Introduction](01-introduction.md), then
  [Getting started](02-getting-started.md) and the [tutorial](03-tutorial.md).
- **Coming from a framework:** [Coming from React, Vue or Svelte](04-coming-from.md).
- **Looking something up:** the Reference part — [components](37-components-reference.md),
  [CLI](38-cli.md), [configuration](39-configuration.md), [diagnostics](40-diagnostics.md).
- **Something is wrong:** [Troubleshooting](45-troubleshooting.md).

## Contents

<!-- contents -->
| # | Chapter | What it covers |
|---|---|---|
| | **Start** | |
| 1 | [Introduction](01-introduction.md) | What WebFluent is, what it does for you, where it fits and where it does not, and how to find your way around this guide. |
| 2 | [Getting started](02-getting-started.md) | Install the compiler, make a project, run it, change it and see the change — then set up your editor and see what a build writes. |
| 3 | [Tutorial](03-tutorial.md) | Build a small, complete site — data, components, a store, a form, a theme, a test — and put it on the web. |
| 4 | [Coming from React, Vue or Svelte](04-coming-from.md) | What you already know, mapped to what is here — and the few places WebFluent works differently. |
| | **The basics** | |
| 5 | [Language basics](05-language-basics.md) | The shape of every .wf file: the thirteen kinds of declaration, naming, comments, the two kinds of body, the two layouts and how strings are written. |
| 6 | [Pages and routing](06-pages-and-routing.md) | A page is a route and what it shows. Each page names its own path, the app shell frames every page, and a layout frames a section. |
| 7 | [Elements](07-elements.md) | Everything on screen is an element, and every element is written the same way: a name, one positional value, named props, flags and a block. |
| 8 | [State and reactivity](08-state-and-reactivity.md) | A state is a signal, and only what reads it updates: no tree re-renders, no subscriptions to manage. |
| 9 | [Events](09-events.md) | Handlers on elements, keyboard shortcuts, the events components fire, the overlays a page opens, and drag and drop. |
| 10 | [Control flow](10-control-flow.md) | Five statements decide what a page shows: if, if let, for, show and match. Each adds and removes elements as its condition changes. |
| 11 | [Components](11-components.md) | A component is a reusable element you declare: typed props, events, slots, parts and a body, checked like a built-in. |
| 12 | [Stores](12-stores.md) | A store is state shared by every page and component that uses it, built the first time something reads it. |
| 13 | [Types](13-types.md) | A gradual, structural type checker that changes no output and names every mistake where you wrote it. |
| 14 | [Expressions](14-expressions.md) | The expression language: literals, interpolation, operators, values that choose, calls, methods, formatting, regexes, tokens and await. |
| | **Building** | |
| 15 | [Styling](15-styling.md) | Restyle the whole site with a theme, one element with a style block, and many elements with your own stylesheet — and make the layout responsive. |
| 16 | [Forms and validation](16-forms.md) | Bind controls to state, validate what the reader types, upload files, and show what the server said. |
| 17 | [Data](17-data.md) | Get data onto a page: describe a service once, show a request's states, fetch on demand, and read files at build time. |
| 18 | [Real-time](18-realtime.md) | Hold a connection open — a socket, server-sent events, a channel every tab hears, a line to another reader's page, a worker — and read the network itself. |
| 19 | [Offline](19-offline.md) | Turn on a service worker the build writes for you: the site works with no network, writes wait for the connection, and new versions are offered, not forced. |
| 20 | [Motion](20-motion.md) | Declare how an element enters and leaves, how a list staggers and how pages change — the runtime plays it, or skips it for readers who asked for less. |
| 21 | [Internationalisation](21-i18n.md) | Keep one JSON file per locale, read it with t(), and switch the whole site's language — and its direction — at once. |
| 22 | [Accessibility](22-accessibility.md) | Build pages that work with a keyboard, a screen reader or voice control: what the built-ins do, what the compiler checks, and what is still yours to do. |
| 23 | [Security](23-security.md) | What the compiler guarantees, what it cannot, and the decisions left to you: markup, secrets, storage, headers and a deployment checklist. |
| 24 | [Testing](24-testing.md) | Tests written in WebFluent: render a component and read what it shows, or click through it in a real browser — then check every page in one. |
| | **Shipping** | |
| 25 | [Project structure](25-project-structure.md) | Where files go, how the compiler reads them, how names work across a project, and how a project grows without getting lost. |
| 26 | [Static or single-page](26-static-and-spa.md) | The same source builds as pre-rendered pages or as one app shell. What each gives you, what the pre-rendered page holds, and how dynamic routes are rendered. |
| 27 | [Content](27-content.md) | Markdown, pages written as .md files, code on the page, and what the build writes for search engines and link previews. |
| 28 | [Media](28-media.md) | Add pictures the build resizes for every screen, video with captions, audio with a transcript, icons, carousels and frames. |
| 29 | [Deploying](29-deploying.md) | The build is a folder of static files. Put it on GitHub Pages, Netlify, Vercel, Cloudflare Pages, a server of your own, or a container. |
| 30 | [Environments](30-environments.md) | Set values that change per build (an API address, a feature switch, a public key) from the config, a .env file or the shell, and choose which a page may read. |
| 31 | [Performance](31-performance.md) | Learn what a build already does to be fast, see what it weighs, set budgets that warn, time pages in a browser, and avoid what makes a site slow. |
| 32 | [JavaScript interop](32-javascript-interop.md) | Write plain JavaScript beside your pages and call it by name; load a library from a CDN; hand an element to a script; place custom elements; publish your components. |
| 33 | [Desktop apps](33-desktop-apps.md) | Turn a WebFluent project into a Tauri or Electron desktop app, call the native side from a page, and package it. |
| 34 | [PDF and slides](34-pdf-and-slides.md) | Compile the same language to a paginated PDF document or a slide deck — invoices, reports and talks, laid out with CSS as on the web. |
| 35 | [Server rendering](35-server-rendering.md) | Render a .wf template with JSON on a server — an email, an invoice, a report, an HTML fragment — from the CLI, Rust or Node. |
| 36 | [Tutorials](36-tutorials.md) | Three projects built step by step — a todo app, a statically built notes site, a budget tracker — none needing a server. |
| | **Reference** | |
| 37 | [Components reference](37-components-reference.md) | Every built-in with its props, cases, flags, events, slots and parts — generated from the compiler's own registry. |
| 38 | [Command line and editors](38-cli.md) | Every wf command and flag, the dev server, and the language server in each editor. |
| 39 | [Configuration](39-configuration.md) | Every key of webfluent.app.json, its default, and what it changes. |
| 40 | [Diagnostics](40-diagnostics.md) | Every error and warning the compiler reports — what it means, a program that draws it, the message, and the fix. |
| 41 | [Built-ins](41-built-ins.md) | Every function, browser value, method and global the language gives a program, in one place. |
| 42 | [Design tokens](42-design-tokens.md) | Every token of the baseline theme, with its value: the colours, type, spacing, radii, shadows, motion and breakpoints every built-in is drawn with. |
| 43 | [Runtime API](43-runtime-api.md) | window.WF — the runtime every compiled page calls, usable from a script of your own, the console or a library's callback. |
| 44 | [Grammar](44-grammar.md) | Every keyword and where it may appear, the shapes of declarations, statements and expressions, operators and precedence, and the literals. |
| | **Help** | |
| 45 | [Troubleshooting](45-troubleshooting.md) | The problems people meet, what causes each, and the fix — from installing to deploying, plus the questions that come up most. |
| 46 | [Upgrading](46-upgrading.md) | What changed in each release, what to do when you upgrade, and how wf migrate carries an older project forward. |
| 47 | [Browser support](47-browser-support.md) | Which browsers a WebFluent site targets, and what each newer platform feature falls back to where a browser lacks it. |
| 48 | [Glossary](48-glossary.md) | The words this guide uses, each in a sentence, with where to read more. |
| 49 | [Contributing](49-contributing.md) | How the compiler is put together, how to build and test it, and how to change the language, a built-in or this guide. |
<!-- /contents -->
