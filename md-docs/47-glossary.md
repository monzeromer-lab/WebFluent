# 47. Glossary

<!--
route: guide/glossary
group: help
blurb: The words this guide uses, each in a sentence, with where to read more.
description: The terms this guide uses — signal, static paint, hydration, flag, part, slot, registry, scope and more — each in a sentence.
-->

**Action.** A named function of a page, component or store, whose body does
things: `action add() { … }`. [Chapter 8](08-state-and-reactivity.md#actions-and-locals).

**App shell.** The `app { }` declaration: what every page renders inside,
with a `Router` where the page goes. [Chapter 6](06-pages-and-routing.md#the-app-shell).

**Block.** The `{ … }` after an element: its style, handlers, slot fills and
children, in that order. [Chapter 7](07-elements.md).

**Built-in.** An element the language provides, such as `Button` or `Card`,
known to the compiler through the registry. [Components](36-components-reference.md).

**Case.** One value of an `enum`, written with a dot: `.primary`, `.failed("x")`.
[Chapter 13](13-types.md#enums).

**Chunk.** A page's own script and stylesheet, written separately and loaded
when its route first shows (`build.split`).

**Component.** An element you declare, with typed props, events, slots and
parts: `component UserCard(…) { … }`. [Chapter 11](11-components.md).

**Declaration.** A top-level statement of a file: `page`, `component`,
`store`, `theme`, `app`, `type`, `enum`, `api`, `external`, `const`, `data`,
`image`, `animation` or `test`. [Chapter 5](05-language-basics.md).

**Derived.** A value computed from state, recomputed when what it read
changes: `derived total = …`.

**Design token.** A named value a theme sets and every built-in reads,
written `$name` in a style. [Chapter 41](41-design-tokens.md).

**Effect.** Code that runs when a page shows and again when a state it read
changes, with a `cleanup { }` for what it set up.

**Flag.** A word after a dot that sets a `Bool` prop or picks an enum case:
`Button("Save").primary.lg`. [Chapter 7](07-elements.md#props-cases-and-flags).

**Handler.** `on click { … }` — code run when an event happens.
[Chapter 9](09-events.md).

**Hydration.** The script taking over a pre-rendered page in place, without
drawing it again. [Chapter 26](26-static-and-spa.md#hydration).

**Imperative block.** The body of an action, handler or effect: statements
that do things. Its opposite is a render block.

**Part.** A sub-element called under its owner's name: `Card.Header`,
`Panel.Footer`. [Chapter 11](11-components.md#parts).

**Persist.** A state kept in the browser's storage across visits.
[Chapter 12](12-stores.md#keeping-a-value-across-visits).

**Positional argument.** The one value an element takes without a name — a
button's label, a heading's text.

**Registry.** The compiler's table of every built-in's props, cases, flags,
events, slots and parts; the checker, the linters, the editor and the
reference all read it. `wf registry --json` prints it.

**Render block.** The body of a page, component or element: what to show,
and the state behind it.

**Resource.** A request a page makes and a `match` shows: `resource rows = …`.
[Chapter 17](17-data.md).

**Runtime.** The JavaScript at the head of `app.js` that compiled pages call,
reachable as `window.WF`. [Chapter 42](42-runtime-api.md).

**Scope.** What owns the effects, timers and connections created inside it —
a page, a branch, a list item — and disposes of them when it leaves.

**Signal.** A reactive value: what `state` is. Whatever reads it follows it.

**Slot.** A place in a component its caller fills: the default slot
(`children`), a named slot, or a scoped slot that hands values to its fill.
[Chapter 11](11-components.md#slots).

**Static paint.** The HTML the build pre-renders for a page, from everything
it can know at build time. [Chapter 26](26-static-and-spa.md#what-the-static-paint-knows).

**Store.** Shared state with actions, built the first time something reads
it. [Chapter 12](12-stores.md).

**Theme.** A `theme` declaration: the design tokens it changes.
[Chapter 15](15-styling.md).

**`.wfx`.** A source file whose blocks are written by indentation instead of
braces. [Chapter 5](05-language-basics.md#two-layouts-wf-and-wfx).

## Next

[Contributing](48-contributing.md).
