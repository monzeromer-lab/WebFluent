# Page through results

> Show a long list one page at a time: keep the page number in state, splice it into the request's address, and step it with buttons.

The page: https://webfluent.monzeromer.dev/recipes/page-through-results

A list of thousands of rows is too long to ask for at once. To show it a
page at a time with **Previous** and **Next** buttons, keep the page number
in state and splice it into the address:

**Needs:** a server that answers `/api/rows?page=…&per=…` with a list of
rows. It claims `/`: paste it into a new project, or replace the starter's
`src/pages/Home.wf` with it.

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
  [resource](https://webfluent.monzeromer.dev/learn/15-data.md#show-a-request-with-resource) splices both
  into its address.
- The server answers with the page's `items` and the number of `pages`.
  The `ready` arm lists the items and draws the buttons.
- **Previous** and **Next** change `page`. A change of `page` changes the
  address, so the resource asks again
  ([Fetch again when state changes](https://webfluent.monzeromer.dev/learn/15-data.md#fetch-again-when-state-changes)).
- Each button is disabled at its end of the list, so `page` stays between
  1 and `r.pages`.

To add each page below the ones already shown instead of replacing them,
see [Load more as the reader asks](https://webfluent.monzeromer.dev/recipes/load-more.md).
