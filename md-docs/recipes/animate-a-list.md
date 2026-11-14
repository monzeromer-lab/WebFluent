---
kind: "recipe"
route: "/recipes/animate-a-list"
group: "motion"
description: "Animate a list's items as they arrive, move and leave: a keyed for … by, with animate:, exit: and stagger: on each item's element."
outputs: ["website", "app"]
from: ["/docs/guide/motion#animate-a-list"]
---
# Animate a list

Make a list fill in one item after another, slide its items to their new
places when it is reordered, and fade an item out as it is removed. The
list is a keyed `for … by`, and each item's element carries `animate:`,
`exit:` and `stagger:`:

**Needs:** nothing besides the page. It claims `/` and the name `Tasks`,
as two of the starter's pages do: paste it into a new project, or replace
`src/pages/Home.wf` and remove `src/pages/Tasks.wf`.

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

## How it works

Each change to `items` plays a different motion:

- **An item arrives.** It plays its `animate:`. `stagger: "40ms"` delays
  each item's enter by its index × 40ms, so a list fills in one item after
  another.
- **An item moves** — a sort, a reverse, an insert above it. It slides to
  its new place. This is a *FLIP* transform: the runtime measures where the
  node was and where it is, and animates between them. It works because
  `by item.id` keeps the item's node alive
  ([Control flow](../learn/09-control-flow.md)).
- **An item leaves.** It plays its `exit:` first, and the runtime removes
  the node when it ends.

A `for` with no `by`, or a key that changes, makes a new node for the item,
so it fades out and in again instead of moving. Key the list by what
identifies the item.

A reader who asked for reduced motion sees items appear, move and go at
once: no animation, no FLIP and no wait before removal
([Respect reduced motion](motion-defaults.md#respect-reduced-motion)).
