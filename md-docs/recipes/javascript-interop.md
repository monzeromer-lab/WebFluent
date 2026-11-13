---
chapter: 39
kind: "recipe"
route: "guide/interop"
group: "outputs"
blurb: "Write plain JavaScript beside your pages and call it by name; load a library from a CDN; hand an element to a script; place custom elements; publish your components."
description: "Your own .js under src/, libraries via meta.scripts, mount and cleanup on any element, Portal, wf:render, Element for custom elements, and publishing yours."
---
# 39. JavaScript interop

This chapter shows you how to use JavaScript from a WebFluent project: your
own scripts, libraries from a CDN, and custom elements. WebFluent has no
package manager of its own and does not need one. A `.js` file is part of
the project the same way a `.css` file is: put it under `src/`, and your
pages can call its functions by name.

Pick the section for what you want to do:

| You want to… | Use | Section |
|---|---|---|
| call JavaScript of your own | a `.js` file under `src/` | [Call your own script](#call-your-own-script) |
| use a library from a CDN | `meta.scripts` in the config | [Load a library from a CDN](#load-a-library-from-a-cdn) |
| hand an element to a script, and take it back | `mount:`, `update:`, `cleanup:` on any element | [Hand an element to a script](#hand-an-element-to-a-script-with-mount-update-and-cleanup) |
| draw into a node a script made (a tooltip, a popup) | `Portal(into: node) { … }` | [Draw into a node a script made](#draw-into-a-node-a-script-made-with-portal) |
| let a script find elements itself | the `wf:render` event | [Listen for `wf:render`](#listen-for-wf-render-in-a-script-that-finds-elements-itself) |
| place somebody else's custom element | `Element("tag-name", …)` | [Place a custom element](#place-a-custom-element-from-a-library) |
| use your components from React, Vue or a plain page | `output_type: "elements"` | [Publish your components](#publish-your-components-as-custom-elements) |

## Before you start

You need a WebFluent project ([Getting started](../learn/01-getting-started.md)) and
a script written as a plain browser script: no `import`, no `export`, no
build step.

## Call your own script

To call a function of your own, put it in a `.js` file under `src/`. Write
it the way you would write any file a `<script src>` loads.

This script tilts an element toward the pointer, and returns a handle the
page can update and destroy:

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

This page calls `initTilt` on a card, and changes how far it tilts as the
reader moves a slider:

```wf
page Gallery(path: "/gallery", title: "Gallery", description: "Cards that lean.") {
    state max = 15

    Heading("Gallery").h1
    Card(class: "tilt",
         mount: (n) => initTilt(n, max),
         update: (tilt) => tilt.setMax(max),
         cleanup: (tilt) => tilt.destroy()) {
        Text("Move the pointer over me.")
    }
    Slider(bind: max, min: 0, max: 45, label: "How far")
}
```

How the build treats a script:

- **The file runs as written.** The build copies it to `build/js/tilt.js`
  byte for byte and links it with `<script defer>` on every page, before the
  compiled code. A stack trace points at your own line.
- **Its names are in scope.** Every top-level `function`, `class`, `const`,
  `let` and `var`, and anything assigned to `window.x`, is a name your `.wf`
  code can call directly. What a function or an IIFE keeps inside stays
  private, as it does in the browser.
- **A call is checked.** `initTilt(n, 1, 2)` is a `T10` error where it is
  written, because the function takes one or two arguments. Its JSDoc types
  the arguments ([Type a script with JSDoc](#type-a-script-with-jsdoc)). A
  name the file does not declare is still a `T13`.
- **A class is constructed.** WebFluent has no `new`. A call to a class a
  script declares compiles to `new`.
- **One name has one meaning.** Two scripts that declare the same name, or
  a script name that is also a component, a store, a `const`, or a name the
  language or the browser owns (`format`, `log`, `fetch`), is an error that
  names both.
- **An ES module is refused.** A file with a top-level `import` or `export`
  cannot run from a `<script src>`, so the build stops and says so. Load a
  published module from the config instead
  ([Load a library from a CDN](#load-a-library-from-a-cdn)).

The tools know your scripts too: `wf serve` rebuilds when a script changes,
`wf audit` lists what each one declares, `wf types` prints each function's
signature, and the editor completes, documents and jumps to them.

### Type a script with JSDoc

To have calls checked against a function's types, describe the function in
a JSDoc comment. JSDoc is a comment, so the file stays exactly what the
browser runs. The checker reads it this way:

| JSDoc | WebFluent |
|---|---|
| `string` · `number` · `boolean` | `String` · `Number` · `Bool` |
| `object` · `{ a: T }` | `Map`, or a shape of the fields written |
| `T[]` · `Array<T>` | `[T]` |
| `?T` · `T \| null` · `[name]` | `T?`, or a parameter a call may leave out |
| `Promise<T>` | `T`, as an action that awaits it reads it |
| `(v: T) => R` | a function: you may pass an action or a lambda |
| `HTMLElement`, anything else | `Any` |

With no comment at all, every parameter is `Any`, so adding a script never
breaks a build. Each tag only narrows what a call may pass. `@returns` types
the result.

### Let a script report, and keep the class in WebFluent

When the way an element looks depends on something only a script can see
(an intersection, a scroll position, a library's event), hand the script an
action. An action is a plain function at run time. The script calls it, and
the class stays in a `class:` the compiler can read.

This script reports whether an element is on screen:

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

This page sets `is-visible` on the card while it is on screen:

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

Each class has one owner. A class that `class:` names belongs to
WebFluent, and a class a script adds with `classList.add` belongs to the
script: `class:` only ever takes off what it put on.

## Load a library from a CDN

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

## Hand an element to a script with `mount:`, `update:` and `cleanup:`

To give an element to a library and take it back when the element goes,
use the three lifetime props. Every element and every component call takes
them:

- `mount: (node) => …` runs each time the element is made: the first
  paint, a route change, an `if` branch arriving, a list item added. What
  it returns is the *handle* the other two receive.
- `update: (handle) => …` runs again whenever the state it reads changes.
- `cleanup: (handle) => …` runs when the element leaves (its page, its
  branch or its list item), so nothing a library made outlives it.

On a component call, the three belong to the component's root element,
unless the component declares a prop of that name itself. A `mount:` that
throws removes itself and logs the error; the page stays up.

To get a bare node for a library, use `Host`. It has the same lifetime:
`Host(tag: "canvas", …)` makes the element the library wants. It makes a
`div` by default, or a `span`, `canvas`, `svg`, `section`, `figure`, `pre`,
`p`, `ul` or `table`.

## Draw into a node a script made with `Portal`

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

## Listen for `wf:render` in a script that finds elements itself

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

## Place a custom element from a library

To place a custom element that a library defines, use `Element` with the
element's tag. This page places Stripe's pricing table and notes when it is
ready:

```wf
page Pricing(path: "/pricing", title: "Pricing", description: "Plans.") {
    state loaded = false
    Heading("Pricing").h1
    Element("stripe-pricing-table", publishableKey: "pk_live_…", pricing-table-id: "prctbl_1") {
        on ready { loaded = true }
    }
}
```

- The tag comes first. It is lower case, with a hyphen, as every custom
  element's tag is; any other tag is an `E116` error.
- A named argument is an attribute: `publishableKey` is written
  `publishable-key`. An attribute follows state like any other value.
- An `on` handler hears the events the element fires, whatever their names.
- The block is the element's children.
- `class:`, `style { }` and `mount:` work as on any other element.

The script that defines the element (your own, or one in `meta.scripts`)
upgrades it when it runs.

## Publish your components as custom elements

To use your components from another framework or a plain page, build them
as custom elements. Name the components in `build.elements`:

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

Each name becomes a standard custom element. `PriceTag` is `<price-tag>`.
A one-word component gets a prefix, so its tag has the hyphen a custom
element needs: `Rating` is `<wf-rating>`. The build writes `elements.js`,
`styles.css`, and a page that lists the tags it published.

This HTML places a published component on any page:

```html
<script src="/elements.js" defer></script>
<link rel="stylesheet" href="/styles.css">

<price-tag label="Pro" amount="29" sale></price-tag>
```

How a published element behaves:

- **An attribute is a prop.** It is read when the element is connected, and
  again whenever it changes. A `Number` prop is read as a number, and a
  `Map` or a list as JSON.
- **A `Bool` prop is true when the attribute is present**, as HTML reads
  `disabled`, and false when it is `="false"`, so a framework that writes
  the string still works.
- **An event the component declares** is dispatched as a `CustomEvent`
  that bubbles.
- **What the component created goes** when the element leaves the
  document.

React, Vue, Svelte, Angular, Rails, WordPress and a plain page can all
place a tag. That is why WebFluent publishes custom elements rather than
an adapter per framework: the interface every framework shares is the web
platform itself.

## Call `WF` from a script of your own

The runtime is `window.WF`. A script under `src/`, the console, or a
library's callback can call it: `WF.navigate("/about")`,
`WF.toast("Saved", "success")`, `WF.storeSnapshot("Cart")`. The
[runtime reference](../reference/runtime-api.md) lists every function.

> [!WARNING]
> A build keeps only the runtime modules its own code reaches. A script that
> calls a module nothing else uses needs `"build": { "runtime": "full" }`,
> or the call fails in the browser.

A script from another origin must be listed in `meta.scripts`, which widens
the policy for it. The build's CSP check refuses an inline `<script>`
([Security](../about/security.md#ship-a-content-security-policy)).

## Migrate from `external`

Before 4.2, you described JavaScript with `external` declarations; 4.2
removed them. `wf migrate` rewrites a project:

- A remote module becomes a `meta.scripts` entry. Its calls keep working,
  typed `Any`.
- A local module becomes a plain script under `src/`, with its exports
  removed and its signatures as JSDoc. `X.f(…)` is rewritten to `f(…)`.
- An `external element` call site becomes `Element("tag-name", …)`.

## Next

[Components reference](../reference/components.md): every built-in with its props, cases, flags, events, slots and parts — generated from the compiler's own registry.
