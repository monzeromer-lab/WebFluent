---
kind: "recipe"
route: "/recipes/filters-in-the-address"
group: "interaction"
description: "Keep a list's filter or sort in the query string, so it survives a reload, a shared link and the back button."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#keep-filters-in-the-address"]
---
# Keep filters in the address

A filter held in a `state` is lost on reload and missing from a link the
reader shares. To make it survive a reload, a shared link and the back
button, read it from `query` instead:

```wf
const PRODUCTS = [
    { name: "Kettle", price: 39 },
    { name: "Apron", price: 18 },
    { name: "Mug", price: 9 },
]

page Products(path: "/products", title: "Products", description: "Everything we sell.") {
    derived sort = query.sort ?? "name"
    derived shown = if sort == "price" { PRODUCTS.sortBy(p => p.price) } else { PRODUCTS.sortBy(p => p.name) }
    Heading("Products").h1
    Row(gap: .sm) {
        Link("By name", to: "/products?sort=name")
        Link("By price", to: "/products?sort=price")
    }
    for p in shown by p.name {
        Text("{p.name}: ${p.price}")
    }
}
```

Choose **By price** and the list reorders to Mug, Apron, Kettle, and the
address becomes `/products?sort=price`. Reload the page or share the link,
and it opens sorted by price.

## How it works

- [`query`](../learn/05-pages-and-routing.md#query-strings-and-fragments)
  is the address's query string as a map: `query.sort` is `"price"` for
  `?sort=price`, and null when the address has no `sort`.
- `sort` is a [`derived`](../learn/07-state-and-reactivity.md#derived)
  value, so `??` gives it a default and it follows the address. `shown`
  is derived from `sort`, so the list reorders whenever the address does.
- Each choice is a `Link` to the same page with another query. A query
  string is not part of the route, so `/products?sort=price` is still the
  `Products` page; only `sort` changes.

To change the query from an action instead of a link, call
`navigate("/products?sort=price")`.
