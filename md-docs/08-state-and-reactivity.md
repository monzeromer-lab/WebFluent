# 8. State and reactivity

<!--
route: guide/state
group: basics
blurb: Reactivity here is fine-grained: a state is a signal, and only the thing that read it updates.
description: state, derived, effect, actions and let, persist, timers, clean-up, element handles and the browser's values.
-->

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
[chapter 13](13-types.md) has the type language. Assigning a value of another
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
    effect {
        log("query is now {query}")
    }
    effect {
        document.title = "Search: {query}"
        cleanup { document.title = "Search" }
    }
    Input(bind: query, placeholder: "Search")
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
  while a call of it runs ([chapter 16](16-forms.md#pending-actions)).
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

A `persist` may say more about itself — which storage, a version with
migrations, whether to follow other tabs — in a block after it; that and the
rest of the policy are in [Stores](12-stores.md#keeping-a-value-across-visits).
It works in a store too, where every page shares it. The key is the owner's
name and the state's (`Prefs.theme`), so a page and a store may each have a
`theme`. Storage that is blocked or full falls back to the initial value.

## Timers

`every(ms) { }` repeats; `after(ms) { }` fires once. The interval is a number
of milliseconds, or a duration: `every(5.seconds)`, `after(2.minutes)`. Both stop when what
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

## Automatic clean-up

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

Some names are always in scope, kept current by the runtime, so a page reads
the browser the way it reads its own state:

| Name | Is | Updates when |
|---|---|---|
| `viewport` | `.width`, `.height`, and `.sm` `.md` `.lg` `.xl` — whether the window is at least that breakpoint | a breakpoint is crossed (through `matchMedia`, not on every pixel) |
| `query` | the URL's query string as a map: `query.tab` for `?tab=x` | the URL changes |
| `hash` | the URL's `#fragment`, without the `#` | the URL changes |
| `theme` | `"light"`, `"dark"` or `"system"` — the reader's choice ([Styling](15-styling.md#dark-mode)) | `setTheme(…)` is called |
| `now` | a `DateTime`, every minute; `now(every: 1.seconds)` for a finer clock ([Types](13-types.md#now)) | the clock ticks |
| `network` | `.online`, `.effectiveType`, `.saveData`, `.downlink`, `.queued` ([Real-time](18-realtime.md#the-network-as-a-value)) | the connection changes |
| `update` | `.available` and `.apply()` — a new version of an offline site ([Offline](19-offline.md)) | a new build is installed |
| `locale`, `dir` | the current locale and `"ltr"`/`"rtl"`, with i18n configured ([i18n](21-i18n.md)) | `setLocale(…)` is called |

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

Breakpoints are the `screen-sm` … `screen-xl` design tokens — 640, 768, 1024
and 1280px unless a theme moves them. A `state` you declare with one of
these names shadows it. In the static paint they are unknown, so what depends
on them is drawn once the page is live — for layout, prefer
[responsive values](15-styling.md#responsive-values), which are right in the
first paint.

## Common mistakes

- **Writing state in a `derived`.** A derived value computes; it never
  assigns. Put the assignment in an action or an `effect`.
- **An effect that writes what it reads.** `effect { n = n + 1 }` reads `n`,
  so it runs again, forever. Compute it with `derived` instead.
- **Mutating a list and expecting a copy.** `items.push(x)` changes `items`
  in place and notifies its readers; `let copy = items` is the same list.
  Use `items.slice()` or `[...items]` for a copy.
- **Reading an element handle during set-up.** A `ref:` is filled once the
  element exists; read it in a handler, an `effect` or `after(0) { }`.
- **Expecting a component to re-run.** A component's body runs once. A value
  that should follow a prop or a state is a `derived`, or an expression
  written where it is shown.

## Where values come from, in order

1. Route parameters and props (`slug`, `_ label`).
2. `state`, `persist`, `derived`, `resource`, actions of the body.
3. A `use`d store's members, through the store's name (`Cart.total`).
4. Loop variables and arm bindings inside their block.
5. Constants, `data` and `image` declarations, `env.X`, and the browser's
   values above.
6. Browser globals: `window`, `document`, `localStorage`, `JSON`, `Math`,
   `Date`, `fetch`, `setTimeout`, `console`, `navigator`, `location`.

## Next

[Events](09-events.md).
