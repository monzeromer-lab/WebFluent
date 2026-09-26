# WebFluent v4.1 Release Notes

## Security

### A private `env` value no longer reaches the bundle

4.0 made a page that *reads* a non-public `env` name a compile error — and
then wrote the whole `env` map into `app.js` as `const env = {…}` anyway, so
a value like `STRIPE_SECRET` in `webfluent.app.json` was downloaded by every
reader. The bundle now carries only the names a page may read: those
beginning `PUBLIC_` and those `public_env` lists. **If a site built with 4.0
or 4.0.1 had a secret in `env`, rotate it.**

## Added

### Offline

Name `offline` in the config and `wf build` writes a service worker that
stores the site for when the network is gone — the shell, the routes
`precache` names with their own chunks, and a `fallback` page for the rest —
and fetches what it did not write by the policies `cache` names. It is
versioned by a hash of everything the build wrote, so a deploy that changes a
byte is a new version; the page reads `update.available` and calls
`update.apply()`, which takes it and reloads **once**. With `sync`, a write
made with no network is kept in IndexedDB and sent in order when the
connection returns — by Background Sync where the browser has it, so a
closed tab still sends it — and `network.queued` says how many wait. Under
`wf serve` the worker takes itself away.

It is held to a real Chrome in CI (`tests/browser/offline.mjs`, 24 checks
across a pre-rendered and a single-page build): the site stored, a stored
route served with the server gone and the fallback for one that is not, a
write kept, counted and received exactly once, and an update offered and
taken with one reload.

### Peers

`peer link = rtc(signal: …)` opens a WebRTC data channel straight to another
page, with the handle and the `match` of a socket — `connecting`, `open`,
`closed(c)`, `error(e)`, `.send`, `.messages`, `.last`. The language supplies
the channel, not a server: the offer, the answer and the routes each side
can be reached by go out through `signal:`, over whatever the app already
has, and what the other side sent is handed to `link.signal(m)`. One side is
the `initiator`; `ice:` names STUN or TURN servers. A page that leaves
closes its peer, and the other side reads `closed` at once. Held to two
pages in a real Chrome in CI (`tests/browser/peer.mjs`).

### The documentation, rewritten

The guide is 48 chapters in six parts — Start, the basics, Building,
Shipping, Reference and Help — with a tutorial that ends in a deployed
site, chapters for forms, real-time, offline, accessibility, testing,
deploying, environments, performance and JavaScript interop, and reference
chapters for every command, config key, diagnostic, built-in, design token,
runtime function and keyword. It is held to the compiler: every example is
compiled, every page example is run and clicked through, every diagnostic
example draws its code, every link lands, and a test fails when the language
has something the guide does not name.

### A name nothing declares is a compile error

`Button(huge)` and `on click { cuont = count + 1 }` built, and threw a
`ReferenceError` in the browser the first time the page ran. A name that no
`state`, `const`, prop, action, store, `api`, `external` or browser global
declares is now `T13`, reported where it is written, with the spelling to
check. A template's names are its data's, so `wf render` is not held to it.

### `wf render` is held to what a build is

A template that named a component nothing declares rendered
`<!-- unknown component -->`, and a flag a component does not take was
dropped, both with exit status 0 — an invoice with a typo went out that way.
The render now stops on everything a build stops on except `T13`. It also
takes `--token NAME=VALUE` for a design token over the theme's, reads an
indented `.wfx` template, and finds a `data` file beside the template, as
the guide always said it did.

### The Node binding, as `webfluent` on npm

`npm install webfluent` — a thin wrapper around `wf render`, at the
compiler's version, published by the release once the repository has an npm
token. `withTokens()` reached nothing before and does now; `withTheme()` is
only sent when set, instead of naming a `default` theme; `renderSlides()` is
new; a template that does not compile throws what `wf` said. Its tests run
in CI against the `wf` the job built.

### A title that looks its entry up

`title: "{posts.find(p => p.slug == slug)?.title ?? slug} — Blog"` was
written into every file as those characters: only a bare parameter name was
filled in. A title may splice any expression over the route's parameters
and the program's constants and data; the build works it out for each file
and the router again on each visit.

### `env` from a `.env` file and the shell

`env.NAME` reads the config's `env` map, then a `.env` file beside it, then
the shell — so a pipeline can give staging and production their own values
without editing a file. The shell supplies a name the config or `.env`
declares, or a public one.

### A config key nothing reads is reported

`"defaultLocale"` in `webfluent.app.json` was dropped without a word; the
build now warns and names the key it meant (`default_locale`). `wf init
-t static` itself wrote the misspelling, and so did the documentation site.

### And

- **Pagination**: `resource list = Api.items(page: page, paginate: .page)`,
  then `list.items`, `list.loadMore()`, `list.hasMore`. The compiler emitted
  the option and the checker refused it; the runtime now has it.
- **A route's parameters in its title**: `title: "{slug} — Blog"` gives each
  pre-rendered file, and the live tab, that route's value.
- **`?lang=ar`** opens a page in that locale — the address the build's
  `hreflang` alternates always pointed at.
- **Every standard DOM event** in `on …`: `dblclick`, `pointerdown`,
  `scroll`, `paste`, `drop` and the rest, without a warning.
- **`on open { }`** on a socket, as a peer and a channel had it.
- **A class a module exports** is constructed when an `external` calls it,
  so Chart.js and most modern libraries work through `external` + `Host`.
- `wf init` writes a `.gitignore`; `wf generate page` gives the page a
  description.

## Fixed

### A sidebar's items on a phone, and in Arabic

A `Sidebar` is a column of fixed height that scrolls, and its items shrank
to fit instead: with more than a screenful, every link was squeezed to 18px,
too small to tap. They keep their size and the column scrolls. On a phone a
closed sidebar waits off its own edge — in a right-to-left page it slid the
wrong way and covered half the screen.

### An action's parameters

`action move(by: Number) { n = n + by }` on a page or a component compiled
`by` as a signal, `_by()`, which does not exist: every such action threw the
first time it ran. A store's actions were never affected.

### What the guide's examples found

Writing every example out as a real page and running it found programs the
checks accepted and the browser rejected:

- `head { meta(content: post?.title) }` read a `derived` before it existed.
- `Text("{rows.state}")` showed a signal's source; a resource's `state`,
  `data`, `error` and a socket's `messages` are now read as their values.
- `navigator` and `location` compiled as signals.
- `app { state … }` never declared its state.
- `now(every: 1.seconds)` called a `now` nothing declared.
- An action handed over as a value, `addEventListener("scroll", track)`, was
  read as a signal.
- A type an `external` declares could not annotate a value.
- `Backend.avatar.progress` was refused by the checker.
- Two declarations on one line — `theme T { color-primary: #0F766E
  radius-md: 14px }` — were read as one, setting the first token to both.
- `S04` never reported two pages on one route; `A12` counted an `h1` in each
  branch of an `if` as two.
- A test that clicks failed in any project with a dark theme.
- The CSP check took a code sample's `onClick=` for an attribute.
- Warnings printed as `Warning: Error: …`.
- The Node binding did not look where the installer puts `wf`.

### A `site_url` with the subpath in it

`meta.site_url` written as the site's whole address — `https://x.github.io/docs`
beside a `base_path` of `/docs` — gave every canonical link, link preview and
sitemap entry the base path twice: `/docs/docs/`. It is read once now, so
either spelling means one address.

### A browser value in text

`Text("{viewport.width}")`, a clock on `now`, `network.online` — any text
that read one of the browser's values was computed once and never moved.
Only conditions were drawn live: the check that decides whether a text
follows its values knew signals and translations and none of these.

### `text.lines()` on a live page

Every runtime module shares one closure, and the network module declared a
`lines` of its own — the reader for a streamed response. Wherever that
module was built in, the later declaration won: `text.lines()` handed back
an async generator, so the static paint showed the lines and the live page
replaced them with `undefined`. The stream reader has its own name now.

### Escapes in a splice

A string written inside a splice keeps its own escapes:
`"n={("a\"b").length}"` refused to build, because the escape was undone
before the splice was read. And the older spelling `{a ?? \"x\"}`, which
the compiler has always said it accepts, was shown as text rather than
read as a splice.

### `url.query()` and `url.with(…)` paint at build time

They painted nothing until the script ran. The static paint now computes
both as the browser's `URL` does — values decoded, a key replaced where it
stood, the search written back form-encoded.

### Held to each other

Behind these, three checks: every method's value at build time is compared
with the runtime's, in Node, for 61 samples — anything the build cannot
know says why; every runtime function the compiler can call must exist;
and no two runtime modules may declare the same name.

## The documentation site

Written for a phone first: every rule is the phone's, and `md`, `lg` and
`xl` add to it. The search is in the phone's menu, every link and control a
reader taps is at least 24px (44px in a drawer), a wide table or code block
scrolls inside its own box, and no page scrolls sideways at any width from
320px to 1920px, in English or Arabic — each checked on every route in a
real browser. The search index is its own file, fetched the first time the
search box is used, instead of 20 kB carried by every page.

## Editors

- **Zed**: the grammar reads WebFluent 4 — `api`, connections, `validate`,
  refined types, the language's literals — where it used to mark them as
  errors, and a call with a bare-name argument (`remove(i)`) no longer
  fails to parse. It is held to every example in the guide. A string that
  begins `"/// …"` — a sample of code — is text, not a doc comment.
- **Neovim, Helix and Emacs**: the guide's CLI chapter has the configuration
  for each.
- **VS Code 0.3.0**: attached to this release as a `.vsix`. It is bundled —
  the 0.2.0 package could not start — finds `wf-lsp.exe` on Windows, and
  downloads the server from the latest release when it is not on `PATH`.

# WebFluent v4.0.1 Release Notes

## Fixed

### A translation paints wherever it is written

The static paint resolved `t(…)` only when it was the whole of a text.
`Text(t("name"))` painted; `"Hello, {t("name")}"`, `"{n} " + t("items")`,
an `if` on a translation, and a translated prop that a component spliced
into its own text all painted **empty**, and stayed empty until the page's
script ran — so a reader without JavaScript, and a crawler, saw nothing
there. A static or server-rendered page with more than one locale almost
certainly had some.

The build-time evaluator now answers `t("key")` and `t("key", { … })`
wherever they appear, from the default locale, with the same plural
picking, placeholder filling and fallback to the key that `Text(t(…))`
always had. Nothing to change in your source: rebuild, and the text is in
the HTML.

### The documentation site

The Security chapter was missing from the navigation and the two chapters
after it were numbered one short; the Styling chapter was behind the guide
by five rows of the design-token table; an Arabic reader saw each page's
breadcrumb and prev/next titles in English; and the header read 3.0. The
site is now held to the guide by two tests, so its pages cannot fall behind
the chapters they are generated from again.

# WebFluent v4.0 Release Notes

Thirteen things the language could not do, or did badly. The runtime is
now modular and a static page carries **3.7 kB gzipped** of it where it
used to carry 18.3 kB.

## Breaking

Read this section before upgrading. `wf migrate` does the mechanical part
and names the rest with the file and the line — run it first:

```bash
wf migrate .
```

### `env` splits in two

`env.NAME` is replaced at build time and the value lands in the bundle, so
only a **public** name may be read from a page, a component, a store or an
`api` block: one beginning `PUBLIC_`, or one the new `public_env` list
names. Any other name is a compile error at the line that reads it; the
values still reach `wf render`, which runs on a server.

`wf migrate` adds every name your pages already read to `public_env`, so
the build keeps working, and prints the list — **read it**, because an API
key in there was a leak before and is a leak now.

### An `on*` attribute is a compile error

`Button("x", onclick: expr)` is the one place an attribute value is
executed. Write `Button("x") { on click { … } }`. No automatic rewrite: the
old value was a string of JavaScript.

### A URL a browser would run is refused

`href:`, `src:`, `to:`, `poster:` and `navigate()` take `http`, `https`,
`mailto`, `tel`, `sms`, `ftp` or a relative URL. A literal is refused where
it is written; a value that only exists at run time is refused at the
moment it is used. `javascript:`, `data:`, `vbscript:`, `blob:` and
`file:` have no replacement.

### Stores are built on first read

Nothing is constructed at boot. The order stores are declared in no longer
matters — a `derived` that read a store declared below it used to read
`undefined` — and a store nothing reads costs nothing. **If a `derived` of
yours had a side effect that ran at boot, it now runs when something reads
it**; `store X(eager: true)` restores the old timing.

### A `persist` value follows the site's other tabs

`localStorage` is shared between tabs, so a write in one now arrives in
the others. `sync: false` keeps a value to its own tab.

### The runtime surface

`WF.store(def)` is `WF.store(name, define, options)`, and `WF.host(…)` —
which was a `Url`'s host — no longer builds a `Host`; that is `WF.attach`.
Only a hand-written script is affected.

### Smaller

- `wf init` writes `"csp": true`. An existing project opts in.
- The guide renumbered: the components reference is chapter 20 and the
  cookbook 21, with the new security chapter at 19.
- A layout no longer reflows on its own below 768px — that landed in the
  responsive work; `.stacks` is how a `Row`, `Grid` or `Column` asks to,
  and `wf migrate` adds it.

## Added

### The runtime is modular

28 feature modules, and a build carries only the ones its program reaches
— read from the bundle it just wrote, so the set cannot fall behind the
compiler. `wf build --stats` prints what the output weighs and which
modules it holds. `build.budget` warns over a gzipped size;
`build.runtime: "full"` ships everything for a program the scan cannot
see.

### Eleven types the language brings with it

`Date`, `Time`, `DateTime`, `Duration`, `Money`, `Url`, `Email`, `Color`,
`Uuid`, `File`, `Secret` — each a plain JSON value at run time, with
`@2026-03-14`, `3.days` and `€12.99` as literals, arithmetic that runs at
build time as well as in the browser, and refinements (`Number(1..=30)`,
`String(minLength: 8)`) checked at every literal. `uuid()` is where a
`Uuid` comes from.

### A service, described once

`api Backend(base:) { … get users(page: Number) -> [User] }` — or
`api B from "openapi.json"` — compiles to one object of typed, cached,
cancellable endpoints: typed errors, retry with backoff, SWR caching with
`ETag` revalidation, dedupe, upload progress, pagination. `socket`,
`stream` and `channel` for a connection the page holds open, closed by the
scope that opened it.

### Validation beside the state it guards

`validate email { required  email  async "Taken" { … } }`. The control
bound to that state shows the message without being told to, with the
`role="alert"` / `aria-invalid` / `aria-describedby` plumbing. `Form(bind:)`
gives `valid`, `errors`, `touched`, `pending`, `values`, `reset()`,
`submit()` and `apply(serverErrors)`.

### A real image pipeline

`image hero = "hero.jpg"` reads the file at build time and writes it at
every width a page asks for. `Image(hero, sizes:, placeholder: .blur)` is
a `<picture>` with `width`/`height` from the real file, so nothing shifts.
PDF and slides embed the picture rather than a grey rectangle.

### Responsive values, without JavaScript

`Grid(columns: { base: 1, md: 2, lg: 3 })` compiles to one class and one
media query per step, so the right layout is in the first paint. The
breakpoints are design tokens.

### Motion on the Web Animations API

An interrupted animation gives way to the one that replaced it; a spring
is sampled into a `linear()` easing. New: `on: .enterView`, `shared:` for
an element carried across a route change, `.expand` to the height of the
content, `count:` on a number, and `sequence { step(after: "120ms") { … } }`.
**Nothing the DOM must end up as depends on an animation finishing** — a
page that is not painting still opens the box, arrives at the number and
completes the route change.

### Stores: lifetime, location, and a way to see them

`store Cart(scope: .app | .session | .route)`, and a `persist` policy:
`in:`, `version:`, `sync:` and `migrate 1 -> 2 { … }`, so a value an older
build wrote is carried forward rather than discarded. `wf serve` gets a
**stores** panel: the tree, every action with the state on each side of
it, and a click to put one back.

### Security, as a subject the language takes seriously

Beside the breaking changes above: `Unsafe.Html(markup)` is the one door
for markup, `sanitize(html)` is an allow-list that runs at build time and
in the browser from the same list, `rel="noopener noreferrer"` comes with
any `target:`, `meta.integrity` emits SRI, and **the build reads its own
output back and holds it to the policy it ships**. `wf audit` prints every
`Unsafe.*`, every other origin, everything kept on the reader's machine,
every `env` name, the policy and the dependency list. A new chapter 19
covers the threat model and a deployment checklist.

### A test that acts

`wf test` ran a body through the template engine and read the result,
which meant no test in the language could exercise a button: a click runs
a handler, which runs an action, which changes a store, which repaints,
and a renderer does none of that.

A test with an interaction in it is now compiled into a real page,
served, and run in a headless browser — same compiler, same runtime, same
output a reader gets.

```wf
test "reporting an incident opens one more" {
    use IncidentStore
    state draft = ""
    Text("{IncidentStore.open} open")
    Input(bind: draft, label: "What happened")
    Button("Report") { on click { IncidentStore.report(draft) } }

    expect "2 open"
    type "Queue backing up" into "What happened"
    click "Report"
    expect "3 open"
}
```

`click "Save"`, `type "Ada" into "Name"`, `press "Escape" in "Search"` and
`expect` run in the order written — which is the whole meaning, since what
a click did is only visible in the expect that follows it. Everything is
found the way a reader finds it: a button's text, a control's label, its
placeholder, its `aria-label` — the same names the accessibility checks
hold a page to. A handler that throws fails the test even where the
expects would have passed.

A test that only looks is still rendered and snapshotted, and needs no
browser.

### `wf verify` — every page, in a real browser

```bash
wf build && wf verify
```

It starts a headless Chrome, serves the output and opens every route the
project has, failing on an uncaught exception, a `console.error`, a file
that did not arrive, an image that failed or a page that rendered no
text. It reports the first contentful paint, the node count and the bytes
each route fetched; `--budget MS` fails a route that paints too slowly
and `--json` is the same report for a pipeline. It also says which
built-ins no page drew — the classes the pages carried are the list of
what actually ran.

The DevTools Protocol client is written here, like the PDF writer and the
gzip encoder: it speaks to a process the build started, on the loopback
address, so it needs no TLS, no compression and no fragmentation, and a
dependency would be larger than the problem.

### Interop, in both directions

`external Chart from "…" { fn … type … }` is a typed import, checked at
every call site, emitted as a real ESM module with its origin in the
policy. `Host(mount:, update:, cleanup:)` is a node with a lifetime — the
`ref` + `effect` + `cleanup` written by hand, with the cleanup impossible
to forget. `external element` places somebody else's custom element, and
`output_type: "elements"` publishes yours: `PriceTag` becomes
`<price-tag>`, usable from React, Vue, Svelte, Rails, WordPress or a plain
page.

### Strings that do not fight you

Raw strings `#"…"#`, block strings `"""…"""` with the source's indentation
removed, and formatted splices `{total:.currency}`.

## Fixed

### A store's actions now speak the whole language

A store's action and a page's handler are compiled by two different
emitters, and each kept its own table of what a WebFluent method becomes in
JavaScript. The tables drifted. Inside a store action, `items.remove(i)`
compiled to `store.items.remove(i)`, `word.toUpper()` to
`store.word.toUpper()`, `items.contains(x)` to `store.items.contains(x)`,
and every one of the 39 scalar methods — `due.plus(days: 5)`,
`total.times(2)`, `site.host()` — to a method call on the value itself. A
`Date` is a string at run time and `Money` a map, so each of these threw
the first time its action ran. The page painted correctly until then, which
is why nothing caught them: the source parses, type-checks and lints clean,
and only the generated JavaScript is wrong.

There is one table now, read by both emitters, so a method added to one
cannot go missing from the other.

### Removing from a list no longer empties it

`items = items.remove(i)` on a page `state` compiled to a `set` inside a
`set`. The inner one removed the item; the outer one then set the list to
what `set` hands back, which is nothing. The item went, and the list went
with it. The removal now evaluates to the new list, so both the statement
`items.remove(i)` and the assignment are correct.

### The static paint knows WebFluent's own spellings

`toUpper`, `toLower` and `contains` were missing from the build-time
evaluator, which knew only `toUpperCase`, `toLowerCase` and `includes`. A
page that used them painted an empty element, and it stayed empty for a
reader without JavaScript and for a crawler.

- A `derived` that read a store declared later in the file read
  `undefined`.
- A page `state` named `key`, `value`, `event`, `e` or `params` compiled
  to a bare identifier: it rendered as nothing and never updated.
- `Input(bind: Store.member)` compiled to a control with no binding at
  all — a silent no-op.
- The static paint wrote `style="display:none"` for a hidden `show`, which
  a strict `style-src` forbids.
- The documentation site was painting `￾` and `￿` in inline code.
- `frame-ancestors` was shipped in a `<meta>` tag, where a browser ignores
  it.
- An `api` block's `on request` / `on response` / `on error` hooks
  compiled to empty functions: everything written in one was generated
  into a string and thrown away.
- A handler's parameter in an `api` hook and in a `socket`, `stream` or
  `channel` (`on message(m)`) compiled to `_m()` — a signal that was never
  made, so the handler threw on the first message.
- `match` over a `socket`, `stream` or `channel` called the handle as a
  function, which threw and took the whole page's first paint with it.
- `if let x = …` inside an action, a handler or an `effect` read the bound
  name as a signal, so the branch that had just checked it was not null
  used a name that did not exist.
- `T06` only held a store to its members where the body wrote `use`, and
  never inside another store or an `api` block — so `Store.mispelt()` in
  any of those built without a word.
- `U04` did not see the reads in an `api` block's `headers { }` or hooks,
  and reported the store members only those reach as unread.
- `form.pending`, `form.errors`, `form.touched` and `form.apply(…)` were
  `T05`: the checker's idea of a form handle was missing half of it.

## Coverage

Every page of every example — the fixtures, both `wf init` templates, the
documentation site and a real 130-file project — is loaded in a headless
Chrome on every change: **124 pages, zero errors**, and every one of the
52 built-ins that can render in a browser is drawn by at least one of
them. `just browser` runs the sweep; it reports nothing to do where there
is no browser.

It is also how a real bug in the dashboard fixture's own store was
found the moment it had a test that clicked: `resolve` changed a field of
an item in place, so the list stayed the same list and nothing repainted.

That sweep is how three of the fixes above were found: a route change
that never completed, a box that never opened and a number that never
moved, each of them waiting on an animation frame that a page which is
not painting never gets.

## Diagnostics

New codes: `P01`/`P02` (a persisted value's version chain), `P03` (a
route-scoped store that persists), `V03` (an `Unsafe.Html`, and whether it
is sanitised), `T12` (a `Secret` where it would escape).


# WebFluent v3.2.1 Release Notes

## Fixed

### A store's action is no longer mistaken for a list method

`Todos.remove(id)` called the action the store declares. The JavaScript
back end matched the method table on the name alone, without looking at
what it was called on, so it compiled to `Todos.splice(id, 1)` — a store
is an object of state and actions, never a list, so the handler threw
`Todos.splice is not a function` on the first click. The page painted
perfectly until then, which is why nothing caught it.

A store's own members now come first, so an action may be named after a
list method. The shadowing was not limited to `remove`: every name in the
table — `push`, `filter`, `map`, `take`, `contains`, `first`, `last` — hid
an action of the same name. `Todos.items.remove(0)` still reaches the
list, since that names the list rather than the store.

### `items.remove(i)` repaints

`remove` on a list held by `state`, `persist` or a store's member spliced
it in place, so the signal kept the array it already had: nothing reading
it repainted and a persisted list was never written. It now goes through
the signal, as `push` does, via the new `WF.removeAt(list, index)`.

### Navigation items drawn by a `for` are links in a static build

A `Sidebar.Item(to:)` reached through a loop pre-rendered as a bare `<li>`
with the destination dropped: the components reference on the site, whose
index is `for c in shown { if … { Sidebar.Item(to: …) } }`, shipped a
navigation panel with no links in it. The static back end read `to:` as a
literal only, so a destination that needs the loop's binding — `"/docs/{p}"`
— resolved to nothing. It now reads it through the build-time scope.

The published site was also rebuilt: its bundle still carried the
`<li to="…">` from before the 3.2.0 fix, so clicking a component in the
reference sidebar did nothing at all, silently.

### The install scripts download a file that exists

`install.sh` and `install.ps1` asked for Rust target triples —
`wf-$VERSION-x86_64-unknown-linux-gnu.tar.gz`,
`wf-$VERSION-x86_64-pc-windows-msvc.zip` — but the release publishes
`wf-<version>-<arch>-<os>`: `x86_64-linux`, `x86_64-macos`,
`aarch64-macos`, `x86_64-windows`. Every run downloaded a 404 page and
handed it to `tar`. `install.sh` also rejected macOS outright, despite
the release carrying both Mac builds, and appended its `PATH` line again
on every run.

Both now resolve the right asset, cover Apple Silicon and Intel Macs,
fail with a readable message instead of a corrupt unpack (`curl -f`,
unpacked through a temporary directory), and add the `PATH` line once.
Linux arm64, which has no prebuilt binary, is told to use Cargo.

`cargo install webfluent` is documented in the guide and the README,
which had been pointing at a `0.2.0-alpha` Debian package.

## Testing

`tests/cookbook.rs` extracts the three applications from
`md-docs/20-cookbook.md` — the guide itself, not a copy — builds them and
runs them against the fake DOM in `tests/js/cookbook.test.mjs`, clicking
through add, toggle, remove and clear. `tests/docs_parse.rs` now runs the
JavaScript back end over every guide block and rejects a bundle that calls
a store member the store does not declare, which parsing and type-checking
could not see. A navigation item inside a `for` or an `if` is held to being
a link in every backend.

# WebFluent v3.2 Release Notes

## Added

### The guide: `md-docs/`

A twenty-chapter developer guide, from `wf init` to three complete
applications: getting started, the language basics, pages and routing,
elements, state, events and forms, control flow, components, stores,
types, expressions, styling, motion, data, internationalisation, content,
outputs, tooling, a components reference generated from the registry
(`scripts/components-reference.py`), and a cookbook. Every code block is
parsed, semantically checked, type-checked and held to the linters by
`tests/docs_parse.rs`, and the cookbook's applications build and were
walked through in a browser. The site's documentation will be rewritten
from it.

### Syntax colouring

`Code(…, language: "wf")` — or `json`, `bash`, `css` — and a Markdown
fence in one of those languages are coloured, at build time by
`codegen::highlight` and in the browser by `WF.highlight`, the same
tokens either way: keywords, capitalised names, strings, numbers and
colours, `name:` props, `$tokens`, comments and a shell prompt, each a
`wf-tok-*` span taking its colour from the theme's `syntax-*` tokens. The
baseline theme now carries those tokens and `term-bg`/`term-ink`/`term-dim`,
and a code block is painted on `term-bg` — a console stays dark in both
themes — with `white-space: pre` and a horizontal scroll. `sun` and `moon`
join the icons; an `icon:` that reads state follows it.

### The documentation site, from the guide

`site/` is written from `md-docs/` by `scripts/site-from-guide.py`: one page
per chapter under `site/src/pages/guide/` (routes `/docs/guide/<chapter>`),
framed by `DocsShell` — the header with search over every chapter and
section, the chapter nav with its numbers, the article at a reading
measure, the headings rail and the links to the chapters either side —
with prose as `Markdown` (inline code, links and lists render), fences as
`CodeBlock` (coloured, with a Copy button), shell blocks as `Terminal` and
pipe tables as `Table`. The components reference is data: `wf registry
--json` becomes `site/src/registry.json` (`scripts/site-data.py`), an
index page and one page per built-in at `/docs/reference/<name>` with its
signature, props, flags and cases, events and parts under tabs, a filter
over the index, and the entries nearby. The landing page, the chapter
page, the reference and the phone layout follow the design canvas
"WebFluent — site and documentation" to the pixel, in the MO design
system: graphite for structure, blue for intent, cyan for data, Inter and
JetBrains Mono, a console that stays dark in both themes, a theme toggle,
light and dark, and Arabic chrome with the chapters kept left-to-right.

### The language

- A string splice takes any expression: `"{format(total, .currency)}"`,
  `"{[1, 2].length}"`, `"{a ?? "none"}"` and
  `"{if ok { "yes" } else { "no" }}"` — a splice may hold a string of its
  own, as long as the group closes on the line. A brace group that is not
  an expression (`"{key: value}"` in prose) stays text; one with a slip in
  it is an error that names the splice.
- `() => expr`: a lambda of no parameters, for `setTimeout(() => go(), 300)`.
- A record construction carries the fields it left out — each field's
  default, `null` for an optional — so `Todo(id: "a", title: "b")` holds
  `done: false` wherever it goes.
- `await fetch(url, options)` in an action or a handler is the parsed body,
  and a failed response throws (`WF.request`): the same request a
  `resource` makes. The browser's own is `window.fetch`.
- `items.push(x)` on a `state`, a `persist` or a store's member sets a new
  list, so what reads it repaints and a persisted one is written.
- `Modal(visible:)`/`Dialog(visible:)` take any expression that reads
  state — a store's member, a condition — and write a state or a store
  member back when the browser closes the dialog itself.
- A page's `guard:` and `redirect:` work: a route whose guard is false
  sends the reader to its redirect (default `/`) instead of painting.
- A page's own `meta` in `head { }` replaces the standard tag of the same
  `property` or `name`, so a post page has one `og:title`, its own.
- The registry lists the icons the runtime draws (`registry::ICONS`,
  pinned to the runtime's table); an `Icon("…")` or `icon:` that names
  another is a warning, and the language server completes from the same
  list.

## Fixed

- The static paint and the template engine bind an `if let` name: the
  branch used to read it as unknown and paint an empty heading.
- A store action's `try`/`catch`, `for` and `else if` used to fall through
  to the page emitter and read `_x()` signals a store does not have.
- `Form(bind: form)` declared the handle twice — as a form and as a ref —
  which the browser refused.
- A `Checkbox`, `Radio` or `Switch` drops nothing now: `aria-*`, `data-*`,
  `name`, `id` and `disabled` reach the `<input>`, and radios bound to one
  state share a `name` so the arrow keys move between them.
- `navigate("/?filter=done")` and a `Link(to:)` with a query string route
  by the path and keep the query readable through `query`; a link to the
  route with a query is the current one.
- The single-page shell addresses its assets from the site root, so a deep
  link served by the SPA fallback (`/deploys/8f2c`) loads `app.js` rather
  than the shell again.
- `WF._basePath` is a getter again; a static build's links no longer start
  with `undefined`.
- A bare `slot` followed by `on key(…)` on the next line names no slot
  `on`.
- The vocabulary lint knows a page's route parameters, and its `const` and
  `data` names.
- A page framed by a `layout:` is judged with the layout's outline: the
  `h1` a layout draws is the page's (`A12`), and a heading level skipped
  across the boundary is seen (`A11`).
- `Grid(columns:)` is a `data-cols` attribute the stylesheet reads — a
  build with `build.csp` on used to have its grids blocked as inline
  styles — and a `columns:` that reads state follows it.
- `wf serve` serves a site built with `build.base_path` under that path,
  as its host will, and sends `/` there.
- A Markdown link or image to a site-relative path takes the base path,
  at build time and in the browser.
- A `Breadcrumb.Item` without `to` paints as a `<span>`, not an `<li>`
  outside a list.
- The skip link hides by a transform, not an off-canvas offset that a
  right-to-left page could scroll to.
- A `:param` page's static files carry their own route as the canonical
  link and sharing URL, not the pattern with `:slug` in it.
- An `IconButton` with a `class:` keeps the engine's classes; its `icon:`,
  `label:` and title follow state.
- A `Sidebar.Item` or `Breadcrumb.Item` with a `to:` reached through a
  `for` or an `if` is a link in the browser too; it used to be a bare
  `<li to="…">` that went nowhere.

---

# WebFluent v3.1 Release Notes

## Added

### The language core

- `for` in actions, handlers and effects — with an index, over a range
  (`1..10`, `1..=10`), or a keyed list; `try { } catch e { }`;
  destructuring `let { a, b } = m` and `let [x, y] = l`.
- Optional chaining `a?.b`, `a?.m()`, `a?.[i]`: null when the base is,
  the rest of the chain with it, typed as `T?`.
- `if let x = v { a } else { b }` as a value.
- Regular expressions `/pattern/flags`, with `test`, `exec`, and a string's
  `match`, `replace`, `replaceAll`, `split`, `search`; a `/` after an
  operand still divides.
- Spread `[...a, b]`, `{ ...m, k: v }`.
- `const NAME = value` at the top level, and `env.X` from the config's
  `"env"` map, fixed at build time.
- Record composition `type Admin = User { role: String }`; enums whose
  cases carry a payload, `enum Status { idle, failed(reason: String) }`,
  written `.failed("x")` and matched `.failed(r) { … }` (a match
  expression binds one name); `["failed", "x"]` at run time, a bare case
  its name.
- Structural shapes: a map literal is typed by the fields it was written
  with, so `form.nam` is a `T05` and a literal given where a record is
  wanted is checked field by field; a field only some items of a list have
  is `T?`.
- Helpers `sortBy`, `groupBy`, `unique`, `take`, `first`, `last`,
  `flatMap`, `sum` on lists and `capitalize`, `truncate` on strings —
  in the browser and at build time.
- `format(value, .style, option)` — `.number`, `.integer`, `.decimal`,
  `.currency`, `.percent`, `.compact`, `.date`, `.time`, `.datetime`,
  `.relative`, or a date pattern such as `"yyyy-MM-dd HH:mm"` — and
  `ago(date)`, in the i18n locale when the project has one and the
  document's language otherwise; the static paint and the template engine
  format from a table of the common locales and currencies.
- The type checker places an error at the expression it names when the
  source is at hand; hover shows a record's inherited fields and an enum's
  payloads; the tree-sitter grammar and the Zed extension know every new
  form.

### Tooling

- `wf fmt` formats a project's sources: indentation by block depth,
  spacing around braces, trailing blanks and blank-line runs — held to the
  file's tokens, comments kept; `wf fmt --check` for CI. `--to wfx|wf`
  still changes the layout.
- `wf serve` rebuilds on save, reloads the page, and shows a failed build's
  diagnostics over the page until the next build passes.
- Plural forms: `t("items", { count: n })` picks `items.one`, `items.other`
  and the locale's other categories from the translation file.
- A route change with `Router(transition:)` plays through the View
  Transitions API where the browser has it, the class-based animation
  standing in elsewhere.
- `wf docs`: a self-contained component gallery of every built-in and of
  the project's own declarations.
- `Markdown(text)`: a small Markdown rendered the same way at build time
  and by the runtime, the text escaped first; a `.md` file under `src/` is
  a page, its front matter the page's attributes.
- `data posts = "posts.json"`: a JSON file as a constant, read at build
  time; `paths:` on a `:param` page renders the static page once per value
  and lists each in the sitemap.
- `wf test`: `test "name"(data: { … }) { elements  expect "text"  expect
  not "text" }` declarations under `tests/`, rendered through the template
  engine with the project's declarations at hand, held to their
  expectations and to snapshots (`--update` accepts a new render).
- The language server renames — a component, a store, a store member, a
  local — across the project, inside interpolations too, and offers
  `Extract component` on a selection of elements, with what they read as
  typed props and the stores they use `use`d again.
- Unused warnings `U01`–`U05`: state, derived values, actions, components
  and store members the program declares and never reads, in the build's
  output and the editor.

### Components and reactivity

- Scoped slots: `slot row(item: Todo, index: Number)`, used as
  `row(item: it, index: i)`, filled as `row(t, i) { … }`; the fill is drawn
  again when a value it is handed changes.
- Parts of a component: `part Header(_ text: String) { … }` inside
  `component Panel`, called as `Panel.Header("…")`.
- A component's enum props are written to its root element as
  `data-<prop>="<case>"`.
- `ref: name` on an element: a handle read as the element itself
  (`name.focus()`).
- `every(ms) { }`, `after(ms) { }` and `effect { … cleanup { … } }`; what a
  page, a branch, a list item, a match arm or a slot creates is disposed of
  when it leaves.
- `persist name = value` in a page, a component or a store: kept in the
  browser's storage across visits.
- `viewport`, `query`, `hash` and `theme`: the browser as values, kept
  current; a declared name shadows them.
- `on key("ctrl+k") { }` on an element or at the top of a page.
- `animation Name { from { } to { } }` keyframes, played with `animate:
  .Name` or written into a style; container queries pass through.
- Dark mode: `"theme": { "dark": "Night" }` applies a second theme under
  `prefers-color-scheme: dark` and on `setTheme("dark")`, kept across
  visits.
- `Form(bind: form)`: `form.valid`, `form.values`, `form.reset()`; an async
  action's `name.pending`.
- A page's `head { meta(…) link(…) script(…) }`, painted at build time and
  kept current on the live page.
- The template engine binds a component's positional props and defaults,
  and every component is registered before a page renders; the static
  paint substitutes props through every expression and into branches,
  loops and matches.

### The indented layout: `.wfx`

The one grammar, written by indentation. A `.wfx` file opens a block with
a line indented deeper than the one before it and closes it with a dedent;
inside parentheses, brackets and any braces you write, the layout is free,
so a multi-line argument list, a map literal and a one-line
`{ on click { save() } }` read as they always did. Nothing else changes:
the parser, the checks, the output, the language server, the template
engine and `wf generate` read and write both layouts, and a project may
hold both.

`wf fmt --to wfx` rewrites a project's `.wf` files as `.wfx`, and `--to wf`
the other way — a change of layout and nothing else, held to a
byte-identical build across every fixture, the documentation site, every
`wf init` template and Halyard, and refused for a file whose indentation
does not already follow its braces. `wf migrate --wfx` migrates a
WebFluent 2 project straight into the indented layout.

The Zed extension gains a second grammar and language for `.wfx`,
generated from the same `grammar.js`, with its own indent queries; the
VS Code extension opens `.wfx` files. Specified in `spec/SYNTAX_V2.md`
§8a.

### For tools: `wf registry --json`, `wf types --json`, and edits by name

The registry as JSON — every built-in with its positional prop, props and
their cases, flags, events, slots, parts and attribute families, plus the
universal props — and a project's own declarations the same way: enums,
records, components with their props, events and slots, stores with their
members, pages with their routes. The structured edit operations speak the
grammar by name: `SetProp`/`RemoveProp`, `SetFlag`/`RemoveFlag`,
`SetHandler`/`RemoveHandler`, `SetSlot`/`RemoveSlot`, beside the older
`AddModifier`/`SetArg` that stay as aliases; two insertions at one place
land in the order they were asked for; a `.wfx` file is edited through its
braced spelling (`apply_edits_to`). WebFluent Studio reads both.

### `wf build` reads source files in name order

The order files are read in decides the numbering of a page's elements,
and it followed the file system's own order — so the same project could
build differently on another disk. It is alphabetical now, `App.wf` first.

---

# WebFluent v3.0 Release Notes

The grammar release. WebFluent's surface syntax grew by accretion until it
broke its own rules — a misspelled modifier did nothing, one `{ }` meant
five things, a style value was a string in which a bare word was a token
or a variable depending on a lookup, routes were declared twice, a project's
own components could not declare a variant, an event or a slot. WebFluent 3
is one language with one lexeme per meaning, a registry that describes every
built-in once, a type checker, motion on any element, and `wf migrate`,
which rewrites a project mechanically: every fixture, the documentation
site, every `wf init` template and the Halyard demo (129 files) migrate to a
**byte-identical build**.

**This release is a hard cut.** `wf build` and the language server read only
the new grammar; a file in the old one is refused with a pointer to
`wf migrate`. See *Migrating* at the end.

---

## The grammar

```wf
enum Tone { neutral, info, danger }
type Todo { id: String, title: String, done: Bool = false, tone: Tone = .neutral }

/// A todo, as one row.
component TodoRow(_ label: String, todo: Todo, compact: Bool = false) {
    event toggle(id: String)
    slot trailing
    Row(align: .center, gap: .sm) {
        style {
            padding: 6px 0
            background: $surface
            --accent: {todo.color}
            &:hover { background: $surface-hover }
        }
        transition { background: 150ms ease-out }
        on click { emit toggle(todo.id) }
        Text(label).bold
        trailing
    }
}

page Home(path: "/", title: "Todos", layout: AppShell(crumb: "Todos")) {
    use Todos
    resource list = fetch("/api/todos")
    match list {
        loading { Spinner }
        error(e) { Alert(e.message).danger }
        ready(v) {
            for t in v by t.id {
                TodoRow(t.title, todo: t, exit: .fadeOut).fadeIn {
                    on toggle(id) { Todos.toggle(id) }
                    trailing { Badge("new").info }
                }
            }
        }
    }
    Button("Add").primary.lg { on click { Todos.add(draft) } }
}

app { Navbar(brand: "Todos") { Navbar.Links { Link("Home", to: "/") } }  Router(transition: .fade) }
```

- **Lowercase declarations**: `page`, `component`, `store`, `theme`, `app`,
  and the new `type` and `enum`. There are no keyword tokens: `type:` and
  `on = !on` are what they look like.
- **Flags**: `.primary.lg` after the parentheses — a `Bool` prop or a case
  exactly one enum prop has; a case two props share is named,
  `tone: .info`. A word nothing declares is an error that lists what the
  component takes. `Input(…).text`, `Heading("x").h1`, `Spacer.sm`.
- **One positional argument**, the component's declared positional prop,
  then named ones. An enum prop takes a case: `Row(gap: .md)`.
- **Parts under their owner**: `Table.Head`/`Body`/`Row`/`Cell`,
  `Select.Option("Label", value: "v")`, `Tabs.Page`, `Card.Header`.
- **One block, in order**: `style`, `transition`, `on` handlers, slot fills,
  children. `on click { }` and `on click(e) { }`; a `Button`'s block holds
  what it shows, `on click` what it does.
- **Style values are CSS**: `padding: 1rem 2rem`, `background: $surface`,
  `width: {pct}%`, `&:hover { }`, `@media (…) { }`, `--name: value`. A
  `$token` is validated against the theme; `$md` on `padding` is
  `$spacing-md`.
- **Components declare** typed props (`_ name: String`, `active: Bool =
  false`), `event`s fired with `emit`, and `slot`s filled by name.
- **Pages own their routes**: `Router` takes no children; routes match by
  specificity. `layout: Shell(args)` wraps a page in a component with a
  default slot.
- **`resource` and `match`** replace the `fetch` block; `for … by key` keys a
  list; `if let x = expr { }` unwraps; `??`, `T?`, `[T]`, `null`, `await`
  in actions and handlers, `if` and `match` as values, `.case`, `$token` in
  expressions.
- `Store.load(id)` at the top of a page is set-up code; `let x` is a local
  in a handler or action.

The full grammar, with why it changed and the migration table, is
`spec/SYNTAX_V2.md`.

## Added

### `wf migrate`

`wf migrate [path] [--check] [--stdout]` rewrites every `.wf` under `src/`
in place — spelling only: the migrated project builds to what it built
before. What needs a decision is reported as a note. `--check` reports
without writing; `--stdout` prints one file's result.

### The registry

`src/registry` describes every built-in once — its positional prop, props
with their types and enum cases, flags, events, slots, parts, attribute
families — and the compiler's checks, the linters, the language server's
hover and completion, the tree-sitter vocabulary check and the tests all
read it. The `secondary` tone reaches `Badge`, `Text` and `Icon`;
`Spinner` has sizes and no tone.

### The type checker

Gradual and structural: everything it cannot resolve is `Any`, which agrees
with everything, so every existing project checks without an annotation,
and each annotation — `state x: [Todo]`, a prop's type, a `type`'s fields,
`resource x: [Deploy]` — narrows what it can say. It reports, as errors with
hints: a value of the wrong type (`T01`), a case an enum lacks (`T02`), a
value that may be `null` read without `if let`, `??` or a `!= null` check
(`T04`), a field or method a record lacks (`T05`), a member a store lacks
(`T06`), a list or record as a condition (`T07`), a `for` over a non-list
(`T08`), an `emit` that does not match its event (`T09`), a call with the
wrong arguments (`T10`), a `match` with nothing to match (`T11`), and a
`bind:` on a control of the wrong kind. Specified in `spec/TYPES.md`.

### Motion

The universal props — an animation as a flag (`.fadeIn`), `exit:`,
`.fast`/`.slow`, `duration:`, `delay:`, `easing:`, `stagger:` — on **any**
element, built-in or the project's own. On a branch's root they are the
branch's; anywhere else the element plays its own exit before it is
removed, wherever it was. A keyed list slides a moved item from where it
was. `Router(transition: .fade | .slide, duration:)` plays one page out and
the next in. `transition { }` blocks take `$token` timings. `prefers-reduced-motion`
is honoured by the runtime as well as the stylesheet: no class, no wait.

### Compiler diagnostics

Errors now: a parse error, a reference to nothing, a flag, case, part,
prop, event, slot or layout nothing declares, an ambiguous flag, a
positional argument out of place, a `match` arm the value cannot take, a
loose statement in a render block, a clause out of order, the type
checker's findings, and a file in the old grammar. `V01` names the flag you
meant (`` did you mean `.center`? ``); a named argument a built-in does not
declare is a warning and is written as an attribute.

### Tooling

- **Language server**: hover and completion are driven by the registry and
  the type checker — flags after `.`, cases after `tone: .`, `$tokens` in a
  style value, events after `on`, a component's events after `emit`, the
  arms a `match` still lacks, a component's slots, `layout:` candidates,
  props not yet written, a record's fields and a list's methods after a
  typed value; hover shows a flag as the prop it sets and a name's inferred
  type; go to definition follows `layout:`.
- **Editors**: the tree-sitter grammar is rewritten for the new grammar (a
  small external scanner reads style values), with the Zed queries and
  snippets and the VS Code TextMate grammar; every `.wf` in the repository
  and in Halyard parses without an error node.
- **Structured edits** (`EditOp`) write the grammar of the file they edit:
  `AddModifier` appends a flag, `SetStyle` writes a raw value.
- The documentation's `wf` blocks are parsed by the test suite
  (`tests/docs_parse.rs`), so an example cannot go stale.

## Breaking changes

- **The old grammar is refused.** `wf build`, `wf serve`, the template
  engine, the language server and the Node binding read WebFluent 3 only.
  Run `wf migrate`.
- **A `Button`'s block of statements is no longer its click handler**;
  write `on click { }`. The migrator does.
- **A word nothing declares is an error**, where it used to be the `V01`
  warning; `Alert(elevated)` and its `V02` are gone with the modifier
  words.
- **The runtime's public surface is renamed** for what each thing does:
  `h`→`el`, `condRender`→`when`, `listRender`→`each`, `showRender`→`show`,
  `wfFetch`→`fetch` beside the new `resource`, `createStore`→`store`,
  `createRouter`→`router`, `definePage`→`page`, `showToast`→`toast`,
  `bindDialog`→`dialog`, `bindPopup`→`popup`, `tablist`→`tabs`,
  `offCanvas`→`drawer`; new `match`, `mark`, `emit`. The compiler and the
  runtime ship together; nothing outside a build's own bundle should have
  named these.
- **`t("key", name: value)` never parsed**; interpolation takes a map:
  `t("key", { name: value })`. The documentation said otherwise.
- **Library**: `webfluent::parse_source(src, file)` is the one way in;
  `Lexer`/`Parser` of the old grammar are `#[doc(hidden)]` behind
  `wf migrate`. `Expr::Record` is a new expression; `sema::check`,
  `sema::types::check` and `sema::lower` run between parsing and codegen,
  and the linters see the lowered program.

## Fixed

- A list was rebuilt whole on every change, losing focus, scroll and
  animation state in items that had not changed; `for … by key` keeps the
  nodes of the items that stay.
- The route table depended on source order; it is now ordered by
  specificity, then path, whether the `Router` had children or not.
- `easing:` on an element was accepted and ignored; it sets the timing
  function, and `spring`, `smooth`, `bouncy`, `easeOut` and the rest mean
  the same there as in a `transition` block.
- A multi-line call that gained motion props from a migrated `animate(…)`
  clause had them written on the closing parenthesis's line; they take a
  line of their own.
- Lint hints spoke the old grammar (`Heading("Page Title", h1)`).

## Migrating

1. **Install 3.0 and run `wf migrate`** in the project. Read the notes it
   prints: each names a file and a line where the migrator could not decide
   — a modifier word no spelling exists for, an imperative `fetch` to
   rewrite as `let x = await fetch(…)`. Commit the result; `wf build` is
   byte-identical for a project that needed no notes.
2. **Expect errors that were warnings.** A misspelled flag, a prop a
   component does not declare, a case an enum lacks: the build stops and
   says what it takes.
3. **Annotate what you want checked.** Nothing is required; `state items:
   [Todo] = []` and `resource r: [Deploy] = fetch(…)` are where the type
   checker starts to see.
4. **If anything of yours named `WF.*` runtime functions** — a custom
   stylesheet does not, a `script` in `index.html` might — rename per the
   table above.
5. **Zed users**: the extension's 2.0 pins the new grammar; the old one
   highlights nothing useful on migrated files.

---

# WebFluent v2.2 Release Notes

A release about what a built site is like to use and to load. The Halyard
demo (https://github.com/monzeromer-lab/Halyard) was put in front of
Lighthouse and a screen reader, and everything either one found that the
compiler could have prevented, it now prevents; the same site also asked
for stylesheets of its own. Halyard's pages went from 63–71 to 87–99 on
mobile performance and to 100 on accessibility, with every finding that
remained pointed at by `wf build`. Nothing in this release is breaking.

---

## Added

### Your own stylesheets

Any `.css` file under `src/` is part of the build: read in path order,
bundled into `styles.css` after the engine's rules and before the rules
`style { }` blocks compile to, minified with the rest. `var(--token)`
reaches every design token. An element names a rule with `class:`, which
adds to the engine's classes rather than replacing them, and follows state:

```wf
Card(class: "feature feature--first") { … }
Heading("Forms", h1, class: if loud { "hero--loud" } else { "" })
```

`V02` accepts a modifier class one of these files defines
(`.wf-alert--elevated` makes `Alert(elevated)` real), in the build and in
the editor.

### Form fields

`Input` and `Select` with `label:`, `hint:` or `error:` become a field: the
label is a `<label for>` the control, the hint and the error are linked by
`aria-describedby`, the error (a string, empty when there is none) is a
`role="alert"` hidden until it has text and sets `aria-invalid`. The static
paint and the client agree, so hydration adopts it.

```wf
Input(text, bind: name, label: "Name", hint: "As on your passport", error: nameError)
```

### Accessibility in the runtime

- A route change settles the reader on the new page: focus moves to its
  heading (or the main landmark), the viewport returns to the top, the title
  is announced in a polite live region.
- `Carousel` is a labelled region whose slides are groups; it has named
  previous/next/dot controls, a pause button, stops for reduced motion and
  hidden tabs, and hides the slides that are not showing from assistive
  technology.
- `Tooltip` opens on focus as well as hover, closes on Escape, and describes
  its trigger with `aria-describedby`.
- `Dropdown` menus follow the ARIA menu pattern: `role="menu"` with
  `menuitem`s, arrow keys, Home/End, Escape, and focus returned to the
  trigger.

### Lint rules

- `A14` — a `role:` that requires particular children (`tablist`, `menu`,
  `list`, `radiogroup`, …) holding something else. A user component is
  judged by its root element, and a role given through a prop by what the
  call passed or the prop's default.
- `A15` — an `aria-label` that does not contain the control's visible text:
  its label, the literals in its block, and what a component in the block
  renders from its literal props (`Kbd(text: "⌘K")` in a search button).
- Every element rule now points at the element's own line and column.

### Build output

- `build.minify` is honoured at last: the bundle and the sheets lose their
  comments and whitespace, and nothing else. Halyard's `app.js`: 691 KB →
  212 KB.
- `build.split` (on by default) writes each page as `pages/<Name>.js`,
  linked beside `app.js` by a static page and fetched by the router the
  first time its route shows in a single-page build.
- The stylesheet carries only the component rules the project uses.
- Literal `style { }` declarations compile to shared classes instead of
  inline styles — one rule for a hundred identical cards, on the static
  paint before any JavaScript runs.
- The rules only one page can reach ship in `pages/<Name>.css`, linked by
  that page and loaded by the router before it draws; what the `App` body
  or two pages share stays in `styles.css`. Halyard: 144 KB → 49 KB shared.
- `build.compress` (on by default) writes `<file>.gz` beside every text
  output over a kilobyte, with a DEFLATE encoder of the engine's own —
  within a percent of zlib's default level, no dependency — for hosts that
  serve a precompressed file when they have one. `wf serve` sends it, and
  compresses on the way out what has none, so a Lighthouse run against the
  dev server measures what a deployed site sends.
- The static paint evaluates derived values and pure actions at build time,
  so a seeded list is painted with its rows rather than left to hydration.
- Images decode asynchronously; the first on a page is eager with
  `fetchpriority="high"`, the rest lazy, unless `loading:` says otherwise.

### Editor support

The Zed extension and the Tree-sitter grammar were rebuilt from scratch and
live in this repository (`editors/zed`, `editors/tree-sitter-webfluent`).
The language server reads the whole project the way `wf build` does — a
store declared in another file resolves, a component called across files
is known — and reports positions in the units each client expects.

## Fixed

- `class:` as an attribute replaced the engine's classes; it adds to them.
- `A14` reported a component whose root took its role from a prop as a
  plain control.
- `wf serve` read a query string as part of a file name.
- The language server matched a document to its project by URL text rather
  than by path, and could miss a file with an escaped character in its path.

---

# WebFluent v2.1 Release Notes

A release driven by building a real application. The Halyard demo
(https://github.com/monzeromer-lab/Halyard) — a nineteen-screen product design
implemented entirely in `.wf` — found every place where the language did less
than its documentation promised, and a handful of things a design system needs
that it had no way to say. Each is fixed or added here, with a test. Nothing in
this release is breaking.

---

## Added

### State and media in `style { }`

A style block may now hold blocks for how an element looks while hovered,
focused, active, disabled, or in an ARIA state, beside its `@media` blocks:

```wf
Button("Save") {
    style {
        background: "var(--brand)"
        hover { background: "var(--brand-hover)" }
        focus { outline: "2px solid var(--focus-ring)" }
        pressed { color: "var(--text-accent)" }
        @media (max-width: 768px) { width: "100%" }
    }
}
```

States: `hover focus active disabled placeholder focus-within current pressed
selected checked expanded invalid`. The ARIA ones key off the element's own
`aria-*` attribute, so state and paint cannot disagree. These compile into
`styles.css` under a content-hashed class — one rule per distinct block, shared
by every element that writes it, and nothing injected at run time (CSP-safe).
`@media` blocks, which used to become a run-time `<style>`, take the same path.

### Attributes by name

A hyphenated named argument is an attribute: `Trow(aria-selected: isSelected)`,
`Badge("Ready", data-tone: "success")`. A value that reads state follows it, and
an `aria-*` value of `false` is kept as the string `"false"`, which is a real
ARIA state. A keyword after the hyphen is fine (`data-state:`).

### Custom properties and reactive style values

`style { --edge: color }` sets a custom property. Any style value that reads
state is now kept in step with it, so `style { height: "{pct}%" }` moves.

### Components

- A component's block reaches it as `children`, in both the SPA and SSG
  backends. `AppShell(crumb: "Logs") { … }` is one component.
- Props are live: `Chip(pressed: showErrors)` repaints when `showErrors`
  changes. Declared defaults reach the SPA.
- A handler written on a component call — the click shorthand or an explicit
  `on:…` block — attaches to the component's root element.

### Vocabulary and layout

- `Row`/`Stack`/`Grid` `gap: xs|sm|md|lg|xl`, `align:` and `justify:` (six
  values each) are real utility classes in both stylesheets.
- Modifiers `fluid circle multiple ordered xs sm md lg xl header`:
  `List(ordered)` is an `<ol>`, `Tcell("Build", header)` is a `<th scope=col>`
  wherever it is written.
- `Table(caption: "…")` gives the table a visually hidden caption; a `Tcell`
  inside `Thead` is a `<th>`.
- `Link(to: "/docs", active: "prefix")`: links get `.active` and
  `aria-current="page"` from the router, on the static paint too.
- `Option("value", "Label")` submits the value and shows the label.
- Theme token and property names may have digit-leading segments
  (`viz-1`, `radius-2xl`).

### Config

`meta.fonts` and `meta.stylesheets` add `<link rel="preconnect">` and
`<link rel="stylesheet">` to both HTML shells; the CSP policy widens for those
origins.

### Language

Two-parameter lambdas `(a, b) => expr`; a keyword after a dot (`Array.from`);
`Number`, `String`, `Boolean`, `Map` as globals; a map literal as an arrow body;
store `derived` values may read other derived values and actions.

## Fixed

- `Icon("home")` rendered the word "home"; only `icon:` drew the glyph.
- `IconButton` dropped every named argument but `icon` and `label`, and the
  static backends painted the label as visible text next to the glyph. It is
  the accessible name (`aria-label`, `title`) only, and the glyph is drawn once.
- `Progress(value: expr)` was painted once and never moved.
- Lambda parameters inside pages compiled as signal reads (`_x()`).
- An action parameter named like another action read the store member
  (`move(step)` beside `step`).
- `placeholder`, `disabled`, `src` and the other recognised attribute names
  were set once even when bound to state.
- A `<select>` bound to state always showed its first option: the value was
  applied before the options existed (runtime), and a slot inside a `Select`
  component was appended after the value (codegen).
- An `else if` chain with a final `else` skipped every branch.
- A `Slider` or `Input` with both `bind:` and `on:input` kept only one.
- A `Button`'s block that mixed content with an action ran the action at
  render time; the action statements are its click handler now.
- A sub-component dropped its style block, attributes and handlers in the SPA.
- `createStore` bound derived values before actions, so a derived value that
  called an action threw.
- The router never set `document.title`.
- `wf build` did not run the V01/V02 vocabulary lint or the semantic check;
  it does, with per-file locations.
- The catch-all page was written to a directory literally named `*`. It is
  `404.html` at the root, and it addresses its assets from the site root, so a
  dynamic route under a static build hydrates.
- `wf serve` did not serve a pre-rendered `<route>/index.html`.
- The heading-outline lints (A11, A12) could not see headings inside
  components; A03 did not accept `aria-labelledby`; A13 warned about a white
  label on `--color-primary` even when no `primary` variant was used.
- In structural mode the sheet stacked every `Row` under 768px; it no longer
  paints layout decisions the author did not make.

---

# WebFluent v2.0 Release Notes

A release about correctness. The language gained themes and SEO; the output
gained the accessibility, responsiveness and search metadata it should always
have had. Several long-standing bugs are fixed, and the tests that would have
caught them now exist.

**This release contains breaking changes.** See *Migrating* at the end.

---

## Breaking changes

### Themes are written in WebFluent

The four built-in palettes (`default`, `dark`, `minimal`, `brutalist`) are gone
from the compiler. A theme is source now:

```wf
Theme Brand {
    token color-primary: "#0F766E"
    token radius-md: "14px"
}
```

Declare one and it is used; declare several and select with
`"theme": { "name": "Brand" }`. Tokens you do not name keep their baseline value.

`dark`, `minimal` and `brutalist` ship as `examples/themes/*.wf` — copy one into
your `src/` and the existing `"name"` in your config keeps working. A config that
still names a removed palette **fails the build** with a message pointing at the
replacement, rather than silently rendering as the default.

`theme.mode` and `theme.extends` were declared in config and never read by
anything. Both are removed.

### `Modal` and `Dialog` are native `<dialog>` elements

They were `div`s with an `.open` class, which meant no focus trap, no inert
background, no Escape-to-close and no `aria-modal`. They now use `showModal()`,
so the browser supplies all four. If you styled `.wf-modal` directly, note that
it no longer positions or hides itself — the element does that — and the backdrop
is `::backdrop`.

### Library signature changes

- `generate_css(tokens)` / `generate_css_with(tokens, builtin)` take resolved
  tokens rather than a theme name. Use `themes::resolve_tokens(program, config)`.
- `render_page_html` / `render_page_html_studio` take a `SiteContext` instead of
  five separate parameters.
- `generate_html(config, program)` takes the program, for the page metadata.
- `Template::with_theme(name)` selects a `Theme` declared in the template rather
  than naming a built-in palette.

---

## Fixed

- **`for` loops produced bundles that crashed.** The loop bound a plain callback
  parameter but every reference to it was emitted as a signal read, so any
  non-empty list threw `ReferenceError` on first render. `wf init -t spa` shipped
  this.
- **Zero-width spaces in every static page.** The SSG and template renderers
  emitted U+200B after every closing tag — invisible in an editor, present in
  `textContent`, in copied text and in what a crawler indexed.
- **`Heading("x", h1)` did not produce an `<h1>`** under `wf build`. The static
  renderers switched the tag; the SPA codegen never did, and emitted a
  `wf-heading--h1` class no stylesheet defines.
- **`style { }` blocks were dropped on nineteen components** — every one handled
  by a dedicated emitter, because styling was applied after the dispatch that
  returned early.
- **The sidebar rendered transparent** — no background at all in structural mode,
  and surface-on-white behind a 1.23:1 border in the full one.
- **Navigation vanished on a phone.** The sidebar was `display: none` below 768px
  and navbar links below 480px, with no control to bring either back. Both are
  off-canvas now, with a toggle, a scrim, Escape-to-close and focus return.
- **Structural mode had no focus indicator at all**, and `.wf-input` /
  `.wf-select` removed the outline without replacing it visibly.
- **`theme.builtin: "structural"` never reached `wf build`** — only the studio
  honoured it.
- **The language server had not compiled for several releases.** Nothing in the
  repository built it; `cargo build` at the root builds the root package alone.
  Its completion list had drifted to 43 missing modifiers and eight invented ones.
- **`Link("Home", to: "/")` drew a spurious accessibility warning**, and both the
  PDF and slides scaffolds warned on their own first build.
- **PDF images ignored their `alt`** and drew `[Image]` regardless;
  `Progress(max: 0)` rendered a full bar instead of an empty one.

---

## Added

### Search and sharing

Set `meta.site_url` and every page gets a self-referencing absolute canonical,
Open Graph and Twitter card tags, `hreflang` alternates with `x-default`, and
JSON-LD (`WebSite`, `Organization`, `WebPage`/`Article`, `BreadcrumbList`). Builds
write `sitemap.xml` and `robots.txt`.

`Page` accepts `description`, `image`, `type` and a bare `noindex`.

Without `site_url`, tags that need an absolute URL are omitted rather than
guessed at — a relative canonical causes problems later.

### Accessibility

Tabs carry the full `tablist`/`tab`/`tabpanel` wiring with roving tabindex and
arrow keys. Dropdowns report `aria-expanded` and close on Escape. Toasts and
alerts are live regions. Every page has a `<main>` landmark and a skip link.
Animations respect `prefers-reduced-motion`. Pointer targets meet the WCAG 2.2
minimum of 24×24.

### Performance

The type and spacing scales are fluid (`clamp()`), so a page responds without a
media query — and without losing to an inline `style { font-size: … }`. Images
default to `loading="lazy"` and `decoding="async"`. The bundle loads with
`defer`. The default font stack no longer names webfonts nothing loads.

The static renderer now paints lists and conditionals over seeded data instead of
leaving `<!--wf-for-->` for the client.

### Security

`"build": { "csp": true }` emits a strict CSP and a `_headers` file. The dev
server sets `nosniff`, `Referrer-Policy` and `X-Frame-Options`.

### Diagnostics

`A13` (theme contrast, checked at compile time because themes are now source),
`S01`–`S04` (title, description, description length, duplicate route) and `V02`
(a real modifier whose class no stylesheet defines).

### For tooling

`CompiledSite` carries the resolved tokens, the declared themes with their spans,
flattened diagnostics, and the reason a theme failed to resolve.

---

## Migrating

1. **If your config names `dark`, `minimal` or `brutalist`:** copy the matching
   `examples/themes/*.wf` into your `src/`. Your existing `"theme": { "name": … }`
   then resolves to it. The build tells you this if you forget.
2. **Remove `theme.mode` and `theme.extends`** if present; they are ignored.
3. **If you styled `.wf-modal` or `.wf-dialog` directly**, re-check it against a
   native `<dialog>`.
4. **Add `meta.site_url`** to get canonical URLs, sharing cards and a sitemap.
   Nothing breaks without it; those tags are simply not emitted.
5. **Expect new warnings.** `S02` fires on every page with no description. None
   of them fails a build.
