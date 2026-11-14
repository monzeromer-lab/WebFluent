---
kind: "recipe"
route: "/recipes/send-on-unload"
group: "data"
description: "Record that something happened even as the reader leaves — an analytics event, a \"reader left\" signal — with `beacon(url, data)`."
from: ["/docs/guide/realtime#send-a-message-as-the-page-unloads"]
outputs: ["website", "app"]
---
# Send a message as the page unloads

`beacon(url, data)` records that something happened even when the reader
is leaving — an analytics event, a "reader left" signal. A `fetch` started
as the page unloads is cancelled; a beacon is delivered. This checkout page
sends one when the reader leaves it, whether they close the tab, go to
another site or switch away:

```wf
page Checkout(path: "/checkout", title: "Checkout", description: "Pay for the order.") {
    state paid = false

    on visibilitychange {
        if document.visibilityState == "hidden" {
            beacon("/analytics", { event: "left-checkout", paid: paid })
        }
    }

    Heading("Checkout").h1
    Button("Pay").primary { on click { paid = true } }
}
```

## How it works

`beacon` posts a small body with `navigator.sendBeacon`, which the browser
delivers even as the page unloads, where a `fetch` would be cancelled. It
returns nothing to wait for: the page goes on, and so may the reader.

`on visibilitychange`, written at the top of the page, listens on the
document for as long as the page shows. The page turns `hidden` whenever
the reader leaves it: closing the tab, following a link to another site,
switching tabs or apps, or locking the phone. That is the last moment a
page can count on, and the one browsers recommend for this; `unload` and
`beforeunload` often never fire on a phone. Because a hidden page may come
back, the beacon can be sent more than once for one visit: count visits on
the server, or keep a `state sent = false` and send only the first time.
Going to another page of the same site does not hide the document, so it
sends nothing; the listener stops with the page.

### Send on a click instead

To record a step the reader takes, send the beacon from the handler. The
request still arrives if the click also takes the reader away:

```wf
page Pay(path: "/pay", title: "Pay", description: "Pay for the order.") {
    Heading("Pay").h1
    Button("Pay").primary { on click { beacon("/analytics", { event: "checkout", value: 42 }) } }
}
```
