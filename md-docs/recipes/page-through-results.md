---
kind: "recipe"
route: "/recipes/page-through-results"
group: "data"
description: "Show a long list one page at a time: keep the page number in state, splice it into the request's address, and step it with buttons."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#page-through-results"]
---
# Page through results

A list of thousands of rows is too long to ask for at once. To show it a
page at a time with **Previous** and **Next** buttons, keep the page number
in state and splice it into the address:

```wf
page Rows(path: "/", title: "Rows", description: "Every row, twenty at a time.") {
    state page = 1
    state perPage = 20
    resource rows = fetch("/api/rows?page={page}&per={perPage}")
    Heading("Rows").h1
    match rows {
        loading { Spinner }
        error(e) { Text(e.message).danger }
        ready(r) {
            for row in r.items by row.id { Text(row.name) }
            Row(gap: .sm, align: .center) {
                Button("Previous", disabled: page == 1) { on click { page = page - 1 } }
                Text("Page {page} of {r.pages}")
                Button("Next", disabled: page >= r.pages) { on click { page = page + 1 } }
            }
        }
    }
}
```

## How it works

- `page` and `perPage` are states, and the
  [resource](../learn/15-data.md#show-a-request-with-resource) splices both
  into its address.
- The server answers with the page's `items` and the number of `pages`.
  The `ready` arm lists the items and draws the buttons.
- **Previous** and **Next** change `page`. A change of `page` changes the
  address, so the resource asks again
  ([Fetch again when state changes](../learn/15-data.md#fetch-again-when-state-changes)).
- Each button is disabled at its end of the list, so `page` stays between
  1 and `r.pages`.

To add each page below the ones already shown instead of replacing them,
see [Load more as the reader asks](load-more.md).
