# 10. Control flow

<!--
route: guide/control-flow
group: basics
blurb: Five statements decide what shows: if, if let, for, show and match. Each creates and removes elements as its condition changes.
description: if, if let, for … by, show and match, in render blocks and in actions.
-->

Inside a render block, five statements decide what shows: `if`, `if let`,
`for`, `show` and `match`. Each creates and removes elements as its condition
changes, and each animates if you ask it to ([chapter 20](20-motion.md)).

## `if` / `else if` / `else`

```wf
page Status(path: "/") {
    state count = 0
    if count == 0 {
        Text("Nothing yet.").muted
    } else if count < 10 {
        Text("A few: {count}")
    } else {
        Text("Many: {count}").bold
    }
    Button("+") { on click { count = count + 1 } }
}
```

The branch's elements are created when the condition turns true and removed
when it turns false; state declared inside a component in the branch starts
over each time. A condition must be a `Bool` (or a value that may be null,
which is `false` when null); a list or a record as a condition is an error
(`T07`), since it would always be true.

## `if let`

`if let name = value { }` binds the non-null value inside the branch — the
way to read a `T?` without the checker objecting:

```wf
type User { name: String, email: String? }

page Profile(path: "/") {
    state user: User? = null
    if let u = user {
        Text("Signed in as {u.name}")
        if let mail = u.email { Text(mail).muted } else { Text("No email on file").muted }
    } else {
        Button("Sign in") { on click { user = User(name: "Sam", email: null) } }
    }
}
```

`if let x { }` (no `=`) is the same for a name already in scope. The narrowing
holds inside the branch only; `x != null` and `x && …` narrow the same way.

## `for`

```wf
type Todo { id: String, title: String, done: Bool = false }

page List(path: "/") {
    state todos: [Todo] = [Todo(id: "a", title: "Write"), Todo(id: "b", title: "Ship", done: true)]

    for todo in todos by todo.id {                      // keyed: each item keeps its nodes
        Row(gap: .sm) {
            Checkbox(checked: todo.done, label: todo.title)
        }
    }
    for todo, i in todos {                              // with the index
        Text("{i + 1}. {todo.title}")
    }
    for n in 1..=3 { Badge("{n}") }                     // over a range
}
```

`by expr` names the key: with it, an item that stays in the list keeps its
nodes — focus, scroll and animation state survive an insert, a removal or a
move, and moves slide. Without a key the list is rebuilt when it changes.
Keys must be unique; the runtime warns once when two items share one.
Looping over something that is not a list is an error (`T08`).

## `show`

`show` keeps the elements in the DOM and toggles their visibility, where `if`
creates and destroys them. Use it for something expensive to rebuild or whose
state must survive being hidden:

```wf
page Panel(path: "/") {
    state open = true
    Button("Toggle") { on click { open = !open } }
    show open {
        Card { Text("Still here when hidden; my state survives.") }
    }
}
```

## `match`

`match` shows one arm at a time. Over an **enum** it chooses by case; a case
with a payload hands it to the arm:

```wf
enum Status { idle, running(job: String), failed(reason: String), done(count: Number) }

page Job(path: "/") {
    state s: Status = .idle
    match s {
        .idle { Button("Start") { on click { s = .running("build-42") } } }
        .running(job) { Row { Spinner.sm  Text("Running {job}")  Button("Fail") { on click { s = .failed("timed out") } } } }
        .failed(reason) { Alert("Failed: {reason}").danger }
        .done(n) { Text("Done, {n} steps") }
    }
    Text(match s { .idle { "waiting" } .failed(r) { r } else { "busy" } })   // as a value
}
```

Over a **resource** it chooses by the request's state — [chapter 17](17-data.md).

An arm binds one name per part of the payload; `else` covers the rest. Every
case of an enum has to be handled or covered by `else` only in a match
*expression* (which needs an `else` to have a value); a `match` statement may
list only the arms it shows.

## Expressions that choose

`if` and `match` are also values, and `??` picks a fallback:

```wf
page Choose(path: "/") {
    state n = 5
    state nick: String? = null
    derived size = if n > 10 { "big" } else { "small" }
    derived label = nick ?? "anonymous"
    Text("{size} {label}")
}
```

## In actions and handlers

The same statements work in an imperative block, on values rather than
elements, plus `try`/`catch`:

```wf
page Sync(path: "/") {
    state rows: [Map] = []
    state error = ""
    action pull() {
        try {
            let r = await fetch("/api/rows")
            let { items } = r
            rows = []
            for item, i in items {
                if i >= 100 { return }
                rows.push(item)
            }
        } catch e {
            error = e.message
        }
    }
    Button("Pull") { on click { pull() } }
    if error != "" { Alert(error).danger }
    Text("{rows.length} rows")
}
```

## Next

[Components](11-components.md).
