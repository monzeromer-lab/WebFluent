# 32. JavaScript interop

<!--
route: guide/interop
group: shipping
blurb: Write plain JavaScript beside your pages and call it by name; load a library from a CDN; hand an element to a script and get it back; place custom elements; publish your components for any framework.
description: Your own .js files under src/, libraries via meta.scripts, mount and cleanup on any element, wf:render, Element for custom elements, and publishing yours.
-->

WebFluent has no package manager of its own, and does not need one to use
the JavaScript ecosystem. A `.js` file is part of the project the way a
`.css` file is: put it under `src/` and its functions are there.

| You want to… | Use |
|---|---|
| call JavaScript of your own | a `.js` file under `src/` |
| use a library from a CDN | `meta.scripts` in the config |
| hand an element to a script, and take it back | `mount:`, `update:`, `cleanup:` on any element |
| let a script find elements itself | the `wf:render` event |
| place somebody else's custom element | `Element("tag-name", …)` |
| use your components from React, Vue or a plain page | `output_type: "elements"` |

## Your own script

A plain browser script, written as any `<script src>` file is written — no
`export`, no `import`, no build step:

```js
// src/tilt.js
/**
 * Tilt an element toward the pointer.
 * @param {HTMLElement} node
 * @param {number} [max] how far, in degrees
 */
function initTilt(node, max = 15) {
  const move = (e) => {
    const r = node.getBoundingClientRect();
    const x = (e.clientX - r.left) / r.width - 0.5;
    node.style.transform = `rotateY(${x * max}deg)`;
  };
  node.addEventListener("pointermove", move);
  return {
    setMax(m) { max = m; },
    destroy() { node.removeEventListener("pointermove", move); },
  };
}
```

```wf
page Gallery(path: "/gallery", title: "Gallery", description: "Cards that lean.") {
    state max = 15

    Heading("Gallery").h1
    Card(class: "tilt",
         mount: (n) => initTilt(n, max),
         update: (t) => t.setMax(max),
         cleanup: (t) => t.destroy()) {
        Text("Move the pointer over me.")
    }
    Slider(bind: max, min: 0, max: 45, label: "How far")
}
```

- **The file runs as written.** It is copied to `build/js/tilt.js` byte for
  byte and linked with `<script defer>` on every page, before the compiled
  code — so a stack trace points at your line.
- **Its names are in scope.** Every top-level `function`, `class`, `const`,
  `let` and `var`, and anything assigned to `window.x`, is a name `.wf` code
  calls directly. What a function or an IIFE keeps inside stays private, as
  it does in the browser.
- **A call is checked.** `initTilt(n, 1, 2)` is a `T10` where it is written —
  the script takes one or two arguments — and its JSDoc types the rest
  (below). Nothing about it is a guess: a name the file does not declare is
  still a `T13`.
- **A class is constructed.** WebFluent has no `new`; a call to a class a
  script declares compiles to one.
- **One name, one meaning.** Two scripts declaring one name, or a script
  name that is also a component, store, `const` or a name the language or
  the browser owns (`format`, `log`, `fetch`), is an error naming both.
- **An ES module is refused.** A file with a top-level `import` or `export`
  cannot run from a `<script src>`, so the build stops on it and says so. A
  published module is loaded from the config (below).

`wf serve` rebuilds when a script changes, `wf audit` lists what each one
declares, `wf types` prints each function's signature, and the editor
completes, documents and jumps to them.

### Types from JSDoc

JSDoc is a comment, so the file stays exactly what the browser runs. The
checker reads it:

| JSDoc | WebFluent |
|---|---|
| `string` · `number` · `boolean` | `String` · `Number` · `Bool` |
| `object` · `{ a: T }` | `Map`, or a shape of the fields written |
| `T[]` · `Array<T>` | `[T]` |
| `?T` · `T \| null` · `[name]` | `T?`, or a parameter a call may leave out |
| `Promise<T>` | `T`, as an action that awaits it reads it |
| `(v: T) => R` | a function — an action or a lambda may be passed |
| `HTMLElement`, anything else | `Any` |

With no comment at all every parameter is `Any`, so adding a script never
breaks a build; each tag only narrows what a call may pass.

### A script reports; WebFluent owns the class

When how an element looks depends on something only a script can see — an
intersection, a scroll position, a library's event — hand the script an
action. An action is a plain function at run time, and the class stays a
`class:` the compiler can read:

```js
// src/visible.js
/**
 * Say whether `node` is on screen, each time that changes.
 * @param {HTMLElement} node
 * @param {(v: boolean) => void} onChange
 */
function watchVisible(node, onChange) {
  const io = new IntersectionObserver(([e]) => onChange(e.isIntersecting));
  io.observe(node);
  return { destroy: () => io.disconnect() };
}
```

```wf
page Story(path: "/story", title: "Story", description: "It fades in.") {
    state visible = false
    action setVisible(v: Bool) { visible = v }

    Heading("Story").h1
    Card(class: { "is-visible": visible },
         mount: (n) => watchVisible(n, setVisible),
         cleanup: (w) => w.destroy()) {
        Text("Here I am.")
    }
}
```

One owner per class: a class `class:` names is WebFluent's, and a class a
script adds with `classList.add` is the script's — `class:` only ever takes
off what it put on.

## A library from a CDN

The compiler never fetches anything, so it cannot read a file on another
origin. A library is a URL the config lists, and your own scripts use it:

```json
{ "meta": {
    "scripts": ["https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"],
    "integrity": {
      "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js": "sha384-…"
    }
} }
```

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

```wf
page Sales(path: "/sales", title: "Sales", description: "How it is going.") {
    state values: [Number] = [12, 19, 7, 15]

    Heading("Sales").h1
    Host(tag: "canvas",
         mount: (c) => drawSales(c, values),
         cleanup: (chart) => chart.destroy())
}
```

- **Order.** Each entry is linked with `<script defer>` in the order
  listed, before your own scripts and the compiled code, so a library is
  there when the code that uses it runs.
- **Out of order.** `{ "src": "…", "async": true }` loads a script nothing
  on the page waits for — analytics, a tag manager — with `<script async>`.
  Deferred scripts run in order, so a large one listed plainly would hold
  the compiled code, and the page's first full paint, until it had
  arrived; an `async` one runs whenever it arrives instead. A module, or a
  script whose `globals` `.wf` calls, cannot be `async` (`E111`).
- **Integrity.** `meta.integrity` pins the file; a library from another
  origin without a hash draws a warning.
- **The policy.** The Content-Security-Policy is widened by exactly the
  origins listed, so a declared library is never blocked by the policy that
  ships beside it.
- **A module.** `{ "src": "https://…/confetti.mjs", "module": true, "as":
  "Confetti" }` imports an ES module and puts its exports on
  `window.Confetti`. Most CDNs also serve a plain build, which needs
  neither.
- **Calling it from `.wf` directly.** `{ "src": "…", "globals": ["Chart"] }`
  puts the names a library defines in scope, each `Any` — the compiler
  cannot see their shape. A class among them is constructed when called.

## `mount:`, `update:`, `cleanup:` — on any element

Every element and every component call takes the three:

- `mount: (node) => …` runs each time the element is made — the first
  paint, a route change, an `if` branch arriving, a list item added. What it
  returns is the handle the other two get.
- `update: (handle) => …` runs again whenever the state it reads changes.
- `cleanup: (handle) => …` runs when the element leaves — its page, its
  branch or its list item — so nothing a library made outlives it.
- On a component call they belong to the component's root element, unless
  the component declares a prop of that name itself.
- A mount that throws takes itself out and logs the error; the page stays.

`Host` is the same lifetime on a bare node: `Host(tag: "canvas", …)` makes
the element a library wants — `div` by default, or `span`, `canvas`, `svg`,
`section`, `figure`, `pre`, `p`, `ul`, `table`.

## `wf:render` — for a script that finds elements itself

The page is drawn after your scripts run, drawn again over a pre-rendered
one, and drawn anew on every route, so a script that does its own
`querySelectorAll` listens for the drawing to land:

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

It fires once per drawing, not for an `if` branch or a list item arriving
later — that is what `mount:` is for.

## Somebody else's custom element

```wf
page Pricing(path: "/pricing", title: "Pricing", description: "Plans.") {
    state loaded = false
    Heading("Pricing").h1
    Element("stripe-pricing-table", publishableKey: "pk_live_…", pricing-table-id: "prctbl_1") {
        on ready { loaded = true }
    }
}
```

`Element` places any custom element by its tag — lower case, with a hyphen,
as every custom element's is. A named argument is an attribute
(`publishableKey` is written `publishable-key`) and follows state like any
other value; an `on` handler hears the events it fires, whatever their
names; the block is its children; `class:`, `style { }` and `mount:` work as
on anything else. The script that defines it — your own, or one in
`meta.scripts` — upgrades it when it runs.

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

The runtime is `window.WF`: a script under `src/`, the console, or a
library's callback can use it — `WF.navigate("/about")`,
`WF.toast("Saved", "success")`, `WF.storeSnapshot("Cart")`. The
[runtime reference](42-runtime-api.md) lists every function.

Two things to know:

- A build keeps only the runtime modules its own code reaches. A script that
  calls a module nothing else uses needs `"build": { "runtime": "full" }`.
- A script from another origin must be listed in `meta.scripts`, which
  widens the policy for it. An inline `<script>` is refused by the build's
  CSP check ([Security](23-security.md#content-security-policy)).

## Coming from `external`

Before 4.2, JavaScript was described with `external` declarations; 4.2
removed them. `wf migrate` rewrites a project: a remote module becomes a
`meta.scripts` entry (its calls keep working, typed `Any`), a local module
becomes a plain script under `src/` with its signatures as JSDoc and
`X.f(…)` rewritten to `f(…)`, and `external element` call sites become
`Element("tag-name", …)`.

## Next

[PDF documents and slide decks](33-pdf-and-slides.md).
