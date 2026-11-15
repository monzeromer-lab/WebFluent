# WebFluent 7.0: strict types and a new data layer

Written 2026-11-15, against 6.2.0 (commit 39767cde). Approved 2026-11-15;
nothing is built yet.

Two plans in one file. **Part I** makes the whole language strictly and
statically typed. **Part II** redesigns how a page gets and changes data,
with a new command, `wf api`, that writes a service's contract from its
OpenAPI file as modules. They were written together and use the same
spellings, so they read as one design and ship as one release, 7.0.
Sections are numbered within each part; a reference such as "section 4"
means that part's own section.

## How the two fit

- **Shared pieces, built once.** Both need `Json` (open JSON that must be
  matched or decoded before use), `-> T` on actions, results written
  `T ! E` beside `T?`, and `T.decode` (a value checked against its
  type when it arrives). Whichever part's stage reaches them first builds
  them; the other uses them.
- **One cut, no switch.** 7.0 is strict and has the new data layer from its
  first release. There is no loose mode, no warning period and no setting
  that brings the old behaviour back; `lints` and `// wf-allow` cannot
  lower the new errors. Every error that has a known fix carries it.
- **One migration.** `wf migrate` 6 → 7 runs Part II's step first (it
  owns `fetch`, `resource`, `api` and `catch` around requests), then
  Part I's (it writes the types the old checker inferred). 6.3 may ship
  `wf migrate --to 7 --check`, which only reports the size of the move.
- **One rule set for diagnostics.** Part I adds `T23`–`T35`; Part II adds
  the `N` (network) family. Part II's `N13` is folded into Part I's
  `T26` (an action that returns a value must say its type), and `N11` is
  the message `T23`/`T24` give at a request's boundary.
- **The 6.x line takes only fixes** from a `6.x` branch while this lands
  on `master`, so no user sees a half-strict release.

## The order of work, both parts together

Each stage builds, passes the full suite, migrates `site/`, the examples
and the fixtures in the same commit, and ships its docs.

| # | Stage | From | Notes |
|---|---|---|---|
| 1 | Stop losing and duplicating data (no new syntax) | II, stage 1 | Worth doing whatever else is decided; can also go to 6.x as fixes |
| 2 | Measure: `wf types --holes` and the count of every untyped place | I, stage 0 | The number every later stage drives down |
| 3 | The shared spellings: `Json`, `[K: V]`, `(A) -> R`, `-> T`, `T ! E`, `T.decode` | I, stage 1 and II, stage 2 | Built once for both parts |
| 4 | Results, `HttpError`, `try`, patterns | II, stage 2 | Needs 3 |
| 5 | Typed surfaces: the browser, events, `env`, `query`, the registry's props | I, stage 2 | Needs 3; runs beside Part II's stages |
| 6 | Reads as one value (`.loading`, `.failed`, `.loaded`) | II, stage 3 | Needs 4 |
| 7 | Declared signatures | I, stage 3 | Needs 5 |
| 8 | The transport: middleware, HTTP caching, decoding in every build | II, stage 4 | Needs 4 |
| 9 | Contracts and writes: groups, `invalidates`, `idempotent` | II, stage 5 | Needs 6 |
| 10 | Closing the agreement rules | I, stage 4 | Needs 7 |
| 11 | Boundaries: `data` files, templates, tests, `persist` | I, stage 5 | Needs 10 |
| 12 | Pages, streams, offline | II, stage 6 | Needs 4 |
| 13 | Fakes for tests | II, stage 7 | Needs 4 |
| 14 | The OpenAPI reader and `wf api` | II, stage 8 | Needs 4 |
| 15 | Migration: both 6 → 7 steps, every fix, `lints` refusing the new codes | I, stage 6 and II, stage 9 | Needs everything before it |
| 16 | 7.0: `Any` and `Map` leave the grammar; release | I, stage 7 | The holes count, asserted zero |

Stages 4–14 are two lines of work that meet at 15, and can run in
parallel once stage 3 is in. Part II's results stage (4) comes first; then
4 → 6 → 9 is a chain, and 8, 12, 13 and 14 can each run beside it. Part
I's stages go in order: 5 → 7 → 10 → 11.

## Open questions

Each part ends with its own, each with a recommendation: Part I has ten
(section 5), Part II has ten (section 10). They agree where they overlap —
results are written `T ! E`; required types are errors at once, old code
included; every response and every script result is checked in
production; and the rest of the language is as strict as the data layer.

---

## Part I — Strict typing: the whole language

Written 2026-11-15, against 6.2.0 (commit 39767cde). Nothing here is built
yet: this is the plan to approve first.

The data-fetching plan (Part II, sections 4 and 5.2) makes the data
layer strongly and statically typed and leaves the rest of the language to
this plan (its open question 5). This plan applies the same rules
everywhere else, with the same spellings, so the two read as one design.

### In short

- **`Any` goes.** Today a value the checker cannot work out is `Any`, and
  `Any` agrees with every type in both directions
  (`src/check/sema/types/ty.rs:196`). That one rule lets more than a hundred
  places in the checker fail open. After this plan, every value has one type the
  checker knows and the source can write, and a program that checks has no
  `Any` in it anywhere.
- **`Map` goes.** A map of unknown keys becomes a record (`type User {…}`),
  a dictionary (`[String: Number]`, new), or `Json` — open JSON that must be
  matched or decoded before use.
- **Signatures are declared; bodies are inferred.** Parameters, props and
  an action's result say their type; `let`, `derived`, lambdas inside a
  call and a `state` with a self-evident first value are inferred.
- **No implicit conversions.** A `Duration` is not a number, a `Date` is not
  a `String`, a `Number` is not a condition.
- **Data from outside is checked when it arrives**: a `data` file at build
  time, a template's JSON at render time, a `Json` value when it is decoded,
  a script's return value on every call.
- **Strict is the language, not a setting.** There is no `"types"` switch,
  no loose mode, and no way to lower or silence a typing error: `lints`
  and `// wf-allow` refuse every code this plan adds. A program either
  types or does not build.
- **Migration**: 7.0 is strict from its first release. Every hole is an
  error carrying an automatic fix, and `wf migrate` (6 → 7) writes the
  inferred types in before the first 7.0 build. `Any` and `Map` leave the
  grammar in 7.0. Nothing changes in 6.x.

---

### 1. Where values are untyped today

Paths are under `src/check/sema/types/` unless given in full. Every
example compiles today with no finding.

#### 1a. The rules that let anything through

| # | Where | What | Example that passes |
|---|---|---|---|
| H1 | `ty.rs:196` | `Any` is assignable to and from every type | `state n: Number = JSON.parse(s)` |
| H2 | `ty.rs:203–205` | `Map`, a map literal's shape and every record agree with each other, without looking at fields | `state u: User = { nmae: "Ada" }` — a `User` with no `name` |
| H3 | `ty.rs:237`, `ty.rs:262–263` | Two types with nothing in common join to `Any` instead of being refused | `derived x = if ok { 1 } else { "none" }` is `Any`, and `x * 2` passes |
| H4 | `ty.rs:161`, `src/syntax/parser/v2.rs:941–942` | `Any` and `Map` can be written as types | `component Card2(data: Any)` |
| H5 | `src/syntax/parser/v2.rs:2235–2239` | A parameter written without a type is `Any` | `action add(item) { items = items.concat([item]) }` |
| H6 | `ty.rs:313` | `Any` counts as a number for arithmetic | `derived total = rows.reduce((a, r) => a + r.price, 0)` with `rows: [Map]` |
| H7 | `ty.rs:199–223` | Implicit conversions: a text scalar fits `String`, `Duration` and `Number` fit each other | `Text(due)` with `due: Date`; `after(3.seconds + 500) { … }` |

#### 1b. Declarations whose type is never known

| # | Where | What | Example |
|---|---|---|---|
| D1 | `mod.rs:937–939` | An action's result is `Any` (or a promise of `Any`) to every caller, whatever it returns | `action headers() { return { auth: token } }` — `Api.headers().nope` passes |
| D2 | `mod.rs:359–361` | A store reading a store declared later sees `Any` ("`Any` until met") — the result depends on declaration order | `store A { derived n = B.count + 1 }` before `store B` |
| D3 | `mod.rs:858–862` | A `state` that starts `null` is `Any?` | `state picked = null` then `picked.nmae` |
| D4 | `expr.rs:410` | An empty list is `[Any]` | `state items = []`, then `items.concat([1, "a"])` |
| D5 | `expr.rs:412` and on | An empty map `{}` has no fields, so nothing can be read from it — or with `Map`, anything can | `state form = {}` |
| D6 | `mod.rs:326–328` | A `data` file with no annotation is `Any` | `data posts = "posts.json"` |
| D7 | `mod.rs:870–885` | A `resource` over a plain address with no annotation is a resource of `Any` | `resource rows = fetch("/api/rows")` (the data plan's `N11`) |
| D8 | `mod.rs:907` | A connection handle (`socket`, `stream`, `channel`, `peer`, `worker`) is `Any` | `chat.sned(m)` passes |
| D9 | `mod.rs:914`, `handles.rs:15` | A `ref:` handle and `form.element` are `Any` | `search.focuss()` passes |
| D10 | `expr.rs:513` | `Name(…)` for a name that is neither a type nor an enum is `Any` | — |

#### 1c. Expressions that give up

| # | Where | What | Example |
|---|---|---|---|
| E1 | `expr.rs:820` | A field of a `Map` — or of anything not listed — is `Any` | `r.findings` with `r: Map` |
| E2 | `expr.rs:287`, `expr.rs:302` | Indexing anything but a list or string is `Any` | `headers["X-Id"]` |
| E3 | `expr.rs:527–529` | A lambda's parameters are `Any` when nothing says what it is given | `let byName = (a, b) => a.name.localeCompare(b.name)` |
| E4 | `method.rs:482` | Many string methods (`padStart`, `at`, `charAt`, …) return `Any` | `s.at(0).toUpperCase()` |
| E5 | `method.rs:253–581` (twelve places) | A list method or a method of `Any`/`Map` returns `Any` | `rows.reduce(…)` |
| E6 | `expr.rs:747–751` | A field of a list or a string other than `length` is `Any` | `items.lenght` (no `T05`) |
| E7 | `tables.rs:210`, `tables.rs:239` | `.native()` and a `File`'s `text()`, `arrayBuffer()` are `Any` | `await file.text()` |
| E8 | `stmt.rs:405` | `for` over `Any` or `Map` gives `Any` items | `for row in result.findings { … }` |
| E9 | `stmt.rs:773–847` | A `match` arm's binding is `Any` when the matched value is `Any`, a bare case, or a request's `.status` | `match s { .failed(r) { … } }` with `s` untyped |

#### 1d. What comes from outside

| # | Where | What | Example |
|---|---|---|---|
| X1 | `stmt.rs:396` | `catch e` is `Any` | `catch e { error = e.mesage }` |
| X2 | `names.rs:131–135`, `names.rs:170` | `window`, `document`, `console`, `JSON`, `Math`, `Date`, `navigator`, `location`, `Object`, `Array`, `Promise`, `fetch`, timers, `event`, `params`, `locale`, `dir` and any unlisted global are `Any` | `JSON.parse(text).itmes` |
| X3 | `names.rs:137`, `names.rs:150` | `env` and `query` are `Map` | `env.PUBLIC_APII` |
| X4 | `element.rs:238`, `stmt.rs:380` | A DOM event handler's parameter is `Any` | `on keydown(e) { if e.kye == "Enter" { … } }` |
| X5 | `element.rs:271`, `stmt.rs:33` | A component event's extra values beyond its declaration are `Any` | — |
| X6 | `jsdoc.rs:67`, `jsdoc.rs:82`, `jsdoc.rs:169–179`; `call.rs:154`, `call.rs:218`, `call.rs:238–239` | A project script's function with no JSDoc, a union (`string \| number`), `*`, an unknown name, a class instance, a value the scanner cannot see — all `Any` | `initTilt(node, "fifteen")` with no JSDoc |
| X7 | `expr.rs:583`, `members.rs:149`, `members.rs:160`, `expr.rs:756` | A request error's `body` and a resource's `items` and `error` are `Any` | (replaced by the data plan's `HttpError`) |
| X8 | `stmt.rs:517–523` | The legacy `fetch … { }` block binds its data and error as `Any` | — |
| X9 | `src/registry/mod.rs:59` | 72 built-in props are `PropType::Any`: `Portal(into:)`, `Tabs(value:)`, `Masonry(columns:)`, `AspectRatio(ratio:)`, `VirtualList(reveal:)`, … | `Tabs(value: [1, 2])` |
| X10 | `refine.rs:27` | `required`, `custom`, `async`, `matches` and `oneOf` accept `Any` | — |
| X11 | `src/template/mod.rs:291` | A template rendered with data ignores `T13`: every top-level name is whatever the JSON holds | `Text(customer.nmae)` renders an empty string |
| X12 | `mod.rs:527` | The type checker passes over `test` declarations, so `data: { … }` seeds are not held to the state they seed | `test "x"(data: { count: "two" }) { … }` |

Beside these, `Type::Any` is also what the checker returns *after* it has
reported an error, so one mistake does not cascade into twenty. That use is
right and stays, under its own name (section 2.1).

#### 1e. How much the site and the examples rely on it

Counts over `site/src` (213 `.wf` files) and `examples/` (71). The docs
site is a docs site: some hits are inside code samples it shows, so these
are upper bounds. Real code is named where it matters.

| Hole | `site/` | `examples/` | Notes |
|---|---|---|---|
| `Map` written as a type | 41 in 19 files | 1 | Real code: `PlaygroundShell.wf`, `HomeShell.wf`, `Example.wf` (`state result: Map?`, `derived findings: [Map]`, `action heard(r: Map)`, `const OUTPUT_PAGES: Map`) |
| `Any` written as a type | 51 in 2 files | 1 | 42 are reference signatures in text (`Builtins.wf`); real code: `examples/tutorials/hard-pocket/.../Settings.wf:27`, `let cells: [Any]` |
| `state` with no type | 535 (123 typed) | 20 (6 typed) | About 50 need one: 25 start with `[`, 12 with `{`, 1 with `null`, the rest from calls. The other ~485 start with a string, number, boolean or record literal and are already complete |
| untyped parameter | 2 of 482 | 0 | Parameters are typed almost everywhere already |
| action returning a value | 38 `return`s in 13 files | 6 in 2 | Each action needs `-> T` |
| `catch e` | 32 in 21 files | 0 | Mostly `e.message` |
| `await fetch(…)` | 23 in 13 files | 0 | The data plan's migration |
| `resource` with no type | 18 (8 typed) | 0 | The data plan's migration |
| `data` with no type | 1 (10 typed) | 6 (1 typed) | `examples/templates`, `examples/documents` |
| browser globals | 129 in 47 files | 3 | Most used: `navigator.clipboard`, `window.WF`, `location.*`, `document.addEventListener`, `console.log`, `Object.keys`, `Math.*` |
| `env.` / `query.` | 39 / 31 | 0 / 6 | Both become typed with no source change |
| handler with an event parameter | 191 in 36 files | 5 | Become typed with no source change, except misspelled fields |
| `ref:` | 309 in 16 files | 0 | Become `Element`, no source change |
| `derived` with no type | 242 (11 typed) | — | Inferred; stays as it is |
| `const` with no type | 88 | 42 | Inferred from a literal; stays |

So the work in real code is small: about 50 `state` annotations, about 50
action results, the `Map`s in three playground components, 32 `catch`
blocks that already only read `.message`, and the data plan's own changes.

---

### 2. The rules

These are the data plan's R13 and section 5.2, applied to the whole
language. Each rule says what it rules out today.

#### 2.1 One type per value, and every type can be written

1. **No `Any`.** No value a program can name is `Any`. The checker's
   recovery type after an error is renamed `Type::Error`: it is never
   written in source, never shown in a hover or `wf types`, and only exists
   where an error has already been reported. A test holds the whole site
   and every example to "no `Error`, no `Any`" once they check clean
   (stage 7).
2. **Every type the checker knows can be written.** If you can hover it, you
   can declare it. That needs three spellings the language lacks today
   (2.2): dictionaries, functions, and type aliases.
3. **Agreement is exact.** `A` fits `B` only when it is `B`, or `B?` and `A`
   is `B` or `null`, or a list or dictionary whose items fit, or a map
   literal whose fields are exactly a record's (each required one present,
   none unknown, each fitting) — H2 goes.
4. **Two branches that disagree are an error, not a join to `Any`.**
   `if ok { 1 } else { "none" }` is `T27`; `[1, "a"]` is `T27`. `null` in
   one branch still makes the other `T?`, as today.

#### 2.2 The spellings

All without angle brackets. New ones in bold.

| Written | Means | Example |
|---|---|---|
| `String`, `Number`, `Bool` | as today | |
| `Date`, `Money`, `Url`, … | the eleven built-in scalars, as today | |
| `[T]` | a list of `T` (today) | `[Todo]` |
| `T?` | a `T`, or `null` (today) | `User?` |
| **`[K: V]`** | a dictionary: keys of `K` (`String`, or an enum), values of `V` | `[String: Number]`, `[Region: Money]` |
| **`(A, B) -> R`** | a function; `()` as the result means it returns nothing (there is no `Void`) | `(Todo) -> Bool`, `(String) -> ()` |
| **`type Name = T`** | an alias, read as `T` | `type Scores = [String: Number]` |
| **`Json`** | open JSON: matched or decoded before use | `state raw: Json = Json.null` |
| `T ! E`, `T!`, `! E`, `!` | results (the data plan, 5.2) | `-> Person ! CreatePersonError` |
| `Number(1..=30)`, `String(minLength: 8)` | refinements (today) | |

**Dictionaries** answer the open-keyed `Map`s: `query` is `[String: String]`,
`form.errors` is `[String: String]`, a lookup table is
`[Region: Money]`. Reading a key gives a `V?` — a key may be missing —
whether written `d["k"]` or `d.k`. `.keys()`, `.values()`, `.entries()`,
`.has(k)`, `.with(k, v)` and `.without(k)` are its methods; a dictionary
literal is a map literal written where one is wanted:
`state hits: [String: Number] = {}`.

**Function types** are what a prop that takes a lambda declares
(`component Picker(text: (Option) -> String)`) and what an action is when
passed as a value. An action's type is `(A) -> R`, or `(A) -> R ! E`.

**`Json`** is a built-in enum, as the data plan defines it:

```wf
enum Json {
    object(fields: [String: Json])
    array(items: [Json])
    string(value: String)
    number(value: Number)
    bool(value: Bool)
    null
}
```

You use one by matching it, or — far more often — by decoding it into a
declared type. **Every declared record, enum and alias gets a `decode`
member**, `T.decode(json) -> T ! DecodeError`, where `DecodeError` is the
record the data plan's `HttpError.decode` carries (`at: String`,
`message: String`). No generics: the type is the receiver.

```wf
type Settings { theme: String, columns: Number }

action restore(text: String) -> ! DecodeError {
    settings = try Settings.decode(JSON.parse(text))
}
```

**What goes away.** `Any` and `Map` as written types, removed in 7.0.
`Int`/`Float` stay the aliases of `Number` they are.

#### 2.3 Declared signatures, inferred bodies

What another declaration, another file or a caller sees is declared; what
only the body sees is inferred. The checker then never needs to infer one
declaration to check another, which removes D2's order dependence.

| What | Rule | Example |
|---|---|---|
| A parameter: action, component prop, page parameter, event, endpoint | **Declared**, always (H5) | `action add(title: String)` |
| An action's result | **Declared** with `-> T` when it returns a value; nothing when it returns nothing; `-> T ! E` when it can fail (D1) | `action headers() -> [String: String] { … }` |
| A `state` or `persist` | **Inferred** when its first value says everything — a string, number, boolean, scalar, enum case, record construction, or a list or map literal whose items do; **declared** otherwise (D3–D5) | `state n = 0` ✓ · `state items = []` → `state items: [Todo] = []` |
| A `derived` | **Inferred**, as today; in a store, resolved in dependency order, and **declared** only when two members read each other (`T35`) | `derived open = items.filter(t => !t.done)` |
| A `const` | **Inferred** from its value, as today | `const PAGE = 20` |
| A `data` file | **Declared** (D6) | `data posts: [Post] = "posts.json"` |
| A `resource` | As the data plan says: from the endpoint, or annotated | |
| `let` | **Inferred** | `let total = rows.map(r => r.price).sum()` |
| A lambda's parameters | **Inferred** from what it is passed to; **declared** when nothing says (E3) | `rows.filter(r => r.live)` ✓ · `let byName = (a: User, b: User) => …` |
| `catch e` | **`Error`**, a built-in record: `name: String`, `message: String`, `stack: String?`. The runtime wraps anything thrown that is not an `Error` in one, so the type is true (X1) | `catch e { error = e.message }` — no change |
| A connection | **Typed by its declaration**: `send`/`receive` types; a socket that declares none sends and receives `Json` (D8) | `chat.send(Outgoing(text: draft))` |
| `ref:` | **`Element`**, a built-in handle type (2.5) (D9) | `search.focus()` |
| A `match` arm | From the matched type; matching `Json` binds the payload's type (E9) | |

A literal list or map "says everything" when it has at least one item and
its items agree (`[1, 2]`, `{ name: "", age: 0 }`). An empty one says
nothing, so `state items = []` is `T25` with a fix that writes the type the
checker sees assigned to it later (`items = items.concat([Todo(…)])` →
`[Todo]`).

#### 2.4 No implicit conversions

| Today | Strict | Fix |
|---|---|---|
| `Text(due)`, `due: Date` | `T34` — a `Date` is not a `String` | `Text("{due}")` or `Text("{due:.date(long)}")` — a splice is the one place a value becomes text, and it says so |
| `3.seconds + 500` | `T34` | `3.seconds + 500.ms` |
| `after(n) { … }` with `n: Number` | `T34` where a `Duration` is wanted | `after(n.ms)` |
| `"Total: " + n` | `T18` — `+` takes two numbers, two strings, or two durations | `"Total: {n}"` |
| `if items.length { … }`, `if name { … }` | `T29` — a condition is a `Bool` (extends `T07`) | `!= 0`, `!= ""` |
| `a \|\| "default"` | `T29` | `a ?? "default"` |
| `!count` | `T29` | `count == 0` |
| a list or record spliced into text | `T20` becomes an error | `.join(", ")`, a field |

A splice stays the one sanctioned way to show a value: it shows a `String`,
`Number`, `Bool`, a scalar (formatted by its style), or an enum case's name.
`T19` (a `T?` spliced) stays a warning.

#### 2.5 The browser, typed

Browser names keep meaning exactly what the browser means (the data plan's
R8) — they are just typed. `src/registry/browser.rs` grows from a list of
names into a table of declarations, written once and read by the checker,
the editor and `wf types`:

```text
JSON.parse(text: String) -> Json                 throws SyntaxError, as the browser does
JSON.stringify(value: Json) -> String            any type that is plain data fits
localStorage.getItem(key: String) -> String?
localStorage.setItem(key: String, value: String)
Math.max(a: Number, b: Number) -> Number         … every Math member
Object.keys(d: [String: V]) -> [String]          and on a record
navigator.clipboard.writeText(text: String) -> ()     async
location.pathname: String
document.title: String
console.log(values: …)                           anything but a Secret
window.open(url: Url) -> ()
setTimeout(f: () -> (), ms: Number) -> Number
```

The table covers what the site and the examples use (about forty members)
and the common rest of `Math`, `JSON`, storage, `location`, `history`,
`navigator.clipboard`, `document`'s title and listeners, and timers. A
member it does not list is `T30`:

```text
error[T30]: the language does not type `navigator.bluetooth`
  = help: Reach it from a script under src/ and say its types in JSDoc;
          the call is then checked like any other
```

That is the existing door for the browser's long tail (*Your own
JavaScript*): a plain script with JSDoc, typed at the call site.

**DOM events** are typed by name: `on keydown(e)` binds a `KeyboardEvent`
(`key`, `code`, `shiftKey`, `ctrlKey`, `altKey`, `metaKey`, `repeat`,
`preventDefault()`, `stopPropagation()`, `target`, `currentTarget`), `on
click(e)` a `PointerEvent` (`clientX`, `clientY`, `button`, the modifier
keys), and likewise `InputEvent`, `SubmitEvent`, `FocusEvent`,
`WheelEvent`, `DragEvent`, `ClipboardEvent`, `ScrollEvent`, falling back to
`Event`. The runtime's gestures get theirs: `Swipe`, `Drop`, `DragMove`.
`target` and `currentTarget` are `Element` (X4).

**`Element`** is a built-in handle with the members a page reaches for:
`value: String`, `checked: Bool`, `focus()`, `blur()`, `select()`,
`scrollIntoView()`, `getBoundingClientRect() -> Rect`, `dataset: [String:
String]`, `contains(other: Element) -> Bool`, `closest(selector: String) ->
Element?`. Anything else is `T30`, with the same pointer to a script.

**The language's own values** become exact: `env` is a record of the names
this build knows, each `String` (so `env.PUBLIC_APII` is `T05`, with the
nearest name); `query` is `[String: String]`; `params` is a record of the
page's own parameters; `locale` and `dir` are `String`; `form.errors` and
`form.touched` are `[String: String]` and `[String: Bool]`, `form.values`
`[String: Json]` (X2, X3).

#### 2.6 Project scripts

A script under `src/` stays plain JavaScript; its JSDoc is its signature,
as today. What changes is what happens without one (X6):

| JSDoc | Today | Strict |
|---|---|---|
| none, on a function called from `.wf` | `Any` in, `Any` out | `T31`, with a fix that writes a JSDoc block from the call sites (`@param {number} max`) |
| `{string\|number}` | `Any` | `T31`: the language has no unions — use two functions, or `{*}` |
| `{*}`, `{any}`, `{object}`, `{Object}` | `Any`, `Map` | `Json` |
| `{Object<string, number>}`, `{Record<string, T>}` | `Map` | `[String: Number]` |
| `{Array}` | `[Any]` | `T31`: say the item, `{string[]}` |
| `{HTMLElement}`, `{Element}` | `Any` | `Element` |
| `{Promise<T>}` | `T` (awaited) | as today |
| a class | instance `Any` | a type named after the class, whose members are its JSDoc'd methods |
| `@returns` missing | `Any` | `()` — returns nothing; reading the result is `T10` |
| `@typedef {{ id: string, n: number }} Row` | not read | a record `Row` the program may name |

Every call into a script, in every build, checks what comes back against
its `@returns`, and a mismatch throws a `DecodeError` naming the function:
a script is code the checker cannot see, so its promise is held where it
is kept. There is no build in which a value typed `Row` may not be a
`Row`.

#### 2.7 Data that arrives

The data plan's R5 — validate at the boundary — for each way data enters
that is not a request:

- **`data` files** (D6) are declared and checked **at build time** against
  the JSON on disk. A mismatch is `T33`, naming the file and the JSON path:
  `posts.json: $[3].date is "March 5", not a Date`. `wf migrate` writes
  the type from the file (stage 6).
- **Template data** (X11). A template says what it takes as its page's
  parameters, as a route says what it takes:

  ```wf
  type Customer { name: String, email: Email }
  type Line { name: String, price: Money }

  page Invoice(path: "/", title: "Invoice {number}",
               number: String, customer: Customer, items: [Line], paid: Bool) {
      Heading("Invoice #{number}").h1
      Text("Customer: {customer.name}")
      for item in items { Text("{item.name}: {item.price}") }
  }
  ```

  `T13` then applies to templates like everything else, and every render
  (`wf render`, `Template::render_*`, the Node wrapper) decodes the JSON
  against those parameters first. A mismatch is an error naming the path
  (`$.items[1].price is missing`), never an empty string on the page. The
  Rust API is unchanged: anything `Serialize` still goes in.
- **`wf test` data** (X12). The type checker reads `test` declarations; a
  `data:` key names a state of the body or a store, and its value is held
  to that state's type (`T33`). A key nothing declares is `T05`.
- **`Json`** is decoded with `T.decode(…)`, at run time, in every build.
- **`persist`**: what storage hands back is decoded against the declared
  type when read; a value that does not fit is treated as missing and the
  default is used, with the `migrate` steps as today. Today it is trusted.

#### 2.8 Built-ins

The registry's 72 `PropType::Any` props (X9) each get a real type:
`Portal(into: Element?)`, `Tabs(value: String)`, `AspectRatio(ratio:
Number)`, `VirtualList(reveal: String?)`. A `PropType` becomes a type
reference the checker reads, rather than the coarse kinds it is today
(`Str`, `Num`, `State`, `Any`, …), so `wf registry --json`, the studio's
inspector and the docs show the real type.

Responsive props (`columns:`, `gap:`, `span:`) take `T` or
`[Breakpoint: T]`, where `enum Breakpoint { base, sm, md, lg, xl }` is
built in. This is the one place a prop accepts two types; it is written in
the registry and shown as `Number or [Breakpoint: Number]`, and a component
that forwards one declares `[Breakpoint: Number]` (open question 3).

`validate` rules (X10) are typed by what they guard: `required` takes any
type, `oneOf([…])` a list of the guarded type, `matches(other)` a state of
the same type.

---

### 3. Migration and diagnostics

#### 3.1 No switch, one cut

The data plan's R12 now reads "change in one release, never silently;
automate the migration", for this reason: a typing rule that can be turned
off is not a rule: a program would be typed or not depending on a config file,
a library would have to work both ways, and "reliable" would mean "reliable
if you asked". So strictness ships whole, once, and is never optional.

| Release | What happens |
|---|---|
| **6.x** | Nothing changes. 6.3 may ship `wf migrate --to 7 --check`, a report of what the 6 → 7 step will rewrite and what it cannot, so a team can see the size of the move before it upgrades. It changes no build. |
| **7.0** | Every rule of section 2 is in force, as errors. `Any` and `Map` are no longer types ("`Map` is gone; write a record, `[String: T]` or `Json`"). Every error that has a known fix carries it (`wf fix`, the editor's quick fix), and `wf migrate` applies them all at once (3.2). `lints` and `// wf-allow` cannot lower or accept `T23`–`T35`, nor the errors they extend (`T01`, `T05`, `T13`, `T20`). There is no `"types"` key; writing one is `E112`. |

The cost lands once, on upgrade, and `wf migrate` carries most of it: in
the site and the examples, everything in 1e but a handful of `Map`s that
need a record drafted is rewritten mechanically.

**One rule across both plans.** The data plan follows the same cut: every
old form of its section 8 is an error with a fix in 7.0, `N11` is the
boundary's message for `T23`/`T24`, and its `N13` is folded into `T26`, so
there is no corner of the language where `Any` survives.

#### 3.2 What `wf migrate` writes (6 → 7)

It runs the 6.x checker over the project — the one place the old rules
still run, and only to read what they inferred — then writes back each
type wherever it is complete:

| Before | After |
|---|---|
| `state items = []` (later assigned `[Todo]`) | `state items: [Todo] = []` |
| `state picked = null` (later assigned a `User`) | `state picked: User? = null` |
| `action headers() { return { auth: tok } }` | `action headers() -> [String: String] { … }` |
| `action load() { … await … return r.items }` | `action load() -> [Item] { … }` |
| `data posts = "posts.json"` | `data posts: [Post] = "posts.json"` and, in `src/types/posts.wf`, `type Post { slug: String, title: String, date: Date }` drafted from the file |
| `state result: Map? = null` | a record drafted from the fields the program reads (`result?.findings`, `r.version`) — `type Result { findings: [Finding], version: String }` — else `Json?` |
| `component C(data: Any)` | the type every call site passes, when they agree; else `Json` and a note |
| `action add(item)` | the type every call site passes |
| `Text(due)` | `Text("{due}")` |
| `if items.length { … }` | `if items.length != 0 { … }` |
| `a \|\| b` with `a` not `Bool` | `a ?? b` |
| a script function with no JSDoc | a JSDoc block from its call sites |
| a template's top-level names | page parameters, typed from how they are used, and from `--data sample.json` when given |

Where it cannot be sure it writes `Json` and lists the place, so nothing
it writes is a guess presented as fact. Like the 2 → 3 step, it changes
types and nothing else: the migrated project builds to the same JavaScript
except where a decode is now inserted (`data`, `persist`, templates). The
step runs after the data plan's 6 → 7 step, which owns `fetch`, `resource`,
`api` and `catch` around requests.

#### 3.3 New diagnostics

`T23`–`T35`, after `T22`. All are errors from 7.0, none can be lowered by
`lints` or accepted by `// wf-allow`, and "fix" says what `wf fix` and the
editor write.

| Code | Severity | What |
|---|---|---|
| `T23` | error | `Any` written as a type. Fix: the inferred type, else `Json` |
| `T24` | error | `Map` written as a type. Fix: a drafted record, `[String: T]`, or `Json` |
| `T25` | error | A `state`, `persist` or `data` whose type its value does not say (`[]`, `{}`, `null`, a call returning `Json`). Fix: the type assigned to it later |
| `T26` | error | An action that returns a value without declaring `-> T`. Fix: the inferred result. (the data plan's `N13` is folded into it) |
| `T27` | error | Branches, list items or `??` sides with no common type |
| `T28` | error | A `Json` value read as if it were something else — `raw.name`, `raw + 1` — rather than matched or decoded |
| `T29` | error | A condition, `!`, `&&` or `\|\|` on something that is not a `Bool`. Fix: the comparison it meant, or `??` |
| `T30` | error | A browser member, or an `Element` member, the language does not type |
| `T31` | error | A project-script function called from `.wf` with no JSDoc, or with a type the language cannot say. Fix: JSDoc from the call sites |
| `T32` | error | A parameter, or a lambda parameter nothing gives a type to, written without one. Fix: the type its callers pass |
| `T33` | error | Data that does not match its declared type: a `data` file at build time, a test's `data:`, a template's data at render time |
| `T34` | error | An implicit conversion: a scalar where `String` is wanted, `Duration` and `Number` mixed. Fix: a splice, `.ms` |
| `T35` | error | Two store members whose types depend on each other with neither declared |

Changed: `T01`, `T05`, `C01` now apply to a map literal given where a record
is wanted (H2); `T13` applies to templates; `T20` becomes an error; `T07`'s
cases fold into `T29`.

---

### 4. Stages

Each stage builds, passes the full suite, and ships its docs (`AGENTS.md`,
the guide's types chapter, `md-docs/`). Unit tests go in
`src/check/sema/types/tests.rs` and beside the code; integration tests in
`tests/*.rs`; browser tests in `tests/browser/*.mjs`.

**Stage 0 — Measure.** Rename the recovery type to `Type::Error`; add
`wf types --holes`, which lists every binding, member, parameter and
expression result that is `Any`, with its place and why (which of H1–X12).
*Tested by* a snapshot test that runs it over `site/` and every example and
records the count per hole kind: the number every later stage must drive
down, and the place a regression shows first.

**Stage 1 — Spellings.** Parse and check `[K: V]`, `(A) -> R`,
`type X = T`, `Json` and its cases, `-> T` on actions, typed lambda
parameters, `T.decode`. Codegen: dictionaries are plain objects; `decode`
is a generated walker per type (shared with the data plan's response
decoding). *Tested by* parser round-trips and `wf fmt` stability for each
spelling; checker tests for each agreement rule; a runtime test that
`User.decode` accepts a good value and names the path of a bad one, in
Node and in a browser.

**Stage 2 — Typed surfaces.** The browser table (2.5), DOM event types,
`Element`, typed `env`, `query`, `params`, `form`, connections, `catch e:
Error` with the runtime wrapping non-errors, and real types for the
registry's 72 props. *Tested by* one checker test per table entry family
(a correct use passes, a misspelt member is `T05`/`T30` with the nearest
name), a browser test that a thrown string arrives as an `Error`, and the
registry's existing completeness tests extended to "no prop is untyped".
The hole count drops by every X entry.

**Stage 3 — Declared signatures.** Action results from `-> T`; store
members resolved in dependency order, `T35` on a cycle (fixes D2); `state`
inference limited to self-evident values (`T25`); `T32` for untyped
parameters; JSDoc rules (`T31`), `@typedef`, classes as types, the
return check on every call. *Tested by* a test that declares stores
in both orders and gets the same types; one test per row of the table in
2.3; a browser test that a script returning the wrong shape throws, in a
`wf serve` build and in a production one alike.

**Stage 4 — Closing the agreement rules.** H1–H3 and H6–H7: exact
agreement, `T27`, `T29`, `T34`, `T28`, map literals checked field by field
against records. *Tested by* the checker suite, and a test that every
fixture under `tests/fixtures/` either checks or carries the expected new
codes.

**Stage 5 — Boundaries.** `data` files checked at build time (`T33`);
template parameters and render-time decoding; `test` declarations checked,
`data:` seeds held to their states; `persist` decoded on read. *Tested by*
a `data` fixture with one bad record (the message names `$[3].date`);
Rust and Node template tests that a missing field fails the render with
its path; a `wf test` fixture whose seed has the wrong type; a browser test
that a stored value of the old shape falls back to the default.

**Stage 6 — Migration.** The fixes for every new error; `wf migrate`'s
6 → 7 step (3.2); `wf fix --code T25` and the rest; `lints` and
`wf-allow` refusing the typing codes. *Tested by* migrate fixtures
(before, after, and the notes it prints); a test that a migrated project
builds to the same JavaScript as before, decode points aside; `wf migrate
--check` reporting nothing on a second run; a test that `"lints": { "T23":
"warn" }` and `// wf-allow(T29)` are refused.

**Stage 7 — 7.0.** `Any` and `Map` leave the grammar; release. *Tested
by* the parser's refusal tests, and the stage 0 snapshot reading zero
holes for `site/` and every example, now asserted rather than recorded:
no `Any` and no `Error` in any binding of a program that checks.

**How the work lands without a switch.** Stages 1–6 land on `master` one
after another, and each one that adds an error migrates `site/`, the
examples and the test fixtures in the same commit, so the suite stays
green and `master` is always strict as far as it has got. Nothing is
released between stage 1 and 7.0: the 6.x line takes only fixes, from a
`6.x` branch. A user never sees a half-strict release.

The data plan's stages and these share stage 1 (`Json`, `-> T`, `T ! E`,
`decode`). Whichever plan starts first builds them once; the other uses
them.

---

### 5. Open questions, each with a recommendation

1. **Must a `state` always be declared, or may it be inferred?**
   *Inferred when its first value says everything.* About 485 of the
   site's 535 untyped states start with a literal whose type is complete;
   writing `: String` beside `= ""` adds noise and no safety. A store's
   members are read from other files, but a literal's type is as visible
   as an annotation. Only the ~50 that start empty or `null` need one.
2. **`[K: V]` for dictionaries, or a named built-in like `Dict`?**
   *`[K: V]`.* It reads as `[T]` does, needs no angle brackets, and is what
   Swift writes. A name would need a way to say its key and value types,
   which is generics again.
3. **Should a prop be allowed two types (responsive values)?** *Only the
   registry's responsive props, `T` or `[Breakpoint: T]`, and nothing a
   program declares.* The alternative — always `[Breakpoint: T]` — makes
   `Grid(columns: 3)` into `Grid(columns: { base: 3 })` in every program
   for no safety gained. Unions in general stay out: an enum with payloads
   says the same thing with names.
4. **Check script return values in production too?** *Yes, always.*
   A check that only runs while the author is looking leaves the reader
   with a value typed `Row` that is not one, which is the bug this plan
   exists to remove. The check is a walk of what came back, cheap beside
   the call; a hot loop that cannot afford it belongs inside the script.
5. **Keep `JSON.parse` returning `Json`, or add a language `Json.parse`
   that returns a result?** *Keep the browser's name and type it.*
   `JSON.parse(text) -> Json`, throwing as the browser does; `T.decode`
   then turns `Json` into a type. A second parser spelt `Json.parse` beside
   `JSON.parse` would be two names for one thing, against R8.
6. **Where does "an action that returns a value must declare it" live —
   the data plan's `N13`, or this plan's `T26`?** *`T26`, and an error in
   7.0 for old code too.* It is about all actions, not requests. The data
   plan now folds its `N13` into `T26` and drops the warning for old code
   from `N11`: with no switch in the language, there is no older code that
   is allowed to be less typed.
7. **How does an unknown browser member get used?** *Through a project
   script with JSDoc (`T30`'s fix), not an escape hatch in `.wf`.* A
   `Json`-typed `window.anything` would bring `Any` back under another
   name. The table grows by what real projects ask for.
8. **A release of warnings before the errors?** *No.* A warning period is
   a switch by another name: for a whole major the language would accept
   both typed and untyped programs, libraries would be written to the
   looser one, and the errors would break them later anyway. One cut, with
   `wf migrate` doing the work and `--check` in 6.3 showing its size first,
   is less total pain and leaves no mode behind.
9. **Should `persist` drop a value that no longer fits, or fail?**
   *Drop it and use the default, with a `console.warn` in development.* A
   reader's stored settings must never stop a page from loading; `version`
   and `migrate` remain the way to keep old values.
10. **Should `Text(n)` with a `Number` need a splice?** *No.* A `Number` and
    a `Bool` show as themselves with no formatting choice to make; the rule
    in 2.4 is about scalars (a `Date` has many spellings) and `Duration`.
    `Text` takes `String`, `Number` or `Bool`, as a splice does.

---

### Appendix: where things are

| What | Where |
|---|---|
| The checker's types, agreement and join | `src/check/sema/types/ty.rs` |
| Declarations, stores, actions, hoisting | `src/check/sema/types/mod.rs` |
| Expressions, fields, lambdas | `src/check/sema/types/expr.rs`, `method.rs`, `call.rs` |
| Statements, `catch`, `for`, `match` | `src/check/sema/types/stmt.rs` |
| Browser names and the language's values | `src/check/sema/types/names.rs`, `handles.rs`, `src/registry/browser.rs` |
| Project scripts' JSDoc | `src/check/sema/types/jsdoc.rs`, `call.rs` |
| Built-in props | `src/registry/mod.rs` (`PropType`), `src/registry/builtins.rs` |
| Type spellings in the grammar | `src/syntax/parser/v2.rs` (`parse_type_ref`, `parse_params`) |
| Templates | `src/template/mod.rs` |
| Diagnostic codes | `src/diagnostics/codes.rs` |
| Migration | `src/cli/migrate.rs` |
| The data-fetching plan | Part II of this file |

---

## Part II — Data fetching: a redesign

Written 2026-11-15, against 6.2.0 (commit 39767cde). Nothing here is built
yet: this is the plan to approve first. It opens with a full example of
the new way; the rest explains it. Second version: the first was
turned down as not following good engineering practice. This one starts
from a list of named practices (section 4) and every decision in the design
points back to one of them.

### The new way, in one example

A small team directory: a list of people you can search and page through,
a form that adds someone and shows the server's field errors, and a button
that removes someone at once and puts them back if the server refuses. The
service is described by an OpenAPI file. Every line below is the proposed
design; none of it compiles today. Sections 4 and 5 explain each part.

#### The layout

```text
openapi.json                       the service's contract, from its team
src/
├── api/backend/
│   ├── service.wf                 yours: where the service is and how it is reached
│   └── generated/                 written by `wf api sync`; never edited
│       ├── types.wf
│       └── people.wf
├── stores/Session.wf              the signed-in user's token
├── stores/Directory.wf            the data layer: the reads and the writes
└── pages/People.wf                the view
tests/people.wf                    the page, against a fake service
```

Made with two commands, and kept current with a third:

```bash
wf api add openapi.json --name Backend
wf api sync
wf api check          # in CI: fails when generated/ is behind openapi.json
```

#### 1. The contract, generated

`src/api/backend/generated/types.wf` — every schema in the spec:

```wf
// Code generated by `wf api sync` from openapi.json. DO NOT EDIT.

/// Someone on the team.
type Person {
    id: Uuid
    name: String
    email: Email
    role: Role
    joinedAt: DateTime
}

enum Role { engineer, designer, manager }

/// What adding someone takes.
type NewPerson {
    name: String
    email: Email
    role: Role
}

/// One page of people, and where the next one starts.
type PersonPage {
    items: [Person]
    nextCursor: String?
}

/// RFC 9457 problem details, with the fields that failed.
type ValidationProblem = Problem {
    errors: [FieldError]
}

type FieldError {
    field: String
    message: String
}

/// Why getting or removing someone can fail.
enum GetPersonError {
    notFound(problem: Problem)                       on 404
    http(error: HttpError)
}

enum DeletePersonError {
    notFound(problem: Problem)                       on 404
    http(error: HttpError)
}

/// Why adding someone can be refused.
enum CreatePersonError {
    unprocessableContent(problem: ValidationProblem) on 422
    conflict(problem: Problem)                       on 409
    http(error: HttpError)
}
```

Each operation's error responses become an *error enum*: one case per
status the spec declares, named by the status (`422` is
`unprocessableContent`), its body typed, and the `http` case every error
enum has for everything else — no network, a timeout, a status the spec
does not list.

`src/api/backend/generated/people.wf` — the endpoints tagged `people`, an
interface the rest of the program depends on. Every signature is typed in
full: `-> T ! E` is "a `T`, or an `E`"; `-> T!` is "a `T`, or an
`HttpError`" for an operation that declares no errors; `-> ! E` answers with
no body:

```wf
// Code generated by `wf api sync` from openapi.json. DO NOT EDIT.

api Backend.people {
    /// People whose name matches `q`, a page at a time.
    get listPeople(q: String?, after: String?) at "people" -> PersonPage!

    get getPerson(id: Uuid) at "people/{id}" -> Person ! GetPersonError

    post createPerson(body: NewPerson) at "people" -> Person ! CreatePersonError

    delete deletePerson(id: Uuid) at "people/{id}" -> ! DeletePersonError
}
```

#### 2. The service, written by you

`src/api/backend/service.wf` — the transport, and what the spec cannot say:

```wf
// Written once by `wf api add`. Yours: `wf api sync` never changes it.

api Backend(base: env.PUBLIC_API ?? "https://api.example.com/v1") {
    timeout: 15.seconds
    credentials: .sameOrigin
    retry: .backoff(times: 3)          // GET, PUT and DELETE only: a POST is never sent twice

    // Run around every call, in this order.
    middleware [
        bearer(Session.token),         // Authorization: Bearer …
        refreshOn401(Session.refresh), // one fresh token, then the call once more
        requestId,                     // X-Request-Id
    ]

    // Which reads each write makes stale. Checked: a name that is not an
    // endpoint, or an argument it does not take, is an error.
    invalidations {
        people.createPerson     invalidates [people.listPeople]
        people.deletePerson(id) invalidates [people.listPeople, people.getPerson(id)]
    }

    // How a list endpoint pages.
    pagination {
        people.listPeople pages .cursor(param: after, next: p => p.nextCursor, items: p => p.items)
    }
}
```

`src/stores/Session.wf`:

```wf
store Session(scope: .session) {
    state token = ""

    /// A new token for the one that expired: nothing, or why not. Never throws.
    action refresh() -> ! {
        let fresh: TokenAnswer = try await http.post("/auth/refresh")   // http: not through Backend's middleware
        token = fresh.token
    }
}

type TokenAnswer { token: String }
```

#### 3. The data layer

`src/stores/Directory.wf` — the one place that reads and writes people:

```wf
store Directory {
    state query = ""

    /// Everyone matching the search, gathered a page at a time: a paged read
    /// of Person, its type taken from the endpoint.
    resource people = Backend.people.listPeople(q: query, placeholder: .previous)

    /// Add someone. The list refreshes by itself: createPerson invalidates it.
    action add(draft: NewPerson) -> Person ! CreatePersonError {
        return await Backend.people.createPerson(body: draft)
    }

    /// Remove someone now; put them back if the server could not do it.
    action remove(p: Person) -> ! DeletePersonError {
        // On a paged read, `update` changes the items it has gathered.
        let undo = Backend.people.listPeople.update(items => items.filter(x => x.id != p.id))
        let outcome = await Backend.people.deletePerson(id: p.id)
        match outcome {
            .ok                  { }
            .err(.notFound(_))   { }          // already gone: it stays gone
            .err(.http(_))       { undo() }   // not removed: put them back
        }
        return outcome
    }
}
```

#### 4. The view

`src/pages/People.wf`:

```wf
page People(path: "/people", title: "People", description: "Everyone on the team.") {
    use Directory

    state typed = ""
    state name = ""
    state email = ""
    state role: Role = .engineer
    validate name { required }
    validate email { required  email }

    Heading("People").h1

    // The read follows Directory.query; it changes once the typing stops.
    Input(bind: typed, label: "Search").search {
        on input(debounce: 300.ms) { Directory.query = typed }
    }

    // One value, one case at a time: nothing to keep in step.
    match Directory.people {
        .loading {
            Skeleton(height: "20px")
        }
        .failed(e) {                                       // e: HttpError (listPeople is PersonPage!)
            match e {
                .offline        { Alert("You are offline.").warning }
                .status(403, _) { Alert("You may not see the team.").danger }
                else            { Alert(e.message).danger }
            }
            Button("Try again") { on click { Directory.people.reload() } }
        }
        .loaded(page, refreshing, stale) {
            if refreshing { Spinner.sm }
            if let problem = stale { Text("Showing what was here: {problem.message}").muted }   // stale: HttpError?

            for p in page.items by p.id {
                Row(justify: .between) {
                    Text(p.name)
                    Text(p.email).muted
                    Button("Remove").danger {
                        on click {
                            match await Directory.remove(p) {
                                .ok                { toast("{p.name} removed") }
                                .err(.notFound(_)) { toast("{p.name} was already gone") }
                                .err(.http(e))     { toast("{p.name} is still here: {e.message}", tone: .danger) }
                            }
                        }
                    }
                }
            }
            if page.hasMore {
                Button("Load more") { on click { page.loadMore() } }
            }
        }
    }

    Heading("Add someone").h2
    Form(bind: form) {
        on submit {
            match await Directory.add(NewPerson(name: name, email: email, role: role)) {
                // Every case of Person ! CreatePersonError, or the build stops (T15).
                .ok(p)                          { toast("{p.name} added")  form.reset() }
                .err(.unprocessableContent(pr)) { form.apply(pr.errors) }       // pr: ValidationProblem
                .err(.conflict(pr))             { toast(pr.detail ?? "Already on the team") }
                .err(.http(e))                  { toast(e.message, tone: .danger) }
            }
        }
        Input(bind: name, label: "Name")
        Input(bind: email, label: "Email").email
        Select(bind: role, label: "Role") {
            Select.Option("Engineer", value: .engineer)
            Select.Option("Designer", value: .designer)
            Select.Option("Manager", value: .manager)
        }
        Button("Add", type: .submit, disabled: !form.valid).primary
    }
}
```

What happens when **Add** succeeds, with no code for it: `createPerson`
invalidates `listPeople`, so `Directory.people` refreshes in place — the
list stays on screen, `refreshing` is true for the moment the request
takes, and the new person appears with the other rows keeping their nodes.

#### 5. The test

`tests/people.wf` — the view and the data layer, against a fake that the
compiler holds to the `Backend.people` interface:

```wf
fake OneEngineer for Backend.people {
    listPeople(q, after) {
        .ok(PersonPage(items: [
            Person(id: "0b6c4c1e-7d3a-4f0e-9a51-2f1f6c0e9d11", name: "Ada", email: "ada@example.com",
                   role: .engineer, joinedAt: @2026-01-05T09:00Z),
        ], nextCursor: null))
    }
    createPerson(body) {
        if body.email.domain() == "example.com" {
            .ok(Person(id: uuid(), name: body.name, email: body.email, role: body.role, joinedAt: @2026-11-15T10:00Z))
        } else {
            .err(.unprocessableContent(ValidationProblem(title: "Invalid", status: 422,
                 errors: [FieldError(field: "email", message: "Use your work address")])))
        }
    }
    deletePerson(id) { .err(.http(.status(500, null))) }
}

test "lists the team"(fakes: [OneEngineer]) {
    People
    expect "Ada"
}

test "shows the server's field error"(fakes: [OneEngineer]) {
    People
    type "Grace" into "Name"
    type "grace@elsewhere.org" into "Email"
    click "Add"
    expect "Use your work address"
    expect called Backend.people.createPerson
}

test "puts a person back when removing fails"(fakes: [OneEngineer]) {
    People
    click "Remove"
    expect "Ada is still here"
    expect "Ada"
}
```

No test reaches the network: a call to an endpoint the fake does not
answer fails the test and names the endpoint.

#### What this example shows, against today

| | Today | Here |
|---|---|---|
| A request's state | `.state == "ready"` plus `.data`, `.error`, `.refreshing` | one value with three cases: `.loading`, `.failed(e)`, `.loaded(page, refreshing, stale)` |
| A failed call | an exception, `catch e` typed `Any` | a result (`Person ! CreatePersonError`), matched case by case; every body typed, no `Any` anywhere |
| What a call can return | `-> User`, and `errors { 422 -> Map }` that nothing checks | `-> Person ! CreatePersonError`, every status a typed case |
| An action's result | inferred, or nothing | declared: `action add(…) -> Person ! CreatePersonError` |
| Refreshing after a write | `people.reload()` by hand, or nothing | `invalidates`, declared once, checked |
| Retrying | every method, a `POST` included | `GET`, `PUT`, `DELETE` only |
| Auth and token refresh | `headers { }` and `on error { return "retry" }` | `middleware [bearer(…), refreshOn401(…)]` |
| Paging | `paginate: .page` and a `state page` the runtime writes | `pages .cursor(…)` in the contract |
| Removing at once, undoing on failure | `optimistic(…)` and a hidden rollback | `let undo = …update(…)`, `undo()` |
| The contract | inside the compiler (`from`), or written by hand | `generated/`, committed, reviewed, checked in CI |
| Testing | the real network, or the `loading` arm | a typed `fake` |

---

### In short

Today a WebFluent page gets data four ways — `resource x = fetch(…)`,
`resource x = Backend.ep(…)`, `await fetch(…)` in an action, `await
Backend.ep(…)` in an action — and each behaves a little differently. Under
them is one good engine (`http.js`). Over it, the surface breaks basic
rules: a request's state is five separate signals and a string, failures
are exceptions with string kinds, `fetch` shadows the browser's function
with a different one, `POST`s are retried, the server's `Cache-Control` is
ignored, and nothing can be tested without a network. Section 2 lists 22
defects, several of which lose or duplicate data.

The redesign, in one line each:

1. **Layers.** A *client* (where the service is and how it is reached), a
   *contract* (typed endpoints — an interface), a *data layer* (stores
   that hold reads and do writes) and *views* (pages that show them).
2. **Strongly typed, end to end.** Every parameter, body, answer and
   error body has a declared, named type; there is no `Any` or `Map` in a
   contract, a call or a fake; every signature, an action's included, is
   written down; every answer is decoded against its type at run time, in
   every build. JSON whose shape is open is `Json`, which must be matched
   before use.
3. **Types that cannot lie.** A read is one value with three cases —
   `.loading`, `.failed(e)`, `.loaded(value)` — not a string plus four
   flags. A call returns a *result*, written `T ! E` beside `T?`:
   `.ok(value)` or `.err(e)`, where `E` is the endpoint's *error enum* —
   one typed case per status it declares, and `http(HttpError)` for
   everything else.
4. **Errors are values.** No exception for an expected failure; `match`
   every case (the checker says when one is missing) or pass it up with
   `try`. Transport, status and decoding failures are different cases.
5. **The standards decide the defaults.** HTTP's method semantics (only
   idempotent requests are retried, or sent again from the offline queue),
   HTTP caching (`Cache-Control`, `ETag`, `Vary`), problem details (RFC
   9457) for error bodies, `{id}` path templates as OpenAPI writes them.
6. **Explicit, not inferred.** A write says what it invalidates, in the
   contract, checked by the compiler. Nothing is cached, shared, retried or
   refreshed because of a guess.
7. **Composition for cross-cutting concerns.** Auth, logging and token
   refresh are *middleware* — `middleware name(req, next) { … }` — not hooks with a
   magic return string.
8. **Tests substitute the contract.** A `fake` implements an `api` group;
   the compiler checks it against the interface; a call nothing fakes fails
   the test.
9. **One name, one meaning.** `fetch` is the browser's. WebFluent's ad-hoc
   request is `http.get(…)`, `http.post(…)`.
10. **`wf api`** writes the contract from an OpenAPI file — one file per
    tag, shared types apart, marked `@generated`, never edited, rewritten
    whole, checked in CI.
11. **One cut, automated.** 7.0 is the new design and nothing else:
    today's forms are errors that carry their fix, `wf migrate` rewrites a
    project in one step, and no setting brings the old behaviour back.

Section 9 breaks it into stages. Stage 1 fixes the defects that lose or
duplicate data, and is worth doing whatever else is decided.

---

### 1. How it works today

#### The four ways in

```wf
type User { id: String, name: String }

api Backend(base: "/api") {
    get  users(page: Number = 1) -> [User]
    get  user(id: String) at "users/:id" -> User
    post createUser(body: Map) -> User
        errors { 422 -> Map }
}

page Users(path: "/users", title: "Users", description: "Everyone.") {
    state page = 1
    resource rows = Backend.users(page: page)          // 1. a read, over an endpoint
    resource stats = fetch("/api/stats")               // 2. a read, over an address
    state draft = ""

    action add() {
        let u = await Backend.createUser(body: { name: draft })   // 3. a call, in an action
        let ping = await fetch("/api/ping")                        // 4. a fetch, in an action
        rows.reload()
    }
    …
}
```

#### What each compiles to

| Written | Compiled | Runtime |
|---|---|---|
| `api Backend(…) { … }` | `WF.api({ name, base, endpoints: { users: { method, path } } })` | `api.js` → one callable per endpoint, each through `send` |
| `resource r = Backend.ep(args)` | `WF.resource(Backend.ep, { call: true, args: () => ({…}) })` | `net.js` `resource` → re-runs when `args()` changes |
| `resource r = fetch(url, opts)` | `WF.resource(() => url, { …opts })` | the same, re-runs when `url()` changes |
| `await fetch(url, opts)` | `WF.request(url, opts)` | `net.js` `request` → `send`, cache `.none` |
| `await Backend.ep(args)` | `Backend.ep({…})` | `send` |
| `match r { loading … error(e) … ready(v) … }` | `WF.match(parent, () => r.state(), …)` | `match.js`, keyed on the arm |

`send` (`http.js`) is the engine: timeout, an abort that follows the
caller's signal, retries with backoff and `Retry-After`, a 256-entry cache
keyed by method, address, body and credentials, `ETag` / `If-Modified-Since`
revalidation, joining a request already in flight, upload progress over
XHR, the `on request` / `on response` / `on error` hooks, the offline write
queue, and — under `wf serve` — holding a response to its declared type.

`resource` (`net.js`) is a good piece of work: a first load is `loading`, a
refresh keeps `ready` and the data, a refresh that fails keeps the data and
sets `error`, an answer is reconciled into what was shown so keyed rows keep
their nodes, a superseded answer never lands, and leaving the page aborts
the request. None of that should be lost.

---

### 2. What is wrong

Each problem below was checked against the code and, where marked
**(probe)**, by compiling a small project with `wf 6.0` and reading the
output.

#### 2a. Bugs: things that do not do what the docs say

**B1. A positional argument to an endpoint is always called `id`.** (probe)

```wf
api Backend(base: "/api") { get user(userId: String) at "users/:userId" -> User }
resource one = Backend.user(uid)
```

compiles to `WF.resource(Backend.user, { call: true, args: () => ({ id:
_uid() }) })` (`src/web/codegen/js/net.rs`, `resource_call`: "A single
positional argument is the one the path names" — it is hard-coded to
`id`). The path wants `userId`, finds it empty, and the call ends as
`.aborted` — so the resource stays `loading` for ever. `wf check` says
nothing.

**B2. A resource's `body:` and `headers:` are read once.** (probe)

```wf
state q = ""
resource hits = fetch("/api/search", method: "POST", body: { query: q })
```

compiles to `WF.resource("/api/search", { method: "POST", body: ({ query:
_q() }) })`. `_q()` is called once, when the page draws; typing into the
search box never asks again. Only the address is followed. The tutorial's
"Send options with a request" example is exactly this page.

**B3. `.status(code, body)` cannot be matched.** (probe) The reference says
an error is `.status(code, body)`, and the runtime makes it so
(`netError` builds `["status", code, body]`), but

```wf
error(e) { match e { .status(code, body) { Text("status {code}") }  else { … } } }
```

is `T13: nothing declares code`. The checker does not know the error's
cases, so only bare `.offline` / `.timeout` arms work.

**B4. `errors { 422 -> T }` does nothing.** The parser keeps it
(`Endpoint.errors`), but `emit_api` never writes it, and the checker does
not type `e.body` with it. It is documentation that looks like a contract.

**B5. "Two readers of one address make one request" — only with a cache
policy.** `requestOptions` gives a `fetch(…)` resource `cache: { kind:
"none" }`, and `send` only joins a request in flight through a cache entry,
which `.none` never makes. Two `resource x = fetch("/api/same")` on one page
make two requests; so do a page and a component asking for the same user.

**B6. `Backend.users.invalidate()` does not refresh the screen.** It
deletes cache entries (`http.js` `invalidate`). A `resource` showing users
does not hear about it; it only asks again on `reload()`, its own
`invalidate()`, `on:` or a change of what it reads. The tutorial's
optimistic example ends with `Backend.users.invalidate()` and nothing on the
page changes.

**B7. An endpoint with no `at` is requested at its name.** `post
createUser(body: Map)` is `POST /api/createUser`. That is rarely a real
route; it is a silent default that only shows up as a 404.

**B8. Dead code.** `wfFetch` (the WebFluent 2 `fetch x from url { loading
… success … }` runtime) and `fetchOptions` in `net.js` are reachable from
nothing the current grammar writes; `("fetch", "wfFetch")` is still in the
runtime's name table (`src/web/runtime/mod.rs:150`).

The rest were found by reading the code (file and line given); they follow
from it directly but were not each run.

**B9. The editor never reads the spec.** `api Backend from "openapi.json"`
is expanded only by `read_project_collecting_via`
(`src/project/sources.rs:186`). The language server, the WebAssembly
compiler and the template engine never call `openapi::expand`, so in the
editor a spec-backed service has no endpoints: `Backend.getUser(…)` is a
false `T06`, and hover, completion and go-to-definition see nothing.

**B10. The OpenAPI reader breaks path parameters that are not one word.**
`/users/{user_id}` becomes the path `users/:user_id` and the parameter
`userId` (`openapi.rs:716` camel-cases the prop, `:727` keeps the path) —
they never meet, so the call is never sent. `{user-id}` does not match the
runtime's `:name` pattern at all.

**B11. The OpenAPI reader drops what does not fit, silently.**
- A parameter's `in` is never read: header and cookie parameters are sent
  in the query string.
- A request body's content type is dropped: multipart and form bodies are
  sent as JSON.
- A response's media type is dropped: a `text/plain` or binary answer has
  no `as:` and fails as `.parse`.
- A response that is a `$ref`, and the `default`, `4XX`, `5XX` keys, are
  dropped; so is an inline object's shape (it becomes `Map`).
- `serde_json` runs without `preserve_order`, so "the first content type"
  and "the first 2xx" mean the alphabetically first.
- Tags, security schemes, servers, `deprecated` and examples are ignored;
  an operation name is never checked to be a name (`getV1Users.json`), and
  two endpoints of one name are not reported.

**B12. An endpoint named like a list helper is rewritten.** `method_to_js`
(`src/web/codegen/js/expr.rs:59`) turns `x.remove(i)` into a splice, and it
does so for a service too: `Backend.remove(id: x)` does not call the
endpoint. So do `filter`, `first`, `contains`, `split` and the rest.

**B13. Endpoint calls are typed as if they were already awaited.**
`Backend.user(id: x)` without `await` checks as `User`, not as a promise,
so `T17` never fires for an endpoint (`check/sema/types/method.rs:137`). A
missing required parameter is not reported. An endpoint with no `->`
returns `Null` to the checker, where the docs say `Any`. A resource's
annotation is never compared with its endpoint's return type: `resource u:
[User] = Backend.user(…)` passes. `rows.relaod()` passes too: a resource's
methods are not checked.

**B14. Per-call `headers:` and `retry:` go into the query string.** The
checker allows them on a call; the runtime's list of call options
(`api.js:52`) does not have them, so they are sent as `?headers=[object
Object]`.

**B15. A `File` parameter must be called `file`.** The build records the
parameter's name (`fileField`), but `api.js:66` only looks at `given.file`;
`post avatar(id: String, image: File)` sends the image as query text.

**B16. Two uploads at once are one upload.** An async action called again
with the same arguments joins the first call; the arguments are compared
by `JSON.stringify`, and every `File` stringifies to `{}`
(`runtime/modules/busy.js:22`).

**B17. A write kept offline returns nothing, typed as something.** The call
resolves `undefined` while the checker says `User`; and when it is sent
later, a `4xx` deletes it with nothing told to the page (`offline.js:173`).

**B18. A page and a store encode a `fetch` address differently.** A page's
`await fetch("/x/{id}")` URL-encodes `id`; a store's does not
(`expr.rs:683` against `:760`).

**B19. Two copies of a dead grammar.** The WebFluent 1 `fetch x from url {
loading … success … }` node is still matched in the scope builder, the
checker, the static paint, the printer and the linters, for a parser only
`wf migrate` runs.

And three gaps, not bugs:

- A `match` over a resource always paints `loading` in the static page
  and in a template render (`ssg.rs:821`, `template/mod.rs:1479`): a PDF can
  never show fetched data, even when the data is handed in.
- No browser suite exercises an `api`, an endpoint, `optimistic` or `await
  fetch`; no project in `examples/` uses `api` or `resource`; `openapi.rs`
  has no unit tests of its own.
- `wf verify` fails every page whose API is not reachable, with no way to
  answer for it.

#### 2b. Design problems

**P1. Two homes for server data.** A page can hold a list as a `resource`
(read-only, refreshed, cached) or as a `state` filled by a `load()` action
(writable, never refreshed). The tutorial teaches both, and the second is
what anyone who needs to *change* the list ends up with:

```wf
store Todos {
    state items: [Todo] = []
    action load() { items = await fetch("/api/todos") }
    action add(t: String) {
        let created = await fetch("/api/todos", method: "POST", body: { title: t })
        items.push(created)
    }
}
```

That is a hand-written cache: no refresh on focus, no dedupe, no stale
marker, and the page and the server drift. Every library that has made data
fetching pleasant (TanStack Query, SWR, RTK Query, Apollo, Remix) has the
same answer: the server's data lives in one cache, reads subscribe to it,
writes tell it what changed. WebFluent has the cache and the reads — it is
missing the writes.

**P2. `fetch` means three things.** In a `resource` it is a declarative
read whose options are syntax (`parse_resource` looks for the word). In an
action it is `WF.request`, a one-off call that returns parsed JSON and
throws. In a project script it is the browser's. A `resource` cannot be
anything else: `resource n = Stats.compute(range)` (an async store action)
or `resource n = loadFromScript(id)` is a parse error, because the right
side must be `fetch(…)` or a capitalised call.

**P3. Options are spread over five places, with different rules.** A
policy may be written on the service (`timeout:`), on the endpoint
(`cache:`), on the call (`cache:`, `retry:`), on the resource (`on:`,
`paginate:`) or on `fetch(…)`; which ones follow state differs (B2), and
`on:` / `paginate:` are only allowed in a resource, which the checker does
not say.

**P4. Errors are typed in prose only.** `e` in `error(e)` is a `NetError`
whose case names are checked but whose payloads bind nothing (B3), and in
`catch e` it is `Any`, so `e.mesage` passes there. `.error` is `Any?` in
one table of the checker and `NetError?` in another. Nothing distinguishes "the network is down" from "the server said
no" from "the server said something we could not read" at the type level,
which is the distinction libcurl, reqwest and httpx all make first.

**P5. Every failure in an action is an exception.** A form that wants to
show a `422`'s field errors writes `try { … } catch e { if e.status == 422 {
form.apply(e.body) } else { … } }`. There is no way to get the outcome as a
value, the way `Result` does in Rust or `{ data, error }` does in
openapi-fetch.

**P6. Pagination is spelled with a borrowed word.** `paginate: .page`
uses the enum case `.page` as *the name of the argument*, needs a `state
page` the runtime then writes to, knows only page numbers, and learns that
the list ended by asking for an empty page. Cursor APIs and `Link` headers
— most real APIs — cannot be expressed.

**P7. Retrying after a `401` returns a magic string.** `on error(e) { …
return "retry" }`: any other string, or a typo, silently does not retry.

**P8. Optimistic updates hold the wrong thing.** `optimistic(holder, f)`
takes a `state`; a `resource`'s data cannot be its holder, so an optimistic
change forces P1's hand-written cache.

**P9. Nothing to test with.** `wf test` cannot fake a response, so a test
of a page that fetches sees the `loading` arm, or the real network. A
template render cannot hand a resource its answer.

**P10. The OpenAPI import is all or nothing, and invisible.** `api Backend
from "openapi.json"` builds the whole surface inside the compiler — and
not even the editor sees it (B9). You
cannot read the endpoint list in the repository, review what a spec change
did to it in a diff, split it by area, or fix one endpoint the spec gets
wrong without forking the spec.

**P11. A request's state is a string and four flags.** A resource is
`.state` (`"loading"`, `"ready"` or `"error"`), `.data`, `.error`,
`.refreshing` and `.pending`, each its own signal. What they mean is in
their combinations — `"ready"` with an `error` is "stale"; `"loading"`
with `data` is impossible but representable — and a typo in `users.state ==
"redy"` is a comparison that is always false. The cure is the one every
typed language uses: one sum type whose cases carry what they need.

**P12. Two spellings for one idea.** A resource's arms are bare words
(`loading`, `error(e)`, `ready(v)`); every enum's arms are dotted cases
(`.idle`). An error's case is a string in `e.kind` and a dotted case in a
`match`.

**P13. `fetch` is not `fetch`.** In WebFluent `await fetch(url)` returns
parsed JSON and throws on a `404`; the browser's returns a `Response` and
does not. Anyone who knows the platform reads it wrong, and the docs have
to explain `window.fetch`.

**P14. The service mixes transport and contract.** One `api` block holds
where the service is, its timeout and retry, its auth headers, its hooks
*and* every endpoint. There is no way to describe the endpoints without the
transport, and so nothing a test can substitute.

#### 2b′. Defects found against the standards

**B20. Writes are retried.** `retriable` (`http.js:84`) looks at the error,
never the method: with `retry: .backoff(times: 3)` on a service, a `POST`
that times out after the server took it is sent three more times. HTTP
(RFC 9110 §9.2.2) allows automatic retries only for idempotent methods.

**B21. The offline queue resends writes blindly.** Any method but `GET` and
`HEAD` that failed for want of a network is kept and sent again later
(`http.js:319`) — including a `POST` whose request reached the server and
whose answer was lost. With no idempotency key, that is a duplicate order.

**B22. The server's caching headers are ignored.** The cache's lifetime is
whatever the page says (`cache: .swr(60.seconds)`) or nothing; `Cache-Control:
no-store`, `private`, `max-age` and `Vary` are not read (RFC 9111). A
`no-store` answer about one user can be cached and shown to the next, if a
page asked for `.swr`.

#### 2c. What is good and stays

- The engine's mechanics: timeouts, backoff with jitter and `Retry-After`,
  `ETag` revalidation, joining a request already in flight, upload
  progress, the cache keyed by credentials.
- A read's behaviour: a first load against a refresh that keeps the page,
  answers reconciled so keyed rows keep their nodes, a superseded answer
  dropped, the request aborted when its scope leaves.
- One declaration per service, and reading it from OpenAPI.
- Connections (`socket`, `stream`, `channel`, `peer`, `worker`): a
  different job, out of scope.

---

### 3. What others do, and what fits WebFluent

#### JavaScript and TypeScript (weighted most)

| Design | The idea | Take it? |
|---|---|---|
| `fetch` + `AbortController` | A request is a promise; cancellation is a signal you pass. Transport failure rejects, an HTTP error status does not. | Keep `window.fetch` raw. Our abort-by-scope is already better than a hand-passed signal. |
| async / await | Sequential code over promises. | Already ours in actions. |
| **TanStack Query** | *Queries* (reads, keyed, cached, deduped, refetched on focus/reconnect, stale-while-revalidate by default) vs *mutations* (writes, `onMutate` / `onError` / `onSettled`), and `invalidateQueries(key)` refetches the active queries under a key prefix. `isLoading` vs `isFetching`. Infinite queries take `getNextPageParam`. | **Yes, the split.** Reads are `resource`, writes are actions; invalidation reaches the screen. Not its client-chosen `staleTime`: we cache what HTTP says (R4). |
| SWR | The key *is* the URL; `mutate(key)` revalidates everywhere it is read; dedupe within a window. | Joining identical requests in flight, yes. A URL as the cache key, yes, with `Vary` (RFC 9111). |
| **RTK Query** | Endpoints declared once as `query` or `mutation`; `providesTags` / `invalidatesTags` connect them, so a mutation refetches the right queries with no call-site code. | **Yes — closest to our `api` block.** `invalidates [...]` on a write endpoint, declared and checked; no inferred default. |
| Apollo | A normalised cache by `id`: a mutation's returned object updates every query holding it. | No: `reconcile` already keeps rows by `id`; invalidate-and-refresh is enough. |
| tRPC | End-to-end types: the client is the server's types. Query vs mutation is in the procedure. | The idea that a call site is typed from the contract — we have it through `api`/OpenAPI. |
| Remix / React Router loaders + actions | Reads run before render; after any action, every loader on the page revalidates. Forms work without JS. | The read/write split, yes. Refreshing *every* read after any write, no: it hides what depends on what. We have no server, so loaders stay out. |
| Next.js `fetch(…, { next: { tags } })` + `revalidateTag` | Tags on reads, invalidated by name. | Same as RTK's tags. |
| React Suspense / `use(promise)` | Reading a pending value suspends to the nearest boundary. | No: `match` is explicit and reads better in a page; Suspense's implicit boundaries are what people find hardest. |
| openapi-typescript / **openapi-fetch** | Types only, generated from the spec; calls are `client.GET("/users/{id}", …)` and return `{ data, error }` with `error` typed per status. | Yes: a call returns a result, its error typed per status (`HttpError.status(code, body)`). |
| **orval** | Generates a client from OpenAPI, `tags-split` mode writes one file per tag plus a schemas folder, and can generate MSW mocks from examples. | **Yes — the shape of `wf api`**; mocks become typed `fake`s, answered from the spec's `example`s on request. |

#### Rust

| Design | The idea | Take it? |
|---|---|---|
| reqwest | A client with defaults (base, timeout, headers); a builder per request; a non-2xx is *not* an error until you call `.error_for_status()`; `.json::<T>()` decodes to a type. | The client-with-defaults is our `api`. A decode failure is its own error (`.decode`). |
| `Result<T, E>` and `?` | Errors are values; `?` passes them up; `match` takes them apart. | Yes: every call returns a result (`T!`); `try` is our `?`. |
| `async` / `.await`, cancellation by drop | Dropping a future cancels it. | Ours: leaving a scope cancels its reads. |
| serde | The type *is* the decoder; a missing field is an error at the boundary. | Yes: the declared type checks every answer, in every build. |
| progenitor | Generates a typed client from OpenAPI, with `ResponseValue<T>` and `Error<E>` typed per status. | Same lesson as openapi-fetch. |

#### Python

| Design | The idea | Take it? |
|---|---|---|
| requests / httpx | `Session`/`Client` with `base_url`, auth, hooks; `raise_for_status()`; httpx has a **default timeout** (5 s) because "no timeout" hangs. | Yes: a default timeout (30 s) when a service says none. Hooks become middleware (httpx transports). |
| pydantic | Validate at the boundary; a bad payload is a typed error naming the field (`users.1.name: field required`). | Yes, in every build, as `.decode(at)`. |
| openapi-python-client | One module per tag (`api/users/get_user.py`), `models/`, sync and async variants, and a `_detailed` variant returning status + parsed body. Regenerates by overwriting the package. | Module-per-tag, and overwriting a generated package whole, yes. |

#### C

| Design | The idea | Take it? |
|---|---|---|
| libcurl | Everything explicit: an easy handle per transfer, a multi handle for many; `CURLcode` for *transport* errors kept apart from the HTTP *status* (`CURLOPT_FAILONERROR` merges them, on request). | The lesson: transport error, HTTP status and decoding are three different things. `HttpError` keeps them apart. Its explicitness, in the defaults: nothing happens to a request the code or the server did not ask for. |

What does *not* fit: Suspense-style implicit loading (WebFluent is explicit
and pre-renders the loading case); a normalised entity cache (Apollo —
too much machinery for the gain); loaders that run on a server (WebFluent
builds static files).

---

### 4. The practices this design follows

Each is a rule the design keeps everywhere; section 5 cites them as
**[R1]**…**[R12]**.

| | Practice | Where it comes from | What it rules out here |
|---|---|---|---|
| R1 | **Make impossible states unrepresentable.** A value that can be in one of several states is one sum type, each case carrying what it needs. | Elm's `RemoteData`, Rust enums, "parse, don't validate" | `.state` strings beside `.data`/`.error`/`.refreshing` flags (P11) |
| R2 | **Errors are values.** An expected failure is returned, handled with `match` or passed up explicitly; exceptions are for bugs. | Rust `Result` and `?`, Go, openapi-fetch's `{ data, error }`, Swift `throws` with `try` | throwing on a `404`; `catch e` typed `Any` (P4, P5) |
| R3 | **Explicit over implicit.** Behaviour that changes data — caching, sharing, retrying, invalidating — happens because code or the server said so. | The Zen of Python; RTK Query's declared tags | inferring invalidation from paths; a client-chosen cache lifetime by default |
| R4 | **Follow the standards.** | RFC 9110 (method semantics), RFC 9111 (caching), RFC 9457 (problem details), the IETF Idempotency-Key draft, RFC 6570 / OpenAPI path templates | retrying `POST` (B20); resending it offline (B21); ignoring `Cache-Control` (B22); `:id` paths that differ from the spec's `{id}` |
| R5 | **Validate at the boundary.** Data from outside is decoded into a declared type once, where it enters; a mismatch is an error value naming the place. | serde, pydantic, zod | dev-only shape checks; `Any` from `await fetch` |
| R6 | **Separation of concerns, in layers.** Transport, contract, data, view; each knows only the one below it. | Clean / hexagonal architecture, the repository pattern | a page building `"/api/users/{id}"`; one block mixing timeouts and endpoints (P14) |
| R7 | **Depend on interfaces; substitute in tests.** | Dependency inversion (SOLID), test doubles | tests that need the network or string-matched URL interception (P9) |
| R8 | **One name, one meaning; do not shadow the platform.** | Principle of least surprise | WebFluent's `fetch` meaning something other than the browser's (P13); two spellings of a case (P12) |
| R9 | **Composition over configuration** for cross-cutting concerns. | Middleware in Koa, ky, tower (Rust), httpx transports | hooks whose return value is the string `"retry"` (P7) |
| R10 | **Structured lifetimes.** Work belongs to a scope and ends with it, unless it must not (a write). | Structured concurrency (Trio, Kotlin), `AbortController` | reads that outlive their page; writes cut off by navigation |
| R11 | **Generated code is never edited.** Marked, regenerated whole and deterministically, checked in CI; customisation lives beside it. | Go's `// Code generated … DO NOT EDIT.`, protobuf, orval, `linguist-generated` | hand-patched clients that drift; clever regeneration that merges edits |
| R12 | **Change in one release, never silently; automate the migration.** A breaking change lands whole in a major, every refused form says what to write and carries the fix, and `wf migrate` applies them all. No setting keeps the old behaviour and no release accepts both. | Semantic versioning, Rust editions, `go fix` | a release that silently changes what working code does; a mode in which the language is less strict |
| R13 | **Strong, static typing.** Every value has one type the checker knows and the source can name; signatures are declared and bodies inferred; no value converts implicitly; data from outside is checked against its type when it arrives. | Rust, TypeScript's `strict`, Haskell, Elm | `Any`, `Map` and untyped `catch e` at the boundary; a call whose result type cannot be written; a response trusted without being checked |

Beside these, secure defaults throughout: a timeout on every request,
credentials sent to the same origin only, no token in `persist`.

---

### 5. The design

#### 5.1 The layers

```text
 view        pages and components    show reads, call actions
   │
 data        stores                  own the reads (resource) and the writes (actions)
   │
 contract    api groups              typed endpoints: input, output, errors, what a write invalidates
   │
 transport   api service             base address, timeout, retry, cache, middleware
```

**[R6]** A page may still declare a `resource` over an endpoint — small
sites need no store — but it never builds a URL: an address written in a
page or component is a warning (`N10`) that says to put it in a contract.
Ad-hoc requests (5.10) are for scripts and prototypes.

#### 5.2 Types: strong, static, written down

**[R13]** A request is where a program meets data it did not make, so this
is where typing matters most. The rules, everywhere in the data layer:

1. **Every value that crosses the boundary has a declared, named type**:
   a parameter, a body, an answer, an error's body. `Any` and `Map` are not
   allowed in a contract, an `http` call or a fake (`N11`, an error). JSON
   whose shape really is open is `Json` — a value you must `match`
   (`.object`, `.array`, `.string`, `.number`, `.bool`, `.null`) before you
   use it, as Rust's `serde_json::Value` is — never a value that agrees with
   everything.
2. **Every type can be written.** Nothing the checker knows about a value is
   kept somewhere source cannot name, so a value can always be stored,
   passed and declared without losing its type.
3. **Signatures are declared; bodies are inferred.** An action that returns
   a value says what, as an endpoint already does: `action add(p:
   NewPerson) -> Person ! CreatePersonError`. Inside it, `let` and
   expressions are inferred. An action that returns nothing declares
   nothing.
4. **Checked twice.** At build time by the checker; at run time every
   answer is decoded against its type, in every build (R5). A value typed
   `Person` *is* a `Person`.
5. **No implicit conversions.** A result is not its value until it is
   matched or passed through `try`; a `T?` is not a `T` (`T04`, as today);
   no number is read from a string.

The spellings, beside the ones the language already has — no angle
brackets **[R8]**:

| Written | Means |
|---|---|
| `[T]` | a list of `T` (today) |
| `T?` | a `T`, or `null` (today) |
| `T ! E` | a `T`, or an error of the error enum `E` |
| `T!` | a `T`, or an `HttpError` — for a call that declares no errors of its own |
| `! E`, `!` | nothing, or an error — only as a declared result |

**An error enum** is an enum with one case that holds an `HttpError`, and
whose other cases each say which status they stand for:

```wf
/// Why adding someone can be refused.
enum CreatePersonError {
    unprocessableContent(problem: ValidationProblem) on 422
    conflict(problem: Problem)                       on 409
    http(error: HttpError)                           // everything else
}
```

The checker holds an error enum to that shape: one `http(…)` case, each
other case one payload and one status, no status twice (`N12`). Because
every error enum has its `http` case, a transport failure always has
somewhere to go, and `try` can pass one up from any call (5.6).

**A result** (`T ! E`) has two cases: `.ok(value)` — `.ok` alone when
there is no value — and `.err(e)`, `e` an `E`.

**A read** of an endpoint `-> T ! E` is one value with three cases:

| Case | Meaning |
|---|---|
| `.loading` | nothing to show yet |
| `.failed(e)` | the first load failed; `e` is an `E` |
| `.loaded(value, refreshing, stale)` | something to show; `refreshing` while it is asked again; `stale` is the `E?` of a refresh that failed |

**[R1]** Every state a read can be in is exactly one case. "Loaded, and the
last refresh failed" is `.loaded(value, false, e)`; "loading, with data"
cannot be written. The case replaces `.state`, `.data`, `.error` and
`.refreshing`. To read one fact without a `match`, a read has members
computed from the one value — `users.value` (`T?`), `users.isLoading`,
`users.isRefreshing`, `users.error` (`E?`) — which cannot disagree with
it. A read is annotated with its value's type, as today: `resource stats:
Stats = http.get("/api/stats")`; the checker compares the annotation with
the endpoint's, and a mismatch is `T01`.

Patterns, the same for every enum:

- a `match` on a result, a read or an error enum must cover every case or
  say `else` (`T15`, as for any enum);
- an arm may bind fewer values than its case carries, and `_` binds
  nothing;
- a case pattern may hold a literal and nest: `.err(.http(.status(503, _)))`;
- `if let .err(e) = outcome { … }` takes a case pattern as `match` does.

```wf
match users {
    .loading                  { Skeleton(height: "20px") }
    .failed(e)                { ErrorPanel(e) { on retry { users.reload() } } }
    .loaded(list, refreshing) {
        if refreshing { Spinner.sm }
        for u in list by u.id { UserRow(u) }
    }
}
```

#### 5.3 `HttpError`

What went wrong between the page and the server, kept apart as libcurl and
reqwest keep them **[R4, R5]**:

| Case | When |
|---|---|
| `.offline` | no network |
| `.timeout(after: Duration)` | no answer in time |
| `.cancelled` | its scope left, or `.cancel()` |
| `.transport(message: String)` | DNS, TLS, CORS, a refused connection |
| `.decode(at: String, message: String)` | the answer is not the declared type: `$[1].name is missing` |
| `.status(code: Number, problem: Problem?)` | a status the endpoint does not declare; `problem` is the body when it is `application/problem+json` (RFC 9457), else `null` |

Every `HttpError` has `.message`, for showing. A declared status never
reaches `.status`: it is its own case of the endpoint's error enum, with
its own typed body. So there is no `Any` anywhere in a failure.

`Problem` is built in, as RFC 9457 defines it: `type: Url?`, `title:
String?`, `status: Number?`, `detail: String?`, `instance: Url?`, and
`extensions: Json`. A project extends it like any record: `type
ValidationProblem = Problem { errors: [FieldError] }`.

```wf
match await Backend.people.createPerson(body: draft) {
    .ok(person)                        { navigate("/people/{person.id}") }
    .err(.unprocessableContent(p))     { form.apply(p.errors) }               // p: ValidationProblem
    .err(.conflict(p))                 { Alert(p.detail ?? "Already there").warning }
    .err(.http(.offline))              { toast("You are offline.") }
    .err(.http(e))                     { toast(e.message, tone: .danger) }
}
```

#### 5.4 The transport: a service

```wf
api Backend(base: env.PUBLIC_API ?? "/api/v1") {
    timeout: 10.seconds                       // default 30 s when unset
    credentials: .sameOrigin
    retry: .backoff(times: 3)                 // applied to idempotent requests only
    middleware [bearer(Session.token), refreshOn401(Session.refresh), requestId]
}
```

**Middleware [R9].** A middleware is a function of the request and the rest
of the chain, returning the result — the shape Koa, ky and Rust's tower
share. The built-ins cover the common cases (`bearer(token)`,
`header(name, value)`, `refreshOn401(action)`, `requestId`, `log`); a
project writes its own with `middleware`, whose two parameters are typed
by what a middleware is — as a handler's are by its event — and which
returns what `next` returns:

```wf
store Session {
    state token = ""
    action refresh() { … }      // uses `try`, so it returns a result
}

/// Retry once with a fresh token when the server says the old one expired.
middleware refreshOn401(req, next) {
    let first = await next(req)
    if let .err(.status(401, _)) = first {
        try await Session.refresh()
        return await next(req)
    }
    return first
}
```

The order is the order written; each runs around the next. This replaces
`headers { }`, `on request`, `on response` and `on error` with one
mechanism, and the `"retry"` string with a call **[R8, R9]**. Concurrent
`401`s share one refresh because an async action called again with the
same arguments joins the running call.

**Retries [R4].** A retry policy applies to safe and idempotent methods
(`GET`, `HEAD`, `OPTIONS`, `PUT`, `DELETE`) and to nothing else, with
exponential backoff, full jitter and `Retry-After`. A `POST` or `PATCH` is
retried only when its endpoint is marked `idempotent`, which sends an
`Idempotency-Key` made once per call and kept across its retries. Fixes B20.

**Caching [R3, R4].** The client is an HTTP cache as RFC 9111 describes for a
private cache: it stores a `GET` answer only when `Cache-Control` allows,
for as long as `max-age` says, serves it stale only within
`stale-while-revalidate`, revalidates with `ETag` / `Last-Modified`,
honours `no-store`, `no-cache` and `Vary`, and keys by credentials too.
With no caching headers nothing is stored. A contract may override the
server for one endpoint (`cache: .maxAge(60.seconds)`), and the override is
visible where the endpoint is declared. Identical `GET`s in flight are
always joined: they are safe by definition. Fixes B5 and B22.

#### 5.5 The contract: groups of endpoints

A service's endpoints are declared in *groups*, each in its own file if
wanted, each an interface a test can substitute **[R6, R7]**:

```wf
enum UserLookupError {
    notFound(problem: Problem)  on 404
    http(error: HttpError)
}

enum CreateUserError {
    unprocessableContent(problem: ValidationProblem)  on 422
    http(error: HttpError)
}

api Backend.users {
    /// Every user, a page at a time.
    get list(page: Number = 1, q: String?) at "users" -> [User]!

    get byId(id: Uuid) at "users/{id}" -> User ! UserLookupError

    post create(body: NewUser) at "users" -> User ! CreateUserError
        invalidates [list]

    patch rename(id: Uuid, body: Rename) at "users/{id}" -> User ! UserLookupError
        idempotent
        invalidates [list, byId(id)]

    delete remove(id: Uuid) at "users/{id}" -> ! UserLookupError
        invalidates [list, byId(id)]

    get avatar(id: Uuid) at "users/{id}/avatar" -> Blob!
    get search(q: String, tenant: String in header "X-Tenant") at "users/search" -> [User]!
}
```

What each part says, and the rule behind it:

- **`at` is required** and uses `{name}` as OpenAPI and RFC 6570 do **[R3,
  R4]**. No path from the endpoint's name (B7). Every `{name}` must be a
  parameter, and a path parameter may never be empty, `.` or `..` — checked
  at the call when the value is known, else the call ends
  `.err(.http(.cancelled))` with a message, as today.
- **Where a parameter goes is declared, not guessed.** In the path when the
  path names it, in the body when it is `body`, `in header "X-…"` or `in
  cookie` when it says so, otherwise in the query **[R3]**. A positional
  argument is the *first* parameter, whatever its name (fixes B1).
- **The whole signature is typed [R13]**: every parameter, the answer and
  its error enum, written `-> T ! E` (or `-> T!` when the endpoint declares
  no statuses of its own, `-> ! E` when it answers with no body). `Any` and
  `Map` are `N11`, an error. The answer's media type follows the type: a
  record or a list is JSON, `String` is text, `Blob` and `Bytes` are binary
  — no separate `as:` (fixes B11's media types).
- **`invalidates [...]`** names the reads a successful write makes stale,
  by endpoint and, optionally, arguments; the compiler checks every name
  and argument **[R3]**. With no `invalidates`, a write invalidates nothing.
- **`idempotent`** allows retries and offline replay of a `POST`/`PATCH`,
  with an `Idempotency-Key` **[R4]**.
- A group has no transport settings of its own: it is reached through its
  service.

`api Backend from "openapi.json"` stays as a one-line way to get the same
groups from a spec; `wf api` (section 6) writes them out as files.

#### 5.6 Calling an endpoint

`await Backend.users.create(body: draft)` is a `User ! CreateUserError`.
Nothing throws **[R2]**, and nothing is left untyped **[R13]**. Three ways
to use it:

```wf
// 1. Handle every case here. The checker says if one is missing.
action submit() {
    match await Backend.users.create(body: draft) {
        .ok(u)                          { navigate("/users/{u.id}") }
        .err(.unprocessableContent(p))  { form.apply(p.errors) }
        .err(.http(e))                  { toast(e.message, tone: .danger) }
    }
}

// 2. Pass it up with `try`. The action declares what it returns.
action invite(email: Email) -> User ! CreateUserError {
    let user = try await Backend.users.create(body: NewUser(email: email))
    try await Backend.mail.welcome(id: user.id)      // -> !: its HttpError becomes .http(…)
    return user
}

// 3. Let the action hold it: every action already has `.pending`; it gains
//    `.result`, typed `(User ! CreateUserError)?`: the last outcome, or null.
Button("Invite", loading: invite.pending) { on click { invite(email) } }
if let .err(.unprocessableContent(p)) = invite.result { Alert(p.title ?? "Check the form").danger }
```

**`try expr`** is Swift's and Zig's word for Rust's `?`: it gives the `.ok`
value, or returns the `.err` from the enclosing action. Its rules keep the
types exact:

- the action must declare a result with an error enum (`-> T ! E`, `-> ! E`,
  `-> T!`, `-> !`); `try` in an action that declares none is `N03`;
- an error of the same enum is passed up as it is;
- a bare `HttpError` (from a call typed `T!` or `!`) becomes the action's
  `.http(e)`;
- an error of another enum is passed up only through its `http` case,
  which every error enum has: `.http(e)` becomes the action's `.http(e)`.
  Any other case of the other enum must be handled first — `try` on a call
  whose `E` has cases the action's enum does not is `N03`, naming them;
- in a handler, `try` passes the error to the page's error reporting (the
  dev overlay under `wf serve`, `console.error` otherwise).

`try { } catch` stays for exceptions a project script throws, and its `e`
is the script's `Error`, not `Any`.

**Lifetimes [R10].** A call that only reads (a safe method) is cancelled
when the scope that started it leaves; a write runs to its end, so
navigating away never half-saves.

#### 5.7 Reads: `resource`

```wf
store Directory {
    state page = 1
    state query = ""
    resource users = Backend.users.list(page: page, q: query)   // a read of [User]
    action next() { page = page + 1 }
}
```

- The right side is a call of an endpoint (or `http.get`, 5.10). Every
  argument is followed: when `page` or `query` changes, the read asks again
  (fixes B2) **[R1]** — as a first load (`.loading`) because the answer
  would be about something else, unless `placeholder: .previous` keeps the
  last value shown while the new one loads.
- `refresh:` says when to ask again besides that: `.onReconnect` (the
  default), `.onFocus`, `.every(30.seconds)`, `.never`.
- Members: `reload()`, `cancel()`, and the read-only views of 5.2.
- **Invalidation reaches the screen.** When a write invalidates `list`, or
  code calls `Backend.users.list.invalidate()`, every read of it that is
  shown refreshes in place: `.loaded(value, refreshing: true)`, then the new
  value, reconciled so rows keep their nodes (fixes B6).
- A read's request is cancelled when its scope leaves **[R10]**.
- A `resource` over a write endpoint is an error (`N02`): a write is an
  action.

#### 5.8 Optimistic updates

Explicit, with an undo handle, no hidden rollback **[R3]**:

```wf
// put setDone(id: Uuid, body: SetDone) at "todos/{id}/done" -> Todo!
action toggle(t: Todo) -> ! {
    let undo = Backend.todos.list.update(list => list.map(x => if x.id == t.id { Todo(id: x.id, title: x.title, done: !x.done) } else { x }))
    if let .err(e) = await Backend.todos.setDone(id: t.id, body: SetDone(done: !t.done)) {   // e: HttpError
        undo()
        toast(e.message, tone: .danger)
        return .err(e)
    }
}
```

`endpoint.update(args?, change)` changes what every read of it shows and
returns the function that puts it back. On a paged read it changes the
items gathered so far: `update(items => items.filter(…))`. On success the write's
`invalidates` refreshes it from the server. Replaces `optimistic` and the
implicit `attempt` rollback; the old form is an error in 7.0 whose fix
writes the new one **[R12]**.

#### 5.9 Pagination

How an endpoint pages is part of its contract, not of each call **[R6]**:

```wf
api Social.posts {
    get feed(after: String?) at "feed" -> FeedPage
        pages .cursor(param: after, next: p => p.nextCursor, items: p => p.posts)
    get list(page: Number = 1) at "posts" -> [Post]
        pages .number(param: page)                 // ends at the first empty page
    get search(q: String) at "search" -> [Post]
        pages .link                                // the Link header's rel="next" (RFC 8288)
}

resource feed = Social.posts.feed()               // a paged read of Post
```

For a generated contract, the same line goes in `service.wf`'s
`pagination { }` (section 6, rule 6).

A paged read's value has `items`, `hasMore` and `loadMore()`; the read writes no
state of the page's (P6). A numbered pager (`Pagination(bind: page)`)
pages nothing: each page is its own read, and going back is a cache hit
when the server allows caching.

#### 5.10 Ad-hoc requests: `http`

For a script, a prototype or a one-off, the built-in service `http` has
`get`, `post`, `put`, `patch` and `delete`, with the same results and rules
as any endpoint, and a type for the answer:

```wf
resource stats: Stats = http.get("/api/stats")             // a read of Stats
let created: Todo = try await http.post("/api/todos", body: draft)
```

A call with no declared type is `N11`. `fetch` is the browser's `fetch`,
untouched, in every context **[R8]** (fixes P13).

#### 5.11 Streaming

An endpoint that answers line by line (NDJSON, a log tail) is marked
`streamed`, and returns the list its lines make: `get tail(id: String) at
"logs/{id}" -> [LogLine] streamed`. A read over it is `.loaded` from the
first line, its list growing as lines arrive,
and an action can walk it with `for await line in …`. Sockets and
server-sent events stay as they are.

#### 5.12 Offline

Only a request that may safely be sent twice is kept and replayed: a `GET`
is never queued (it has no effect to replay), and a write is queued only
when it is idempotent — `PUT`, `DELETE`, or `idempotent` with its key
**[R4]** (fixes B21). A queued write's call returns
`.err(.offline(queued: true))`, so the page knows the answer did not come
(fixes B17); when it is finally sent, its `invalidates` are applied; when
the server then refuses it, `network.refused` lists it.

#### 5.13 Testing: fakes

A `fake` implements a group, and the compiler checks it against the
group's interface: every endpoint it answers, every argument and every
result type **[R7]**.

```wf
fake UsersWithOne for Backend.users {
    list(page, q) { .ok([User(id: "0b6c4c1e-7d3a-4f0e-9a51-2f1f6c0e9d11", name: "Ada")]) }
    create(body) {
        if body.email.domain() == "example.com" { .ok(User(id: uuid(), name: "New")) }
        else { .err(.status(422, ValidationProblem(errors: [FieldError(field: "email")]))) }
    }
}

test "rejects an outside address"(fakes: [UsersWithOne]) {
    SignUp
    type "ada@elsewhere.org" into "Email"
    click "Join"
    expect "email"
    expect called Backend.users.create
}
```

- A call to an endpoint no fake answers fails the test, naming it; no test
  reaches the network.
- `expect called` / `expect not called` check that a write went out.
- With a spec, `fake … from examples` answers from the spec's `example`s.
- The same fakes serve `wf verify --fakes tests/fakes.wf` for a site whose
  API is elsewhere, and a `wf render` template is handed a read's value in
  its data: `{ "users": { "loaded": [ … ] } }`, painting `.loaded`.

The static page still paints `.loading` for a read it cannot know.

---

### 6. The new command: `wf api`

#### What it is for

`api Backend from "openapi.json"` builds the contract inside the compiler,
where nobody can read, review or diff it (P10, B9). `wf api` writes the same
contract as files, following the rules every mature generator follows
**[R11]**: generated files are never edited, they are rewritten whole, the
output is deterministic, and CI checks it is current.

#### Commands

```bash
wf api add openapi.json --name Backend [--out src/api/backend]
wf api sync [Backend]        # rewrite the generated files from the spec
wf api check [Backend]       # exit 1 when sync would change anything (CI)
wf api eject Backend         # turn `api Backend from "…"` into files
wf api list [--json]         # services, their specs, groups and endpoints
```

`add` records the service in `webfluent.app.json`:

```json
{ "apis": { "Backend": { "spec": "openapi.json", "out": "src/api/backend" } } }
```

`wf api` reads a local file only: the compiler never fetches, and `curl -o
openapi.json …` is one line.

#### What it writes

For a spec with the tags `users` and `billing`:

```text
src/api/backend/
├── service.wf          yours: written once by `add`, never by `sync`
└── generated/          `sync` owns this folder entirely
    ├── types.wf        every schema: types and enums
    ├── users.wf        api Backend.users { … }
    ├── billing.wf      api Backend.billing { … }
    └── default.wf      endpoints with no tag, when there are any
```

Every generated file begins with the marker Go, protobuf and GitHub
recognise:

```wf
// Code generated by `wf api sync` from openapi.json. DO NOT EDIT.
```

and `add` writes `src/api/backend/generated/** linguist-generated=true` to
`.gitattributes`, so a review collapses the diff.

`service.wf`, which the author owns:

```wf
// Written once by `wf api add`. Yours: `wf api sync` never changes it.
api Backend(base: env.PUBLIC_API ?? "https://api.example.com/v1") {   // from servers[0]
    timeout: 30.seconds
    middleware [bearer(Session.token)]   // the spec's security scheme is `bearerAuth`
}
```

`generated/users.wf`:

```wf
// Code generated by `wf api sync` from openapi.json. DO NOT EDIT.

api Backend.users {
    /// Every user, a page at a time.
    get listUsers(page: Number = 1, q: String?) at "users" -> [User]!
    get getUser(id: Uuid) at "users/{id}" -> User ! GetUserError
    post createUser(body: NewUser) at "users" -> User ! CreateUserError
}
```

and in `generated/types.wf`, beside the schemas:

```wf
enum GetUserError {
    notFound(problem: Problem)                       on 404
    http(error: HttpError)
}

enum CreateUserError {
    unprocessableContent(problem: ValidationProblem) on 422
    http(error: HttpError)
}
```

#### Rules it follows

1. **Names are the spec's** **[R8]**: the `operationId` as written, made a
   WebFluent name only where it must be (`list-users` → `listUsers`), and
   no cleverer than that — no stripping of tag names. Without an
   `operationId`, the method and the path (`get /users/{id}/keys` →
   `getUsersIdKeys`), with a warning to add one. A name the spec gives twice
   is an error naming both operations. Types keep their schema names.
   An operation's error enum is its name and `Error` (`CreateUserError`);
   each case is the status's reason phrase, camelCased (`404` `notFound`,
   `409` `conflict`, `422` `unprocessableContent`); a `4XX` or `5XX` range is
   `clientError` / `serverError`; `default` is not a case — it is what
   `http(.status(code, problem))` already holds. An operation that declares
   no error statuses is `-> T!`, with no enum.
2. **Groups are the tags**, camelCased; an operation with several tags is
   in its first.
3. **Paths are the spec's**, `{id}` and all.
4. **Everything is typed, and nothing is dropped silently [R13].** A
   `oneOf` with a `discriminator` becomes an enum with one case per
   record; a `oneOf` without one becomes an enum whose cases are tried in
   order when the answer is decoded. An inline object becomes a record
   named by its place (`CreateUserBody`). A schema that says nothing about
   its shape (`{}`, `additionalProperties` alone) is `Json`, or `[String:
   T]` when it says the values' type. What the reader still cannot express
   (an XML body) stops `sync` with the operation named, unless `--skip`
   names it; there is no `Any` fallback.
5. **Order is the spec's** (the reader keeps the file's order; B11),
   and the output is formatted by `wf fmt`, so a spec change is the smallest
   diff.
6. **Customise beside, never inside.** A `type` the project declares with a
   generated type's name replaces it; an endpoint the spec lacks goes in a
   group of the project's own (`api Backend.extras { … }`);
   `invalidates` and `pages` for a generated endpoint go in `service.wf`:

   ```wf
   api Backend(base: …) {
       invalidations {
           users.createUser invalidates [users.listUsers]
           users.deleteUser(id) invalidates [users.listUsers, users.getUser(id)]
       }
       pagination {
           users.listUsers pages .cursor(param: after, next: p => p.nextCursor, items: p => p.items)
       }
   }
   ```

   (OpenAPI has no standard for either; the `x-invalidates` and
   `x-pagination` extensions in a spec are read too, for teams that own the
   spec.)
7. **Regeneration overwrites `generated/` whole**, removing files for tags
   that are gone. A generated file that lacks the marker is refused rather
   than overwritten — it was not written by `sync`.
8. **`wf api check` in CI**, and `wf check` warns (`N07`) when a recorded
   spec has changed since its files were written.

#### How it relates to `from`

One reader serves both, so a spec means the same thing either way: `sync`
prints the declarations `from` builds. `from` stays for a prototype; `wf
api eject` turns it into files in one step. The reader is fixed first
(B9–B11): it runs in the build, the language server, the WebAssembly
compiler and the template engine; keeps the file's order; reads `in`,
media types, response `$ref`s, `default` / `4XX`, inline objects (as
records named by their place, `CreateUserBody`), tags, `operationId`s,
security schemes, `servers` and examples; and reads YAML as well as JSON.

---

### 7. What changes in the runtime

| Module | Change |
|---|---|
| `http.js` | Results instead of throws: `send` resolves `["ok", value]` / `["err", error]`. A middleware chain replaces the hooks. Retries only for idempotent requests, with `Idempotency-Key`. The cache becomes an RFC 9111 private cache (reads `Cache-Control`, `Vary`, `Expires`) and a store with subscribers, so `invalidate` refreshes what is shown. In-flight joining for every `GET`. Decoding against the declared type in every build, producing `.decode(at)`. Default timeout 30 s. |
| `net.js` | `resource` exposes one value with three cases (a signal) plus computed members; arguments are one thunk, all followed; `placeholder`, `refresh`, paged and streamed reads. `request` becomes `http`. `optimistic`/`attempt` removed for `update`. `wfFetch`, `fetchOptions` removed. |
| `api.js` | Services and groups; parameters placed by their declared location; `{name}` templates; each write carries its compiled invalidation list and calls `invalidate` on `.ok`; `endpoint.update`. |
| `offline.js` | Queues only idempotent writes; reports refusals; applies invalidations on replay. |
| new `fake.js` (tests only) | Routes a group's calls to a `fake`; never in a production build. |

The checker gains result types (`T ! E`, `T!`, `! E`), error enums with
`on <status>`, declared action results, `Json`, `Problem`, reads as a
three-case value, `HttpError`,
paged and `streamed` reads, literal and nested case patterns, `try` as an
expression, `middleware` and `fake` declarations, groups, parameter
locations, `invalidates` and `idempotent`. The decoder for each declared
type is generated at build time (today's `shape_of`, made complete and
always on).

---

### 8. Migration and diagnostics

#### One cut, in 7.0 [R12]

Every form below is an error in 7.0 (`N08`), and every one carries its
fix (`wf fix`, the editor's quick fix); `wf migrate` 6 → 7 applies them all
at once. There is no warning period and no setting that keeps the old
behaviour: a release that accepted both would be a language with a loose
mode, which the strict-typing plan (Part I, 3.1)
rules out for the whole language. `lints` and `// wf-allow` cannot lower or
accept any `N` code that is an error. 6.3 may ship `wf migrate --to 7
--check`, which only reports what the step will rewrite.

| Today | 7.0 |
|---|---|
| `resource r: T = fetch(url, …)` | `resource r: T = http.get(url, …)`; with no annotation, the fix asks for one (`N11`) |
| `get user(id) -> User` with `errors { 404 -> Problem }` | `-> User ! UserError` and a generated `enum UserError { notFound(problem: Problem) on 404  http(error: HttpError) }`; with no `errors`, `-> User!` |
| `body: Map`, `-> Map`, `-> Any` | a named record the fix drafts from how the value is used, else `Json` |
| an action that `return`s a value | the same action with its result declared, the type the checker inferred |
| `await fetch(url, …)` in an action | `try await http.get/post(url, …)` |
| `match r { loading … error(e) … ready(v) … }` | `match r { .loading … .failed(e) … .loaded(v) … }` |
| `r.state == "ready"`, `r.data`, `r.refreshing` | `r.isLoaded`, `r.value`, `r.isRefreshing` |
| `at "users/:id"` | `at "users/{id}"` |
| an endpoint with no `at` | `at "<name>"`, with a note to check it (B7) |
| `headers { }`, `on request/response/error` | `middleware [header(…), …]` and a written middleware |
| `return "retry"` | `refreshOn401(…)` or a middleware calling `next` again |
| `paginate: .page` on a call | `pages .number(param: page)` on the endpoint |
| `on: .focus` | `refresh: .onFocus` |
| `optimistic(holder, f)` | `let undo = endpoint.update(f)` and an explicit `undo()` |
| `try { await … } catch e { … }` around a call | `match await … { .ok … .err(.http(e)) … }`, a case per declared status |

What changes with no rewrite, which `migrate` and the release notes state:
`POST`/`PATCH` are no longer retried or replayed offline unless
`idempotent`; nothing is cached unless the server or the contract says so;
every response is checked against its type, so a server that does not
match its contract now fails where it used to pass.

#### New diagnostics, family `N` (network)

| Code | Severity | What |
|---|---|---|
| `N01` | error | A path template naming no parameter, or a parameter the call cannot place |
| `N02` | error | A `resource` over a write |
| `N03` | error | `try` in an action that declares no error, or on a call whose error enum has cases the action's does not |
| `N04` | warning | A result that nothing reads (an `await` whose outcome is dropped) |
| `N05` | error | `invalidates` naming an endpoint or argument the contract does not have |
| `N06` | error | A `fake` that does not match its group's interface |
| `N07` | warning | A spec changed since `wf api sync` |
| `N08` | error | A form 7.0 removed (each with its fix) |
| `N09` | warning | `retry` or offline replay asked of a non-idempotent write |
| `N10` | warning | An address written in a page or component instead of a contract |
| `N11` | error | `Any` or `Map` in a contract, an `http` call, a fake or a read's annotation — in 7.0 both are gone from the whole language (strict-typing `T23`, `T24`), and `N11` is the message they give at the boundary |
| `N12` | error | An error enum without exactly one `http(HttpError)` case, a case with no status or two payloads, or a status twice |
| `N13` | — | Folded into the strict-typing plan's `T26` (an action that returns a value without declaring it), an error for every action, old or new |

`T11` and `T21` learn reads and results; `T21` (no handling of the
failure case) becomes an error for both.

---

### 9. Stages

Each stage builds, passes the full suite, and ships the docs for what it
adds. Rust tests sit beside the code and in `tests/*.rs`; browser suites
are `tests/browser/*.mjs` in headless Chrome against a small local test
server that records every request.

**Stage 1 — Stop losing and duplicating data (no new syntax).**
B20 and B21 first (retry and replay only idempotent requests); then B1,
B2, B12, B14, B15, B16, B18 (calls that never send or send the wrong
thing); B3, B4, B13 (the checker); B5, B6 (joining and invalidation); B8,
B19 (dead code).
*Tests*: a browser suite `fetch.mjs` — the first to exercise `api` —
asserting a timed-out `POST` is sent once, an offline `POST` is not
replayed, typing re-asks, two readers make one request, `invalidate()`
refreshes the screen; codegen and checker unit tests for the rest.

**Stage 2 — Results (`T!`), `HttpError`, `try`, patterns.**
`T ! E`, `T!`, error enums with `on <status>`, `HttpError`, `Problem`,
`Json`; declared action results; literal and nested patterns; `try`
expressions and their conversion rule; `.result` on actions; `http` beside
the old `fetch` refused (`N08`, with its fix); `N03`, `N11`, `N12`; `T26`
from the strict-typing plan.
*Tests*: checker tests per pattern and per `try` rule; parser and printer
round trips; a browser test of a 422 handled three ways (5.6).

**Stage 3 — Reads as one value.**
The three cases, computed members, followed arguments, `refresh:`,
`placeholder:`, the old arms and members refused with fixes.
*Tests*: browser tests for each case transition (load, refresh, stale,
first-load failure, argument change).

**Stage 4 — The transport.**
Middleware, the RFC 9111 cache, decoding in every build, `Idempotency-Key`,
default timeout.
*Tests*: unit tests of the cache against the RFC's cases (max-age, no-store,
Vary, revalidation); browser tests of middleware order and of a 401
refreshed once for three concurrent calls.

**Stage 5 — Contracts and writes.**
Groups, `{name}` templates, parameter locations, `invalidates`,
`idempotent`, `endpoint.update`, `N01`–`N05`, `N09`–`N11`.
*Tests*: checker tests for each rule; browser: create → list refreshes,
rename → detail and list refresh, optimistic undo on a 500.

**Stage 6 — Pages, streams, offline.**
`pages`, paged reads, `streamed` endpoints, the offline changes.
*Tests*: browser suites for each paging style against the test server; the
existing `offline.mjs` extended.

**Stage 7 — Fakes.**
`fake`, `expect called`, fakes in `wf verify`, reads' values in `wf
render` data, `N06`.
*Tests*: `wf test` over a fixture project; template tests.

**Stage 8 — The reader and `wf api`.**
The reader fixed and extended (B9–B11, YAML, tags, examples), and the five
commands.
*Tests*: unit tests for the reader; golden folders for three specs (a small
one, Petstore, a large real one) in `tests/fixtures/openapi/`, each built;
`from` and `sync` produce the same `wf types --json`; `check` fails on a
stale folder; a hand-edited file without the marker is refused.

**Stage 9 — Migration and docs.**
`wf migrate` 6 → 7 and every `N08` fix; the data chapter rewritten around
the layers; recipes (a form with server errors, a list that refreshes after
a create, testing with a fake, a client from OpenAPI); `AGENTS.md`; the
React/TanStack comparison; the Arabic pages.

Stage 1 can start at once. Stages 2 → 3 → 5 are a chain; 4, 6, 7 and 8
can each run beside it once 2 is in.

---

### 10. Open questions, each with a recommendation

1. **How is a result written?** *`T ! E`, beside `T?`*, with `T!` for a
   call that declares no errors and `! E` for one with no value. Every part
   is a named type, so every result can be written down (R13) without
   angle brackets, which would be the only ones in the language.
2. **`try` or Rust's postfix `?`?** *`try`.* `?` already means "may be
   null" in types and starts `?.` and `??` in expressions.
3. **Required types: an error at once?** *Yes, for everything, old code
   included.* A warning for code written before 7.0 would be a loose mode
   for one major; the fixes and `wf migrate` carry old code across in one
   step instead (R12, and the strict-typing plan's 3.1).
4. **Decode every response in production?** *Yes, always, with no way to
   turn it off.* A value typed `Person` that is not one is the bug strong
   typing exists to prevent; the walk is linear in the answer and cheap
   beside the network.
5. **Should the rest of the language be this strict?** *Yes, with no
   option.* Part I does it: `Any` and `Map` leave
   the language, signatures are declared, there is no implicit conversion,
   and there is no setting that turns any of it off. The two plans share
   `Json`, `-> T`, `T ! E` and `T.decode`; whichever starts first builds
   them once, and both ship in 7.0.
6. **Cache only what the server allows?** *Yes.* It is the standard, and a
   client that guesses gets private data wrong. `cache: .maxAge(…)` on an
   endpoint is the visible escape for servers that send no headers.
7. **Should a page be allowed a `resource` at all, or only a store?**
   *Allowed.* Small pages need no store; `N10` keeps URLs out of views,
   which is the part of the rule that matters.
8. **Keep `api … from "openapi.json"`?** *Yes, as the quick path*, sharing
   the reader with `wf api`.
9. **YAML: a dependency or our own reader?** *Our own reader for the
   subset OpenAPI documents use*, in keeping with the project owning its
   formats; a spec using anchors or tags we do not read gets a clear error
   and the one-line conversion.
10. **Where does `invalidates` go for a generated contract?** *In
   `service.wf`'s `invalidations { }`*, plus the `x-invalidates`
   extension for teams that own their spec.

---

### Appendix: where things are

| What | Where |
|---|---|
| Grammar for `resource`, `api`, endpoints | `src/syntax/parser/v2.rs` (`parse_resource`, `parse_api`, `parse_endpoint`) |
| AST | `src/syntax/parser/ast.rs` (`ResourceDecl`, `ApiDecl`, `Endpoint`, legacy `FetchDecl`) |
| OpenAPI reader | `src/project/openapi.rs` (`expand`, called from `src/project/sources.rs:186` only) |
| Checker | `src/check/sema/types/{method,expr,stmt,members,names,call}.rs`, `src/check/sema/mod.rs` |
| Codegen | `src/web/codegen/js/net.rs` (resources, connections), `store.rs` (`emit_api`), `expr.rs` (`fetch` and endpoint calls), `flow.rs` (`sameCall`, `attempt`) |
| Static paint, templates, tests | `src/web/codegen/ssg.rs:821`, `src/template/mod.rs:1479`, `wf test` steps in `ast.rs:349` and `testing.rs` |
| Existing tests | `tests/language_4_0.rs` (services, OpenAPI), `tests/store_resource.rs`, `tests/offline.rs`; browser `store_resource.mjs`, `offline.mjs`, `leaks.mjs`, `busy.mjs` |
| Runtime | `src/web/runtime/modules/http.js` (engine), `net.js` (`resource`, `request`, `optimistic`), `api.js` (services, connections), `match.js`, `offline.js` |
| Tutorial | `md-docs/learn/15-data.md`; recipes `page-through-results`, `load-more`, `sign-in-with-a-session-cookie`, `api-on-the-same-origin`, `debounce-a-search` |
