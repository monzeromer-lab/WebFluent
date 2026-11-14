---
chapter: 7
kind: "tutorial"
route: "/learn/07-state-and-reactivity"
from: ["/docs/guide/state"]
group: "language"
blurb: "A state is a signal, and only what reads it updates: no tree re-renders, no subscriptions to manage."
description: "state, derived, effect, actions and let, persist, timers, automatic clean-up, element handles and the browser's values."
learned: ["Declare a `state` and change it from a handler or an action", "Work a value out with `derived` instead of keeping a second state in step", "Reach outside the page with an `effect` and its `cleanup`", "Keep a value across visits with `persist`, and run timers with `every` and `after`", "Read `viewport`, `query`, `now` and the browser's other values as live values"]
---
# 7. State and reactivity

This chapter covers state and reactivity in WebFluent. A `state` is a value
the page can change. Anything that reads it — a text, an attribute, a
`derived` value, an `effect`, a style splice — updates when it changes, and
nothing else does. This is *fine-grained reactivity*: there is no
re-render of a component tree, and you never subscribe or unsubscribe by hand.

In this chapter:

- `state`, `derived` and `effect` — the three kinds of reactive value
- actions and `let` locals — the code that changes state
- `persist`, timers and element handles
- how the runtime cleans up after a page, a branch or a list item
- the browser's own values: `viewport`, `query`, `now` and the rest
- where a name is looked up, and the mistakes people make

## `state`

You declare a state at the top of a page, component or store body, with an
initial value. This counter shows the whole idea — one state, a text that
reads it, and a button that writes it:

```wf
page Counter(path: "/", title: "Counter", description: "A state, read and written.") {
    state count = 0                  // inferred: Number
    state name: String = "Sam"       // declared
    state items: [String] = []       // a list of strings
    state selected: String? = null   // may be null
    state user = { name: "Sam", tags: ["admin"] }   // a map literal: its fields are its type

    Heading("Counter").h1
    Text("{name} has clicked {count} times")
    Button("+1") { on click { count = count + 1 } }
}
```

Each click sets `count`, and only the `Text` that reads it changes.

> [!NOTE]
> **Coming from React?** `useState` returns a value and a setter, and a
> change runs the whole component again. A `state` is assigned directly,
> `count = count + 1`, and the body never runs again: only what reads
> `count` updates. A value that should follow a prop or a state is a
> `derived`. See [Coming from React](../coming-from/react.md#things-that-work-differently).

The type of a state is read off its initial value, or written after a colon.
[Types](12-types.md) has the type language. Assigning a value of another type
is an error (`T01`).

You write state only in an imperative block — a handler, an action or an
effect. You can:

- assign it: `count = count + 1`
- change a place inside it: `user.name = "Ann"`, `items[0] = "Pen"`
- call a method that changes it: `items.push("Pen")`, `items.sort()`

A change inside a value gives the state a new copy with that place changed, so
everything that reads it repaints, and a `persist` value is written.

The *static paint* — the HTML the build pre-renders for a page — seeds every
state with its initial value, so a pre-rendered page shows what the first
paint in the browser would.

## `derived`

A derived value is computed from state, and computed again when any state it
read changes. This cart works out its subtotal, total and whether it is empty:

```wf
page Cart(path: "/", title: "Cart", description: "Totals worked out from state.") {
    state items = [{ name: "Pen", price: 2.5, qty: 4 }, { name: "Pad", price: 6, qty: 1 }]
    state taxRate = 0.2
    derived subtotal = items.reduce((sum, i) => sum + i.price * i.qty, 0)
    derived total = subtotal * (1 + taxRate)
    derived empty = items.length == 0

    Heading("Your cart").h1
    Text("Subtotal: {format(subtotal, .currency)}")
    Text("Total: {format(total, .currency)}")
    if empty { Text("Your cart is empty.").muted }
}
```

The rules:

- A derived value may read state, other derived values and the browser's
  values, and may call an action that returns a value.
- It is never assigned to (`X01`), and an action it calls may not assign
  state (`X04`).
- The static paint seeds it from the initial state.

## `effect`

An effect runs once, then again whenever a state it read changes. Its
`cleanup { }` block runs before the next run, and when the page — or the
branch the effect lives in — leaves:

```wf
page Watch(path: "/", title: "Watch", description: "Effects that follow a search.") {
    state search = ""
    effect {
        log("search is now {search}")
    }
    effect {
        document.title = "Search: {search}"
        cleanup { document.title = "Search" }
    }
    Heading("Watch").h1
    Input(bind: search, placeholder: "Search")
}
```

Use an effect for the world outside the page: the document title, a
third-party widget, a subscription. Keep computation in `derived`.

> [!WARNING]
> An effect that writes what it reads — `effect { n = n + 1 }` — starts its own
> next run and never settles. The compiler refuses it (`X02`). Compute the
> value with `derived` instead.

## Actions and locals

An action is a named function of a page, component or store. Its body is an
imperative block: assignments, `let` locals, calls, `if`, `for`, `try`,
`return` and `await`. This to-do list adds, removes and loads items with
three actions:

```wf
page Todos(path: "/", title: "To-dos", description: "Actions that change a list.") {
    state items: [String] = []
    state draft = ""

    action add() {
        let text = draft.trim()
        if text == "" { return }
        items.push(text)
        draft = ""
    }

    action remove(index: Number) {
        items = items.filter((_, i) => i != index)
    }

    action load() {
        let r = await fetch("/api/todos")
        let { todos } = r
        items = todos
    }

    Heading("To-dos").h1
    Input(bind: draft, placeholder: "What needs doing?") { on keydown(e) { if e.key == "Enter" { add() } } }
    Button("Add").primary { on click { add() } }
    for item, i in items {
        Row { Text(item)  IconButton(icon: "trash", label: "Remove") { on click { remove(i) } } }
    }
}
```

What an action's body can hold:

- **Locals.** `let x = …` declares a local of the action or handler.
  `let { a, b } = m` and `let [x, y] = l` destructure a map and a list.
- **`await`.** Using it makes the action asynchronous, and `name.pending` is
  `true` while a call of it runs ([Forms](14-forms.md#show-that-a-request-is-running)).
- **Loops.** `for item in list { }`, `for item, i in list { }` and
  `for n in 1..=10 { }`.
- **Errors.** `try { } catch e { }` guards a call that may fail.
- **`return`.** `return` leaves the action; `return x` leaves with a value.

A `let` is the action's own. After `let list = items`, `list.push(x)` changes
`list` and leaves `items` as it was; assign it back with `items = list`, or
write `items.push(x)`, which gives the state a new list.

You call a page's action by name, `add()`, and a store's through the store,
`Todos.add()`.

## `persist`

`persist` declares a state kept in the browser's storage across visits. It is
read back on the next visit and written on every change. These two
preferences survive a reload:

```wf
page Prefs(path: "/", title: "Preferences", description: "Settings kept across visits.") {
    persist scheme = "light"
    persist perPage = 20
    Heading("Preferences").h1
    Select(bind: scheme, label: "Theme") {
        Select.Option("Light", value: "light")
        Select.Option("Dark", value: "dark")
    }
    Slider(bind: perPage, min: 10, max: 100, step: 10, label: "Rows per page")
}
```

How it behaves:

- The storage key is the owner's name and the state's (`Prefs.scheme`), so a
  page and a store may each have a value of the same name.
- It works in a store too, where every page shares it.
- When storage is blocked or full, the value falls back to its initial value.
- A block after the declaration can say which storage to use, give a version
  with migrations, and choose whether to follow other tabs. That block and the
  rest of the policy are in [Stores](11-stores.md#keeping-a-value-across-visits).

## Timers

`every(ms) { }` repeats, and `after(ms) { }` fires once. The interval is a
number of milliseconds or a duration: `every(5.seconds)`, `after(2.minutes)`.
This page counts the seconds it has been open and hides a welcome message
after three:

```wf
page Clock(path: "/", title: "Clock", description: "A timer and a one-off delay.") {
    state seconds = 0
    state toast: String? = "Welcome"
    every(1000) { seconds = seconds + 1 }
    after(3000) { toast = null }
    Heading("Clock").h1
    Text("{seconds}s on this page")
    if let message = toast { Alert(message).info }
}
```

Both stop when what declared them leaves the page: a branch that closes, a
list item that is removed, a route that changes.

## Automatic clean-up

Everything a body creates — effects, timers, listeners, element handles —
belongs to the *scope* it was created in: the page, branch, list item, `match`
arm or slot whose nodes it sits among. When that scope's nodes leave, the
runtime disposes of what it created:

- an `if` branch turns false, an item drops out of a `for`, a `match` arm
  changes, or the route changes;
- intervals are cleared, effects stop running, and `cleanup { }` blocks run.

You never unsubscribe by hand.

## Element handles

`ref: name` on an element gives you a *handle*: a name that reads as the
element itself. Here a button and the `/` key both put the cursor in the
search box:

```wf
page Search(path: "/", title: "Search", description: "A handle on an input.") {
    state q = ""
    Heading("Search").h1
    Input(bind: q, ref: box, placeholder: "Search")
    Button("Focus the box") { on click { box.focus() } }
    Button("Clear") { on click { q = ""  box.focus() } }
    on key("/") { box.focus() }
}
```

The `ref:` declares the name; you do not declare it anywhere else. Anything
the element has is there: `box.value`, `box.scrollIntoView()`.

A handle is filled once the element exists, so set-up code at the top of the
body cannot use it yet. Read it in a handler, an `effect` or `after(0) { }`.

## The browser as values

Some names are always in scope and kept current by the runtime, so a page
reads the browser the way it reads its own state:

| Name | Is | Updates when |
|---|---|---|
| `viewport` | `.width`, `.height`, and `.sm` `.md` `.lg` `.xl` — whether the window is at least that breakpoint | a breakpoint is crossed (through `matchMedia`, not on every pixel) |
| `query` | the URL's query string as a map: `query.tab` for `?tab=x` | the URL changes |
| `hash` | the URL's `#fragment`, without the `#` | the URL changes |
| `theme` | `"light"`, `"dark"` or `"system"` — the reader's choice ([Styling](13-styling.md#dark-mode)) | `setTheme(…)` is called |
| `now` | a `DateTime`, every minute; `now(every: 1.seconds)` for a finer clock ([Types](12-types.md#now)) | the clock ticks |
| `network` | `.online`, `.effectiveType`, `.saveData`, `.downlink`, `.queued` ([Real-time](../recipes/show-the-network.md)) | the connection changes |
| `update` | `.available` and `.apply()` — a new version of an offline site ([Offer a new version](../recipes/offer-a-new-version.md)) | a new build is installed |
| `locale`, `dir` | the current locale and `"ltr"`/`"rtl"`, with i18n configured ([Translate a site](../recipes/translate-a-site.md)) | `setLocale(…)` is called |

This page reads four of them:

```wf
page Where(path: "/", title: "Where", description: "The browser, read as values.") {
    Heading("Where am I?").h1
    Text(if viewport.md { "A wide window" } else { "A narrow window" })
    derived tab = query.tab ?? "all"
    Text("Tab: {tab}, section: {hash}")
    Text("Theme: {theme}")
    if !network.online { Alert("You are offline.").warning }
}
```

The rules:

- The breakpoints are the `screen-sm` … `screen-xl` design tokens: 640, 768,
  1024 and 1280px unless a theme moves them.
- They are the language's names, so a `state`, prop or `const` of the same
  name is an error (`E118`).
- Copy one into a `state` and you keep the value it had when the page
  opened: `state clock = now(every: 1.seconds)` never ticks (`X06`). Read it
  where it is shown, or in a `derived`.
- The static paint does not know them, so what depends on them is drawn once
  the page is live. For layout, prefer
  [responsive values](13-styling.md#responsive-values), which are right in
  the first paint.

## Where values come from, in order

When a body reads a name, the nearest declaration wins. The compiler looks
in this order, innermost first:

1. A `let` local, a lambda's parameter or a caught error, inside its block.
2. Loop variables and arm bindings (`if let`, a `match` arm), inside their
   block.
3. Route parameters and props (`slug`, `_ label`).
4. `state`, `persist`, `derived`, `resource` and actions of the body. A
   body member may not share a name with a parameter or a prop (`E102`).
5. A `use`d store's members, always through the store's name
   (`Cart.total`).
6. Constants, `data` and `image` declarations, `env.X`, and the browser's
   values above.
7. Browser globals: `window`, `document`, `localStorage`, `JSON`, `Math`,
   `Date`, `fetch`, `setTimeout`, `console`, `navigator`, `location`.

So with `state name = "outer"`, `for name in items by name { Text(name) }`
shows each item, and a `const name` is hidden by both.

## Summary

| You write | It is | Section |
|---|---|---|
| `state count = 0` | a value the page changes; its readers update | [`state`](#state) |
| `derived total = …` | a value computed from state, never assigned | [`derived`](#derived) |
| `effect { … cleanup { … } }` | code that runs when what it read changes | [`effect`](#effect) |
| `action add() { … }` | a named function that changes state | [Actions and locals](#actions-and-locals) |
| `let x = …` | a local of an action or handler | [Actions and locals](#actions-and-locals) |
| `persist scheme = "light"` | a state kept across visits | [`persist`](#persist) |
| `every(1000) { … }`, `after(3000) { … }` | a repeating timer and a one-off delay | [Timers](#timers) |
| `ref: box` | a handle on an element | [Element handles](#element-handles) |
| `viewport`, `query`, `hash`, `theme`, `now`, `network`, `update`, `locale`, `dir` | the browser, read as values | [The browser as values](#the-browser-as-values) |

[Grammar](../reference/grammar.md) and [Built-in functions](../reference/built-ins.md) are
the reference for each.

## Quiz

```quiz
A page shows `Heading("Counter")` and `Text("{count}")`. A click runs `count = count + 1`. What runs again?
- The page's body, which draws the heading and the text again -- A body runs once, when the page opens; nothing re-renders after it.
+ The `Text` that reads `count`, and nothing else -- Only what reads a state follows it; the heading reads no state.
- The page and every component inside it -- There is no component tree to re-render; a component's body runs once too.
```

```quiz
`items` is a state. Which `total` stays equal to the sum of `items` as items are added?
- `state total = items.sum()` -- A state takes its initial value once; adding an item leaves the total as it was.
+ `derived total = items.sum()` -- A derived value is worked out again whenever a state it read changes.
- `derived total = 0`, with `total = items.sum()` in each action that adds an item -- A derived value is never assigned (X01).
```

```quiz
Which of these clocks ticks every second?
- `state clock = now(every: 1.seconds)` -- A state takes `now` once, when the page opens (X06).
+ `derived clock = now(every: 1.seconds)` -- A derived value follows `now` as it ticks.
- `derived clock = now` -- This one ticks, but once a minute: `now` alone keeps minutes, and `now(every: 1.seconds)` asks for seconds.
```

## Common mistakes

**A state that copies `now` once.** A `state` takes its initial value when the
page opens and keeps it, so a clock or an "… ago" worked out from it never
moves:

```wf expect=X06
page Feed(path: "/", title: "Feed", description: "When this was posted.") {
    state posted = @2026-03-14T09:30:00Z
    state clock = now(every: 1.seconds)
    Heading("Feed").h1
    Text("Posted {posted.until(clock).minutes()} minutes ago")
}
```

Declare it `derived clock = now(every: 1.seconds)`. The same holds for
`viewport`, `query`, `hash`, `theme`, `network` and `update`
([X06](../errors/X06.md)).

**An effect that writes what it reads.** Each run changes `visits`, which
starts the next run, so the page never settles:

```wf expect=X02
page Visits(path: "/", title: "Visits", description: "A count that runs away.") {
    state visits = 0
    effect { visits = visits + 1 }
    Heading("Visits").h1
    Text("{visits} visits")
}
```

Change the state in a handler or an action, or work the value out with
`derived` ([X02](../errors/X02.md)).

**Assigning a derived value.** A `derived` is worked out from what it reads,
so nothing may set it:

```wf expect=X01
page Cart(path: "/", title: "Cart", description: "A total you cannot set.") {
    state items = [2.5, 6]
    derived total = items.sum()
    Heading("Cart").h1
    Text("Total: {total}")
    Button("Clear") { on click { total = 0 } }
}
```

Change what it reads instead, `items = []`, and `total` follows
([X01](../errors/X01.md)).

## Next

[Events](08-events.md).
