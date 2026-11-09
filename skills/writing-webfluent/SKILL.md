---
name: writing-webfluent
description: Write and change WebFluent (.wf, .wfx) projects with the wf toolchain - pages, components, stores, styling with design tokens, translations, tests, and fixing what the compiler reports. Use when a folder has webfluent.app.json or .wf files, or when asked to build a site, an app, a PDF or slides in WebFluent.
license: MPL-2.0
compatibility: Needs the wf command on PATH (install.sh or install.ps1 in the WebFluent repository).
metadata:
  author: Monzer Omer
  version: "5.12.0"
---

# Writing WebFluent

WebFluent is a language that compiles to HTML, CSS and JavaScript, and to PDF.
The compiler, `wf`, is also the best documentation you have: it lists every
component, prop, store member, token and diagnostic, for the version that's
installed. Ask it before you guess.

## Before you write

1. `wf --version`. The project's own `AGENTS.md` is the reference for the
   version that made it: read it, by section (`grep -n '^## ' AGENTS.md`, then
   the lines you need). With no `AGENTS.md`, use
   [references/language.md](references/language.md), the reference of
   WebFluent 5.12.0.
2. Read the file you're changing, and a neighbour of it. Write as the project
   does: its layout, its names, its words.
3. Ask the compiler for what you're about to use:

   | You need | Ask |
   |---|---|
   | What a component takes: props, flags, events, slots, parts | `wf components --json` |
   | What the project declares: enums, types, stores, pages | `wf types --json` |
   | What a name means at a place | `wf scope src/pages/Home.wf:12` |
   | The design tokens, and their contrast | `wf theme` |
   | Validation rules and patterns | `wf rules` |
   | What a locale is missing | `wf translations --missing ar` |
   | Every setting of `webfluent.app.json` | `wf config schema` |

   Never guess a prop or a store member. A member the store doesn't have is
   `T06`, and a prop the component doesn't declare is `C02`.

## The loop

1. Make the change.
2. `wf check`. Each finding has a code, and the message and the code are the
   compiler's own: never invent one. `wf explain T06` says what a code means and
   how to fix it. `wf fix --dry-run` shows the fixes the compiler can make, and
   `wf fix --code C02` makes one code's.
3. `wf fmt` settles the layout.
4. `wf test` runs the project's tests, and `wf build` compiles it.
5. `wf verify` loads every page in a headless Chrome and fails on a thrown
   error, a `console.error`, a missing file or a failed image. Run it before
   you say a page works.

Don't silence a finding by deleting what it's about. Fix it, or say why not.

## The grammar in brief

- A file declares `page`, `component` or `store`, each with a capital.
- Inside: `state` changes, `derived` is computed from other values, `action`
  does things, and `persist` is kept across visits.
- `use Cart`, then `Cart.member`. Only members the store declares exist.
- An element is capitalised. One positional argument comes first, then
  `name: value`, and flags come after the parentheses close, with a dot.
  Children go in a block, and handlers are `on click { … }`.
- `if … { } else { }`, `for item in list by item.id { }`, and `match` for an
  enum.
- Strings splice: `"{count} left"`. A translation is `t("key")`.
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

- **Flags and enum values take a dot.** `Button("Save").primary.lg` and
  `Row(gap: .md, justify: .between)`. Without the dot, as in `gap: md`, it's
  read as a name nothing declares: `T13`.
- **A flag goes after the parentheses close.** `Button("Save").primary` is
  right, and `Button.primary("Save")` isn't. `Card.Header` is a part, which is
  something else.
- **A flag is only what the component declares.** `Button` has `.outlined` and
  tones like `.danger`, and no `.ghost`; `Card` has `.flat`, `.elevated` and
  `.outlined`. A flag nothing declares is `E103`, so look it up in
  `wf components` first.
- **Tokens, not values.** In a `style { }` block, write `padding: $spacing-md`
  and `color: $color-text-muted`, not `16px` or a hex value. `wf theme` lists
  every token, and a hard-coded colour won't follow the dark theme.
- **A `style` block is CSS,** and it comes first in its block, before the
  children (`E002` otherwise). Its values aren't quoted, and two on one line are
  separated by `;`, never `,`. There's an example after this list.
- **Names the language keeps.** A project can't use `env`, `t`, `theme`,
  `query`, `hash`, `now`, `locale`, `dir`, a built-in function, or a word of the
  grammar (`state`, `on`, `by`, `show` …) as its own name: `E118`. `event`, `e`,
  `value` and `key` are fine.
- **Await what fetches.** An action that awaits the network is awaited where
  its result is read (`T17`).
- **`use` names a store exactly as it's declared,** with its capital (`X05`).
- **Hidden direction controls** in source are reported as `E119`. Don't paste
  them in from elsewhere.

A `style` block, first in its block, with tokens:

```wf
Card.flat {
    style { padding: $spacing-md; border-radius: $radius-md }
    Text("Rotate every 90 days.") {
        style { color: $color-text-muted }
    }
}
```

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

page Keys(path: "/keys", title: "Keys", description: "The keys and when they rotate.") {
    state open = true
    Heading("Keys").h1
    if open {
        Panel("Keys") {
            on closed { open = false }
            Text("Rotate every 90 days.")
        }
    }
}
```

## Accessible and translated from the start

- Every field has a label: `label:` on an `Input`, or `aria-label:` when the
  design shows none. Never only a placeholder.
- Status is never colour alone: say it in words, as a `Badge` does.
- Words the reader sees go through `t("key")` once the project has
  `i18n.locales`, with every key in every locale's file (`I03` is a key one
  locale lacks). Right to left follows the locale. Use `start` and `end`, not
  `left` and `right`.

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
keep them.

## Where things go

`src/pages/` has one page per file, alongside `src/components/`,
`src/stores/` and `src/translations/<locale>.json`. `public/` is copied to the
root of the build. `build/` is output: never edit it. Plain browser scripts
may sit under `src/` when a page needs one (the reference's "JavaScript
interop"), but reach for one only when the language has no way.
`wf generate page Name` scaffolds a file in the project's layout.

## The reference

[references/language.md](references/language.md) is the whole language. Its
"For agents" section has ten idioms and the mistakes models make most, and
"Compiler diagnostics" lists every code. Read a section when you need it, not
the whole file.
