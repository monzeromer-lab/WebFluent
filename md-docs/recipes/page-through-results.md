---
kind: "recipe"
route: "/recipes/page-through-results"
group: "data"
description: "To show a long list a page at a time, keep the page number in state and splice it into the address:"
from: ["/docs/guide/recipes#page-through-results"]
---
# Page through results

To show a long list a page at a time, keep the page number in state and
splice it into the address:

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

A change of `page` changes the address, so the resource asks again.
