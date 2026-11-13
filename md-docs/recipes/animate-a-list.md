---
kind: "recipe"
route: "/recipes/animate-a-list"
group: "look"
description: "To animate the items of a list as they are added, removed and moved, use a keyed `for … by` and give the item's element `animate:`, `exit:` and `stagger:`:"
from: ["/docs/guide/motion#animate-a-list"]
---
# Animate a list

To animate the items of a list as they are added, removed and moved, use a
keyed `for … by` and give the item's element `animate:`, `exit:` and
`stagger:`:

```wf
type Item { id: Number, text: String }

page Tasks(path: "/", title: "Tasks", description: "A list that animates.") {
    state items: [Item] = [Item(id: 1, text: "First"), Item(id: 2, text: "Second")]
    state next = 3
    action add() { items.push(Item(id: next, text: "Item {next}"))  next = next + 1 }
    action shuffle() { items = items.reverse() }
    Heading("Tasks").h1
    Row(gap: .sm) {
        Button("Add") { on click { add() } }
        Button("Reverse") { on click { shuffle() } }
    }
    for item in items by item.id {
        Card(animate: .fadeIn, exit: .fadeOut, stagger: "40ms") {
            Row(justify: .between) {
                Text(item.text)
                Button("Remove").sm { on click { items = items.filter(i => i.id != item.id) } }
            }
        }
    }
}
```

What each change does:

- **An item arrives.** It plays its `animate:`. `stagger: "40ms"` delays
  each item's enter by its index × 40ms, so a list fills in one item after
  another.
- **An item moves** — a sort, a reverse, an insert above it. It slides to
  its new place. This is a *FLIP* transform: the runtime measures where the
  node was and where it is, and animates between them. It works because
  `by` keeps the item's node alive.
- **An item leaves.** It plays its `exit:` first.
