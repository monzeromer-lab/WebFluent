---
chapter: 49
kind: "reference"
route: "/reference/glossary"
group: "reference"
blurb: "The words this guide uses, each in a sentence, with where to read more."
description: "The terms this guide uses — state, signal, pre-rendered page, hydration, flag, part, slot, registry, scope and more — each in a sentence, with a link."
---
# 49. Glossary

This glossary defines the words this guide uses, in alphabetical order.
Each entry is one sentence, with a link to the chapter that explains the
idea. The guide uses each term in the same sense every time.

**Action.** A named function of a page, component or store whose body
changes state or does other work: `action add() { … }`
([State and reactivity](../learn/07-state-and-reactivity.md#actions-and-locals)).

**App shell.** The `app { }` declaration: what every page is drawn inside,
with a `Router` where the current page goes
([Pages and routing](../learn/05-pages-and-routing.md#the-app-shell)).

**Block.** The `{ … }` after an element, holding its style, transitions,
handlers, slot fills and children, in that order
([Elements](../learn/06-elements.md#the-call-shape)).

**Built-in.** An element the language provides, such as `Button` or
`Card`, which the compiler knows through the registry
([Components reference](components.md)).

**Case.** One value of an `enum`, written with a dot: `.primary`,
`.failed("x")` ([Types](../learn/12-types.md#enums)).

**Chunk.** A page's own script and stylesheet, written to their own files
and loaded the first time the page's route shows, when `build.split` is on
([Configuration](configuration.md#build)).

**Component.** An element you declare, with typed props, events, slots and
parts: `component UserCard(…) { … }` ([Components](../learn/10-components.md)).

**Declaration.** A top-level statement of a file: `page`, `component`,
`store`, `theme`, `app`, `type`, `enum`, `api`, `const`, `data`, `image`,
`animation` or `test` ([Language basics](../learn/04-language-basics.md#declarations)).

**Derived value.** A value worked out from state and worked out again when
what it reads changes: `derived total = …`
([State and reactivity](../learn/07-state-and-reactivity.md#derived)).

**Design token.** A named value that a theme sets and every built-in reads,
written `$name` in a style block and `var(--name)` in a stylesheet
([Design tokens](design-tokens.md)).

**Diagnostic.** A finding the compiler reports, with a code such as `T05`,
a severity, a place and, where it is known, a fix; also called a
*finding* ([Diagnostics](../errors/diagnostics.md)).

**Effect.** Code that runs when its page shows and again when a state it
read changes, with a `cleanup { }` for what it set up
([State and reactivity](../learn/07-state-and-reactivity.md#effect)).

**Element.** Anything a page draws by a capitalised name — a built-in such
as `Button`, or a component of your own ([Elements](../learn/06-elements.md)).

**Enum.** A type with a fixed set of cases, declared
`enum Tone { calm, loud }` ([Types](../learn/12-types.md#enums)).

**Event.** Something that happens — a click, a key, or an event a
component declares and fires with `emit` — that a handler answers
([Events](../learn/08-events.md)).

**Flag.** A word after a dot that sets a `Bool` prop or picks an enum
case: `Button("Save").primary.lg`
([Elements](../learn/06-elements.md#props-cases-and-flags)).

**Handler.** Code run when an event happens, written `on click { … }`
([Events](../learn/08-events.md#handlers)).

**Hydration.** The script taking over a pre-rendered page in place,
without drawing it again
([Static sites and SPAs](../about/static-and-spa.md#hydration)).

**Imperative block.** The body of an action, handler or effect, made of
statements that do things; the opposite of a render block
([Language basics](../learn/04-language-basics.md#bodies)).

**Layout.** A component a page names with `layout:` to frame it, which
draws the page in its default slot
([Pages and routing](../learn/05-pages-and-routing.md#layouts)).

**Lint.** A rule the compiler checks beyond whether the program can work,
such as an accessibility rule or an unused-value rule; the `lints` setting
changes what any finding counts as ([Configuration](configuration.md#lints)).

**Page.** A declaration with a `path:` that the router draws when the
address matches it: `page Home(path: "/") { … }`
([Pages and routing](../learn/05-pages-and-routing.md#declaring-a-page)).

**Part.** A sub-element called under its owner's name: `Card.Header`,
`Panel.Footer` ([Components](../learn/10-components.md#parts)).

**Persist.** A state kept in the browser's storage across visits:
`persist scheme = "light"`
([Stores](../learn/11-stores.md#keeping-a-value-across-visits)).

**Positional argument.** The one value an element takes without a name,
such as a button's label or a heading's text
([Elements](../learn/06-elements.md#the-call-shape)).

**Pre-rendered page.** The HTML the build writes for a page from
everything it can know at build time, shown before any script runs; also
called the *static paint*
([Static sites and SPAs](../about/static-and-spa.md#what-the-static-paint-knows)).

**Project script.** A `.js` file under `src/`: a plain browser script,
linked on every page as written, whose top-level names `.wf` code calls
directly ([JavaScript interop](../recipes/javascript-interop.md#call-your-own-script)).

**Prop.** A value a component or built-in takes, passed by name — or, for
the one marked `_`, by position ([Components](../learn/10-components.md#props)).

**Registry.** The compiler's table of every built-in's props, cases,
flags, events, slots and parts, which the checker, the linters, the editor
and the reference all read; `wf registry --json` prints it
([Command line](cli.md)).

**Render block.** The body of a page, component or element, which
describes what to show and the state behind it
([Language basics](../learn/04-language-basics.md#bodies)).

**Resource.** A request a page makes, whose loading, error and ready states
a `match` shows: `resource rows = fetch(…)`
([Fetching data](../learn/15-data.md#show-a-request-with-resource)).

**Route parameter.** A `:name` segment of a page's path, which the page
declares as a typed parameter: `page User(path: "/user/:id", id: String)`
([Pages and routing](../learn/05-pages-and-routing.md#route-parameters)).

**Runtime.** The JavaScript at the head of `app.js` that the compiled
pages call, reachable as `window.WF` ([Runtime API](runtime-api.md)).

**Scope.** What owns the effects, timers and connections created inside it
— a page, a branch, a list item — and disposes of them when it leaves
([State and reactivity](../learn/07-state-and-reactivity.md#automatic-clean-up)).

**Signal.** A reactive value, which is what a `state` is underneath:
whatever reads it updates when it changes
([Runtime API](runtime-api.md#state)).

**Single-page app (SPA).** A build with one HTML shell, whose pages the
script swaps without a reload
([Static sites and SPAs](../about/static-and-spa.md#two-shapes-of-build)).

**Slot.** A place in a component that its caller fills: the default slot
(`children`), a named slot, or a scoped slot that hands values to its fill
([Components](../learn/10-components.md#slots)).

**Snapshot.** The HTML a test drew, kept under `tests/__snapshots__/` and
compared on every later run ([Testing](../learn/16-testing.md#snapshots)).

**State.** A value of a page, component or store that the page shows and
that changes: `state count = 0`
([State and reactivity](../learn/07-state-and-reactivity.md#state)).

**Static build.** A build with `build.ssg: true`, which writes a
pre-rendered HTML file for every page
([Static sites and SPAs](../about/static-and-spa.md#two-shapes-of-build)).

**Static paint.** Another name for the pre-rendered page: the HTML the
build writes for a page before any script runs
([Static sites and SPAs](../about/static-and-spa.md#what-the-static-paint-knows)).

**Store.** Shared state with actions, built the first time something
reads it ([Stores](../learn/11-stores.md)).

**Template engine.** WebFluent used from a server, through `wf render`, Rust
or Node, to render a `.wf` template with JSON data to HTML or PDF
([Server rendering](../recipes/server-rendering.md)).

**Theme.** A `theme` declaration: the design tokens it changes from the
baseline ([Styling](../learn/13-styling.md)).

**Type.** A record declared with `type`, or any of the types a value can
have, which the checker holds every use to ([Types](../learn/12-types.md)).

**Validate block.** A `validate x { … }` block beside a state, whose rules
the control bound to that state shows
([Forms](../learn/14-forms.md#validate-a-field)).

**`.wfx`.** A source file whose blocks are written by indentation instead
of braces ([Language basics](../learn/04-language-basics.md#two-layouts-wf-and-wfx)).

## Next

[Troubleshooting](../errors/troubleshooting.md): the problems people meet, what causes each, and the fix — from installing to deploying, plus the questions that come up most.
