---
kind: "recipe"
route: "/recipes/talk-between-tabs"
group: "realtime"
description: "To keep tabs of the same site in step — a cart, a sign-in, a setting — declare a `channel` with `broadcast(name)`."
from: ["/docs/guide/realtime#talk-to-the-site-s-other-tabs"]
---
# Talk to the site's other tabs

To keep tabs of the same site in step — a cart, a sign-in, a setting —
declare a `channel` with `broadcast(name)`. Every tab that opens a channel
of that name hears what the others post with `.post(value)`:

```wf
store Cart {
    state count = 0
    action add() { count = count + 1 }
    action merge(m: Map) { count = m.items }
}

page Shop(path: "/", title: "Shop", description: "A cart every tab agrees on.") {
    channel cart = broadcast("cart") { on message(m) { Cart.merge(m) } }

    Heading("Shop").h1
    …
    Text("{Cart.count} in the cart")
    Button("Add to cart") { on click { Cart.add()  cart.post({ items: Cart.count }) } }
}
```

Open the page in two tabs and add to the cart in one: the other tab's count
follows. A tab does not hear its own posts. `cart.last` is the last message
another tab posted. A channel's state is `open`, or `closed` in a browser
without `BroadcastChannel`.
