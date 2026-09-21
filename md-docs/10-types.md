# 10. Types

WebFluent has a gradual, structural type checker. It runs in `wf build` and
in the editor, before anything is emitted, and its findings are errors with
a hint. A program that declares no types checks as it always did — what the
checker cannot resolve is `Any`, and `Any` agrees with everything — and
every annotation you add narrows what it can say.

## The type language

| Type | Written | Values |
|---|---|---|
| String | `String` | `"text"`, `"with {interpolation}"` |
| Number | `Number` | `42`, `3.14` |
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
  as `data-<prop>` ([chapter 8](08-components.md#enum-props)).

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
Anything else is `Any`.

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

The checker trusts the annotation; it does not read the server's schema.

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

Each comes with a hint: a conversion (`Number(value)`, `"{value}"`), the
cases an enum takes, the fields a record has, the unwrapping forms above.

What the checker does not do: it does not check a lambda's body against a
return type, does not change a name's type across assignments (`state x =
1` stays a `Number`; assigning a string to it is `T01`), and does not read
the schema of a `fetch`.

## In the editor

Hover shows a name's inferred type (`state draft: String`, `derived
remaining: Number`, `Todos.add: action(String)`); completion after `.`
offers a record's fields, an enum's cases where one is wanted, and the
methods of a `String`, a `Number` or a list. A type error is a diagnostic
at the argument it belongs to. [Chapter 18](18-tooling.md#editors) sets it up.

## Next

[Expressions](11-expressions.md).
