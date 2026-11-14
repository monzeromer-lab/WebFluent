---
kind: "recipe"
route: "/recipes/library-from-a-cdn"
group: "interop"
description: "Load a JavaScript library from another origin with meta.scripts: in order, async or after the page, pinned by a hash, allowed by the policy."
outputs: ["website", "app"]
from: ["/docs/guide/interop#load-a-library-from-a-cdn"]
---
# Load a library from a CDN

To use a library from another origin, list its URL in `meta.scripts`. The
compiler never fetches anything, so it cannot read a file on another
origin; a script of your own calls the library instead. This page throws
confetti with the `canvas-confetti` library when the reader presses a
button:

**Needs:** the script `src/celebrate.js` and the `meta.scripts` setting,
both shown below.

```wf
page Launch(path: "/launch", title: "Launch", description: "A button that celebrates.") {
    Heading("We shipped").h1
    Button("Celebrate").primary { on click { celebrate() } }
}
```

This script calls the library:

```js
// src/celebrate.js
/** Throw confetti up from the bottom of the window. */
function celebrate() {
  if (typeof confetti === "undefined") return; // not arrived yet
  const fire = confetti.create(null, { resize: true });
  fire({ particleCount: 120, spread: 70, origin: { y: 0.9 } });
}
```

This config, in `webfluent.app.json`, loads the library:

```json
{ "meta": {
    "scripts": ["https://cdn.jsdelivr.net/npm/canvas-confetti@1.9.3/dist/confetti.browser.min.js"]
} }
```

Press **Celebrate** and confetti bursts up from the bottom of the window.

## How it works

An entry in `meta.scripts` loads this way:

- **In order.** The build links each entry with `<script defer>` in the
  order listed, before your own scripts and the compiled code. A library is
  there when the code that uses it runs, so `celebrate` finds `confetti`.
- **Allowed by the policy.** The build widens the Content-Security-Policy
  by exactly the origins listed, so the policy that ships beside a declared
  library never blocks it.
- **Called from your script.** `src/celebrate.js` is a plain script whose
  names are in scope, typed by its JSDoc
  ([Call your own JavaScript](call-your-own-script.md)). A library that
  draws into an element of the page, such as a chart, is handed one by
  `Host`: see [Draw a chart from a library](chart-from-a-library.md).
- **Pinned.** `meta.integrity` pins a file to a hash, keyed by the same
  URL, and the build writes it on the tag with `crossorigin="anonymous"`.
  A library from another origin without a hash draws a `D05` warning. The
  build never fetches the file to work the hash out: copy it from the CDN,
  or compute it from the file you tested.

This command prints the hash of a downloaded copy, to add as
`"sha384-<hash>"` under the URL in `meta.integrity`:

```bash
openssl dgst -sha384 -binary confetti.browser.min.js | openssl base64 -A
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
  The confetti above could load this way too, since nothing calls it until
  a press, and `celebrate` does nothing if pressed before it arrives.
- **As a module.** `{ "src": "https://…/confetti.mjs", "module": true,
  "as": "Confetti" }` imports an ES module and puts its exports on
  `window.Confetti`. Most CDNs also serve a plain build, which needs
  neither key.
- **Called from `.wf` directly.** `{ "src": "…", "globals": ["confetti"] }`
  puts the names a library defines in scope, each typed `Any`, because the
  compiler cannot see their shape. A class among them is constructed when
  called.

## Troubleshooting

| What you see | What to do |
|---|---|
| `D05` on a library from another origin | Add its hash to `meta.integrity` |
| `E111` on an entry | Drop `async` or `load: "after"` from a module, or from an entry with `globals` |
| `ReferenceError` naming the library's global | Check the URL; an `async` or `load: "after"` entry may not have arrived yet when your code runs |
