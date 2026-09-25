# The WebFluent Guide

WebFluent is a language for websites. A file describes what a page shows — its
elements, the state behind them, what happens when the reader acts — and the
compiler turns it into a static site, a single-page app, a PDF or a slide deck,
with the accessibility, search and performance work done for you.

This guide takes a developer from the first `wf init` to a complete application,
in the order the ideas build on each other. Every code block is real WebFluent 3
and is parsed, checked and type-checked by the test suite, so what you read is
what compiles; the cookbook's applications build and run.

## Contents

| # | Chapter | What it covers |
|---|---|---|
| 1 | [Getting started](01-getting-started.md) | Install, a first project, build and serve, the project layout, the config file |
| 2 | [Language basics](02-language-basics.md) | Files and declarations, naming, comments, the two layouts (`.wf` and `.wfx`) |
| 3 | [Pages and routing](03-pages-and-routing.md) | Page attributes, routes and parameters, layouts, the app shell, `head`, the 404 page |
| 4 | [Elements](04-elements.md) | The element call shape: positional, named props, flags, enum cases, the block and its order, attributes |
| 5 | [State and reactivity](05-state-and-reactivity.md) | `state`, `derived`, `effect`, actions and `let`, `persist`, timers, refs, the browser's values |
| 6 | [Events and forms](06-events-and-forms.md) | Handlers, keys, two-way binding, forms, pending actions, theme switching |
| 7 | [Control flow](07-control-flow.md) | `if`, `if let`, `for … by`, `show`, `match` |
| 8 | [Components](08-components.md) | Props, events, slots (including scoped slots), parts, layouts |
| 9 | [Stores](09-stores.md) | Shared state across pages, actions, derived values, persistence |
| 10 | [Types](10-types.md) | The type language, records and extension, enums and payloads, shapes, inference and narrowing, what the checker reports |
| 11 | [Expressions](11-expressions.md) | Literals and interpolation, operators, `if`/`match` as values, lambdas, methods and helpers, `format`/`ago`, regexes, tokens, `await`, `const`/`env` |
| 12 | [Styling](12-styling.md) | Design tokens and themes, `style { }` with splices and nesting, `transition { }`, classes and stylesheets, dark mode, layout without CSS |
| 13 | [Motion](13-motion.md) | Enter and exit animations, keyed lists and stagger, `animation` keyframes, route transitions, reduced motion |
| 14 | [Data](14-data.md) | `resource` and `match`, reactive URLs, `await fetch` in actions, `data` files, static paths, `const` and `env` |
| 15 | [Internationalisation](15-i18n.md) | Translation files, `t()`, plural forms, right-to-left, locale-aware formatting |
| 16 | [Content](16-content.md) | `Markdown`, `.md` pages with front matter, static builds, search and sharing tags, code and media |
| 17 | [Outputs](17-outputs.md) | SPA, static site, PDF documents, slide decks, rendering templates with data (CLI, Rust, Node) |
| 18 | [Tooling](18-tooling.md) | Every `wf` command, `wf serve`, diagnostics by family, `wf fmt`, `wf test`, `wf docs`/`registry`/`types`, the language server and editors |
| 19 | [Security](19-security.md) | What the compiler guarantees, what it cannot, sessions, `env`, CSP, headers and a deployment checklist |
| 20 | [Components reference](20-components-reference.md) | Every built-in with its props, cases, flags, events, slots and parts (generated from the compiler) |
| 21 | [Cookbook](21-cookbook.md) | Three complete applications, and recipes for the things every site needs |

## How to read it

- New to WebFluent: read 1–8 in order; they are short and each example runs.
- Coming from React, Vue or Svelte: skim 2 and 4, then read 5, 8 and 9 — the
  reactivity model is signals, the components are declared and typed, and the
  compiler checks what you write against what the components declare.
- Looking something up: chapter 20 is the reference, chapter 18 has the
  diagnostics codes, chapter 10 the type checks, and chapter 21 the recipes.

## The five ideas

1. **A file is a list of declarations.** `page`, `component`, `store`, `theme`,
   `app`, `type`, `enum`, `const`, `data`, `animation`, `test`. Pages own their
   routes; there is no separate route table.
2. **An element is a typed call.** `Button("Save", tone: .primary).lg { on click { save() } }`.
   The one positional argument, named props, `.flag`s, then a block. The
   compiler knows every built-in's props and cases, and every component you
   declare, so a misspelling is an error, never a silent no-op.
3. **State is a signal.** `state count = 0`; anything that reads `count` follows
   it. `derived` values recompute, `effect`s re-run, the DOM updates.
4. **Styles are CSS, written where the element is.** `style { padding: $md; &:hover { background: $surface-hover } }`
   — raw values, design tokens with `$`, nested rules as CSS nesting spells them.
5. **The build is the whole site.** HTML pre-rendered per page, one stylesheet
   pruned to what you used, a small runtime, a sitemap, Open Graph tags,
   content-security headers — from `wf build`, with nothing to configure.
