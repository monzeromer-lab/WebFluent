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

- **New to WebFluent:** [Introduction](about/introduction.md), then
  [Getting started](learn/01-getting-started.md) and the [tutorial](learn/02-tutorial-shelf.md).
- **Coming from a framework:** [Coming from React, Vue or Svelte](coming-from/react-vue-svelte.md).
- **Looking something up:** the Reference part — [components](reference/components.md),
  [CLI](reference/cli.md), [configuration](reference/configuration.md), [diagnostics](errors/diagnostics.md).
- **Something is wrong:** [Troubleshooting](errors/troubleshooting.md).

## Contents

<!-- contents -->
| # | Chapter | What it covers |
|---|---|---|
| | **Get started** | |
| 1 | [Introduction](about/introduction.md) | What WebFluent is, what it does for you, where it fits and where it does not, and how to find your way around this guide. |
| 2 | [Getting started](learn/01-getting-started.md) | Install the compiler, make a project, run it, change it and see the change — then set up your editor, or WebFluent Studio, and see what a build writes. |
| 3 | [Tutorial: Shelf](learn/02-tutorial-shelf.md) | Build a small, complete site — data, components, a store, a form, a theme, a test — and put it on the web. |
| 4 | [Coming from React, Vue or Svelte](coming-from/react-vue-svelte.md) | What you already know, mapped to what is here — and the few places WebFluent works differently. |
| | **Tutorials and recipes** | |
| 5 | [Tutorial: Todos](learn/03-tutorial-todos.md) | Build a todo list kept in the reader's browser — types, a store that remembers, a component, a filter in the address and tests that click. |
| | **Concepts** | |
| 9 | [Language basics](learn/04-language-basics.md) | The shape of every .wf file: the thirteen kinds of declaration, naming, comments, the two kinds of body, the two layouts and how strings are written. |
| 10 | [Pages and routing](learn/05-pages-and-routing.md) | A page is a route and what it shows. Each page names its own path, the app shell frames every page, and a layout frames a section. |
| 11 | [Elements](learn/06-elements.md) | Everything on screen is an element, and every element is written the same way: a name, one positional value, named props, flags and a block. |
| 12 | [State and reactivity](learn/07-state-and-reactivity.md) | A state is a signal, and only what reads it updates: no tree re-renders, no subscriptions to manage. |
| 13 | [Events](learn/08-events.md) | Handlers on elements, keyboard shortcuts, the events components fire, the overlays a page opens, and drag and drop. |
| 14 | [Control flow](learn/09-control-flow.md) | Five statements decide what a page shows: if, if let, for, show and match. Each adds and removes elements as its condition changes. |
| 15 | [Components](learn/10-components.md) | A component is a reusable element you declare: typed props, events, slots, parts and a body, checked like a built-in. |
| 16 | [Stores](learn/11-stores.md) | A store is state shared by every page and component that uses it, built the first time something reads it. |
| 17 | [Types](learn/12-types.md) | A gradual, structural type checker that changes no output and names every mistake where you wrote it. |
| 18 | [Expressions](reference/expressions.md) | The expression language: literals, interpolation, operators, values that choose, calls, methods, formatting, regexes, tokens and await. |
| | **Guides: interface** | |
| 19 | [Styling](learn/13-styling.md) | Restyle the whole site with a theme, one element with a style block, and many elements with your own stylesheet — and make the layout responsive. |
| 22 | [Accessibility](about/accessibility.md) | Build pages that work with a keyboard, a screen reader or voice control: what the built-ins do, what the compiler checks, and what is still yours to do. |
| | **Guides: data** | |
| 23 | [Forms and validation](learn/14-forms.md) | Bind controls to state, validate what the reader types, upload files, and show what the server said. |
| 24 | [Fetching data](learn/15-data.md) | Get data onto a page: describe a service once, show a request's states, fetch on demand, and read files at build time. |
| | **Guides: quality** | |
| 29 | [Testing](learn/16-testing.md) | Tests written in WebFluent: render a component and read what it shows, or click through it in a real browser — then check every page in one. |
| 30 | [Security](about/security.md) | What the compiler guarantees, what it cannot, and the decisions left to you: markup, secrets, storage, headers and a deployment checklist. |
| 31 | [Performance](about/performance.md) | Learn what a build already does to be fast, see what it weighs, set budgets that warn, time pages in a browser, and avoid what makes a site slow. |
| | **Guides: shipping** | |
| 32 | [Project structure](reference/project-structure.md) | Where files go, how the compiler reads them, how names work across a project, and how a project grows without getting lost. |
| 33 | [Static or single-page](about/static-and-spa.md) | The same source builds as pre-rendered pages or as one app shell. What each gives you, what the pre-rendered page holds, and how dynamic routes are rendered. |
| | **Other outputs** | |
| 37 | [PDF and slides](learn/17-pdf-and-slides.md) | Compile the same language to a paginated PDF document or a slide deck — invoices, reports and talks, laid out with CSS as on the web. |
| | **Reference** | |
| 40 | [Components reference](reference/components.md) | Every built-in with its props, cases, flags, events, slots and parts — generated from the compiler's own registry. |
| 41 | [Command line and editors](reference/cli.md) | Every wf command and flag, the dev server, and the language server in each editor. |
| 42 | [Configuration](reference/configuration.md) | Every key of webfluent.app.json, its default, and what it changes. |
| 43 | [Diagnostics](errors/diagnostics.md) | Every error and warning the compiler reports — what it means, a program that draws it, the message, and the fix. |
| 44 | [Built-ins](reference/built-ins.md) | Every function, browser value, method and global the language gives a program, in one place. |
| 45 | [Design tokens](reference/design-tokens.md) | Every token of the baseline theme, with its value: the colours, type, spacing, radii, shadows, motion and breakpoints every built-in is drawn with. |
| 46 | [Runtime API](reference/runtime-api.md) | window.WF — the runtime every compiled page calls, usable from a script of your own, the console or a library's callback. |
| 47 | [Grammar](reference/grammar.md) | Every keyword and where it may appear, the shapes of declarations, statements and expressions, operators and precedence, and the literals. |
| 48 | [Browser support](reference/browser-support.md) | Which browsers a WebFluent site targets, and what each newer platform feature falls back to where a browser lacks it. |
| 49 | [Glossary](reference/glossary.md) | The words this guide uses, each in a sentence, with where to read more. |
| | **Help** | |
| 50 | [Troubleshooting](errors/troubleshooting.md) | The problems people meet, what causes each, and the fix — from installing to deploying, plus the questions that come up most. |
| 51 | [Upgrading](about/upgrading.md) | What changed in each release, what to do when you upgrade, and how wf migrate carries an older project forward. |
| 52 | [Contributing](about/contributing.md) | How the compiler is put together, how to build and test it, and how to change the language, a built-in or this guide. |
| | **Reference** | |
| 53 | [Library](reference/library.md) | The compiler as a library: the Rust crate's documented API, the Node package, and wf.js in a browser. |
| 54 | [Editors](reference/editors.md) | The language server, wf-lsp, and what the VS Code and Zed extensions and the Tree-sitter grammars each support. |
<!-- /contents -->
