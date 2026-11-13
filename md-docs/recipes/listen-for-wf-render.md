---
kind: "recipe"
route: "/recipes/listen-for-wf-render"
group: "interop"
description: "The page is drawn after your scripts run, drawn again over a pre-rendered page, and drawn anew on every route."
from: ["/docs/guide/interop#listen-for-wf-render-in-a-script-that-finds-elements-itself"]
---
# Run a script after each page draws

The page is drawn after your scripts run, drawn again over a pre-rendered
page, and drawn anew on every route. A script that runs its own
`querySelectorAll` therefore listens for the drawing to land, with the
`wf:render` event on `document`:

```js
// src/tilt-all.js
document.addEventListener("wf:render", (e) => {
  // e.detail.route is the route just drawn, e.detail.params its parameters.
  for (const el of document.querySelectorAll(".tilt-me:not([data-tilted])")) {
    el.dataset.tilted = "";
    initTilt(el);
  }
});
```

`wf:render` fires once per drawing: the first mount, the takeover of a
pre-rendered page, and every route. It does not fire for an `if` branch or
a list item that arrives later. Use `mount:` for those.
