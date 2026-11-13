---
kind: "recipe"
route: "/recipes/expand-to-content"
group: "look"
description: "CSS cannot animate to `height: auto`."
from: ["/docs/guide/motion#open-a-box-to-the-height-of-its-content"]
---
# Open a box to the height of its content

CSS cannot animate to `height: auto`. To open a box to whatever height its
content needs, use `.expand`: it measures the height `auto` would give,
animates to that number, then hands the height back so the box keeps
growing with its content:

```wf
page Details(path: "/", title: "Details", description: "A box that opens smoothly.") {
    state open = false
    Heading("Details").h1
    Button("Details") { on click { open = !open } }
    show open {
        Stack(animate: .expand) { Text("As much text as there is.") }
    }
}
```

`.expand` on the way in is `.collapse` on the way out, so the box needs no
`exit:`.
