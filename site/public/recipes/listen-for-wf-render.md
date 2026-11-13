# Run a script after each page draws

> Run a script that finds elements itself after every drawing of the page: the first mount, the takeover of a pre-rendered page, and each route.

The page: https://webfluent.monzeromer.dev/recipes/listen-for-wf-render

A script that finds elements itself, with `querySelectorAll`, has to run
after the page is drawn. The page is drawn after your scripts run, drawn
again over a pre-rendered page, and drawn anew on every route, so the
script listens for the `wf:render` event on `document`. This page shows
Markdown with links to other sites, and a script opens each of them in a
new tab:

```wf
const NOTES = """
    Read the [WebFluent guide](https://webfluent.monzeromer.dev/docs), or
    the [MDN page on links](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a).
    """

page Notes(path: "/notes", title: "Notes", description: "Links in Markdown that open in a new tab.") {
    Heading("Notes").h1
    Markdown(NOTES, class: "notes")
}
```

The script is a plain file under `src/`
([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)):

```js
// src/new-tab.js
document.addEventListener("wf:render", (e) => {
  // e.detail.route is the route that was drawn, e.detail.params its parameters.
  for (const a of document.querySelectorAll(".notes a")) {
    const href = a.getAttribute("href") || "";
    if (a.hasAttribute("target") || !href.startsWith("http")) continue;
    a.setAttribute("target", "_blank");
    a.setAttribute("rel", "noopener noreferrer");
  }
});
```

## How it works

- The build links the script on every page before the compiled code, so
  the listener is in place before the first drawing.
- `wf:render` fires on `document` once per drawing: the first mount, the
  takeover of a pre-rendered page, and every route. `e.detail` carries the
  `route` that was drawn and its `params`.
- The script skips a link that already has a `target`, so a second drawing
  of the same page changes nothing twice.
- The links come from `Markdown`, which writes them itself, so the page
  has no element of its own to put a `target` on. The script adds it,
  with `rel="noopener noreferrer"` beside it so the new tab cannot reach
  back into this one.

## Variations

`wf:render` does not fire for an `if` branch or a list item that arrives
later. For an element that comes and goes inside a page, hand it to the
script with `mount:` instead
([Hand an element to a script](https://webfluent.monzeromer.dev/recipes/hand-an-element-to-a-script.md)).
