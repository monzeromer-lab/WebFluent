---
kind: "recipe"
route: "/recipes/talk-between-tabs"
group: "realtime"
description: "Keep a site's open tabs in step — a cart, a sign-in, a setting — with a `channel` opened by `broadcast(name)`, which every other tab hears."
from: ["/docs/guide/realtime#talk-to-the-site-s-other-tabs"]
outputs: ["website", "app"]
---
# Talk to the site's other tabs

A `channel` keeps tabs of the same site in step — a cart, a sign-in, a
setting. Every tab that opens a channel of the same name hears what the
others post, with no server involved. This shop keeps its cart count the
same in every tab:

```wf
store Cart {
    state count = 0
    action add() { count = count + 1 }
    action merge(m: Map) { count = m.items }
}

page Shop(path: "/", title: "Shop", description: "A cart every tab agrees on.") {
    use Cart
    channel cart = broadcast("cart") { on message(m) { Cart.merge(m) } }

    Heading("Shop").h1
    Text("{Cart.count} in the cart")
    Button("Add to cart") { on click { Cart.add()  cart.post({ items: Cart.count }) } }
}
```

## How it works

- **`channel cart = broadcast("cart")`** joins the channel named `cart`. A
  channel is a page's for as long as the page shows.
- **`cart.post(value)`** sends a value to every other tab on the channel.
  Here the button adds to the cart, then posts the new count.
- **`on message(m)`**, in the channel's block, hears what another tab
  posted, and hands it to the store's `merge` action.

Open the page in two tabs and add to the cart in one: the other tab's count
follows. A tab does not hear its own posts, and both tabs must be on the
same origin. `cart.last` is the last message another tab posted. A
channel's state is `open`, or `closed` in a browser without
`BroadcastChannel`.
