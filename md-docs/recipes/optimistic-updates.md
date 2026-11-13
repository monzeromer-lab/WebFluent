---
kind: "recipe"
route: "/recipes/optimistic-updates"
group: "data"
description: "To show a change before the server agrees, keep the old value and put it back if the request fails:"
from: ["/docs/guide/recipes#update-at-once-and-roll-back-on-failure"]
---
# Update at once, and roll back on failure

To show a change before the server agrees, keep the old value and put it
back if the request fails:

```wf
type Item { id: String, liked: Bool }

page Likes(path: "/", title: "Likes", description: "A like that shows at once.") {
    state items: [Item] = [Item(id: "a", liked: false)]
    state error = ""
    action like(id: String) {
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

`optimistic(holder, change)` does the same with less code inside an
action; see [Fetching data](../learn/15-data.md#show-a-change-before-the-server-agrees).
