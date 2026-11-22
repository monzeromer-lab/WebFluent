### Attributes

Any named argument a built-in does not declare is written to its root
element as an HTML attribute, with a warning; `aria-*`, `data-*` and the
global attributes (`id`, `role`, `tabindex`, `title`, `hidden`, `lang`,
`dir`) are declared families and draw none:

```wf
Button("Errors", aria-pressed: showErrors, data-tone: "danger") { on click { showErrors = !showErrors } }
Table.Row(aria-selected: isSelected) { Table.Cell("…") }
Badge("Ready", data-tone: "success")
```

A value that reads state follows it. `aria-*` keeps a `false` value as the
string `"false"` (a real ARIA state); any other attribute given `false` is
omitted.

Every built-in takes them, and its `on` handlers, on the element it draws:
its root (a `VirtualList`'s scrolling box, a `Splitter`'s handle, a `Tabs`'
or a `Carousel`'s box, a `Sidebar`'s panel, a `Breadcrumb`'s `<nav>`, a
chart's `<figure>`, a `Modal`'s `<dialog>`, the `<img>` of a `<picture>`)
or, for a control drawn in a box (`Switch`, `Checkbox`, `Radio`, `Slider`,
`DatePicker`, `FileUpload`), its input — `hidden`, `lang` and `dir` stay on
its box. A `role:` or `aria-label:` written on it takes the place of its
own (a `Splitter`'s `separator`, a `Switch`'s `switch`, a `Breadcrumb`'s
name); a `Sidebar`'s `id:` is the one its toggle points at. The pre-render
and the template engine write the same, as far as the build knows them:

```wf
VirtualList(rowHeight: 24, role: "listbox", tabindex: "0", aria-activedescendant: "ref-{current}") {
    on keydown(e) { if e.key == "ArrowDown" { current = current + 1 } }
    for r in refs by r.id { Text(r.id, id: "ref-{r.id}", role: "option") }
}
```

`class:` does not replace the element's classes, it adds to them.
`Card(class: "feature wide")` renders `class="wf-card feature wide"`, and a
value that reads state is followed — the classes it named last time come
off, the ones it names now go on, and a class a script added is left alone.
It is how an element picks up a rule from the project's own stylesheets (see
[Stylesheets](#stylesheets)). A map turns each class on with its own
condition, and a list joins strings and maps, skipping `null`:

```wf
Text(todo.title, class: { "is-done": todo.done, "is-urgent": todo.priority > 2 })
Card(class: ["feature", tone, if open { "is-open" }])
```

