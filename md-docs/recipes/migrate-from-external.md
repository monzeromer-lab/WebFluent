---
kind: "recipe"
route: "/recipes/migrate-from-external"
group: "interop"
description: "Move a WebFluent 4 project off external declarations: wf migrate turns each into a meta.scripts entry, a plain script under src/, or an Element call."
outputs: ["website", "app"]
from: ["/docs/guide/interop#migrate-from-external"]
---
# Upgrade a WebFluent 4 project that uses external

Before 4.2, you described JavaScript with `external` declarations; 4.2
removed them, and the parser now refuses one. `wf migrate` rewrites a
project. This is a page after the migration: it calls a function of a
plain script, and places a custom element by its tag:

```wf
// Your publishable key, from the Stripe dashboard.
const STRIPE_KEY = "pk_test_123"

page Pricing(path: "/pricing", title: "Pricing", description: "Plans, on a card that leans.") {
    Heading("Pricing").h1
    Card(mount: (n) => initTilt(n, 10)) { Text("Pro") }
    Element("stripe-pricing-table", publishableKey: STRIPE_KEY)
}
```

The function it calls is the old local module, now a plain script under
`src/`, its `export` taken off and the declaration's signature written
above it as JSDoc:

```js
// src/vendor/tilt.js
/**
 * @param {*} node
 * @param {number} max
 */
function initTilt(node, max) {
  node.addEventListener("pointermove", (e) => {
    const r = node.getBoundingClientRect();
    const x = (e.clientX - r.left) / r.width - 0.5;
    node.style.transform = `rotateY(${x * max}deg)`;
  });
}
```

This is the same page as WebFluent 4.1 wrote it, with the module in
`public/vendor/tilt.js`. It is not compiled here, because 4.2 refuses
`external`:

```wf no-check
external Tilt from "./vendor/tilt.js" {
    fn initTilt(node: Any, max: Number)
}

external element PricingTable("stripe-pricing-table") {
    prop publishableKey: String
}

const STRIPE_KEY = "pk_test_123"

page Pricing(path: "/pricing", title: "Pricing", description: "Plans, on a card that leans.") {
    Heading("Pricing").h1
    Card(mount: (n) => Tilt.initTilt(n, 10)) { Text("Pro") }
    PricingTable(publishableKey: STRIPE_KEY)
}
```

To migrate, run `wf migrate` in the project. `--check` writes nothing and
says what would change:

```bash
wf migrate --check
wf migrate
```

## How it works

`wf migrate` takes each `external` out of the source and puts what it said
where 4.2 says it:

- **A remote module** (`external X from "https://…"`) becomes a
  `meta.scripts` entry with `"module": true` and `"as": "X"`, and its
  `integrity:` moves to `meta.integrity`. Its calls keep working, typed
  `Any`, since the compiler cannot read a remote file
  ([Load a library from a CDN](library-from-a-cdn.md)).
- **A local module** (`external X from "./x.js"`, a file in `public/`)
  becomes a plain script at the same path under `src/`, with its exports
  removed and its signatures as JSDoc. `X.f(…)` is rewritten to `f(…)`
  ([Call your own JavaScript](call-your-own-script.md)).
- **An `external element`** call site becomes `Element("tag-name", …)`,
  with its arguments and handlers as they were
  ([Place a custom element from a library](place-a-custom-element.md)).

The step lists every `.js` file already under `src/`, because each now
ships on every page. It runs before `wf migrate`'s other steps, since a
file with an `external` in it does not parse.

## Troubleshooting

| What `wf migrate` says | What to do |
|---|---|
| A module imported a file that is not in `public/` | Put its code in a `.js` file under `src/` yourself |
| A module imports other modules, so it stays in `public/` | Write a script under `src/` that does what it did, and call that |
| A module had an `export default` | It is kept as a plain statement, which names nothing; give the value a name if a page calls it |
