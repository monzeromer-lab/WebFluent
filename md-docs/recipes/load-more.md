---
kind: "recipe"
route: "/recipes/load-more"
group: "data"
description: "To add the next page of results below the ones shown, call an endpoint with `paginate: .page`:"
from: ["/docs/guide/recipes#load-more"]
---
# Load more as the reader asks

To add the next page of results below the ones shown, call an endpoint with
`paginate: .page`:

```wf
type Item { id: String, name: String }

api Catalog(base: "/api") {
    get items(page: Number = 1) -> [Item]
}

page Items(path: "/items", title: "Items", description: "Everything, a page at a time.") {
    state page = 1
    resource list = Catalog.items(page: page, paginate: .page)
    Heading("Items").h1
    for item in list.items by item.id { Text(item.name) }
    if list.hasMore {
        Button("Load more", disabled: list.state == "loading") { on click { list.loadMore() } }
    }
}
```

`list.items` is every item loaded so far, and `list.loadMore()` asks for the
next page.
