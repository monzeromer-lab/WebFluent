# 9. Stores

A store is state shared by every page and component that uses it: the cart,
the signed-in user, the theme, a cache of what was fetched. It holds `state`,
`derived` values and actions, and it lives for the life of the app.

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

## Persistence

`persist` in a store keeps a member across visits, keyed by the store's name:

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

## When to use a store, when a page's state

- Read by more than one page, or must outlive a navigation: a store.
- Read by one page and its components: page `state`, passed down as props.
- Read by one component: its own `state`.

## Next

[Types](10-types.md).
