# 45. Upgrading

<!--
route: guide/upgrading
group: help
blurb: What changed in each release, what to do when you upgrade, and how wf migrate carries an older project forward.
description: Installing a new version, what changed in 4.1, 4.0.1 and 4.0, and moving a WebFluent 2 or 3 project forward with wf migrate.
-->

## Installing a newer version

Run the installer again — it replaces the binary in place — or
`cargo install webfluent --force`. `wf --version` says what you have. Update
`wf-lsp` and the editor extension with it: the language server is built from
the same compiler.

The full notes for every release are in
[`RELEASE_NOTES.md`](../RELEASE_NOTES.md); what follows is what an upgrade
asks of you.

## 4.1 (next release)

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
