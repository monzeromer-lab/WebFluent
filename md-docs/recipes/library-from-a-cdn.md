---
kind: "recipe"
route: "/recipes/library-from-a-cdn"
group: "interop"
description: "To use a library from another origin, list its URL in `meta.scripts`."
from: ["/docs/guide/interop#load-a-library-from-a-cdn"]
---
# Load a library from a CDN

To use a library from another origin, list its URL in `meta.scripts`. The
compiler never fetches anything, so it cannot read a file on another
origin; your own scripts call the library instead.

This config loads Chart.js and pins the file with its integrity hash:

```json
{ "meta": {
    "scripts": ["https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"],
    "integrity": {
      "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js": "sha384-…"
    }
} }
```

This script draws a chart with it:

```js
// src/charts.js
/**
 * A bar chart of `values`, one bar a quarter.
 * @param {HTMLCanvasElement} canvas
 * @param {number[]} values
 */
function drawSales(canvas, values) {
  return new Chart(canvas, {
    type: "bar",
    data: { labels: ["Q1", "Q2", "Q3", "Q4"], datasets: [{ label: "Sales", data: values }] },
  });
}
```

This page hands the script a canvas, and destroys the chart when the page
leaves:

```wf
page Sales(path: "/sales", title: "Sales", description: "How it is going.") {
    state values: [Number] = [12, 19, 7, 15]

    Heading("Sales").h1
    Host(tag: "canvas",
         mount: (c) => drawSales(c, values),
         cleanup: (chart) => chart.destroy())
}
```

How an entry in `meta.scripts` loads:

- **In order.** The build links each entry with `<script defer>` in the
  order listed, before your own scripts and the compiled code. A library is
  there when the code that uses it runs.
- **Out of order.** `{ "src": "…", "async": true }` loads a script that
  nothing on the page waits for (analytics, a tag manager) with
  `<script async>`. Deferred scripts run in order, so a large one listed
  plainly would hold back the compiled code, and the page's first full
  paint, until it arrived. An `async` script runs whenever it arrives
  instead. A module, or a script whose `globals` your `.wf` code calls,
  cannot be `async` (`E111`).
- **After the page.** `{ "src": "…", "load": "after" }` puts no tag in the
  HTML at all: the runtime requests the script once the page has loaded and
  drawn its first frame. A browser fetches everything a page names while it
  loads, and a page's speed is measured against what it asked for before it
  first painted. A large analytics tag named in the HTML can cost a phone's
  Largest Contentful Paint more than a second, even with `async`. The
  script's origin still joins the policy. A reader who leaves within the
  first second is not counted. The same limits as `async` apply (`E111`).
- **Pinned.** `meta.integrity` pins the file to a hash. A library from
  another origin without a hash draws a `D05` warning.
- **Allowed by the policy.** The build widens the Content-Security-Policy
  by exactly the origins listed, so the policy that ships beside a declared
  library never blocks it.
- **As a module.** `{ "src": "https://…/confetti.mjs", "module": true,
  "as": "Confetti" }` imports an ES module and puts its exports on
  `window.Confetti`. Most CDNs also serve a plain build, which needs
  neither key.
- **Called from `.wf` directly.** `{ "src": "…", "globals": ["Chart"] }`
  puts the names a library defines in scope, each typed `Any`, because the
  compiler cannot see their shape. A class among them is constructed when
  called.
