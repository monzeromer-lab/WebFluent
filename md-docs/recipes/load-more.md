---
kind: "recipe"
route: "/recipes/load-more"
group: "data"
description: "Add the next page of results below the ones shown when the reader asks: an api endpoint called with paginate: .page and a Load more button."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#load-more"]
---
# Load more as the reader asks

A **Load more** button keeps what the reader has already seen and adds the
next page below it. To build one, call an endpoint with `paginate: .page`:

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
        Button("Load more", disabled: list.pending) { on click { list.loadMore() } }
    }
}
```

## How it works

- `Catalog` [describes the service](../learn/15-data.md#describe-a-service-with-api):
  its `items` endpoint takes a `page` and returns a list of `Item`.
- The resource calls it with `paginate: .page`, which gives it `.items`,
  `.loadMore()` and `.hasMore`
  ([Load results a page at a time](../learn/15-data.md#load-results-a-page-at-a-time)).
- `list.items` is every item loaded so far, so the `for` lists them all and
  `item.name` is checked against `Item`.
- `list.loadMore()` moves `page` on by one and asks for the next page, as a
  refresh: the items already shown stay while it runs.
- `list.pending` is true while a request runs, so the button cannot ask
  twice. `list.hasMore` turns false when a page comes back empty, and the
  button goes.
