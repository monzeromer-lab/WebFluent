# WebFluent v3.2.1 Release Notes

## Fixed

### A store's action is no longer mistaken for a list method

`Todos.remove(id)` called the action the store declares. The JavaScript
back end matched the method table on the name alone, without looking at
what it was called on, so it compiled to `Todos.splice(id, 1)` — a store
is an object of state and actions, never a list, so the handler threw
`Todos.splice is not a function` on the first click. The page painted
perfectly until then, which is why nothing caught it.

A store's own members now come first, so an action may be named after a
list method. The shadowing was not limited to `remove`: every name in the
table — `push`, `filter`, `map`, `take`, `contains`, `first`, `last` — hid
an action of the same name. `Todos.items.remove(0)` still reaches the
list, since that names the list rather than the store.

### `items.remove(i)` repaints

`remove` on a list held by `state`, `persist` or a store's member spliced
it in place, so the signal kept the array it already had: nothing reading
it repainted and a persisted list was never written. It now goes through
the signal, as `push` does, via the new `WF.removeAt(list, index)`.

### Navigation items drawn by a `for` are links in a static build

A `Sidebar.Item(to:)` reached through a loop pre-rendered as a bare `<li>`
with the destination dropped: the components reference on the site, whose
index is `for c in shown { if … { Sidebar.Item(to: …) } }`, shipped a
navigation panel with no links in it. The static back end read `to:` as a
literal only, so a destination that needs the loop's binding — `"/docs/{p}"`
— resolved to nothing. It now reads it through the build-time scope.

The published site was also rebuilt: its bundle still carried the
`<li to="…">` from before the 3.2.0 fix, so clicking a component in the
reference sidebar did nothing at all, silently.

## Testing

`tests/cookbook.rs` extracts the three applications from
`md-docs/20-cookbook.md` — the guide itself, not a copy — builds them and
runs them against the fake DOM in `tests/js/cookbook.test.mjs`, clicking
through add, toggle, remove and clear. `tests/docs_parse.rs` now runs the
JavaScript back end over every guide block and rejects a bundle that calls
a store member the store does not declare, which parsing and type-checking
could not see. A navigation item inside a `for` or an `if` is held to being
a link in every backend.

# WebFluent v3.2 Release Notes

## Added

### The guide: `md-docs/`

A twenty-chapter developer guide, from `wf init` to three complete
applications: getting started, the language basics, pages and routing,
elements, state, events and forms, control flow, components, stores,
types, expressions, styling, motion, data, internationalisation, content,
outputs, tooling, a components reference generated from the registry
(`scripts/components-reference.py`), and a cookbook. Every code block is
parsed, semantically checked, type-checked and held to the linters by
`tests/docs_parse.rs`, and the cookbook's applications build and were
walked through in a browser. The site's documentation will be rewritten
from it.

### Syntax colouring

`Code(…, language: "wf")` — or `json`, `bash`, `css` — and a Markdown
fence in one of those languages are coloured, at build time by
`codegen::highlight` and in the browser by `WF.highlight`, the same
tokens either way: keywords, capitalised names, strings, numbers and
colours, `name:` props, `$tokens`, comments and a shell prompt, each a
`wf-tok-*` span taking its colour from the theme's `syntax-*` tokens. The
baseline theme now carries those tokens and `term-bg`/`term-ink`/`term-dim`,
and a code block is painted on `term-bg` — a console stays dark in both
themes — with `white-space: pre` and a horizontal scroll. `sun` and `moon`
join the icons; an `icon:` that reads state follows it.

### The documentation site, from the guide

`site/` is written from `md-docs/` by `scripts/site-from-guide.py`: one page
per chapter under `site/src/pages/guide/` (routes `/docs/guide/<chapter>`),
framed by `DocsShell` — the header with search over every chapter and
section, the chapter nav with its numbers, the article at a reading
measure, the headings rail and the links to the chapters either side —
with prose as `Markdown` (inline code, links and lists render), fences as
`CodeBlock` (coloured, with a Copy button), shell blocks as `Terminal` and
pipe tables as `Table`. The components reference is data: `wf registry
--json` becomes `site/src/registry.json` (`scripts/site-data.py`), an
index page and one page per built-in at `/docs/reference/<name>` with its
signature, props, flags and cases, events and parts under tabs, a filter
over the index, and the entries nearby. The landing page, the chapter
page, the reference and the phone layout follow the design canvas
"WebFluent — site and documentation" to the pixel, in the MO design
system: graphite for structure, blue for intent, cyan for data, Inter and
JetBrains Mono, a console that stays dark in both themes, a theme toggle,
light and dark, and Arabic chrome with the chapters kept left-to-right.

### The language

- A string splice takes any expression: `"{format(total, .currency)}"`,
  `"{[1, 2].length}"`, `"{a ?? "none"}"` and
  `"{if ok { "yes" } else { "no" }}"` — a splice may hold a string of its
  own, as long as the group closes on the line. A brace group that is not
  an expression (`"{key: value}"` in prose) stays text; one with a slip in
  it is an error that names the splice.
- `() => expr`: a lambda of no parameters, for `setTimeout(() => go(), 300)`.
- A record construction carries the fields it left out — each field's
  default, `null` for an optional — so `Todo(id: "a", title: "b")` holds
  `done: false` wherever it goes.
- `await fetch(url, options)` in an action or a handler is the parsed body,
  and a failed response throws (`WF.request`): the same request a
  `resource` makes. The browser's own is `window.fetch`.
- `items.push(x)` on a `state`, a `persist` or a store's member sets a new
  list, so what reads it repaints and a persisted one is written.
- `Modal(visible:)`/`Dialog(visible:)` take any expression that reads
  state — a store's member, a condition — and write a state or a store
  member back when the browser closes the dialog itself.
- A page's `guard:` and `redirect:` work: a route whose guard is false
  sends the reader to its redirect (default `/`) instead of painting.
- A page's own `meta` in `head { }` replaces the standard tag of the same
  `property` or `name`, so a post page has one `og:title`, its own.
- The registry lists the icons the runtime draws (`registry::ICONS`,
  pinned to the runtime's table); an `Icon("…")` or `icon:` that names
  another is a warning, and the language server completes from the same
  list.

## Fixed

- The static paint and the template engine bind an `if let` name: the
  branch used to read it as unknown and paint an empty heading.
- A store action's `try`/`catch`, `for` and `else if` used to fall through
  to the page emitter and read `_x()` signals a store does not have.
- `Form(bind: form)` declared the handle twice — as a form and as a ref —
  which the browser refused.
- A `Checkbox`, `Radio` or `Switch` drops nothing now: `aria-*`, `data-*`,
  `name`, `id` and `disabled` reach the `<input>`, and radios bound to one
  state share a `name` so the arrow keys move between them.
- `navigate("/?filter=done")` and a `Link(to:)` with a query string route
  by the path and keep the query readable through `query`; a link to the
  route with a query is the current one.
- The single-page shell addresses its assets from the site root, so a deep
  link served by the SPA fallback (`/deploys/8f2c`) loads `app.js` rather
  than the shell again.
- `WF._basePath` is a getter again; a static build's links no longer start
  with `undefined`.
- A bare `slot` followed by `on key(…)` on the next line names no slot
  `on`.
- The vocabulary lint knows a page's route parameters, and its `const` and
  `data` names.
- A page framed by a `layout:` is judged with the layout's outline: the
  `h1` a layout draws is the page's (`A12`), and a heading level skipped
  across the boundary is seen (`A11`).
- `Grid(columns:)` is a `data-cols` attribute the stylesheet reads — a
  build with `build.csp` on used to have its grids blocked as inline
  styles — and a `columns:` that reads state follows it.
- `wf serve` serves a site built with `build.base_path` under that path,
  as its host will, and sends `/` there.
- A Markdown link or image to a site-relative path takes the base path,
  at build time and in the browser.
- A `Breadcrumb.Item` without `to` paints as a `<span>`, not an `<li>`
  outside a list.
- The skip link hides by a transform, not an off-canvas offset that a
  right-to-left page could scroll to.
- A `:param` page's static files carry their own route as the canonical
  link and sharing URL, not the pattern with `:slug` in it.
- An `IconButton` with a `class:` keeps the engine's classes; its `icon:`,
  `label:` and title follow state.
- A `Sidebar.Item` or `Breadcrumb.Item` with a `to:` reached through a
  `for` or an `if` is a link in the browser too; it used to be a bare
  `<li to="…">` that went nowhere.

---

# WebFluent v3.1 Release Notes

## Added

### The language core

- `for` in actions, handlers and effects — with an index, over a range
  (`1..10`, `1..=10`), or a keyed list; `try { } catch e { }`;
  destructuring `let { a, b } = m` and `let [x, y] = l`.
- Optional chaining `a?.b`, `a?.m()`, `a?.[i]`: null when the base is,
  the rest of the chain with it, typed as `T?`.
- `if let x = v { a } else { b }` as a value.
- Regular expressions `/pattern/flags`, with `test`, `exec`, and a string's
  `match`, `replace`, `replaceAll`, `split`, `search`; a `/` after an
  operand still divides.
- Spread `[...a, b]`, `{ ...m, k: v }`.
- `const NAME = value` at the top level, and `env.X` from the config's
  `"env"` map, fixed at build time.
- Record composition `type Admin = User { role: String }`; enums whose
  cases carry a payload, `enum Status { idle, failed(reason: String) }`,
  written `.failed("x")` and matched `.failed(r) { … }` (a match
  expression binds one name); `["failed", "x"]` at run time, a bare case
  its name.
- Structural shapes: a map literal is typed by the fields it was written
  with, so `form.nam` is a `T05` and a literal given where a record is
  wanted is checked field by field; a field only some items of a list have
  is `T?`.
- Helpers `sortBy`, `groupBy`, `unique`, `take`, `first`, `last`,
  `flatMap`, `sum` on lists and `capitalize`, `truncate` on strings —
  in the browser and at build time.
- `format(value, .style, option)` — `.number`, `.integer`, `.decimal`,
  `.currency`, `.percent`, `.compact`, `.date`, `.time`, `.datetime`,
  `.relative`, or a date pattern such as `"yyyy-MM-dd HH:mm"` — and
  `ago(date)`, in the i18n locale when the project has one and the
  document's language otherwise; the static paint and the template engine
  format from a table of the common locales and currencies.
- The type checker places an error at the expression it names when the
  source is at hand; hover shows a record's inherited fields and an enum's
  payloads; the tree-sitter grammar and the Zed extension know every new
  form.

### Tooling

- `wf fmt` formats a project's sources: indentation by block depth,
  spacing around braces, trailing blanks and blank-line runs — held to the
  file's tokens, comments kept; `wf fmt --check` for CI. `--to wfx|wf`
  still changes the layout.
- `wf serve` rebuilds on save, reloads the page, and shows a failed build's
  diagnostics over the page until the next build passes.
- Plural forms: `t("items", { count: n })` picks `items.one`, `items.other`
  and the locale's other categories from the translation file.
- A route change with `Router(transition:)` plays through the View
  Transitions API where the browser has it, the class-based animation
  standing in elsewhere.
- `wf docs`: a self-contained component gallery of every built-in and of
  the project's own declarations.
- `Markdown(text)`: a small Markdown rendered the same way at build time
  and by the runtime, the text escaped first; a `.md` file under `src/` is
  a page, its front matter the page's attributes.
- `data posts = "posts.json"`: a JSON file as a constant, read at build
  time; `paths:` on a `:param` page renders the static page once per value
  and lists each in the sitemap.
- `wf test`: `test "name"(data: { … }) { elements  expect "text"  expect
  not "text" }` declarations under `tests/`, rendered through the template
  engine with the project's declarations at hand, held to their
  expectations and to snapshots (`--update` accepts a new render).
- The language server renames — a component, a store, a store member, a
  local — across the project, inside interpolations too, and offers
  `Extract component` on a selection of elements, with what they read as
  typed props and the stores they use `use`d again.
- Unused warnings `U01`–`U05`: state, derived values, actions, components
  and store members the program declares and never reads, in the build's
  output and the editor.

### Components and reactivity

- Scoped slots: `slot row(item: Todo, index: Number)`, used as
  `row(item: it, index: i)`, filled as `row(t, i) { … }`; the fill is drawn
  again when a value it is handed changes.
- Parts of a component: `part Header(_ text: String) { … }` inside
  `component Panel`, called as `Panel.Header("…")`.
- A component's enum props are written to its root element as
  `data-<prop>="<case>"`.
- `ref: name` on an element: a handle read as the element itself
  (`name.focus()`).
- `every(ms) { }`, `after(ms) { }` and `effect { … cleanup { … } }`; what a
  page, a branch, a list item, a match arm or a slot creates is disposed of
  when it leaves.
- `persist name = value` in a page, a component or a store: kept in the
  browser's storage across visits.
- `viewport`, `query`, `hash` and `theme`: the browser as values, kept
  current; a declared name shadows them.
- `on key("ctrl+k") { }` on an element or at the top of a page.
- `animation Name { from { } to { } }` keyframes, played with `animate:
  .Name` or written into a style; container queries pass through.
- Dark mode: `"theme": { "dark": "Night" }` applies a second theme under
  `prefers-color-scheme: dark` and on `setTheme("dark")`, kept across
  visits.
- `Form(bind: form)`: `form.valid`, `form.values`, `form.reset()`; an async
  action's `name.pending`.
- A page's `head { meta(…) link(…) script(…) }`, painted at build time and
  kept current on the live page.
- The template engine binds a component's positional props and defaults,
  and every component is registered before a page renders; the static
  paint substitutes props through every expression and into branches,
  loops and matches.

### The indented layout: `.wfx`

The one grammar, written by indentation. A `.wfx` file opens a block with
a line indented deeper than the one before it and closes it with a dedent;
inside parentheses, brackets and any braces you write, the layout is free,
so a multi-line argument list, a map literal and a one-line
`{ on click { save() } }` read as they always did. Nothing else changes:
the parser, the checks, the output, the language server, the template
engine and `wf generate` read and write both layouts, and a project may
hold both.

`wf fmt --to wfx` rewrites a project's `.wf` files as `.wfx`, and `--to wf`
the other way — a change of layout and nothing else, held to a
byte-identical build across every fixture, the documentation site, every
`wf init` template and Halyard, and refused for a file whose indentation
does not already follow its braces. `wf migrate --wfx` migrates a
WebFluent 2 project straight into the indented layout.

The Zed extension gains a second grammar and language for `.wfx`,
generated from the same `grammar.js`, with its own indent queries; the
VS Code extension opens `.wfx` files. Specified in `spec/SYNTAX_V2.md`
§8a.

### For tools: `wf registry --json`, `wf types --json`, and edits by name

The registry as JSON — every built-in with its positional prop, props and
their cases, flags, events, slots, parts and attribute families, plus the
universal props — and a project's own declarations the same way: enums,
records, components with their props, events and slots, stores with their
members, pages with their routes. The structured edit operations speak the
grammar by name: `SetProp`/`RemoveProp`, `SetFlag`/`RemoveFlag`,
`SetHandler`/`RemoveHandler`, `SetSlot`/`RemoveSlot`, beside the older
`AddModifier`/`SetArg` that stay as aliases; two insertions at one place
land in the order they were asked for; a `.wfx` file is edited through its
braced spelling (`apply_edits_to`). WebFluent Studio reads both.

### `wf build` reads source files in name order

The order files are read in decides the numbering of a page's elements,
and it followed the file system's own order — so the same project could
build differently on another disk. It is alphabetical now, `App.wf` first.

---

# WebFluent v3.0 Release Notes

The grammar release. WebFluent's surface syntax grew by accretion until it
broke its own rules — a misspelled modifier did nothing, one `{ }` meant
five things, a style value was a string in which a bare word was a token
or a variable depending on a lookup, routes were declared twice, a project's
own components could not declare a variant, an event or a slot. WebFluent 3
is one language with one lexeme per meaning, a registry that describes every
built-in once, a type checker, motion on any element, and `wf migrate`,
which rewrites a project mechanically: every fixture, the documentation
site, every `wf init` template and the Halyard demo (129 files) migrate to a
**byte-identical build**.

**This release is a hard cut.** `wf build` and the language server read only
the new grammar; a file in the old one is refused with a pointer to
`wf migrate`. See *Migrating* at the end.

---

## The grammar

```wf
enum Tone { neutral, info, danger }
type Todo { id: String, title: String, done: Bool = false, tone: Tone = .neutral }

/// A todo, as one row.
component TodoRow(_ label: String, todo: Todo, compact: Bool = false) {
    event toggle(id: String)
    slot trailing
    Row(align: .center, gap: .sm) {
        style {
            padding: 6px 0
            background: $surface
            --accent: {todo.color}
            &:hover { background: $surface-hover }
        }
        transition { background: 150ms ease-out }
        on click { emit toggle(todo.id) }
        Text(label).bold
        trailing
    }
}

page Home(path: "/", title: "Todos", layout: AppShell(crumb: "Todos")) {
    use Todos
    resource list = fetch("/api/todos")
    match list {
        loading { Spinner }
        error(e) { Alert(e.message).danger }
        ready(v) {
            for t in v by t.id {
                TodoRow(t.title, todo: t, exit: .fadeOut).fadeIn {
                    on toggle(id) { Todos.toggle(id) }
                    trailing { Badge("new").info }
                }
            }
        }
    }
    Button("Add").primary.lg { on click { Todos.add(draft) } }
}

app { Navbar(brand: "Todos") { Navbar.Links { Link("Home", to: "/") } }  Router(transition: .fade) }
```

- **Lowercase declarations**: `page`, `component`, `store`, `theme`, `app`,
  and the new `type` and `enum`. There are no keyword tokens: `type:` and
  `on = !on` are what they look like.
- **Flags**: `.primary.lg` after the parentheses — a `Bool` prop or a case
  exactly one enum prop has; a case two props share is named,
  `tone: .info`. A word nothing declares is an error that lists what the
  component takes. `Input(…).text`, `Heading("x").h1`, `Spacer.sm`.
- **One positional argument**, the component's declared positional prop,
  then named ones. An enum prop takes a case: `Row(gap: .md)`.
- **Parts under their owner**: `Table.Head`/`Body`/`Row`/`Cell`,
  `Select.Option("Label", value: "v")`, `Tabs.Page`, `Card.Header`.
- **One block, in order**: `style`, `transition`, `on` handlers, slot fills,
  children. `on click { }` and `on click(e) { }`; a `Button`'s block holds
  what it shows, `on click` what it does.
- **Style values are CSS**: `padding: 1rem 2rem`, `background: $surface`,
  `width: {pct}%`, `&:hover { }`, `@media (…) { }`, `--name: value`. A
  `$token` is validated against the theme; `$md` on `padding` is
  `$spacing-md`.
- **Components declare** typed props (`_ name: String`, `active: Bool =
  false`), `event`s fired with `emit`, and `slot`s filled by name.
- **Pages own their routes**: `Router` takes no children; routes match by
  specificity. `layout: Shell(args)` wraps a page in a component with a
  default slot.
- **`resource` and `match`** replace the `fetch` block; `for … by key` keys a
  list; `if let x = expr { }` unwraps; `??`, `T?`, `[T]`, `null`, `await`
  in actions and handlers, `if` and `match` as values, `.case`, `$token` in
  expressions.
- `Store.load(id)` at the top of a page is set-up code; `let x` is a local
  in a handler or action.

The full grammar, with why it changed and the migration table, is
`spec/SYNTAX_V2.md`.

## Added

### `wf migrate`

`wf migrate [path] [--check] [--stdout]` rewrites every `.wf` under `src/`
in place — spelling only: the migrated project builds to what it built
before. What needs a decision is reported as a note. `--check` reports
without writing; `--stdout` prints one file's result.

### The registry

`src/registry` describes every built-in once — its positional prop, props
with their types and enum cases, flags, events, slots, parts, attribute
families — and the compiler's checks, the linters, the language server's
hover and completion, the tree-sitter vocabulary check and the tests all
read it. The `secondary` tone reaches `Badge`, `Text` and `Icon`;
`Spinner` has sizes and no tone.

### The type checker

Gradual and structural: everything it cannot resolve is `Any`, which agrees
with everything, so every existing project checks without an annotation,
and each annotation — `state x: [Todo]`, a prop's type, a `type`'s fields,
`resource x: [Deploy]` — narrows what it can say. It reports, as errors with
hints: a value of the wrong type (`T01`), a case an enum lacks (`T02`), a
value that may be `null` read without `if let`, `??` or a `!= null` check
(`T04`), a field or method a record lacks (`T05`), a member a store lacks
(`T06`), a list or record as a condition (`T07`), a `for` over a non-list
(`T08`), an `emit` that does not match its event (`T09`), a call with the
wrong arguments (`T10`), a `match` with nothing to match (`T11`), and a
`bind:` on a control of the wrong kind. Specified in `spec/TYPES.md`.

### Motion

The universal props — an animation as a flag (`.fadeIn`), `exit:`,
`.fast`/`.slow`, `duration:`, `delay:`, `easing:`, `stagger:` — on **any**
element, built-in or the project's own. On a branch's root they are the
branch's; anywhere else the element plays its own exit before it is
removed, wherever it was. A keyed list slides a moved item from where it
was. `Router(transition: .fade | .slide, duration:)` plays one page out and
the next in. `transition { }` blocks take `$token` timings. `prefers-reduced-motion`
is honoured by the runtime as well as the stylesheet: no class, no wait.

### Compiler diagnostics

Errors now: a parse error, a reference to nothing, a flag, case, part,
prop, event, slot or layout nothing declares, an ambiguous flag, a
positional argument out of place, a `match` arm the value cannot take, a
loose statement in a render block, a clause out of order, the type
checker's findings, and a file in the old grammar. `V01` names the flag you
meant (`` did you mean `.center`? ``); a named argument a built-in does not
declare is a warning and is written as an attribute.

### Tooling

- **Language server**: hover and completion are driven by the registry and
  the type checker — flags after `.`, cases after `tone: .`, `$tokens` in a
  style value, events after `on`, a component's events after `emit`, the
  arms a `match` still lacks, a component's slots, `layout:` candidates,
  props not yet written, a record's fields and a list's methods after a
  typed value; hover shows a flag as the prop it sets and a name's inferred
  type; go to definition follows `layout:`.
- **Editors**: the tree-sitter grammar is rewritten for the new grammar (a
  small external scanner reads style values), with the Zed queries and
  snippets and the VS Code TextMate grammar; every `.wf` in the repository
  and in Halyard parses without an error node.
- **Structured edits** (`EditOp`) write the grammar of the file they edit:
  `AddModifier` appends a flag, `SetStyle` writes a raw value.
- The documentation's `wf` blocks are parsed by the test suite
  (`tests/docs_parse.rs`), so an example cannot go stale.

## Breaking changes

- **The old grammar is refused.** `wf build`, `wf serve`, the template
  engine, the language server and the Node binding read WebFluent 3 only.
  Run `wf migrate`.
- **A `Button`'s block of statements is no longer its click handler**;
  write `on click { }`. The migrator does.
- **A word nothing declares is an error**, where it used to be the `V01`
  warning; `Alert(elevated)` and its `V02` are gone with the modifier
  words.
- **The runtime's public surface is renamed** for what each thing does:
  `h`→`el`, `condRender`→`when`, `listRender`→`each`, `showRender`→`show`,
  `wfFetch`→`fetch` beside the new `resource`, `createStore`→`store`,
  `createRouter`→`router`, `definePage`→`page`, `showToast`→`toast`,
  `bindDialog`→`dialog`, `bindPopup`→`popup`, `tablist`→`tabs`,
  `offCanvas`→`drawer`; new `match`, `mark`, `emit`. The compiler and the
  runtime ship together; nothing outside a build's own bundle should have
  named these.
- **`t("key", name: value)` never parsed**; interpolation takes a map:
  `t("key", { name: value })`. The documentation said otherwise.
- **Library**: `webfluent::parse_source(src, file)` is the one way in;
  `Lexer`/`Parser` of the old grammar are `#[doc(hidden)]` behind
  `wf migrate`. `Expr::Record` is a new expression; `sema::check`,
  `sema::types::check` and `sema::lower` run between parsing and codegen,
  and the linters see the lowered program.

## Fixed

- A list was rebuilt whole on every change, losing focus, scroll and
  animation state in items that had not changed; `for … by key` keeps the
  nodes of the items that stay.
- The route table depended on source order; it is now ordered by
  specificity, then path, whether the `Router` had children or not.
- `easing:` on an element was accepted and ignored; it sets the timing
  function, and `spring`, `smooth`, `bouncy`, `easeOut` and the rest mean
  the same there as in a `transition` block.
- A multi-line call that gained motion props from a migrated `animate(…)`
  clause had them written on the closing parenthesis's line; they take a
  line of their own.
- Lint hints spoke the old grammar (`Heading("Page Title", h1)`).

## Migrating

1. **Install 3.0 and run `wf migrate`** in the project. Read the notes it
   prints: each names a file and a line where the migrator could not decide
   — a modifier word no spelling exists for, an imperative `fetch` to
   rewrite as `let x = await fetch(…)`. Commit the result; `wf build` is
   byte-identical for a project that needed no notes.
2. **Expect errors that were warnings.** A misspelled flag, a prop a
   component does not declare, a case an enum lacks: the build stops and
   says what it takes.
3. **Annotate what you want checked.** Nothing is required; `state items:
   [Todo] = []` and `resource r: [Deploy] = fetch(…)` are where the type
   checker starts to see.
4. **If anything of yours named `WF.*` runtime functions** — a custom
   stylesheet does not, a `script` in `index.html` might — rename per the
   table above.
5. **Zed users**: the extension's 2.0 pins the new grammar; the old one
   highlights nothing useful on migrated files.

---

# WebFluent v2.2 Release Notes

A release about what a built site is like to use and to load. The Halyard
demo (https://github.com/monzeromer-lab/Halyard) was put in front of
Lighthouse and a screen reader, and everything either one found that the
compiler could have prevented, it now prevents; the same site also asked
for stylesheets of its own. Halyard's pages went from 63–71 to 87–99 on
mobile performance and to 100 on accessibility, with every finding that
remained pointed at by `wf build`. Nothing in this release is breaking.

---

## Added

### Your own stylesheets

Any `.css` file under `src/` is part of the build: read in path order,
bundled into `styles.css` after the engine's rules and before the rules
`style { }` blocks compile to, minified with the rest. `var(--token)`
reaches every design token. An element names a rule with `class:`, which
adds to the engine's classes rather than replacing them, and follows state:

```wf
Card(class: "feature feature--first") { … }
Heading("Forms", h1, class: if loud { "hero--loud" } else { "" })
```

`V02` accepts a modifier class one of these files defines
(`.wf-alert--elevated` makes `Alert(elevated)` real), in the build and in
the editor.

### Form fields

`Input` and `Select` with `label:`, `hint:` or `error:` become a field: the
label is a `<label for>` the control, the hint and the error are linked by
`aria-describedby`, the error (a string, empty when there is none) is a
`role="alert"` hidden until it has text and sets `aria-invalid`. The static
paint and the client agree, so hydration adopts it.

```wf
Input(text, bind: name, label: "Name", hint: "As on your passport", error: nameError)
```

### Accessibility in the runtime

- A route change settles the reader on the new page: focus moves to its
  heading (or the main landmark), the viewport returns to the top, the title
  is announced in a polite live region.
- `Carousel` is a labelled region whose slides are groups; it has named
  previous/next/dot controls, a pause button, stops for reduced motion and
  hidden tabs, and hides the slides that are not showing from assistive
  technology.
- `Tooltip` opens on focus as well as hover, closes on Escape, and describes
  its trigger with `aria-describedby`.
- `Dropdown` menus follow the ARIA menu pattern: `role="menu"` with
  `menuitem`s, arrow keys, Home/End, Escape, and focus returned to the
  trigger.

### Lint rules

- `A14` — a `role:` that requires particular children (`tablist`, `menu`,
  `list`, `radiogroup`, …) holding something else. A user component is
  judged by its root element, and a role given through a prop by what the
  call passed or the prop's default.
- `A15` — an `aria-label` that does not contain the control's visible text:
  its label, the literals in its block, and what a component in the block
  renders from its literal props (`Kbd(text: "⌘K")` in a search button).
- Every element rule now points at the element's own line and column.

### Build output

- `build.minify` is honoured at last: the bundle and the sheets lose their
  comments and whitespace, and nothing else. Halyard's `app.js`: 691 KB →
  212 KB.
- `build.split` (on by default) writes each page as `pages/<Name>.js`,
  linked beside `app.js` by a static page and fetched by the router the
  first time its route shows in a single-page build.
- The stylesheet carries only the component rules the project uses.
- Literal `style { }` declarations compile to shared classes instead of
  inline styles — one rule for a hundred identical cards, on the static
  paint before any JavaScript runs.
- The rules only one page can reach ship in `pages/<Name>.css`, linked by
  that page and loaded by the router before it draws; what the `App` body
  or two pages share stays in `styles.css`. Halyard: 144 KB → 49 KB shared.
- `build.compress` (on by default) writes `<file>.gz` beside every text
  output over a kilobyte, with a DEFLATE encoder of the engine's own —
  within a percent of zlib's default level, no dependency — for hosts that
  serve a precompressed file when they have one. `wf serve` sends it, and
  compresses on the way out what has none, so a Lighthouse run against the
  dev server measures what a deployed site sends.
- The static paint evaluates derived values and pure actions at build time,
  so a seeded list is painted with its rows rather than left to hydration.
- Images decode asynchronously; the first on a page is eager with
  `fetchpriority="high"`, the rest lazy, unless `loading:` says otherwise.

### Editor support

The Zed extension and the Tree-sitter grammar were rebuilt from scratch and
live in this repository (`editors/zed`, `editors/tree-sitter-webfluent`).
The language server reads the whole project the way `wf build` does — a
store declared in another file resolves, a component called across files
is known — and reports positions in the units each client expects.

## Fixed

- `class:` as an attribute replaced the engine's classes; it adds to them.
- `A14` reported a component whose root took its role from a prop as a
  plain control.
- `wf serve` read a query string as part of a file name.
- The language server matched a document to its project by URL text rather
  than by path, and could miss a file with an escaped character in its path.

---

# WebFluent v2.1 Release Notes

A release driven by building a real application. The Halyard demo
(https://github.com/monzeromer-lab/Halyard) — a nineteen-screen product design
implemented entirely in `.wf` — found every place where the language did less
than its documentation promised, and a handful of things a design system needs
that it had no way to say. Each is fixed or added here, with a test. Nothing in
this release is breaking.

---

## Added

### State and media in `style { }`

A style block may now hold blocks for how an element looks while hovered,
focused, active, disabled, or in an ARIA state, beside its `@media` blocks:

```wf
Button("Save") {
    style {
        background: "var(--brand)"
        hover { background: "var(--brand-hover)" }
        focus { outline: "2px solid var(--focus-ring)" }
        pressed { color: "var(--text-accent)" }
        @media (max-width: 768px) { width: "100%" }
    }
}
```

States: `hover focus active disabled placeholder focus-within current pressed
selected checked expanded invalid`. The ARIA ones key off the element's own
`aria-*` attribute, so state and paint cannot disagree. These compile into
`styles.css` under a content-hashed class — one rule per distinct block, shared
by every element that writes it, and nothing injected at run time (CSP-safe).
`@media` blocks, which used to become a run-time `<style>`, take the same path.

### Attributes by name

A hyphenated named argument is an attribute: `Trow(aria-selected: isSelected)`,
`Badge("Ready", data-tone: "success")`. A value that reads state follows it, and
an `aria-*` value of `false` is kept as the string `"false"`, which is a real
ARIA state. A keyword after the hyphen is fine (`data-state:`).

### Custom properties and reactive style values

`style { --edge: color }` sets a custom property. Any style value that reads
state is now kept in step with it, so `style { height: "{pct}%" }` moves.

### Components

- A component's block reaches it as `children`, in both the SPA and SSG
  backends. `AppShell(crumb: "Logs") { … }` is one component.
- Props are live: `Chip(pressed: showErrors)` repaints when `showErrors`
  changes. Declared defaults reach the SPA.
- A handler written on a component call — the click shorthand or an explicit
  `on:…` block — attaches to the component's root element.

### Vocabulary and layout

- `Row`/`Stack`/`Grid` `gap: xs|sm|md|lg|xl`, `align:` and `justify:` (six
  values each) are real utility classes in both stylesheets.
- Modifiers `fluid circle multiple ordered xs sm md lg xl header`:
  `List(ordered)` is an `<ol>`, `Tcell("Build", header)` is a `<th scope=col>`
  wherever it is written.
- `Table(caption: "…")` gives the table a visually hidden caption; a `Tcell`
  inside `Thead` is a `<th>`.
- `Link(to: "/docs", active: "prefix")`: links get `.active` and
  `aria-current="page"` from the router, on the static paint too.
- `Option("value", "Label")` submits the value and shows the label.
- Theme token and property names may have digit-leading segments
  (`viz-1`, `radius-2xl`).

### Config

`meta.fonts` and `meta.stylesheets` add `<link rel="preconnect">` and
`<link rel="stylesheet">` to both HTML shells; the CSP policy widens for those
origins.

### Language

Two-parameter lambdas `(a, b) => expr`; a keyword after a dot (`Array.from`);
`Number`, `String`, `Boolean`, `Map` as globals; a map literal as an arrow body;
store `derived` values may read other derived values and actions.

## Fixed

- `Icon("home")` rendered the word "home"; only `icon:` drew the glyph.
- `IconButton` dropped every named argument but `icon` and `label`, and the
  static backends painted the label as visible text next to the glyph. It is
  the accessible name (`aria-label`, `title`) only, and the glyph is drawn once.
- `Progress(value: expr)` was painted once and never moved.
- Lambda parameters inside pages compiled as signal reads (`_x()`).
- An action parameter named like another action read the store member
  (`move(step)` beside `step`).
- `placeholder`, `disabled`, `src` and the other recognised attribute names
  were set once even when bound to state.
- A `<select>` bound to state always showed its first option: the value was
  applied before the options existed (runtime), and a slot inside a `Select`
  component was appended after the value (codegen).
- An `else if` chain with a final `else` skipped every branch.
- A `Slider` or `Input` with both `bind:` and `on:input` kept only one.
- A `Button`'s block that mixed content with an action ran the action at
  render time; the action statements are its click handler now.
- A sub-component dropped its style block, attributes and handlers in the SPA.
- `createStore` bound derived values before actions, so a derived value that
  called an action threw.
- The router never set `document.title`.
- `wf build` did not run the V01/V02 vocabulary lint or the semantic check;
  it does, with per-file locations.
- The catch-all page was written to a directory literally named `*`. It is
  `404.html` at the root, and it addresses its assets from the site root, so a
  dynamic route under a static build hydrates.
- `wf serve` did not serve a pre-rendered `<route>/index.html`.
- The heading-outline lints (A11, A12) could not see headings inside
  components; A03 did not accept `aria-labelledby`; A13 warned about a white
  label on `--color-primary` even when no `primary` variant was used.
- In structural mode the sheet stacked every `Row` under 768px; it no longer
  paints layout decisions the author did not make.

---

# WebFluent v2.0 Release Notes

A release about correctness. The language gained themes and SEO; the output
gained the accessibility, responsiveness and search metadata it should always
have had. Several long-standing bugs are fixed, and the tests that would have
caught them now exist.

**This release contains breaking changes.** See *Migrating* at the end.

---

## Breaking changes

### Themes are written in WebFluent

The four built-in palettes (`default`, `dark`, `minimal`, `brutalist`) are gone
from the compiler. A theme is source now:

```wf
Theme Brand {
    token color-primary: "#0F766E"
    token radius-md: "14px"
}
```

Declare one and it is used; declare several and select with
`"theme": { "name": "Brand" }`. Tokens you do not name keep their baseline value.

`dark`, `minimal` and `brutalist` ship as `examples/themes/*.wf` — copy one into
your `src/` and the existing `"name"` in your config keeps working. A config that
still names a removed palette **fails the build** with a message pointing at the
replacement, rather than silently rendering as the default.

`theme.mode` and `theme.extends` were declared in config and never read by
anything. Both are removed.

### `Modal` and `Dialog` are native `<dialog>` elements

They were `div`s with an `.open` class, which meant no focus trap, no inert
background, no Escape-to-close and no `aria-modal`. They now use `showModal()`,
so the browser supplies all four. If you styled `.wf-modal` directly, note that
it no longer positions or hides itself — the element does that — and the backdrop
is `::backdrop`.

### Library signature changes

- `generate_css(tokens)` / `generate_css_with(tokens, builtin)` take resolved
  tokens rather than a theme name. Use `themes::resolve_tokens(program, config)`.
- `render_page_html` / `render_page_html_studio` take a `SiteContext` instead of
  five separate parameters.
- `generate_html(config, program)` takes the program, for the page metadata.
- `Template::with_theme(name)` selects a `Theme` declared in the template rather
  than naming a built-in palette.

---

## Fixed

- **`for` loops produced bundles that crashed.** The loop bound a plain callback
  parameter but every reference to it was emitted as a signal read, so any
  non-empty list threw `ReferenceError` on first render. `wf init -t spa` shipped
  this.
- **Zero-width spaces in every static page.** The SSG and template renderers
  emitted U+200B after every closing tag — invisible in an editor, present in
  `textContent`, in copied text and in what a crawler indexed.
- **`Heading("x", h1)` did not produce an `<h1>`** under `wf build`. The static
  renderers switched the tag; the SPA codegen never did, and emitted a
  `wf-heading--h1` class no stylesheet defines.
- **`style { }` blocks were dropped on nineteen components** — every one handled
  by a dedicated emitter, because styling was applied after the dispatch that
  returned early.
- **The sidebar rendered transparent** — no background at all in structural mode,
  and surface-on-white behind a 1.23:1 border in the full one.
- **Navigation vanished on a phone.** The sidebar was `display: none` below 768px
  and navbar links below 480px, with no control to bring either back. Both are
  off-canvas now, with a toggle, a scrim, Escape-to-close and focus return.
- **Structural mode had no focus indicator at all**, and `.wf-input` /
  `.wf-select` removed the outline without replacing it visibly.
- **`theme.builtin: "structural"` never reached `wf build`** — only the studio
  honoured it.
- **The language server had not compiled for several releases.** Nothing in the
  repository built it; `cargo build` at the root builds the root package alone.
  Its completion list had drifted to 43 missing modifiers and eight invented ones.
- **`Link("Home", to: "/")` drew a spurious accessibility warning**, and both the
  PDF and slides scaffolds warned on their own first build.
- **PDF images ignored their `alt`** and drew `[Image]` regardless;
  `Progress(max: 0)` rendered a full bar instead of an empty one.

---

## Added

### Search and sharing

Set `meta.site_url` and every page gets a self-referencing absolute canonical,
Open Graph and Twitter card tags, `hreflang` alternates with `x-default`, and
JSON-LD (`WebSite`, `Organization`, `WebPage`/`Article`, `BreadcrumbList`). Builds
write `sitemap.xml` and `robots.txt`.

`Page` accepts `description`, `image`, `type` and a bare `noindex`.

Without `site_url`, tags that need an absolute URL are omitted rather than
guessed at — a relative canonical causes problems later.

### Accessibility

Tabs carry the full `tablist`/`tab`/`tabpanel` wiring with roving tabindex and
arrow keys. Dropdowns report `aria-expanded` and close on Escape. Toasts and
alerts are live regions. Every page has a `<main>` landmark and a skip link.
Animations respect `prefers-reduced-motion`. Pointer targets meet the WCAG 2.2
minimum of 24×24.

### Performance

The type and spacing scales are fluid (`clamp()`), so a page responds without a
media query — and without losing to an inline `style { font-size: … }`. Images
default to `loading="lazy"` and `decoding="async"`. The bundle loads with
`defer`. The default font stack no longer names webfonts nothing loads.

The static renderer now paints lists and conditionals over seeded data instead of
leaving `<!--wf-for-->` for the client.

### Security

`"build": { "csp": true }` emits a strict CSP and a `_headers` file. The dev
server sets `nosniff`, `Referrer-Policy` and `X-Frame-Options`.

### Diagnostics

`A13` (theme contrast, checked at compile time because themes are now source),
`S01`–`S04` (title, description, description length, duplicate route) and `V02`
(a real modifier whose class no stylesheet defines).

### For tooling

`CompiledSite` carries the resolved tokens, the declared themes with their spans,
flattened diagnostics, and the reason a theme failed to resolve.

---

## Migrating

1. **If your config names `dark`, `minimal` or `brutalist`:** copy the matching
   `examples/themes/*.wf` into your `src/`. Your existing `"theme": { "name": … }`
   then resolves to it. The build tells you this if you forget.
2. **Remove `theme.mode` and `theme.extends`** if present; they are ignored.
3. **If you styled `.wf-modal` or `.wf-dialog` directly**, re-check it against a
   native `<dialog>`.
4. **Add `meta.site_url`** to get canonical URLs, sharing cards and a sitemap.
   Nothing breaks without it; those tags are simply not emitted.
5. **Expect new warnings.** `S02` fires on every page with no description. None
   of them fails a build.
