# WebFluent — Agent Role File

You are an expert WebFluent developer. WebFluent is a web-first programming language that compiles to HTML, CSS, JavaScript, and PDF. You write `.wf` source files and configure projects via `webfluent.app.json`.

## Project Structure

```
project/
├── webfluent.app.json        # Config: theme, build, i18n, meta
├── src/
│   ├── App.wf                # Root: navbar, router, footer
│   ├── pages/                # One .wf per page
│   ├── components/           # Reusable components
│   ├── stores/               # Shared state stores
│   ├── *.css                 # The project's own stylesheets, anywhere under src/
│   └── translations/         # i18n JSON files (en.json, ar.json)
├── public/                   # Static assets → copied to build root
└── build/                    # Compiled output
```

Files in `public/` are copied to the **root** of the build output (not nested under `public/`). Example: `public/logo.png` → `build/logo.png`.

## CLI

```bash
wf init <name> -t spa|static|pdf|slides   # Create project
wf build [-d DIR] [--stats]               # Compile (--stats: what it weighs)
wf serve [-d DIR]                         # Dev server (localhost:3000): rebuilds on save, reloads the page, shows a failed build's error over it
wf generate page|component|store <name>   # Scaffold a file, in the project's layout
wf fmt [path] [--check] [--stdout]        # Format a project's sources (--check fails when one would change)
wf fmt --to wfx|wf [path] [--stdout]      # Switch a project between .wf and .wfx
wf test [path] [--update]                 # Run the `test "…" { }` declarations under tests/
wf docs [-d DIR] [-o OUT]                 # A component gallery: every built-in, and what the project declares
wf migrate [path] [--check] [--wfx]       # WebFluent 2 → 3 (and to .wfx), then 3 → 4
wf verify [path] [--json] [--budget MS]   # Load every page in a real browser: errors, failed requests, paint timing
wf audit [path] [--json]                  # What the project trusts: markup, origins, storage, env, policy, dependencies
wf registry [--json]                      # Every built-in: props, cases, flags, events, slots, parts
wf types [path] [--json]                  # What a project declares: enums, types, components, stores, pages
```

`wf verify` builds nothing — it loads what `wf build` already wrote. It
starts a headless Chrome, serves the output, opens every route the
project has, and fails on an uncaught exception, a `console.error`, a
file that did not arrive, an image that failed, or a page that rendered
no text. It prints the first-contentful-paint, the node count and the
bytes each route fetched, says which built-ins no page drew, and takes
`--budget MS` to fail a route that paints too slowly. `--json` is the
same report for a pipeline. It needs a Chrome or Chromium on the machine
(or `WF_CHROME` pointing at one) and says so plainly when there is none.

```
  16 route(s) in http://127.0.0.1:39785
    ok   /                                    236ms     613 nodes    447.0 kB   11 req
    ok   /app/deployments                      68ms     657 nodes    360.6 kB   10 req

  16 page(s), 0 problem(s)
```

`wf serve` puts a **stores** button in the corner of every page it serves:
what each store that has been built is holding, a log of every action with
the state on each side of it, and a click to put a store back to before one
ran.

`wf serve` builds first, then watches `src/`, `public/` and the config
(`dev.hot_reload`, on by default): a save rebuilds, and every page it
serves carries `/__wf/dev.js`, which reloads the page when a build lands
and draws the error of a build that failed over the page — with the
diagnostics — until the next build passes. `/__wf/status` is what it
polls.

`wf test` renders every `test "name" { … }` declared under `tests/` (or in
`src/`) through the template engine, with the project's components,
stores, types and constants at hand, and holds the render to what it
expects and to a snapshot in `tests/__snapshots__/<file>/<name>.html`
(written when missing, rewritten with `--update`). A build ignores tests.

```wf
test "greets by name" {
    Greeting("Sam")
    expect "Hello, Sam"
    expect not "Hello, world"
}
test "lists the data"(data: { items: ["a", "b"] }) {
    state open = true
    for it in items { if open { Text(it) } }
    expect "b"
}
test "reporting an incident opens one more" {
    use IncidentStore
    state draft = ""
    Text("{IncidentStore.open} open")
    Input(bind: draft, label: "What happened")
    Button("Report") { on click { IncidentStore.report(draft) } }

    expect "2 open"
    type "Queue backing up" into "What happened"
    click "Report"
    expect "3 open"
}
```

A test with a step that **acts** is compiled into a real page, served and
run in a headless Chrome (or `WF_CHROME`), because a click runs a handler,
which runs an action, which repaints; a test that only looks needs no
browser and is compared to its snapshot as well. The steps run in the
order written, everything is found by the name a reader sees (a button's
text, a control's label, placeholder or `aria-label`), and a handler that
throws fails the test.

| Step | What it does |
|---|---|
| `expect "text"` · `expect not "text"` | What the page must, or must not, show |
| `click "Save"` | Clicks whatever carries that name |
| `type "Ada" into "Name"` | Types into the control that label names |
| `press "Enter"` · `press "Escape" in "Search"` | A key, on the focused element or a named one |

`wf docs` writes `docs/index.html`: a self-contained gallery of every
built-in — props, cases, flags, events, slots, parts — and of the
project's own components, enums, types, stores and pages, read from the
same registry the compiler and the editor read.

`wf types --json` lists a project's `external` declarations beside its own,
so a tool knows what a call site may pass to either.

`wf registry --json` and `wf types --json` are the registry and a project's
declarations as a tool reads them — the studio's inspector offers a dropdown
for an enum prop, a toggle for a flag, a handler editor for an event and a
drop target for a slot from these, and a call site of the project's own
`UserCard` reads like one of `Button`.

## Core Syntax

One lexeme, one meaning. Declarations are lowercase keywords; elements are
capitalised; a `.flag` sets a boolean prop or picks a case; `name: value`
passes a prop; a block is the element's children; `on click { }` is a
handler; `style { }` is raw CSS. What a component takes — its props, cases,
events, slots and parts — comes from one registry the compiler, the
linters and the language server all read, and anything that resolves to
nothing is an error, never a silent no-op.

### Pages

```wf
page Home(path: "/", title: "Home") {
    state count = 0

    Container {
        Heading("Welcome").h1
        Text("Count: {count}")
        Button("+1").primary { on click { count = count + 1 } }
    }
}
```

- `path` — URL route. A dynamic segment is a typed parameter of the page:
  `page User(path: "/user/:id", id: String) { Text(id) }`
- `title` — Browser tab title, and the heading of a search result; may name
  the page's parameters, `title: "{slug} — Blog"`, or splice an expression
  over them and the constants: `"{posts.find(p => p.slug == slug)?.title ?? slug} — Blog"`
- `description` — The snippet a search result and a link preview show
- `image` — Link-preview image, site-relative or absolute
- `type` — `"website"` (default) or `"article"`
- `noindex: true` — keeps the page out of search results and out of the sitemap
- `layout: Shell(crumb: "Home")` — the component that frames the page; the
  page renders in its default slot
- `guard` — Expression that must hold for the route to render
- `redirect` — Where to send the visitor when the guard fails

```wf
page Post(path: "/blog/slow-roasting", title: "Slow roasting, explained",
          description: "Why we take eleven minutes over it.",
          type: "article", image: "/roast.png") { Container { Text("…") } }
```

Pages own their routes: `app` places a bare `Router`, and the table is built
from every page's `path`, ordered by specificity (static segments before
`:params`, `*` last), so declaration order cannot shadow a route.

A page may add tags of its own to the document's head — painted at build
time where the value is known, kept current on the live page, and gone
when the page is:

```wf
page Post(path: "/p/:slug", title: "Post", slug: String) {
    head {
        meta(property: "og:image", content: post.image)
        link(rel: "canonical", href: "https://example.com/p/{slug}")
        script(src: "/analytics.js", defer: true)
    }
    …
}
```

### Components

```wf
/// A person, at a glance.
component UserCard(_ name: String, role: String, active: Bool = true) {
    Card.elevated {
        Row(align: .center, gap: .md) {
            Avatar(initials: "U").primary
            Stack {
                Text(name).bold
                Text(role).muted
            }
            if active {
                Badge("Active").success
            }
        }
    }
}

// Usage
UserCard("Monzer", role: "Developer")
UserCard("Sam", role: "Designer").active
```

- Prop types: `String`, `Number`, `Bool`, `Map`, `Any`, `[T]` (a list), `T?`
  (may be null), or a declared `type` or `enum`
- `_ name: Type` — the one prop a call may pass positionally; it must be first
- Default values: `active: Bool = true`; a `Bool` prop is set with a flag
  (`.active`), an enum prop with a flag or by name (`.loud`, `tone: .loud`)
- A `///` comment above a declaration, a prop or an event is its
  documentation, shown on hover

A prop that reads state stays live inside the component: `Chip(pressed:
showErrors)` repaints when `showErrors` changes.

#### Slots

A component places its caller's block with `children`, and may declare
named slots the caller fills with `name { … }`. A fill is compiled in the
caller's scope, so it reads the caller's state and loop bindings; the
component only decides where it lands.

```wf
component Panel(_ title: String) {
    slot trailing
    Card {
        Row(justify: .between) { Heading(title).h3  trailing }
        children
    }
}

Panel("Keys") {
    trailing { Badge("Beta") }
    Text("Rotate every 90 days.")
    Button("Generate").primary { on click { generate() } }
}
```

A page's `layout:` is a component with a default slot, called with the page
as its block.

A **scoped slot** hands its fill values: the component declares what it
hands over, uses the slot with those values by name, and the caller's fill
names them in order.

```wf
component Rows(items: [Todo]) {
    slot row(item: Todo, index: Number)
    for it, i in items by it.id { row(item: it, index: i) }
}

Rows(items: todos) { row(t, i) { Text("{i}: {t.title}") } }
```

A component may declare **parts** — components of its own, called under
its name:

```wf
component Panel(_ title: String) {
    part Header(_ text: String) { Heading(text).h3 }
    part Footer { Divider }
    Card { Text(title)  children }
}

Panel("Keys") { Panel.Header("Rotate")  Text("…")  Panel.Footer }
```

A component's enum props are written to its root element as
`data-<prop>="<case>"` (`data-tone="loud"`), so a stylesheet can select on
them.

#### Events

A component declares the events it fires and their arguments; `emit` fires
one; the caller handles it with `on name(args) { }`. A DOM event written on
a component call — `on click { }` — attaches to the component's root
element, so a styled button component is clickable wherever it is used.

```wf
component TodoRow(_ label: String, done: Bool = false) {
    event toggle(id: String)
    Row {
        Checkbox(checked: done) { on change { emit toggle(label) } }
        Text(label)
    }
}

TodoRow("Milk") { on toggle(id) { Todos.toggle(id) } }
```

### Types and enums

```wf
enum Tone { calm, loud }
type Todo { id: String, title: String, done: Bool = false, tone: Tone = .calm, note: String? = null }

store Todos {
    state items: [Todo] = []
    action add(title: String) { items = items.concat([Todo(id: uuid(), title: title)]) }
}
```

### The types the language brings with it

Eleven types come with the language. Each is a **plain JSON value at run
time** — no wrapper — so it crosses a `fetch`, a `persist`, the static
paint and the template engine unchanged, and `.native()` hands a real
platform object to any library that wants one.

| Type | Carried by | Written | What it can do |
|---|---|---|---|
| `Date` | `"2026-03-14"` | `@2026-03-14` | `.year() .month() .day() .weekday()`, `.plus(days: 3)`, `.minus(…)`, `.isBefore(d) .isAfter(d) .isSame(d)`, `.until(d)`, `.startOfDay() .startOfWeek() .startOfMonth() .endOfDay()`, `.native()` |
| `Time` | `"09:30"` | `@09:30` | `.hour() .minute() .second()`, `.plus(minutes: 30)`, the comparisons |
| `DateTime` | ISO 8601 | `@2026-03-14T09:30Z` | the above, `.date() .time()`, `.inZone("Europe/Berlin")` |
| `Duration` | milliseconds | `3.days`, `90.minutes`, `250.ms` | `.days() .hours() .minutes() .seconds() .ms()` |
| `Money` | `{ amount: 1299, currency: "EUR" }` | `€12.99`, `$9.99` | `.plus(m) .minus(m) .times(n) .convert(rate, "USD")`, the fields `.amount` and `.currency`; `format(m)` needs no style |
| `Url` | a string | `"https://example.com"` | `.host() .path() .query()`, `.with(query: { page: 2 })` |
| `Email` | a string | `"ada@example.com"` | `.domain()` |
| `Color` | a string | `#0F766E` | `.mix(other, 0.2) .lighten(n) .darken(n) .alpha(0.5) .contrast(other)` |
| `Uuid` | a string | `uuid()` makes one | the methods of a string |
| `File` | the browser's `File` | what `FileUpload` yields | `.preview()`, the fields `.name .size .type` |
| `Secret` | a string | — | nothing: it may not be shown, logged, persisted or put in text |

Minor units for money, so nothing drifts. Milliseconds for a duration, so
it adds to a moment. Everything the language computes is a **call**;
everything the carrier already has is a **field** — so `price.amount` is a
field and `due.year()` is a call.

```wf
type Booking {
    id: Uuid
    guest: Email
    from: Date
    nights: Number(1..=30)
    total: Money
}

state due: Date = @2026-03-14
derived checkout = due.plus(days: 5)          // 2026-03-19
derived late = now.date().isAfter(checkout)
Text("Due {format(total)} on {checkout:.date(long)}")
```

A literal written where one of these is wanted is read as one and held to
its shape: `state site: Url = "example"` is an error where it is written.
A `type` the program declares wins over the name — a project with its own
`type Color { … }` means that one.

`now` joins `viewport`, `query`, `hash` and `theme` as a browser value: a
`DateTime` that keeps itself current, every minute by default, or as often
as a page asks — `now(every: 1.seconds)`.

**A condition on a type.** Any type may say what its values must be, and
the checker holds every value it can read to it:

```wf
state nights: Number(1..=30) = 12
state pass: String(minLength: 8) = "correcthorse"
state start: Date(after: @2026-01-01) = @2026-03-14
```

`min`, `max`, `minLength`, `maxLength`, `pattern`, `after`, `before`, and a
range for the two ends. A value that arrives at run time is validation's to
refuse, from the same condition.

**A secret is not a string.** `Secret` is refused by the compiler wherever
it would escape: shown by an element, spliced into text (which is how one
ends up in a URL), logged, or kept with `persist`. Each is a `T12`.

A record is built with named fields, `Todo(id: "1", title: "x")`; a case is
written `.calm`. The type checker reads every declared type and infers the
rest (literals, records, lists, the built-in methods of strings, numbers and
lists, store members, lambdas). Anything unresolved is `Any`, which agrees
with everything, so a program that declares no types checks as it always
did; every annotation narrows what the checker can say. See
[Compiler diagnostics](#compiler-diagnostics) for what it reports.

A record may extend another, and a case may carry a payload:

```wf
type Admin = User { role: String }                       // every field of User, plus role
enum Status { idle, failed(reason: String), done(count: Number, label: String) }

state s: Status = .idle
action fail() { s = .failed("timed out") }
match s {
    .idle { Text("…") }
    .failed(r) { Text("Failed: {r}") }                   // one name per part of the payload
    .done(n, l) { Text("{n} {l}") }
}
derived note = match s { .failed(r) { r } else { "" } } // one name in a match expression
```

A case with a payload is `["failed", "timed out"]` at run time and a bare
case its name, so `s == .idle` still compares. A map literal has the
shape it was written with: `state form = { name: "", age: 0 }` lets the
checker read `form.name` and refuse `form.nam` (`T05`); a field only some
items of a list have is `T?`; an empty `{}` or a spread says nothing about
the keys.

### Constants and `env`

```wf
const API = "/api/v1"
const PAGE_SIZE: Number = 20
resource rows = fetch("{API}/rows?limit={PAGE_SIZE}")
Text(env.PUBLIC_APP_NAME)
```

`const` declares a top-level value read everywhere; `env.X` reads the
`"env": { "PUBLIC_APP_NAME": "…" }` map of `webfluent.app.json`, the
shell's variables and a `.env` file, fixed at build time. A page, a
component, a store or an `api` may only read a name that begins `PUBLIC_`
or that `public_env` lists — everything it reads is in the bundle, so any
other name there is a compile error.

### Images the build processes

`image hero = "media/hero.jpg"` reads the picture at build time and writes
it again at every width a page will ask for:

```wf
image hero = "hero.jpg"

page Home(path: "/", title: "Home") {
    Image(hero, alt: "The team at the launch",
          sizes: "(max-width: 768px) 100vw, 1200px",
          placeholder: .blur)
    Text("It is {hero.width} by {hero.height}, and mostly {hero.color}.")
}
```

What the build writes: `hero.<hash>.<width>.webp` at each width in
`media.widths` that is smaller than the original (never up-scaled), plus
the original under its own hashed name — so a host can cache every one of
them forever. What the page gets is a `<picture>` with a `<source>` per
format, `width` and `height` from the real file (so nothing shifts when it
arrives), and a placeholder: `.blur` inlines a 16-pixel-wide copy as a data
URI, `.color` fills the box with the image's average colour, `.none` does
neither.

The name is a value too: `hero.src`, `.width`, `.height`, `.color`,
`.placeholder`, `.srcset`.

```json
{ "build": { "media": {
    "formats": ["webp"], "widths": [480, 960, 1440, 1920],
    "quality": 78, "pipeline": true
} } }
```

`media.pipeline: false` copies the file as `public/` always did. The work
is cached in `.wf-cache/media/` by content hash, so a build that changes no
image does no image work. The PDF and slides backends embed the real
picture — the grey `[Image]` rectangle is gone.

`data posts = "posts.json"` (or `data posts: [Post] = "content/posts.json"`)
is a constant whose value is a JSON file's, read at build time from the
project directory or its `src/`; it is inlined into the bundle and seeded
into the static paint. A page on a `:param` route names the values a
static build renders it for with `paths:`, one value per parameter (a map
by name for several):

```wf
data posts = "posts.json"
page Post(path: "/p/:slug", title: "Post", slug: String, paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    Heading(post?.title ?? "?").h1
}
```

The static build writes `p/<slug>/index.html` for each, with `slug` and
`params.slug` seeded, and lists them in the sitemap.

A `.md` file under `src/` is a page: front matter between `---` lines
names `path` (default `/<stem>`, `/` for `index`), `title`, `description`,
`layout`, `image`, `type` and `noindex`; the rest is its Markdown, placed
as `Markdown(…)` in the page's body (inside the layout's default slot when
one is named).

```md
---
title: About us
description: Who we are.
layout: Shell
---
# About

We make *things*.
```

### Attributes

Any named argument a built-in does not declare is written to its root
element as an HTML attribute, with a warning; `aria-*`, `data-*` and the
global attributes (`id`, `role`, `tabindex`, `title`, `hidden`, `lang`,
`dir`) are declared families and draw none:

```wf
Button("Errors", aria-pressed: showErrors, data-tone: "danger") { on click { showErrors = !showErrors } }
Table.Row(aria-selected: isSelected) { Table.Cell("…") }
Badge("Ready", data-tone: "success")
```

A value that reads state follows it. `aria-*` keeps a `false` value as the
string `"false"` (a real ARIA state); any other attribute given `false` is
omitted.

`class:` does not replace the element's classes, it adds to them.
`Card(class: "feature wide")` renders `class="wf-card feature wide"`, and a
value that reads state is followed — the classes it named last time come
off, the ones it names now go on. It is how an element picks up a rule from
the project's own stylesheets (see [Stylesheets](#stylesheets)).

### App (Router + Layout)

`app` is the root of the site: what wraps every page, and a bare `Router`
where the current page renders. The Router can sit at any depth.

```wf
// Simple: Router at top level
app {
    Navbar {
        Navbar.Brand { Link("Ledger", to: "/") }
        Navbar.Links { Link("Home", to: "/") }
    }
    Router
}
```

```wf
// Sidebar layout: Router nested inside Row > Container
app {
    Row {
        style { min-height: 100vh }

        Sidebar {
            style {
                width: 220px
                position: fixed
                top: 0
                left: 0
                bottom: 0
                background: #1A1A19
            }
            Sidebar.Header { Text("My App").bold }
            Sidebar.Item(to: "/", icon: "home") { Text("Home") }
            Sidebar.Item(to: "/about", icon: "info") { Text("About") }
        }

        Container {
            style {
                margin-left: 220px
                flex: 1
            }

            Router
        }
    }
}
```

`Router(transition: .fade, duration: "200ms")` plays the old page out and the
new one in on a route change (`.slide` slides them). A page that names a
`layout:` is framed by that component instead of, or as well as, the app's
own chrome.

### Stores (Shared State)

A store is **built the first time something reads it**. Declaring one does
nothing, so the order stores are declared in cannot matter — a `derived`
may read a store declared below it — and a store nothing reads costs
nothing, in time or in bytes.

```wf
store Cart(scope: .app) { … }         // the default: as long as the page is open
store Session(scope: .session) { … }  // this tab's; what it keeps goes in the tab's storage
store Filters(scope: .route) { … }    // this route's; dropped when the route changes
store Boot(eager: true) { … }         // built at boot, for set-up that must run
```

```wf
store CartStore {
    state items = []
    state total = 0

    derived count = items.length

    action add(product: Map) {
        items = items.concat([product])
        total = total + product.price
    }

    action getHeaders() {
        let h = {}
        h["Authorization"] = "Bearer " + accessToken
        return h
    }

    action clear() {
        items = []
        total = 0
    }
}

// Usage in any page/component:
page Shop(path: "/shop") {
    use CartStore
    Text("Cart: {CartStore.count} items")
    Button("Clear") { on click { CartStore.clear() } }
}
```

### State & Reactivity

```wf
state count = 0                      // Signal — reactive variable
state draft: String = ""             // with a declared type
derived double = count * 2           // Computed — auto-updates
effect { log(count) }                // Side effect — runs on change
```

- UI elements that reference state variables auto-update when values change
- String interpolation: `"Hello, {name}!"` — reactive in UI elements
- A call at the top of a page or component — `Store.load(id)` — is set-up
  code, run once when it renders

```wf
persist theme = "light"               // kept in the browser across visits (page, component or store)
persist items: [Item] = [] {          // and what a value says about itself
    in: .local                        // .local (default) · .session (this tab)
    version: 2                        // the version of the shape this build writes
    sync: false                       // don't adopt what another tab wrote
    migrate 1 -> 2 { old.map(i => Item(id: i.id, qty: i.count)) }
}
every(1000) { tick = tick + 1 }       // a timer; stops when its page, branch or item leaves
after(3000) { toast = null }          // once
effect { watch(count)  cleanup { unwatch() } }   // run before the next run, and on leaving
Input(bind: q, ref: search)           // a handle on the element: search.focus(), search.value
Text(if viewport.md { "wide" } else { "narrow" })   // viewport.width/height/sm/md/lg/xl, kept current
Text(query.tab ?? "all")              // the URL's ?tab=…, and `hash` its #fragment
```

What a page, a branch, a list item, a match arm or a slot creates — effects,
timers, listeners, handles — is disposed of when it leaves; an async action's
`name.pending` is true while a call of it runs (`Button("Save",
disabled: save.pending)`, `Api.sync.pending` on a store).

### Events

```wf
Button("Click") {
    on click { doSomething() }
}
Input(bind: query).text {
    on input { search(query) }
}
Form {
    on submit(event) { event.preventDefault()  saveData() }
}
Input(bind: q) {
    on keydown(e) { if e.key == "Escape" { q = "" } }
}
```

DOM events: `click`, `input`, `change`, `submit`, `focus`, `blur`,
`keydown`, `keyup`, `keypress`, `mouseenter`, `mouseleave`, and every other
standard one — `dblclick`, `contextmenu`, `pointerdown`, `pointermove`,
`touchstart`, `wheel`, `scroll`, `paste`, `drop`, `dragover`, `ended`,
`animationend`, … (`wf registry --json` lists them). The handler's
parameter names the DOM event; without one, `event` is in scope. An
element's block is ordered `style` → `transition` → `on …` → slot fills →
children.

`on key("ctrl+k") { … }` answers to one key: modifiers `ctrl`, `shift`,
`alt`, `meta`/`cmd`, then the key as the browser names it (`k`, `Enter`,
`Escape`/`esc`, `ArrowDown`/`down`, `space`). On an element it listens
there; at the top of a page it listens on the document for as long as the
page shows; `on key("Escape", e)` names the event too.

`Form(bind: form)` hands a handle on the form: `form.valid`, `form.errors`
by field name, `form.touched`, `form.pending` (an async rule is still
asking), `form.values` by field `name:`, `form.reset()`, `form.submit()`,
and `form.apply(serverErrors)` for what a 422 said.

### What a value must be

A `validate` block sits beside the state it guards. The control bound to
that state then shows what the rules say — no `error:` to write, and the
accessible plumbing (`role="alert"`, `aria-invalid`, `aria-describedby`)
is the same one `error:` has always used.

```wf
api Backend(base: "/api") {
    get emailTaken(email: String) -> Bool
    post signUp(body: Map)
}

page Signup(path: "/join", title: "Join", description: "Make an account.") {
    state email = ""
    state password = ""
    state confirm = ""
    state note = ""

    validate email {
        required
        email
        async "That address is taken" { !(await Backend.emailTaken(email: email)) }
    }
    validate password {
        required
        minLength(8) "Use at least 8 characters"
        pattern(/[0-9]/) "Include a number"
    }
    validate confirm { matches(password) "The two passwords differ" }

    Form(bind: form, show: .onBlur) {        // .onBlur (default) · .onSubmit · .live
        on submit { Backend.signUp(body: { email: email, password: password }) }
        Input(bind: email, label: "Email").email
        Input(bind: password, label: "Password").password
        Input(bind: confirm, label: "Confirm").password
        Textarea(bind: note, label: "Anything else?", rows: 4, maxLength: 200)
        Button("Join", type: .submit, disabled: !form.valid || form.pending).primary
    }
}
```

The rules: `required`, `email`, `url`, `minLength(n)`, `maxLength(n)`,
`min(v)`, `max(v)`, `pattern(/…/)`, `matches(other)`, `oneOf([…])`,
`custom "…" { expr }`, `async "…" { await … }`. Each takes an optional
message; without one the rule's own is used, which a project's
translations replace by naming `form.required`, `form.email` and so on.
A rule is checked against what it guards — `minLength` on a `Number` is
`T01` — and every rule but `required` passes an empty value, so a blank
optional field is one message, not two.

**A refined type validates itself.** `state nights: Number(1..=30)` needs
no rule written for the range; the condition on the type is already one.

A submit that fails shows every message, moves focus to the first field
that has one and announces it, and does not run the handler.

### Actions, `let`, `return` and `await`

```wf
store AuthStore {
    state accessToken = ""

    action getHeaders() {
        let h = {}
        h["Authorization"] = "Bearer " + accessToken
        return h
    }

    action load() {
        let r = await fetch("/api/me")
        accessToken = r.token
    }
}
```

`let` declares a local of an action or handler; `await` inside one makes
it async; `return` leaves it with a value or without. `for` loops (with an
index, over a range), `try { } catch e { }` and destructuring `let` are
imperative statements too:

```wf
action sync() {
    try {
        let r = await fetch("/api/sync")
        let { items, total } = r
        let [first, second] = items
        for item, i in items { log("{i}: {item.title}") }
        for n in 1..=total { count = n }
    } catch e {
        error = e.message
    }
}
```

### Browser Globals

Standard browser APIs are available without any prefix — they compile directly to their JavaScript equivalents:

```wf
// Storage
localStorage.setItem("token", tok)
sessionStorage.getItem("key")

// Window & Document
window.open("https://example.com")
document.title = "New Title"

// Console
console.log("debug info")

// JSON
payload = JSON.parse(responseText)
text = JSON.stringify(obj)

// Timers
setTimeout(callback, 1000)

// Other globals: Math, Date, Array, Object, Promise, Error,
// parseInt, parseFloat, encodeURIComponent, atob, btoa, fetch,
// alert, confirm, prompt, RegExp, Map, Set, etc.
```

### Expressions

`x => expr` and `(a, b) => expr` are lambdas with one expression as the
body, returning a map with `(x) => { key: value }`; `a ?? b` takes `b` when
`a` is null; `if c { a } else { b }`, `if let x = v { a } else { b }` and
`match t { .calm { 1 } else { 2 } }` are values; `$token` is a design
token; `.case` is a case of an enum; `a?.b`, `a?.m()`, `a?.[i]` read
through null; `/…/flags` is a regular expression; `...x` spreads a list or
a map; `a..b` and `a..=b` are ranges.

```wf
derived open = incidents.filter(i => !i.resolved)
derived byAge = rows.slice().sort((a, b) => a.age - b.age)
derived label = if let s = selected { s.title } else { "none" }
derived city = user?.address?.city ?? "—"
derived valid = /^[\w.]+@[\w.]+$/.test(email)
derived slug = title.toLowerCase().replace(/[^a-z0-9]+/g, "-")
derived merged = { ...defaults, ...overrides, open: true }
derived all = [...pinned, ...rest]
derived pages = 1..=pageCount
derived byTeam = rows.groupBy(r => r.team)
derived top = rows.sortBy(r => -r.score).take(3)
derived names = rows.map(r => r.name).unique().join(", ")
derived short = description.truncate(80)
```

List helpers: `sortBy(f)`, `groupBy(f)` (a map of lists), `unique()`,
`take(n)`, `first()`, `last()`, `flatMap(f)`, `sum()`, beside the
JavaScript methods; string helpers: `capitalize()`, `truncate(n)`,
`dedent()` (the common indentation off a block of text), `lines()`,
`words()`, and `match`, `replace`, `replaceAll`, `split`, `search` with a
regex. Every one runs at build time too, so the static paint shows the
result.

### Formatting

```wf
Text(format(price, .currency))                 // $1,234.50 — the page's locale
Text(format(price, .currency, "EUR"))          // €1,234.50
Text(format(count, .integer))                  // 1,235
Text(format(ratio, .percent, 1))               // 25.6%
Text(format(views, .compact))                  // 1.2K
Text(format(when, .date, "long"))              // March 5, 2024
Text(format(when, "yyyy-MM-dd HH:mm"))         // a pattern: yyyy yy MMMM MMM MM M dd d EEEE EEE HH H hh h mm ss a
Text(ago(when))                                // 3 minutes ago · yesterday · in 2 weeks
```

`format(value, .style, option)` and `ago(date)` speak the i18n locale when
the project has one (a change of locale redraws them) and the document's
language otherwise. Styles: `.number` (the default), `.integer`,
`.decimal` (option: places, 2), `.currency` (option: the code, `USD`),
`.percent` (of a fraction; option: places, 0), `.compact`, `.date`,
`.time`, `.datetime` (option: `short`, `medium`, `long`, `full`),
`.relative`. The static paint formats with a table of the common locales
and currencies and English names; the browser's `Intl` takes over on
hydration.

### Control Flow

```wf
// Conditional
if isLoggedIn {
    Text("Welcome!")
} else {
    Button("Log In") { on click { navigate("/login") } }
}

// A value that may be null, bound when it is not
if let user = session.user {
    Text(user.name)
}

// Loop, keyed: an item keeps its nodes across inserts, removals and moves
for item in items by item.id {
    Card { Text(item.name) }
}

// Loop with index
for item, index in items {
    Text("{index}. {item.name}")
}

// Show/hide (keeps in DOM, toggles visibility)
show isVisible {
    Modal { Text("Content") }
}

// One arm by a value: a resource's states, or an enum's cases
match tone {
    .calm { Text("Calm") }
    else { Text("Loud") }
}
```

### Data Fetching

```wf
resource users = fetch("/api/users")
match users {
    loading { Spinner }
    error(err) { Alert("Failed: {err.message}").danger }
    ready(users) {
        for user in users by user.id {
            Text(user.name)
        }
    }
}
```

A resource declares its type — `resource users: [User] = fetch(…)` — and
the `ready` arm's binding has it. Options: `fetch(url, method: "POST",
body: { key: value }, headers: { "Authorization": token })`. The request
repeats when a URL that reads state changes.

Map literals support **quoted string keys** for headers and special field names:

```wf
resource data = fetch("/api/users", method: "POST",
    headers: { "Content-Type": "application/json", "X-Api-Key": apiKey },
    body: { action: "create", token: sessionToken })
match data {
    ready(data) { Text("Done") }
    else { Spinner }
}
```

Reserved words (`action`, `token`, `error`, `state`, etc.) work as map keys.

### A service, described once

`api` is where a service lives: its address, how it is reached, and what it
has. Every call site is then typed, cached and cancellable, and the day the
server changes its contract the build says so.

```wf
type User { id: String, name: String }

api Backend(base: env.PUBLIC_API ?? "/api/v1") {
    timeout: 10.seconds
    retry: .backoff(times: 3, on: [.network, .timeout, .status5xx])
    credentials: .sameOrigin           // .omit · .sameOrigin · .include — the
                                       // session cookie, which is where a
                                       // session belongs; `persist` is not

    on request(r)  { r.headers["X-Request-Id"] = uuid() }
    on response(r) { log("{r.status} {r.url}") }

    get    users(page: Number = 1, q: String?)   -> [User]
    get    user(id: String) at "users/:id"       -> User
    post   createUser(body: Map)                 -> User
        errors { 422 -> Map }
    delete removeUser(id: String) at "users/:id"
    post   avatar(id: String, file: File) at "users/:id/avatar" -> Url
}
```

Or read the whole surface from a specification, and let the compiler check
every call against it:

```wf
api Backend from "openapi.json" (base: env.PUBLIC_API)
```

Every endpoint, parameter, response type and error shape comes from the
spec, and each named schema becomes a `type` the program can name.

An endpoint is called like anything else, and a `resource` over one is
typed by what it returns:

```wf
resource users = Backend.users(page: n, cache: .swr(60.seconds), on: .focus)

match users {
    loading { Skeleton }
    error(e) {
        match e {
            .offline { Alert("You are offline.").warning }
            .timeout { Alert("The server took too long.").warning }
            else     { Alert(e.message).danger }
        }
        Button("Try again") { on click { users.reload() } }
    }
    ready(list) { for u in list by u.id { Text(u.name) } }
}
```

What comes with it, at every call site:

| | |
|---|---|
| Abort | on scope leave, on an argument that reads state changing, on `.cancel()`, on timeout |
| Errors | `.offline`, `.timeout`, `.aborted`, `.parse`, `.network`, `.status(code, body)` — the body decoded to the declared type, and `.message`, `.status`, `.headers` to read |
| Retry | exponential backoff with jitter, per error kind, honouring `Retry-After` |
| Cache | `.swr(60.seconds)`, `.none`, `.forever` — keyed by method, address and body, with `ETag`/`If-None-Match` revalidation and `304` served from what is held |
| Dedupe | two readers of one address make one request |
| Refetch | `on: .focus`, `.reconnect`, `.interval(30.seconds)` |
| Pagination | `paginate: .page` gives `.items`, `.loadMore()`, `.hasMore` |
| Progress | a `File` parameter makes `Backend.avatar.progress` a `Number` signal |
| Bodies | JSON, text, a `File` (sent as a form), `FormData`, `Blob`, `ArrayBuffer`, `URLSearchParams` |
| Responses | JSON, `as: .text`, `.blob`, `.arrayBuffer`, `204` as no value, and `.lines()` for a streamed response |
| Elsewhere | `Backend.users.invalidate()`, `.prefetch(args)`, `.key(args)`, `.url(args)` |

`await fetch(url, options)` in an action goes through the same engine, and
`window.fetch` is untouched for anything that wants the browser's own.

### A connection the page holds open

A socket, a stream of events, a channel every tab hears, or a peer — a
direct line to another reader's page. Each is closed when the page that
opened it leaves, so a route change cannot leak one.

```wf
socket chat = ws("wss://example.com/chat", protocols: ["v2"], heartbeat: 20.seconds) {
    send    Outgoing                  // typed both ways
    receive Incoming
    on message(m) { Feed.add(m) }
}

match chat {
    connecting { Spinner.sm }
    open       { for m in chat.messages by m.id { Bubble(m) } }
    closed(c)  { Alert("Disconnected ({c.code})").warning }
    error(e)   { Alert(e.message).danger }
}
Button("Send") { on click { chat.send(Outgoing(text: draft))  draft = "" } }

stream ticks = sse("/events", events: ["price", "status"])
effect { if let p = ticks.last("price") { price = p } }

channel cart = broadcast("cart") { on message(m) { Cart.merge(m) } }
Button("Sync") { on click { cart.post({ items: Cart.count }) } }

beacon("/analytics", { event: "checkout", value: total })   // survives unload
```

A socket reconnects with backoff, keeps itself alive with a heartbeat, and
holds what was sent while it was down. A stream resumes where it was.

Each is a handle, not a signal: `.state` (what a `match` reads),
`.messages`, `.last(kind)`, `.error`, `.closure` (`.code`, `.reason`),
`.close()`, and `.send(v)` on a socket or a peer, `.post(v)` on a channel.

#### A peer

`peer` opens a WebRTC data channel straight to another page. The language
supplies the channel, not a server: what the two sides must tell each other
to connect — an offer, an answer, the routes each can be reached by — goes
out through `signal:`, over whatever the app already has, and what the other
side sent is handed to `link.signal(m)`. One side is the `initiator`.

```wf
page Room(path: "/room", title: "Room", description: "Two readers, one line.") {
    state heard = ""
    channel lobby = broadcast("room") { on message(m) { link.signal(m) } }
    peer link = rtc(signal: m => lobby.post(m), initiator: query.host == "1",
                    ice: [{ urls: "stun:stun.example.com:3478" }]) {
        on message(m) { heard = m.text }
    }

    Heading("Room").h1
    match link {
        connecting { Text("Waiting for the other side…").muted }
        open       { Button("Wave") { on click { link.send({ text: "hello" }) } } }
        closed(c)  { Text("They left.") }
        error(e)   { Alert(e.message).danger }
    }
    Text(heard)
}
```

A `broadcast` channel signals between two tabs of one browser; between two
readers the signal rides the app's own socket or `api`. `ice:` names the
servers that help two readers behind different networks find each other —
none by default, which connects readers on the same network only. A page
that leaves closes its peer, and the other side reads `closed` at once.

### The network as a value

Like `viewport` and `theme`, read where it matters:

```wf
if !network.online { Alert("You are offline — changes are queued.").warning }
Image(hero, alt: "…", loading: if network.saveData { "lazy" } else { "eager" })
```

`network.online`, `.effectiveType` (`4g`, `3g`, …), `.saveData`, `.downlink`,
and `.queued` — the writes waiting for the connection (see Offline).

### Offline

Name `offline` in the config and `wf build` writes a service worker, `sw.js`,
that stores the site for when the network is gone:

```json
{ "offline": {
    "precache": ["/", "/docs/*"],
    "fallback": "/offline",
    "cache": { "/api/*": "network-first" },
    "sync": true
} }
```

- `precache` — the routes a first visit stores, as globs (default `"/"`),
  each with its own chunk and sheet, and the shell with them. A stored route
  loads with no network; a navigation goes to the network first when there is
  one, so a page that changed is not served stale.
- `fallback` — a page's `path`, shown for a route that was not stored.
- `cache` — how a path the build did not write is fetched: `network-first`,
  `cache-first`, `stale-while-revalidate` or `network-only`. Anything not
  named passes through untouched.
- `sync` — a write (any method but `GET` and `HEAD`, through `fetch` or an
  `api`) that fails for want of a network is kept and sent, in order, when
  the connection returns — by Background Sync where the browser has it, so a
  closed page still sends it. The call resolves with nothing, so what it
  showed stays shown. A server that answers has the write, even to refuse
  it; one that is down or asks to wait keeps it for later. A `File` body is
  not kept.

The worker is versioned by a hash of everything the build wrote, so a
deploy that changes a byte is a new version. It installs and waits:

```wf
page Home(path: "/", title: "Home", description: "The front page.") {
    if update.available {
        Button("A new version is ready — reload") { on click { update.apply() } }
    }
    if network.queued > 0 {
        Text("{network.queued} change(s) will be sent when you are back online").muted
    }
    Heading("Home").h1
}
```

`update.apply()` takes the new version and reloads the page once — not on
the first install, and not twice for one update. Under `wf serve` the
worker takes itself away, so an edit is never hidden behind its own cache.

### Showing a change before the server agrees

`optimistic(holder, change)` shows it at once. If anything later in the
action throws, what was shown is taken back — the action rolls itself back:

```wf
action rename(id: String, name: String) {
    optimistic(Todos.items, items => items.map(i => if i.id == id { { ...i, title: name } } else { i }))
    await Backend.updateUser(id: id, body: { name: name })
    Backend.users.invalidate()
}
```

### Somebody else's code

The compiler cannot read the other side, so an `external` **is** the
contract: every call site is checked against it.

```wf
external Chart from "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/+esm" {
    integrity: "sha384-…"                      // emitted as a modulepreload hash
    fn Chart(canvas: Any, config: Map) -> ChartHandle
    type ChartHandle { update(data: Map)  destroy()  width: Number }
}

external element Stripe("stripe-pricing-table") {
    prop publishableKey: String                // the attribute `publishable-key`
    event ready()
}
```

A module goes in `externals.js` — its own file, linked with `<script
type="module">`, because a module has its own scope and a split build's
page chunks are classic scripts — and its origin joins `script-src`.

`Host` is a node with a lifetime: the `ref` + `effect` + `cleanup` written
by hand, with the cleanup impossible to forget.

```wf
Host(tag: "canvas",                            // div · span · canvas · svg · section · figure · pre · p · ul · table
     mount: (node) => Chart.Chart(node, config),
     update: (chart) => chart.update(rows),    // again whenever the state it reads changes
     cleanup: (chart) => chart.destroy())      // when its page, branch or item leaves
```

An `external element` is placed like a component, its props the attributes
a framework would write and its events DOM events the page hears:

```wf
Stripe(publishableKey: key) { on ready { loaded = true } }
```

### Publishing the project as custom elements

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

`PriceTag` becomes `<price-tag>`; a one-word component is prefixed so its
tag has the hyphen a custom element needs (`<wf-rating>`). The build writes
`elements.js`, `styles.css` and a page listing the tags. An attribute is a
prop — a `Number` read as a number, a `Map` or list as JSON, a `Bool` true
when present and false when `="false"` — read again whenever it changes; an
event the component declares is a bubbling `CustomEvent`; what the
component made goes when the element leaves the document. React, Vue,
Svelte, Rails, WordPress and a plain page can all place a tag, which is why
this is one output rather than an adapter per framework.

### Navigation

```wf
navigate("/path")                    // Programmatic
Link("About", to: "/about")          // Declarative
```

### The indented layout: `.wfx`

The same grammar, with blocks written by indentation instead of braces. A
file named `.wfx` is read that way: a line whose next line is indented
deeper opens a block, and a dedent closes every block it leaves. Nothing
else changes — the parser, the checks, the output and the editor support
are the same, and a project may mix `.wf` and `.wfx` files.

```wfx
type Todo { id: String, title: String }

store Todos
    state items: [Todo] = []
    action add(title: String)
        items = items.concat([Todo(id: uuid(), title: title)])

component TodoRow(_ label: String, todo: Todo)
    Row
        Text(label)

page Home(path: "/", title: "Todos")
    use Todos
    state draft = ""
    Row(align: .center, gap: .sm)
        style
            padding: 6px 0
            &:hover
                background: $surface-hover
        on click
            draft = ""
        Text("Todos").bold
    if Todos.items.length == 0
        Text("Nothing yet").muted
    else
        for t in Todos.items by t.id
            TodoRow(t.title, todo: t)
    Button("Add").primary { on click { Todos.add(draft) } }
```

- Inside parentheses, brackets, and braces you write yourself, layout is
  free, as it is in a `.wf` file: a multi-line argument list, a map
  literal, a one-line `{ on click { save() } }` all read the same.
- An empty block is written `{ }`.
- A comment line counts for nothing; blank lines count for nothing.
- Indentation must be consistent: a line that lines up with no enclosing
  block is an error.

`wf fmt --to wfx` rewrites a project's `.wf` files as `.wfx` (and `--to
wf` the other way); it is a change of layout and nothing else — the build
is byte-identical — and it refuses a file whose indentation does not
already follow its braces rather than change what it says. `wf migrate
--wfx` migrates a WebFluent 2 project straight into the indented layout.
`wf generate` writes `.wfx` in a project that has `.wfx` files and no
`.wf`.

`wf fmt` (no `--to`) formats a project's sources in place: four spaces a
level, a line that closes a block one level out, trailing blanks gone,
tabs made spaces, runs of blank lines folded to one, a brace one space
from what it opens (`Row{` → `Row {`, `}else` → `} else`). A value carried
over a line by a trailing comma is left as written; comments stay where
they are. The result is held to the file's tokens, so formatting never
changes what a file says; `wf fmt --check` writes nothing and fails when a
file would change; a `.wfx` file is normalised through its braced spelling.

## Built-in Components

### Layout

| Component | Usage |
|-----------|-------|
| `Container` | `Container { ... }` — centered max-width wrapper |
| `Host` | `Host(tag: "canvas", mount: (n) => …, update: (h) => …, cleanup: (h) => …)` — a node handed to somebody else's code, with a lifetime |
| `Row` | `Row(gap: md, align: center, justify: between) { ... }` — horizontal flex. `gap`: xs sm md lg xl · `align`: start center end stretch baseline · `justify`: start center end between around evenly |
| `Column` | `Column(span: 6) { ... }` — 12-column grid child |
| `Grid` | `Grid(columns: 3, gap: md) { ... }` — CSS grid; takes the same `gap`/`align`/`justify` as `Row` |
| `Stack` | `Stack(gap: md) { ... }` — vertical flex; takes the same `gap`/`align`/`justify` as `Row` |
| `Spacer` | `Spacer()` or `Spacer(sm)` `Spacer(xl)` — vertical space |
| `Divider` | `Divider()` — horizontal line |

### Navigation

| Component | Usage |
|-----------|-------|
| `Navbar` | `Navbar { Navbar.Brand { ... } Navbar.Links { ... } Navbar.Actions { ... } }` |
| `Sidebar` | `Sidebar { Sidebar.Header { ... } Sidebar.Item(to: "/", icon: "home") { ... } Sidebar.Divider }` |
| `Link` | `Link(to: "/path") { Text("Label") }` — the link whose `to` matches the current route carries `.active` and `aria-current="page"`; `active: .prefix` also matches routes beneath it |
| `Tabs` | `Tabs { Tabs.Page("Tab 1") { ... } Tabs.Page("Tab 2") { ... } }` |
| `Breadcrumb` | `Breadcrumb { Breadcrumb.Item(to: "/") { Text("Home") } Breadcrumb.Item { Text("Current") } }` |
| `Menu` | `Menu(trigger: "Options") { Menu.Item { ... } }` |

### Data Display

| Component | Usage |
|-----------|-------|
| `Card` | `Card.elevated { Card.Header { ... } Card.Body { ... } Card.Footer { ... } }` — variants: `.flat`, `.elevated`, `.outlined` |
| `Table` | `Table(caption: "Deployments") { Table.Head { Table.Row { Table.Cell("Col") } } Table.Body { Table.Row { Table.Cell("Val") } } }` — cells inside `Table.Head` are `<th scope="col">`, as is `Table.Cell("Build").header` anywhere (a header cell a component renders); `caption` is the table's accessible name, rendered visually hidden |
| `List` | `List { List.Item { Text("Item 1") } List.Item { Text("Item 2") } }` — `List.ordered` for numbered |
| `Badge` | `Badge("Label").primary` — tones: `.primary`, `.secondary`, `.success`, `.danger`, `.warning`, `.info`; `.pill` |
| `Tag` | `Tag("JavaScript")` |
| `Avatar` | `Avatar(src: "/photo.jpg", alt: "User")` or `Avatar(initials: "MO").primary` — flags: `.sm`, `.lg`, `.primary` |
| `Tooltip` | `Tooltip("Click to save") { Button("Save").primary }` — wraps children, shows text on hover |

### Form & Input

| Component | Usage |
|-----------|-------|
| `Input` | `Input(bind: var, placeholder: "...", label: "Name", hint: "As on your passport", error: nameError).text` — with `label:`, `hint:` or `error:` the control is wrapped in a field: the label is a `<label for>`, the hint and the error are linked by `aria-describedby`, the error (a string, empty when there is none) is announced as it appears and sets `aria-invalid` |
| `Select` | `Select(bind: var, label: "Choose", hint: "...", error: roleError) { Select.Option("Label 1", value: "val1") }` — field wrapping as `Input`; an option's positional is its visible label, `value:` what is bound (the label when there is no `value:`) |
| `Checkbox` | `Checkbox(bind: var, label: "Agree")` |
| `Radio` | `Radio(bind: var, value: "opt1", label: "Option 1")` |
| `Switch` | `Switch(bind: var, label: "Enable")` |
| `Slider` | `Slider(bind: volume, min: 0, max: 100, step: 1, label: "Volume")` — range input with reactive value. `aria-*` arguments reach the input; with `aria-valuetext:` the raw number is not shown beside the track (you are showing it) |
| `DatePicker` | `DatePicker(bind: selectedDate, label: "Start Date", min: "2026-01-01")` — date input |
| `FileUpload` | `FileUpload(accept: "image/*", label: "Upload Photo") { on change(e) { handleFile(e) } }` — flags: `.multiple` |
| `Textarea` | `Textarea(bind: note, label: "Notes", rows: 4, maxLength: 200)` — several lines; with `maxLength` it counts what is left |
| `Form` | `Form(bind: form, show: .onBlur) { on submit { save() } ... }` — `show`: when a `validate` message appears (`.onBlur`, `.onSubmit`, `.live`) |

Input types (a flag): `.text`, `.email`, `.password`, `.number`, `.search`, `.tel`, `.url`, `.date`, `.time`, `.datetime`, `.color`

### Feedback

| Component | Usage |
|-----------|-------|
| `Alert` | `Alert("Message").success` — tones: `.success`, `.danger`, `.warning`, `.info` |
| `Toast` | `Toast("Saved!").success` — temporary notification |
| `Modal` | `Modal(visible: showModal, title: "Title") { ... Modal.Footer { ... } }` |
| `Dialog` | `Dialog(visible: show, title: "Confirm") { ... }` |
| `Spinner` | `Spinner` or `Spinner.lg` |
| `Progress` | `Progress(value: 75, max: 100)` |
| `Skeleton` | `Skeleton(height: "20px", width: "200px")` or `Skeleton(size: "48px").circle` — flags: `.circle` |

### Actions

| Component | Usage |
|-----------|-------|
| `Button` | `Button("Label").primary.lg` — click handler: `Button("Save") { on click { doSave() } }` |
| `IconButton` | `IconButton(icon: "close", label: "Close")` or `IconButton(icon: "edit", label: "Edit").primary { on click { editItem() } }` — flags: `.sm`, `.lg`, `.primary`, `.danger` |
| `ButtonGroup` | `ButtonGroup { Button("A") Button("B") }` |
| `Dropdown` | `Dropdown(label: "Actions") { Dropdown.Item { ... } }` |

Button tones: `.primary`, `.secondary`, `.success`, `.danger`, `.warning`, `.info`
Button flags: `.sm`, `.lg`, `.full`, `.rounded`, `.pill`, `.outlined`; `type: .submit` inside a form

### Media

| Component | Usage |
|-----------|-------|
| `Image` | `Image(src: "/photo.jpg", alt: "Description")`, or `Image(hero, alt: "…", sizes: "…", placeholder: .blur)` with an `image` the program declares — decoded asynchronously; the first image on a page loads eagerly at high priority (it is usually the largest paint), the rest lazily. `loading:` overrides |
| `Video` | `Video(src: "/video.mp4", captions: "/clip.en.vtt", poster: "/frame.jpg").controls` — `muted`, `loop`, `playsinline`; without `captions` it draws an `A09` |
| `Audio` | `Audio(src: "/talk.mp3", transcript: "/talk.txt").controls` — the transcript is a link beneath the player |
| `Icon` | `Icon("home")` or `Icon("search").lg.primary` — 32 built-in SVG icons rendered inline |
| `Carousel` | `Carousel(autoplay: true, interval: 5000) { Carousel.Slide { Image(src: "...") } }` — slide track with dots and autoplay |

### Typography

| Component | Usage |
|-----------|-------|
| `Text` | `Text("Hello").bold.muted.center` |
| `Heading` | `Heading("Title").h1` — levels: `.h1` … `.h6` (`.h2` is the default) |
| `Code` | `Code("const x = 1").block` — `.block` for multi-line |
| `Blockquote` | `Blockquote { Text("Quote text") }` |
| `Unsafe.Html` | `Unsafe.Html(sanitize(body))` — markup, as markup. The one door for HTML a page did not write; every use draws a `V03` |
| `Markdown` | `Markdown(text)` — a small Markdown rendered as HTML, at build time and live: `#` headings, paragraphs, fenced code, `>` quotes, one-level `-`/`1.` lists, `---`, `` `code` ``, `**strong**`, `*em*`, `[text](url)`, `![alt](src)`; the text is escaped first, so HTML in it is shown, not run |

Text flags: `.bold`, `.italic`, `.underline`, `.uppercase`, `.lowercase`, `.left`, `.center`, `.right`, `.muted`, `.sm`, `.lg`, `.heading`, `.subtitle`, and the tones `.primary`, `.secondary`, `.danger`, `.success`, `.warning`, `.info`

### Component Details

#### Sidebar

Structural codegen with sub-components for app navigation:

```wf
Sidebar {
    Sidebar.Header { Text("My App").heading }
    Sidebar.Item(to: "/", icon: "home") { Text("Home") }
    Sidebar.Item(to: "/settings", icon: "settings") { Text("Settings") }
    Sidebar.Divider
    Sidebar.Item(to: "/logout", icon: "logout") { Text("Logout") }
}
```

Sub-components: `Sidebar.Header`, `Sidebar.Item`, `Sidebar.Divider`

A `Sidebar.Item` whose `to` matches the current route is marked `.active` +
`aria-current="page"`, like a `Link`; `active: .prefix` matches sub-routes.

#### Breadcrumb

Navigation breadcrumb with proper separator rendering:

```wf
Breadcrumb {
    Breadcrumb.Item(to: "/") { Text("Home") }
    Breadcrumb.Item(to: "/docs") { Text("Docs") }
    Breadcrumb.Item { Text("Components") }
}
```

The last item (without `to:`) renders as the current page (no link).

#### Tooltip

Wraps children, shows text on hover:

```wf
Tooltip("Click to save") {
    Button("Save").primary
}
```

#### Avatar

Displays an image or initials:

```wf
Avatar(src: "/photo.jpg", alt: "User")
Avatar(initials: "MO").primary
Avatar(initials: "J").lg
```

Flags: `.sm`, `.lg`, `.primary`

#### Skeleton

Loading placeholder:

```wf
Skeleton(height: "20px", width: "200px")
Skeleton(size: "48px").circle
```

Flags: `.circle`

#### Carousel

Slide track with dots and autoplay:

```wf
Carousel(autoplay: true, interval: 5000) {
    Carousel.Slide { Image(src: "/1.jpg", alt: "Slide 1") }
    Carousel.Slide { Image(src: "/2.jpg", alt: "Slide 2") }
}
```

#### IconButton

Icon-only button with aria-label:

```wf
IconButton(icon: "close", label: "Close")
IconButton(icon: "edit", label: "Edit").primary { on click { editItem() } }
IconButton(icon: "menu", label: "Actions", aria-haspopup: "menu", aria-expanded: open) { on click { toggleMenu() } }
```

Flags: `.sm`, `.lg`, `.primary`, `.danger`. The `label` is the accessible name (`aria-label` and `title`), never visible text; every other named argument (`type`, `disabled`, `aria-*`, `data-*`) is an attribute as on `Button`.

#### Slider

Range input with reactive value:

```wf
Slider(bind: volume, min: 0, max: 100, step: 1, label: "Volume")
```

#### DatePicker

Date input:

```wf
DatePicker(bind: selectedDate, label: "Start Date", min: "2026-01-01")
```

#### FileUpload

Styled file input:

```wf
FileUpload(accept: "image/*", label: "Upload Photo") {
    on change(event) { handleFile(event) }
}
FileUpload(accept: ".pdf,.doc", label: "Documents").multiple
```

### Icon System

WebFluent includes 32 built-in SVG icons, rendered inline. Available icons:

`home`, `menu`, `search`, `close`, `user`, `settings`, `check`, `plus`, `minus`, `edit`, `trash`, `star`, `heart`, `mail`, `bell`, `download`, `upload`, `eye`, `link`, `calendar`, `filter`, `info`, `warning`, `arrow-left`, `arrow-right`, `chevron-down`, `chevron-right`, `chevron-left`, `logout`, `copy`, `sun`, `moon`

Usage:

```wf
Icon("home")
Icon("search").lg.primary
IconButton(icon: "close", label: "Close")
Sidebar.Item(to: "/", icon: "home") { Text("Home") }
```

## Flags Reference

A flag is written tight after the element, its arguments or another flag:
`Button("Save").primary.lg`. It sets a `Bool` prop (`.required`,
`.outlined`, `.pill`) or picks the case of an enum prop (`.lg` is `size:
.lg`, `.primary` is `tone: .primary`); a case that more than one prop has
is named — `tone: .info`. Each component's flags are listed in its entry
above; the language server offers them after `.`, and a flag the
component does not take is an error.

**Sizes**: `.sm`, `.md`, `.lg` (`Spacer` and `gap:` also take `.xs`, `.xl`)
**Tones**: `.primary`, `.secondary`, `.success`, `.danger`, `.warning`, `.info`
**Shape**: `.rounded`, `.pill`
**Surface**: `.flat`, `.elevated`, `.outlined`
**Width**: `.full`, `.fluid`
**Text**: `.bold`, `.italic`, `.underline`, `.uppercase`, `.lowercase`, `.left`, `.center`, `.right`, `.muted`, `.heading`, `.subtitle`
**Heading level**: `.h1` … `.h6`
**Input type**: `.text`, `.email`, `.password`, `.number`, `.search`, `.tel`, `.url`, `.date`, `.time`, `.datetime`, `.color`
**Animation** (on every element): `.fadeIn`, `.fadeOut`, `.slideUp`, `.slideDown`, `.slideLeft`, `.slideRight`, `.scaleIn`, `.scaleOut`, `.bounce`, `.shake`, `.pulse`, `.spin`, `.expand`, `.collapse`
**Speed** (on every element): `.fast` (150ms), `.slow` (500ms)

## Animation

Every element takes the universal motion props: an enter animation as a
flag or `animate:`, `exit:`, `delay:`, `duration:`, `speed:` (`.fast`,
`.normal`, `.slow` — the flags `.fast` and `.slow` are this prop),
`stagger:` inside a list, `easing:`, `on:` (when the animation plays —
`.mount` or `.enterView`), `shared:` (a name it keeps across a route
change) and `count:` (a number that counts to its new value). Beside
them, every element also takes `class:` and `ref:`.

An element that stands still plays its enter animation as a stylesheet
rule, so a static page animates before its script has loaded. Everything
the runtime plays — a branch arriving or leaving, a list reordering, a
route changing, a replay — goes through the Web Animations API, where an
animation that is interrupted gives way to the one that replaced it.

```wf
// Mount animation (plays once on appear)
Card.elevated.fadeIn { Text("…") }
Heading("Title").h1.slideUp.slow

// Enter and exit: on the root of an if, for or show, the branch plays
// them; anywhere else the element plays its own exit when it leaves
if showPanel {
    Card(exit: .fadeOut).scaleIn { Text("Animated panel") }
}

// List with stagger; a keyed list moves items instead of rebuilding them
for item in items by item.id {
    Text(item.name, exit: .fadeOut, stagger: "50ms").slideUp
}

// Replay on hover (via event)
Card.outlined.fadeIn {
    on mouseenter(event) { replayAnimation(event.currentTarget, "fadeIn") }
}

// Plays when the reader has scrolled to it, and once
Section(animate: .fadeIn, on: .enterView) { Text("Further down") }

// A box that opens to the height of its content, which CSS cannot do
show details { Stack(animate: .expand) { Text("As much text as there is") } }

// A number that arrives rather than jumps, formatted the whole way
Text(format(revenue, .currency), count: "600ms")

// The same element carried from one page to the next
Image(post.cover, alt: post.title, shared: "cover-{post.id}")

// Orchestration: `after:` is measured from the step before it, so these
// start at 0ms, 120ms and 240ms. It writes the clock onto the elements
// as `delay:` and leaves nothing of itself behind.
sequence {
    step { Heading("Welcome").h1.fadeIn }
    step(after: "120ms") { Text("What we do.").slideUp }
    step(after: "120ms") { Button("Start").primary.slideUp }
}

// A handle, for a page that drives one itself: it plays when it is made
Card(ref: panel) { on mouseenter { animate(panel, "pulse", "400ms") } }

// Transition block (declarative, per-property; tokens allowed)
Button("Hover") {
    transition {
        background: 200ms ease
        transform: $d-fast spring
    }
}

// CSS transition property (in style block)
Button("Hover") {
    style {
        transition: all 200ms ease
    }
}
```

A route change animates too: `Router(transition: .fade | .slide, duration:
"200ms")` in `app`. Where the browser has the View Transitions API the
change plays through it — the browser's own crossfade, or the slide the
sheet defines for `[data-wf-transition="slide"]`, which a project's own
`::view-transition-*` rules may restyle — and the class-based animation is
the fallback. A reader who asked for less motion
(`prefers-reduced-motion`) gets none: nothing started, no wait, a box that
expands simply open and a number that counts simply there.

`easing:` takes a named easing — `.standard`, `.spring`, `.ease`,
`.easeIn`, `.easeOut`, `.easeInOut`, `.linear`, `.bouncy`, `.smooth` — or
any CSS timing function as a string. Each name is a design token
(`$ease-standard`, `$ease-spring`), so a theme retunes every animation at
once; a spring is sampled into the `linear()` easing the platform runs, so
it overshoots and settles as a spring does. `motion` in the config sets
what an animation does when the element says nothing:

```json
{ "motion": { "duration": "180ms", "easing": "$ease-standard" } }
```

## Styling

### Style Blocks

```wf
Card {
    style {
        background: #f0f0f0
        border-radius: 1rem
        padding: 2rem
        box-shadow: 0 2px 8px rgba(0,0,0,0.1)
        transition: all 200ms ease
    }
}
```

Style properties use CSS names (hyphenated). A value is raw CSS to the end
of its line or a `;` — no quotes — so anything CSS accepts is written as
CSS: `rgba(0,0,0,0.1)`, `0 4px 12px`, `"Inter", sans-serif`. Two more
things may appear in a value: `$token`, a design token (`$surface`, or the
short name a property's group resolves — `padding: $xl` is `$spacing-xl`),
and `{expr}`, a splice that reads state. A value that reads state follows
it: `style { width: {pct}% }` or `style { background: {tone} }` repaints
whenever `pct` or `tone` changes; a value without a splice compiles to a
stylesheet rule the element carries by class.

### Style support in PDF and Slides

The PDF and Slides backends honor the same set of style properties on `Slide` (slides only) and on layout containers (`Container`, `Column`, `Stack`, `Grid`, `Card`, `Section`). Anything else emits a `warning[pdf]:` or `warning[slides]: unsupported style property '<name>' on <Component>` (deduped per build).

| Property | Values | Notes |
|----------|--------|-------|
| `background` / `background-color` | `"#hex"`, `"linear-gradient(...)"` | Paint color or PDF axial-shading gradient |
| `padding`, `padding-{top,right,bottom,left}` | `Npt`, `Npx`, `N` | Insets the container's child rendering |
| `border` | `"Npt #hex"` (CSS shorthand, simplified) | Width + color |
| `border-color` | `"#hex"` | |
| `border-width` | `Npt` | |
| `border-radius` | `Npt` | Rounds the bg + border |
| `box-shadow` | `"X Y #hex"` | Offset rect; blur is ignored |
| `width`, `height` | `Npt`, `N%` | Fixed-size or percent of parent |
| `color`, `font-family`, `font-size`, `text-align` | as CSS | On `Text`/`Heading` |

Linear gradient syntax: `linear-gradient(to bottom, #color1, #color2)`, `linear-gradient(45deg, #c1, #c2)`, or named directions `to top|right|bottom|left|top-right|...`. Two color stops only.

The `Slide` element accepts `style { background }` (full-bleed page background). Setting `background` on a `Container` paints the rect under that container's measured box (or its explicit `width`/`height`).

Painting order inside a styled container: shadow → background → border → children. Children paint on top of decoration via content-stream splicing.

### Responsive values

Any layout prop takes one value per breakpoint. It compiles to a class and
a media query — no JavaScript, no resize listener, and the right layout in
the very first paint:

```wf
Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg }) { … }
Row(direction: { base: .column, md: .row }) { … }
Column(span: { base: 12, md: 6 }) { … }
```

The steps are `base`, `sm`, `md`, `lg`, `xl`, and each is a **design
token** — `screen-sm` … `screen-xl` — so a theme moves them and every
media query, and `viewport.md`, move with it. A step a value omits keeps
whatever the step below it said.

The same names are sugar inside a `style { }` block: `@md { … }` is
`@media (min-width: $screen-md)`.

```wf
Heading("Title").h1 {
    style {
        font-size: 24px
        @md { font-size: 34px }
    }
}
```

**A layout stays as it is written.** The engine used to reflow every `Row`
into a column and every `Grid` into one column below 768px, with
`!important` — which took twelve overrides and five `!important`s to undo
on one site. It does not any more. A layout that should reflow says so:

```wf
Row.stacks { … }                              // a column on a narrow screen
Grid(columns: { base: 1, md: 3 }) { … }       // or say what it does at each width
```

`wf migrate` adds `.stacks` to every `Row`, `Grid` and `Column` of a
WebFluent 2 project, so the old behaviour carries over, written down.

`viewport` stays for what is genuinely behavioural — a drawer instead of a
sidebar — and follows `matchMedia`, which fires once when the answer
changes, rather than on every pixel of a drag.

### Responsive Styles with @media

Style blocks support `@media` queries for responsive design:

```wf
Sidebar {
    style {
        width: 260px
        position: fixed

        @media (max-width: 768px) {
            display: none
        }
    }
}

Heading("Title").h1 {
    style {
        font-size: 3rem

        @media (max-width: 768px) {
            font-size: 1.5rem
        }
    }
}
```

A `@media` block is compiled into `styles.css` under a class named by its
content, and the element carries the class — so a hundred identical cards
share one rule, and the static paint has it before JavaScript runs. Values
inside it take `$tokens` like any other style value.

### Nested rules

What an inline style cannot say — how an element looks while hovered,
focused, pressed or disabled — is a nested rule in the same `style { }`,
in CSS nesting syntax: `&` is the element.

```wf
Button("Publish changes") {
    style {
        background: $brand
        color: $on-brand
        &:hover { background: $brand-hover }
        &:active { transform: translateY(1px) }
        &:focus-visible { outline: 2px solid $focus-ring; outline-offset: 2px }
        &:disabled { opacity: 0.45 }
    }
}
Input(placeholder: "shop.example.com").text {
    style { &::placeholder { color: $text-tertiary } }
}
Link("Home", to: "/") {
    style { &[aria-current="page"] { font-weight: 600 } }
}
```

Any selector CSS nesting allows: `&:hover`, `&:focus-visible` (the keyboard
focus ring, not a ring on every click), `&:active`, `&:disabled`,
`&::placeholder`, `&:focus-within`, and the ARIA states a control keeps in
its attributes — `&[aria-current="page"]`, which the router sets on a
`Link`; `&[aria-pressed="true"]`, `&[aria-selected="true"]`,
`&[aria-checked="true"]`, `&[aria-expanded="true"]`,
`&[aria-invalid="true"]` — so the rule keys off the attribute assistive
technology reads. These are stylesheet rules, so their values must be
known at build time: literals and tokens, not state. They override the
element's base declarations while the state or media condition holds (they
are emitted `!important`, since the base is inline); when a state rule and a
media rule set the same property, the media rule wins.

A rule that has to vary per element reads a custom property the element sets
— a custom property can follow state, the rule cannot:

```wf
Button(label) {
    style {
        --hover-bg: {hoverColor}  // state, prop or derived
        &:hover { background: var(--hover-bg) }
    }
}
```

A custom property of the element's own is read with `var(--name)`; `$name`
is reserved for design tokens.

### Stylesheets

What no `style { }` block can say — a selector over several elements, a
`@keyframes`, a `@font-face`, a class a dozen elements share by name, a rule
on `html` or `::selection` — goes in a `.css` file, and any `.css` file under
`src/` is part of the build:

```
src/
├── styles.css            # global rules
└── components/
    ├── Card.wf
    └── card.css          # beside the component it styles
```

```css
/* src/components/card.css */
.feature {
  container-type: inline-size;
}
.feature--first::before {
  content: counter(feature, upper-roman);
  color: var(--text-secondary);
}
```

```wf
Card(class: "feature feature--first") { ... }
Card(class: tone) { ... }        // a state: the classes follow it
```

The files are read in path order and bundled into `styles.css` after the
engine's rules (so on equal specificity yours win) and before the rules that
`style { }` blocks compile to (which carry tripled specificity, as the
inline styles they replace beat any sheet). They are minified with the rest.
`var(--token)` reaches every design token the theme declares. There is no
scoping: a class is global, and named on purpose.

`meta.stylesheets` in the config still links a sheet that is *not* built —
a file in `public/` or a URL — ahead of `styles.css`.

### Themes

A theme is written in WebFluent, in your own `src/`:

```wf
theme Brand {
    color-primary: #3B82F6
    color-secondary: #8B5CF6
    font-family: Inter, sans-serif
    radius-md: 0.5rem
}
```

Every token you do not name keeps its baseline value, so a theme is only as
large as the difference you want. A theme may also declare tokens of its own
(`surface-raised: #131519`, `viz-1: #ff6a2b`); every token becomes a custom
property on `:root`, usable from any style block as `$surface-raised` and
from a stylesheet as `var(--surface-raised)`. Declare one and it is used
automatically; declare several and pick one with `"theme": { "name":
"Brand" }`.

**Dark mode.** Declare a second theme with the tokens that change and name
it in the config: `"theme": { "name": "Brand", "dark": "Night" }`. Its
tokens apply under `prefers-color-scheme: dark`, and whenever the reader
chose — `setTheme("dark")`, `setTheme("light")`, `setTheme("system")`,
kept across visits; `theme` reads the choice.

```wf
theme Night { color-background: #0B1220  color-text: #E5E7EB  color-surface: #111827 }
```

**Keyframes.** `animation Name { from { … } 50% { … } to { … } }` declares
keyframes: play them with `animate: .Name` or `exit: .Name` on any element,
or write `animation: Name 1s infinite` in a style. `@container (…) { }`
inside a style block is a container query, passed through as written.

The baseline names system fonts on purpose. A theme that names a web font
lists where to fetch it, and every page links it ahead of `styles.css` with a
`preconnect` to its origin:

```json
{ "meta": { "fonts": ["https://fonts.googleapis.com/css2?family=Manrope:wght@400..800&display=swap"] } }
```

`meta.stylesheets` links extra stylesheets the same way — a file in `public/`
by site-relative path, or a URL. It is for the few things no element-level
`style { }` can say (`html { background }`, `::selection`), not for component
styling, which belongs in `.wf` source.

For values a machine supplies (a deploy pipeline injecting a brand colour),
`theme.tokens` in `webfluent.app.json` still applies, on top of the theme:

```json
{ "theme": { "tokens": { "color-primary": "#8B5CF6" } } }
```

## Build output

```
build/
├── index.html            # the shell (SPA) or the pre-rendered page (SSG)
├── app.js                # the runtime this program needs, stores, components
├── pages/<Name>.js       # one chunk per Page, loaded when its route shows
├── pages/<Name>.css      # the style rules only that page reaches
├── styles.css            # tokens, only the component rules the project uses,
│                         # the project's own .css files, and the compiled
│                         # style { } rules more than one page shares
├── sw.js                 # with "offline" in the config: the service worker
├── *.gz                  # every text file above, gzipped, beside itself
└── …                     # public/ copied to the root, sitemap.xml, robots.txt
```

`app.js` opens with the runtime — but only the part of it the program
reaches. The runtime is a set of feature modules (`each`, `router`, `net`,
`format`, `icons`, `carousel`, …), and a build reads what it needs from the
bundle it just wrote: every `WF.<name>` the code generator emitted, closed
over each module's dependencies, plus the glyphs the program names, which
are the only ones the icon table keeps. A page that shows static text
carries 3 of the 37 modules — an `app.js` of 3.7 kB gzipped; a `for` adds
`each`, a second page adds `router`, a `resource` adds `net`.

Nothing is hand-marked, so the set cannot fall behind the compiler. A name
a build cannot see — an icon assembled at run time, a `WF.*` call in a
hand-written `<script>` — is the one case it gets wrong, and
`"build": { "runtime": "full" }` ships every module for it.

`:root` carries only the design tokens something in the output names —
a stylesheet, a script, a page, a file copied from `public/` — and the
tokens those name in turn.

`wf build --stats` prints what the build weighs: every text file with its
gzipped size and how it moved since the last build, and each runtime module
with its size and what reached it.

```
  What it weighs
    app.js                                10.6 kB  (3.6 kB gzipped)  −0.1 kB
    styles.css                            10.0 kB  (2.7 kB gzipped)

  Runtime: 3 of 37 modules, 23.4 kB of 220.3 kB (before minifying)
    core          18.6 kB  always
    pages          2.5 kB  WF.page
    hydrate        0.9 kB  WF.hydrate
    left out: motion helpers format form text when match slot each show …
```

`build.budget` names a gzipped size an output must stay under. A build over
it prints a warning and goes on — a build that fails on size is a decision
for a project's own CI, which can read the same numbers:

```json
{ "build": { "budget": { "app.js": "40 kB", "styles.css": "30 kB" } } }
```

Each build records what it weighed in `.wf-sizes.json` beside the project
(not in the output, which is published), so the next `--stats` shows the
difference.

`build.compress` (on by default) writes `<file>.gz` beside every text
output over a kilobyte — HTML, JavaScript, CSS, SVG, JSON, XML — for a host
that serves a precompressed file when it has one (nginx `gzip_static on`,
Apache `MultiViews`, most CDNs and static hosts). The compressor is the
engine's own, within a percent of zlib's default level, so the build has no
dependency for it; `wf serve` sends the same files, and compresses on the
way out what has none, so what Lighthouse measures against the dev server is
what a deployed site sends. Hosts that ignore `.gz` files lose nothing.

`build.minify` (on by default) strips comments and whitespace from the
bundle and the sheets. `build.split` (on by default) writes each page as its
own chunk: a static build's page links `app.js` and its chunk side by side,
and a single-page build fetches a chunk the first time its route is shown.
A literal in a `style { }` block is compiled into a stylesheet rule under a
class named by the block's content, so a hundred identical cards share one
rule; only a value that reads state is set on the element. With `split`, a
rule only one page can reach — in its body, or through components no other
page uses — is written to `pages/<Name>.css`, which the page links after
`styles.css` and the router loads before drawing the page; rules the `App`
body or two pages reach stay in `styles.css`.

Images are given `decoding="async"`; the first on a page is eager with
`fetchpriority="high"` and every later one `loading="lazy"`, unless the
element sets `loading:` itself.

## Runtime

`app.js` opens with the runtime, reachable as `window.WF` from the console,
a `script`, or the studio. The compiled pages call it; a hand-written script
may too. Its surface:

| Group | Functions |
|---|---|
| State | `signal(v)` → getter with `.set`/`.update`/`.subscribe`; `effect(fn)`; `computed(fn)` |
| Elements | `el(tag, attrs, …children)` — a thunk attribute or child follows state; `text(v)`; `props(given, defaults)`; `onRoot(node, event, fn)`; `classes(node, fn)` |
| Bodies | `when(parent, cond, then, else, anim)`; `each(parent, list, item, {key, index, enter, exit, …})` — keyed items keep their nodes across inserts, removals and moves; `show(parent, cond, body, anim)`; `match(parent, key, arg, arms)` |
| Motion | `animate(target, name, duration)` → a handle with `play`/`cancel`/`finished`, playing when it is made; `replay(node, name)`; `animateIn`/`animateOut(node, name, duration, delay, easing)`; `expand(node, open, duration, easing)` to the measured height; `countTo(node, from, to, duration, format)` and `counted(node, valueFn, duration, format)`; `onEnterView(node, name, …)`; `shared(node, name)`; `mark(node, attrs)` puts `data-wf-exit`/`data-wf-delay`/`data-wf-duration`/`data-wf-animate` on a component's root — what `when`, `each`, `show` and `match` play before removing an element, and what times its enter animation |
| Routing | `router(routes, container, {transition, duration})` — `fade` or `slide` plays the old page out and the new one in; `navigate(path)`; `params()`; `activeLink(a, href, prefix)`; `page(name, fn)`; `loadPage`; `loadSheet` |
| Data | `resource(url, options)` → `{state, data, error, reload}`; `request(url, options)` → the parsed body (`await fetch(…)` in an action compiles to it); `fetch(url, options)`; `store(name, define, options)` — a proxy that builds `define()` on first read; `dropRouteStores()`; `emit(props, event, …args)`; `locales(default, tables)` → `WF.i18n` |
| Stores, seen | `watchStores(fn)` → every action with the state on each side of it, and the function that stops it; `storeSnapshot(name?)`; `restoreStore(name, values)`. Nothing is recorded while nothing is watching |
| Widgets | `toast(message, tone, ms)`; `dialog(el, read, write)`; `popup`; `tabs`; `drawer`; `announce(text)`; `carousel`; `tooltip`; `menu`; `field` |
| Safety | `safeUrl(url)` → the URL where a browser may follow it, `""` where it may not; `sanitize(html)` → the allow-listed markup. Both have a build-time twin, pinned by a shared case table |
| Text | `markdown(text)` → HTML; `highlight(code, lang)` → HTML with `wf-tok-*` spans (`wf`, `json`, `bash`, `css`), the twin of `codegen::highlight` |
| Interop | `attach(node, mount, update, cleanup)` — what `Host` compiles to; `jsonAttr(raw)` — an attribute holding JSON, as the prop it stands for |
| Boot | `mount(fn, container)`; `hydrate(fn, container)`; `mainOf(root)`; `setBasePath`; `setSsgMode`; `__debug`, `__reg` (the studio's) |
| Scopes | `onCleanup(fn)` — run when what owns it leaves; `attempt(fn)` — an action that rolls itself back on a throw |
| Keys | `onKey(node, combination, fn)`; `keyIs(event, combination)` — what `on key("ctrl+k")` compiles to |
| Values | `caseOf(v)`, `payload(v)` read an enum case and its payload; `dateOf`, `timeOf`, `urlWith`, `urlQuery` are the scalar helpers behind `.date()`, `.time()`, `.with()` and `.query()` |

## Search and Sharing

Set `meta.site_url` and the compiler writes everything a search engine and a link
preview need. Without it, the tags that require an absolute URL are omitted
rather than guessed at — a relative canonical causes problems later, so none is
better than a wrong one.

```json
{
  "meta": {
    "site_url": "https://example.com",
    "site_name": "Ledger",
    "description": "Fallback for pages that set none",
    "image": "/card.png",
    "sitemap": true
  }
}
```

Every page then carries:

- a self-referencing absolute `<link rel="canonical">`
- Open Graph and Twitter card tags, built from the page's title, description and image
- `<link rel="alternate" hreflang="…">` for each locale plus `x-default`, when i18n
  declares more than one
- JSON-LD (`WebSite`, `Organization`, `WebPage`/`Article`, and a `BreadcrumbList`
  derived from the route)

A build also writes `sitemap.xml` and `robots.txt`. Dynamic routes, catch-alls and
`noindex` pages are left out of the sitemap. `priority` and `changefreq` are not
emitted: Google ignores both.

## Security

What the compiler guarantees, as properties of the output rather than
advice:

- **Text is text.** Every string an element shows goes in as a text node.
  `Unsafe.Html(markup)` is the one door for markup, named so a review
  greps for it, and every use draws a `V03`.
- **No inline handlers.** `Button("x", onclick: expr)` is a compile error
  pointing at `on click { }`.
- **No URL a browser would run.** `href:`, `src:`, `to:`, `poster:` and
  `navigate()` take `http`, `https`, `mailto`, `tel`, `sms`, `ftp` or a
  relative URL. A literal is refused where it is written; anything else
  goes through `WF.safeUrl` at the moment it is used. `javascript:`,
  `data:`, `vbscript:`, `blob:` and `file:` are refused.
- **`rel="noopener noreferrer"`** on any link with `target:`.
- **A `Secret` cannot escape** — shown, spliced, logged or persisted is a
  `T12`.

`sanitize(html)` is an allow-list of elements and attributes — no script,
no style, no frame, no form, no `on*`, and every `href`/`src` through the
scheme check. It runs at build time and in the browser from the same list,
so a pre-rendered page shows the sanitised body immediately.

```wf
Unsafe.Html(sanitize(post.body))     // markup from outside the project
```

**Sessions belong in an httpOnly cookie**, not in `persist`: any script
that reaches the page can read `localStorage`. `api B(…) { credentials:
.sameOrigin }` sends the cookie with every call and the page holds no
token at all.

**`env` splits in two.** `env.NAME` is replaced at build time and the value
is in the bundle, so only a public name may be read from a page, a
component, a store or an `api` block: one that begins `PUBLIC_`, or one
`public_env` lists. Any other name is a compile error naming the file and
the line; the values still reach `wf render`, which runs on a server.

```json
{ "env": { "PUBLIC_API": "/api/v1", "STRIPE_SECRET": "sk_…" }, "public_env": [] }
```

`wf audit` prints what a review asks for: every `Unsafe.*` and whether it
is sanitised, every other origin, everything kept on the reader's machine,
every `env` name and whether it is public, the policy this build ships, and
what the compiler depends on. `--json` for a tool; it never fails a build.

### Headers and the policy

`"build": { "csp": true }` — on for a project `wf init` creates — emits a
`Content-Security-Policy` meta tag and a `_headers` file for hosts that
read one. `frame-ancestors` is only in `_headers`, because a browser
ignores it in a meta tag. The policy is widened by exactly the origins
`meta.fonts` and `meta.stylesheets` declare, so a declared font is never
blocked by the policy that ships beside it.

**The build reads its own output back and holds it to that policy**: an
inline `<script>` or `<style>`, a `style=` attribute, an `on*` attribute,
or a script or stylesheet from an origin the policy never named stops the
build. One exception is widened rather than refused — a `style { }` value
that reads state is written on the element, so `style-src` gains
`'unsafe-inline'` and the build names the pages that caused it.

`meta.integrity` gives a declared external asset its subresource-integrity
hash (emitted with `crossorigin="anonymous"`); a build warns about one that
has none. The build never fetches the file to work the hash out.

## i18n (Internationalization)

### Translation files

```json
// src/translations/en.json
{ "nav.home": "Home", "greeting": "Hello, {name}!" }

// src/translations/ar.json
{ "nav.home": "الرئيسية", "greeting": "أهلاً، {name}!" }
```

### Config

```json
{
    "i18n": {
        "default_locale": "en",
        "locales": ["en", "ar"],
        "dir": "src/translations"
    }
}
```

### Usage

```wf
Text(t("nav.home"))                    // Translated text
Text(t("greeting", { name: user.name }))          // With interpolation
Text(t("items", { count: n }))                    // A plural form, picked by `count`
Button("EN") { on click { setLocale("en") } }     // Switch locale
Button("AR") { on click { setLocale("ar") } }     // Auto-RTL for Arabic
```

A message with plural forms is several keys: `"items.one": "{count}
item"`, `"items.other": "{count} items"` — and `items.zero`, `items.two`,
`items.few`, `items.many` where the locale's rules call for them
(`Intl.PluralRules`); `t("items", { count: n })` picks the form, falling
back to `items.other` and then `items`. The static paint picks by the
English rule (one for exactly one); the live page by the locale's.
`format` and `ago` speak the locale too.

RTL locales (automatic `dir="rtl"`): `ar`, `he`, `fa`, `ur`

## SSG (Static Site Generation)

```json
{ "build": { "ssg": true, "base_path": "/my-site" } }
```

- Pre-renders each page to its own `index.html`
- JavaScript hydrates for interactivity
- Static content visible immediately (no blank screen)
- Dynamic content (state, if/for, fetch) handled by JS after hydration

## PDF Output

```json
{
    "build": {
        "output_type": "pdf",
        "pdf": {
            "page_size": "A4",
            "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
            "default_font": "Helvetica",
            "default_font_size": 12,
            "output_filename": "report.pdf"
        }
    }
}
```

### PDF-specific components

| Component | Purpose |
|-----------|---------|
| `Document(page_size: "A4")` | Root document wrapper |
| `Section` | Groups content with spacing |
| `Paragraph` | Block of text with paragraph spacing |
| `Header` | Content repeated at top of every page |
| `Footer` | Content repeated at bottom of every page |
| `PageBreak()` | Forces a new page |

Supported in PDF: `Text`, `Heading`, `Table`, `List`, `Code`, `Blockquote`, `Divider`, `Alert`, `Badge`, `Progress`, `Card`, `Image`, `Spacer`, `Container`, `Row`, `Stack`, `Grid`

**Rejected in PDF** (compile error): `Button`, `Input`, `Select`, `Checkbox`, `Switch`, `Form`, `Modal`, `Dialog`, `Toast`, `Router`, `Navbar`, `Sidebar`, `Tabs`, `Video`, `Carousel`, and all event handlers.

Page sizes: `A4`, `A3`, `A5`, `Letter`, `Legal`
Fonts: Helvetica, Helvetica-Bold, Times-Roman, Times-Bold, Courier, Courier-Bold (all Base14)

### PDF example

```wf
page Report(path: "/", title: "Report") {
    Document(page_size: "A4") {
        Header {
            Text("Company Inc.").muted.sm.right
        }
        Footer {
            Text("Confidential").muted.sm.center
        }
        Section {
            Heading("Q1 Report").h1
            Text("Revenue grew 15% this quarter.")

            Table {
                Table.Head { Table.Row { Table.Cell("Region") Table.Cell("Revenue") } }
                Table.Body { Table.Row { Table.Cell("North America") Table.Cell("$2.4M") } }
            }

            PageBreak
            Heading("Highlights").h2
            List {
                Text("Launched 3 new products")
                Text("Expanded to 5 markets")
            }
        }
    }
}
```

## Slides Output (PDF Slide Decks)

PDF deck output where **one `Slide` = one PDF page** (no flow pagination).

```json
{
    "build": {
        "output_type": "slides",
        "slides": {
            "size": "16:9",
            "default_font": "Helvetica",
            "default_font_size": 24,
            "margin": 60,
            "show_slide_numbers": true,
            "footer_text": "My Deck",
            "background_color": "#1A1A19",
            "chrome_color": null,
            "output_filename": "deck.pdf"
        }
    }
}
```

`background_color` paints every slide full-bleed with the given color (override per-slide via `Slide { style { background } }`). `chrome_color` overrides the slide-number/footer color; if `null`, it auto-flips between dark and light grey based on the slide's background luminance.

A deck must be wrapped in a `Presentation { ... }` block inside a `Page` body. Slide elements must not appear outside `Presentation` (compile error).

### Slide kinds

| Component | Purpose |
|-----------|---------|
| `Presentation { ... }` | Deck root — children must be slide elements only |
| `Slide { ... }` | Freeform slide; top-aligned content |
| `TitleSlide("Title", subtitle: "Subtitle")` | Cover slide; title 56pt bold + subtitle 28pt grey, vertical-centered |
| `SectionSlide("Label").primary` | Full-bleed colored band, white centered label (48pt bold). Tones: `.primary`, `.success`, `.danger`, `.warning`, `.info` |
| `TwoColumn { Container { ... } Container { ... } }` | Two equal columns with a 24pt gutter — requires exactly 2 `Container` children |
| `ImageSlide(src: "...", caption: "...")` | Image slide with optional caption; `src` is required |

Body components inside a `Slide` (or inside `TwoColumn`'s columns): `Text`, `Heading` (auto-scaled ~2× for slides), `List`, `Container`, `Stack`, `Column`, `Grid`, `Section`, `Spacer`, `Divider`, `if`/`for` (static iteration only).

### Slide sizing (`slides.size`)

- `"16:9"` → 960×540pt (default)
- `"4:3"` → 720×540pt
- `"A4-landscape"` → 841.89×595.28pt
- `"WIDTHxHEIGHT"` (e.g. `"800x600"`) → explicit points

Or override with `slides.width` + `slides.height` (in points).

### Slide chrome (opt-in via config)

- `slides.show_slide_numbers: true` → `n / total` in bottom-right
- `slides.footer_text: "..."` → text in bottom-left

Both render in 11pt grey at the bottom margin.

### Overflow

Content that exceeds the bottom margin is **clipped** and a warning is printed to stderr (`warning[slides]: slide N content overflows; truncated`). The build does **not** fail on overflow.

### Rejected in slides

Same interactive components as PDF (`Button`, `Input`, `Form`, `Modal`, `Router`, `Navbar`, `Video`, etc.) plus PDF document components (`Document`, `Paragraph`, `PageBreak`, `Header`, `Footer`) — slides have their own footer chrome via config.

### Slides example

```wf
page Deck(path: "/", title: "Q1 Review") {
    Presentation {
        TitleSlide("Q1 Review", subtitle: "Company Inc. — March 2026")

        Slide {
            Heading("Highlights").h1
            List {
                Text("Launched 3 new products")
                Text("Expanded to 5 markets")
                Text("Revenue grew 15%")
            }
        }

        TwoColumn {
            Container {
                Heading("Wins").h3
                Text("New enterprise deals")
            }
            Container {
                Heading("Risks").h3
                Text("Supply chain delays")
            }
        }

        ImageSlide(src: "chart.png", caption: "Q1 revenue by region")

        SectionSlide("Q2 Plan").primary

        Slide {
            Heading("Thanks").h1
            Text("Questions?")
        }
    }
}
```

## Strings

A plain `"…"` string reads the escapes `\n \t \r \\ \" \{ \}` and `{…}`
splices. Two other forms exist for text a plain string makes you spell
twice:

```wf
// Raw: no escapes, no splices — the text is what is written. As many `#`
// as it takes for the text not to close it.
const SAMPLE = #"page Home(path: "/") { state n = 0 }"#
const TRICKY = ##"a raw string ends with "# — like that"##

// Block: over as many lines as it likes, escapes and splices on, and the
// indentation the source gave it removed. The closing `"""` says how far
// the text was indented, and that much comes off every line.
const LETTER = """
    Dear {name},
      thank you.
    """                        // "Dear Ada,\n  thank you."
```

A splice may say how to show its value — `{value:.style}` or
`{value:.style(option)}`, which is `format(value, .style, option)` written
where it is read:

```wf
Text("Total: {total:.currency}")          // Total: $1,234.50
Text("{count:.integer} items")            // 1,235 items
Text("Shipped {when:.date(long)}")        // Shipped March 5, 2026
```

A splice opens with a name, a `(` or a `[`, and closes on the same line;
anything else — `"{key: value}"` in prose, `"{"` alone — is text.

## Compiler diagnostics

`wf build` prints every diagnostic with the file and line it came from.
Errors stop the build; warnings do not.

**Errors — the program cannot mean what it says.** A parse error; a
reference to nothing — an undeclared component, a page's `layout:` that
names no component or one without a default slot, two pages or two
components with one name; a flag, case, part, event, slot or `emit` the
registry or the component's declaration does not know (`Button has no
flag or enum case `huge``, with the flags it takes); and what the type
checker finds:

| Code | What it means |
|---|---|
| `T01` | A value of the wrong type given to a prop, a state, a field, a parameter or an assignment — `` `count` of `C` is `String`, but `Number` is wanted `` |
| `T02` | A case the enum does not have |
| `T04` | A value that may be `null` read as if it were not; `if let`, `??` or a `!= null` check narrows it |
| `T05` | A field or method a record does not have |
| `T06` | A member or action a store does not have |
| `T07` | A list, record or action used as a condition, which is always true |
| `T08` | A `for` over something that is not a list |
| `T09` | An `emit` whose arguments do not match the event's |
| `T10` | A call with the wrong number or kind of arguments |
| `T11` | A `match` on something that is neither a resource nor an enum, or with arms of the wrong kind |
| `T12` | A `Secret` where it would escape — shown, spliced into text, logged, or kept with `persist` |
| `T13` | A name, or a function called, that nothing declares — a ReferenceError in the browser. Not applied to a template rendered with data |

Two more are errors of the same kind, reported where they are written: an
`on*` attribute (script in an attribute), and a `href`/`src`/`to` literal
naming a scheme a browser runs. A non-public `env` name read from a page
stops the build too.

An endpoint a service does not have is `T06`, and an argument it does not
take is `T10`.

A value whose type the checker cannot work out is `Any`, which agrees with
everything; a *name* nothing declares is `T13`.

**Warnings.**

| Rule | What it means |
|---|---|
| `A01` | An `Image` with no `alt` (a decorative one takes `alt: ""`) |
| `A02` | An `IconButton` with no accessible name — `label:` is its name, never visible text |
| `A03` | An `Input` with neither `label:` nor `placeholder:` |
| `A04` | A `Checkbox`, `Radio`, `Switch`, `Slider` or `Textarea` with no `label:` |
| `A05` | A `Button` with no text |
| `A06` | A `Link` with no text |
| `A07` | A `Heading` with no text |
| `A08` | A `Modal` or `Dialog` with no `title:`, which is its accessible name |
| `A09` | A `Video` with no `controls`, or a `Video`/`Audio` with no `captions:`/`transcript:` |
| `A10` | A `Table` with no header row (`Table.Head`) |
| `A11` | A heading level skipped — `h2` straight to `h4` |
| `A12` | A page with no `h1` |
| `A13` | A theme's colour pairing falls below the WCAG AA contrast ratio |
| `A14` | A `role:` that requires particular children (`tablist` → `tab`, `list` → `listitem`, `menu` → `menuitem`, …) holds a control that is not one |
| `A15` | A control's `aria-label` does not contain its visible text — its label, the literals in its block, and what a component in the block renders from its literal props — so what a voice-control user says does not match what they see |
| `S01` | A page has no title |
| `S02` | A page has no description, so its search snippet is written for it |
| `S03` | A description longer than ~160 characters, which a search result truncates |
| `S04` | Two pages claim the same route |
| `P01` | A `persist` at `version: n` with nothing to bring an older version forward, so what a returning reader had is discarded |
| `P02` | A `migrate` step above the declared `version:`, which never runs |
| `P03` | A `persist` inside a `store(scope: .route)`: the route change drops the store, and the next read builds it again from storage, so the value comes straight back |
| `V01` | A bare word in an argument that nothing in scope declares — a name misspelled, or a flag written without its dot (`did you mean `.center`?`) |
| `V02` | A flag whose class no stylesheet — the engine's or one of the project's `.css` files — defines; the registry keeps this from happening for the built-ins |
| `V03` | An `Unsafe.Html`, and whether what it puts in went through `sanitize` |
| `U01`–`U02` | A `state` or `derived` value nothing in its page or component reads (an assignment alone does not read it) |
| `U03` | A component nothing places, names as a layout, or reaches as a part |
| `U04` | A store member — state, derived, action — nothing reads, inside the store or as `Store.member` |
| `U05` | An action nothing calls; a name that starts with `_` is understood to be unused on purpose |
| — | A named argument a built-in does not declare, written to the element as an attribute; a prop a component does not declare, passed anyway |

The heading-outline rules (`A11`, `A12`) do not apply to `Presentation` or
`Document` output, where an `h1` per slide or per section is correct.

## Migrating

`wf migrate [path] [--check] [--stdout]` does two things.

**WebFluent 2 → 3** rewrites every `.wf` under `src/` in place, with a
note for anything that needed a decision. It is a change of spelling and
nothing else: the migrated project builds to what it built before. A
WebFluent 2 file given to `wf build` is refused with a pointer to it.

**WebFluent 3 → 4** is not a change of spelling but of what the compiler
allows, so it runs over the project rather than the files. It adds every
`env` name the pages already read to `public_env` — preserving what the
project did, and printing the list, because "public" means anyone who
opens the site may read the value. It then names, with the file and the
line, everything 4 refuses that 3 allowed: an `on*` attribute, a
`javascript:` or `data:` URL. Those have no automatic rewrite. Finally it
states the changes that need no edit and are worth knowing: a store is
built on first read rather than at boot, a `persist` value follows the
site's other tabs, and `WF.store`/`WF.host` were renamed.

## Configuration Reference (webfluent.app.json)

```json
{
    "name": "My App",
    "version": "1.0.0",
    "author": "Name",
    "theme": {
        "name": "Brand",
        "tokens": {},
        "builtin": "full"
    },
    "build": {
        "output": "./build",
        "minify": true,
        "ssg": false,
        "base_path": "",
        "csp": false,
        "split": true,
        "compress": true,
        "sourcemap": false,
        "runtime": "auto",
        "budget": {},
        "media": { "formats": ["webp"], "widths": [480, 960, 1440, 1920], "quality": 78, "pipeline": true },
        "output_type": "spa",
        "elements": [],
        "pdf": {
            "page_size": "A4",
            "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
            "default_font": "Helvetica",
            "default_font_size": 12,
            "output_filename": null
        },
        "slides": {
            "size": "16:9",
            "width": null,
            "height": null,
            "default_font": "Helvetica",
            "default_font_size": 24,
            "margin": 60,
            "show_slide_numbers": false,
            "footer_text": null,
            "background_color": null,
            "chrome_color": null,
            "output_filename": null
        }
    },
    "dev": { "port": 3000, "hot_reload": true },
    "motion": { "duration": "180ms", "easing": "$ease-standard" },
    "meta": {
        "title": "",
        "description": "",
        "favicon": "",
        "touch_icon": "",
        "lang": "en",
        "fonts": [],
        "stylesheets": [],
        "integrity": {}
    },
    "env": {},
    "public_env": [],
    "offline": {
        "precache": ["/"],
        "fallback": null,
        "cache": {},
        "sync": false
    },
    "i18n": {
        "default_locale": "en",
        "locales": ["en"],
        "dir": "src/translations"
    }
}
```

## Template Engine (Server-Side Rendering)

WebFluent can be used as a **template engine** from Rust or Node.js to render `.wf` templates to HTML or PDF with JSON data.

### CLI

```bash
# Render to HTML
wf render template.wf --data data.json --format html -o output.html

# Render to HTML fragment (no <html> wrapper)
wf render template.wf --data data.json --format html-fragment

# Render to PDF
wf render template.wf --data data.json --format pdf -o report.pdf

# Pipe JSON from stdin
echo '{"name":"Monzer"}' | wf render template.wf --format html

# With theme
wf render template.wf --data data.json --format html --theme Brand

# With a design token over the theme's (repeatable)
wf render template.wf --data data.json --token color-primary=#8B5CF6
```

### Rust API

```rust
use webfluent::Template;
use serde_json::json;

let tpl = Template::from_str("Container { Heading(\"Hello, {name}!\").h1 }")?;
// or: Template::from_file("templates/invoice.wf")?;

let html   = tpl.render_html(&json!({"name": "World"}))?;           // Full HTML doc
let frag   = tpl.render_html_fragment(&json!({"name": "World"}))?;  // Fragment only
let pdf    = tpl.render_pdf(&json!({"name": "World"}))?;            // Vec<u8>
let slides = tpl.render_slides(&json!({"name": "World"}))?;         // Vec<u8> (PDF deck)

// Selecting one of several themes declared in the template
let html = tpl.with_theme("Brand")
    .with_tokens(&[("color-primary", "#8B5CF6")])
    .render_html(&data)?;
```

### Node.js API

```javascript
// npm install webfluent — a wrapper around `wf render`; needs `wf` installed
const { Template } = require('webfluent');

const tpl = Template.fromString('Container { Heading("Hello, {name}!").h1 }');
// or: Template.fromFile('templates/invoice.wf');

const html = tpl.renderHtml({ name: "World" });           // Full HTML string
const frag = tpl.renderHtmlFragment({ name: "World" });   // Fragment string
const pdf  = tpl.renderPdf({ name: "World" });             // Buffer

// Selecting one of several themes declared in the template
const html = tpl.withTheme('Brand')
    .withTokens({ 'color-primary': '#8B5CF6' })
    .renderHtml(data);
```

### Template Data Context

Data is a JSON object. Top-level keys become template variables:

```wf
// template.wf
page Invoice(path: "/", title: "Invoice") {
    Container {
        Heading("Invoice #{number}").h1
        Text("Customer: {customer.name}")

        for item in items {
            Card { Text(item.name).bold Text("${item.price}") }
        }

        if paid { Badge("PAID").success }
        else    { Badge("UNPAID").danger }
    }
}
```

```json
{
    "number": "INV-001",
    "customer": { "name": "Acme Corp" },
    "items": [{ "name": "Widget", "price": 9.99 }],
    "paid": true
}
```

**Supported in templates**: all layout, typography, data display components, `for` loops, `if/else`, string interpolation, style blocks, flags, themes.

**Not supported**: `state`, `derived`, `effect`, handlers (`on click`), navigation, stores, animations, `resource`.

## Key Rules

1. **Every page needs a path**: `page Name(path: "/route") { ... }`
2. **State is reactive**: any UI referencing a state variable auto-updates
3. **Flags are written with a dot, after the parentheses**: `Button("Label").primary.lg` — order doesn't matter. A flag is a Bool prop or an enum case the component declares; a word nothing declares is an error. A prop, state or derived name is an expression, so a component with a `text` prop writes `Text(text)` and means the prop
4. **One positional argument, then named ones**: `Input(bind: myVar, placeholder: "...").text` — an enum-valued prop takes a case: `Row(gap: .md, align: .center)`
5. **Braces for children/body**: `Card { Card.Body { Text("content") } }`
6. **Parts use dot, after the owner**: `Card.Header`, `Card.Body`, `Card.Footer`, `Navbar.Brand`, `Table.Row`, `Select.Option`
7. **Event handlers**: `on click { ... }` inside an element's block, `on click(e) { ... }` when the body reads the event. A `Button`'s block holds what it shows; `on click` holds what it does
8. **String interpolation is reactive**: `Text("Count: {count}")` updates when `count` changes
9. **Imports via `use`**: `use StoreName` to access shared stores
10. **No semicolons needed**: statements are newline-separated; `;` separates two on one line
11. **`return` in actions**: `return expr` returns a value from store actions
12. **Style values are CSS**: `padding: 1rem 2rem`, `background: $surface`, `width: {pct}%` — no quotes; `$name` is a design token, `{expr}` a live value
13. **`@media` and nested rules inside style blocks**: `@media (max-width: 768px) { display: none }` and `&:hover { … }` compile to stylesheet rules scoped to the element
14. **Router nests anywhere**: `Router` can be inside `Row`, `Container`, `Stack`, or any layout wrapper at any depth; pages own their routes
15. **Browser globals are not prefixed**: `localStorage`, `window`, `console`, `JSON`, `Math`, `Date`, `setTimeout`, `fetch`, `Promise`, etc. compile as-is
15. **Both `!=` and `!==`**: both inequality operators are supported (both compile to `!==` in JS)
16. **Quoted map keys**: `{ "Content-Type": "application/json" }` — use for HTTP headers and hyphenated keys
17. **Reserved words as map keys**: `{ action: "approve", token: tok }` — all keywords work as map keys
18. **`public/` copies to build root**: files in `public/` land at the root of the output directory, not nested
19. **Slides need a Presentation wrapper**: `Page X { Presentation { Slide { ... } } }` — slide elements outside `Presentation` are a compile error
20. **One Slide = one PDF page**: slides do not flow across pages; overflow is clipped with a stderr warning
