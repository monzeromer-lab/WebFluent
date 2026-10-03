# 45. Upgrading

<!--
route: guide/upgrading
group: help
blurb: What changed in each release, what to do when you upgrade, and how wf migrate carries an older project forward.
description: Installing a new version, what changed in 5.0, 4.2, 4.1, 4.0.1 and 4.0, and moving a WebFluent 2 or 3 project forward with wf migrate.
-->

## Installing a newer version

Run the installer again — it replaces the binary in place — or
`cargo install webfluent --force`. `wf --version` says what you have. Update
`wf-lsp` and the editor extension with it: the language server is built from
the same compiler.

The full notes for every release are in
[`RELEASE_NOTES.md`](../RELEASE_NOTES.md); what follows is what an upgrade
asks of you.

## 5.0

5.0 makes the compiler refuse much more of what used to compile and then
fail in the browser. A project that built under 4.x may stop on errors it
always had. Start with a check, which writes nothing:

```bash
wf check
```

Every finding names its code, its place and what to do; `wf explain CODE`
prints the code's entry in [Diagnostics](39-diagnostics.md). Where the fix
is known, your editor offers it as a quick fix, and `wf check --format
json` lists it as edits.

**New errors** — each is a program that cannot work as written:

- **Types** (`T`): a field, member or method that does not exist; a value
  of the wrong type; a value that may be `null` read as if it were not; a
  `match` that misses a case; arithmetic on what is not a number; a
  comparison that is always the same; an `await` where it cannot run; a
  name nothing declares.
- **Writing what cannot change** (`X01`): a `const`, a `derived` value, a
  prop, a route parameter, a loop variable.
- **Components** (`C`): a required prop left out, a prop the component does
  not declare (`C02`), a part outside its owner.
- **Routes** (`R`): a link to a path no page matches (`R01`), a `:param`
  with no page parameter, an `app` with no `Router` or two; two pages on
  one path (`S04`).
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

**Lowering what you cannot fix today.** `"lints": { "R01": "warn" }` in the
config lowers a code or a family; an error may be lowered only when it ships
no broken page — `R01`, `C02`, `S04`, `I01`, `I02`, `I04`. One finding on
one line is accepted with `// wf-allow(CODE)`
([Diagnostics](39-diagnostics.md#u07-an-allow-that-allows-nothing)).

**What the compiler now writes for you** instead of reporting:

- A change inside a state's value — `form.name = v`, `items[i].done = v`,
  `items.sort()` — is compiled into the state given an updated copy, so
  what reads it repaints and `persist` keeps it.
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
  `at "users/:id"` used to request `users/`, the whole collection; it now
  ends as `.aborted`, and a resource over it stays `loading`.
- **One list item that throws no longer blanks the list**: the rest draws,
  and under `wf serve` the item says what it threw.
- **An effect that never settles is stopped** after 100 runs instead of
  overflowing the stack.
- **Each build keeps what its persisted values start as** in
  `.wf-cache/`, so the next can warn about a shape that changed (`D04`) and
  `wf verify --returning-visitor` can load your pages as a returning reader.
  Keep `.wf-cache/` between CI runs if you can.

New: `wf check`, `wf explain`, `--format json|sarif|github` and
`--deny-warnings` on `wf build` and `wf check`, quick fixes in the editor,
`persist … { key: id }` for one stored value per instance, and `wf verify`
visiting `:param` routes. [Command line](37-cli.md) has each.

## 4.2

**`external` is gone.** Run `wf migrate`: a remote module becomes a
`meta.scripts` entry, a local module a plain script under `src/` with its
signatures as JSDoc, and an `external element` call `Element("tag", …)`.
A call into a remote module that `external` typed is `Any` afterwards.

**Every `.js` file under `src/` now ships**, linked on every page before the
compiled code. `wf migrate` lists the ones it finds; move any that are not
meant for the browser out of `src/`.

New:

- **Your own JavaScript**: a plain `.js` file under `src/`, its top-level
  functions in scope by name, typed by its JSDoc
  ([chapter 32](32-javascript-interop.md)).
- **`meta.scripts`** loads a library from a CDN, a module with `as:`, and
  names its `globals`.
- **`mount:`, `update:`, `cleanup:` on any element**, and a `wf:render`
  event for a script that finds elements itself.
- **`Element("tag-name", …)`** places any custom element.
- **`class:` takes a map and a list**: `{ "is-done": done }`,
  `["card", tone, if open { "is-open" }]`; an `if` with no `else` is `null`
  when it fails ([chapter 15](15-styling.md#classes-and-your-own-stylesheet)).
- **`V04`** warns about a `class:` that borrows one of the built-ins' `wf-`
  classes.

Fixed: what an `if` branch, a `match` arm, a list item or a slot fill made
— a timer, an effect, a `Host`'s cleanup — now goes when its page does; a
route change used to leave it running.

## 4.1

Nothing to change in your code. New:

- **Offline**: name `offline` in the config for a service worker, writes kept
  for later, and an update flow ([chapter 19](19-offline.md)).
- **Peers**: `peer link = rtc(signal: …)`, a WebRTC data channel to another
  page ([chapter 18](18-realtime.md#a-peer)).
- **`env` from a `.env` file and the shell**, on top of the config
  ([chapter 30](30-environments.md)).
- **Unknown config keys are reported**, with the key they probably meant.
- **Pagination**: `paginate: .page`, `.items`, `.loadMore()`, `.hasMore`.
- **`?lang=` opens a page in that locale**, as the `hreflang` alternates
  promise.
- **`class`-exported libraries work through `external`**: a class is
  constructed when called.
- **Every standard DOM event** is accepted in `on …`.

Fixed, and worth knowing because they could have bitten you:

- **A private `env` value no longer reaches the bundle.** 4.0 refused a page
  that *read* a private name, but still wrote the whole `env` map into
  `app.js`. If you deployed 4.0 or 4.0.1 with a secret in `env`, rotate it.
- An action's parameters are its own: `action move(by: Number)` read `by` as
  a signal and threw.
- `head { }` tags may read a page's `derived` values.
- A resource's `state`, `data` and `error`, and a socket's `messages`, show
  their values in text.
- `navigator` and `location` compile as the browser's.
- Two declarations on one line — `theme T { color-primary: #0F766E  radius-md: 14px }`
  — are two, not one.
- `S04` reports two pages on one route; `A12` counts an `h1` in each branch of
  an `if` as one.

## 4.0

4.0 changed what the compiler allows. `wf migrate` handles the mechanical
part and names every place that needs a decision:

```bash
wf migrate --check
wf migrate
```

- **`env` is public or private.** A page may only read a name beginning
  `PUBLIC_` or listed in `public_env`. The migration adds every name your
  pages already read to `public_env` and prints the list: read it, because
  public means anyone who opens the site can read the value.
- **An `on*` attribute is an error** (`onclick: "…"`); write
  `on click { }`.
- **A URL a browser would run is refused** — `javascript:`, `data:`,
  `vbscript:`, `blob:`, `file:` in `href:`, `src:`, `to:`, `poster:` or
  `navigate()`.
- **Stores are built on first read**, not at boot. A store whose set-up must
  run anyway takes `eager: true`.
- **A `persist` value follows other tabs.** `sync: false` opts out.
- **`WF.store` and `WF.host` were renamed** in the runtime surface.
- **Layouts stay as written**: a `Row` no longer turns into a column on a
  narrow screen by itself. The migration adds `.stacks` to keep the old
  behaviour.

## 3.x

WebFluent 3 introduced the grammar this guide describes: lowercase
declarations, one positional argument, flags after a dot, `on click { }`
handlers, `style { }` blocks of CSS.

## From WebFluent 2

A WebFluent 2 file (`Page Home (path: "/") { … }`) is refused by `wf build`
with a pointer to the migration. `wf migrate` rewrites every `.wf` under
`src/` into the current grammar — a change of spelling that builds to what it
built before — and `wf migrate --wfx` writes the result in the indented
layout.

## Next

[Browser support](46-browser-support.md).
