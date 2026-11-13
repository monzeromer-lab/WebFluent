---
chapter: 16
kind: "tutorial"
route: "guide/stores"
group: "concepts"
blurb: "A store is state shared by every page and component that uses it, built the first time something reads it."
description: "Stores: declaration, scopes, use, actions, persist and migrations, connections, loading data, debugging, testing, and when to use one."
---
# 16. Stores

A *store* holds state that more than one page or component shares: the
cart, the signed-in user, the theme, a cache of what was fetched. It holds
`state`, `derived` values and actions, and every page that reads it sees
the same values.

The one thing to remember: **a store is built the first time something
reads it.** Declaring a store does nothing. The first `Cart.items`,
`Cart.count` or `Cart.add(…)` builds it, and every later read uses that
same instance. As a result, the order you declare stores in does not
matter — a `derived` value may read a store declared below it — and a store
nothing reads costs nothing, in time or in bytes.

**In this chapter**

- Declaring a store and reading it from pages
- How long a store lives: its scope
- Actions, including async actions
- Keeping a value across visits with `persist`, and changing its shape later
- Connections and data a store owns
- Inspecting and testing a store
- When to use a store, and when a page's own state

## A first store

This program declares a `Cart` store and reads it from two pages:

```wf
type Item { id: String, name: String, price: Number, qty: Number = 1 }

store Cart {
    state items: [Item] = []
    derived count = items.length
    derived total = items.reduce((sum, i) => sum + i.price * i.qty, 0)
    action add(item: Item) { items.push(item) }
    action remove(id: String) { items = items.filter(i => i.id != id) }
}

page Shop(path: "/", title: "Shop", description: "Things to buy.") {
    use Cart
    Button("Add a pen").primary { on click { Cart.add(Item(id: "pen", name: "Pen", price: 2.5)) } }
    Text("{Cart.count} items, {format(Cart.total, .currency)}")
}

page CartPage(path: "/cart", title: "Cart", description: "What you are buying.") {
    use Cart
    for item in Cart.items by item.id {
        Row(justify: .between) {
            Text("{item.name} × {item.qty}")
            Button("Remove").sm { on click { Cart.remove(item.id) } }
        }
    }
    if Cart.count == 0 { Text("Your cart is empty.").muted }
}
```

Clicking **Add a pen** changes **0 items, $0.00** to **1 items, $2.50**,
and the `/cart` page lists the pen.

- `store Cart { … }` declares the store and its members.
- `use Cart` in a page says the page reads the store. The page then reads
  members as `Cart.items` and `Cart.count`, and calls actions as
  `Cart.add(…)`.
- Everything that reads `Cart.count`, on any page, updates when it changes.

The checker knows every member and its type. `Cart.cont` is an error
(`T06`, with the nearest member it does have), and `Cart.add("pen")` is an
error because `add` takes an `Item` (`T01`).

## Store members

A store holds the same members a page does: `state`, `persist`, `derived`
values and actions. This fuller `Cart` adds a coupon kept across visits and
an action that raises the quantity of an item already in the cart:

```wf
type Item { id: String, name: String, price: Number, qty: Number = 1 }

store Cart {
    state items: [Item] = []
    persist coupon = ""

    derived count = items.reduce((n, i) => n + i.qty, 0)
    derived subtotal = items.reduce((sum, i) => sum + i.price * i.qty, 0)
    derived discount = if coupon == "SAVE10" { subtotal * 0.1 } else { 0 }
    derived total = subtotal - discount

    action add(item: Item) {
        let found = items.find(i => i.id == item.id)
        if found != null {
            found.qty = found.qty + 1
        } else {
            items.push(item)
        }
    }
    action remove(id: String) { items = items.filter(i => i.id != id) }
    action clear() { items = [] }
}
```

Inside the store you read its members by their bare names: `items`,
`count`. Outside it, a page or component reads them through the store's
name: `Cart.items`, `Cart.count`.

You may declare a store in any `.wf` file the build reads. By convention it
goes in `src/stores/`, and `wf generate store Cart` writes it there. A page
reaches a store by name from anywhere. `use` says which stores a page
reads, for the person reading the code and for the editor; it does not
limit what the page can reach.

## Store scope

A store's *scope* says how long it lives. You set it in the declaration:

```wf
store Cart(scope: .app) { … }        // the default: as long as the page is open
store Session(scope: .session) { … } // this tab's
store Filters(scope: .route) { … }   // this route's
```

| Scope | Lives | Use it for |
|---|---|---|
| `.app` (the default) | as long as the page is open | the cart, the signed-in user |
| `.session` | as long as the tab; what it keeps goes in the tab's own storage, gone when the tab is | state for one tab |
| `.route` | until the route changes | the filters on a list, a wizard's step |

When the route changes, a `.route` store drops what it holds. The next read
builds it again from its initial values.

`store X(eager: true)` builds the store at boot instead of on first read.
Use it for the rare store whose set-up must run whether or not a page reads
it. If a `derived` value of yours has a side effect that used to run when
the page loaded, `eager: true` keeps it running then.

## Actions

A store's actions are its interface: pages change the store by calling
them. An action may be asynchronous — it uses `await` — and an async action
exposes `pending`, which is `true` while a call of it runs:

```wf
store Session {
    state user: Map? = null
    state error = ""
    derived loggedIn = user != null

    action login(email: String, password: String) {
        error = ""
        try {
            let r = await fetch("/api/login", { method: "POST", body: { email: email, password: password } })
            user = r.user
        } catch e {
            error = "Could not sign in: {e.message}"
        }
    }

    action logout() {
        user = null
        navigate("/")
    }
}

page Login(path: "/login", title: "Sign in", description: "Sign in.") {
    use Session
    state email = ""
    state password = ""
    Form {
        on submit { Session.login(email, password) }
        Input(bind: email, label: "Email").email.required
        Input(bind: password, label: "Password").password.required
        Button("Sign in", type: .submit, disabled: Session.login.pending).primary
    }
    if Session.error != "" { Alert(Session.error).danger }
}
```

The **Sign in** button is disabled while `Session.login` runs.

Inside an action, a bare name is the store's own member — `items`, `error`
— unless something nearer declares it. A `let`, a parameter, a `for`
binding, a lambda's parameter or a caught error belongs to the action until
its block ends. So `let items = …` in an action reads and writes that local
variable and leaves the store's `items` alone.

An action may read other stores through their names. Stores are global, so
a store needs no `use`. A `derived` value may call one of the store's own
actions to compute a value, but not one that assigns state: that is `X04`,
because the derived value would change state every time it is read.

## Keeping a value across visits

`persist` declares a member whose value is kept in the browser's storage
and read back on the next visit. It is stored under the store's name and
the member's name, so a page and a store may each persist a `theme`.
`persist` works the same way at the top of a page or a component.

```wf
store Prefs {
    persist scheme = "system"
    persist density = "comfortable"
    persist recent: [String] = []
    action visited(path: String) {
        recent = [path, ...recent.filter(p => p != path)].take(5)
    }
}
```

By default the value goes in `localStorage`, and **a write in another tab
of the same site arrives in this one**. `localStorage` is shared between
tabs, and two tabs that disagree about the theme would be a bug.

### Options

A block after the declaration changes where and how the value is kept:

```wf
type Item { id: String, qty: Number = 1 }

store Cart {
    persist items: [Item] = [] {
        in: .local          // .local (default, across visits) · .session (this tab) · .app (the project's own)
        version: 2          // the version of the shape this build writes
        sync: false         // don't take what another tab wrote
        migrate 1 -> 2 { old.map(i => Item(id: i.id, qty: i.count)) }
    }
}
```

| Option | Meaning |
|---|---|
| `in:` | Where the value lives: `.local` (the default, kept across visits), `.session` (this tab), or `.app` (see [Storage the project provides](#storage-the-project-provides)) |
| `version:` | The version of the value's shape that this build writes |
| `sync:` | `false` stops this page taking a value another tab or window wrote |
| `key:` | Keeps one stored value per key (see [One stored value per instance](#one-stored-value-per-instance)) |
| `migrate n -> m { … }` | Brings a value written at version `n` forward to `m` |

### Versions and migrations

A version is a promise to returning readers. Once you declare a shape with
`version:`, the page brings a value written by an older build forward one
step at a time. Each `migrate n -> n+1` step reads the old value as `old`
and returns the new one. A reader who last visited two deploys ago keeps
their cart instead of finding it empty.

The build checks the chain:

- A gap in the chain is a `P01` warning: an older value would be discarded.
- A step above the declared version is a `P02` warning: it never runs.
- A value written by a *newer* build is left alone, and the page uses the
  initial value. This build cannot know what a later shape means.

The build also remembers the shape of every persisted value, in
`.wf-cache/persist.json`. When a shape changes and `version:` was not
raised, it warns (`D04`): a returning reader's browser still holds the old
shape, which the page would read as the new one.

> [!WARNING]
> A `persist` inside a `store(scope: .route)` does not reset with the route.
> The route change drops the store, and the next read builds it again — from
> storage, so the value comes straight back. The build reports this as
> `P03`. Use `state` for what the route owns, and a wider scope for what
> outlives it.

### One stored value per instance

A `persist` in a component uses one stored value for every place the
component is drawn. Three `Panel`s that each persist `open` share one
value, so opening one opens them all. `key:` names what tells the instances
apart, and each key gets its own stored value:

```wf
component Panel(_ title: String) {
    persist open = false { key: title }
    Button(title) { on click { open = !open } }
    if open { children }
}
```

The build warns (`D03`) about a component that persists a value with no
`key:` and is placed more than once, or in a loop.

### A key that follows state

`key:` is read as state. When what it reads changes, the member takes the
value kept under the new key, and writes go there. A desktop app keeps its
layout per project this way. A store's key may read the store itself, or
another store:

```wf
store Layout {
    state project = "halyard"
    persist side = 240 {
        key: project
        in: .app
    }
}
```

### Storage the project provides

`in: .app` keeps the value somewhere other than the browser's two storages:
a desktop app's store reached through its native bridge, or an extension's
storage. It reads `window.wfStorage.app`, which a script of the project's
under `src/` defines. The object has this shape:
`{ get(name), set(name, text), remove?(name), watch?(name, fn) }`.

- `get` answers at once or with a promise, so an IPC call is fine. Until it
  answers, the value is its declared initial value. A write made before the
  answer is not undone by it, and an answer for a key that has since
  changed is ignored.
- `set` receives the JSON text to keep.
- `watch(name, fn)`, when present, hears another writer, such as another
  window, and returns a function that stops it. `sync: false` turns it off.

If no `window.wfStorage.app` is defined, the value is kept in
`localStorage`, and `wf serve` says so on the console.

This script connects `.app` to Tauri's IPC:

```js
// src/app-store.js — Tauri's IPC, reached when a value is read or written
const call = (command, args) => window.__TAURI__.core.invoke(command, args);
window.wfStorage = {
  app: {
    get: (name) => call("store_get", { name }),
    set: (name, text) => call("store_set", { name, text }),
  },
};
```

See [Desktop apps](../recipes/desktop-apps.md) for the rest of a desktop project.

### Secrets are never persisted

`persist token: Secret = ""` is an error (`T12`). The point of the `Secret`
type is that its value never reaches a place any script can read, and the
browser's storage is such a place. Keep a session in an httpOnly cookie
instead (see [Security](../about/security.md)).

## A connection a store holds

A store may hold a connection — a `socket`, a `stream`, a `channel`, a
`peer` or a `worker` (see [Real-time](../recipes/realtime.md)) — for every page at
once. The connection opens when the store is first read and closes with the
store; a `.route` store's connection closes when its route leaves. The
store's actions and handlers name the connection as it is declared, and a
page reads it through the store:

```wf
store Live {
    state last = ""
    socket feed = ws("wss://example.com/feed") {
        on message(m) { last = m.text }
    }
    action say(text: String) { feed.send({ text: text }) }
}

page Feed(path: "/", title: "Feed", description: "One line for every page.") {
    Heading("Feed").h1
    match Live.feed {
        open { Text(Live.last) }
        else { Text("Reconnecting…") }
    }
    Button("Wave") { on click { Live.say("hello") } }
}
```

## Loading data into a store

A store can own a `resource` (see [Data and APIs](15-data.md)). It is asked
for the first time something reads the store, and lives as long as the
store does — not as long as the page that happened to read it first — so
another page that reads the store finds what has already arrived, and
nothing is asked for again. It is abandoned with the store: a `.route`
store's when its route leaves, to be asked for afresh when the route comes
back.

Inside the store it is `rows`, anywhere else `Deploys.rows`, and its
`state`, `data`, `error`, `refreshing` and `pending` follow it in both. Its
address and options read the store's own state, so an action that moves
one asks again:

```wf
type Deploy { id: String, status: String }

store Deploys {
    state page = 1
    resource rows: [Deploy] = fetch("/api/deploys?page={page}")
    derived failing = (rows.data ?? []).filter(d => d.status == "failed").length
    action next() { page = page + 1 }
    action refresh() { rows.reload() }
}

page DeploysPage(path: "/deploys", title: "Deploys", description: "Every deploy.") {
    Heading("Deploys").h1
    Text("{Deploys.failing} failing")
    match Deploys.rows {
        loading { Spinner() }
        error(e) { Text(e.message) }
        ready(rows) {
            for d in rows by d.id { Text("{d.id}: {d.status}") }
        }
    }
    Button("Refresh", disabled: Deploys.rows.pending) { on click { Deploys.refresh() } }
    Button("Next page") { on click { Deploys.next() } }
}
```

A resource over a service's endpoint is owned the same way —
`resource list = Catalog.items(page: page, paginate: .page)` — and its
`loadMore()` moves the store's `page`. The resource is the store's to
replace: `Deploys.rows = …` is an `X01`, as it is inside the store.

When an action decides what is kept — a reply merged into rows already
held, a list built from several calls — fill the store's state from a
`fetch` instead, and call the action where the data is first wanted:

```wf
store Feed {
    state items: [Map] = []
    action more() {
        let r = await fetch("/api/feed?after={items.length}")
        items = [...items, ...r.items]
    }
}
```

## Stores in the pre-rendered page

A static build pre-renders each page to HTML — the *static paint*. The
static paint seeds each store's state from its initial values, so
`Cart.count` is painted as `0`, and a store seeded with rows paints the
rows. A value an action changes at run time is corrected once the page is
live.

## Looking at one while it runs

`wf serve` puts a **stores** button in the corner of the page. It opens on
what every store that has been built is holding, and a log of every action
that has run: the arguments it was given, and the state before and after
it. Clicking an entry puts the store back to what it held before that
action, which is usually enough to find a bug.

The same tools are on `window.WF`, for a console or a script:

| Call | What it does |
|---|---|
| `WF.storeSnapshot()` | Returns what every built store holds |
| `WF.storeSnapshot("Cart")` | Returns what one store holds |
| `WF.restoreStore("Cart", values)` | Puts a store back to `values` |
| `WF.watchStores(fn)` | Calls `fn` for every action, and returns the function that stops it |

Nothing is recorded while nothing is watching, so a deployed site pays
nothing for any of it. See [Runtime API](../reference/runtime-api.md).

## Testing a store

`wf test` renders a test with the project's stores at hand, and a test's
`data` seeds a store by its name:

```wf
test "the cart shows what is in it"(data: { Cart: { items: [{ id: "a", qty: 2 }] } }) {
    CartSummary()
    expect "2 items"
}
```

A test that clicks a button runs the store's actions in a real browser.
See [Testing](16-testing.md).

## When to use a store

Put a value where the code that reads it can reach it, and no wider:

| The value is… | Keep it in |
|---|---|
| read by more than one page, or must outlive a navigation | a store |
| owned by one screen, and should go when the screen does | `store X(scope: .route)` |
| read by one page and its components | the page's `state`, passed down as props |
| read by one component | the component's own `state` |

## Quick reference

| Syntax | Meaning |
|---|---|
| `store Name { … }` | Declare a store, built on first read |
| `store Name(scope: .route)` | How long it lives: `.app` (the default), `.session` or `.route` |
| `store Name(eager: true)` | Build it at boot |
| `use Name` | Say a page or component reads it |
| `Name.member` · `Name.action(…)` | Read a member, call an action |
| `Name.action.pending` | `true` while an async action runs |
| `persist x = v` | Keep a member across visits |
| `persist x = v { in: …, version: n, sync: false, key: k }` | Where and how it is kept |
| `migrate n -> m { … old … }` | Bring an older stored value forward |

## Next

[Types](12-types.md).
