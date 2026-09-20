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
wf build [-d DIR]                         # Compile
wf serve [-d DIR]                         # Dev server (localhost:3000)
wf generate page|component|store <name>   # Scaffold a file, in the project's layout
wf fmt --to wfx|wf [path] [--stdout]      # Switch a project between .wf and .wfx
wf migrate [path] [--check] [--wfx]       # WebFluent 2 → 3 (and to .wfx)
wf registry [--json]                      # Every built-in: props, cases, flags, events, slots, parts
wf types [path] [--json]                  # What a project declares: enums, types, components, stores, pages
```

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
- `title` — Browser tab title, and the heading of a search result
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
    action add(title: String) { items = items.concat([Todo(id: uid(), title: title)]) }
}
```

A record is built with named fields, `Todo(id: "1", title: "x")`; a case is
written `.calm`. The type checker reads every declared type and infers the
rest (literals, records, lists, the built-in methods of strings, numbers and
lists, store members, lambdas). Anything unresolved is `Any`, which agrees
with everything, so a program that declares no types checks as it always
did; every annotation narrows what the checker can say. See
[Compiler diagnostics](#compiler-diagnostics) for what it reports.

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
    Navbar(brand: "Ledger") { Navbar.Links { Link("Home", to: "/") } }
    Router
    Footer
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
`keydown`, `keyup`, `keypress`, `mouseenter`, `mouseleave`. The handler's
parameter names the DOM event; without one, `event` is in scope. An
element's block is ordered `style` → `transition` → `on …` → slot fills →
children.

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
it async; `return` leaves it with a value or without.

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
data = JSON.parse(responseText)
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
`a` is null; `if c { a } else { b }` and `match t { .calm { 1 } else { 2 } }`
are values; `$token` is a design token; `.case` is a case of an enum.

```wf
derived open = incidents.filter(i => !i.resolved)
derived byAge = rows.slice().sort((a, b) => a.age - b.age)
derived label = if selected != null { selected.title } else { "none" }
```

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

## Built-in Components

### Layout

| Component | Usage |
|-----------|-------|
| `Container` | `Container { ... }` — centered max-width wrapper |
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
| `Form` | `Form { on submit { save() } ... }` |

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
| `Image` | `Image(src: "/photo.jpg", alt: "Description")` — decoded asynchronously; the first image on a page loads eagerly at high priority (it is usually the largest paint), the rest lazily. `loading:` overrides |
| `Video` | `Video(src: "/video.mp4", controls: true)` |
| `Icon` | `Icon("home")` or `Icon("search").lg.primary` — 30 built-in SVG icons rendered inline |
| `Carousel` | `Carousel(autoplay: true, interval: 5000) { Carousel.Slide { Image(src: "...") } }` — slide track with dots and autoplay |

### Typography

| Component | Usage |
|-----------|-------|
| `Text` | `Text("Hello").bold.muted.center` |
| `Heading` | `Heading("Title").h1` — levels: `.h1` … `.h6` (`.h2` is the default) |
| `Code` | `Code("const x = 1").block` — `.block` for multi-line |
| `Blockquote` | `Blockquote { Text("Quote text") }` |

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

WebFluent includes 30 built-in SVG icons, rendered inline. Available icons:

`home`, `menu`, `search`, `close`, `user`, `settings`, `check`, `plus`, `minus`, `edit`, `trash`, `star`, `heart`, `mail`, `bell`, `download`, `upload`, `eye`, `link`, `calendar`, `filter`, `info`, `warning`, `arrow-left`, `arrow-right`, `chevron-down`, `chevron-right`, `chevron-left`, `logout`, `copy`

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
**Animation** (on every element): `.fadeIn`, `.fadeOut`, `.slideUp`, `.slideDown`, `.slideLeft`, `.slideRight`, `.scaleIn`, `.scaleOut`, `.bounce`, `.shake`, `.pulse`, `.spin`
**Speed** (on every element): `.fast` (150ms), `.slow` (500ms)

## Animation

Every element takes the universal motion props: an enter animation as a
flag or `animate:`, `exit:`, `delay:`, `duration:` (or a speed flag),
`stagger:` inside a list, and `easing:`.

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
"200ms")` in `app`. A reader who asked for less motion
(`prefers-reduced-motion`) gets none: no class, no wait.

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

Four starting points ship in `examples/themes/` — copy one into `src/` and edit
it. They are ordinary source files, not engine settings.

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
├── app.js                # runtime, stores, components — shared by every page
├── pages/<Name>.js       # one chunk per Page, loaded when its route shows
├── pages/<Name>.css      # the style rules only that page reaches
├── styles.css            # tokens, only the component rules the project uses,
│                         # the project's own .css files, and the compiled
│                         # style { } rules more than one page shares
├── *.gz                  # every text file above, gzipped, beside itself
└── …                     # public/ copied to the root, sitemap.xml, robots.txt
```

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
| Motion | `animate(target, name, duration)`; `replay(node, name)`; `animateIn`/`animateOut(node, name, duration, delay)`; `mark(node, attrs)` puts `data-wf-exit`/`data-wf-delay`/`data-wf-duration`/`data-wf-animate` on a component's root — what `when`, `each`, `show` and `match` play before removing an element, and what times its enter animation |
| Routing | `router(routes, container, {transition, duration})` — `fade` or `slide` plays the old page out and the new one in; `navigate(path)`; `params()`; `activeLink(a, href, prefix)`; `page(name, fn)`; `loadPage`; `loadSheet` |
| Data | `resource(url, options)` → `{state, data, error, reload}`; `fetch(url, options)`; `store(def)`; `emit(props, event, …args)`; `locales(default, tables)` → `WF.i18n` |
| Widgets | `toast(message, tone, ms)`; `dialog`; `popup`; `tabs`; `drawer`; `announce(text)`; `carousel`; `tooltip`; `menu`; `field` |
| Boot | `mount(fn, container)`; `hydrate(fn, container)`; `setBasePath`; `setSsgMode`; `__debug` (studio) |

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

## Security headers

`"build": { "csp": true }` emits a strict `Content-Security-Policy` meta tag and a
`_headers` file for hosts that read one. The policy is widened by exactly the
origins `meta.fonts` and `meta.stylesheets` declare (`style-src`, and
`font-src` for the files a font stylesheet references), so a declared font is
never blocked by the policy that ships beside it. The generated output already satisfies
`script-src 'self'` with no `unsafe-inline` — the compiler writes external files
and binds events with `addEventListener` rather than inline `on*` attributes. It
is off by default because a site that later embeds a third-party script would
find it blocked, and that should be a deliberate choice.

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
        "defaultLocale": "en",
        "locales": ["en", "ar"],
        "dir": "src/translations"
    }
}
```

### Usage

```wf
Text(t("nav.home"))                    // Translated text
Text(t("greeting", { name: user.name }))          // With interpolation
Button("EN") { on click { setLocale("en") } }     // Switch locale
Button("AR") { on click { setLocale("ar") } }     // Auto-RTL for Arabic
```

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

## Escaping Braces

In strings, `{` starts interpolation. To use literal braces (e.g., in code blocks), escape with `\{` and `\}`:

```wf
Code("function() \{ return 42; \}").block
```

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

Anything the checker cannot resolve is `Any`, which agrees with everything.

**Warnings.**

| Rule | What it means |
|---|---|
| `A01`–`A12` | WCAG element checks — alt text, form labels, heading outline, table headers. A control is named by `label:`, or by `aria-label:` / `aria-labelledby:` when its visible label is a separate element |
| `A13` | A theme's colour pairing falls below the WCAG AA contrast ratio |
| `A14` | A `role:` that requires particular children (`tablist` → `tab`, `list` → `listitem`, `menu` → `menuitem`, …) holds a control that is not one |
| `A15` | A control's `aria-label` does not contain its visible text — its label, the literals in its block, and what a component in the block renders from its literal props — so what a voice-control user says does not match what they see |
| `S01` | A page has no title |
| `S02` | A page has no description, so its search snippet is written for it |
| `S03` | A description longer than ~160 characters, which a search result truncates |
| `S04` | Two pages claim the same route |
| `V01` | A bare word in an argument that nothing in scope declares — a name misspelled, or a flag written without its dot (`did you mean `.center`?`) |
| `V02` | A flag whose class no stylesheet — the engine's or one of the project's `.css` files — defines; the registry keeps this from happening for the built-ins |
| — | A named argument a built-in does not declare, written to the element as an attribute; a prop a component does not declare, passed anyway |

The heading-outline rules (`A11`, `A12`) do not apply to `Presentation` or
`Document` output, where an `h1` per slide or per section is correct.

## Migrating from WebFluent 2

`wf migrate [path] [--check] [--stdout]` rewrites a project written in
the grammar of WebFluent 2 into the current one — every `.wf` under
`src/`, in place, with a note for anything that needed a decision. It is a
change of spelling and nothing else: the migrated project builds to what
it built before. A WebFluent 2 file given to `wf build` is refused with a
pointer to the migration.

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
        "output_type": "spa",
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
    "dev": { "port": 3000 },
    "meta": {
        "title": "",
        "description": "",
        "favicon": "",
        "lang": "en",
        "fonts": [],
        "stylesheets": []
    },
    "i18n": {
        "defaultLocale": "en",
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
wf render template.wf --data data.json --format fragment

# Render to PDF
wf render template.wf --data data.json --format pdf -o report.pdf

# Pipe JSON from stdin
echo '{"name":"Monzer"}' | wf render template.wf --format html

# With theme
wf render template.wf --data data.json --format html --theme Brand
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
const { Template } = require('@aspect/webfluent');

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
