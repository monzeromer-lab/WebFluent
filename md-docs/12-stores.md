# 9. Stores

A store is state shared by every page and component that uses it: the cart,
the signed-in user, the theme, a cache of what was fetched. It holds `state`,
`derived` values and actions.

**A store is built the first time something reads it.** Declaring one does
nothing; the first `Cart.items`, `Cart.count` or `Cart.add(…)` builds it,
and everything after that is the same instance. So the order stores are
declared in does not matter — a `derived` may read a store declared below
it — and a store nothing reads costs nothing, in time or in bytes.

## Declaring a store

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

Inside the store its members are read bare (`items`, `count`); outside, a
page or component says `use Cart` and reads them as `Cart.items`,
`Cart.count`, and calls `Cart.add(…)`.

A store may be declared in any `.wf` file the build reads — `src/stores/`
by convention, and `wf generate store Cart` puts it there — and is reached
by name from anywhere. `use` says which stores a page reads, for the reader
and for the editor; it does not scope anything.

## How long it lives

```wf
store Cart(scope: .app) { … }        // the default: as long as the page is open
store Session(scope: .session) { … } // this tab's
store Filters(scope: .route) { … }   // this route's
```

`.route` is the natural home for what belongs to one screen — the filters
on a list, a wizard's step. When the route changes, what it holds goes with
it, and the next read builds it again from its initial values. `.session`
changes where what it keeps is written: the tab's own storage, gone when
the tab is.

`store X(eager: true)` builds it at boot instead, for the rare store whose
set-up has to happen whether or not a page reads it. If a `derived` of
yours has a side effect that used to run when the page loaded, this is the
switch that keeps it running then.

## Using a store

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

Everything that reads `Cart.count` — on any page — updates when it changes.
The checker knows every member and its type: `Cart.cont` is an error (`T06`),
and `Cart.add("pen")` is an error because `add` takes an `Item` (`T01`).

## Actions

A store's actions are its API. They may be asynchronous, and an async action
exposes `pending`:

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

A derived value may call one of the store's own actions, and an action may
read other stores through their names — stores are global, so no `use` is
needed inside a store.

## Keeping a value across visits

`persist` keeps a member in the browser's storage, keyed by the store's
name. It works the same way at the top of a page or a component.

```wf
store Prefs {
    persist theme = "system"
    persist density = "comfortable"
    persist recent: [String] = []
    action visited(path: String) {
        recent = [path, ...recent.filter(p => p != path)].take(5)
    }
}
```

By default the value goes in `localStorage`, and a write in **another tab
of the same site arrives here** — `localStorage` is shared between them, so
two tabs that disagree about the theme are a bug, not a feature.

The block says otherwise:

```wf
store Cart {
    persist items: [Item] = [] {
        in: .local          // .local (default, across visits) · .session (this tab)
        version: 2          // the version of the shape this build writes
        sync: false         // don't take what another tab wrote
        migrate 1 -> 2 { old.map(i => Item(id: i.id, qty: i.count)) }
    }
}
```

**A version is a promise.** Once a shape is declared with a `version:`, a
value written by an older build is brought forward one step at a time —
each `migrate n -> n+1` reads the old value as `old` and returns the new
one — so a reader who last visited two deploys ago keeps their cart instead
of finding it empty. A gap in the chain is a `P01` warning, and a step
above the declared version a `P02`: both say the value would be discarded.

A value written by a *newer* build is left alone and the initial value
used: this build cannot know what a later shape means.

**A `.route` store that persists is a contradiction.** The route change
drops the store, and the next read builds it again — from storage, so the
value comes straight back. The build says so (`P03`). What the route owns
is `state`; what outlives it wants a wider scope.

**A `Secret` is never written down.** `persist token: Secret = ""` is a
`T12` — the whole point of the type is that it does not reach a place any
script can read.

## Loading data into a store

A store can own a `resource` too, or fill its state from a fetch in an
action called at the top of a page:

```wf
store Deploys {
    state rows: [Map] = []
    state loaded = false
    action load() {
        if loaded { return }
        let r = await fetch("/api/deploys")
        rows = r.rows
        loaded = true
    }
}

page DeploysPage(path: "/deploys", title: "Deploys", description: "Every deploy.") {
    use Deploys
    Deploys.load()
    if !Deploys.loaded { Spinner }
    for d in Deploys.rows by d.id { Text("{d.id}: {d.status}") }
}
```

A call at the top of a page's body runs once when the page renders — set-up
code, in the right place.

## Stores in the static paint

A pre-rendered page seeds a store's state from its initial values, so
`Cart.count` paints as `0` and a store seeded with rows paints the rows. A
value an action changes at run time is corrected once the page is live.

## Looking at one while it runs

`wf serve` puts a **stores** button in the corner of the page. It opens on
what every store that has been built is holding, and a log of every action
that has run — the arguments it was given, and the state on each side of
it. Clicking an entry puts the store back to what it held before that
action, which is as much time travel as a bug usually needs.

The same surface is on `window.WF` for a console or a script:
`WF.storeSnapshot()` for the tree, `WF.storeSnapshot("Cart")` for one,
`WF.restoreStore("Cart", values)` to put one back, and
`WF.watchStores(fn)` to be told about every action, which returns the
function that stops it. Nothing is recorded while nothing is watching, so a
deployed site pays nothing for any of it.

## Testing one

`wf test` renders a declaration with the project's stores at hand, and a
test's `data` seeds them by name:

```wf
test "the cart shows what is in it"(data: { Cart: { items: [{ id: "a", qty: 2 }] } }) {
    CartSummary()
    expect "2 items"
}
```

## When to use a store, when a page's state

- Read by more than one page, or must outlive a navigation: a store.
- Belongs to one screen and should go when it does: `store X(scope: .route)`.
- Read by one page and its components: page `state`, passed down as props.
- Read by one component: its own `state`.

## Next

[Types](10-types.md).
