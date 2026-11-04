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
- **Looking something up:** the Reference part — [components](40-components-reference.md),
  [CLI](41-cli.md), [configuration](42-configuration.md), [diagnostics](43-diagnostics.md).
- **Something is wrong:** [Troubleshooting](50-troubleshooting.md).

## Contents

<!-- contents -->
| # | Chapter | What it covers |
|---|---|---|
| | **Get started** | |
| 1 | [Introduction](01-introduction.md) | What WebFluent is, what it does for you, where it fits and where it does not, and how to find your way around this guide. |
| 2 | [Getting started](02-getting-started.md) | Install the compiler, make a project, run it, change it and see the change — then set up your editor, or WebFluent Studio, and see what a build writes. |
| 3 | [Tutorial: Shelf](03-tutorial.md) | Build a small, complete site — data, components, a store, a form, a theme, a test — and put it on the web. |
| 4 | [Coming from React, Vue or Svelte](04-coming-from.md) | What you already know, mapped to what is here — and the few places WebFluent works differently. |
| | **Tutorials and recipes** | |
| 5 | [Tutorial: Todos](05-tutorial-todos.md) | Build a todo list kept in the reader's browser — types, a store that remembers, a component, a filter in the address and tests that click. |
| 6 | [Tutorial: Field Notes](06-tutorial-field-notes.md) | Build a statically generated notebook: typed data files, a page per note, Markdown pages, search and tags, and what a pre-rendered page can know. |
| 7 | [Tutorial: Pocket](07-tutorial-pocket.md) | Build a budget tracker: money and dates, derived totals, a storage migration, validated forms, a dialog and shortcuts, two languages and offline. |
| 8 | [Recipes](08-recipes.md) | Small, complete pages for the jobs most projects meet: debounced search, paging, optimistic updates, a confirm dialog, dark mode and more. |
| | **Concepts** | |
| 9 | [Language basics](09-language-basics.md) | The shape of every .wf file: the thirteen kinds of declaration, naming, comments, the two kinds of body, the two layouts and how strings are written. |
| 10 | [Pages and routing](10-pages-and-routing.md) | A page is a route and what it shows. Each page names its own path, the app shell frames every page, and a layout frames a section. |
| 11 | [Elements](11-elements.md) | Everything on screen is an element, and every element is written the same way: a name, one positional value, named props, flags and a block. |
| 12 | [State and reactivity](12-state-and-reactivity.md) | A state is a signal, and only what reads it updates: no tree re-renders, no subscriptions to manage. |
| 13 | [Events](13-events.md) | Handlers on elements, keyboard shortcuts, the events components fire, the overlays a page opens, and drag and drop. |
| 14 | [Control flow](14-control-flow.md) | Five statements decide what a page shows: if, if let, for, show and match. Each adds and removes elements as its condition changes. |
| 15 | [Components](15-components.md) | A component is a reusable element you declare: typed props, events, slots, parts and a body, checked like a built-in. |
| 16 | [Stores](16-stores.md) | A store is state shared by every page and component that uses it, built the first time something reads it. |
| 17 | [Types](17-types.md) | A gradual, structural type checker that changes no output and names every mistake where you wrote it. |
| 18 | [Expressions](18-expressions.md) | The expression language: literals, interpolation, operators, values that choose, calls, methods, formatting, regexes, tokens and await. |
| | **Guides: interface** | |
| 19 | [Styling](19-styling.md) | Restyle the whole site with a theme, one element with a style block, and many elements with your own stylesheet — and make the layout responsive. |
| 20 | [Motion](20-motion.md) | Declare how an element enters and leaves, how a list staggers and how pages change — the runtime plays it, or skips it for readers who asked for less. |
| 21 | [Media](21-media.md) | Add pictures the build resizes for every screen, video with captions, audio with a transcript, icons, carousels and frames. |
| 22 | [Accessibility](22-accessibility.md) | Build pages that work with a keyboard, a screen reader or voice control: what the built-ins do, what the compiler checks, and what is still yours to do. |
| | **Guides: data** | |
| 23 | [Forms and validation](23-forms.md) | Bind controls to state, validate what the reader types, upload files, and show what the server said. |
| 24 | [Fetching data](24-data.md) | Get data onto a page: describe a service once, show a request's states, fetch on demand, and read files at build time. |
| 25 | [Real-time](25-realtime.md) | Hold a connection open — a socket, server-sent events, a channel every tab hears, a line to another reader's page, a worker — and read the network itself. |
| 26 | [Offline](26-offline.md) | Turn on a service worker the build writes for you: the site works with no network, writes wait for the connection, and new versions are offered, not forced. |
| | **Guides: content** | |
| 27 | [Content](27-content.md) | Markdown, pages written as .md files, code on the page, and what the build writes for search engines and link previews. |
| 28 | [Internationalisation](28-i18n.md) | Keep one JSON file per locale, read it with t(), and switch the whole site's language — and its direction — at once. |
| | **Guides: quality** | |
| 29 | [Testing](29-testing.md) | Tests written in WebFluent: render a component and read what it shows, or click through it in a real browser — then check every page in one. |
| 30 | [Security](30-security.md) | What the compiler guarantees, what it cannot, and the decisions left to you: markup, secrets, storage, headers and a deployment checklist. |
| 31 | [Performance](31-performance.md) | Learn what a build already does to be fast, see what it weighs, set budgets that warn, time pages in a browser, and avoid what makes a site slow. |
| | **Guides: shipping** | |
| 32 | [Project structure](32-project-structure.md) | Where files go, how the compiler reads them, how names work across a project, and how a project grows without getting lost. |
| 33 | [Static or single-page](33-static-and-spa.md) | The same source builds as pre-rendered pages or as one app shell. What each gives you, what the pre-rendered page holds, and how dynamic routes are rendered. |
| 34 | [Deploying](34-deploying.md) | The build is a folder of static files. Put it on GitHub Pages, Netlify, Vercel, Cloudflare Pages, a server of your own, or a container. |
| 35 | [Environments](35-environments.md) | Set values that change per build (an API address, a feature switch, a public key) from the config, a .env file or the shell, and choose which a page may read. |
| 36 | [Desktop apps](36-desktop-apps.md) | Turn a WebFluent project into a Tauri or Electron desktop app, call the native side from a page, and package it. |
| | **Other outputs** | |
| 37 | [PDF and slides](37-pdf-and-slides.md) | Compile the same language to a paginated PDF document or a slide deck — invoices, reports and talks, laid out with CSS as on the web. |
| 38 | [Server rendering](38-server-rendering.md) | Render a .wf template with JSON on a server — an email, an invoice, a report, an HTML fragment — from the CLI, Rust or Node. |
| 39 | [JavaScript interop](39-javascript-interop.md) | Write plain JavaScript beside your pages and call it by name; load a library from a CDN; hand an element to a script; place custom elements; publish your components. |
| | **Reference** | |
| 40 | [Components reference](40-components-reference.md) | Every built-in with its props, cases, flags, events, slots and parts — generated from the compiler's own registry. |
| 41 | [Command line and editors](41-cli.md) | Every wf command and flag, the dev server, and the language server in each editor. |
| 42 | [Configuration](42-configuration.md) | Every key of webfluent.app.json, its default, and what it changes. |
| 43 | [Diagnostics](43-diagnostics.md) | Every error and warning the compiler reports — what it means, a program that draws it, the message, and the fix. |
| 44 | [Built-ins](44-built-ins.md) | Every function, browser value, method and global the language gives a program, in one place. |
| 45 | [Design tokens](45-design-tokens.md) | Every token of the baseline theme, with its value: the colours, type, spacing, radii, shadows, motion and breakpoints every built-in is drawn with. |
| 46 | [Runtime API](46-runtime-api.md) | window.WF — the runtime every compiled page calls, usable from a script of your own, the console or a library's callback. |
| 47 | [Grammar](47-grammar.md) | Every keyword and where it may appear, the shapes of declarations, statements and expressions, operators and precedence, and the literals. |
| 48 | [Browser support](48-browser-support.md) | Which browsers a WebFluent site targets, and what each newer platform feature falls back to where a browser lacks it. |
| 49 | [Glossary](49-glossary.md) | The words this guide uses, each in a sentence, with where to read more. |
| | **Help** | |
| 50 | [Troubleshooting](50-troubleshooting.md) | The problems people meet, what causes each, and the fix — from installing to deploying, plus the questions that come up most. |
| 51 | [Upgrading](51-upgrading.md) | What changed in each release, what to do when you upgrade, and how wf migrate carries an older project forward. |
| 52 | [Contributing](52-contributing.md) | How the compiler is put together, how to build and test it, and how to change the language, a built-in or this guide. |
<!-- /contents -->
