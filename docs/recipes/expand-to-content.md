# Open a box to the height of its content

> Open a box to the height of its content, which CSS cannot animate to: .expand measures height: auto, animates to it, then lets the box grow.

The page: https://webfluent.monzeromer.dev/recipes/expand-to-content

CSS cannot animate to `height: auto`, so a box of details that should slide
open to whatever height its text needs takes `.expand`. Press **Details**
and the box opens; press it again and it closes:

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

## How it works

- `show open` toggles whether the box is visible, and the box animates as
  it appears and goes.
- `animate: .expand` measures the height `auto` would give, animates to
  that number, then hands the height back, so the box keeps growing with
  its content.
- `.expand` on the way in is `.collapse` on the way out, so the box needs
  no `exit:`.

A reader who asked for reduced motion sees the box open at once
([Respect reduced motion](https://webfluent.monzeromer.dev/recipes/motion-defaults.md#respect-reduced-motion)).
