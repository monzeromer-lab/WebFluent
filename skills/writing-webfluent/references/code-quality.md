# Code quality in WebFluent

These are the conventions the canonical projects follow:
- the `wf init` starters;
- the `easy-todos`, `medium-field-notes` and `hard-pocket` tutorials;
- the documentation site, which is written in WebFluent;
- WebFluent Studio, a 116-file app.

The anti-patterns come from what those projects had to fix. Code that
follows this file passes `wf check` with no warnings, and stays easy to
change.

The rule under all of it: **say what a thing is, and let the compiler hold
you to it.**
- A `type` instead of a `Map`.
- An `enum` instead of a string.
- A `derived` instead of a copy kept in sync by hand.
- A label instead of a placeholder.

## Contents

- [Project layout and names](#project-layout-and-names)
- [Types first](#types-first)
- [Components](#components)
- [State: state, derived, stores](#state-state-derived-stores)
- [Actions](#actions)
- [Data from a server](#data-from-a-server)
- [Accessibility](#accessibility)
- [Translations](#translations)
- [Performance](#performance)
- [Security](#security)
- [Comments](#comments)
- [Review checklist](#review-checklist)

## Project layout and names

```
src/
├── App.wf               # app { … Router }: the chrome every page shares
├── theme.wf             # theme Day { … }  theme Night { … }
├── types.wf             # the records and enums several files share
├── pages/Home.wf        # one page per file, named for the page
├── components/          # one component per file, or a few per feature file
├── stores/              # one store per area of the app
├── styles.css           # global rules; feature sheets sit beside their files
└── translations/en.json
tests/                   # `test "…" { }`, one file per feature
public/                  # copied to the build's root
```

- **Declarations are PascalCase**: `page Home`, `component TaskRow`,
  `store Cart`, `type Invoice`, `enum Tone`. State, derived values, actions,
  props and locals are camelCase. A `const` is `UPPER_SNAKE`.
- **A file is named for what it declares**: `pages/Home.wf`,
  `components/TaskRow.wf`. As a folder grows, group by feature
  (`src/billing/Invoice.wf`, `src/billing/invoice.css`, `src/billing/Billing.wf`)
  rather than by kind.
- **Name a page after what it shows.** Give it a suffix only when a component
  already has the name (`page NotePage` beside `component NoteCard`).
- **Name things by what they hold, in words.**
  - Not `t`, `d`, `x`, `data`, `item2`.
  - The language reserves `t`, `env`, `query`, `hash`, `now`, `theme`, `dir`,
    `locale`, `params`, `viewport`, `network`, `update` and its built-in
    functions as names (`E118`), so a single-letter loop item is often an
    error anyway: `for todo in todos`, `filter(row => …)`.
- **`.wf` with four-space indentation**, formatted by `wf fmt`. Use `.wfx`
  only if the project already does.
- **`.gitignore`**: `build/`, `.wf-cache/`, `.wf-sizes.json`, `.env`.
- **Never edit `build/`.** It's output.

## Types first

Declare the shape of every record the app moves around, and an enum for
every closed set of words. The checker then catches a misspelled field
(`T05`), a missing case (`T15`), a value that may be null (`T04`) and a wrong
argument (`T01`). Any of those would otherwise reach a reader as `undefined`.

```wf
/// How urgent an issue is. A flag on `LevelBadge`, and `data-level` for CSS.
enum Level { low, high, critical }

/// One issue in the tracker.
type Issue {
    id: Uuid
    title: String
    level: Level = .low
    opened: Date
    closed: Bool = false
    note: String? = null
}
```

- **Prefer the built-in types**:
  - `Uuid`, `Date`, `DateTime`, `Duration`, `Email` and `Url`;
  - `Money` (minor units, never a float of dollars);
  - `Color`, and `Secret` for anything that must never be shown or kept.
- **Refine where a value has limits**: `nights: Number(1..=30)`,
  `pass: String(minLength: 8)`. The condition is checked on literals at build
  time and validates input at run time with no `validate` written.
- **`Map` and `Any` say nothing.** Use them only at a boundary you haven't
  typed yet, and narrow them as soon as you know the shape. (Studio's 63 `Any`s
  in one store are debt, not a model.)
- **Types several files read** go in `src/types.wf`, or in the feature's own
  file.

## Components

Make a component for anything drawn more than once, or that has state of its
own: a row, a card, a badge, a form section. A one-off block stays inline in
its page.

```wf
/// A titled box. `trailing` fills the end of the heading row;
/// `Panel.Note` is a footnote placed in its block.
component Panel(_ title: String) {
    slot trailing
    part Note(_ text: String) { Text(text, class: "panel__note").muted.sm }
    Card(class: "panel") {
        Row(justify: .between, align: .center) {
            Heading(title).h2
            trailing
        }
        children
    }
}
```

**Props.**
- **One obvious argument.** If a component has one, it's positional and
  marked `_`, and it comes first: `TaskRow(_ task: Task)`,
  `Panel(_ title: String)`. Everything else is named.
- **Variants are enums, not strings.** `tone: Tone = .calm` gives the caller
  a flag (`.loud`), lets the checker refuse a typo, and writes
  `data-tone="loud"` on the root, so CSS styles each case without a single
  `if` in the component:

  ```css
  .level-badge[data-level="critical"] { background: var(--color-danger); }
  ```

  Don't use a `tone: String` with a chain of `if tone == "danger" { … }` to
  pick colours. That is the most common smell in older WebFluent code.
- **A Bool prop is a flag**: `UserCard("Sam").active`.
- **Defaults make the common call short.** A prop with a sensible default
  should have one.
- **Records go in whole**: `IssueRow(issue)`, not
  `IssueRow(title: issue.title, level: issue.level, …)`.

**A value down, an event up.** A component never changes its caller's state
directly and takes no `bind:`. It declares an `event` and `emit`s it, and the
caller decides what happens.

```wf
/// One issue in a list. The row never changes the store itself.
component IssueRow(_ issue: Issue) {
    /// The reader asked to close this issue.
    event close(id: Uuid)
    derived label = match issue.level { .low { "Low" } .high { "High" } .critical { "Critical" } }

    Row(class: ["issue-row", if issue.closed { "is-closed" }], align: .center, gap: .sm) {
        Badge(label, class: "level-badge", data-level: "{issue.level}").pill
        Text(issue.title, class: "issue-row__title")
        if !issue.closed {
            Button("Close", aria-label: "Close {issue.title}").sm { on click { emit close(issue.id) } }
        }
    }
}
```

**Slots and parts.**
- `children` places the caller's block.
- `slot name` declares a named place, and a scoped slot hands the fill
  values (`slot row(item: Todo, index: Number)`).
- `part Header { … }` gives the component sub-pieces called as
  `Panel.Header`.
- A layout for pages is a component with a default slot:
  `page X(…, layout: Shell)`.

**Match an enum exhaustively.** A `match` expression that covers every case
needs no `else`. Leave the `else` out, and adding a case becomes a `T15` at
every place that must handle it.

**Keep the root single.** `drag:`, `on longpress`, attributes and `on click`
written on a component's call attach to its root element. A body that is an
`if`, a `for` or several siblings has no single root (`C07`). Wrap it in a
`Stack` or `Row`.

**Don't shadow the built-ins.** A component called `Footer`, `Chart` or
`Card` is never drawn, because the built-in wins (`V11`). Prefix it
(`SiteFooter`).

## State: state, derived, stores

| It is… | Write |
|---|---|
| a value the reader or an action changes, used by one component | `state` in that component |
| used by one page and its children | `state` in the page, passed down as props |
| read by several pages | a `store` |
| for one screen only, dropped when it leaves | `store X(scope: .route)` |
| worked out from other values | `derived`: never a `state` kept in sync by an `effect` |
| kept across visits | `persist`, with a `version:` once its shape changes |
| shareable in a link | in the address: `query.show`, `hash` |
| a server's data | a `resource` over an `api` endpoint |

```wf
/// The tracker's issues, kept in this browser between visits.
store Issues {
    persist items: [Issue] = [] {
        version: 2
        // Version 1 kept `urgent: Bool`; version 2 keeps a `Level`.
        migrate 1 -> 2 {
            old.map(i => Issue(id: i.id, title: i.title, opened: i.opened, level: if i.urgent { .high } else { .low }))
        }
    }

    derived open = items.filter(i => !i.closed)
    derived critical = open.filter(i => i.level == .critical).length

    action report(title: String, level: Level) {
        let text = title.trim()
        if text == "" { return }
        items = [...items, Issue(id: uuid(), title: text, level: level, opened: now.date())]
    }

    action close(id: Uuid) {
        items = items.map(i => if i.id == id { { ...i, closed: true } } else { i })
    }
}
```

**Stores.**
- One store per area of the app (`Cart`, `Session`, `Layout`), with its
  domain data and its UI state in separate stores.
- A store is built the first time something reads it, so the order of
  declaration doesn't matter.
- **Read only what the store declares.** `Cart.total` when the store says
  `sum` is `T06`. Look it up with `wf types --json`, never guess.
- **Every member is used** (`U04`), and every action is called (`U05`).
  Delete what nothing reads, or prefix it with `_` if that is on purpose.

**`derived`, not `effect`.**
- An `effect` that writes a value from other values is a `derived` written
  the long way, and one that writes what it reads never settles (`X02`).
- Keep `effect` for the world outside: the title, a library, `setTheme`,
  `setLocale`, a listener with its `cleanup`.

**State that reads the browser.** `state clock = now(every: 1.seconds)`
holds the first value forever (`X06`). Anything that reads `now`, `viewport`,
`query`, `hash`, `theme` or `network` is a `derived`.

**A filter belongs in the address.** `derived showAll = query.show == "all"`
survives a reload and a shared link. On a static site, the pre-render can't
know the query, so draw every item and hide the ones that don't match.

## Actions

- **`let` every local.** A local assigned without `let` is a store or page
  member, or an error (`T13`).
  - **One silent case:** an assignment to a name the language owns (`dir`,
    `locale`) is not reported. It overwrites the document's direction.
  - Always write `let dir = if asc { 1 } else { -1 }`.
- **Replace, don't mutate.**
  - `items = [...items, x]`, `items.map(i => if i.id == id { { ...i, done: true } } else { i })`,
    `items.filter(i => i.id != id)`.
  - The compiler already turns `items[i].done = v` and `items.push(x)` into a
    copy, but the replacing form reads the same everywhere and suits every
    holder.
- **Return early on bad input**: `let text = title.trim()` then
  `if text == "" { return }`.
- **`await` what fetches**, and wrap it in `try { } catch e { failure = e.message }`.
  An async action's result read without `await` is `T17`. `save.pending` is
  true while a call of `save` runs, so use it for a spinner or a disabled
  button instead of a hand-kept flag.
- **Reading the first item.**
  - Never `filter(…)[0]` or `xs[0]` (`T04`: it may be nothing).
  - Use `xs.find(…)`, `xs.first() ?? fallback`, or `if let first = xs.first() { … }`.
  - A derived `hasItems = xs.length > 0` doesn't narrow anything for the
    checker. `if let`, `?.` and `??` do.
- **Show a change before the server agrees.** `optimistic(holder, change)`
  rolls itself back if the action throws.

## Data from a server

- **Describe the service once** in an `api` block (base, timeout, retry,
  credentials, endpoints with types), or read it from an OpenAPI file with
  `api Backend from "openapi.json" (base: …)`.
  - Every call is then typed, cached, deduplicated and cancellable.
  - Raw `fetch` in a page is for the odd one-off.
- **A `resource` is matched with every arm**: `loading`, `error(e)` (`T21` if
  it's missing) and `ready(x)`.
- **Inside `ready`:**
  - show `x.refreshing` as a small spinner, not a full reload;
  - show `x.error` (a failed refresh) as a stale-data note;
  - handle the empty list.
- **Error kinds.** Branch on them (`.offline`, `.timeout`, `.status(code, body)`)
  rather than showing `e.message` for everything. Give a "Try again"
  (`x.reload()`).
- **Key every list** from the server by its id (`for u in users by u.id`), so
  a refresh redraws only what changed.
- **Sessions** belong in an httpOnly cookie (`credentials: .sameOrigin`),
  never in `persist`.

See [recipes.md](recipes.md) for a complete service, resource and page.

## Accessibility

The compiler checks most of this (the `A` family), so a warning-free build is
most of the way there.

- **One `h1` per page**, with headings in order (`A11`, `A12`). Give every
  page a `title:` and a `description:` (`S01`, `S02`).
- **A name for every control.**
  - `label:` on `Input`, `Select`, `Textarea`, `Checkbox`, `Switch`, `Slider`.
  - `aria-label:` only when the design truly shows no label. Never a
    placeholder alone (`A03`).
- **An `IconButton` is named by its `label:`** (`A02`). Name a row's action
  after the row, `label: "Remove {todo.title}"`: a screen reader hears which
  row, and a test can click it by name.
- **`aria-label` contains the visible text** (`A15`): a button that shows
  "Save" may be labelled "Save draft", not "Store".
- **Status is never colour alone.** A `Badge` says the word, and an enum
  drives both the word and the colour.
- **Toggles say their state**: `aria-pressed: active`, `aria-expanded: open`.
  Style them with `&[aria-pressed="true"]`, so the look follows what
  assistive technology reads.
- **Images.** Every `Image` has `alt` (`A01`), and a decorative one has
  `alt: ""`. A `Table` has a `Table.Head` (`A10`) and a `caption:`. A `Modal`
  has a `title:` (`A08`). A `Video` has `captions:` (`A09`).
- **Use built-ins over `Host`.** `Menu`, `Popover`, `Modal`, `Tabs` and
  `Tooltip` bring focus handling, Escape and roles; a hand-made overlay must
  re-create all of it. `trapFocus: open` keeps Tab inside anything else that
  opens.

## Translations

- **Choose a mode at the start.**
  - Keyed: `t("nav.home")` with every key in every `translations/<locale>.json`.
  - Literals: `"i18n": { "literals": true }`. The default locale's text is
    its own key, and other locales translate it.
  - `wf init --locales en,ar` sets up keyed mode.
- **Find the gaps.** `wf translations --missing ar` lists what a locale
  lacks (`I03`), and `wf translations --unused` lists what nothing names.
- **Plurals** are separate keys: `items.one`, `items.other`, plus `zero`,
  `two`, `few` and `many` for Arabic. Pass `{ count: n }` and the form is
  picked for you. Don't build plurals with `if n == 1`.
- **Numbers and dates** go through `format(…)`, `{x:.currency}` and `ago(…)`,
  which speak the locale. Never concatenate them by hand.
- **Right to left.**
  - RTL follows the locale. Write `margin-inline-start`, `padding-inline`,
    `border-inline-end` and `inset-inline-start`, never `left`/`right`, in
    style blocks and CSS.
  - Flip direction icons under `[dir="rtl"]`.
  - Give a code or figure inside Arabic text its own LTR run
    (`.ltr { direction: ltr; unicode-bidi: isolate; }`).

## Performance

- **Key every list that changes**: `for x in xs by x.id`. Without `by`, items
  holding state restart when the list changes (`U09`).
- **Long lists.** A list of hundreds goes in a `VirtualList` whose body is one
  keyed `for` (`V12` otherwise), with a height its rows don't set.
- **Responsive values for layout**, not `viewport`: use
  `Grid(columns: { base: 1, md: 3 })` and `.stacks`. They compile to CSS and
  are right in the first paint. Keep `viewport` for behaviour (a drawer
  instead of a sidebar).
- **Images.** Declare them with `image hero = "hero.jpg"`, or reference a
  `public/` file literally, and the build writes sized WebP. Set `sizes:`.
- **The default build.** Leave `"runtime": "auto"`, `minify`, `split` and
  `compress` on.
- **Measure.** `wf build --stats` shows what each change costs, and
  `build.budget` warns when an output grows past what you set.
- **Large data.** A `data` file is inlined into the bundle. Large data is a
  `resource`.

## Security

- **Markup from outside the project** goes through
  `Unsafe.Html(sanitize(x))`, never bare (`V03` names every use).
  `Markdown(x).inert` is for text somebody else wrote.
- **`env`.** Only `PUBLIC_` names (or `public_env`) are read from pages
  (`E108`), because everything in the bundle is public.
- **Secrets.** A `Secret` can't be shown, spliced, logged or persisted
  (`T12`). Secrets live on a server or in the OS keychain.
- **No inline handlers or `javascript:` URLs.** The compiler refuses both
  (`E106`, `E107`).
- **Keep `build.csp: true`.** Widen it with `meta.connect`, `meta.img` and
  `meta.frame` by exactly the origins you use. `wf audit` lists what the
  project trusts.

## Comments

- **`///` documents.** Put it above a page, component, store, type, enum, prop,
  event, `state`, `derived` or `action`. Editors show it on hover, and
  `wf docs` puts it in the gallery.
- **Say why, not what.** "Version 1 kept `urgent: Bool`" earns its line;
  "the issues store" doesn't.
- **`//`** is for a note inside a body.

## Review checklist

Run these, in order, and read every finding:

1. `wf check`: no errors and no warnings. Never silence a finding by deleting
   what it's about. If it must stay, `// wf-allow(CODE)` on the line, with a
   comment saying why.
2. `wf fmt --check`
3. `wf test`
4. `wf build --stats`: nothing grew that shouldn't have.
5. `wf verify`: every route loads in Chrome with no error, no failed request
   and some text.

Then read the diff for what the compiler can't see:

- [ ] Every record is a `type` and every closed set of words an `enum`; no
      `Map` or `Any` where the shape is known.
- [ ] No `state` kept in sync by hand: it's a `derived`.
- [ ] No component reaches into a store it could be handed as a prop; events
      go up.
- [ ] Every list from data is keyed with `by`.
- [ ] Every `resource` handles loading, error, empty and refreshing.
- [ ] Every control has a visible label; every action button says what it
      acts on.
- [ ] Colours, sizes and spacing come from tokens; logical sides, not
      left/right.
- [ ] Every string a reader sees goes through translations if the project is
      translated.
- [ ] New behaviour has a test that says what the reader sees.
