---
kind: "recipe"
route: "/recipes/debounce-a-search"
group: "interaction"
description: "To wait until the reader stops typing before you search, keep what they typed apart from what you search for:"
from: ["/docs/guide/recipes#debounce-a-search"]
---
# Debounce a search

To wait until the reader stops typing before you search, keep what they
typed apart from what you search for:

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

The effect runs again on every keystroke, and each run schedules a check.
Only the check whose value is still current writes `q`, and the resource
follows `q`. The timer is the browser's `setTimeout`, because `after(ms)`
is a declaration with a body, not a statement an effect can hold.
