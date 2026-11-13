# Send a message as the page unloads

> Record that something happened even as the reader leaves — an analytics event, a "reader left" signal — with `beacon(url, data)`.

The page: https://webfluent.monzeromer.dev/recipes/send-on-unload

`beacon(url, data)` records that something happened even when the reader
is leaving — an analytics event, a "reader left" signal. A `fetch` started
as the page unloads is cancelled; a beacon is delivered. This checkout page
sends one when the reader pays:

```wf
page Checkout(path: "/checkout", title: "Checkout", description: "Pay for the order.") {
    Heading("Checkout").h1
    Button("Pay").primary { on click { beacon("/analytics", { event: "checkout", value: 42 }) } }
}
```

## How it works

`beacon` posts a small body with `navigator.sendBeacon`, which the browser
delivers even as the page unloads, where a `fetch` would be cancelled. It
returns nothing to wait for: the page goes on, and so may the reader.
