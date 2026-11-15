# Strict typing: the whole language

Written 2026-11-15, against 6.2.0 (commit 39767cde). Nothing here is built
yet: this is the plan to approve first.

The data-fetching plan (`plans/data-fetching/README.md` on
`claude/fetch-redesign-plan-e98caf`, sections 4 and 5.2) makes the data
layer strongly and statically typed and leaves the rest of the language to
this plan (its open question 5). This plan applies the same rules
everywhere else, with the same spellings, so the two read as one design.

## In short

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

## 1. Where values are untyped today

Paths are under `src/check/sema/types/` unless given in full. Every
example compiles today with no finding.

### 1a. The rules that let anything through

| # | Where | What | Example that passes |
|---|---|---|---|
| H1 | `ty.rs:196` | `Any` is assignable to and from every type | `state n: Number = JSON.parse(s)` |
| H2 | `ty.rs:203–205` | `Map`, a map literal's shape and every record agree with each other, without looking at fields | `state u: User = { nmae: "Ada" }` — a `User` with no `name` |
| H3 | `ty.rs:237`, `ty.rs:262–263` | Two types with nothing in common join to `Any` instead of being refused | `derived x = if ok { 1 } else { "none" }` is `Any`, and `x * 2` passes |
| H4 | `ty.rs:161`, `src/syntax/parser/v2.rs:941–942` | `Any` and `Map` can be written as types | `component Card2(data: Any)` |
| H5 | `src/syntax/parser/v2.rs:2235–2239` | A parameter written without a type is `Any` | `action add(item) { items = items.concat([item]) }` |
| H6 | `ty.rs:313` | `Any` counts as a number for arithmetic | `derived total = rows.reduce((a, r) => a + r.price, 0)` with `rows: [Map]` |
| H7 | `ty.rs:199–223` | Implicit conversions: a text scalar fits `String`, `Duration` and `Number` fit each other | `Text(due)` with `due: Date`; `after(3.seconds + 500) { … }` |

### 1b. Declarations whose type is never known

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

### 1c. Expressions that give up

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

### 1d. What comes from outside

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

### 1e. How much the site and the examples rely on it

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

## 2. The rules

These are the data plan's R13 and section 5.2, applied to the whole
language. Each rule says what it rules out today.

### 2.1 One type per value, and every type can be written

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

### 2.2 The spellings

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

### 2.3 Declared signatures, inferred bodies

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

### 2.4 No implicit conversions

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

### 2.5 The browser, typed

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

### 2.6 Project scripts

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

### 2.7 Data that arrives

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

### 2.8 Built-ins

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

## 3. Migration and diagnostics

### 3.1 No switch, one cut

The data plan's R12 says deprecate before removing. That holds for
*spellings* the language drops, but a typing rule that can be turned off
is not a rule: a program would be typed or not depending on a config file,
a library would have to work both ways, and "reliable" would mean "reliable
if you asked". So strictness ships whole, once, and is never optional.

| Release | What happens |
|---|---|
| **6.x** | Nothing changes. 6.3 may ship `wf migrate --to 7 --check`, a report of what the 6 → 7 step will rewrite and what it cannot, so a team can see the size of the move before it upgrades. It changes no build. |
| **7.0** | Every rule of section 2 is in force, as errors. `Any` and `Map` are no longer types ("`Map` is gone; write a record, `[String: T]` or `Json`"). Every error that has a known fix carries it (`wf fix`, the editor's quick fix), and `wf migrate` applies them all at once (3.2). `lints` and `// wf-allow` cannot lower or accept `T23`–`T35`, nor the errors they extend (`T01`, `T05`, `T13`, `T20`). There is no `"types"` key; writing one is `E112`. |

The cost lands once, on upgrade, and `wf migrate` carries most of it: in
the site and the examples, everything in 1e but a handful of `Map`s that
need a record drafted is rewritten mechanically.

**One rule across both plans.** The data plan lets old code keep `Any`
and `Map` in a contract with a warning through 7.x (its `N11`, `N13`).
Under this plan those are errors in 7.0 like everywhere else; the data
plan's table in its section 8 should be changed to match, so there is no
corner of the language where `Any` survives (open question 6).

### 3.2 What `wf migrate` writes (6 → 7)

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

### 3.3 New diagnostics

`T23`–`T35`, after `T22`. All are errors from 7.0, none can be lowered by
`lints` or accepted by `// wf-allow`, and "fix" says what `wf fix` and the
editor write.

| Code | Severity | What |
|---|---|---|
| `T23` | error | `Any` written as a type. Fix: the inferred type, else `Json` |
| `T24` | error | `Map` written as a type. Fix: a drafted record, `[String: T]`, or `Json` |
| `T25` | error | A `state`, `persist` or `data` whose type its value does not say (`[]`, `{}`, `null`, a call returning `Json`). Fix: the type assigned to it later |
| `T26` | error | An action that returns a value without declaring `-> T`. Fix: the inferred result. (The data plan's `N13`; see open question 6) |
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

## 4. Stages

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

## 5. Open questions, each with a recommendation

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
   plan's `N13` and the "warning for code written before 7.0" in its `N11`
   should both go: with no switch in the language, there is no older code
   that is allowed to be less typed.
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

## Appendix: where things are

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
| The data-fetching plan | `plans/data-fetching/README.md` on `claude/fetch-redesign-plan-e98caf` |
