---
name: writing-webfluent
description: Write, review and fix WebFluent (.wf, .wfx) projects with the wf toolchain - pages, components, stores, types, forms, data from APIs, styling with design tokens and themes, translations and right-to-left, tests, PDF documents and slide decks, and desktop and mobile apps (Android, iOS) with Tauri. Use when a folder has webfluent.app.json or .wf files, when asked to build a site, an app, a dashboard, a PDF or slides in WebFluent, or to wrap a WebFluent project in Tauri.
license: MPL-2.0
compatibility: Needs the wf command on PATH (install.sh or install.ps1 in the WebFluent repository). Tests that click, and wf verify, need Chrome or Chromium.
metadata:
  author: Monzer Omer
  version: "6.1.0"
---

# Writing WebFluent

WebFluent is a language that compiles to HTML, CSS and JavaScript, and to PDF.
The compiler, `wf`, is also the best documentation you have. It lists every
component, prop, store member, token and diagnostic for the version that's
installed. Ask it before you guess, and believe it over memory.

## Pick the reference for the task

Read this file, then only the reference the task needs:

| The task | Read |
|---|---|
| Any change: how good WebFluent is written, and the review checklist | [references/code-quality.md](references/code-quality.md) |
| Building a feature: lists, forms, a service, routes, dialogs, settings, dashboards, shortcuts, drag, long lists | [references/recipes.md](references/recipes.md) |
| Colours, themes, dark mode, `style { }`, CSS files, responsive layout, RTL, motion | [references/styling.md](references/styling.md) |
| Writing tests, `wf verify`, CI | [references/testing.md](references/testing.md) |
| A finding from `wf check` you don't understand | [references/diagnostics.md](references/diagnostics.md) |
| A PDF, a slide deck, or rendering a template on a server | [references/documents.md](references/documents.md) |
| An app for Linux, macOS, Windows, Android or iOS with Tauri (or Electron): the bridge to Rust, window chrome, safe areas and the back gesture, packaging and signing | [references/tauri-apps.md](references/tauri-apps.md) |
| The exact grammar, a component's every prop, a config key, a code's meaning | [references/language.md](references/language.md), the whole reference: read it by section |

## Before you write

1. **Check the version.** Run `wf --version`.
   - A project's own `AGENTS.md` is the reference for the version that made
     it. Read it by section (`grep -n '^## ' AGENTS.md`, then the lines you
     need).
   - With no `AGENTS.md`, use [references/language.md](references/language.md),
     the reference of WebFluent 6.1.0.
   - If the compiler and the reference disagree, the compiler is right. Say so.
2. **Read the file you're changing**, and a neighbour of it. Write as the
   project does: its layout, its names, its words.
3. **Ask the compiler** for what you're about to use:

   | You need | Ask |
   |---|---|
   | What a component takes: props, flags, events, slots, parts | `wf components --json` |
   | Every built-in, with its cases and the icon list | `wf registry --json` |
   | What the project declares: enums, types, stores, pages | `wf types --json` |
   | What a name means at a place | `wf scope src/pages/Home.wf:12` |
   | The design tokens, and their contrast | `wf theme` |
   | The animations and eases | `wf motion` |
   | Validation rules and patterns | `wf rules` |
   | What a locale is missing | `wf translations --missing ar` |
   | Every setting of `webfluent.app.json` | `wf config schema` |
   | What `public/` holds and what uses it | `wf assets` |
   | Everything at once, for an agent | `wf context --json` |

   Never guess a prop or a store member. A member the store doesn't have is
   `T06`, and a prop the component doesn't declare is `C02`.

## The loop

1. **Make the change**, the smallest that does the job.
2. **`wf check`.**
   - Each finding has a code, and the message and the code are the compiler's
     own; never invent one.
   - `wf explain T06` says what a code means and how to fix it.
   - `wf fix --dry-run` shows the fixes the compiler can make, and
     `wf fix --code C02` applies one code's.
   - Warnings are work too.
3. **`wf fmt`** settles the layout.
4. **`wf test`** runs the project's tests. Add one for new behaviour.
5. **`wf build`** compiles it. Use `wf build --stats` when weight matters.
6. **`wf verify`** loads every page in a headless Chrome. It fails on a thrown
   error, a `console.error`, a missing file or a failed image. Run it before
   you say a page works.

`wf serve` is the dev server. It rebuilds on save and shows a failed build's
error over the page.

Don't silence a finding by deleting what it's about. Fix it, or say why not.

## The grammar in brief

- A file declares `page`, `component`, `store`, `type`, `enum`, `api` or
  `theme`, each named with a capital. `app { … Router }` is the shell.
- Inside:
  - `state` is a value that changes;
  - `derived` is worked out from other values;
  - `action` does things;
  - `persist` is kept across visits;
  - `effect` reaches the world outside.
- `use Cart`, then `Cart.member`. Only members the store declares exist.
- An element is capitalised. One positional argument comes first, then
  `name: value`, and flags come after the parentheses close, with a dot.
  Children go in a block, and handlers are `on click { … }`.
- `if … { } else { }`, `if let x = maybe { }`, `for item in list by item.id { }`,
  and `match` for an enum or a resource.
- Strings splice: `"{count} left"`, `"{total:.currency}"`. A translation is
  `t("key")`.
- Commas separate arguments. New lines separate statements.

```wf
store Deploys {
    state items: [Map] = []
    state search = ""
    derived shown = items.filter(d => d.service.includes(search))

    action add(service: String) {
        items = items.concat([{ id: service, service: service, status: "queued" }])
    }
}

page DeployList(path: "/deploys", title: "Deploys", description: "Every deploy.") {
    use Deploys
    Heading("Deploys").h1
    Input(bind: Deploys.search, placeholder: "Search by service", aria-label: "Search")
    Button("Add a deploy").primary { on click { Deploys.add("new-service") } }
    if Deploys.shown.length == 0 {
        Text("No deploys yet.")
    } else {
        for d in Deploys.shown by d.id {
            Row(gap: .sm, align: .center) {
                Text(d.service)
                Badge(d.status)
            }
        }
    }
}
```

## What catches models out

- **Flags and enum values take a dot.** Write `Button("Save").primary.lg` and
  `Row(gap: .md, justify: .between)`. Without the dot, as in `gap: md`, the
  value is read as a name nothing declares: `T13`.
- **A flag goes after the parentheses close.** `Button("Save").primary` is
  right, and `Button.primary("Save")` isn't. `Card.Header` is a part, which is
  something else.
- **A flag is only what the component declares.**
  - `Button` has `.outlined` and tones like `.danger`, but no `.ghost`.
  - `Card` has `.flat`, `.elevated` and `.outlined`.
  - A flag nothing declares is `E103`, so look it up in `wf components` first.
- **Names the language keeps.** A project can't use any of these as its own
  name (`E118`):
  - `t`, `env`, `theme`, `query`, `hash`, `now`, `locale`, `dir`, `params`;
  - a built-in function;
  - a word of the grammar (`state`, `on`, `by`, `show` …).

  So `for t in todos` is an error: write `for todo in todos`. `event`, `e`,
  `value` and `key` are fine.
- **`let` every local in an action.** An assignment to a name the language
  owns, such as `dir = -1`, isn't reported and overwrites the page's
  direction.
- **`match` chooses by enum case,** never by string. Map a string to an enum
  in a `derived` with `if … else if …`.
- **Never `xs[0]` or `filter(…)[0]`.** Write `xs.find(…)`, `xs.first() ?? x`,
  or `if let first = xs.first() { … }` (`T04`).
- **Tokens, not values.** In a `style { }` block, write `padding: $md` and
  `color: $text-muted`, not `16px` or a hex value. `wf theme` lists every
  token, and a hard-coded colour won't follow the dark theme.
- **A `style` block is CSS,** and it comes first in its block, before the
  children (`E002` otherwise). Its values aren't quoted, and two on one line
  are separated by `;`, never `,`.
- **Await what fetches.** An action that awaits the network is awaited where
  its result is read (`T17`).
- **`use` names a store exactly as it's declared,** with its capital (`X05`).
- **A `DatePicker` binds a `Date?`**, not a string (`F02`).
- **Hidden direction controls** in source are reported as `E119`. Don't paste
  them in from elsewhere.

A `style` block, first in its block, with tokens:

```wf
page Keys(path: "/keys", title: "Keys", description: "When the keys rotate.") {
    Heading("Keys").h1
    Card.flat {
        style { padding: $spacing-md; border-radius: $radius-md }
        Text("Rotate every 90 days.") {
            style { color: $color-text-muted }
        }
    }
}
```

## Quality in one screen

The full version, with the review checklist, is
[references/code-quality.md](references/code-quality.md).

- **Types first.** A `type` for every record and an `enum` for every closed
  set of words. Use `Money`, `Date`, `Uuid` and `Email` over strings, and
  avoid `Map` and `Any` once you know the shape.
- **Variants are enum props.** A component writes them on its root as
  `data-<prop>`, so CSS styles each case and the component holds no colour
  logic.
- **A value down, an event up.** Components take props and `emit` events;
  they don't reach into their caller's state.
- **`derived`, not a state kept in sync by an `effect`.** A filter lives in
  the address (`query.show`).
- **Actions replace values**: `items = [...items, x]`, `map`, `filter`.
  Return early on a blank input.
- **A service is an `api` block**, and a `resource` over it matches
  `loading`, `error` and `ready`, the empty list too.
- **Every list from data is keyed** with `by`.
- **Every control has a visible label.** An action button names what it acts
  on (`label: "Remove {todo.title}"`). One `h1` per page. Status is never
  colour alone.
- **Translated projects.** Every word a reader sees goes through `t("key")` or
  literals mode, with logical CSS sides (`margin-inline-start`) for right to
  left.
- **`///` comments say why.**

## Components that place what they're given

```wf
component Panel(_ title: String) {
    event closed()
    Card {
        Card.Header {
            Heading(title).h2
            Button("Close").outlined.sm { on click { emit closed() } }
        }
        Card.Body { children }
    }
}

page Vault(path: "/vault", title: "Vault", description: "The keys and when they rotate.") {
    state open = true
    Heading("Vault").h1
    if open {
        Panel("Keys") {
            on closed { open = false }
            Text("Rotate every 90 days.")
        }
    }
}
```

## Tests

Tests live under `tests/` and say what the page shows, not how it's built. A
test that clicks or types runs in Chrome:

```wf
component Counter {
    state count = 0
    Button("Add one") { on click { count = count + 1 } }
    Text("{count} added")
}

test "it starts at none" {
    Counter
    expect "0 added"
    expect not "1 added"
}

test "a click adds one" {
    Counter
    click "Add one"
    expect "1 added"
}
```

`wf test --update` writes the snapshots for new tests. Read them before you
keep them. See [references/testing.md](references/testing.md).

## Where things go

- **Folders.** `src/pages/` has one page per file, alongside `src/components/`,
  `src/stores/`, `src/types.wf`, `src/theme.wf` and
  `src/translations/<locale>.json`. Feature folders are fine once folders
  grow.
- **CSS and scripts.** `.css` files anywhere under `src/` are bundled. Plain
  browser scripts under `src/` are linked on every page. Reach for a script
  only when the language has no way, and type it with JSDoc.
- **Output.** `public/` is copied to the root of the build. `build/` is
  output: never edit it.
- **Scaffolding.** `wf generate page Name` scaffolds a file in the project's
  layout.

## Other shapes of project

- **A PDF or a deck:** `wf init report -t pdf` or `wf init deck -t slides`.
  See [references/documents.md](references/documents.md).
- **A desktop or phone app** (Linux, macOS, Windows, Android, iOS): the same
  project inside Tauri, with one typed script, `src/native.js`, between the
  pages and Rust, and a shell that draws a title bar on a desktop and keeps
  clear of the bars on a phone. See
  [references/tauri-apps.md](references/tauri-apps.md).
- **A translated site:** `wf init site --locales en,ar`.
- **Custom elements for another framework:** `"output_type": "elements"`.
