# Load more as the reader asks

> Add the next page of results below the ones shown when the reader asks: an api endpoint called with paginate: .page and a Load more button.

The page: https://webfluent.monzeromer.dev/recipes/load-more

A **Load more** button keeps what the reader has already seen and adds the
next page below it. To build one, call an endpoint with `paginate: .page`:

**Needs:** a server that answers `/api/items?page=…` with a list of items.

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

The page shows the first page of items and a **Load more** button.
Each press adds the next page beneath what is there, without losing the
reader's place, and the button goes away once there is nothing more.

## How it works

- `Catalog` [describes the service](https://webfluent.monzeromer.dev/learn/15-data.md#describe-a-service-with-api):
  its `items` endpoint takes a `page` and returns a list of `Item`.
- The resource calls it with `paginate: .page`, which gives it `.items`,
  `.loadMore()` and `.hasMore`
  ([Load results a page at a time](https://webfluent.monzeromer.dev/learn/15-data.md#load-results-a-page-at-a-time)).
- `list.items` is every item loaded so far, so the `for` lists them all and
  `item.name` is checked against `Item`.
- `list.loadMore()` moves `page` on by one and asks for the next page, as a
  refresh: the items already shown stay while it runs.
- `list.pending` is true while a request runs, so the button cannot ask
  twice. `list.hasMore` turns false when a page comes back empty, and the
  button goes.
