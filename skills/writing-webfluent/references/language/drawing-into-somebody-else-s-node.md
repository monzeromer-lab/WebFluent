### Drawing into somebody else's node

```wf
for row, i in Completions.rows by i {
    Portal(into: editorHost(row.id), data-popup: "row") {   // a node CodeMirror made
        Text(row.label) {
            on mousedown(e) { e.preventDefault() }          // the editor keeps the focus
            on click { Completions.pick(i) }
        }
    }
}
```

The library keeps the node's place, keys and focus; the page draws what is
in it. `into:` takes an element (what a script returns, JSDoc
`HTMLElement|null`), a `ref:` handle (even one written further down), or
`null`, which keeps the block aside with its nodes and state until a node is
named again; it is read again whenever what it reads changes and once more
when the page is drawn and in the document, and the same box moves. The block is drawn in the scope the portal is written in — its
handlers, `emit`, state, `mount:` and `cleanup:` are that page's or
component's — and a `mount:` inside runs once the block is in place. The
box goes when its page, branch or list item does; the node stays.
Events bubble from the node it is drawn in, not from where it is written,
and `show` does not hide it (its block is not in `show`'s place): use `if`,
or `into: null`.

