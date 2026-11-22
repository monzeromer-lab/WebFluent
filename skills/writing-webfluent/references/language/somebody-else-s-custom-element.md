### Somebody else's custom element

```wf
page Pricing(path: "/pricing", title: "Pricing", description: "Plans.") {
    state loaded = false
    Heading("Pricing").h1
    Element("stripe-pricing-table", publishableKey: "pk_live_…") {   // the attribute `publishable-key`
        on ready { loaded = true }                                   // any event it fires
    }
}
```

`Element` takes the tag first (lower case, with a hyphen); every named
argument is an attribute and follows state; `class:`, `style { }` and
`mount:` work as anywhere. `external` — 4.1's typed declaration of a module
or element — is gone; `wf migrate` rewrites it.

