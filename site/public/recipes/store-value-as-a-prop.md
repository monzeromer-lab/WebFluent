# Pass a store's value to a component

> Pass one value from a store to a component as a prop, so the component stays reusable outside that store.

The page: https://webfluent.monzeromer.dev/recipes/store-value-as-a-prop

A component that needs only one value from a store can take it as a prop
instead of using the store. That keeps the component reusable outside that
store: a test or another page can hand it any number:

**Needs:** nothing besides the code. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
store Cart { state count = 0 }

component CartBadge(count: Number) {
    if count > 0 { Badge("{count}").primary }
}

page Shop(path: "/", title: "Shop", description: "Everything we sell.") {
    use Cart
    Heading("Shop").h1
    Row(gap: .sm, align: .center) {
        Button("Add to cart") { on click { Cart.count = Cart.count + 1 } }
        CartBadge(count: Cart.count)
    }
}
```

No badge shows while the cart is empty. Press **Add to cart** and a badge
reading 1 appears beside the button, and counts up with every press.

## How it works

- `CartBadge` declares a `count` [prop](https://webfluent.monzeromer.dev/learn/10-components.md#props) and
  knows nothing about `Cart`.
- The page reads the [store](https://webfluent.monzeromer.dev/learn/11-stores.md#a-first-store) after
  `use Cart` and passes `Cart.count` down.
- A prop that reads state stays live inside the component, so the badge
  repaints when `Cart.count` changes. The button changes it here; in a shop
  of your own, an action of the store would.
