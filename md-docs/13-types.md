# 13. Types

<!--
route: guide/types
group: basics
blurb: A gradual, structural type checker that changes no output and names every mistake with a hint.
description: The type language, built-in types, records and enums, refinements, inference and narrowing, typed network data, and every check.
-->

WebFluent has a gradual, structural type checker. It runs in `wf build` and
in the editor, before anything is emitted, and its findings are errors with
a hint. A program that declares no types checks as it always did — what the
checker cannot resolve is `Any`, and `Any` agrees with everything — and
every annotation you add narrows what it can say.

## The type language

| Type | Written | Values |
|---|---|---|
| String | `String` | `"text"`, `"with {interpolation}"` |
| Number | `Number` (`Int` and `Float` are accepted as other spellings of it) | `42`, `3.14` |
| Bool | `Bool` | `true`, `false` |
| List | `[String]`, `[Todo]` | `[a, b]` |
| Map | `Map` | `{ key: value }` — an object with unknown keys |
| Optional | `String?`, `Todo?` | a value or `null` |
| Record | the `type`'s name | `Todo(id: "a", title: "b")` |
| Enum | the `enum`'s name | `.case` |
| Any | `Any` | anything; what is not known |

A type is written after a colon on a `state`, a prop, an action parameter,
a `type` field, a route parameter, a `let`, or a `resource`.

```wf
page Typed(path: "/") {
    state name: String = ""
    state count: Number = 0
    state tags: [String] = []
    state picked: String? = null
    state profile: Map = {}
    action greet(who: String, times: Number) { log("hi {who} × {times}") }
    derived pick = picked ?? "none"
    Text("{name} {count} {tags.length} {pick}")
    Button("Go") { on click { greet(name, count) } }
}
```

## The types the language brings with it

Eleven more come with the language. Each is a **plain JSON value at run
time** — no wrapper object — so it crosses a `fetch`, a `persist`, the
static paint and the template engine unchanged; `.native()` hands a real
platform object to a library that wants one.

| Type | Carried by | Written as a literal |
|---|---|---|
| `Date` | `"2026-03-14"` | `@2026-03-14` |
| `Time` | `"09:30"` | `@09:30` |
| `DateTime` | ISO 8601 | `@2026-03-14T09:30Z` |
| `Duration` | milliseconds | `3.days`, `90.minutes`, `250.ms` |
| `Money` | `{ amount: 1299, currency: "EUR" }` | `€12.99`, `$9.99`, `£4.50` |
| `Url` | a string | `"https://example.com"` |
| `Email` | a string | `"ada@example.com"` |
| `Color` | a string | `#0F766E` |
| `Uuid` | a string | `uuid()` makes a fresh one |
| `File` | the browser's `File` | what a `FileUpload` yields |
| `Secret` | a string | — |

Money is in **minor units** — cents, pence — so the arithmetic is a whole
number's and nothing drifts. A duration is milliseconds, so it adds to a
moment. `uuid()` is where a `Uuid` comes from: the platform's own
generator where there is one, a version-4 layout from the best randomness
available where there is not.

```wf
type Booking {
    id: Uuid
    guest: Email
    from: Date
    nights: Number(1..=30)
    total: Money
}

page Stay(path: "/", title: "Stay", description: "One booking.") {
    state from: Date = @2026-03-14
    state nights: Number = 5
    state total: Money = €249.00

    derived checkout = from.plus(days: nights)
    derived weekday = checkout.weekday()
    derived late = now.date().isAfter(checkout)

    Heading("Your stay").h1
    Text("Out on {checkout}, a weekday of {weekday}")
    Text("Due {format(total)}")
    if late { Alert("Checkout has passed").warning }
}
```

Everything the language **computes** is a call; everything the carrier
**already has** is a field. So `total.amount` and `total.currency` are
fields, and `from.year()` and `checkout.weekday()` are calls.

| | |
|---|---|
| `Date` `Time` `DateTime` | `.year() .month() .day() .weekday() .hour() .minute() .second()`, `.plus(days: 3)` and `.minus(…)` (which take `years months weeks days hours minutes seconds ms`), `.isBefore(d) .isAfter(d) .isSame(d)`, `.until(d)` (a `Duration`), `.startOfDay() .startOfWeek() .startOfMonth() .endOfDay()`, `.native()`; `DateTime` also `.date() .time() .inZone("Europe/Berlin")` |
| `Duration` | `.days() .hours() .minutes() .seconds() .ms()`, `.plus(d)` |
| `Money` | `.plus(m) .minus(m) .times(n) .convert(rate, "USD")`; `format(m)` needs no style and no currency code |
| `Url` | `.host() .path() .query()`, `.with(query: { page: 2 })` |
| `Email` | `.domain()` |
| `Color` | `.mix(other, 0.2) .lighten(0.1) .darken(0.1) .alpha(0.5) .contrast(other)` |
| `File` | `.preview()` — an object URL, revoked when the scope that made it leaves |

Adding a month lands on a day that exists: `@2026-01-31.plus(months: 1)` is
the 28th of February, not the 3rd of March. All of it runs at build time
too, so the static paint shows the answer rather than a blank that fills in
when the page hydrates.

### A literal is held to its type

A string written where one of these is wanted is read as one, and checked
where it is written:

```wf
state site: Url = "https://example.com"     // fine
state bad: Url = "example"                  // T01: A URL has a scheme
```

### `now`

`now` is a `DateTime` that keeps itself current — every minute by default,
or as often as a page asks:

```wf
page Feed(path: "/", title: "Feed", description: "What just happened.") {
    state posted: DateTime = @2026-03-14T09:30:00Z
    derived age = posted.until(now)
    Text("{ago(posted)} — {age.minutes()} minutes")
    Text("ticking: {now(every: 1.seconds)}")
}
```

### A condition on a type

Any type may say what its values must be, and every value the compiler can
read is held to it:

```wf
state nights: Number(1..=30) = 12
state pass: String(minLength: 8) = "correcthorse"
state start: Date(after: @2026-01-01) = @2026-03-14
```

`min`, `max`, `minLength`, `maxLength`, `pattern`, `after`, `before`, and a
range (`1..=30` inclusive, `0..10` exclusive of the end) for the two ends.
A value that arrives at run time is validation's to refuse, from the same
condition.

### `Secret` is not a string

A `Secret` is a string the compiler will not let escape. Each of these is a
`T12`:

```wf
state token: Secret = ""
Text(token)                     // shown
Text("Bearer {token}")          // in text — which is how it ends up in a URL
persist token: Secret = ""      // kept in the browser
log(token)                      // in the console, and often in a log service
```

Hold it in `state`, which the visit ends; what must outlive the visit
belongs on the server.

### Your own name wins

A `type` the program declares takes the name back. A project with its own
`type Color { name: String }` means that one everywhere.

## Records: `type`

```wf
enum Tone { neutral, info, danger }

type Todo {
    id: String
    title: String
    done: Bool = false
    tone: Tone = .neutral
    note: String?
}

type Admin = Todo { owner: String }

page Records(path: "/") {
    state todos: [Todo] = [
        Todo(id: "1", title: "Write the docs"),
        Todo(id: "2", title: "Ship", done: true, tone: .info, note: "Friday"),
    ]
    state special: Admin = Admin(id: "3", title: "Review", owner: "sam")
    for t in todos by t.id {
        Row(gap: .sm) {
            Text(t.title)
            if let n = t.note { Badge(n).info }
        }
    }
    Text("{special.owner} owns {special.title}")
}
```

- Fields may be separated by newlines or commas.
- A field with a default may be left out of a construction; a `T?` field
  defaults to `null`.
- `type Admin = Todo { owner: String }` extends a record: an `Admin` has
  every field of `Todo` and then its own.
- Records are structural: `{ id: "a", title: "b" }` given where a `Todo` is
  wanted is a `Todo`, and its fields are checked against the record's.
- A `Map` fits any record and a record fits a `Map`: JSON from `fetch` fits a
  `type` it was never declared as.

## Enums

```wf
enum Size { sm, md, lg }
enum Status { idle, running(job: String), failed(reason: String), done(count: Number) }

page Enums(path: "/") {
    state size: Size = .md
    state s: Status = .idle
    Button("Bigger") { on click { size = .lg } }
    Button("Start") { on click { s = .running("build-42") } }
    Text(match size { .sm { "small" } .md { "medium" } else { "large" } })
    match s {
        .running(job) { Text("Running {job}") }
        .failed(reason) { Alert(reason).danger }
        .done(n) { Text("{n} done") }
        else { Text("Idle") }
    }
}
```

- A case is written `.name`; on its own it is a case of an enum not yet
  known, and where a `Size` is wanted it becomes a `Size` — `.huge` is `T02`.
- A case may carry a payload: `.failed("x")` is checked like a call, and a
  `match` arm binds it: `.failed(r)` gives `r: String`.
- `size == .lg` compares cases; an enum is written to a component's root
  as `data-<prop>` ([chapter 11](11-components.md#enum-props)).

## Map literals are shapes

A map literal has the fields it was written with: `{ name: "", age: 0 }` is
`{ name: String, age: Number }`, and reading `form.nam` is `T05` with the
fields it does have. A shape fits a `Map`, a record with the same fields,
and any shape it agrees with. `{}` and a literal with a spread are plain
`Map`s.

```wf
page Shapes(path: "/") {
    state form = { name: "", age: 0, tags: ["new"] }
    Input(bind: form.name, label: "Name")
    Text("{form.name} is {form.age} with {form.tags.length} tags")
}
```

## Where types come from

**Declared** — annotations, a `type`'s fields, an `enum`'s cases, a page's
route parameters, an event's parameters, a component's props.

**Inferred** — a literal; a list of literals (`[1, 2]` is `[Number]`, `[]`
is `[Any]`); a record construction; a `derived` from its expression; a
`state` from its initial value (one that starts as `null` is `Any?`); a
`for` binding from the list's element type; a `match` arm's binding; an
`if let` binding; a lambda's parameters from what the method expects
(`items.filter(t => …)` gives `t` the item type).

**From the registry** — the type of every built-in prop; what a `bind:`
needs (`Checkbox` and `Switch` a `Bool`, `Slider` a `Number`, `Input` a
`String` or a `Number` by its type).

**From the method tables** — on a `String`: `trim`, `toLowerCase`,
`toUpperCase`, `replace`, `slice`, `padStart`, `repeat`, `capitalize`,
`truncate(n)` → `String`; `length`, `indexOf` → `Number`; `includes`,
`startsWith`, `endsWith` → `Bool`; `split` → `[String]`. On a list: `map`
→ `[U]` from the lambda; `filter`, `slice`, `concat`, `sort`, `sortBy`,
`unique`, `take`, `flatMap` → `[T]`; `find`, `first`, `last` → `T?`;
`some`, `every`, `includes` → `Bool`; `length`, `sum`, `indexOf`,
`findIndex` → `Number`; `join` → `String`; `groupBy` → `Map`; `reduce` →
the initial value's type. On a `Number`: `toFixed`, `toString` → `String`.
Anything else is `Any`. The full lists are in
[Built-ins](40-built-ins.md).

**From stores** — `Todos.items` has the type of the store's state,
`Todos.add` the signature of its action.

Browser globals (`window`, `document`, `JSON`, `Math`, `Date`, …) are `Any`.

## Inference is bidirectional

An expression is checked against the type its context wants when there is
one, and its type is read off it otherwise:

```wf
type Row { id: Number, label: String, score: Number? }

page Infer(path: "/") {
    state rows: [Row] = [Row(id: 1, label: "a"), Row(id: 2, label: "b", score: 9)]
    derived labels = rows.map(r => r.label)                 // [String]
    derived best = rows.find(r => r.score != null)          // Row?
    derived total = rows.reduce((n, r) => n + (r.score ?? 0), 0)   // Number
    derived kind = if total > 5 { "high" } else { "low" }   // String
    derived firstScore = best?.score ?? 0                   // Number
    derived joined = labels.join(", ")                      // String
    Text("{joined} {kind} {firstScore}")
}
```

- `a ?? b` is `a` without its `null`, joined with `b`.
- `a?.b` is `b`'s type made optional, and the rest of the chain with it.
- `if`/`match` as values have the type both arms fit: the same type, `T?`
  when one arm is `null`, else `Any`.
- `-`, `*`, `/`, `%` are `Number`; `+` is a `String` when either side is
  one; comparisons and `&&`, `||`, `!` are `Bool`.
- `await e` on a `fetch` is `Any` unless annotated: `let r: [Row] = await fetch(…)`.
- `let { a, b } = m` and `let [x, y] = l` bind the fields and items.

## Narrowing optionals

A `T?` must be unwrapped before a field or method is read off it (`T04`).
Each of these proves a name non-null within its branch:

```wf
type Todo { id: String, title: String, note: String? }

page Narrow(path: "/") {
    state sel: Todo? = null
    state t: Todo = Todo(id: "1", title: "x")
    if let n = t.note { Text("{n.length} chars") }
    if sel != null { Text(sel.title) }
    if sel { Text(sel.title) }
    derived label = if sel != null { sel.title } else { "none" }
    Text(t.note ?? "no note")
    Text(sel?.title ?? "nothing selected")
    Button("Pick") { on click { sel = t } }
}
```

Narrowing is flow-sensitive within a block: it holds in the branch the
condition guards, not after it.

## Typing what comes from the network

`fetch` returns `Any`. Annotate the resource or the local to type its value:

```wf
type Deploy { id: String, status: String, startedAt: String }

page Deploys(path: "/") {
    resource deploys: [Deploy] = fetch("/api/deploys")
    match deploys {
        loading { Spinner }
        error(e) { Alert("Failed: {e.message}").danger }
        ready(rows) {
            for d in rows by d.id { Text("{d.id} {d.status} {ago(d.startedAt)}") }
        }
    }
}
```

The checker trusts the annotation: it cannot see what a URL will return.
When the server publishes an OpenAPI specification, let the compiler read it
instead — `api Backend from "openapi.json"` types every endpoint, parameter,
response and error from the file, and turns each named schema into a `type`
([Data and APIs](17-data.md#from-a-specification)). An `api` written by hand
types its calls the same way: `get users() -> [User]`.

## The checks

| Code | When |
|---|---|
| `T01` | A value of the wrong type given to a prop, a state, a field, a parameter, an assignment, an argument, or a `bind:` |
| `T02` | A case the enum does not have |
| `T04` | A field or method read off a value that may be `null` |
| `T05` | A field or method a record or shape does not have |
| `T06` | A member or action a store does not have |
| `T07` | A list, a record or an action used as a condition, which is always true |
| `T08` | A `for` over something that is not a list |
| `T09` | An `emit` whose arguments do not match the event's |
| `T10` | A call with the wrong number of arguments |
| `T11` | A `match` on something that is neither a resource nor an enum, or an arm the value cannot take |
| `T12` | A `Secret` where it would escape — shown, spliced into text, logged, or kept with `persist` |

Each comes with a hint: a conversion (`Number(value)`, `"{value}"`), the
cases an enum takes, the fields a record has, the unwrapping forms above.

What the checker does not do: it does not check a lambda's body against a
return type, does not change a name's type across assignments (`state x =
1` stays a `Number`; assigning a string to it is `T01`), and does not know
what a plain `fetch` returns (annotate it, or use an `api`).

## In the editor

Hover shows a name's inferred type (`state draft: String`, `derived
remaining: Number`, `Todos.add: action(String)`); completion after `.`
offers a record's fields, an enum's cases where one is wanted, and the
methods of a `String`, a `Number` or a list. A type error is a diagnostic
at the argument it belongs to. [Chapter 37](37-cli.md#editors) sets it up.

## Next

[Expressions](14-expressions.md).
