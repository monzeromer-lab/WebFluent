# Debounce a search

> Search as the reader types, but only once they pause: on input(debounce: 300.ms) moves what was typed to what is searched for.

The page: https://webfluent.monzeromer.dev/recipes/debounce-a-search

A search box that asks the server on every key sends a request per letter.
To wait until the reader stops typing, keep what they typed apart from
what you search for, and copy one to the other once they pause:

**Needs:** a server that answers `/api/search?q=…` with a list of results.
It claims `/`: paste it into a new project, or replace the starter's
`src/pages/Home.wf` with it.

```wf
page Search(path: "/", title: "Search", description: "Search as you type, without a request per key.") {
    state typed = ""
    state q = ""
    resource hits = fetch("/api/search?q={q}")
    Heading("Search").h1
    Input(bind: typed, placeholder: "Search", aria-label: "Search") {
        on input(debounce: 300.ms) { q = typed }
    }
    if q != "" {
        match hits {
            loading { Spinner.sm }
            error(e) { Text(e.message).danger }
            ready(list) { Text("{list.length} results for {q}") }
        }
    }
}
```

## How it works

- `typed` follows the input on every key, through `bind:`. `q` is what the
  page searches for, and only the handler writes it.
- `debounce: 300.ms` runs the handler once the `input` events have stopped
  coming for 300 ms, so a word typed quickly is one run, with the last of
  them. A wait still pending when the page leaves is dropped.
- The [resource](https://webfluent.monzeromer.dev/learn/15-data.md#show-a-request-with-resource) splices
  `q` into its address, so it asks again each time `q` changes — once per
  pause, not once per key. It also asks once as the page opens, for
  `/api/search?q=`, before anything is typed; the `if q != ""` shows
  nothing of that answer, so the server only has to answer it quickly
  (an empty list will do).

For something that should run while the events keep coming, only less often
— a scroll position, a window being resized — write `throttle:` instead:
`on scroll(e, throttle: 100.ms) { … }` runs at once, then at most once
every 100 ms, and once more where the scrolling stopped. Both are in
[Events](https://webfluent.monzeromer.dev/learn/08-events.md#how-often-a-handler-runs).
