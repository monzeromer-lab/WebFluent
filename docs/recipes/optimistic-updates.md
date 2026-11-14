# Update at once, and roll back on failure

> Show a change before the server agrees: update the state at once, send the request, and put the old value back if it fails.

The page: https://webfluent.monzeromer.dev/recipes/optimistic-updates

A like that waits for the server feels slow. To show the change at once,
update the state before the request, keep the old value, and put it back if
the request fails:

**Needs:** a server that answers `POST /api/like/<id>`. It claims `/`:
paste it into a new project, or replace the starter's `src/pages/Home.wf`
with it.

```wf
type Item { id: String, liked: Bool }

page Likes(path: "/", title: "Likes", description: "A like that shows at once.") {
    state items: [Item] = [Item(id: "a", liked: false)]
    state error = ""
    action like(id: String) {
        error = ""
        let before = items
        items = items.map(i => if i.id == id { Item(id: i.id, liked: !i.liked) } else { i })
        try {
            await fetch("/api/like/{id}", { method: "POST" })
        } catch e {
            items = before
            error = "Could not save. Try again."
        }
    }
    Heading("Likes").h1
    for i in items by i.id {
        Button(if i.liked { "♥" } else { "♡" }, aria-label: "Like").sm { on click { like(i.id) } }
    }
    if error != "" { Toast(error).danger }
}
```

## How it works

- `let before = items` keeps the list as it was. A
  [`let`](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#actions-and-locals) is local
  to the action.
- The next line gives `items` a new list with the one item flipped. The
  button repaints at once, before any request leaves.
- The request is [awaited](https://webfluent.monzeromer.dev/learn/15-data.md#fetch-on-demand-with-await-fetch)
  inside `try`. If it fails, the `catch` puts `before` back and sets
  `error`, and the `Toast` tells the reader.
- The action clears `error` first. A `Toast` shows once, when its branch is
  drawn, so without the reset a second failure would leave `error` as it
  was and show nothing.

## Variations

`optimistic(holder, change)` does the same with less code inside an
action. It applies the change at once, and if anything later in the action
throws, the change is taken back
([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md#show-a-change-before-the-server-agrees)):

```wf
action like(id: String) {
    optimistic(items, list => list.map(i => if i.id == id { Item(id: i.id, liked: !i.liked) } else { i }))
    await fetch("/api/like/{id}", { method: "POST" })
}
```
