# The project's own JavaScript — the plan

> Proposed 2026-10-02, for 4.2. Decisions taken so far are recorded at the end.
> Author: Monzer Omer · Date: 2026-10-02
>
> Every `wf-next` block is **proposed** syntax: it does not compile today, which
> is why those blocks are not fenced as `wf` — the documentation test holds
> those to the grammar that exists.

A `.css` file under `src/` is part of the build: nothing declares it, every
page has it, and `Card(class: "feature")` picks a rule up. A `.js` file has no
such path. The ways in today are:

- `head { script(src: "/x.js") }` — per page only (`app` may not have a
  `head`, `src/parser/v2.rs:1280`), and the tag is removed and re-added on
  every visit, so the script runs again each time (`runtime/modules/head.js`).
- `external X from "./x.js" { fn … }` — the file must be an ES module, every
  function is written down twice, once in JS and once in the declaration, and
  calls are namespaced (`X.fn()`).
- `Host(tag:, mount:, cleanup:)` — the right lifetime, but only on the ten tags
  `Host` takes, never on a `Card` or `Button` that already exists.

And a script that does load cannot follow the page: the router empties its
container on every route (`runtime/modules/router.js:100`), `hydrate` replaces
the pre-rendered DOM rather than adopting it (`runtime/modules/hydrate.js:2`),
and `if`/`for`/`show`/`match` make and drop nodes as state changes. Nothing
tells a script any of this happened — the runtime dispatches no events.

**The rule:** JavaScript gets what CSS has. A `.js` file under `src/` is part
of the project — a plain browser script, written as any `<script src>` file is
written, with no `export`, no `import`, no build step: the file that runs is
the file you wrote. What it declares at the top level is in scope by name,
checked by the compiler, and offered by the editor. A library on another
origin is a URL the project lists, loaded ahead of its own scripts, which use
it as any script uses a library — so `external` goes (§8).

---

## 1. Discovery: what a `.js` file brings into scope

```
src/
├── App.wf
├── tilt.js         function initTilt(node, opts) { … }
├── tilt.css        .tilt { transform-style: preserve-3d }
└── lib/
    └── format.js   const money = (n) => …
```

```wf-next
Card(class: "tilt", mount: (n) => initTilt(n, { max: 15 })) { … }
Text(money(total))
```

Every `.js` file under `src/` is found the way
`codegen::project_css::find_stylesheets` finds sheets: walked, sorted by path.
Each is a **classic script** — what a browser runs from `<script src>`. Every
classic script on a page shares one global scope, which is the scope the
compiled pages (`app.js`, `pages/*.js`, themselves classic scripts) run in, so
a name one declares at its top level is simply there for the page's code.

The names in scope are what the browser makes global:

| Written at the top level | In scope |
|---|---|
| `function f(…)` · `async function f(…)` | `f`, a function |
| `class C { … }` | `C`, called as `C(…)` from WebFluent — the codegen writes `new C(…)`, since the language has no `new` — returning an opaque `C` with the methods its body declares |
| `const x = …` · `let x` · `var x` | `x`; a function when the initialiser is an arrow or `function` expression |
| `window.x = …` · `globalThis.x = …` (at any depth) | `x` |

What is inside a function, a block or an IIFE stays private, as it does in the
browser — so a file that wants helpers of its own wraps them, and exposes what
it means to with `window.x =`.

**Names are global**, as components are. Two files declaring one name, or a
script name that is also a component, store, `const` or `api` the
program declares, is an error naming both places — the rule two components
with one name already follow (and in the browser the second `let`/`class`
would throw anyway). A name the language or the browser owns (`fetch`,
`format`, `navigate`, `window`, …) is an error too: the codegen relies on
those meaning what they mean.

**Modules are not this feature.** A file under `src/` with a top-level
`import` or `export` is reported — "this is an ES module; a script under
`src/` is a plain browser script" — rather than linked in a way that would
break. A module someone else publishes is listed by its URL (§8).

### The scanner

Finding top-level declarations needs a tokenizer that understands strings,
template literals (with `${}` nesting), comments, and the regex-versus-division
ambiguity, plus brace depth — nothing more: it reads the top level and class
bodies, never evaluates. It is hand-written, a few hundred lines, in the spirit
of the PDF writer and the gzip encoder, and held to a case table of real-world
shapes (a minified library, an IIFE that assigns `window.x`, a regex after
`)`, `return /x/`, a template literal holding braces). If real files defeat it,
`oxc_parser` is the fallback — a crate earns its place by the same test the
image codecs passed.

A file the scanner cannot read is reported with its line, and its names are
not in scope: the build never guesses at a declaration. It is still linked —
the browser may well read it fine.

**Files:** `src/project_js/mod.rs` (discovery, the name table),
`src/project_js/scan.rs` (the tokenizer), `src/project_js/jsdoc.rs` (§2).

## 2. Types: from nothing, or from JSDoc

The language's gradual model carries over unchanged. A function with nothing
said about it is `Any` — a function of any arguments returning `Any` — which
agrees with everything, so adding a file never breaks a build. Each thing the
file says narrows it:

```js
/**
 * Tilt an element toward the pointer.
 * @param {HTMLElement} node
 * @param {{ max?: number, glare?: boolean }} [opts]
 * @returns {{ destroy(): void }}
 */
function initTilt(node, opts = {}) { … }
```

| JSDoc | WebFluent |
|---|---|
| `string` · `number` · `boolean` | `String` · `Number` · `Bool` |
| `object` · `Object` · `{ a: T }` | `Map` (a record shape when the fields are written) |
| `T[]` · `Array<T>` | `[T]` |
| `?T` · `T \| null` · `T \| undefined` · `[name]` | `T?` / an optional parameter |
| `HTMLElement` · `Element` · `Node` · `*` · `any` · anything unknown | `Any` |
| `Promise<T>` | `T`, and the call is awaited where an action awaits it |
| `void` | no value |
| `(v: T) => R` · `function(T): R` | a function parameter — an action or a lambda may be passed (§6c) |

The doc comment's prose is the hover text, as `///` is for a component.
JSDoc is a comment, so the file stays exactly what the browser runs. (A
`.d.ts` beside the file is left for later: JSDoc says everything the table
above can use.)

The checker gains one table, `world.scripts`, in place of `world.externals`
(`src/sema/types.rs:432`); `is_known` (`types.rs:~2945`) consults it, so a
name a script declares is no longer `T13`, and a call to one is held to its
signature with the existing `T01`/`T10`.

## 3. The build: what is written and linked

- Each file is copied to `build/js/<path under src>` **as written** — not
  minified, not wrapped, not rewritten — and gzipped beside itself like every
  text output. What the browser runs is what is in `src/`, line for line, so
  a stack trace points at your own code.
- Each is linked `<script src="…" defer>`, in path order, **before** `app.js`,
  so its names exist by the time the page's code runs; `defer` keeps the order
  and lets the HTML parse first. Every page links them — the SPA shell and each
  static page (`codegen/ssg.rs`) — as every page links `styles.css`. A script
  nothing in the `.wf` source calls is still linked: it may be one that drives
  the DOM on its own (§5).
- The CSP check (`codegen/csp.rs:103`) passes them as same-origin scripts.
- `wf build --stats` lists them under their own heading; `build.budget` may
  name one.
- `wf serve` already watches `src/`, so a saved `.js` rebuilds and reloads.
- `wf types --json` lists each script's names with their types; `wf audit`
  lists the files and every URL the project loads; `wf docs` shows them.
- The static paint cannot run a script, so a call to one at build time is a
  value the paint does not know — the page paints what it can and hydration
  fills the rest, as with any browser value. `wf test` tests that act run in
  Chrome with the scripts linked; tests that only look do not run them.

**Files:** `src/cli/build.rs` (beside `project_css` at `:190`),
`src/codegen/html.rs`, `src/codegen/ssg.rs`, `src/codegen/js.rs` (`new` for a
script's class), `src/codegen/csp.rs`, `src/cli/{types,audit,docs}.rs`.

## 4. `mount:`, `update:`, `cleanup:` on every element

Knowing the functions does not keep up with the DOM. What does is `Host`'s
lifetime, given to every element, and to every component call (on its root
element, the way an `on click` on a component call already is):

```wf-next
Card(class: "tilt",
     mount: (n) => initTilt(n, { max: 15 }),
     update: (h) => h.setMax(max),
     cleanup: (h) => h.destroy()) { … }
```

`mount` runs when the element is made — on the first paint, after a route
change, when an `if` branch or a `for` item arrives — and `cleanup` when it
leaves; `update` runs again whenever the state it reads changes. It compiles
to the `WF.attach` call `Host` already emits (`codegen/js.rs:2926`), now for
any element. `Host` stays, for the bare node a library wants (a `canvas` a
chart draws into).

A component that declares a prop named `mount`, `update` or `cleanup` keeps
it; the universal one applies only where nothing declares the name.

**Files:** `src/registry/builtins.rs` (the universal props, beside `class`
and `ref` at `:380`), `src/codegen/js.rs` (`emit_host` generalised),
`src/sema/types.rs` (the three are functions of the node or the handle).

## 5. `wf:render` — for a script that queries the DOM itself

A classic script that wants to do its own `querySelectorAll` gets an event to
do it on. The runtime dispatches `wf:render` on `document` after `mount`,
after `hydrate` and after every route paint, with `{ route, params }` in
`detail`:

```js
document.addEventListener("wf:render", () => {
  for (const el of document.querySelectorAll(".tilt:not([data-tilt])")) initTilt(el);
});
```

It is one `dispatchEvent` in three places (`core.js:mount`,
`hydrate.js`, `router.js` paint). It does not fire for a branch or list item
arriving — that is what `mount:` is for, and the guide says so beside it.

## 6. Classes: on an element, from WebFluent and from a script

What exists today: `class:` adds classes beside the engine's and never
replaces them — `Card(class: "feature wide")`, or `Card(class: tone)` from
state. A value that reads state goes through `WF.classes`
(`runtime/modules/core.js:470`), which removes only the classes **it** added
last time, so a class a script put on with `classList.add` survives every
update WebFluent makes. On a component call `class:` is not forwarded — the
component decides its own markup, and takes a prop it places where it likes.

Four things are missing.

### 6a. A map and a list for `class:`

Two independent conditions today are an `if` that spells out every
combination of strings, and a map is painted as `"[object Object]"`. Both
forms join the string:

```wf-next
Text(t.title, class: { "is-done": t.done, "is-urgent": t.priority > 2 })
Card(class: ["feature", tone, if open { "is-open" }])
```

- **Map:** each key is a class, on while its value is true. Keys are quoted
  strings or bare words (`{ active: isActive }`); a key may hold several
  classes (`"is-on glow"`).
- **List:** each entry is a string, a map, or null; null, `false` and `""`
  are skipped, so `if c { "x" }` with no `else` is enough.
- Either form is flattened to the class list `WF.classes` already works on —
  same add-what-is-new, remove-what-it-added — so a class a script owns is
  still never touched. A form whose every part is a literal compiles to the
  `classList.add` a plain string does today (`codegen/js.rs:3075`).
- The static paint evaluates both, so the first paint carries the right
  classes and hydration changes nothing.
- The checker holds a map's values to `Bool` (`T01`) and a list's entries to
  `String`, a map, or null.

### 6b. A class a script puts on comes back after a rebuild

A node the router, an `if` or a `for` makes again is a new node: what a script
added to the old one is gone. That is §4's job — `mount:` runs on every node
made, so the script puts its classes back on each one:

```wf-next
Card(class: "tilt", mount: (n) => initTilt(n))
```

### 6c. A script reports; WebFluent owns the class

When a class depends on something only a script can see — scroll position, an
intersection, a library's own event — the pattern is that the script reports
and WebFluent decides. An action is a plain function at run time (a page's
`action setVisible` compiles to `function setVisible(v)`), so it is handed to
the script like any callback, and the class stays a `class:` the compiler can
read:

```wf-next
state visible = false
action setVisible(v: Bool) { visible = v }

Card(class: { "is-visible": visible }, mount: (n) => watchVisible(n, setVisible))
```

```js
// src/visible.js
/** @param {HTMLElement} node  @param {(v: boolean) => void} onChange */
function watchVisible(node, onChange) {
  const io = new IntersectionObserver(([e]) => onChange(e.isIntersecting));
  io.observe(node);
  return { destroy: () => io.disconnect() };
}
```

No new mechanism: this needs §1 (the script's name in scope), §4 (`mount:`)
and the JSDoc function type `(v: boolean) => void` read as a parameter that
takes an action (§2's table gains it). The guide teaches it as **the** way a
script changes how an element looks: one owner per class.

### 6d. The editor, and one check

- Completion inside `class: "…"`, in a map's keys and in a list's strings,
  from the classes the project's `.css` files define; hover shows the rule;
  go-to-definition opens the sheet at the selector (§7).
- A `class:` naming a class that begins `wf-` draws a warning (`V04`):
  those are the built-ins', and adding one brings another built-in's rules
  with it. A stylesheet targeting them stays fine — they are stable.
- No warning for a class nothing defines: a class may be a hook for a script.

**Files:** `src/codegen/js.rs` (`class:` forms), `src/codegen/builtin.rs`
(`class_arg`, `author_classes`), `src/codegen/ssg.rs` and the static evaluator
(the paint), `src/sema/types.rs` (the checks), `src/linter/` (the `wf-`
warning), `src/project_js/jsdoc.rs` (function types), `md-docs/15-styling.md`.

## 7. The editor

The LSP reads the same tables the compiler does, so most of this is wiring:

- **Scripts.** `Project` (`crates/wf-lsp/src/project.rs`) gains the name
  table beside `stylesheets`. Completion offers every script name, with its
  signature; hover shows the JSDoc prose and the signature; go-to-definition
  opens the `.js` file at the declaration; `T13` stops firing for them. A change
  to a `.js` file refreshes the project as a `.wf` change does.
- **Stylesheets.** The LSP already reads every `.css` under `src/`, but only so
  a flag does not draw `V02`. Inside `class: "…"` it now completes the class
  names those sheets define; hover shows the rule; go-to-definition opens the
  sheet at the selector. No warning for a class nothing defines: a class may
  be a hook for a script, not a style.

**Files:** `crates/wf-lsp/src/{project,completion,hover,definition}.rs`,
`crates/wf-lsp/src/backend.rs` (the watched-file globs).

## 8. Scripts from elsewhere: a URL in the config, and no `external`

A library from a CDN is not written in WebFluent and is not in the project, so
the project's own scripts are where it is used — exactly as on any page that
loads Chart.js with one tag and draws with it in the next:

```json
{ "meta": {
    "scripts": [
      "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js",
      { "src": "https://cdn.example.com/confetti.mjs", "module": true, "as": "Confetti" }
    ],
    "integrity": {
      "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js": "sha384-…"
    }
} }
```

```js
// src/charts.js
/** @param {HTMLCanvasElement} node  @param {object[]} rows */
function drawSales(node, rows) {
  return new Chart(node, { type: "bar", data: toData(rows) });
}
```

```wf-next
Host(tag: "canvas", mount: (n) => drawSales(n, rows), cleanup: (c) => c.destroy())
```

- **`meta.scripts`** sits beside `meta.stylesheets` and works the same way: a
  URL, or a site-relative path to a file in `public/`. Each is linked
  `<script src defer>` in the order listed, **before** the project's own
  scripts, so a library is there when the code that uses it runs.
- **Integrity** comes from `meta.integrity`, keyed by URL, as it already does
  for a stylesheet or a font — emitted with `crossorigin="anonymous"`, and a
  remote script without one draws a warning. The build never fetches the file.
- **The policy** widens `script-src` by exactly the origins listed, as
  `csp_policy` already does for `meta.fonts` and `meta.stylesheets`
  (`config/project.rs:593`).
- **An ES-module-only library** (`{ "module": true, "as": "Confetti" }`) is
  imported by a generated loader and its exports put on `globalThis.Confetti` —
  the `externals.js` mechanism that exists today, without the declaration.
  A UMD/IIFE build, which most CDNs serve, needs neither.
- **What the compiler sees.** It cannot read a remote file, so a library's own
  globals are not names in WebFluent — the `.wf` source calls the project's
  script (`drawSales`), which is typed by its JSDoc, and the script calls the
  library. A project that wants to call a library straight from `.wf` lists
  the names it provides — `{ "src": "…", "globals": ["Chart"] }` — and they
  are in scope as `Any`.

### Custom elements without `external element`

A web component a script defines (`<stripe-pricing-table>`) is placed with one
new built-in, `Element`, whose positional is the tag — which must contain a
hyphen, as every custom element's does:

```wf-next
Element("stripe-pricing-table", publishable-key: key, pricing-table-id: id) {
    on ready { loaded = true }
}
```

Named arguments are attributes (a value that reads state follows it), `on`
handlers are DOM events on the element, the block is its children, and
`class:`, `ref:` and `mount:` work as on anything else. It is what
`external element` compiled to, without declaring it first.

### What goes

`external` — `external X from …` and `external element` — is removed from the
grammar: `Declaration::External`, `world.externals`/`world.opaque` in the
checker, `externals_module`'s declaration path and the `wf types` section for
it. There are no users to protect (the 4.0 rule), so there is no deprecation
period. `wf migrate` carries a project across:

| Was | Becomes |
|---|---|
| `external X from "https://…" { fn f(…) … }` | `{ "src": "https://…", "module": true, "as": "X" }` in `meta.scripts`, its `integrity:` moved to `meta.integrity`; `X.f(…)` call sites keep working, typed `Any`, and migrate says which signatures were dropped |
| `external X from "./local.js" { … }` | the file moved under `src/`, its top-level `export`s taken off (enough for a module with no `import` to become a plain script; one that imports is reported), the declaration's signatures written into it as JSDoc, and `X.f(…)` rewritten to `f(…)` |
| `external element Stripe("stripe-pricing-table") { prop publishableKey … }` | each `Stripe(publishableKey: k)` rewritten to `Element("stripe-pricing-table", publishable-key: k)` |

**Files:** `src/parser/` and `src/sema/` (the declaration out), `src/config/project.rs`
(`meta.scripts` — strings or `{ src, module, as, globals }`), `src/codegen/{html,ssg,csp,js}.rs`,
`src/registry/builtins.rs` (`Element`), `src/migrate/`.

## 9. Documentation

`AGENTS.md`, `md-docs/`, the guide under `site/`: a "Your own JavaScript"
section beside "Stylesheets" — the file, its functions, JSDoc, `mount:`,
`wf:render`, `meta.scripts` for a library, and `Element` for a web
component. The `external` section is removed. `RELEASE_NOTES.md` opens a 4.2 section.

---

## Order

1. **Scanner and discovery** (§1) with its case table — nothing else can start without it.
2. **Build output** (§3) — the files reach the page; names still `Any`.
3. **Scope and types** (§2) — `T13` stops, JSDoc narrows.
4. **`mount:` / `update:` / `cleanup:`** (§4) and **`wf:render`** (§5) — independent of 1–3; can run beside them.
5. **`class:` map and list forms** (§6a) and the `wf-` warning (§6d) — independent of everything else; can go first.
6. **Editor** (§7), class completion with it.
7. **`meta.scripts`, `Element`, `external` removed, migration** (§8).
8. **Docs and a browser test** (§9) — one `tests/browser/` case: a script in `src/`, a class, `mount:`, a route change, an `if` toggling, the handle cleaned up each time, and a script reporting through an action that flips a `class:` map entry.

## What breaks

- `external` stops compiling, in every form (`wf migrate` rewrites it).
- A call into a CDN library that `external` typed is `Any` after migration.
- A project whose `src/` already holds a `.js` file it did not mean to ship
  now ships it. `wf migrate` lists any it finds.
- A name a script declares can collide with one the program declares; the
  error names both.

## Decisions

| | Decided |
|---|---|
| Where the knowledge lives | In the compiler; the LSP reads it from there |
| What a script is | **A plain browser script** — top-level `function`/`class`/`const`, no `export`/`import`, copied as written |
| How its names are named | **Global names**, like components — not a namespace per file |
| Remote libraries | **URLs in `meta.scripts`**, used from the project's own scripts; `external` is removed |
| Custom elements | `Element("tag-name", …)`, replacing `external element` |
| Parsing JS | Hand-written scanner; `oxc_parser` only if real files defeat it |
| Conditional classes | `class:` takes a map (`{ "is-on": on }`) and a list, beside a string |
| A class a script controls | The script reports through an action; WebFluent owns the class — one owner per class |
| ES modules under `src/` | Reported, not linked; a published module is a `meta.scripts` entry with `module: true` |

## Progress log

| Phase | State |
|---|---|
| 1 Scanner | done 2026-10-02 — `src/project_js/{mod,scan}.rs`, 15-case table; 0 problems over 5,239 real files |
| 2 Build output | done 2026-10-02 — `build/js/…` byte for byte, linked `defer` before `app.js` (SPA, SSG, acting tests); module = error, unreadable = warning, `public/` collision = error; `wf audit` lists them |
| 3 Scope and types | done 2026-10-02 — `Declaration::Script` injected at project read; arity (defaults, `[x]`, `...rest`) `T10`; JSDoc types `T01` + return type; collisions refused; `new` for classes; `wf types` lists them |
| 4 `mount:` · `wf:render` | done 2026-10-02 — universal props on every element and component call (root, unless it declares the prop); `wf:render` once per drawing; fixed: `if`/`match`/`for`/slot scopes now disposed with their owner (a route change leaked them) |
| 5 `class:` forms · `wf-` warning | done 2026-10-02 — map/list forms, else-less `if`, `V04`, IconButton live class fixed |
| 6 Editor | done 2026-10-02 — scripts are project files in the LSP; completion/hover/definition for their names (signature from JSDoc); class completion/hover/definition inside `class:` (string, map key, list entry); server registers `**/*.{wf,wfx,js,css}` watchers and republishes |
| 7 `meta.scripts` · `Element` · migrate | done 2026-10-02 — `meta.scripts` (URL, `module`/`as`, `globals`), loader wraps classes; `Element("tag")` in SPA/SSG/template; `external` removed from the grammar with a pointer; `wf migrate` rewrites all three forms and lists `.js` under `src/` |
| 8 Docs · browser test | done 2026-10-02 — chapter 32 rewritten (its `// src/` js blocks are type-checked and run by the docs tests), AGENTS.md, upgrading, config, glossary, release notes; `external` out of the editor grammars; `tests/browser/scripts.mjs` (26 checks, SSG + SPA) in CI |
