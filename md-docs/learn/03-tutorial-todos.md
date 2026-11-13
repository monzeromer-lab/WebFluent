---
chapter: 3
kind: "tutorial"
route: "/learn/03-tutorial-todos"
from: ["/docs/tutorials/todos"]
group: "tour"
blurb: "Build a todo list kept in the reader's browser — types, a store that remembers, a component, a filter in the address and tests that click."
description: "A beginner WebFluent tutorial: a todo app with a persisted store, a component, a keyed list, a filter in the address and tests, in 20 minutes."
---
# 3. Tutorial: Todos

This is the first of three tutorials that each build a whole project, step by step. Each project is complete in the repository under `examples/tutorials/`, and none needs a server: what a project keeps, it keeps in the reader's browser.

| Tutorial | What you build | What it teaches | Time |
|---|---|---|---|
| [Todos](03-tutorial-todos.md) | A list of things to do: add, tick, rename, filter | Types, a store with `persist`, a component, a keyed list, a filter in the address, tests that click | 20 minutes |
| [Field Notes](../recipes/build-a-notebook-site.md) | A notebook site, statically built, a page per note | `data` files, a layout, `:slug` pages with `paths:`, Markdown pages, search and tags, what a pre-rendered page can know | An hour |
| [Pocket](../recipes/build-a-budget-app.md) | A budget tracker with a dashboard, forms and two languages | `Money` and `Date`, derived totals, a `persist` migration, `validate`, a dialog and shortcuts, i18n and right-to-left text, a script of your own, offline | A couple of hours |

If you have not written WebFluent before, start with [the Shelf tutorial](02-tutorial-shelf.md): it is a gentler first half hour.

**What you will build.** A list of things to do, kept in the reader's
browser: add a todo, tick it, rename it, remove it, and show only the open
ones. It is one page, one store and one component. The finished project is
[`examples/tutorials/easy-todos`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/easy-todos).

**Time:** about 20 minutes.

## Before you start

You need:

- `wf` installed and on your `PATH` — `wf --version` prints `wf 5.11.0` or
  later ([Getting started](01-getting-started.md)).
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

For this tutorial you need only `wf` and Chrome. You write every file
of this project yourself, and the blocks below hold each file whole, except
the tests, where one of six is shown.

## Step 1: Create the project

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
  ([Static sites and SPAs](../about/static-and-spa.md)).
- `"csp": true` ships a content security policy and holds the build to it
  ([Security](../about/security.md)).
- `theme` names two themes, `Paper` and `Night`, which you write in the
  next step.

**Check it:** `wf check` prints `IO Error: No .wf files found in src/`.
That is expected: `src/` is empty until the next step.

## Step 2: Give it a look of its own

The config names a theme, so declare it first. A theme names only the
design tokens it changes; every other token keeps its default
([Styling](13-styling.md)). `Night` is the theme named as `"dark"` in the
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

## Step 3: Describe a todo

A `type` names the fields a value has, and the checker holds every use to
them ([Types](12-types.md)). An `enum` is a fixed set of cases:

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

## Step 4: Write a store that remembers

A store holds state that any page or component can read
([Stores](11-stores.md)). This one keeps the list:

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

## Step 5: Build one row

A component is an element of your own ([Components](10-components.md)).
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
  — can tell them apart ([Accessibility](../about/accessibility.md)).

**Check it:** `wf check` prints `U03` for `TodoRow` (nothing places it yet)
and four `U04` warnings. The store's `toggle`, `rename` and `remove` no
longer appear: the row calls them.

## Step 6: Write the page

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
  the row being edited keeps its cursor ([Control flow](09-control-flow.md)).
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

## Step 7: Write tests

A test draws what it declares, then follows its steps in order. It types
into the control named **New todo** and clicks the button named **Add**,
the way a reader would ([Testing](16-testing.md)). Create
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

## Step 8: Build it

Check, test and build the project:

```bash
wf check
wf test
wf build
```

`wf check` reports every finding and writes nothing. `wf build` writes
`build/`, which any static host serves — [Deploying](../recipes/index.md)
covers GitHub Pages, Netlify and the rest. To load the built page in a
real browser before you deploy, run `wf verify`: it fails on an error, a
missing file or a page that draws nothing.

**Check it:** `wf build` ends with `Build complete.`, and `build/` holds
`index.html`, `app.js` and `styles.css`.

## What you built

- A `type` and an `enum`, held to every use ([Types](12-types.md)).
- A store whose list survives a reload with `persist`, and `derived`
  values that follow it ([Stores](11-stores.md)).
- A component with state of its own, and accessible names on every
  control ([Components](10-components.md)).
- A keyed list, and a filter kept in the address with `query`
  ([Control flow](09-control-flow.md)).
- A theme with a dark variant, and the project's own stylesheet
  ([Styling](13-styling.md)).
- Tests that look and tests that click ([Testing](16-testing.md)).

## Next

[Tutorial: Field Notes](../recipes/build-a-notebook-site.md) builds a statically generated site on what this one taught.
