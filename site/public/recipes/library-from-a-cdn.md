# Load a library from a CDN

> Load a JavaScript library from another origin with meta.scripts: in order, async or after the page, pinned by a hash, allowed by the policy.

The page: https://webfluent.monzeromer.dev/recipes/library-from-a-cdn

To use a library from another origin, list its URL in `meta.scripts`. The
compiler never fetches anything, so it cannot read a file on another
origin; a script of your own calls the library instead. This page draws a
Chart.js chart that way, and destroys it when the page leaves:

```wf
page Sales(path: "/sales", title: "Sales", description: "How it is going.") {
    state values: [Number] = [12, 19, 7, 15]

    Heading("Sales").h1
    Host(tag: "canvas", role: "img", aria-label: "Sales by quarter",
         mount: (c) => drawSales(c, values),
         cleanup: (chart) => chart.destroy())
}
```

This script draws the chart with the library:

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

This config, in `webfluent.app.json`, loads Chart.js:

```json
{ "meta": {
    "scripts": ["https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"]
} }
```

## How it works

An entry in `meta.scripts` loads this way:

- **In order.** The build links each entry with `<script defer>` in the
  order listed, before your own scripts and the compiled code. A library is
  there when the code that uses it runs, so `drawSales` finds `Chart`.
- **Allowed by the policy.** The build widens the Content-Security-Policy
  by exactly the origins listed, so the policy that ships beside a declared
  library never blocks it.
- **Called from your script.** `src/charts.js` is a plain script whose
  names are in scope, typed by its JSDoc
  ([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)). `Host` hands it a
  canvas, and `cleanup:` destroys the chart with the page
  ([Hand an element to a script](https://webfluent.monzeromer.dev/recipes/hand-an-element-to-a-script.md)). A chart
  that follows changing data is in
  [Draw a chart from a library](https://webfluent.monzeromer.dev/recipes/chart-from-a-library.md).
- **Pinned.** `meta.integrity` pins a file to a hash, keyed by the same
  URL, and the build writes it on the tag with `crossorigin="anonymous"`.
  A library from another origin without a hash draws a `D05` warning. The
  build never fetches the file to work the hash out: copy it from the CDN,
  or compute it from the file you tested.

This command prints the hash of a downloaded copy, to add as
`"sha384-<hash>"` under the URL in `meta.integrity`:

```bash
openssl dgst -sha384 -binary chart.umd.min.js | openssl base64 -A
```

## Variations

An entry may be an object instead of a URL, to say how it loads.

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
- **As a module.** `{ "src": "https://…/confetti.mjs", "module": true,
  "as": "Confetti" }` imports an ES module and puts its exports on
  `window.Confetti`. Most CDNs also serve a plain build, which needs
  neither key.
- **Called from `.wf` directly.** `{ "src": "…", "globals": ["Chart"] }`
  puts the names a library defines in scope, each typed `Any`, because the
  compiler cannot see their shape. A class among them is constructed when
  called.

## Troubleshooting

| What you see | What to do |
|---|---|
| `D05` on a library from another origin | Add its hash to `meta.integrity` |
| `E111` on an entry | Drop `async` or `load: "after"` from a module, or from an entry with `globals` |
| `ReferenceError` naming the library's global | Check the URL; an `async` or `load: "after"` entry may not have arrived yet when your code runs |
