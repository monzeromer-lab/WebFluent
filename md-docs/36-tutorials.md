# 36. Tutorials

<!--
route: tutorials
group: shipping
blurb: Three projects built step by step — a todo app, a statically built notes site, a budget tracker — none needing a server.
description: Three WebFluent tutorials, from beginner to advanced — Todos, Field Notes and Pocket — each a complete project with no backend, built step by step.
-->

These three tutorials each build a whole project, step by step, and each
project is complete in the repository under `examples/tutorials/`. None
needs a server: what a project keeps, it keeps in the reader's browser.
Each tutorial stands on its own, so pick the one that matches where you
are, or do them in order.

| Level | Project | What you build | What it teaches | Time |
|---|---|---|---|---|
| **Easy** | [Todos](#easy-todos) | A list of things to do: add, tick, rename, filter | Types, a store with `persist`, a component, a keyed list, a filter in the address, tests that click | 20 minutes |
| **Medium** | [Field Notes](#medium-field-notes) | A notebook site, statically built, a page per note | `data` files, a layout, `:slug` pages with `paths:`, Markdown pages, search and tags, what a pre-rendered page can know | An hour |
| **Hard** | [Pocket](#hard-pocket) | A budget tracker with a dashboard, forms and two languages | `Money` and `Date`, derived totals, a `persist` migration, `validate`, a dialog and shortcuts, i18n and right-to-left text, a script of your own, offline | A couple of hours |

If you have not written WebFluent before, start with
[the Shelf tutorial](03-tutorial.md): it is a gentler first half hour.

## Before you start

You need:

- `wf` installed and on your `PATH` — `wf --version` prints `wf 5.11.0` or
  later ([Getting started](02-getting-started.md)).
- Chrome or Chromium on the machine, for the tests that click and for
  `wf verify`. Set `WF_CHROME` to its path when it is not in a usual place.
- Git, to fetch the finished projects.

Every code block whose first line names a file — `// src/stores/Todos.wf`
— is that file, or a run of lines from it, exactly as the project has it.
A test holds this chapter to the projects, so the code you read here is
the code that builds.

To start from a finished project instead, clone the repository and serve
one:

```bash
git clone https://github.com/monzeromer-lab/WebFluent
cd WebFluent/examples/tutorials/easy-todos
wf serve
```

**Check it:** the terminal prints `Starting dev server at
http://localhost:3000/`, and that address shows the project.

## Easy: Todos

**What you will build.** A list of things to do, kept in the reader's
browser: add a todo, tick it, rename it, remove it, and show only the open
ones. It is one page, one store and one component. The finished project is
[`examples/tutorials/easy-todos`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/easy-todos).

**Time:** about 20 minutes.

**Before you start.** You need only `wf` and Chrome. You write every file
of this project yourself, and the blocks below hold each file whole, except
the tests, where one of six is shown.

### Step 1: Create the project

Make a single-page project and go into it:

```bash
wf init todos -t spa
cd todos
```

Delete everything the template put in `src/` and in `tests/`: you write
every file yourself, and the template's tests would fail without its pages.
Then replace `webfluent.app.json` with this:

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

- `"output_type": "spa"` makes a single-page app: one HTML shell, with
  pages swapped without a reload
  ([Static sites and SPAs](26-static-and-spa.md)).
- `"csp": true` ships a content security policy and holds the build to it
  ([Security](23-security.md)).
- `theme` names two themes, `Paper` and `Night`, which you write in the
  next step.

**Check it:** `wf check` prints `IO Error: No .wf files found in src/`.
That is expected: `src/` is empty until the next step.

### Step 2: Give it a look of its own

The config names a theme, so declare it first. A theme names only the
design tokens it changes; every other token keeps its default
([Styling](15-styling.md)). `Night` is the theme named as `"dark"` in the
config, and the page uses it when the reader's system is set to dark:

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

/// The same page after dark.
theme Night {
    color-primary: #2DD4BF
    color-background: #14171A
    color-surface: #1C2024
    color-text: #ECEEF0
    color-text-muted: #A3ABB3
    color-border: #2C3238
}
```

The page you write in step 6 names classes — `card`, `todo`, `is-done`,
`filters` — and `src/todos.css` styles them. The build bundles every `.css`
file under `src/`, so the file needs no link. These are four of its rules;
copy the whole file from the
[finished project](https://github.com/monzeromer-lab/WebFluent/blob/master/examples/tutorials/easy-todos/src/todos.css):

```css
.todo { flex-wrap: nowrap; padding: 0.6rem 0.25rem; border-top: 1px solid var(--color-border); }
.todo__title { flex: 1; }
.todo.is-done .todo__title { text-decoration: line-through; color: var(--color-text-muted); }
.filters a.is-on { background: var(--color-background); color: var(--color-text); font-weight: 600; }
```

Each colour is a token, `var(--color-border)`, so the dark theme recolours
the page with no rule of its own.

**Check it:** `wf check` prints `No problems in todos.`

### Step 3: Describe a todo

A `type` names the fields a value has, and the checker holds every use to
them ([Types](13-types.md)). An `enum` is a fixed set of cases:

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

- `Uuid` is one of the types the language brings; `uuid()` makes one.
- `done` has a default, so a new todo can leave it out.
- A case is written with a dot, `.open`, wherever a `Filter` is wanted.

Because the checker knows the fields, `todo.titel` is an error where it is
written, and the message names the field you meant.

**Check it:** `wf check` prints `No problems in todos.`

### Step 4: Write a store that remembers

A store holds state that any page or component can read
([Stores](12-stores.md)). This one keeps the list:

```wf
// src/stores/Todos.wf
/// The list, kept in the browser between visits.
store Todos {
    persist items: [Todo] = []

    derived open = items.filter(todo => !todo.done)
    derived done = items.filter(todo => todo.done)

    action add(title: String) {
        let text = title.trim()
        if text == "" { return }
        items = items.concat([Todo(id: uuid(), title: text)])
    }

    action toggle(id: Uuid) {
        items = items.map(todo => if todo.id == id { { ...todo, done: !todo.done } } else { todo })
    }

    action rename(id: Uuid, title: String) {
        let text = title.trim()
        if text == "" { return }
        items = items.map(todo => if todo.id == id { { ...todo, title: text } } else { todo })
    }

    action remove(id: Uuid) {
        items = items.filter(todo => todo.id != id)
    }

    action clearDone() {
        items = items.filter(todo => !todo.done)
    }
}
```

- `persist` is a `state` kept in the browser's storage. The list is still
  there after a reload, and a second tab follows the first, with no storage
  code written.
- A `derived` value follows what it reads: `open` is always the todos not
  yet done.
- An action is the one place state changes. `{ ...todo, done: !todo.done }`
  is the todo with one field changed.
- `rename`, `remove` and `clearDone` change the list in the same way.

**Check it:** `wf check` prints seven `U04` warnings, such as ``warning[U04]:
`Todos.open` is declared but never read``. That is expected: nothing reads
the store yet. The page in step 6 reads every member, and the warnings go.

### Step 5: Build one row

A component is an element of your own ([Components](11-components.md)).
This one draws a single todo, and lets the reader rename it in place:

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

- `_ todo` is the component's one positional prop, so a call reads
  `TodoRow(todo)`.
- Its `state` belongs to each row: renaming one row does not put the others
  in edit mode.
- `class:` adds classes to the element's own. The list form may hold an
  `if`, which adds `is-done` only while the todo is done.
- Every control has an accessible name. The checkbox and the icon buttons
  say which todo they act on, so a screen reader — and the tests in step 7
  — can tell them apart ([Accessibility](22-accessibility.md)).

**Check it:** `wf check` prints `U03` for `TodoRow` (nothing places it yet)
and four `U04` warnings. The store's `toggle`, `rename` and `remove` no
longer appear: the row calls them.

### Step 6: Write the page

The page holds the box for a new todo, the filters and the list. Here is
its first half:

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

- `bind: draft` ties the input to the state both ways: typing changes
  `draft`, and `draft = ""` empties the box.
- `query` is the address's query string as a value, kept current. So
  `filter` follows the address, and `shown` follows `filter`.
- A `match` over an enum must cover every case. If you add a fourth case to
  `Filter`, this `match` is an error until it handles it.

The second half draws the filters and the list, and closes the page. Add it
below the first, after a blank line:

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
    Text("Your list stays in this browser, and nowhere else.", class: "hint")
}
```

- `for … by todo.id` is a keyed list. When a todo is renamed, or the filter
  hides it, only its row changes; the other rows keep their elements, and
  the row being edited keeps its cursor ([Control flow](10-control-flow.md)).
- `animate:` and `exit:` fade a row in and out.
- A class map, `{ "is-on": filter == .all }`, turns a class on while its
  condition holds.

Last, the `app` declaration is the frame every page sits in. `Router`
marks where the current page is drawn:

```wf
// src/App.wf
// The frame every page sits in: a narrow column, centred.
app {
    Container(class: "shell") {
        Router
    }
}
```

Now start the development server in a second terminal, and leave it
running. Every save rebuilds the project and reloads the page:

```bash
wf serve
```

**Check it:** `wf check` prints `No problems in todos.`, and
<http://localhost:3000/> shows a card headed **Todos** with `0 left` and
*Nothing yet — add your first todo above.* Type a todo and press Enter: it
appears, and the count reads `1 left`. Reload the page, and the todo is
still there. Click **Open**, and the address becomes `/?show=open`.

If the server prints `No theme Paper is declared in this project`, you
probably skipped `src/theme.wf` in step 2. If a store member shows as
`U04` here, you probably left out part of the page.

### Step 7: Write tests

A test draws what it declares, then follows its steps in order. It types
into the control named **New todo** and clicks the button named **Add**,
the way a reader would ([Testing](24-testing.md)). Create
`tests/todos.wf` with this test:

```wf
// tests/todos.wf
test "adding a todo puts it in the list" {
    use Todos
    state draft = ""
    Input(bind: draft, aria-label: "New todo")
    Button("Add") { on click { Todos.add(draft)  draft = "" } }
    for todo in Todos.items by todo.id { TodoRow(todo) }
    Text("{Todos.open.length} left")

    expect "0 left"
    type "Buy bread" into "New todo"
    click "Add"
    expect "Buy bread"
    expect "1 left"
}
```

A test that clicks or types runs in a headless Chrome. The finished
project has six tests in
[`tests/todos.wf`](https://github.com/monzeromer-lab/WebFluent/blob/master/examples/tutorials/easy-todos/tests/todos.wf):
two that only look at a row, and four that click — adding a todo, refusing
a blank one, ticking and clearing, and renaming that keeps one row for
each. Copy the file to have them all, then run them:

```bash
wf test
```

**Check it:** with the whole file, `wf test` ends with `6 passed, 0
failed, 2 snapshot(s) written`. The two tests that only look write a
snapshot of what they drew under `tests/__snapshots__/`, and later runs
compare against it.

If a test that clicks reports that no browser was found, Chrome is not
where `wf` looks: set `WF_CHROME` to its path.

### Step 8: Build it

Check, test and build the project:

```bash
wf check
wf test
wf build
```

`wf check` reports every finding and writes nothing. `wf build` writes
`build/`, which any static host serves — [Deploying](29-deploying.md)
covers GitHub Pages, Netlify and the rest. To load the built page in a
real browser before you deploy, run `wf verify`: it fails on an error, a
missing file or a page that draws nothing.

**Check it:** `wf build` ends with `Build complete.`, and `build/` holds
`index.html`, `app.js` and `styles.css`.

### What you built in Todos

- A `type` and an `enum`, held to every use ([Types](13-types.md)).
- A store whose list survives a reload with `persist`, and `derived`
  values that follow it ([Stores](12-stores.md)).
- A component with state of its own, and accessible names on every
  control ([Components](11-components.md)).
- A keyed list, and a filter kept in the address with `query`
  ([Control flow](10-control-flow.md)).
- A theme with a dark variant, and the project's own stylesheet
  ([Styling](15-styling.md)).
- Tests that look and tests that click ([Testing](24-testing.md)).

## Medium: Field Notes

**What you will build.** A notebook kept in public — notes from a garden, a
kitchen and a few trips — built as a static site. The notes are a JSON file
the build reads. Every page, including one page per note, is written out
as HTML that a search engine can read. The one thing a reader changes, the
notes they save for later, stays in their browser. The finished project is
[`examples/tutorials/medium-field-notes`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/medium-field-notes).

**Time:** about an hour.

**Before you start.** You need `wf`, Chrome and a copy of the repository
(see [Before you start](#before-you-start) above). This tutorial shows
the part of each file that teaches something new, so work in the finished
project and read each file as its step comes up:

```bash
cd WebFluent/examples/tutorials/medium-field-notes
wf serve
```

The Todos tutorial explains state, stores and components; this one
assumes them.

### Step 1: Create a static project and its look

A new project of this kind starts from the `static` template:

```bash
wf init field-notes -t static
cd field-notes
```

The project's `webfluent.app.json`:

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

- `"ssg": true` pre-renders every page to its own `index.html`. When the
  page loads, the script takes it over in place — *hydration*
  ([Static sites and SPAs](26-static-and-spa.md)).
- `site_url` lets the build write what search engines and link previews
  read: a canonical address, Open Graph tags, JSON-LD, `sitemap.xml` and
  `robots.txt` ([Content](27-content.md)).

The dark theme names only the tokens that change:

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
of its own, such as `paper-raised` and `tag-bg`. The project's stylesheet,
`src/styles.css`, reads every colour as a token (`var(--tag-bg)`), so the
dark theme needs no rule of its own ([Design tokens](42-design-tokens.md)).

**Check it:** with `wf serve` running, set your system to dark mode. The
page turns dark green and cream without a reload.

### Step 2: Type the data

The notes are data, and a `type` says what each one holds:

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

- `data` reads a JSON file at build time. The result is a constant, typed
  by the annotation, inlined into the bundle and seeded into every
  pre-rendered page ([Data](17-data.md)).
- The checker holds the program to `Note`: a page that reads `note.titel`
  is an error where it is written.
- `date` is a `Date`: a plain `"2026-03-08"` string the language knows how
  to format. `{note.date:.date(long)}` shows `March 8, 2026`.

`src/notes.json` holds seven notes, each with a Markdown `body`.

**Check it:** `wf check` prints `No problems in field-notes.` If one note
in `notes.json` has a field that `Note` does not declare, the check reports
`T05` at the `data` line and names the field; if a note leaves out a field
`Note` requires, it reports `C01`.

### Step 3: Write the shell and a layout

The `app` declaration wraps every page. The current page is drawn where
`Router` stands, inside `<main>`, so the skip link each page carries jumps
past the navigation ([Pages and routing](06-pages-and-routing.md)). The
start of `src/App.wf`:

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

- The link whose `to:` is the page shown is marked `aria-current="page"`.
- `ThemeToggle` is two icon buttons that call `setTheme("dark")` and
  `setTheme("light")`. The reader's choice is kept across visits.
- `Router(transition: .fade, duration: "160ms")` fades one page out and
  the next in.

Each page names the column it is set in with `layout:` — a component whose
default slot is the page:

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

**Check it:** the navbar shows **Notes**, **Tags**, **Saved** and
**About**. Click **Tags**: the page fades over, and the **Tags** link is
marked as the current page.

### Step 4: Build a card

A card shows a note at a glance:

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

- A `///` comment above a component or a prop is its documentation. Your
  editor shows it when you hover a call ([Components](11-components.md)).
- A splice may say how to show its value: `{note.date:.date(long)}` is
  `format(note.date, .date, "long")`, written where it is read
  ([Expressions](14-expressions.md)).
- `TagList` is a row of `#tag` links to `/?tag=…`.

**Check it:** the front page shows a card per note, each with a line such
as `March 8, 2026 · 4 min read`, a title, a summary and its tags.

### Step 5: Search and filter the front page

The front page lists every note, narrowed by a search box and by the tag in
the address. `NoteFinder` works the list out:

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

`q` is bound to the search box, and everything else follows it. `sortBy`,
`flatMap` and `unique` are list helpers the language brings. Each also runs
at build time, so the pre-rendered page shows the sorted list
([State and reactivity](08-state-and-reactivity.md)).

A static site adds one idea here. Its HTML is written once, before any
reader arrives, so the HTML cannot depend on what only the browser knows,
such as the address's query or what is in storage. So the finder draws
every note and hides the ones that do not match:

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

The HTML then holds every note, for a reader without script and for a
search engine, and the filter applies the moment the page is live. The
front page hands the finder the address's tag:

```wf
// src/pages/Home.wf
        // `query.tag` is the URL's `?tag=…`; it is missing when the URL has
        // none, and `?? ""` turns that into the finder's "every note".
        NoteFinder(notes, tag: query.tag ?? "")
```

**Check it:** the front page reads `7 of 7 notes`. Type `tomato` in the
search box, and the count and the cards narrow as you type. Open
`/?tag=garden`, and only the notes tagged `garden` show. Run `wf build` and
open `build/index.html` in a text editor: all seven notes are in it.

If a filtered list is empty in the built HTML, you probably filtered the
`for` itself by `query`; hide the cards instead, as above.

### Step 6: Give each note a page

Each note gets a page at `/notes/<slug>`:

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

- `:slug` in the path is a typed parameter of the page.
- `paths:` names the values a static build renders the page for: one
  `notes/<slug>/index.html` per note, each listed in the sitemap
  ([Content](27-content.md)).
- `title:` and `description:` may be worked out from the parameters, so
  each note's search result shows its own title and summary.

The body of the page is the note's Markdown:

```wf
// src/pages/NotePage.wf
            Markdown(n.body)
```

`Markdown` renders headings, lists, quotes, emphasis and links, at build
time and live. It escapes the text first, so HTML in a note is shown, not
run ([Elements](07-elements.md)). Below the note, `related` lists the three
newest notes that share a tag with it — a `derived` value like any other.

**Check it:** click a note's title. Its page shows the whole note, and the
browser tab shows its title followed by `— Field Notes`. After `wf build`,
`build/notes/` holds seven folders, one per note.

### Step 7: List the tags, and write a page in Markdown

The Tags page groups the notes under each tag:

```wf
// src/pages/Tags.wf
        Grid(columns: { base: 1, md: 2 }, gap: .md, align: .start) {
            for tag in allTags by tag {
                derived filed = notes.filter(n => n.tags.includes(tag)).sortBy(n => n.date).reverse()
```

- A layout prop takes one value per breakpoint: one column on a phone, two
  from `md` up. It compiles to a media query, so the first paint is already
  right ([Styling](15-styling.md)).
- A `derived` value inside a `for` is worked out for each item.

The About page is not a `.wf` file at all. `src/about.md` is a page: its
front matter names its path, title, description and layout, and the rest
is its Markdown:

```markdown
---
path: /about
title: About — Field Notes
description: Who keeps this notebook, why it exists, and how the site is built.
layout: Sheet
---
# About
```

**Check it:** `/tags` shows one card per tag, in one column on a narrow
window and two on a wide one. `/about` shows the About page in the same
column as the others.

### Step 8: Keep the reader's saved notes

A store keeps the notes this reader saves:

```wf
// src/stores/Saved.wf
/// The notes this reader has saved for later, kept in their browser.
store Saved {
    persist slugs: [String] = []

    derived count = slugs.length
```

`persist` keeps the list in the browser across visits. There is no
account, and nothing is sent anywhere ([Stores](12-stores.md)).

The button that saves a note says whether it is pressed, the way assistive
technology reads it:

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
button with `[aria-pressed="true"]`, so the look and the meaning are one
attribute ([Accessibility](22-accessibility.md)). The Saved page lists
`notes.filter(n => Saved.slugs.includes(n.slug))`, and the navbar's badge
shows `Saved.count`.

**Check it:** click **Save** on a card. It reads **Saved**, and the navbar
shows a `1` badge beside **Saved**. Reload the page: the note is still
saved, and `/saved` lists it.

### Step 9: Test it, and run the checks a static site needs

A test reads the project's `data` file, as any page does
([Testing](24-testing.md)):

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

The project has six tests. Run them, check with warnings counted as
failures, build, and load the result in a browser:

```bash
wf test
wf check --deny-warnings
wf build
wf verify
```

`wf verify` loads every route of the built site in a headless Chrome —
each note's page at its `paths:` value — and fails on an error, a missing
file or a page that draws nothing ([Command line](38-cli.md)).

**Check it:** `wf test` reports `6 passed`, `wf build` reports `SSG:
pre-rendered static pages`, and `wf verify` ends with `0 problem(s)`.
`build/` is ready for any static host.

### What you built in Field Notes

- A static build, with canonical addresses, a sitemap and link previews
  ([Content](27-content.md)).
- Typed data read from a JSON file at build time ([Data](17-data.md)).
- A shell, and a layout every page names
  ([Pages and routing](06-pages-and-routing.md)).
- A page per note from one `:slug` page and `paths:`, and a page written in
  Markdown ([Content](27-content.md)).
- A search and a filter that work in pre-rendered HTML
  ([Static sites and SPAs](26-static-and-spa.md)).
- A reader's saved list, with a pressed state assistive technology reads
  ([Accessibility](22-accessibility.md)).

## Hard: Pocket

**What you will build.** A personal budget tracker that lives entirely in
the reader's browser: money in and money out, a budget per category, what
is left of it this month, six months of bars, two languages, a CSV export
and import, and a page that keeps working offline. There is no server and
no account. The finished project is
[`examples/tutorials/hard-pocket`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/hard-pocket).

**Time:** a couple of hours.

**Before you start.** You need `wf`, Chrome and a copy of the repository
(see [Before you start](#before-you-start) above). This tutorial assumes
the Todos tutorial, and each step links the chapter that goes deeper. Like
Field Notes, it shows the part of each file that teaches something new, so
work in the finished project:

```bash
cd WebFluent/examples/tutorials/hard-pocket
wf serve
```

### Step 1: Configure the project

The project's `webfluent.app.json`:

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

It adds three things to the Todos config:

- `i18n` names the languages and the folder their messages live in
  ([Internationalization](21-i18n.md)).
- `offline` makes `wf build` write a service worker that stores the four
  pages, so a second visit opens with no network ([Offline](19-offline.md)).
- `theme` names two themes, light and dark, and the build ships both.

**Check it:** `wf build` prints a line that starts `Offline: sw.js`, then
`2 locales` in its summary.

### Step 2: Describe money

The ledger's types use four of the types the language brings:

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

`Money`, `Date`, `Color` and `Uuid` are each a plain JSON value at run time
([Types](13-types.md)). Money is kept in minor units, `{ amount: 1299,
currency: "USD" }`, so a sum never drifts by a fraction of a cent. You
write it `$12.99` and show it with `format(m)`. What the language works out
is a call — `a.plus(b)`, `a.minus(b)`, `ONE.times(12.5)` — and what the
value carries is a field: `m.amount`. Two constants give every sum a
starting point:

```wf
// src/types.wf
/// Nothing yet: the starting point every sum adds to.
const ZERO: Money = $0.00

/// One dollar — `ONE.times(12.5)` is $12.50, rounded to the cent.
const ONE: Money = $1.00
```

**Check it:** `wf check` prints `No problems in pocket.` Change `$0.00` to
`$0.001`, and the check refuses it: money has two decimals in dollars.
Change it back.

### Step 3: Add up the ledger

Every figure on every page is a `derived` value of the `Ledger` store:

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

Add an entry, and the balance, this month's spending, each budget bar and
the chart all follow, with no code written to keep them in step
([Stores](12-stores.md)). `now` is the browser's clock as a value, so
`today` turns over at midnight. `sum` is an action that only returns a
value, which a derived value may call:

```wf
// src/stores/Ledger.wf
    // The money in a list of entries, added up.
    action sum(list: [Entry]) {
        return list.reduce((total, e) => total.plus(e.amount), ZERO)
    }
```

**Check it:** the dashboard shows a balance, this month's income and
spending, and what is left of the budget. Each is a sum of the sample
ledger.

### Step 4: Change the stored shape without losing data

What a `persist` keeps outlives the build that wrote it: a reader who
visited last month has last month's shape in their browser. The entries
declare which version of the shape this build writes, and how to bring an
older one forward:

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

- `version: 2` says which shape this build writes.
- `migrate 1 -> 2` brings an older value forward. Its body receives the
  stored value as `old`.
- If you raise the version with no migration, the build warns (`P01`). If
  you change the shape without raising the version, it warns too (`D04`),
  from the shapes it keeps in `.wf-cache/` ([Stores](12-stores.md)).
- A first visit opens on a sample ledger dated from today, so the
  dashboard is never empty.

**Check it:** `wf check --deny-warnings` passes. Delete the `migrate`
block and check again: the check reports `P01`. Put the block back.

### Step 5: Fit the shell to the screen

The shell draws a `Sidebar` when `viewport.lg` holds, and a top bar with a
drawer when it does not. This is the drawer:

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

- `viewport` is the window's size as a value. It follows `matchMedia`, so
  the switch happens once, as the window crosses the breakpoint
  ([Styling](15-styling.md)).
- The layout itself — grids that are one column on a phone — uses
  responsive values, `Grid(columns: { base: 1, lg: 2 })`, which need no
  script.
- A second store, `Ui`, holds what the interface is doing — the dialog,
  the menu, the notices — apart from what the ledger holds.

**Check it:** on a wide window, the sidebar shows. Narrow the window: a top
bar replaces it, and its menu button opens the drawer, which a click on the
dimmed page behind it closes.

### Step 6: Draw a dashboard from elements

The built-in `Chart` draws at build time, from data the build can see. This
ledger exists only in the reader's browser, so the chart is made of plain
elements whose heights follow the store:

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

- A `{…}` in a style value reads state, and the property follows it
  ([Styling](15-styling.md)).
- The chart is one `role="img"` with a label, so a screen reader hears what
  it shows rather than twelve unnamed boxes
  ([Accessibility](22-accessibility.md)).

The figures above the chart are one component, placed four times:

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

`lean` is an enum prop. A component writes its enum props to its root
element, here as `data-lean="good"`, and `pocket.css` colours the card by
that attribute ([Components](11-components.md)).

**Check it:** the dashboard shows four cards and six pairs of bars. Add an
expense in step 9's dialog, and this month's bar and the **Spent** card
grow at once.

### Step 7: Keep the filters in the address

The Entries page reads its filters from the address:

```wf
// src/pages/Entries.wf
    // The filters live in the address — `/entries?month=2026-09&category=dining` —
    // so they survive a reload, a shared link and the back button.
    derived month = query.month ?? ""
    derived category = query.category ?? ""
    derived shown = Ledger.newestFirst.filter(e => (month == "" || e.date.slice(0, 7) == month) && (category == "" || e.category == category))
```

Choosing a month calls `navigate("/entries?month=…")`. The address
changes, `query` follows it, and so does the list.

An entry also has a page of its own at `/entries/:id`, where `id` is a
typed parameter. The page looks the entry up with
`Ledger.entries.find(…)` and draws it inside `if let e = entry`. `if let`
binds the value only when it is there, which the checker requires before
the page reads its fields ([Pages and routing](06-pages-and-routing.md)).

**Check it:** on `/entries`, pick a month. The address gains `?month=…`
and the list narrows. Press the browser's back button: the filter goes.

### Step 8: Use one form for new and for edit

`EntryForm` adds an entry or, given one, changes it:

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

- A `validate` block sits beside the state it guards. The control bound to
  that state shows what the rules say — the message, `aria-invalid` and the
  link between them — with nothing more written ([Forms](16-forms.md)).
- A rule with no message of its own shows the translation of
  `form.<rule>`, so `en.json` and `ar.json` each say "that date has not
  happened yet" once.
- The form says it is done with an event, `saved`. The caller decides what
  that means: close the dialog, or go back to the entry.

The submit button follows the form:

```wf
// src/components/EntryForm.wf
        Row(justify: .end, gap: .sm) {
            Button(if entry != null { t("action.save") } else { t("action.add") }, type: .submit, disabled: !form.valid).primary
        }
```

`Form(bind: form)` hands the component a handle on the form. `form.valid`
is whether every rule holds, and the button follows it as the reader
types.

**Check it:** in the new-entry dialog, type `0` as the amount and leave the
field: it says *Enter an amount above zero*, and **Add entry** stays
disabled. Type `12.50`, and the button turns on.

### Step 9: Add a dialog and a shortcut

The app listens for two keys at the top of `src/App.wf`:

```wf
// src/App.wf
    // `n` opens a new entry from anywhere. A key with no modifier is left to
    // the field the reader is typing in, so an `n` in a note is just a letter.
    on key("n") { Ui.compose() }
    on key("Escape") { Ui.closeAll() }
```

`on key` at the top of the app listens on the document for as long as the
page shows ([Events](09-events.md)). The dialog is a `Modal` whose `title:`
is its accessible name. It keeps focus inside while it is open, and Escape
closes it:

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

**Check it:** press `n` anywhere outside a field, and the new-entry dialog
opens with focus inside it. Press Escape, and it closes. Add an entry, and
the dialog closes and a notice says it was added.

### Step 10: Speak two languages

Every word on screen is `t("key")`, read from `src/translations/en.json`
or `ar.json`. A message with plural forms is several keys:

```json
{
  "entries.zero": "No entries",
  "entries.one": "{count} entry",
  "entries.other": "{count} entries"
}
```

- `t("entries", { count: n })` picks the form the language's rules call
  for. English needs two forms and a zero; Arabic has six
  ([Internationalization](21-i18n.md)).
- Switching to Arabic sets `dir="rtl"` on the page, and the sidebar, the
  rows and the grids mirror themselves.
- `format(m)` and dates follow the language too.
- The reader's choice is a `persist` in a small `Prefs` store, and an
  `effect` in the app shell applies it on every visit.

**Check it:** on **Settings**, choose Arabic. The page reads right to
left, and every label is in Arabic. Reload: it stays in Arabic.

If a key shows on screen instead of its words, it is missing from a
translation file; `wf check` reports it as `I01`, and `wf translations`
lists what each locale lacks.

### Step 11: Export and import the data

The Settings page writes the ledger out as a CSV file:

```wf
// src/pages/Settings.wf
    action exportCsv() {
        let lines = Ledger.newestFirst.map(e => csvLine(e))
        saveTextFile("pocket-{Ledger.today}.csv", [CSV_HEADER, ...lines].join("\n"), "text/csv")
    }
```

`saveTextFile` is not WebFluent. It is a function in `src/files.js`, a
plain browser script that the build links on every page. Its JSDoc types
the call, so `saveTextFile(1)` is an error where it is written
([JavaScript interop](32-javascript-interop.md)).

Importing reads a file from a `FileUpload` with `await f.text()`, parses
each line and merges what is new. Then it tells every other open tab:

```wf
// src/pages/Settings.wf
        let added = Ledger.merge(parsed)
        news.post({ count: added })
```

`news` is `channel news = broadcast("pocket")`, a channel every tab of the
site hears. The app shell listens on it and shows a notice
([Realtime](18-realtime.md)). The ledger itself needs no channel: `persist`
already keeps every tab in step.

**Check it:** on **Settings**, export the ledger, and the browser saves
`pocket-<today's date>.csv`. Open a second tab, import the file in the
first: the second tab shows a notice of how many entries were added.

### Step 12: Test it, and run the checks

A test drives the real form by the labels a reader sees — the English
ones, since a test reads the project's default locale
([Testing](24-testing.md)):

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

Pocket has eight tests: the cards and rows draw what they should, a blank
amount is refused, `n` opens the dialog and Escape closes it, and removing
an entry takes it out of every total. Run them and the checks:

```bash
wf test
wf check --deny-warnings
wf build
wf verify
```

**Check it:** `wf test` reports `8 passed`, and `wf verify` ends with `0
problem(s)`. `build/` is a single-page app with a service worker, `sw.js`,
beside it. Put it on a host that answers every path with `index.html`
([Deploying](29-deploying.md)), and the second visit works with no network.

### What you built in Pocket

- A ledger typed with `Money`, `Date`, `Color` and `Uuid`
  ([Types](13-types.md)).
- Totals that are all `derived` values of one store, and a `persist`
  migration that keeps returning readers' data ([Stores](12-stores.md)).
- A shell that changes with the window, and responsive grids
  ([Styling](15-styling.md)).
- A form that validates itself, a dialog and keyboard shortcuts
  ([Forms](16-forms.md), [Events](09-events.md)).
- Two languages, plural forms and right-to-left layout
  ([Internationalization](21-i18n.md)).
- A script of your own, and a channel between tabs
  ([JavaScript interop](32-javascript-interop.md),
  [Realtime](18-realtime.md)).
- A site that works offline ([Offline](19-offline.md)).

## Recipes

These are smaller pieces that most projects need. Each is a whole page you
can build and adapt.

### Debounce a search

To wait until the reader stops typing before you search, keep what they
typed apart from what you search for:

```wf
page Search(path: "/", title: "Search", description: "Search as you type, without a request per key.") {
    state typed = ""
    state q = ""
    action commit(value: String) { if value == typed { q = value } }
    effect {
        let pending = typed
        setTimeout(() => commit(pending), 300)
    }
    resource hits = fetch("/api/search?q={q}")
    Heading("Search").h1
    Input(bind: typed, placeholder: "Search", aria-label: "Search")
    match hits {
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} results for {q}") }
    }
}
```

The effect runs again on every keystroke, and each run schedules a check.
Only the check whose value is still current writes `q`, and the resource
follows `q`. The timer is the browser's `setTimeout`, because `after(ms)`
is a declaration with a body, not a statement an effect can hold.

### Page through results

To show a long list a page at a time, keep the page number in state and
splice it into the address:

```wf
page Rows(path: "/", title: "Rows", description: "Every row, twenty at a time.") {
    state page = 1
    state perPage = 20
    resource rows = fetch("/api/rows?page={page}&per={perPage}")
    Heading("Rows").h1
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

A change of `page` changes the address, so the resource asks again.

### Update at once, and roll back on failure

To show a change before the server agrees, keep the old value and put it
back if the request fails:

```wf
type Item { id: String, liked: Bool }

page Likes(path: "/", title: "Likes", description: "A like that shows at once.") {
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
    Heading("Likes").h1
    for i in items by i.id {
        Button(if i.liked { "♥" } else { "♡" }, aria-label: "Like").sm { on click { like(i.id) } }
    }
    if error != "" { Toast(error).danger }
}
```

`optimistic(holder, change)` does the same with less code inside an
action; see [Data](17-data.md#show-a-change-before-the-server-agrees).

### Build a confirm dialog component

To ask before a destructive action, wrap a `Dialog` in a component that
fires `confirm` or `cancel`:

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

page Account(path: "/", title: "Account", description: "Close your account.") {
    state asking = false
    state gone = false
    Heading("Account").h1
    Button("Delete account").danger { on click { asking = true } }
    Confirm("Delete your account?", visible: asking, danger: true) {
        on confirm { gone = true  asking = false }
        on cancel { asking = false }
        Text("Everything goes. This cannot be undone.")
    }
    if gone { Alert("Account deleted.").info }
}
```

The caller's block fills the default slot, so the question's detail is the
caller's to write.

### Make navigation responsive

To show links on a wide screen and a menu button on a narrow one, branch on
`viewport`:

```wf
page Nav(path: "/", title: "Home", description: "Navigation that fits the screen.") {
    state open = false
    Heading("Home").h1
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

### Show validation messages on a form

To check a form as the reader fills it in, put a `validate` block beside
each state:

```wf
page Signup(path: "/", title: "Sign up", description: "Make an account.") {
    state email = ""
    state password = ""
    state confirm = ""
    state sent = false
    validate email { required  email "Enter a valid email" }
    validate password { required  minLength(8) "At least 8 characters" }
    validate confirm { matches(password) "The passwords differ" }
    Heading("Sign up").h1
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
that fails moves focus to the first problem. [Forms](16-forms.md) lists
every rule.

### Keep the active tab in the address

To make a tab survive a reload, read it from `hash`:

```wf
page Account(path: "/account", title: "Account", description: "Your profile and billing.") {
    derived tab = if hash != "" { hash } else { "profile" }
    Heading("Account").h1
    Row(gap: .sm) {
        Link("Profile", to: "/account#profile")
        Link("Billing", to: "/account#billing")
    }
    if tab == "profile" { Text("Profile settings") }
    if tab == "billing" { Text("Billing settings") }
}
```

### Pass a store's value to a component as a prop

A component that needs only one value from a store can take it as a prop
instead of using the store. That keeps the component reusable outside that
store:

```wf
store Cart { state count = 0 }

component CartBadge(count: Number) {
    if count > 0 { Badge("{count}").primary }
}

page Shop(path: "/", title: "Shop", description: "Everything we sell.") {
    use Cart
    Heading("Shop").h1
    CartBadge(count: Cart.count)
}
```

### Count down

To count down once a second, change a state in an `every` timer:

```wf
page Countdown(path: "/", title: "Countdown", description: "Ten seconds, counted down.") {
    state left = 10
    every(1000) { if left > 0 { left = left - 1 } }
    Heading("Countdown").h1
    Progress(value: 10 - left, max: 10)
    Text(if left == 0 { "Done" } else { "{left}s" })
}
```

The timer stops when the page leaves.

### Sign in with a session cookie

To sign a reader in without holding a token in the page, describe the
service with `credentials: .sameOrigin` and keep the user in a store:

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

The session is an httpOnly cookie the server sets, so the page never holds
a token. `guard:` hides the page from a reader who is signed out, and the
server still checks every request
([Security](23-security.md#keep-the-session-in-a-cookie)). Call `Session.load()`
from the app's set-up, so a returning reader is signed in.

### Load more

To add the next page of results below the ones shown, call an endpoint with
`paginate: .page`:

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

`list.items` is every item loaded so far, and `list.loadMore()` asks for the
next page.

### Keep filters in the address

To make a filter survive a reload, a shared link and the back button, read
it from `query` instead of holding it in a `state`:

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

### Add a dark-mode switch

To let the reader pick light, dark or the system's choice, call `setTheme`
and read `theme`:

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

The switch needs a dark theme named in the config
([Styling](15-styling.md#dark-mode)).

### Copy to the clipboard

To copy text, call the browser's clipboard from an action:

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

### Draw a chart from a library

To use a charting library, list its URL in `meta.scripts`, call it from a
script of your own, and hand it a canvas with `Host`. The full example is
in [JavaScript interop](32-javascript-interop.md#load-a-library-from-a-cdn).

## Where to go next

- The [components reference](37-components-reference.md), for every prop
  and flag.
- `examples/` in the repository: the tutorials, reference PDFs and slide
  decks, and the `tests/fixtures/` projects — a gallery, a dashboard, a
  docs site, an offline site — each complete.
- `wf docs` in your own project, for what *your* components take.

## Next

[Built-in components — reference](37-components-reference.md).
