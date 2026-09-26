# 32. JavaScript interop

<!--
route: guide/interop
group: shipping
blurb: Use a JavaScript library — a chart, a map, an editor — with a typed declaration and a node with a lifetime; place other people's custom elements; publish your components for any framework.
description: Typed external modules, Host for libraries that own a DOM node, other people's custom elements, publishing yours, and WF from a script.
-->

WebFluent has no package manager of its own, and does not need one to use
the JavaScript ecosystem. Four doors, each typed:

| You want to… | Use |
|---|---|
| call a JavaScript library | `external Name from "url" { … }` |
| give a library a DOM node and clean it up | `Host(tag:, mount:, update:, cleanup:)` |
| place somebody else's custom element | `external element Name("tag-name") { … }` |
| use your components from React, Vue or a plain page | `output_type: "elements"` |

## A library, end to end

Chart.js, drawing a bar chart that follows state:

```wf
external Charts from "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/auto/+esm" {
    fn Chart(canvas: Any, config: Map) -> ChartHandle
    type ChartHandle {
        update()
        destroy()
        data: Map
    }
}

page Sales(path: "/sales", title: "Sales", description: "How it is going.") {
    state values: [Number] = [12, 19, 7, 15]

    action redraw(chart: ChartHandle) {
        chart.data.datasets[0].data = values
        chart.update()
    }

    Heading("Sales").h1
    Host(tag: "canvas",
         mount: (node) => Charts.Chart(node, { type: "bar", data: { labels: ["Q1", "Q2", "Q3", "Q4"], datasets: [{ label: "Sales", data: values }] } }),
         update: (chart) => redraw(chart),
         cleanup: (chart) => chart.destroy())
    Button("Add a quarter") { on click { values = [...values, 10] } }
}
```

- **`external Charts from "…"`** imports the module. `fn Chart(…) -> ChartHandle`
  is the contract: `Charts.Chart(node, config)` is checked against it, and a
  call that does not match is an error where it is written, not a
  `TypeError` in a browser. A `type` inside names the shape of what the
  library hands back — methods and fields.
- **A class is constructed.** `Chart` is a JavaScript class; a WebFluent call
  has no `new`, so an exported class is constructed when it is called.
- **`Host`** gives the library its element. `mount` runs once with the node
  and returns whatever the library hands over. `update` runs again whenever
  the state it reads changes — here `values`, read inside `redraw`.
  `cleanup` runs when the page, branch or list item leaves, so the chart is
  destroyed with it.

## `external`, in full

```wf
external Confetti from "https://cdn.jsdelivr.net/npm/canvas-confetti@1.9.3/+esm" {
    integrity: "sha384-…"
    fn default(options: Map)
}
```

- `from` is any module specifier: a URL, or a bare name your import map or
  host resolves (`external d3 from "d3"`).
- `fn name(params) -> Type` declares a function the module exports; the
  return type is optional. A module's default export is `default`.
- `type Name { method(params)  field: Type }` declares a shape the module's
  values have.
- `integrity:` pins the file: the build emits a `<link rel="modulepreload"
  integrity="…">`, the one place the platform lets subresource integrity
  reach a module.

What the build does: writes `externals.js` holding the `import`s, links it
from every page as `<script type="module">` ahead of the bundle, and adds
each module's origin to the Content-Security-Policy, so a declared import is
never blocked by the policy that ships beside it.

## `Host`

`Host(tag: "canvas", mount: …, update: …, cleanup: …)`:

- `tag:` is the element the library is given — `div` by default, or `span`,
  `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul`, `table`.
- `mount: (node) => …` runs once; what it returns is the handle `update` and
  `cleanup` receive.
- `update: (handle) => …` runs whenever what it reads changes.
- `cleanup: (handle) => …` runs when the owning scope leaves.
- A library that throws on mount takes itself out and logs the error; the
  rest of the page stays.

Writing the same by hand is a `ref`, an `effect` and a `cleanup` — and the
cleanup is what people forget. `Host` is the version that cannot.

## Somebody else's custom element

```wf
external element Stripe("stripe-pricing-table") {
    prop publishableKey: String
    prop pricingTableId: String
    event ready()
}
```

```wf
Stripe(publishableKey: key, pricingTableId: "prctbl_1") { on ready { loaded = true } }
```

It is placed like a component. A prop becomes the attribute a framework
would write — `publishableKey` is `publishable-key` — and follows state
like any other value; an event it declares is a DOM event the page hears.

## Publishing your components as custom elements

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

Each name becomes a standards-based custom element: `PriceTag` is
`<price-tag>`, and a one-word component is prefixed so its tag has the
hyphen a custom element needs (`Rating` is `<wf-rating>`). The build writes
`elements.js`, `styles.css` and a page listing the tags it published.

```html
<script src="/elements.js" defer></script>
<link rel="stylesheet" href="/styles.css">

<price-tag label="Pro" amount="29" sale></price-tag>
```

An attribute is a prop, read when the element is connected and again
whenever it changes: a `Number` prop is read as a number, a `Map` or a list
as JSON, and a `Bool` is true when the attribute is present — as HTML reads
`disabled` — and false when it is `="false"`, so a framework that writes
the string still works. An event the component declares is dispatched as a
`CustomEvent` that bubbles. What the component created goes when the
element leaves the document.

React, Vue, Svelte, Angular, Rails, WordPress and a plain page can all
place a tag. That is the whole reason this is the answer rather than an
adapter per framework: **the shared interface between frameworks is the
platform.**


## `WF` from a script of your own

The runtime is `window.WF`: a hand-written script in `public/`, the
console, or a library's callback can use it — `WF.navigate("/about")`,
`WF.toast("Saved", "success")`, `WF.storeSnapshot("Cart")`. The
[runtime reference](42-runtime-api.md) lists every function.

Two things to know:

- A build keeps only the runtime modules its own code reaches. A script that
  calls a module nothing else uses needs `"build": { "runtime": "full" }`.
- A script from another origin must be allowed by the policy: add it to
  `meta.stylesheets`/`meta.fonts` for those, or load it through `external`,
  which widens the policy for you. An inline `<script>` is refused by the
  build's CSP check ([Security](23-security.md#content-security-policy)).

## Next

[PDF documents and slide decks](33-pdf-and-slides.md).
