# 35. Tutorials

<!--
route: tutorials
group: shipping
blurb: Three projects built step by step — a todo app, a statically built notes site, a budget tracker — none needing a server.
description: Three WebFluent tutorials, easy to hard — Todos, Field Notes and Pocket — each a complete project with no backend, built step by step.
-->

Three projects, each built step by step and each complete in the
repository under `examples/tutorials/`. None needs a server: what they
keep, they keep in the reader's browser. Pick the one that matches where
you are — each stands on its own — or do them in order.

| | Project | What you build | What it teaches | Time |
|---|---|---|---|---|
| **Easy** | [Todos](#easy-todos) | A list of things to do: add, tick, rename, filter | Types, a store with `persist`, a component, a keyed list, a filter in the address, tests that click | 20 minutes |
| **Medium** | [Field Notes](#medium-field-notes) | A notebook site, statically built, a page per note | `data` files, a layout, `:slug` pages with `paths:`, Markdown pages, search and tags, what a static page can know | An hour |
| **Hard** | [Pocket](#hard-pocket) | A budget tracker with a dashboard, forms and two languages | `Money` and `Date`, derived totals, a `persist` migration, `validate`, a dialog and shortcuts, i18n and RTL, a script of your own, offline | A couple of hours |

Every block that names a file — `// src/stores/Todos.wf` on its first line
— is that file, or a run of it, word for word: a test holds the chapter to
the projects. To start from a finished one:

```bash
git clone https://github.com/monzeromer-lab/WebFluent
cd WebFluent/examples/tutorials/easy-todos
wf serve
```

If you have not written WebFluent before, [the Shelf tutorial](03-tutorial.md)
is a gentler first half hour.

## Easy: Todos

A list of things to do, kept in the reader's browser: add one, tick it,
rename it, remove it, show the open ones. One page, one store, one
component — about twenty minutes. The finished project is
[`examples/tutorials/easy-todos`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/easy-todos).

### Step 1: a project

```bash
wf init todos -t spa
cd todos
```

Delete what the template put in `src/` — you write every file yourself —
and replace `webfluent.app.json`:

```json
{
  "name": "todos",
  "version": "1.0.0",
  "theme": { "name": "Paper", "dark": "Night" },
  "build": { "output": "./build", "output_type": "spa", "csp": true },
  "meta": {
    "title": "Todos",
    "description": "A small list of things to do, kept in your browser.",
    "lang": "en",
    "favicon": "/favicon.svg"
  }
}
```

`spa` is a single-page app: one HTML shell, pages swapped without a reload
([Static sites and SPAs](26-static-and-spa.md)). `csp` ships a content
security policy and holds the build to it ([Security](23-security.md)).
Run `wf serve` in another terminal and leave it running: every save
rebuilds and reloads the page.

### Step 2: what a todo is

```wf
// src/types.wf
/// One thing to do.
type Todo {
    id: Uuid
    title: String
    done: Bool = false
}

/// Which todos the list shows.
enum Filter { all, open, done }
```

A `type` names the fields a value has, and the checker holds every use to
them: `todo.titel` is an error where it is written, with the field you
meant ([Types](13-types.md)). `Uuid` is one of the types the language brings;
`uuid()` makes one. `done` has a default, so a new todo can leave it out.
An `enum` is a fixed set of cases, written `.open` where a `Filter` is
wanted.

### Step 3: a store that remembers

```wf
// src/stores/Todos.wf
/// The list, kept in the browser between visits.
store Todos {
    persist items: [Todo] = []

    derived open = items.filter(t => !t.done)
    derived done = items.filter(t => t.done)

    action add(title: String) {
        let text = title.trim()
        if text == "" { return }
        items = items.concat([Todo(id: uuid(), title: text)])
    }

    action toggle(id: Uuid) {
        items = items.map(t => if t.id == id { { ...t, done: !t.done } } else { t })
    }
```

A store is state any page or component can read ([Stores](12-stores.md)).
`persist` is `state` that is kept in the browser's storage: the list is
there after a reload, and a second tab follows the first, with no storage
code written. `derived` values follow what they read — `open` is always the
todos not done. An action is the only place state changes; `{ ...t, done:
!t.done }` is the todo with one field changed.

The rest of the store renames, removes and clears in the same way — see
the [whole file](https://github.com/monzeromer-lab/WebFluent/blob/master/examples/tutorials/easy-todos/src/stores/Todos.wf).

### Step 4: one row

```wf
// src/components/TodoRow.wf
/// One row of the list: a box to tick, the title, and buttons to rename and
/// remove it.
component TodoRow(_ todo: Todo) {
    use Todos
    state editing = false
    state draft = todo.title

    action save() {
        Todos.rename(todo.id, draft)
        editing = false
    }

    Row(class: ["todo", if todo.done { "is-done" }], align: .center, gap: .sm) {
        Checkbox(checked: todo.done, aria-label: "Done: {todo.title}") {
            on change { Todos.toggle(todo.id) }
        }
        if editing {
            // `mount:` runs when the input is made: the cursor goes straight in.
            Input(bind: draft, aria-label: "New title for {todo.title}", class: "todo__edit", mount: (n) => n.focus()) {
                on key("Enter") { save() }
                on key("Escape") { draft = todo.title  editing = false }
                on blur { save() }
            }
        } else {
            Text(todo.title, class: "todo__title")
            IconButton(icon: "edit", label: "Rename {todo.title}", class: "todo__action").sm {
                on click { draft = todo.title  editing = true }
            }
        }
        IconButton(icon: "trash", label: "Remove {todo.title}", class: "todo__action").sm {
            on click { Todos.remove(todo.id) }
        }
    }
}
```

A component is an element of your own ([Components](11-components.md)).
`_ todo` is its one positional prop, so a call reads `TodoRow(todo)`. Its
`state` belongs to each row: one row being renamed does not put the others
in edit mode. `class:` adds classes to the element's own — a list may hold
an `if`, which adds `is-done` only while the todo is done. Every control
has an accessible name: the checkbox and the icon buttons say which todo
they act on, so a screen reader — and the tests in step 7 — can tell them
apart ([Accessibility](22-accessibility.md)).

### Step 5: the page

```wf
// src/pages/Home.wf
page Home(path: "/", title: "Todos", description: "A small list of things to do, kept in your browser.") {
    use Todos
    state draft = ""

    // The filter lives in the address — `/?show=open` — so a reload or a
    // shared link keeps it.
    derived filter: Filter = if query.show == "open" { .open } else if query.show == "done" { .done } else { .all }
    derived shown = match filter {
        .open { Todos.open }
        .done { Todos.done }
        .all { Todos.items }
    }

    action add() {
        Todos.add(draft)
        draft = ""
    }

    Stack(class: "card", gap: .md) {
        Row(justify: .between, align: .baseline) {
            Heading("Todos").h1
            Text("{Todos.open.length} left", class: "count")
        }

        Row(class: "new", gap: .sm) {
            Input(bind: draft, placeholder: "What needs doing?", aria-label: "New todo") {
                on key("Enter") { add() }
            }
            Button("Add", disabled: draft.trim() == "").primary {
                on click { add() }
            }
        }
```

`bind: draft` ties the input to the state both ways: typing changes
`draft`, and `draft = ""` empties the box. `query` is the address's query
string as a value, kept current, so `filter` follows the address and
`shown` follows `filter`. A `match` over an enum must cover every case — a
fourth filter added to `Filter` would be an error here until it is
handled. The rest of the page draws the list:

```wf
// src/pages/Home.wf
        Row(class: "filters", gap: .xs, align: .center) {
            Link("All", to: "/?show=all", class: { "is-on": filter == .all })
            Link("Open", to: "/?show=open", class: { "is-on": filter == .open })
            Link("Done", to: "/?show=done", class: { "is-on": filter == .done })
            Spacer
            if Todos.done.length > 0 {
                Button("Clear done").sm {
                    on click { Todos.clearDone() }
                }
            }
        }

        if shown.length == 0 {
            Text(if Todos.items.length == 0 { "Nothing yet — add your first todo above." } else { "Nothing here." }, class: "empty")
        }
        Stack(class: "list") {
            for todo in shown by todo.id {
                TodoRow(todo, animate: .fadeIn, exit: .fadeOut)
            }
        }
    }
```

`for … by todo.id` is a keyed list: when a todo is ticked away by the
filter or renamed, only its row changes — the others keep their nodes, and
the row being edited keeps its cursor ([Control flow](10-control-flow.md)).
`animate:` and `exit:` fade a row in and out. A class map, `{ "is-on":
filter == .all }`, turns a class on while its condition holds.

And the frame every page sits in:

```wf
// src/App.wf
// The frame every page sits in: a narrow column, centred.
app {
    Container(class: "shell") {
        Router
    }
}
```

### Step 6: a look of its own

```wf
// src/theme.wf
/// Warm paper and ink, with one teal for what you can act on.
theme Paper {
    color-primary: #0F766E
    color-background: #F6F4EF
    color-surface: #FFFFFF
    color-text: #1F2328
    color-text-muted: #5E6670
    color-border: #E4E0D8
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif
    radius-md: 10px
}
```

A theme names only the design tokens it changes ([Styling](15-styling.md));
`Night`, named as `"dark"` in the config, is used when the reader's system
is dark. The classes the page names — `card`, `todo`, `is-done`, `filters`
— are styled in `src/todos.css`, which the build bundles because it is
under `src/`:

```css
.todo { flex-wrap: nowrap; padding: 0.6rem 0.25rem; border-top: 1px solid var(--color-border); }
.todo__title { flex: 1; }
.todo.is-done .todo__title { text-decoration: line-through; color: var(--color-text-muted); }
.filters a.is-on { background: var(--color-background); color: var(--color-text); font-weight: 600; }
```

### Step 7: tests

```wf
// tests/todos.wf
test "adding a todo puts it in the list" {
    use Todos
    state draft = ""
    Input(bind: draft, aria-label: "New todo")
    Button("Add") { on click { Todos.add(draft)  draft = "" } }
    for t in Todos.items by t.id { TodoRow(t) }
    Text("{Todos.open.length} left")

    expect "0 left"
    type "Buy bread" into "New todo"
    click "Add"
    expect "Buy bread"
    expect "1 left"
}
```

A test draws what it declares and then follows its steps in order: it
types into the control named "New todo" and clicks the button named "Add",
the way a reader would, in a headless Chrome ([Testing](24-testing.md)).
`wf test` runs every test under `tests/`; the project has six — a blank
todo is refused, ticking and clearing empties the list, renaming keeps one
row for each.

### Step 8: build it

```bash
wf check
wf test
wf build
```

`wf check` reports every finding with nothing written; `wf build` writes
`build/`, which any static host serves — [Deploying](29-deploying.md) has
GitHub Pages, Netlify and the rest. `wf verify` loads the built page in a
real browser and fails on an error, a missing file or a page that draws
nothing.

## Medium: Field Notes

A notebook kept in public — notes from a garden, a kitchen and a few trips
— built as a static site. The notes are a JSON file read when the site is
built; every page, one per note included, is written out as HTML a search
engine reads; the one thing a reader changes, the notes they save for
later, stays in their browser. About an hour. The finished project is
[`examples/tutorials/medium-field-notes`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/medium-field-notes).

### Step 1: a static project and its look

```bash
wf init field-notes -t static
cd field-notes
```

```json
{
  "name": "field-notes",
  "version": "1.0.0",
  "author": "Field Notes",
  "theme": { "name": "Meadow", "dark": "Dusk" },
  "build": {
    "output": "./build",
    "ssg": true,
    "csp": true
  },
  "meta": {
    "title": "Field Notes",
    "description": "Notes from a small garden, a busy kitchen and the roads between them.",
    "lang": "en",
    "site_url": "https://field-notes.example.com",
    "site_name": "Field Notes",
    "favicon": "/favicon.svg",
    "image": "/card.png",
    "sitemap": true
  }
}
```

`ssg` pre-renders every page to its own `index.html`, and the script takes
the page over when it loads ([Static sites and SPAs](26-static-and-spa.md)).
`site_url` lets the build write what search engines and link previews read:
a canonical address, Open Graph tags, JSON-LD, `sitemap.xml` and
`robots.txt` ([Content](27-content.md)).

```wf
// src/theme.wf
/// The dark theme: only the tokens that change.
theme Dusk {
    color-primary: #86C9A6
    color-secondary: #E8A086
    color-background: #121614
    color-surface: #1A201D
    color-text: #E9E5DA
    color-text-muted: #A6AFA9
    color-border: #2B3430
    paper-raised: #171C19
    tag-bg: #213129
    tag-text: #A9DCC1
}
```

The light theme, `Meadow`, sets the same names. A theme may declare tokens
of its own — `paper-raised`, `tag-bg` — and the project's stylesheet,
`src/styles.css`, reads every colour as a token (`var(--tag-bg)`), so the
dark theme needs no rule of its own ([Design tokens](41-design-tokens.md)).

### Step 2: typed data

```wf
// src/types.wf
/// One note in the notebook.
type Note {
    slug: String
    title: String
    date: Date
    tags: [String]
    summary: String
    minutes: Number
    body: String
}

/// Every note, read from `notes.json` when the site is built.
data notes: [Note] = "notes.json"
```

`data` reads a JSON file at build time: it is a constant, typed by the
annotation, inlined into the bundle and seeded into every pre-rendered page
([Data](17-data.md)). The checker holds the program to `Note`: a page that
reads `note.titel` is an error where it is written. `date` is a `Date`, a
plain `"2026-03-08"` string the language knows how to format — `{note.date:.date(long)}`
is "March 8, 2026". `src/notes.json` holds seven notes, each with a Markdown
`body`.

### Step 3: the shell and a layout

```wf
// src/App.wf
app {
    use Saved

    Navbar(class: "site-nav") {
        Navbar.Brand {
            Link("Field Notes", to: "/", class: "brand")
        }
        Navbar.Links {
            Link("Notes", to: "/")
            Link("Tags", to: "/tags")
            Link(to: "/saved", class: "saved-link") {
                Text("Saved")
                if Saved.count > 0 {
                    Badge("{Saved.count}").pill
                }
            }
            Link("About", to: "/about")
        }
        Navbar.Actions {
            ThemeToggle
        }
    }

    Router(transition: .fade, duration: "160ms")
```

`app` is what wraps every page; the current page is drawn where `Router`
stands, inside `<main>`, so the skip link a page carries jumps past the
navigation ([Pages and routing](06-pages-and-routing.md)). The link whose
`to:` is the page shown is marked `aria-current="page"`. Each page names
the column it is set in with `layout:` — a component whose default slot is
the page:

```wf
// src/components/Sheet.wf
/// The column a page is set in, and the frame every page names with
/// `layout: Sheet`. A narrow sheet is for reading; a wide one holds a grid.
component Sheet(wide: Bool = false) {
    slot
    Container(class: ["sheet", if wide { "sheet--wide" }]) {
        children
    }
}
```

`ThemeToggle` is two icon buttons that call `setTheme("dark")` and
`setTheme("light")`; the choice is kept across visits.

### Step 4: a card

```wf
// src/components/NoteCard.wf
/// A note at a glance: when it was written, its title, a summary, its tags
/// and the button that saves it.
component NoteCard(
    _ note: Note,
    /// Keeps the card in the page but out of sight — what a filter does.
    hidden: Bool = false
) {
    Card(class: "note-card", hidden: hidden) {
        Stack(gap: .sm) {
            Row(justify: .between, align: .center, gap: .sm) {
                Text("{note.date:.date(long)} · {note.minutes} min read", class: "note-meta").sm
                SaveButton(note.slug, title: note.title)
            }
            Link(to: "/notes/{note.slug}", class: "note-card__link") {
                Heading(note.title).h2
            }
            Text(note.summary, class: "note-card__summary")
            TagList(note.tags)
        }
    }
}
```

A `///` comment above a component or a prop is its documentation, shown
when the editor hovers a call ([Components](11-components.md)). A splice
may say how to show its value: `{note.date:.date(long)}` is
`format(note.date, .date, "long")` written where it is read
([Expressions](14-expressions.md)). `TagList` is a row of `#tag` links to
`/?tag=…`.

### Step 5: the front page — a search and a filter in the address

```wf
// src/components/NoteFinder.wf
    state q = ""

    derived needle = q.trim().toLowerCase()
    derived newest = all.sortBy(n => n.date).reverse()
    derived shown = newest.filter(n =>
        (tag == "" || n.tags.includes(tag))
        && (needle == ""
            || n.title.toLowerCase().includes(needle)
            || n.summary.toLowerCase().includes(needle)))
    derived visible = shown.map(n => n.slug)
    derived allTags = all.flatMap(n => n.tags).unique().sort()
```

`q` is bound to the search box; everything else follows it. `sortBy`,
`flatMap` and `unique` are list helpers the language brings, and each runs
at build time too, so the pre-rendered page shows the sorted list
([State and reactivity](08-state-and-reactivity.md)).

```wf
// src/components/NoteFinder.wf
            // Every note is drawn, and the ones that do not match are hidden.
            // A pre-rendered page cannot know the URL's query, so a list
            // filtered by it would be missing from the HTML; this way the
            // page carries every note, and the filter applies once it is live.
            for note in newest by note.slug {
                NoteCard(note, hidden: !visible.includes(note.slug))
            }
```

This is the one idea a static site adds: the HTML is written once, before
any reader arrives, so it cannot depend on what only the browser knows — the
address's query, what is in storage. Drawing every note and hiding the
misses means the HTML holds them all, for a reader without script and for a
search engine, and the filter applies the moment the page is live. The
front page hands the finder the address's tag:

```wf
// src/pages/Home.wf
        // `query.tag` is the URL's `?tag=…`; it is missing when the URL has
        // none, and `?? ""` turns that into the finder's "every note".
        NoteFinder(notes, tag: query.tag ?? "")
```

### Step 6: a page per note

```wf
// src/pages/NotePage.wf
page NotePage(path: "/notes/:slug", slug: String,
    title: "{notes.find(n => n.slug == slug)?.title ?? slug} — Field Notes",
    description: "{notes.find(n => n.slug == slug)?.summary ?? slug}",
    type: "article",
    layout: Sheet,
    paths: notes.map(n => n.slug)) {

    derived note = notes.find(n => n.slug == slug)
```

`:slug` in the path is a typed parameter of the page. `paths:` names the
values a static build renders it for — one `notes/<slug>/index.html` per
note, each listed in the sitemap ([Content](27-content.md)). `title:` and
`description:` may compute from the parameters, so each note's search
result shows its own title and summary. The body is the note's Markdown:

```wf
// src/pages/NotePage.wf
            Markdown(n.body)
```

`Markdown` renders headings, lists, quotes, emphasis and links, at build
time and live; the text is escaped first, so HTML in a note is shown, not
run ([Elements](07-elements.md)). Below it, `related` is the three newest
notes that share a tag — a `derived` value like any other.

### Step 7: tags, and a page written in Markdown

```wf
// src/pages/Tags.wf
        Grid(columns: { base: 1, md: 2 }, gap: .md, align: .start) {
            for tag in allTags by tag {
                derived filed = notes.filter(n => n.tags.includes(tag)).sortBy(n => n.date).reverse()
```

A layout prop takes one value per breakpoint — one column on a phone, two
from `md` up — compiled to a media query, so the first paint is right
([Styling](15-styling.md)). A `derived` value inside a `for` is worked out
per item.

The About page is not a `.wf` file at all: `src/about.md` is a page,
its front matter naming its path, title, description and layout:

```markdown
---
path: /about
title: About — Field Notes
description: Who keeps this notebook, why it exists, and how the site is built.
layout: Sheet
---
# About
```

### Step 8: the reader's saved notes

```wf
// src/stores/Saved.wf
/// The notes this reader has saved for later, kept in their browser.
store Saved {
    persist slugs: [String] = []

    derived count = slugs.length
```

`persist` keeps the list in the browser across visits — there is no
account and nothing is sent anywhere ([Stores](12-stores.md)). The button
that saves a note says whether it is pressed, the way assistive technology
reads it:

```wf
// src/components/SaveButton.wf
/// Saves a note for later, or takes it off the saved list.
component SaveButton(_ slug: String, title: String) {
    use Saved

    Button(if Saved.slugs.includes(slug) { "Saved" } else { "Save" },
        aria-pressed: Saved.slugs.includes(slug),
        aria-label: if Saved.slugs.includes(slug) { "Saved: {title}" } else { "Save: {title}" },
        class: "save-button").sm.outlined {
        on click { Saved.toggle(slug) }
    }
}
```

`aria-pressed` follows the state, and `styles.css` styles the pressed
button with `[aria-pressed="true"]` — the look and the meaning are one
attribute ([Accessibility](22-accessibility.md)). The Saved page lists
`notes.filter(n => Saved.slugs.includes(n.slug))`, and the navbar's badge
is `Saved.count`.

### Step 9: tests, and the checks a static site deserves

```wf
// tests/NoteFinder.wf
test "a search that matches nothing says so, and clears" {
    NoteFinder(notes)
    type "zeppelin" into "Search the notes"
    expect "Nothing matches"
    expect "0 of 7 notes"
    click "Clear the search"
    expect "7 of 7 notes"
    expect not "Nothing matches"
}
```

A test reads the project's `data` file like any page does
([Testing](24-testing.md)). Then:

```bash
wf test
wf check --deny-warnings
wf build
wf verify
```

`wf verify` loads every route of the built site in a headless Chrome — each
note's page at its `paths:` value — and fails on an error, a missing file or
a page that draws nothing ([Command line](37-cli.md)). `build/` is ready
for any static host.

## Hard: Pocket

A personal budget tracker that lives entirely in the reader's browser:
money in and money out, a budget per category, what is left of it this
month, six months of bars, two languages, a CSV out and back in, and a page
that keeps working offline. No server and no account. A couple of hours;
it assumes the easy tutorial, and every step links the chapter that goes
deeper. The finished project is
[`examples/tutorials/hard-pocket`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/hard-pocket).

### Step 1: a project

```json
{
  "name": "pocket",
  "version": "1.0.0",
  "author": "WebFluent tutorials",
  "theme": { "name": "Pocket", "dark": "PocketNight" },
  "build": {
    "output": "./build",
    "output_type": "spa",
    "csp": true
  },
  "meta": {
    "title": "Pocket",
    "description": "A budget that lives in your browser: income, spending and what is left, month by month.",
    "lang": "en"
  },
  "i18n": {
    "default_locale": "en",
    "locales": ["en", "ar"],
    "dir": "src/translations"
  },
  "offline": {
    "precache": ["/", "/entries", "/categories", "/settings"],
    "fallback": "/"
  }
}
```

Three settings this project adds to the easy one: `i18n` names the
languages and where their messages live ([Internationalization](21-i18n.md));
`offline` makes `wf build` write a service worker that stores the four
pages, so a second visit opens with no network ([Offline](19-offline.md));
and the two themes, light and dark, both of which the build ships.

### Step 2: what money is

```wf
// src/types.wf
/// Which way the money went.
enum Kind { income, expense }

/// Something money is spent on, with what the reader means to spend on it
/// in a month.
type Category {
    id: String
    name: String
    color: Color
    budget: Money
}

/// One line of the ledger: money in, or money out.
type Entry {
    id: Uuid
    date: Date
    amount: Money
    kind: Kind
    category: String
    note: String = ""
}
```

`Money`, `Date`, `Color` and `Uuid` are types the language brings, each a
plain JSON value at run time ([Types](13-types.md)). Money is kept in
minor units, `{ amount: 1299, currency: "USD" }`, so a sum never drifts by a
fraction of a cent; it is written `$12.99` and shown with `format(m)`. Its
methods are calls — `a.plus(b)`, `a.minus(b)`, `ONE.times(12.5)` — and its
carrier's fields are fields: `m.amount`.

```wf
// src/types.wf
/// Nothing yet: the starting point every sum adds to.
const ZERO: Money = $0.00

/// One dollar — `ONE.times(12.5)` is $12.50, rounded to the cent.
const ONE: Money = $1.00
```

### Step 3: a ledger that adds itself up

```wf
// src/stores/Ledger.wf
    derived today = now.date()
    derived newestFirst = entries.sortBy(e => e.date).reverse()

    derived income = sum(entries.filter(e => e.kind == .income))
    derived spent = sum(entries.filter(e => e.kind == .expense))
    derived balance = income.minus(spent)

    derived thisMonth = entries.filter(e => e.date.startOfMonth().isSame(today.startOfMonth()))
    derived monthIncome = sum(thisMonth.filter(e => e.kind == .income))
    derived monthSpent = sum(thisMonth.filter(e => e.kind == .expense))
    derived monthBudget = categories.reduce((s, c) => s.plus(c.budget), ZERO)
    derived budgetLeft = monthBudget.minus(monthSpent)
```

Every figure on every page is a `derived` value of the store: add an entry
and the balance, this month's spending, each budget bar and the chart all
follow, with nothing written to keep them in step ([Stores](12-stores.md)).
`now` is the browser's clock as a value, so `today` turns over at midnight.
`sum` is an action that only returns a value, which a derived value may
call:

```wf
// src/stores/Ledger.wf
    // The money in a list of entries, added up.
    action sum(list: [Entry]) {
        return list.reduce((total, e) => total.plus(e.amount), ZERO)
    }
```

### Step 4: changing the shape without losing anyone's data

```wf
// src/stores/Ledger.wf
    // Version 1 of Pocket kept an amount as a plain, signed number of
    // dollars (`-52.4` was money out). Version 2 keeps a `Money` and a
    // `Kind`, so `migrate 1 -> 2` turns what a returning reader's browser
    // holds into the new shape instead of throwing it away.
    persist entries: [Entry] = SAMPLE_ENTRIES.map(s => Entry(
            id: s.id,
            date: now.date().minus(days: s.ago),
            amount: s.amount,
            kind: s.kind,
            category: s.category,
            note: s.note,
        )) {
        version: 2
        migrate 1 -> 2 {
            old.map(e => Entry(
                    id: e.id,
                    date: e.date,
                    amount: ONE.times(Math.abs(e.amount)),
                    kind: if e.amount < 0 { .expense } else { .income },
                    category: e.category,
                    note: e.note ?? "",
                ))
        }
    }
```

What a `persist` keeps outlives the build that wrote it: a reader who
visited last month has last month's shape in their browser. `version: 2`
says what this build writes; `migrate 1 -> 2` brings an older value
forward, given as `old`. Raise the version without a migration and the
build warns (`P01`); change the shape without raising it and it warns too
(`D04`), from the shapes it keeps in `.wf-cache/`
([Stores](12-stores.md)). A first visit opens on a sample ledger dated
from today, so the dashboard is never empty.

### Step 5: a shell that fits the screen

```wf
// src/App.wf
            if Ui.menuOpen {
                Host(tag: "div", class: "scrim") { on click { Ui.closeAll() } }
                Stack(gap: .md, class: "drawer", animate: .slideRight, exit: .fadeOut) {
                    on click { Ui.menuOpen = false }
                    Brand
                    NavLinks
                }
            }
```

The shell draws a `Sidebar` when `viewport.lg` holds and a top bar with
this drawer when it does not. `viewport` is the window's size as a value,
following `matchMedia`, so the switch happens once as the window crosses
the breakpoint ([Styling](15-styling.md)). The layout itself — grids that
are one column on a phone — is responsive values, `Grid(columns: { base: 1,
lg: 2 })`, which need no script. A second store, `Ui`, holds what the
interface is doing — the dialog, the menu, the notices — apart from what the
ledger holds.

### Step 6: a dashboard drawn from elements

```wf
// src/components/MonthChart.wf
/// Six months of money in and money out, as pairs of bars.
///
/// The built-in `Chart` draws at build time, from data the build can see.
/// This ledger only exists in the reader's browser, so the chart is made of
/// plain elements whose heights follow the store: `height: {share}%` is a
/// style value that reads state, and repaints when it changes.
component MonthChart(months: [MonthTotal], top: Number) {
    Stack(gap: .sm) {
        Row(class: "chart", align: .end, role: "img", aria-label: t("chart.label")) {
            for m in months by m.month {
                Stack(gap: .xs, align: .center, class: "chart__month") {
                    Row(gap: .xs, align: .end, class: "chart__bars") {
                        Host(tag: "span", class: "chart__bar chart__bar--in") {
                            style { height: {m.income.amount / top * 100}% }
                        }
                        Host(tag: "span", class: "chart__bar chart__bar--out") {
                            style { height: {m.spent.amount / top * 100}% }
                        }
                    }
```

A `{…}` in a style value reads state, and the property follows it
([Styling](15-styling.md)). The chart is one `role="img"` with a label, so
a screen reader hears what it shows rather than twelve unnamed boxes
([Accessibility](22-accessibility.md)). The figures above it are one
component placed four times:

```wf
// src/pages/Dashboard.wf
    Grid(columns: { base: 2, xl: 4 }, gap: .md) {
        KpiCard(t("kpi.balance"), value: Ledger.balance, note: t("kpi.balanceNote"))
        KpiCard(t("kpi.income"), value: Ledger.monthIncome, note: t("kpi.thisMonth"), lean: .good)
        KpiCard(t("kpi.spent"), value: Ledger.monthSpent, note: t("kpi.thisMonth"), lean: .bad)
        KpiCard(t("kpi.left"), value: Ledger.budgetLeft, note: t("kpi.leftNote", { budget: format(Ledger.monthBudget) }),
            lean: if Ledger.budgetLeft.amount < 0 { .bad } else { .good })
    }
```

`lean` is an enum prop, and a component's enum props are written to its
root element as `data-lean="good"`, which `pocket.css` colours by
([Components](11-components.md)).

### Step 7: filters in the address

```wf
// src/pages/Entries.wf
    // The filters live in the address — `/entries?month=2026-09&category=dining` —
    // so they survive a reload, a shared link and the back button.
    derived month = query.month ?? ""
    derived category = query.category ?? ""
    derived shown = Ledger.newestFirst.filter(e => (month == "" || e.date.slice(0, 7) == month) && (category == "" || e.category == category))
```

Choosing a month calls `navigate("/entries?month=…")`; the address changes,
`query` follows it, and so does the list. An entry has a page of its own at
`/entries/:id`, where `id` is a typed parameter; the page looks the entry
up with `Ledger.entries.find(…)` and draws it inside `if let e = entry` —
`if let` binds the value only when it is there, which the checker requires
before reading its fields
([Pages and routing](06-pages-and-routing.md)).

### Step 8: one form for new and for edit

```wf
// src/components/EntryForm.wf
/// The form for a new entry — or, given one, for changing it. It checks what
/// the reader typed, writes it to the ledger, and says so with `saved`.
component EntryForm(entry: Entry? = null) {
    use Ledger
    event saved()

    state kind: Kind = entry?.kind ?? .expense
    state amount: Number? = if let e = entry { e.amount.amount / 100 } else { null }
    state date: Date? = entry?.date ?? Ledger.today
    state category = entry?.category ?? "groceries"
    state note = entry?.note ?? ""

    validate amount {
        required
        min(0.01)
    }
    // Not in the future. With no message of its own, the rule shows the
    // translation of `form.max`; `Ledger.today` is read each time it checks.
    validate date {
        required
        max(Ledger.today)
    }
    validate note { maxLength(60) }
```

A `validate` block sits beside the state it guards, and the control bound
to that state shows what the rules say — the message, `aria-invalid`, the
link between them — with nothing written ([Forms](16-forms.md)). A rule
without a message of its own shows the translation of `form.<rule>`, so
`en.json` and `ar.json` each say "that date has not happened yet" once. The
form says when it is done with an event; the caller decides what that means
— close the dialog, or go back to the entry:

```wf
// src/components/EntryForm.wf
        Row(justify: .end, gap: .sm) {
            Button(if entry != null { t("action.save") } else { t("action.add") }, type: .submit, disabled: !form.valid).primary
        }
```

`Form(bind: form)` hands a handle on the form; `form.valid` is whether
every rule holds, and the button follows it as the reader types.

### Step 9: a dialog and a shortcut

```wf
// src/App.wf
    // `n` opens a new entry from anywhere. A key with no modifier is left to
    // the field the reader is typing in, so an `n` in a note is just a letter.
    on key("n") { Ui.compose() }
    on key("Escape") { Ui.closeAll() }
```

`on key` at the top of the app listens on the document for as long as the
page shows ([Events](09-events.md)). The dialog is a `Modal` whose `title:`
is its accessible name; it traps focus while open, and `Escape` closes it:

```wf
// src/App.wf
    Modal(visible: Ui.composing, title: t("compose.title")) {
        if Ui.composing {
            EntryForm {
                on saved {
                    Ui.composing = false
                    Ui.notify(t("toast.added"))
                }
            }
        }
    }
```

### Step 10: two languages

```json
{
  "entries.zero": "No entries",
  "entries.one": "{count} entry",
  "entries.other": "{count} entries"
}
```

Every word on screen is `t("key")`, read from `src/translations/en.json` or
`ar.json`. A message with plural forms is several keys, and `t("entries", {
count: n })` picks the form the language's rules call for — Arabic has six
([Internationalization](21-i18n.md)). Switching to Arabic sets `dir="rtl"`
on the page, and the sidebar, the rows and the grids mirror themselves.
`format(m)` and dates follow the language too. The reader's choice is a
`persist` in a small `Prefs` store, applied on every visit by an `effect`
in the app shell.

### Step 11: your data out, and back in

```wf
// src/pages/Settings.wf
    action exportCsv() {
        let lines = Ledger.newestFirst.map(e => csvLine(e))
        saveTextFile("pocket-{Ledger.today}.csv", [CSV_HEADER, ...lines].join("\n"), "text/csv")
    }
```

`saveTextFile` is not WebFluent: it is a function in `src/files.js`, a plain
browser script the build links on every page and whose JSDoc types the
call, so `saveTextFile(1)` is an error where it is written
([JavaScript interop](32-javascript-interop.md)). Importing reads a file
from a `FileUpload` with `await f.text()`, parses each line, and merges what
is new; then it tells every other open tab:

```wf
// src/pages/Settings.wf
        let added = Ledger.merge(parsed)
        news.post({ count: added })
```

`news` is `channel news = broadcast("pocket")`, a channel every tab of the
site hears; the app shell listens on it and shows a toast
([Realtime](18-realtime.md)). The ledger itself needs no channel —
`persist` already keeps every tab in step.

### Step 12: tests, and the checks

```wf
// tests/pocket.wf
test "an entry added through the form moves the balance" {
    use Ledger
    Text("Balance {format(Ledger.balance)}")
    EntryForm

    expect "Balance $16,533.56"
    click "Money in"
    type "250" into "Amount"
    click "Add entry"
    expect "Balance $16,783.56"
```

The test drives the real form by the labels a reader sees — the English
ones, since a test reads the project's default locale — in a headless
Chrome ([Testing](24-testing.md)). Pocket has eight: the cards and rows
render what they should, a blank amount is refused, `n` opens the dialog
and `Escape` closes it, removing an entry takes it out of every total.

```bash
wf test
wf check --deny-warnings
wf build
wf verify
```

`build/` is a single-page app with a service worker beside it: put it on a
host that serves `index.html` for every path ([Deploying](29-deploying.md)),
and the second visit works on a plane.

## Recipes

Smaller pieces every project meets, each a block you can drop into a page.

### Debounced search

```wf
page Search(path: "/") {
    state typed = ""
    state q = ""
    action commit(value: String) { if value == typed { q = value } }
    effect {
        let pending = typed
        setTimeout(() => commit(pending), 300)
    }
    resource hits = fetch("/api/search?q={q}")
    Input(bind: typed, placeholder: "Search", aria-label: "Search")
    match hits {
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} results for {q}") }
    }
}
```

The effect re-runs on every keystroke; each run schedules a check, and only
the one whose value is still current commits to `q`, which the resource
follows. (`after(ms)` is a declaration of a body, not a statement, so a
timer inside an effect is the browser's `setTimeout`.)

### Pagination

```wf
page Rows(path: "/") {
    state page = 1
    state perPage = 20
    resource rows = fetch("/api/rows?page={page}&per={perPage}")
    match rows {
        loading { Spinner }
        error(e) { Text(e.message).danger }
        ready(r) {
            for row in r.items by row.id { Text(row.name) }
            Row(gap: .sm, align: .center) {
                Button("Previous", disabled: page == 1) { on click { page = page - 1 } }
                Text("Page {page} of {r.pages}")
                Button("Next", disabled: page >= r.pages) { on click { page = page + 1 } }
            }
        }
    }
}
```

### Optimistic update with rollback

```wf
type Item { id: String, liked: Bool }

page Likes(path: "/") {
    state items: [Item] = [Item(id: "a", liked: false)]
    state error = ""
    action like(id: String) {
        let before = items
        items = items.map(i => if i.id == id { Item(id: i.id, liked: !i.liked) } else { i })
        try {
            await fetch("/api/like/{id}", { method: "POST" })
        } catch e {
            items = before
            error = "Could not save. Try again."
        }
    }
    for i in items by i.id {
        Button(if i.liked { "♥" } else { "♡" }, aria-label: "Like").sm { on click { like(i.id) } }
    }
    if error != "" { Toast(error).danger }
}
```

### A confirm dialog component

```wf
component Confirm(_ title: String, visible: Bool, danger: Bool = false) {
    event confirm()
    event cancel()
    slot
    Dialog(visible: visible, title: title) {
        children
        Row(gap: .sm, justify: .end) {
            Button("Cancel") { on click { emit cancel() } }
            Button("Confirm", tone: if danger { .danger } else { .primary }) { on click { emit confirm() } }
        }
    }
}

page Use(path: "/") {
    state asking = false
    state gone = false
    Button("Delete account").danger { on click { asking = true } }
    Confirm("Delete your account?", visible: asking, danger: true) {
        on confirm { gone = true  asking = false }
        on cancel { asking = false }
        Text("Everything goes. This cannot be undone.")
    }
    if gone { Alert("Account deleted.").info }
}
```

### Responsive navigation

```wf
page Nav(path: "/") {
    state open = false
    if viewport.md {
        Row(gap: .md) {
            Link("Home", to: "/")
            Link("Docs", to: "/docs")
            Link("Pricing", to: "/pricing")
        }
    } else {
        IconButton(icon: "menu", label: "Open navigation") { on click { open = !open } }
        show open {
            Stack(gap: .sm, animate: .slideDown) {
                Link("Home", to: "/")
                Link("Docs", to: "/docs")
                Link("Pricing", to: "/pricing")
            }
        }
    }
}
```

### A form with validation messages

```wf
page Signup(path: "/") {
    state email = ""
    state password = ""
    state confirm = ""
    state sent = false
    validate email { required  email "Enter a valid email" }
    validate password { required  minLength(8) "At least 8 characters" }
    validate confirm { matches(password) "The passwords differ" }
    Form(bind: form) {
        on submit { sent = true  form.reset() }
        Input(bind: email, name: "email", label: "Email").email
        Input(bind: password, name: "password", label: "Password").password
        Input(bind: confirm, name: "confirm", label: "Confirm password").password
        Button("Create account", type: .submit, disabled: !form.valid).primary
    }
    if sent { Alert("Welcome aboard.").success }
}
```

Each control shows its own message when the reader leaves it, and a submit
that fails focuses the first problem. [Forms](16-forms.md) has every rule.

### Tabs with the active tab in the URL

```wf
page Account(path: "/account") {
    derived tab = if hash != "" { hash } else { "profile" }
    Row(gap: .sm) {
        Link("Profile", to: "/account#profile")
        Link("Billing", to: "/account#billing")
    }
    if tab == "profile" { Text("Profile settings") }
    if tab == "billing" { Text("Billing settings") }
}
```

### Reading a store from a component without `use`

A component that only needs one value takes it as a prop, which keeps it
reusable outside that store:

```wf
store Cart { state count = 0 }

component CartBadge(count: Number) {
    if count > 0 { Badge("{count}").primary }
}

page Shop(path: "/") {
    use Cart
    CartBadge(count: Cart.count)
}
```

### Countdown

```wf
page Countdown(path: "/") {
    state left = 10
    every(1000) { if left > 0 { left = left - 1 } }
    Progress(value: 10 - left, max: 10)
    Text(if left == 0 { "Done" } else { "{left}s" })
}
```

### Signing in with a session cookie

```wf
type User { id: String, name: String }

api Auth(base: "/api") {
    credentials: .sameOrigin
    get me() -> User
    post signIn(body: Map) -> User
        errors { 401 -> Map }
    post signOut()
}

store Session {
    state user: User? = null
    state checked = false
    derived signedIn = user != null
    action load() {
        try { user = await Auth.me() } catch e { user = null }
        checked = true
    }
    action signIn(email: String, password: String) {
        user = await Auth.signIn(body: { email: email, password: password })
        navigate("/account")
    }
    action signOut() {
        await Auth.signOut()
        user = null
        navigate("/")
    }
}

page SignIn(path: "/sign-in", title: "Sign in", description: "Sign in to your account.", noindex: true) {
    state email = ""
    state password = ""
    state problem = ""
    validate email { required  email }
    validate password { required }
    action send() {
        problem = ""
        try { await Session.signIn(email, password) } catch e { problem = "That email and password do not match." }
    }
    Heading("Sign in").h1
    if problem != "" { Alert(problem).danger }
    Form(bind: form) {
        on submit { send() }
        Input(bind: email, label: "Email").email
        Input(bind: password, label: "Password").password
        Button("Sign in", type: .submit, disabled: send.pending).primary
    }
}

page Account(path: "/account", title: "Your account", description: "Your account.", guard: Session.signedIn, redirect: "/sign-in") {
    use Session
    Heading("Your account").h1
    if let u = Session.user { Text("Signed in as {u.name}") }
    Button("Sign out") { on click { Session.signOut() } }
}
```

The session is an httpOnly cookie the server sets; the page never holds a
token. `guard:` hides the page from the signed-out, and the server still
checks every request ([Security](23-security.md#sessions-and-tokens)).
Call `Session.load()` from the app's set-up so a returning reader is signed
in.

### Load more

```wf
type Item { id: String, name: String }

api Catalog(base: "/api") {
    get items(page: Number = 1) -> [Item]
}

page Items(path: "/items", title: "Items", description: "Everything, a page at a time.") {
    state page = 1
    resource list = Catalog.items(page: page, paginate: .page)
    Heading("Items").h1
    for item in list.items by item.id { Text(item.name) }
    if list.hasMore {
        Button("Load more", disabled: list.state == "loading") { on click { list.loadMore() } }
    }
}
```

### Filters in the URL

```wf
page Products(path: "/products", title: "Products", description: "Everything we sell.") {
    derived sort = query.sort ?? "name"
    Heading("Products").h1
    Row(gap: .sm) {
        Link("By name", to: "/products?sort=name")
        Link("By price", to: "/products?sort=price")
    }
    Text("Sorted by {sort}")
}
```

A filter in the query string survives a reload, a shared link and the back
button, where a `state` would not.

### A dark-mode switch

```wf
page Settings(path: "/settings", title: "Settings", description: "How the site looks.") {
    Heading("Settings").h1
    Row(gap: .sm) {
        Button("Light", outlined: theme != "light") { on click { setTheme("light") } }
        Button("Dark", outlined: theme != "dark") { on click { setTheme("dark") } }
        Button("Match the system", outlined: theme != "system") { on click { setTheme("system") } }
    }
}
```

It needs a dark theme named in the config ([Styling](15-styling.md#dark-mode)).

### Copy to the clipboard

```wf
page Share(path: "/share", title: "Share", description: "Copy the link.") {
    state copied = false
    action copy(text: String) {
        await navigator.clipboard.writeText(text)
        copied = true
    }
    Heading("Share").h1
    Button("Copy the link") { on click { copy(location.href) } }
    if copied { Toast("Copied").success }
}
```

### A chart

A charting library is a URL in `meta.scripts`, called from a script of your
own and handed a canvas with `Host`; the full example is in
[JavaScript interop](32-javascript-interop.md#a-library-from-a-cdn).

## Where to go next

- The [components reference](36-components-reference.md) for every prop
  and flag.
- `examples/` in the repository: the tutorials, reference PDFs and slide
  decks, and the `tests/fixtures/` projects — a gallery, a dashboard, a docs
  site, an offline site — each complete.
- `wf docs` in your own project, for what *your* components take.

## Next

[Built-in components — reference](36-components-reference.md).
