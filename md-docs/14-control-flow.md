# 14. Control flow

<!--
route: guide/control-flow
group: concepts
blurb: Five statements decide what a page shows: if, if let, for, show and match. Each adds and removes elements as its condition changes.
description: if, if let, for … by, VirtualList, show and match, in render blocks and in actions, and if and match as values.
-->

Five statements decide what a page shows: `if`, `if let`, `for`, `show` and
`match`. Each one follows the state it reads, adding and removing elements as
its condition changes, so you never update the page by hand. Each animates
when you ask it to ([Motion](20-motion.md)).

In this chapter:

- `if`, `else if` and `else`, and `if let` for a value that may be null
- `for` over a list, with a key — and `VirtualList` for thousands of rows
- `show`, which hides elements without removing them
- `match`, which shows one arm by an enum's case
- `if` and `match` as values, and the same statements in actions

The statements in this chapter appear in a *render block* — the body of a
page, component or element, which says what to draw. The last section shows
them in an *imperative block* — a handler or an action, which runs code.

## `if` / `else if` / `else`

`if` shows a branch while its condition holds. This page shows one of three
messages, by how many times the button was pressed:

```wf
page Status(path: "/", title: "Status", description: "A message chosen by a count.") {
    state count = 0
    Heading("Status").h1
    if count == 0 {
        Text("Nothing yet.").muted
    } else if count < 10 {
        Text("A few: {count}")
    } else {
        Text("Many: {count}").bold
    }
    Button("+1") { on click { count = count + 1 } }
}
```

How it behaves:

- A branch's elements are created when its condition turns true, and removed
  when it turns false.
- State declared inside a component in the branch starts over each time the
  branch is created.
- A condition must be a `Bool`, or a value that may be null, which counts as
  `false` when it is null.
- A list or a record as a condition is an error (`T07`), since it would always
  be true.

## `if let`

`if let name = value { }` binds the value inside the branch when it is not
null. It is the way to read a value of type `T?` (one that may be null)
without the type checker objecting:

```wf
type User { name: String, email: String? }

page Profile(path: "/", title: "Profile", description: "A user who may not be signed in.") {
    state user: User? = null
    Heading("Profile").h1
    if let u = user {
        Text("Signed in as {u.name}")
        if let mail = u.email { Text(mail).muted } else { Text("No email on file").muted }
    } else {
        Button("Sign in") { on click { user = User(name: "Sam", email: null) } }
    }
}
```

The rules:

- `if let x { }`, with no `=`, does the same for a name already in scope.
- The narrowing holds inside the branch only. `if x != null { }` and
  `if x && … { }` narrow `x` inside their branch the same way; the right-hand
  side of the `&&` is not narrowed.
- The binding follows the value. When `user` becomes another user, the branch
  shows the new one in place. It is drawn again only when the value goes to
  `null` and back.

## `for`

`for` draws its block once for each item of a list. This page loops three
ways — by key, with the index, and over a range:

```wf
type Todo { id: String, title: String, done: Bool = false }

page Checklist(path: "/", title: "Checklist", description: "Three kinds of loop.") {
    state todos: [Todo] = [Todo(id: "a", title: "Write"), Todo(id: "b", title: "Ship", done: true)]
    Heading("Checklist").h1

    for todo in todos by todo.id {                      // keyed: each item keeps its nodes
        Row(gap: .sm) {
            Checkbox(checked: todo.done, label: todo.title) { on change { todo.done = !todo.done } }
        }
    }
    for todo, i in todos {                              // with the index
        Text("{i + 1}. {todo.title}")
    }
    for n in 1..=3 { Badge("{n}") }                     // over a range
}
```

`by expr` names the *key*: the value that tells one item from another. With a
key:

- An item that stays in the list keeps its nodes. Focus, scroll position and
  animation state survive an insert, a removal or a move, and moves slide.
- An item keeps its nodes under a new value too — a refreshed response's rows
  are all new objects. Its body reads the item and the index live, so a
  changed row changes in place, and its components' state with it.
- Keys must be unique. The runtime warns once when two items share one.
- A key may read the index as well as the item
  (`for t, i in rows by "{t.id}-{i}"`).

Without a key, the list is rebuilt whenever it changes. Looping over something
that is not a list is an error (`T08`).

> [!WARNING]
> Do not key by the index alone. A key of the index moves with the position,
> not the item: after an insert every item has another's key, and nothing is
> kept. Key by what identifies the item.

### Long lists

A list of thousands of rows — a project's files, a search's results, a long
diff — belongs inside a **`VirtualList`**. The box scrolls, and only the rows
in view and a few either side are drawn; two spacers stand for the rest, so the
scrollbar is the whole list's. Its body is one keyed `for`.

This page windows ten thousand file paths of a fixed height, and a list of
search hits whose rows are measured:

```wf
type Hit { id: String, heading: Bool, text: String }

page Search(path: "/", title: "Search", description: "Results, a window at a time.") {
    state rows = (0..10000).map((i) => "src/file{i}.wf")
    state hits: [Hit] = []
    state current = ""
    Heading("Search").h1
    VirtualList(rowHeight: 32, label: "Files") {
        style { height: 400px }
        for path in rows by path { Text(path) { style { height: 32px; margin: 0 } } }
    }
    VirtualList(estimate: 24, reveal: current, label: "Results") {
        style { height: 400px }
        for h in hits by h.id {
            if h.heading { Heading(h.text).h2 } else { Text(h.text) }
        }
    }
}
```

Its props:

- **`rowHeight:`** says every row is that tall. It is the fast path.
- Without `rowHeight:`, each row is measured once drawn, and **`estimate:`**
  stands for the rows not yet drawn. When a row above what is in view turns
  out taller or shorter, the scroll moves by as much, so what the reader is
  looking at stays put.
- **`reveal:`** reads a row's key. When it changes, that row is scrolled into
  view — the selected file as the arrow keys move, the current match.
- **`label:`** is the box's accessible name.

How it draws:

- A row keeps its nodes while it stays in the window, and is updated in place
  when its item changes, as a keyed `for`'s rows are. A row scrolled out of
  the window is let go.
- Give rows no vertical margins: a row's height is its box's.
- A tree is a list of the rows that are showing. Flatten it in a `derived`
  (each open folder followed by its children) and window that.
- A list drawn before its box is laid out — inside an `if` that has turned
  true, in a panel not shown yet — draws only the few rows it keeps past the
  window until the box has a height. The first height it has is laid out before that frame is painted,
  so the first frame shows the rows in view, not a guess at them trimmed a
  frame later. A row `reveal:` asks for in the meantime is scrolled to then,
  so that frame shows it. A list hidden and shown again is laid out the same
  way.
- The box takes what any element takes ([Elements](11-elements.md)): a
  `role: "listbox"`, a `tabindex:`, an `aria-activedescendant:` that follows
  the current row, `data-*`, and its `on` handlers — an `on keydown` that
  moves through a list of references, for example.
- The pre-rendered page paints the first 60 rows. The runtime draws the window
  when it takes over.
- With anything else in its body, or a `for` with no `by`, it draws every row
  (`V12`).

> [!IMPORTANT]
> The box needs a height that does not come from its rows — `style { height:
> 400px }`, a `max-height:`, or a grid cell or flex row that sets one. A box
> as tall as what it holds grows until every row is drawn.

## `show`

`show` keeps its elements in the page and toggles whether they are visible;
`if` creates and destroys them. Use `show` for something expensive to rebuild,
or whose state must survive being hidden:

```wf
page Panel(path: "/", title: "Panel", description: "A panel hidden, not removed.") {
    state open = true
    Heading("Panel").h1
    Button("Toggle") { on click { open = !open } }
    show open {
        Card { Text("Still here when hidden; my state survives.") }
    }
}
```

## `match`

`match` shows one arm at a time. Over an **enum** it chooses by case, and a
case with a *payload* — values carried by the case — hands them to its arm.
This job moves through its cases as the buttons are pressed:

```wf
enum Status { idle, running(job: String), failed(reason: String), done(count: Number) }

page Job(path: "/", title: "Job", description: "One arm per case of an enum.") {
    state s: Status = .idle
    Heading("Job").h1
    match s {
        .idle { Button("Start") { on click { s = .running("build-42") } } }
        .running(job) { Row { Spinner.sm  Text("Running {job}")  Button("Fail") { on click { s = .failed("timed out") } } } }
        .failed(reason) { Alert("Failed: {reason}").danger }
        .done(n) { Text("Done, {n} steps") }
    }
    Text(match s { .idle { "waiting" } .failed(r) { r } else { "busy" } })   // as a value
}
```

Over a **resource** it chooses by the request's state — loading, error or
ready ([Fetching data](24-data.md)).

The rules:

- An arm binds one name per part of the payload: `.failed(reason)`.
- `else` covers every case without an arm of its own.
- Without an `else`, a `match` must give every case of the enum an arm, or it
  is an error (`T15`). That holds for a `match` statement and a `match`
  expression alike; an expression that covers every case needs no `else`.
- An arm is drawn again when the arm changes, not when its value does. A new
  payload of the same case — `.failed("refused")` after `.failed("timed
  out")` — reaches what the arm drew in place.

## Expressions that choose

`if` and `match` are also values, and `??` picks a fallback for a value that
may be null:

```wf
page Choose(path: "/", title: "Choose", description: "Values picked by a condition.") {
    state n = 5
    state nick: String? = null
    derived size = if n > 10 { "big" } else { "small" }
    derived label = nick ?? "anonymous"
    Heading("Choose").h1
    Text("{size} {label}")
}
```

An `if` value with no `else` is `null` when its condition fails.
[Expressions](18-expressions.md) has the rest.

## In actions and handlers

The same statements work in an imperative block, on values rather than
elements, plus `try`/`catch`. This action fetches rows and keeps the first
hundred:

```wf
page Sync(path: "/", title: "Sync", description: "Control flow inside an action.") {
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
    Heading("Sync").h1
    Button("Pull") { on click { pull() } }
    if error != "" { Alert(error).danger }
    Text("{rows.length} rows")
}
```

## Summary

| You write | It shows | Section |
|---|---|---|
| `if c { … } else if d { … } else { … }` | the first branch whose condition holds | [`if`](#if-else-if-else) |
| `if let u = user { … }` | a branch with the value, when it is not null | [`if let`](#if-let) |
| `for todo in todos by todo.id { … }` | the block once per item, keyed | [`for`](#for) |
| `for t, i in todos { … }`, `for n in 1..=3 { … }` | with the index; over a range | [`for`](#for) |
| `VirtualList(rowHeight: 32) { for … by … }` | only the rows in view | [Long lists](#long-lists) |
| `show open { … }` | its block, hidden rather than removed | [`show`](#show) |
| `match s { .idle { … } else { … } }` | one arm by case, or by a resource's state | [`match`](#match) |
| `if c { a } else { b }`, `match s { … }`, `a ?? b` | a value | [Expressions that choose](#expressions-that-choose) |
| `try { … } catch e { … }` | in an action: a guard on what may fail | [In actions and handlers](#in-actions-and-handlers) |

## Next

[Components](15-components.md).
