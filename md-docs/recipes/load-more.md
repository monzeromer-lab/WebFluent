---
kind: "recipe"
route: "/recipes/load-more"
group: "data"
description: "Add the next page of results below the ones shown when the reader asks: an api endpoint that says how it pages, and a Load more button."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#load-more"]
---
# Load more as the reader asks

A **Load more** button keeps what the reader has already seen and adds the
next page below it. To build one, say on the endpoint how it pages:

**Needs:** a server that answers `/api/items?page=…` with a list of items.

```wf
type Item { id: String, name: String }

api Catalog(base: "/api") {
    get items(page: Number = 1) at "items" -> [Item]!
        pages .number(param: page)
}

page Items(path: "/items", title: "Items", description: "Everything, a page at a time.") {
    resource list = Catalog.items()
    Heading("Items").h1
    for item in list.items by item.id { Text(item.name) }
    if list.hasMore {
        Button("Load more", loading: list.pending) { on click { list.loadMore() } }
    }
}
```

The page shows the first page of items and a **Load more** button.
Each press adds the next page beneath what is there, without losing the
reader's place, and the button goes away once there is nothing more.

## How it works

- `Catalog` [describes the service](../learn/15-data.md#describe-a-service-with-api):
  its `items` endpoint takes a `page` and returns a list of `Item`, and
  `pages .number(param: page)` says it pages by that number
  ([Load results a page at a time](../learn/15-data.md#load-results-a-page-at-a-time)).
- `list.items` is every item loaded so far, so the `for` lists them all and
  `item.name` is checked against `Item`.
- `list.loadMore()` asks for the next page, as a refresh: the items already
  shown stay while it runs. The read keeps the page number; nothing on the
  page holds it.
- `list.pending` is true while a request runs, so the button shows it is
  busy and cannot ask twice. `list.hasMore` turns false when a page comes
  back empty, and the button goes.
- A server that pages by cursor, or by a `Link` header, says so instead:
  `pages .cursor(param: after, next: p => p.nextCursor, items: p => p.items)`
  (a cursor comes with a page that is a record, so it says where the items
  are), or `pages .link`.
