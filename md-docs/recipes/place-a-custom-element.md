---
kind: "recipe"
route: "/recipes/place-a-custom-element"
group: "interop"
description: "Place a custom element a library defines, such as Stripe's pricing table, with Element: its tag, attributes that follow state, and its events."
outputs: ["website", "app"]
from: ["/docs/guide/interop#place-a-custom-element-from-a-library"]
---
# Place a custom element from a library

To place a custom element that a library defines, use `Element` with the
element's tag. This page places Stripe's pricing table, and shows a
spinner until the table says it is ready:

```wf
// Your publishable key, from the Stripe dashboard.
const STRIPE_KEY = "pk_test_123"

page Pricing(path: "/pricing", title: "Pricing", description: "Plans.") {
    state loaded = false

    Heading("Pricing").h1
    if !loaded { Spinner }
    Element("stripe-pricing-table", publishableKey: STRIPE_KEY, pricing-table-id: "prctbl_1") {
        on ready { loaded = true }
    }
}
```

The script that defines the element loads from `meta.scripts` in
`webfluent.app.json`:

```json
{ "meta": {
    "scripts": [{ "src": "https://js.stripe.com/v3/pricing-table.js", "async": true }]
} }
```

## How it works

- The tag comes first. It is lower case, with a hyphen, as every custom
  element's tag is; any other tag is an `E116` error.
- A named argument is an attribute: `publishableKey` is written
  `publishable-key`. An attribute follows state like any other value.
- An `on` handler hears the events the element fires, whatever their names.
- The block is the element's children.
- `class:`, `style { }` and `mount:` work as on any other element.

The script that defines the element (your own, or one in `meta.scripts`)
upgrades it when it runs. Nothing on the page waits for this one, so it
loads `async` ([Load a library from a CDN](library-from-a-cdn.md#variations)).

## Troubleshooting

| What you see | What to do |
|---|---|
| `E116` on an `Element` | Write the tag as a custom element's: lower case, with a hyphen |
| The element stays empty | The script that defines it has not run: check its entry in `meta.scripts` |
| The element is blocked in the console | It loads more than its script, such as a frame from its own origin; add that origin to `meta.frame` or `meta.connect` |
