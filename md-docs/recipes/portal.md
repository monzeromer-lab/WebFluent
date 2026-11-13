---
kind: "recipe"
route: "/recipes/portal"
group: "interop"
description: "Some libraries make a node and leave it empty for the page to fill: an editor's tooltip, a map's popup, a chart's legend."
from: ["/docs/guide/interop#draw-into-a-node-a-script-made-with-portal"]
---
# Draw into a node a script made

Some libraries make a node and leave it empty for the page to fill: an
editor's tooltip, a map's popup, a chart's legend. To fill it, use
`Portal(into: node) { … }`. The library keeps the node's place, its keys
and its focus; the page draws what is in it.

This script starts a CodeMirror editor, loaded from `meta.scripts` as `CM`,
with a hover tooltip whose words the page draws:

```js
// src/editor.js
// CodeMirror places each tooltip and keeps its keys; the node it makes is
// left empty, and the page draws what is in it.
const tooltips = new Map();
let made = 0;

/**
 * The node CodeMirror made for a tooltip, or null once it has gone.
 * @param {string} id
 * @returns {HTMLElement|null}
 */
function editorHost(id) {
  return tooltips.get(id) ?? null;
}

/**
 * An editor in `node`. `opened(id, text)` hears a tooltip open, with the
 * words under the pointer; `closed(id)` hears it close.
 * @param {HTMLElement} node
 * @param {(id: string, text: string) => void} opened
 * @param {(id: string) => void} closed
 */
function startEditor(node, opened, closed) {
  const hover = CM.hoverTooltip((view, pos) => ({
    pos,
    create() {
      const id = `hover-${++made}`;
      const dom = document.createElement("div");
      tooltips.set(id, dom);
      opened(id, view.state.doc.lineAt(pos).text);
      return { dom, destroy() { tooltips.delete(id); closed(id); } };
    },
  }));
  return new CM.EditorView({ parent: node, extensions: [CM.basicSetup, hover] });
}
```

This page keeps the open tooltip's id in state and draws the words into
its node:

```wf
page Edit(path: "/edit", title: "Edit", description: "An editor whose tooltip the page draws.") {
    state tip: String? = null
    state words = ""
    action opened(id: String, text: String) {
        tip = id
        words = text
    }
    action closed(id: String) {
        if tip == id { tip = null }
    }

    Heading("Edit").h1
    Host(mount: (n) => startEditor(n, opened, closed), cleanup: (view) => view.destroy())
    if let id = tip {
        Portal(into: editorHost(id), data-popup: "hover") {
            Text(words)
        }
    }
}
```

How a portal behaves:

- **It reads its target again.** `into:` is read like any other
  expression: when a state or a list item it reads changes, it is read
  again. A tooltip that replaces another has a new id, so the block
  follows it to the new node.
- **It moves; it is not drawn again.** When `into:` names another node, the
  same box moves there with its nodes and their state: a field half typed
  in, a list scrolled halfway. `null` keeps the box aside, and when a node
  is named again the same nodes come back.
- **It belongs to where it is written.** Its handlers, its `emit`, the
  state it reads and changes, and its `mount:` and `cleanup:` belong to the
  page or the component it is written in. The node is only where they are
  drawn. A `mount:` inside the block runs once the block is drawn and in
  place, so it meets its node in the document when the node it goes into
  is there.
- **It leaves with its scope.** When the page, the branch or the list item
  it is written in leaves, the box leaves the node with it. The node belongs
  to the library, and stays.
- **The box adds nothing to the layout.** It is a `div` with
  `display: contents`, so the node lays out the block as if the box were not
  there. An `id:`, a `class:`, `data-*` and `aria-*` go on the box.
- **It can target a handle.** `Portal(into: panel)` draws into the element
  a `ref: panel` names, even one written further down the page. Where it
  goes is read once more when the drawing is over and the page is in the
  document, so an element of the page's own that `document.getElementById`
  finds is found too. A handle is not a state: an element the handle names
  only later, in an `if` that opens afterwards, is not followed.
- **Its events bubble from where it is drawn.** A press inside the block
  bubbles through the node it is in and that node's parents, not through
  the elements around the place it is written. An `on click` on a `Stack`
  that holds the portal does not hear it. Put the handler on the portal, or
  inside its block.
- **`show` does not hide it.** `show` hides what is drawn in its own place,
  and a portal's block is drawn somewhere else. Remove it with `if`, or
  keep it aside with `into: null`.
- **The pre-rendered page has only a marker.** The *static paint* (the HTML
  the build pre-renders for a page) has a marker where a portal is, and
  nothing of its block. The live page draws the block once its script has
  run. A test that looks at a portal runs in a browser
  ([Testing](../learn/16-testing.md#test-a-portal)).

A list that the library keeps (an editor's completions, each row a node it
made) is a keyed `for` of portals. A press on a row should leave the focus
in the editor: `on mousedown(e) { e.preventDefault() }` does that, and
`on click` still runs. This fragment draws one portal per row:

```wf
for row, i in Completions.rows by i {
    Portal(into: editorHost(row.id), data-popup: "row") {
        Text(row.label) {
            on mousedown(e) { e.preventDefault() }
            on click { Completions.pick(i) }
        }
    }
}
```

A `Portal` without `into:` is a `C01` error.
