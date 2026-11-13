---
kind: "recipe"
route: "/recipes/debounce-a-search"
group: "interaction"
description: "Search as the reader types, but only once they pause: keep what they typed apart from what you search for, and let a timer move one to the other."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#debounce-a-search"]
---
# Debounce a search

A search box that asks the server on every key sends a request per letter.
To wait until the reader stops typing, keep what they typed apart from
what you search for, and copy one to the other only after a pause:

```wf
page Search(path: "/", title: "Search", description: "Search as you type, without a request per key.") {
    state typed = ""
    state q = ""
    action commit(value: String) { if value == typed { q = value } }
    effect {
        let pending = typed
        setTimeout(() => commit(pending), 300)
    }
    resource hits = fetch("/api/search?q={q}")
    Heading("Search").h1
    Input(bind: typed, placeholder: "Search", aria-label: "Search")
    match hits {
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} results for {q}") }
    }
}
```

## How it works

- `typed` follows the input on every key, through `bind:`. `q` is what the
  page searches for, and only `commit` writes it.
- The [effect](../learn/07-state-and-reactivity.md#effect) reads `typed`,
  so it runs again on every keystroke. Each run keeps the value it saw in
  `pending` and schedules a check 300 ms later.
- When a check runs, `commit` writes `q` only if `pending` is still what the
  reader has typed. A check from an earlier key finds `typed` has moved on
  and does nothing, so only the last pause gets through.
- The [resource](../learn/15-data.md#show-a-request-with-resource) splices
  `q` into its address, so it asks again each time `q` changes — once per
  pause, not once per key.

The timer is the browser's `setTimeout`, because
[`after(ms)`](../learn/07-state-and-reactivity.md#timers) is a declaration
with a body, not a statement an effect can hold.
