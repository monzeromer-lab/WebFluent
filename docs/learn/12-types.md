# Types

> The type language, built-in types, records and enums, refinements, inference and narrowing, typed network data, and the checks the compiler runs.

The page: https://webfluent.monzeromer.dev/learn/12-types

This chapter covers WebFluent's types and the checker that holds a program to
them. The checker runs in `wf build`, `wf check` and the
editor, before any output is written. It is *gradual*: a program that
declares no types still checks, because whatever the checker cannot work
out has the type `Any`, and `Any` agrees with everything. Every annotation
you add lets the checker say more. It is also *structural*: a value fits a
type when it has the right fields, whatever it was declared as. The checker
changes nothing in the output; it only reports.

**In this chapter**

- The type language, and the types that come with the language
- Records (`type`), enums and the shapes of map literals
- Where the checker gets a type from, and how it infers one
- Narrowing a value that may be `null`
- Typing data from the network
- The checks, and what the checker does not check

## A first type error

This page declares an `Invoice` record and builds one with the wrong kind
of value for `total`:

```wf expect=T01
type Invoice { id: String, total: Number, paid: Bool = false }

page Invoices(path: "/", title: "Invoices", description: "What is owed.") {
    state invoice: Invoice = Invoice(id: "INV-001", total: "120")
    Heading("Invoice {invoice.id}").h1
    Text("Owed: {invoice.total}")
}
```

The build stops, pointing at the field:

```text
error[T01]: `total` of `Invoice` is `Number`, but `"120"` is `String`
 --> src/App.wf:4:53
```

Writing `total: 120` fixes it. Had the invoice been an untyped map,
`{ id: "INV-001", total: "120" }`, the page would build, and
`invoice.total` would be a string at run time.

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

You write a type after a colon on a `state`, a prop, an action parameter, a
`type` field, a route parameter, a `let` or a `resource`. This page uses
several:

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

Eleven more types come with the language. Each is a **plain JSON value at
run time**, with no wrapper object, so it crosses a `fetch`, a `persist`,
the static paint (the HTML the build pre-renders for a page) and the
template engine unchanged. `.native()` hands a real platform object to a
library that wants one.

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

Money is held in **minor units** — cents, pence — so its arithmetic is
whole-number arithmetic and nothing drifts. A duration is milliseconds, so
it adds to a moment. `uuid()` is where a `Uuid` comes from: it uses the
platform's own generator where there is one, and otherwise builds a
version-4 UUID from the best randomness available.

This page books a stay with dates, money and a refined number:

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

| Type | Methods |
|---|---|
| `Date` `Time` `DateTime` | `.year() .month() .day() .weekday() .hour() .minute() .second()`, `.plus(days: 3)` and `.minus(…)` (which take `years months weeks days hours minutes seconds ms`), `.isBefore(d) .isAfter(d) .isSame(d)`, `.until(d)` (a `Duration`), `.startOfDay() .startOfWeek() .startOfMonth() .endOfDay()`, `.native()`; `DateTime` also `.date() .time() .inZone("Europe/Berlin")` |
| `Duration` | `.days() .hours() .minutes() .seconds() .ms()`, `.plus(d)` |
| `Money` | `.plus(m) .minus(m) .times(n) .convert(rate, "USD")`; `format(m)` needs no style and no currency code |
| `Url` | `.host() .path() .query()`, `.with(query: { page: 2 })` |
| `Email` | `.domain()` |
| `Color` | `.mix(other, 0.2) .lighten(0.1) .darken(0.1) .alpha(0.5) .contrast(other)` |
| `File` | `.preview()` — an object URL, revoked when the scope that made it leaves; the fields `.name .size .type` |

Adding a month lands on a day that exists: `@2026-01-31.plus(months: 1)` is
the 28th of February, not the 3rd of March. All of these methods also run
at build time, so the static paint shows the answer instead of a blank that
fills in when the page hydrates.

### A literal is held to its type

A string written where one of these types is wanted is read as that type,
and checked where you wrote it. The second `state` on this page does not
build:

```wf expect=T01
page Links(path: "/", title: "Links", description: "A URL held to its type.") {
    state site: Url = "https://example.com"     // fine
    state bad: Url = "example"                  // T01: a URL has a scheme
    Heading("Links").h1
    Text("{site.host()} {bad}")
}
```

### `now`

`now` is a `DateTime` that keeps itself current. It updates every minute by
default, or as often as a page asks with `now(every: …)`:

```wf
page Feed(path: "/", title: "Feed", description: "What happened lately.") {
    state posted: DateTime = @2026-03-14T09:30:00Z
    derived age = posted.until(now)
    Text("{ago(posted)} — {age.minutes()} minutes")
    Text("ticking: {now(every: 1.seconds)}")
}
```

Read `now` where it is shown or in a `derived` value. A `state` takes its
initial value once, so `state clock = now(every: 1.seconds)` stays at the
moment the page opened (`X06`; see
[State and reactivity](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md)).

### A condition on a type

Any type may say what its values must be. The compiler holds every value it
can read to that condition, as it does to the three states on this page:

```wf
page Stay(path: "/", title: "Stay", description: "Values held to a condition.") {
    state nights: Number(1..=30) = 12
    state pass: String(minLength: 8) = "correcthorse"
    state start: Date(after: @2026-01-01) = @2026-03-14
    Heading("Stay").h1
    Text("{nights} nights from {start}, {pass.length} characters")
}
```

The conditions are `min`, `max`, `minLength`, `maxLength`, `pattern`,
`after` and `before`, and a range for the two ends at once: `1..=30`
includes the end, `0..10` excludes it. A value that arrives at run time —
typed into a field, say — is refused by validation, from the same
condition, so a refined type needs no `validate` rule for it (see
[Forms](https://webfluent.monzeromer.dev/learn/14-forms.md)).

### `Secret` is not a string

A `Secret` is a string the compiler will not let escape. This page lets one
escape four ways, and each is a `T12`:

```wf expect=T12
page Login(path: "/", title: "Login", description: "Four ways a secret escapes.") {
    state token: Secret = ""
    persist saved: Secret = ""                       // kept in the browser
    Heading("Signed in").h1
    Text(token)                                      // shown
    Text("Bearer {token}")                           // in text, which is how it ends up in a URL
    Button("Debug") { on click { log(token) } }      // in the console, and often in a log service
}
```

Joining it with `+` or handing it to a browser global such as `console`, a
storage or a `fetch` address is a `T12` too, and a secret has no string
members to read. Hold it in `state`, which ends with the visit; what must
outlive the visit belongs on the server.

### Your own name wins

A `type` the program declares takes the name back from the language. A
project with its own `type Color { name: String }` means that record
everywhere it writes `Color`.

## Records: `type`

A *record* is a value with named, typed fields, declared with `type`:

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
    for todo in todos by todo.id {
        Row(gap: .sm) {
            Text(todo.title)
            if let n = todo.note { Badge(n).info }
        }
    }
    Text("{special.owner} owns {special.title}")
}
```

- You may separate fields with newlines or commas.
- You build a record by naming its fields: `Todo(id: "1", title: "x")`. A
  field with a default may be left out, and a `T?` field defaults to
  `null`. Leaving out a field with no default is `C01`.
- A record whose fields can all be left out is built with none given,
  `Offers()`, wherever a value goes: a prop's default (`component
  Card(offers: Offers = Offers())`), a field's default, a `const`, a
  `state`, an argument.
- `type Admin = Todo { owner: String }` extends a record: an `Admin` has
  every field of `Todo`, then its own.
- Records are structural: `{ id: "a", title: "b" }` given where a `Todo` is
  wanted is a `Todo`, and its fields are checked against the record's.
- A `Map` fits any record, and a record fits a `Map`. So JSON from `fetch`
  fits a `type` it was never declared as.

## Enums

An *enum* is a type with a fixed set of *cases*. A case may carry a
*payload* — values that travel with it:

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

- You write a case as `.name`. On its own it is a case of an enum not yet
  known; where a `Size` is wanted it becomes a `Size`, and `.huge` there is
  `T02`.
- A case with a payload, `.failed("x")`, is checked like a call. A `match`
  arm binds the payload: `.failed(r)` gives `r` the type `String`.
- At run time a bare case is its name and a case with a payload is a list,
  `["failed", "timed out"]`, so `size == .lg` still compares cases.
- A `match` with no `else` must give every case an arm (`T15`).
- An enum prop is written to a component's root element as `data-<prop>`
  (see [Enum props](https://webfluent.monzeromer.dev/learn/10-components.md#enum-props)).

> [!NOTE]
> **Coming from TypeScript in React or Vue?** A union of strings such as
> `"idle" | "running" | "failed"` becomes an `enum`, and a field that only
> some states have becomes that case's payload: `.failed(reason: String)`. A
> `match` then has to give every case an arm. See
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

## Shapes of map literals

A map literal has the fields it was written with. `{ name: "", age: 0 }`
has the *shape* `{ name: String, age: Number }`, and reading `form.nam` is
`T05`, with the fields it does have. A shape fits a `Map`, a record with the
same fields, and any shape it agrees with. An empty `{}` and a literal with
a spread say nothing about their keys, so they are plain `Map`s.

```wf
page Shapes(path: "/") {
    state form = { name: "", age: 0, tags: ["new"] }
    Input(bind: form.name, label: "Name")
    Text("{form.name} is {form.age} with {form.tags.length} tags")
}
```

## Where types come from

The checker reads a type from five places.

**Declared:** annotations, a `type`'s fields, an `enum`'s cases, a page's
route parameters, an event's parameters, a component's props.

**Inferred:** a literal; a list of literals (`[1, 2]` is `[Number]`, `[]`
is `[Any]`); a record construction; a `derived` value from its expression;
a `state` from its initial value (one that starts as `null` is `Any?`); a
`for` binding from the list's item type; a `match` arm's binding; an
`if let` binding; a lambda's parameters from what the method expects
(`items.filter(item => …)` gives `item` the item type).

**From the registry:** the type of every built-in prop, and what a `bind:`
needs — `Checkbox` and `Switch` a `Bool`, `Slider` a `Number`, `Input` a
`String` or a `Number` by its input type.

**From the method tables:**

- On a `String`: `trim`, `toLowerCase`, `toUpperCase`, `replace`, `slice`,
  `padStart`, `repeat`, `capitalize`, `truncate(n)` → `String`; `length`,
  `indexOf` → `Number`; `includes`, `startsWith`, `endsWith` → `Bool`;
  `split` → `[String]`.
- On a list: `map` → `[U]` from the lambda; `filter`, `slice`, `concat`,
  `sort`, `sortBy`, `unique`, `take`, `flatMap` → `[T]`; `find`, `first`,
  `last` → `T?`; `some`, `every`, `includes` → `Bool`; `length`, `sum`,
  `indexOf`, `findIndex` → `Number`; `join` → `String`; `groupBy` → `Map`;
  `reduce` → the initial value's type.
- On a `Number`: `toFixed`, `toString` → `String`.

Any other method is `Any`. The full lists are in
[Built-ins](https://webfluent.monzeromer.dev/reference/built-ins.md).

**From stores:** `Todos.items` has the type of the store's state, and
`Todos.add` the signature of its action.

Browser globals (`window`, `document`, `JSON`, `Math`, `Date`, …) are
`Any`.

## Bidirectional inference

The checker works in both directions. When the context wants a type, it
checks the expression against that type; otherwise it reads the type off
the expression. The comments show what each `derived` value is inferred to
be:

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

The rules for operators and forms:

- `a ?? b` is `a` without its `null`, joined with `b`.
- `a?.b` is `b`'s type made optional, and the rest of the chain with it.
- `if` and `match` used as values have the type both arms fit: the same
  type, `T?` when one arm is `null`, and `Any` otherwise.
- `-`, `*`, `/` and `%` give a `Number`; `+` gives a `String` when either
  side is one; comparisons and `&&`, `||`, `!` give a `Bool`.
- `await e` on a `fetch` is `Any` unless you annotate it:
  `let r: [Row] = await fetch(…)`.
- `let { a, b } = m` and `let [x, y] = l` bind the fields and the items.

## Narrowing optionals

A value of type `T?` may be `null`, so you must *narrow* it — prove it is
not `null` — before you read a field or method off it. Reading one without
narrowing is `T04`. Each of these forms narrows a name within its branch:

```wf
type Todo { id: String, title: String, note: String? }

page Narrow(path: "/") {
    state sel: Todo? = null
    state todo: Todo = Todo(id: "1", title: "x")
    if let n = todo.note { Text("{n.length} chars") }
    if sel != null { Text(sel.title) }
    if sel { Text(sel.title) }
    derived label = if sel != null { sel.title } else { "none" }
    Text(todo.note ?? "no note")
    Text(sel?.title ?? "nothing selected")
    Button("Pick") { on click { sel = todo } }
}
```

Narrowing follows the flow of a block: it holds in the branch the condition
guards, not after it. A check for `null` also narrows the `else` branch,
and the code after `if x == null { return }`. An item at a fixed index,
such as `todos[0]`, may be missing, so it counts as `T?` unless a condition
like `todos.length > 0` encloses it.

A list narrows too: `xs.filter(x => x != null)` is a list of the items that
are there, so a `[Todo?]` filtered that way is a `[Todo]` whose items need
no `?.`.

> [!NOTE]
> **Coming from React?** `{user && <Profile user={user} />}` becomes
> `if let u = user { Profile(u) }`, and inside the branch `u` is a `User`, not
> a `User?`. `undefined` is not a separate value to check for: a value that
> may be missing is `T?`, and missing is `null`. See
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

> [!WARNING]
> Splicing a value that may be `null` into text shows the word `null` on
> the page. The checker warns (`T19`); say what to show instead with `??`,
> as in `Text(todo.note ?? "no note")`.

## Typing data from the network

`fetch` returns `Any`, because the checker cannot see what a URL will
return. Annotate the resource or the local variable to type its value:

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

The checker trusts the annotation. A development build (`wf serve`) also
holds each response to its declared type, and a mismatch is a `.parse`
error that names the place, such as `$[1].status is missing`.

When the server publishes an OpenAPI specification, let the compiler read
it instead: `api Backend from "openapi.json"` types every endpoint,
parameter, response and error from the file, and turns each named schema
into a `type`, or an `enum` for a string with cases (see
[Data and APIs](https://webfluent.monzeromer.dev/learn/15-data.md#read-the-service-from-an-openapi-file)). An `api` you write by
hand types its calls the same way: `get users() -> [User]`.

## The checks

The type checker reports these codes. All are errors except `T19`, `T20`
and `T21`, which are warnings.

| Code | When |
|---|---|
| `T01` | A value of the wrong type given to a prop, a state, a field, a parameter, an assignment, an argument, or a `bind:` |
| `T02` | A case the enum does not have |
| `T04` | A field or method read off a value that may be `null` |
| `T05` | A field or method a record or shape does not have |
| `T06` | A member or action a store does not have, or an endpoint a service does not have |
| `T07` | A list, a record or an action used as a condition, which is always true |
| `T08` | A `for` over something that is not a list |
| `T09` | An `emit` whose arguments do not match the event's |
| `T10` | A call with the wrong arguments: too many or too few, a name the callee does not take, or one of the wrong kind |
| `T11` | A `match` on something that is neither a resource nor an enum, or an arm the value cannot take |
| `T12` | A `Secret` where it would escape — shown, spliced into text, joined with `+`, handed to a browser global, logged, or kept with `persist` |
| `T13` | A name, or a function called, that nothing declares — which the browser would throw on |
| `T14` | A comparison that is always the same: `"1" == 1`, a record against a string, an enum against a case it does not have |
| `T15` | A `match` with no `else` that misses a case of the enum, or has a case twice |
| `T16` | A method a number, a string or a list does not have, with the nearest one it does |
| `T17` | An async action's result used before it is awaited, or `await` where it cannot run |
| `T18` | Arithmetic on something that is not a number |
| `T19` | A value that may be `null` spliced into text (warning) |
| `T20` | A list or a record shown as text (warning) |
| `T21` | A resource `match` with no `error` arm and no `else` (warning) |
| `T22` | A handler that names more values than its event sends |

Most findings come with a hint: a conversion (`Number(value)`,
`"{value}"`), the cases an enum takes, the fields a record has, or one of
the narrowing forms above. `wf explain T05` prints a code's full entry, and
[Diagnostics](https://webfluent.monzeromer.dev/errors.md) has every code with an example.

## What the checker does not check

- It does not check a lambda's body against a return type.
- It does not change a name's type across assignments: `state x = 1` stays
  a `Number`, and assigning a string to it is `T01`.
- It does not know what a plain `fetch` returns. Annotate it, or describe
  the service with an `api`.

## In the editor

Hover shows a name's inferred type: `state draft: String`, `derived
remaining: Number`, `Todos.add: action(String)`. Completion after `.`
offers a record's fields, an enum's cases where one is wanted, and the
methods of a `String`, a `Number` or a list. A type error appears as a
diagnostic on the argument it belongs to. To set up the editor, see
[Editors](https://webfluent.monzeromer.dev/reference/cli.md#editors).

## Quick reference

| Syntax | Meaning |
|---|---|
| `state x: T = v` | Annotate a state; props, parameters, fields and `let` take `: T` the same way |
| `[T]` · `T?` · `Map` · `Any` | A list, a value that may be `null`, an object with unknown keys, anything |
| `type Name { field: T = default }` | Declare a record |
| `type B = A { extra: T }` | Extend a record |
| `Name(field: v)` · `Name()` | Build a record |
| `enum Name { a, b(x: T) }` | Declare an enum, with an optional payload per case |
| `.case` · `.case(v)` | Write a case |
| `Number(1..=30)` · `String(minLength: 8)` | A type with a condition |
| `if let x = v { … }` · `v ?? d` · `v?.f` | Narrow or read through `null` |
| `resource r: [T] = fetch(…)` | Type data from the network |

## Common mistakes

**A field read off a value that may be `null`.** `email` is a `String?`, so
`.length` would throw when it is missing:

```wf expect=T04
type User { name: String, email: String? }

page P(path: "/", title: "T", description: "D") {
    state user: User = User(name: "Ada", email: null)
    Heading(user.name).h1
    Text("{user.email.length} characters")
}
```

Narrow it first, `if let mail = user.email { Text("{mail.length} characters") }`,
or read through it with a fallback, `user.email?.length ?? 0`. See
[T04](https://webfluent.monzeromer.dev/errors/T04.md).

**A misspelled field on a map literal.** A map literal has the fields it was
written with, so a typo is caught where it is read:

```wf expect=T05
page P(path: "/", title: "T", description: "D") {
    state form = { name: "", age: 0 }
    Heading("Profile").h1
    Input(bind: form.name, label: "Name")
    Text("Hello, {form.nmae}")
}
```

Use one of the fields the hint lists, `form.name`. See [T05](https://webfluent.monzeromer.dev/errors/T05.md).

**A value that may be `null` spliced into text.** The page would show the
word `null` when there is no note:

```wf expect=T19
type Todo { id: String, title: String, note: String? }

page P(path: "/", title: "T", description: "D") {
    state todo: Todo = Todo(id: "1", title: "Ship")
    Heading(todo.title).h1
    Text("Note: {todo.note}")
}
```

Say what to show instead, `Text("Note: {todo.note ?? "none"}")`, or draw the
line only when there is a note with `if let`. See [T19](https://webfluent.monzeromer.dev/errors/T19.md).

## Next

[Expressions](https://webfluent.monzeromer.dev/reference/expressions.md).
