---
kind: "recipe"
route: "/recipes/store-value-as-a-prop"
group: "interaction"
description: "A component that needs only one value from a store can take it as a prop instead of using the store."
from: ["/docs/guide/recipes#pass-a-store-s-value-to-a-component-as-a-prop"]
---
# Pass a store's value to a component

A component that needs only one value from a store can take it as a prop
instead of using the store. That keeps the component reusable outside that
store:

```wf
store Cart { state count = 0 }

component CartBadge(count: Number) {
    if count > 0 { Badge("{count}").primary }
}

page Shop(path: "/", title: "Shop", description: "Everything we sell.") {
    use Cart
    Heading("Shop").h1
    CartBadge(count: Cart.count)
}
```
