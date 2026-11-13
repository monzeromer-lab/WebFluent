---
kind: "recipe"
route: "/recipes/chart-from-a-library"
group: "interop"
description: "Draw a Chart.js chart on a page: load the library from a CDN, call it from a script of your own, and redraw the chart when the page's data changes."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#draw-a-chart-from-a-library"]
---
# Draw a chart from a library

A charting library such as Chart.js wants a canvas and gives back a chart
object. This page hands a canvas to the library, shows new figures when the
reader switches years, and destroys the chart when the page leaves. You
need a WebFluent project, and the scripts below saved under its `src/`.

```wf
page Sales(path: "/sales", title: "Sales", description: "Sales by quarter, as a bar chart.") {
    state lastYear = false
    derived values: [Number] = if lastYear { [9, 14, 11, 13] } else { [12, 19, 7, 15] }

    Heading("Sales").h1
    Switch(bind: lastYear, label: "Show last year")
    Host(tag: "canvas", role: "img", aria-label: "Sales by quarter",
         mount: (canvas) => drawSales(canvas, values),
         update: (chart) => showSales(chart, values),
         cleanup: (chart) => chart.destroy())
}
```

The page calls two functions of a plain script under `src/`, typed with
JSDoc:

```js
// src/sales-chart.js
/**
 * A bar chart of `values`, one bar a quarter, drawn on `canvas`.
 * @param {HTMLCanvasElement} canvas
 * @param {number[]} values
 */
function drawSales(canvas, values) {
  return new Chart(canvas, {
    type: "bar",
    data: {
      labels: ["Q1", "Q2", "Q3", "Q4"],
      datasets: [{ label: "Sales", data: values }],
    },
  });
}

/**
 * Show new figures on a chart that `drawSales` made.
 * @param {Chart} chart
 * @param {number[]} values
 */
function showSales(chart, values) {
  chart.data.datasets[0].data = values;
  chart.update();
}
```

The library itself loads from a CDN, through `meta.scripts` in
`webfluent.app.json`:

```json
{ "meta": {
    "scripts": ["https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"]
} }
```

## How it works

- **The library loads first.** The build links each `meta.scripts` entry
  with `<script defer>`, in order, before your own scripts and the compiled
  code, so `Chart` exists when `drawSales` runs. It also widens the
  Content-Security-Policy by the CDN's origin, so the policy that ships
  beside the page does not block the library. The compiler never fetches
  the file, so `.wf` code never calls `Chart` directly: your script does.
- **Your script is a plain browser script.** The build copies
  `src/sales-chart.js` to `build/js/` as written and puts its top-level
  names in scope, so the page calls `drawSales` and `showSales` by name.
  Its JSDoc types each call: `values` must be a list of numbers, and a
  call with the wrong number of arguments is a `T10` error
  ([Call your own JavaScript](call-your-own-script.md)). `Chart` and
  `HTMLCanvasElement` are types WebFluent does not know, so they read as
  `Any`.
- **`Host` makes the canvas.** `Host(tag: "canvas", …)` is a bare node for
  somebody else's code. `role: "img"` and `aria-label:` give the chart a
  name a screen reader announces, since a canvas has no text.
- **`mount:` draws the chart.** It runs when the canvas is made, and what
  it returns, the `Chart` object, is the handle `update:` and `cleanup:`
  receive ([Hand an element to a script](hand-an-element-to-a-script.md)).
- **`update:` follows the data.** It reads `values`, so it runs again
  whenever the switch changes `values`, and `showSales` gives the existing
  chart the new figures instead of drawing a new one.
- **`cleanup:` destroys the chart** when the page leaves, so nothing the
  library set up outlives the canvas.

## Variations

To pin the library to the exact file you tested, add its subresource
integrity hash to `meta.integrity`, keyed by the same URL. The build
writes it on the tag with `crossorigin="anonymous"`, and a library from
another origin without one draws a `D05` warning. The compiler never
fetches the file to work the hash out; your CDN lists it, or compute it
from a copy of the file:

```bash
openssl dgst -sha384 -binary chart.umd.min.js | openssl base64 -A
```

The other ways an entry in `meta.scripts` can load (`async`, after the
page, as a module, or with `globals` your `.wf` code calls directly) are
in [Load a library from a CDN](library-from-a-cdn.md).

## Troubleshooting

| What you see | What to do |
|---|---|
| `mount failed ReferenceError: Chart is not defined` in the console | The library did not load: check the URL in `meta.scripts`, and that the page was rebuilt after the config changed |
| The chart draws once and never changes | Make `update:` read the state the chart shows; an `update:` that reads nothing never runs again |
| Two charts on one canvas after a route change | Destroy the chart in `cleanup:` |
