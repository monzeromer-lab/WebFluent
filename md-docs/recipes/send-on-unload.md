---
kind: "recipe"
route: "/recipes/send-on-unload"
group: "realtime"
description: "To record that something happened even when the reader is leaving — an analytics event, a \"reader left\" signal — call `beacon(url, data)`:"
from: ["/docs/guide/realtime#send-a-message-as-the-page-unloads"]
---
# Send a message as the page unloads

To record that something happened even when the reader is leaving — an
analytics event, a "reader left" signal — call `beacon(url, data)`:

```wf
page Checkout(path: "/checkout", title: "Checkout", description: "Pay for the order.") {
    Heading("Checkout").h1
    Button("Pay").primary { on click { beacon("/analytics", { event: "checkout", value: 42 }) } }
}
```

`beacon` posts a small body with `navigator.sendBeacon`, which the browser
delivers even as the page unloads, where a `fetch` would be cancelled.
