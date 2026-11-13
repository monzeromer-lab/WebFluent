---
kind: "recipe"
route: "/recipes/filters-in-the-address"
group: "interaction"
description: "To make a filter survive a reload, a shared link and the back button, read it from `query` instead of holding it in a `state`:"
from: ["/docs/guide/recipes#keep-filters-in-the-address"]
---
# Keep filters in the address

To make a filter survive a reload, a shared link and the back button, read
it from `query` instead of holding it in a `state`:

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
