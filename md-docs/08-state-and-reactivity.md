# 5. State and reactivity

WebFluent's reactivity is fine-grained: a `state` is a signal, and anything
that reads it — a text, an attribute, a `derived` value, an `effect`, a style
splice — follows it. There is no re-render of a tree; the thing that read the
value is what updates.

## `state`

```wf
page Counter(path: "/") {
    state count = 0                  // inferred: Number
    state name: String = ""          // declared
    state items: [String] = []       // a list of strings
    state selected: String? = null   // may be null
    state user = { name: "Sam", tags: ["a"] }   // a map literal: its fields are its type

    Text("{name} has clicked {count} times")
    Button("+") { on click { count = count + 1 } }
}
```

A `state` is declared at the top of a page, component or store body, with an
initial value. Its type is read off the value, or written after a colon;
[chapter 10](10-types.md) has the type language. Assigning a value of another
type is an error (`T01`).

State is written only in an imperative block — a handler, an action, an
effect: `count = count + 1`, `user.name = "Ann"`, `items[0] = "x"`, or through
a method that changes it in place (`items.push("x")`). The static paint seeds
every state with its initial value, so a pre-rendered page shows what the
first paint would.

## `derived`

A derived value is computed from state and recomputed when any state it read
changes:

```wf
page Cart(path: "/") {
    state items = [{ name: "Pen", price: 2.5, qty: 4 }, { name: "Pad", price: 6, qty: 1 }]
    state taxRate = 0.2
    derived subtotal = items.reduce((sum, i) => sum + i.price * i.qty, 0)
    derived total = subtotal * (1 + taxRate)
    derived empty = items.length == 0

    Text("Subtotal: {format(subtotal, .currency)}")
    Text("Total: {format(total, .currency)}")
    if empty { Text("Your cart is empty.").muted }
}
```

A derived value may read other derived values and call actions. It is never
assigned to. It is seeded in the static paint from the initial state.

## `effect`

An effect runs once, and again whenever a state it read changes. Its
`cleanup { }` block runs before the next run and when the page (or the branch
the effect lives in) leaves:

```wf
page Watch(path: "/") {
    state query = ""
    state hits = 0
    effect {
        log("query is now {query}")
        hits = hits + 1
    }
    effect {
        document.title = "Search: {query}"
        cleanup { document.title = "Search" }
    }
    Input(bind: query, placeholder: "Search")
    Text("{hits} runs")
}
```

Use effects for the world outside the page — the document title, a
third-party widget, a subscription. Keep computation in `derived`.

## Actions and locals

An action is a named function of the page, component or store. Its body is an
imperative block: assignments, `let` locals, calls, `if`, `for`, `try`,
`return`, `await`.

```wf
page Todos(path: "/") {
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

    Input(bind: draft, placeholder: "What needs doing?") { on keydown(e) { if e.key == "Enter" { add() } } }
    Button("Add").primary { on click { add() } }
    for item, i in items {
        Row { Text(item)  IconButton(icon: "trash", label: "Remove") { on click { remove(i) } } }
    }
}
```

- `let x = …` declares a local of the action (or handler). `let { a, b } = m`
  and `let [x, y] = l` destructure.
- `await` inside an action makes it asynchronous; `name.pending` is `true`
  while a call of it runs ([chapter 6](06-events-and-forms.md#pending-actions)).
- `for item in list { }`, `for item, i in list { }` and `for n in 1..=10 { }`
  loop; `try { } catch e { }` guards a call that may fail; `return x` leaves
  with a value.
- A page's action is called by name, `add()`; a store's through the store,
  `Todos.add()`.

## `persist`

`persist` is a `state` kept in the browser's storage across visits. It is
read back on the next visit and written on every change:

```wf
page Prefs(path: "/") {
    persist theme = "light"
    persist perPage = 20
    Select(bind: theme, label: "Theme") {
        Select.Option("Light", value: "light")
        Select.Option("Dark", value: "dark")
    }
    Slider(bind: perPage, min: 10, max: 100, step: 10, label: "Rows per page")
}
```

It works in a store too, where every page shares it. The key is the owner's
name and the state's (`Prefs.theme`), so a page and a store may each have a
`theme`. Storage that is blocked or full falls back to the initial value.

## Timers

`every(ms) { }` repeats; `after(ms) { }` fires once. Both stop when what
declared them leaves the page — a branch that closes, a list item that is
removed, a route that changes:

```wf
page Clock(path: "/") {
    state seconds = 0
    state toast: String? = "Welcome"
    every(1000) { seconds = seconds + 1 }
    after(3000) { toast = null }
    Text("{seconds}s on this page")
    if let t = toast { Alert(t).info }
}
```

## Ownership: what leaves, leaves

Everything a body creates — effects, timers, listeners, handles — belongs to
the scope it was created in. When that scope's nodes leave (an `if` branch
turns false, an item drops out of a `for`, a `match` arm changes, a route
changes), the runtime disposes of them: intervals are cleared, effects fall
silent, cleanups run. You never unsubscribe by hand.

## Element handles

`ref: name` on an element gives you a handle, read as the element itself:

```wf
page Search(path: "/") {
    state q = ""
    Input(bind: q, ref: box, placeholder: "Search")
    Button("Focus the box") { on click { box.focus() } }
    Button("Clear") { on click { q = ""  box.focus() } }
    on key("/") { box.focus() }
}
```

The name is declared by its use; `box.value`, `box.scrollIntoView()`, anything
the element has.

## The browser as values

Four names are always in scope, kept current by the runtime:

```wf
page Where(path: "/") {
    Text(if viewport.md { "wide" } else { "narrow" })   // viewport.width, .height, .sm .md .lg .xl
    derived tab = query.tab ?? "all"                      // the URL's ?tab=x
    Text("Tab: {tab}")
    Text("Section: {hash}")                               // the URL's #fragment
    Text("Theme: {theme}")                                // light, dark or system
}
```

Breakpoints: `sm` 640px, `md` 768px, `lg` 1024px, `xl` 1280px. A `state` you
declare with one of these names shadows it. In the static paint they are
unknown, so a condition on them renders as its live branch once hydrated.

## Where values come from, in order

1. Route parameters and props (`slug`, `_ label`).
2. `state`, `persist`, `derived`, `resource`, actions of the body.
3. A `use`d store's members, through the store's name (`Cart.total`).
4. Loop variables and arm bindings inside their block.
5. Constants and `data` files, `env.X`, the browser's values.
6. Browser globals: `window`, `document`, `localStorage`, `JSON`, `Math`,
   `Date`, `fetch`, `setTimeout`, `console`, `navigator`, `location`.

## Next

[Events and forms](06-events-and-forms.md).
