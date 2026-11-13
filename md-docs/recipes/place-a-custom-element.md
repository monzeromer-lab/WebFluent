---
kind: "recipe"
route: "/recipes/place-a-custom-element"
group: "interop"
description: "To place a custom element that a library defines, use `Element` with the element's tag."
from: ["/docs/guide/interop#place-a-custom-element-from-a-library"]
---
# Place a custom element from a library

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
