### Stores (Shared State)

A store is **built the first time something reads it**. Declaring one does
nothing, so the order stores are declared in cannot matter — a `derived`
may read a store declared below it — and a store nothing reads costs
nothing, in time or in bytes.

```wf
store Cart(scope: .app) { … }         // the default: as long as the page is open
store Session(scope: .session) { … }  // this tab's; what it keeps goes in the tab's storage
store Filters(scope: .route) { … }    // this route's; dropped when the route changes
store Boot(eager: true) { … }         // built at boot, for set-up that must run
```

```wf
type Product { name: String, price: Number }

store CartStore {
    state items: [Product] = []
    state total = 0

    derived count = items.length

    action add(product: Product) {
        items = items.concat([product])
        total = total + product.price
    }

    action clear() {
        items = []
        total = 0
    }
}

// Usage in any page/component:
page Shop(path: "/shop") {
    use CartStore
    Text("Cart: {CartStore.count} items")
    Button("Clear") { on click { CartStore.clear() } }
}
```

A store may own a `resource`: asked for when the store is built, kept for
as long as the store lives (not the page that first read it), and
abandoned with it. Its address reads the store's state; its members —
`value`, `isLoading`, `isRefreshing`, `error`, … — are signals, read as
`rows.value` inside the store and `Feed.rows.isLoaded` outside it, and
`match Feed.rows { .loading … .failed(e) … .loaded(rows) … }` draws it.

```wf
store Feed {
    state page = 1
    resource rows: [Row] = http.get("/api/rows?page={page}")
    derived count = rows.value?.length ?? 0
    action next() { page = page + 1 }
}
```

<!-- core -->

