# WebFluent: the core

What most sites and apps are made of, in one file. Read it once, then write.
When you need something it doesn't cover, don't read the whole reference:
find the section in [language.md](language.md) (an index, one line a
section) and read that one file, or ask the compiler (`wf components Button`,
`wf explain T05`, `wf scope src/pages/Home.wf:12`).

## Project and dev loop

```
my-app/
├── webfluent.app.json   # the config
├── src/App.wf           # app { … Router }: what wraps every page
├── src/pages/*.wf       # a page per file
├── src/components/*.wf  src/stores/*.wf  src/types.wf
├── src/*.css  src/*.js  # your own stylesheets and plain scripts, bundled
├── tests/*.wf           # wf test
└── public/              # copied to the root of build/
```

- `wf init my-app -t spa` (an app) or `-t static` (a site, pre-rendered).
- `wf check` after every change. Each finding has a code; `wf explain CODE`
  says what it means, `wf fix` applies the fixes the compiler knows.
- `wf build` writes `build/`: plain static files.
- `wf start` serves the build in production: `$PORT`, its headers and
  compressed files, a single-page build's shell for every route.
  `--proxy /api=http://localhost:9000/api` forwards a path to an API
  elsewhere. Don't write a server of your own.
- **`wf inspect /route` is how you look at a page**: it loads it in Chrome
  and prints what a reader sees — headings, links, buttons and their
  states, fields, tables, text — and anything that threw or failed to
  load. `--do 'click "Name"'` (or `type "pika" into "Search"`, `press
  "Enter"`, `expect "text"`) acts first and shows the page after; `--mobile`
  for a phone, `--screenshot` to also save a PNG, `--proxy` as `wf start`.
  Use it instead of writing a browser script.
- `wf serve` is the dev server, rebuilding on save. It isn't for production.
- `wf test` runs `tests/`; `wf verify` loads every built page in Chrome and
  fails on a thrown error or a failed request.

## Configuration

```json
{
  "name": "Shop",
  "build": { "ssg": true },
  "meta": {
    "title": "Shop", "description": "What we sell.", "lang": "en",
    "connect": ["https://api.example.com"],
    "img": ["https://cdn.example.com"]
  },
  "env": { "PUBLIC_API": "https://api.example.com" }
}
```

- **A page may only call its own origin** until it says otherwise: every
  build ships a Content-Security-Policy. The simplest way to an API
  elsewhere is through your own origin: `api Backend(base: "/api")` in the
  page, `wf start --proxy /api=$API_BASE_URL/api` where it runs, so its
  address is chosen at start, not at build. To call another origin from
  the page instead, name it in `meta.connect` (images in `meta.img`), or
  the browser blocks it.
- **`env.PUBLIC_API`** is read at build time from the config's `env`, a
  `.env` file or the shell (`PUBLIC_API=https://… wf build`). Only names
  that begin `PUBLIC_` (or that `public_env` lists) can be read by a page,
  since everything a page reads ships in the bundle.
- `wf config schema` lists every key.

## Pages and routes

```wf
type Post { slug: String, title: String, body: String }

const POSTS: [Post] = [
    Post(slug: "hello", title: "Hello", body: "The first post."),
]

page Blog(path: "/blog", title: "Blog", description: "Every post.") {
    derived sort = query.sort ?? "new"
    derived shown = if sort == "az" { POSTS.sortBy(p => p.title) } else { POSTS }
    Heading("Blog").h1
    Row(gap: .sm) {
        Link("Newest", to: "/blog?sort=new")
        Link("A to Z", to: "/blog?sort=az")
    }
    for p in shown by p.slug {
        Link(p.title, to: "/blog/{p.slug}")
    }
}

page Article(path: "/blog/:slug", slug: String, title: "{slug} — Blog", description: "A post.") {
    derived post = POSTS.find(p => p.slug == slug)
    if let p = post {
        Heading(p.title).h1
        Text(p.body)
    } else {
        Heading("No such post").h1
        Link("All posts", to: "/blog")
    }
}

page Missing(path: "*", title: "Not found", description: "Nothing lives here.") {
    Heading("Not found").h1
}
```

- A `:name` segment is a typed parameter of the page. `path: "*"` is the
  404 page.
- `query.name` reads the address's `?name=` (`String?`, so give a default
  with `??`). To change it, link or `navigate("/blog?sort=az")`: the page
  redraws from the new address. `hash` is the `#fragment`.
- `navigate("/path")` goes somewhere from a handler.
- Every page has one `h1`, a `title:` and a `description:`.

The app's frame, in `src/App.wf`:

```wf
app {
    Navbar {
        Navbar.Brand { Link("Shop", to: "/") }
        Navbar.Links {
            Link("Home", to: "/")
            Link("Blog", to: "/blog")
        }
    }
    Router
}
```

## Components

```wf
/// A product, at a glance.
component ProductCard(_ name: String, price: Number, sale: Bool = false) {
    event add(name: String)
    Card {
        Heading(name).h2
        Text(format(price, .currency))
        if sale { Badge("Sale").danger }
        Button("Add {name} to the cart").primary {
            on click { emit add(name) }
        }
        children
    }
}

page Shop(path: "/shop", title: "Shop", description: "What we sell.") {
    state added = ""
    Heading("Shop").h1
    ProductCard("Kettle", price: 39, sale: true) {
        on add(n) { added = n }
        Text("Boils in a minute.")
    }
    if added != "" { Text("Added {added}") }
}
```

- `_ name` is the one prop a call may pass without its name; it comes first.
- A default makes a prop optional. A `Bool` prop is also a flag:
  `ProductCard("Kettle", price: 39).sale`.
- `children` places the caller's block. Named slots: `slot trailing`, filled
  by the caller with `trailing { … }`.
- `event` declares what it fires, `emit` fires it, the caller handles it with
  `on add(n) { … }`. Values go down as props, news comes up as events.

## State

```wf
type Todo { id: String, title: String, done: Bool = false }

page Todos(path: "/todos", title: "Todos", description: "What's left to do.") {
    state items: [Todo] = []
    state draft = ""
    derived left = items.filter(todo => !todo.done).length

    action add() {
        let title = draft.trim()
        if title == "" { return }
        items = items.concat([Todo(id: uuid(), title: title)])
        draft = ""
    }

    Heading("Todos").h1
    Text("{left} left")
    Input(bind: draft, label: "New todo")
    Button("Add").primary { on click { add() } }
    for todo in items by todo.id {
        Checkbox(checked: todo.done, label: todo.title) {
            on change { todo.done = !todo.done }
        }
    }
}
```

- `state` changes; what reads it redraws. `derived` is worked out from
  other values, never assigned. `action` does things; `let` is its local.
- An empty value says its type: `state items: [Todo] = []`,
  `state picked: Todo? = null`.
- Replace values rather than mutate them (`concat`, `map`, `filter`).
  `todo.done = …` on a loop's item is fine: it compiles to a copy.
- `persist theme = "light"` is a state kept in the browser across visits.
- `effect { … }` runs when what it reads changes; `every(1000) { … }` and
  `after(3000) { … }` are timers. Each stops when its page does.

## Types

```wf
enum Tone { calm, loud }
type User { id: String, name: String, email: String?, tone: Tone = .calm }
type Admin = User { role: String }
enum Status { idle, failed(reason: String), done(count: Number) }
```

- Records are built with named fields (`User(id: "1", name: "Ada")`);
  a field with a default may be left out. `T?` may be null: read it with
  `?.`, `??` or `if let`.
- A case is written with a dot: `.loud`, `.failed("timed out")`.
  `match s { .idle { … } .failed(r) { … } .done(n) { … } }`.
- `Number`, `String`, `Bool`, `[T]`, `[String: T]`, `Json`, and the
  language's own `Date`, `DateTime`, `Duration`, `Money`, `Url`, `Email`,
  `Uuid`. `Any` and `Map` are not types.

## Stores

```wf
type Item { name: String, price: Number }

store Cart {
    state items: [Item] = []
    derived total = items.map(i => i.price).sum()
    action add(item: Item) { items = items.concat([item]) }
    action clear() { items = [] }
}

page Checkout(path: "/checkout", title: "Checkout", description: "Pay for the cart.") {
    use Cart
    Heading("Checkout").h1
    Text("{Cart.items.length} items, {format(Cart.total, .currency)}")
    Button("Add a kettle") { on click { Cart.add(Item(name: "Kettle", price: 39)) } }
    Button("Empty the cart") { on click { Cart.clear() } }
}
```

A store is built the first time something reads it. Only the members it
declares exist (`T06` otherwise).

## Data fetching

A `resource` is a read: one value in one of three cases, and a `match`
draws each. Every part of the address that reads state is followed: when
`page` changes, it asks again.

```wf
type Pokemon { name: String, url: String }
type PokemonPage { count: Number, results: [Pokemon] }

const API = env.PUBLIC_API

page Dex(path: "/dex", title: "Pokédex", description: "Every Pokémon, a page at a time.") {
    derived page = parseInt(query.page ?? "1")
    resource list: PokemonPage = http.get("{API}/pokemon?offset={(page - 1) * 20}&limit=20", placeholder: .previous)

    Heading("Pokédex").h1
    match list {
        .loading { Spinner }
        .failed(e) {
            Alert("Could not load: {e.message}").danger
            Button("Try again") { on click { list.reload() } }
        }
        .loaded(data, refreshing) {
            if refreshing { Spinner.sm }
            List {
                for p in data.results by p.name {
                    List.Item { Link(p.name, to: "/dex/{p.name}") }
                }
            }
            Link("Next page", to: "/dex?page={page + 1}")
        }
    }
}
```

- A resource declares its type (`resource list: PokemonPage = …`). The
  answer is checked against it as it arrives; one that doesn't fit is a
  `.failed` with the place it went wrong.
- `.loaded(value, refreshing, stale)`: bind as many as you need.
- Without a `match`: `list.value` (`T?`), `list.isLoading`, `list.error`,
  `list.reload()`.
- `placeholder: .previous` keeps the last page on screen while the next
  loads; `refresh: .every(30.seconds)` or `.onFocus` asks again.
- In an action, a request is awaited and matched:

```wf
type Saved { id: String }

store Notes {
    state said = ""
    action save(text: String) {
        let r: Saved! = await http.post("/api/notes", body: { text: text })
        match r {
            .ok(s) { said = "Saved {s.id}" }
            .err(e) { said = e.message }
        }
    }
}

page Note(path: "/note", title: "A note", description: "Write one down.") {
    use Notes
    state text = ""
    Heading("A note").h1
    Textarea(bind: text, label: "Note")
    Button("Save", loading: Notes.save.pending) { on click { Notes.save(text) } }
    Text(Notes.said)
}
```

A service you call from many places is an `api` block: each endpoint typed,
with its path, retries, caching and what a write makes stale. Read
[language/api-services.md](language/api-services.md)
when you need one. `fetch` is not in the language: use `http` or `api`.

## Control flow

```wf
type Row { id: String, label: String, score: Number }

page Scores(path: "/scores", title: "Scores", description: "Who is ahead.") {
    state rows: [Row] = []
    state open = true
    Heading("Scores").h1
    Switch(bind: open, label: "Show the scores")
    show open {
        if rows.length == 0 {
            Text("Nothing yet.").muted
        } else {
            for r, i in rows.sortBy(r => -r.score) by r.id {
                Text("{i + 1}. {r.label}: {r.score}")
            }
        }
    }
}
```

`if`, `if let x = maybe`, `for item, index in list by item.id` (always key
a list by what identifies an item), `show cond` (keeps the nodes, hides
them), `match`.

## Expressions

- Strings splice: `"{count} left"`, `"{price:.currency}"`,
  `"{when:.date(long)}"`. `\{` is a literal brace.
- `a ?? b`, `a?.b`, `if c { a } else { b }` as a value, `x => x * 2`,
  `[...xs, y]`, `{ ...a, open: true }`, `1..=10`.
- Lists: `filter`, `map`, `find`, `some`, `every`, `sortBy`, `groupBy`,
  `unique`, `take(n)`, `first()`, `last()`, `sum()`, `join`, `includes`.
  Never `xs[0]`: `xs.first() ?? fallback`.
- Strings: `trim`, `toLowerCase`, `includes`, `startsWith`, `split`,
  `replace`, `capitalize()`, `truncate(n)`.
- `format(n, .integer | .decimal | .currency | .percent | .compact)`,
  `format(d, .date, "long")`, `ago(when)`. `parseInt`, `Math.*`,
  `JSON.parse`, `localStorage`, `console` are the browser's, typed.

## Common built-ins

`wf components Name` lists any one's props, flags, events and slots.

| For | Use |
|---|---|
| Layout | `Container`, `Row(gap: .md, align: .center, justify: .between)`, `Stack`, `Grid(columns: { base: 1, md: 3 })`, `Spacer`, `Divider` |
| Text | `Heading("…").h1` … `.h6`, `Text("…").bold.muted.sm`, `Link("…", to: "/…")`, `Code`, `Markdown(text)` |
| Media | `Image(src: "/a.png", alt: "…")`, `Icon("search")`, `Avatar` |
| Data | `Card`, `List { List.Item { … } }`, `Table { Table.Head { Table.Row { Table.Cell("…") } } Table.Body { … } }`, `DataTable`, `Badge("…").success`, `Tag`, `Stat`, `DescriptionList` |
| Forms | `Input(bind: x, label: "…")` (`.email`, `.number`, `.search`), `Select(bind: x, label: "…") { Select.Option("…", value: "…") }`, `Checkbox`, `Radio`, `Switch`, `Slider`, `Textarea`, `Form(bind: form) { on submit { … } }` |
| Navigation | `Navbar`, `Sidebar`, `Tabs(bind: tab) { Tabs.Page("…", value: "…") { … } }`, `Breadcrumb`, `Pagination(bind: page, pages: n)`, `Menu` |
| Feedback | `Alert("…").danger`, `Spinner`, `Skeleton(height: "20px")`, `Progress`, `Modal(visible: open, title: "…")`, `toast("Saved")` |
| Actions | `Button("…").primary { on click { … } }`, `IconButton(icon: "close", label: "Close")` |

Flags come after the parentheses, with a dot, and only the ones the
component declares exist (`E103`): tones `.primary .secondary .success
.danger .warning .info`, sizes `.sm .lg`, `.outlined`, `.full`.

`validate email { required  email }` beside a `state` makes the control bound
to it show what's wrong; see
[language/validation.md](language/validation.md).

## Attributes, handlers and refs

- `on click { … }`, `on input { … }`, `on keydown(e) { if e.key == "Enter" { … } }`,
  `on submit(e) { … }`. `on input(debounce: 300.ms) { … }` waits for a pause.
- `on key("mod+k") { … }` at the top of a page is a shortcut.
- Any element takes `id:`, `role:`, `aria-*:`, `data-*:`, `class:` (added to
  its own; a map turns classes on by condition:
  `class: { "is-done": todo.done }`), and `ref: name` for a handle
  (`name.focus()`).

## Styling

```wf
page Plans(path: "/plans", title: "Plans", description: "What it costs.") {
    Heading("Plans").h1
    Grid(columns: { base: 1, md: 3 }, gap: .lg) {
        Card(class: "plan") {
            style {
                padding: $spacing-lg
                border-radius: $radius-md
                &:hover { box-shadow: $shadow-md }
                @md { padding: $spacing-xl }
            }
            Heading("Free").h2
        }
    }
}
```

- A `style { }` block is CSS, first in its element's block. Values aren't
  quoted; two on one line are separated by `;`.
- `$name` is a design token; `wf theme` lists them all. Use tokens, not
  hard-coded colours, so the dark theme follows.
- `&:hover`, `&:focus-visible`, `@md { … }` and `@media (…) { … }` work
  inside it. `{expr}` in a value follows state: `width: {pct}%`.
- A `.css` file anywhere under `src/` is bundled; `class: "plan"` picks up
  its rules, and `var(--color-primary)` reaches a token.
- A `theme Brand { color-primary: #0F766E }` block in `src/` restyles every
  built-in.

## JavaScript interop

Only when the language has no way. A `.js` file under `src/` is a plain
script (no `import`/`export`), linked on every page; a function in it is
called from `.wf` by name, and its JSDoc is its signature:

```js
/**
 * @param {string} text
 * @returns {number}
 */
function wordCount(text) {
  return text.split(/\s+/).filter(Boolean).length;
}
```

## Tests

```wf
test "adding a todo shows it" {
    Todos
    type "Milk" into "New todo"
    click "Add"
    expect "Milk"
    expect "1 left"
}
```

Steps find things by what a reader sees: a button's text, a field's label.
More in [testing.md](testing.md).

## Further reference

Read one section, not the whole reference:
[language.md](language.md) lists each with its size. The other references
here cover code quality, recipes, styling, testing, diagnostics, PDFs and
slides, and Tauri apps.
