---
kind: "recipe"
route: "/recipes/hand-an-element-to-a-script"
group: "interop"
description: "To give an element to a library and take it back when the element goes, use the three lifetime props."
from: ["/docs/guide/interop#hand-an-element-to-a-script-with-mount-update-and-cleanup"]
---
# Hand an element to a script

To give an element to a library and take it back when the element goes,
use the three lifetime props. Every element and every component call takes
them:

- `mount: (node) => …` runs each time the element is made: the first
  paint, a route change, an `if` branch arriving, a list item added. What
  it returns is the *handle* the other two receive.
- `update: (handle) => …` runs again whenever the state it reads changes.
- `cleanup: (handle) => …` runs when the element leaves (its page, its
  branch or its list item), so nothing a library made outlives it.

On a component call, the three belong to the component's root element,
unless the component declares a prop of that name itself. A `mount:` that
throws removes itself and logs the error; the page stays up.

To get a bare node for a library, use `Host`. It has the same lifetime:
`Host(tag: "canvas", …)` makes the element the library wants. It makes a
`div` by default, or a `span`, `canvas`, `svg`, `section`, `figure`, `pre`,
`p`, `ul` or `table`.
