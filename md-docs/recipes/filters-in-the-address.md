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
page Products(path: "/products", title: "Products", description: "Everything we sell.") {
    derived sort = query.sort ?? "name"
    Heading("Products").h1
    Row(gap: .sm) {
        Link("By name", to: "/products?sort=name")
        Link("By price", to: "/products?sort=price")
    }
    Text("Sorted by {sort}")
}
```

## How it works

- [`query`](../learn/05-pages-and-routing.md#query-strings-and-fragments)
  is the address's query string as a map: `query.sort` is `"price"` for
  `?sort=price`, and null when the address has no `sort`.
- `sort` is a [`derived`](../learn/07-state-and-reactivity.md#derived)
  value, so `??` gives it a default and it follows the address.
- Each choice is a `Link` to the same page with another query. A query
  string is not part of the route, so `/products?sort=price` is still the
  `Products` page; only `sort` changes.

To change the query from an action instead of a link, call
`navigate("/products?sort=price")`.
