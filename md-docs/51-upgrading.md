# 51. Upgrading

<!--
route: guide/upgrading
group: help
blurb: What changed in each release, what to do when you upgrade, and how wf migrate carries an older project forward.
description: How to upgrade wf, what each release from 4.0 to 5.12 asks of a project, and moving a WebFluent 2 or 3 project forward with wf migrate.
-->

This chapter tells you what each release asks of your project when you
upgrade, newest first. Most releases need nothing from you; the ones that
do say exactly what to change, and `wf check` points at every place.

The full notes for every release are in
[`RELEASE_NOTES.md`](../RELEASE_NOTES.md). This chapter keeps only what an
upgrade asks of you.

In this chapter:

- [Install a newer version](#install-a-newer-version)
- [Upgrade a project](#upgrade-a-project)
- The notes for each release, from [5.12](#5-12) back to
  [WebFluent 2](#from-webfluent-2)

## At a glance

| Release | What to do |
|---|---|
| [5.12](#5-12) | Run `wf check` (`E118`, `E102`); rename a store member called `valueOf` or `toString` |
| [5.11](#5-11) | Run `wf check` (`C07`, `T22`), then `wf test --update` once you have read the new snapshots |
| [5.10](#5-10) | Run `wf check` and `wf test --update`; check `Column` spans, `Menu` labels and bound checkboxes with their own `on change` |
| [5.9](#5-9) | Separate style declarations with a newline or `;`, never `,` |
| [5.8](#5-8-5-7-and-5-6-x) | Nothing; a `state` that reads a live value may now warn `X06` |
| [5.7, 5.6.2, 5.6.1](#5-8-5-7-and-5-6-x) | Nothing |
| [5.6](#5-6) | Rename any declaration that takes a name the language has (`E118`) |
| [5.5](#5-5) | Handle `null` where an OpenAPI value may be null (`T04`) |
| [5.0](#5-0) | Run `wf check` and fix, or lower, the new errors |
| [4.2](#4-2) | Run `wf migrate` to replace `external`; check which `.js` files under `src/` now ship |
| [4.1](#4-1) | Nothing; rotate any secret a 4.0 build put in `env` |
| [4.0](#4-0) | Run `wf migrate`, and read the list of names it makes public |
| [3.x](#3-x) | The current grammar; nothing beyond 4.0 |
| [WebFluent 2](#from-webfluent-2) | Run `wf migrate` |

## Install a newer version

To upgrade the compiler, run the installer again — it replaces the binary
in place — or, if you installed with Rust:

```bash
cargo install webfluent --force
```

`wf --version` says which version you have. Update `wf-lsp` and the editor
extension at the same time: the language server is built from the same
compiler, and an older one reports what the new `wf` does not
([Troubleshooting](50-troubleshooting.md#the-editor-reports-what-wf-build-does-not)).

## Upgrade a project

After installing, bring each project forward:

1. Read this chapter's notes for every release between the version the
   project last built with and the new one.
2. Run `wf check`. It writes nothing, and every finding names its code, its
   place and what to do.
3. Run `wf test`. If snapshots changed, look at each difference, then run
   `wf test --update`.
4. Build, and load the result with `wf verify`.

## 5.12

5.12 adds names in any script and translation by a project's own words,
and makes every name a program binds its own. Your project may notice
these:

- **A store member may not be named like what every object has.**
  `valueOf`, `toString`, `constructor` and the other names every
  JavaScript object has are `E118` as a store's member, because the page
  threw as soon as it drew one. Rename the member. The same names now work
  as a component's props; `__proto__` as a prop is `E118`.
- **A test file may not declare a name the project declares.** The two
  were one name on the test's page, and every browser test failed with a
  SyntaxError. It is now `E102` at the test's declaration. Rename the
  test's own.
- **`T06` is reported at the member's name.** "`Store` has no member
  `x`" underlines `x`, not `Store`. A tool that reads the line and column
  sees the new place.
- **`output_type: "static"` pre-renders every page**, as `ssg: true`
  does. It used to build a single-page app. A project that set it and
  relied on that sets `"spa"` instead.
- **WebP is lossy at `build.media.quality`**, and a file no smaller than
  its original is not written. Pictures are smaller; a width that saved
  nothing is served from the original.
- **A name that clashed now works.** A program's own name that was one of
  the generated code's, or a browser global's, used to read the wrong
  value; it reads its own now. A page that leaned on the wrong value
  changes.

Run `wf check`, then `wf test`.

## 5.11

5.11 adds to the language and fixes what an editor built on it found. Your
project may notice these:

- **A gesture on a component needs one element to land on.** `drag:`,
  `dropValue:`, `on receive(d)`, `on longpress` and the rest, written on a
  component's call, used to throw at run time. They now land on the
  component's root element. A component whose body is an `if`, a `for`, a
  `match` or several elements has no root, and the call is the new error
  `C07`. To fix it, wrap the body in one `Stack` or `Row`.
- **A handler names at most what its event sends.** `on menu(key, x, y)`
  now reads each value that a component's `event menu(…)` sends. A name
  past the last one is `T22`. A DOM event sends only the event, so
  `on click(e, x)` is `T22` too.
- **A test file's declarations are its own.** A `const`, a component or a
  type written beside the tests in one file under `tests/` is no longer
  seen by another file's tests. Two files may now declare the same name. A
  file that used another file's `const` declares it itself, or moves it
  under `src/`.
- **A test names a control as a reader hears it.** What is `aria-hidden`
  (a menu item's shortcut, a glyph) is not part of the name, and a
  submenu's entry is its label without its items. `click "Paste Ctrl+V"`
  is now `click "Paste"`, and `click "File"` opens a `Menu.Sub(label:
  "File")`. `A15` no longer counts an `aria-hidden` glyph as part of a
  control's visible label either.
- **A block handed to a component reads the names where it was written**
  in `wf test` and the template engine, as the live page and the
  pre-rendered page already did. A snapshot that showed the inner
  component's value of a shared name changes.
- **A `Menu.Item`'s `class:` is added beside the menu's own**, where it
  used to replace it, so the item keeps its padding and its shortcut
  column. A style that made up for the lost class can go.
- **A method of a receiver's own wins over a helper of the same name.**
  `x.last(k)` on a value with a `last` method of its own calls that method
  with every argument, where it used to call the list helper. A store's
  action named like a helper (`preview`, `first`, `take` …), called from
  another store, runs that action.

Run `wf check`, then `wf test --update` once you have read what the new
snapshots show.

## 5.10

5.10 makes pages, pre-rendered pages and tests do what this guide says.
Your project may notice the changes below. Run `wf check` after upgrading.

### Layout and look

- **Column spans leave room for the gap.** A `Column` in a `Row` with a
  gap is narrower by its share of the gap, and spans that add up to 12 sit
  on one line instead of wrapping the last. A layout that made up for the
  old behaviour, such as `span: 8` beside `span: 3`, now leaves a space;
  give it back its `4`. The spans read the gap from `--wf-gap`, which
  `gap:` sets, so a gap your own CSS gives a `Row` (`style { gap: 4px }`)
  needs `--wf-gap: 4px` beside it. A PDF now lays a `Column` out at its
  `span:`, and reads a `calc()` of a percentage and points against the box
  it lands in, as a browser does.
- **The `Navbar` takes the theme's surface and text colours**, so a dark
  or custom theme no longer gets a white bar.
- **A keyed list slides only the items whose index changes.** An item that
  keeps its index while its place on screen moves now jumps there.
- **A `Splitter`'s `reset:` is written as given**, not clamped to `min:`
  and `max:`, and the first drag or key press moves from the edge as
  drawn.

### Controls and forms

- **A `Menu`'s `label:` names its list, not its button.** The button says
  what `trigger:` says. `Menu(label: "File")` with neither `trigger:` nor
  `visible:` now draws a button that says *Menu*; write `trigger: "File"`.
  A menu that `visible:` opens no longer draws a button. A `Dropdown`'s
  `label:` is still its button.
- **A `Form` without `bind:` asks the `validate` rules** of its page or
  component before `on submit`, and shows their messages instead of
  submitting while one fails. Each copy of a component with
  `Form(bind: …)` has its own form, where all its copies shared one.
- **A bound `Checkbox` or `Radio` with its own `on change` writes its
  state first.** The binding's write used to be lost, so a handler that
  wrote the state itself now writes it a second time, and one that toggles
  it (`agreed = !agreed`) undoes the click: take that write out. A
  `Switch`'s `on change`, which never ran, now runs.
- **Text that reads state follows it.** Control labels, a `Table`'s
  `caption:`, tab titles, tooltip text, a modal's `title:`, a menu's
  trigger, an `Avatar`, a `Skeleton`'s sizes and an `Icon`'s positional
  name kept the value they were drawn with; they now change with it. A
  `Checkbox(label: t("…"))` changes language with `setLocale`.
- **An `Avatar` whose `src:` is a run-time `data:` or `blob:` address**
  gets an empty `src`, as any other `src:` does.
- **Built-ins write what they are given.** `role:`, `tabindex:`, `title:`,
  `hidden:`, `aria-*`, `data-*` and `on` handlers on a `VirtualList`,
  `Splitter`, `Tabs`, `Sidebar`, `Breadcrumb`, `Slider`, `DatePicker`,
  `FileUpload`, the layers and the rest now reach their element, where some
  were dropped. `label:` on a `VirtualList`, `Splitter` or `Carousel` is
  written as its `aria-label`, not as text in it. `disabled: true` on a
  `Slider`, `DatePicker` or `FileUpload` disables it. A `Sidebar`'s `id:`
  is its id. Pre-rendered HTML and `wf test` snapshots carry these too:
  look at the difference and run `wf test --update`.

### Layers and keys

- **The Escape that closes an open `Modal` or `Dialog` stays there**: a
  page's `on key("Escape")` no longer runs for it.
- **An anchored layer hides with its anchor** when the anchor scrolls out
  of view, and stays open. Style it through `[data-anchor-hidden]` if you
  need to.
- **A modal's Escape and title.** Over an open popover, the first Escape
  closes the modal and the next closes the popover. A press inside the
  modal no longer closes the popover beneath it. In the pre-rendered page,
  a modal's title is its heading. A tooltip under the pointer lets Escape
  go on to the page.

### Language, state and stores

- **`setLocale` is remembered.** The choice is kept in the reader's
  `localStorage` under `wf:locale`, and the next page opens in it; a
  `?lang=` address still wins. A `persist` store kept for the same purpose
  still works beside it.
- **Stores**: a store's derived values keep following their state after
  the first page that read the store has gone, and one that reads a
  `derived` written below it follows it. The built script declares a store
  after the stores it `use`s.
- **A local of a member's name is the action's own.** A store action that
  wrote a member through a `let` of the same name now writes the local.
- **`data-x: true` passed through a component is `"true"`**, where it was
  `""`, so a selector on `[data-x=""]` stops matching.
- **`Portal` is a built-in's name**: a project's own component or store
  called `Portal` is reported.
- **Markdown reads backslash escapes.** A text with `\*`, `\_`, `\#`, `\[`
  and the like now shows the character alone, and the character takes no
  part in emphasis or a link; `\\` is one backslash. A backslash at the end
  of a paragraph's last line is shown, where it was dropped. A link or
  picture whose address has a scheme a page refuses (`javascript:`,
  `data:`) is drawn as its words.
- **Fewer warnings and errors**: an `Icon` naming one of the 23 new icons
  draws it instead of warning `V08`; `on auxclick`, `on focusin` and
  `on focusout` no longer warn `V09`; `on blur` and `on focus` outside an
  element compile, where they were an error; and `() => value` fits a
  function of no arguments, where it was `T01`.

### Security headers

- **A site that frames its own page**, with a `Frame` whose `src:` is a
  site-relative string (`"/preview/"`, `"/embed/{id}"`), is served
  `X-Frame-Options: SAMEORIGIN` and `frame-ancestors 'self'`. One whose
  same-site address comes from a variable is still served `DENY`, so the
  browser still refuses it. Write the address as a string, such as
  `"/embed/{id}"`.

### Pre-rendered pages and tests

- **Pre-rendered HTML matches the live page.** A `Menu` or `Dropdown` is
  its button and a hidden list, so its entries no longer show before the
  script runs, and a menu that state opens is its list alone. A
  `Checkbox`, `Radio` or `Switch` has its `<input>`, with the control's
  `id:` on it. A part written with flags carries their classes. A `Column`
  with a literal `span:` carries `wf-col--N`. `disabled`, `required` and
  `controls` are left off when false, or when the build cannot know them.
- **`wf test` snapshots change** where they hold a `Menu`, a `Dropdown`, a
  `Column` with a `span:`, a `keyLabel`, a count in a locale with more
  plural forms than English, or a `disabled:`, `required:` or `controls:`
  that is false: look at the difference and run `wf test --update`. An
  `expect` for a `Menu`'s `label:` text no longer finds it (it is an
  `aria-label`), and one for its `trigger:` does. `expect` reads `&#x27;`
  as an apostrophe, and in a test that acts compares with white space
  collapsed. A `click` may now find a tab, a menu item or a radio button
  it used to miss.
- **Tests read as a reader does.** An `expect` that counted on a space
  between two elements that touch ("live : …") now reads them touching. A
  test run in a browser has the project's own CSS, so CSS that hides or
  changes text changes what the test sees, and a stylesheet naming a file
  not in `public/` fails it. Snapshots lose `data-*` attributes that were
  `false`, and gain a call's `data-*`, `aria-*`, `id` and `class:` on a
  component's root. Run `wf test --update` after looking at the
  difference.

### Windows and tools built on the compiler

- **On Windows, type `webfluent`** (or `wf.exe`) wherever this guide says
  `wf`. Each earlier `.msi` had an `UpgradeCode` of its own, so the new
  one cannot replace it: uninstall the old one from *Apps*, or both stay
  installed and on `PATH`.
- **A tool built on the compiler** adds the new fields where it writes a
  struct literal: `doc` on `StateDecl`, `DerivedDecl` and `ActionDecl`,
  `frames_itself` on `BuildConfig`, `only_route` on `BuildOptions` and
  `pages_deferred` on `BuildReport`. It adds a `Length::Mix(..)` arm where
  it matches `paged::style::Length`, and passes a locale to
  `testing::render_test` as its fourth argument. `print::statement` text
  for a documented member starts at its `///`, so replace from
  `print::written_start`. `check_project_via` starts threads, and calls a
  `Project`'s `file_of` and `source_of` up front.

## 5.9

5.9 holds a few things to what this guide already said. Your project may
notice them:

- **A comma between two style declarations is an error** (`E002`).
  `style { width: {w}px, flex: none }` compiled to one declaration whose
  value was the rest of the line, so the browser dropped both. Write each
  declaration on its own line, or separate them with `;`. A comma inside a
  value (`font-family: a, b`) is untouched.
- **A store's `persist … key:` is kept.** It was dropped, so the value was
  kept under no key; now it follows the key, as a page's does. What is
  already stored is found under the same names.
- **A refined type's bounds are checked as the reader types** where a
  `validate` block guards the state: `Number(0..10)` no longer takes 10,
  and `Date(after: …)` and `before:` are held. A field that accepted its
  open end now says *Too large*.
- **`Card(animate: .Drop)` plays a project's own animation** on an element
  that stands still. It did nothing before, so a page may now move where
  it did not.
- **A count of exactly 0 shows `key.zero`** wherever the translation file
  has one, in any language, and the pre-rendered page picks each count's
  form by the default locale's rules rather than English's. A `zero`
  message that never showed now does.
- **A site whose default locale is right to left** is written `<html
  lang="ar" dir="rtl">` from the first paint. `meta.lang` names a region of
  the default locale's language, and is otherwise the default locale.
- **New warnings:** `A17` (a `Frame` with no title) and `V12` (a long list
  that is not windowed). `wf check` names each with its fix.

## 5.8, 5.7 and 5.6.x

Nothing to change.

- **5.8**: a build may now warn `X06` on a `state` whose first value reads
  a live one, such as `now`. Making it `derived` is the fix. A function
  written only to get a script's value onto a page can go back to being the
  value.
- **5.7**: what a page did to live through a refresh can go — a sheet or a
  form kept outside the `match` with a snapshot of the data, or a `derived`
  copy held so the arm would not draw again. A page that wants a spinner on
  every refresh shows one for `name.refreshing`. A hand-written `WF.match`
  arm is handed its value as before, and an accessor of it as a second
  argument; `WF.each` keeps a keyed item's nodes under a new value with
  `live: true`, which is what a compiled `for … by` asks for.
- **5.6.2**: a wrapper written only to keep a nested branch apart — a
  `Stack` styled `display: contents` around a tab's `match`, or a branch
  flattened into one `if let` over a `derived` value — can go.
- **5.6.1**: a `derived` value written only to make a resource's label
  live can go back into the label.

## 5.6

5.6 refuses a name the language already has as a program's own
([`E118`](43-diagnostics.md#e118-a-name-the-language-already-has)): a
`state env`, a loop's `t`, a `persist theme`, an action named `update`.
Each compiled before, and read the language's value where the program
meant its own — a page's `state env` showed `[object Object]`.

Run `wf check`. The message names the declaration and suggests a name of
your own; renaming it and its reads is the whole change. The usual
renames:

| Was | Rename to |
|---|---|
| a loop's or a lambda's `t` (`for t in todos`, `items.filter(t => …)`) | `todo`, `item` |
| `state query` | `search` |
| `persist theme` | `scheme` |
| a route parameter or `state env` | `target`, `stage` |
| an action named `update` | `edit` |

## 5.5

A value an OpenAPI specification marks as nullable is now `T?`, so a read
that ignored `null` — `Text(e.reason.toUpperCase())` — is `T04` where it
compiled before. Use `?.`, `??` or `if let` to say what to do when it is
`null`. A response field with a `format` is now a `DateTime`, a `Url` and
so on, and a `String` assigned to one is `T01`.

## 5.0

5.0 makes the compiler refuse much more of what used to compile and then
fail in the browser. A project that built under 4.x may stop on errors it
always had. Start with a check, which writes nothing:

```bash
wf check
```

Every finding names its code, its place and what to do. `wf explain CODE`
prints the code's entry in [Diagnostics](43-diagnostics.md). Where the fix
is known, your editor offers it as a quick fix, and `wf check --format
json` lists it as edits.

**New errors.** Each is a program that cannot work as written:

- **Types** (`T`): a field, member or method that does not exist; a value
  of the wrong type; a value that may be `null` read as if it were not; a
  `match` that misses a case; arithmetic on what is not a number; a
  comparison that is always the same; an `await` where it cannot run; a
  name nothing declares.
- **Writing what cannot change** (`X01`): a `const`, a `derived` value, a
  prop, a route parameter, a loop variable.
- **Components** (`C`): a required prop left out, a prop the component
  does not declare (`C02`), a part outside its owner.
- **Routes** (`R`): a link to a path no page matches (`R01`), a `:param`
  with no page parameter, an `app` with no `Router` or with two; two pages
  on one path (`S04`).
- **Forms** (`F`): `bind:` to something that cannot be written, or to a
  state the control cannot hold; a `validate` no control can satisfy.
- **Reactivity** (`X`): an effect that writes what it reads every run, a
  `derived` value that calls an action, `use` of a store nothing declares.
- **Data and translations** (`D`, `I`): a persisted value storage cannot
  keep; a `t("key")` no translation has, a placeholder the call does not
  pass, a locale the project does not list.
- **Literals and styles**: a date that does not exist, money with too many
  decimals, a key combination no keyboard sends, a `$token` the theme does
  not declare.
- **The security policy**: a page's `head { script(src: …) }` from an
  origin the config does not declare (`E902`).

**Lower what you cannot fix today.** `"lints": { "R01": "warn" }` in the
config lowers a code or a family. An error may be lowered only when it
ships no broken page: `R01`, `C02`, `S04`, `I01`, `I02`, `I04`. To accept
one finding on one line, write `// wf-allow(CODE)`
([Diagnostics](43-diagnostics.md#u07-an-allow-that-allows-nothing)).

**What the compiler now writes for you** instead of reporting:

- A change inside a state's value — `form.name = v`, `items[i].done = v`,
  `items.sort()` — compiles to the state given an updated copy, so what
  reads it repaints and `persist` keeps it.
- A value spliced into an address — `to:`, `navigate()`, `fetch()` — is
  encoded where it stands for a value.
- A `match` expression that covers every case of an enum needs no `else`.

**Behaviour that changed:**

- **`wf serve` answers a missing fetch with a 404.** A request that is not
  a page navigation, for a file the build did not write, used to get the
  single-page shell — a page of HTML handed to `JSON.parse`. It is a `404`
  now, under `wf verify` and `wf test` too, and `wf serve` prints a line
  for each. Point the page at a server that has the API, or put a file in
  `public/`.
- **An empty path parameter is not sent.** `Backend.user(id: "")` with
  `at "users/:id"` used to request `users/`, the whole collection. It now
  ends as `.aborted`, and a resource over it stays `loading`.
- **One list item that throws no longer blanks the list**: the rest draws,
  and under `wf serve` the item says what it threw.
- **An effect that never settles is stopped** after 100 runs, instead of
  overflowing the stack.
- **Each build keeps what its persisted values start as** in `.wf-cache/`,
  so the next build can warn about a shape that changed (`D04`), and
  `wf verify --returning-visitor` can load your pages as a returning
  reader. Keep `.wf-cache/` between CI runs if you can.

**New in 5.0:** `wf check`, `wf explain`, `--format json|sarif|github` and
`--deny-warnings` on `wf build` and `wf check`, quick fixes in the editor,
`persist … { key: id }` for one stored value per instance, and `wf verify`
visiting `:param` routes. [Command line](41-cli.md) describes each.

## 4.2

**`external` is gone.** Run `wf migrate`. It turns a remote module into a
`meta.scripts` entry, a local module into a plain script under `src/` with
its signatures as JSDoc, and an `external element` call into
`Element("tag", …)`. A call into a remote module that `external` typed is
`Any` afterwards.

**Every `.js` file under `src/` now ships**, linked on every page before
the compiled code. `wf migrate` lists the ones it finds; move any that are
not meant for the browser out of `src/`.

New in 4.2:

- **Your own JavaScript**: a plain `.js` file under `src/`, its top-level
  functions in scope by name, typed by its JSDoc
  ([JavaScript interop](39-javascript-interop.md)).
- **`meta.scripts`** loads a library from a CDN, imports a module with
  `as:`, and names its `globals`.
- **`mount:`, `update:`, `cleanup:` on any element**, and a `wf:render`
  event for a script that finds elements itself.
- **`Element("tag-name", …)`** places any custom element.
- **`class:` takes a map and a list**: `{ "is-done": done }`,
  `["card", tone, if open { "is-open" }]`. An `if` with no `else` is `null`
  when its condition fails ([Styling](19-styling.md#classes-and-your-own-stylesheet)).
- **`V04`** warns about a `class:` that borrows one of the built-ins'
  `wf-` classes.

Fixed in 4.2: what an `if` branch, a `match` arm, a list item or a slot
fill made — a timer, an effect, a `Host`'s cleanup — now goes when its
page does. A route change used to leave it running.

## 4.1

Nothing to change in your code. New in 4.1:

- **Offline**: name `offline` in the config for a service worker, writes
  kept for later, and an update flow ([Offline](26-offline.md)).
- **Peers**: `peer link = rtc(signal: …)`, a WebRTC data channel to another
  page ([Realtime](25-realtime.md#connect-two-readers-pages)).
- **`env` from a `.env` file and the shell**, on top of the config
  ([Environments](35-environments.md)).
- **Unknown config keys are reported**, with the key they probably meant.
- **Pagination**: `paginate: .page`, `.items`, `.loadMore()`, `.hasMore`.
- **`?lang=` opens a page in that locale**, as the `hreflang` alternates
  promise.
- **`class`-exported libraries work through `external`**: a class is
  constructed when called.
- **Every standard DOM event** is accepted in `on …`.

Fixed in 4.1, and worth knowing because they could have affected you:

> [!CAUTION]
> A private `env` value no longer reaches the bundle. 4.0 refused a page
> that *read* a private name, but still wrote the whole `env` map into
> `app.js`. If you deployed 4.0 or 4.0.1 with a secret in `env`, rotate it.

- An action's parameters are its own: `action move(by: Number)` read `by`
  as a signal and threw.
- `head { }` tags may read a page's `derived` values.
- A resource's `state`, `data` and `error`, and a socket's `messages`,
  show their values in text.
- `navigator` and `location` compile as the browser's.
- Two declarations on one line —
  `theme T { color-primary: #0F766E  radius-md: 14px }` — are two, not
  one.
- `S04` reports two pages on one route; `A12` counts an `h1` in each
  branch of an `if` as one.

## 4.0

4.0 changed what the compiler allows. `wf migrate` handles the mechanical
part and names every place that needs a decision. Preview what it would
change, then apply it:

```bash
wf migrate --check
wf migrate
```

- **`env` is public or private.** A page may only read a name beginning
  `PUBLIC_` or listed in `public_env`. The migration adds every name your
  pages already read to `public_env` and prints the list. Read the list:
  public means anyone who opens the site can read the value.
- **An `on*` attribute is an error** (`onclick: "…"`); write
  `on click { }`.
- **A URL a browser would run is refused** — `javascript:`, `data:`,
  `vbscript:`, `blob:`, `file:` in `href:`, `src:`, `to:`, `poster:` or
  `navigate()`.
- **Stores are built on first read**, not at boot. A store whose set-up
  must run anyway takes `eager: true`.
- **A `persist` value follows other tabs.** `sync: false` opts out.
- **`WF.store` and `WF.host` were renamed** in the runtime's functions.
- **Layouts stay as written**: a `Row` no longer turns into a column on a
  narrow screen by itself. The migration adds `.stacks` to keep the old
  behaviour.

## 3.x

WebFluent 3 introduced the grammar this guide describes: lowercase
declarations, one positional argument, flags after a dot, `on click { }`
handlers, and `style { }` blocks of CSS. A 3.x project moves forward with
the 4.0 notes above.

## From WebFluent 2

`wf build` refuses a WebFluent 2 file (`Page Home (path: "/") { … }`) and
points you to the migration. `wf migrate` rewrites every `.wf` under
`src/` into the current grammar. It is a change of spelling only: the
migrated project builds to what it built before. `wf migrate --wfx` writes
the result in the indented layout.

## Next

[Contributing](52-contributing.md): how the compiler is put together, how to build and test it, and how to change the language, a built-in or this guide.
