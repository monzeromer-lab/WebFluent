---
kind: "recipe"
route: "/recipes/server-sent-events"
group: "realtime"
description: "To receive a one-way flow of events from a server, declare a `stream` with `sse(…)`."
from: ["/docs/guide/realtime#listen-to-server-sent-events"]
---
# Listen to server-sent events

To receive a one-way flow of events from a server, declare a `stream` with
`sse(…)`. `events:` names the event types to listen for, and
`ticks.last("price")` is the latest event of that name:

```wf
page Prices(path: "/prices", title: "Prices", description: "Live prices.") {
    state price = 0
    stream ticks = sse("/events", events: ["price"])
    effect { if let p = ticks.last("price") { price = p } }

    Heading("Prices").h1
    match ticks {
        open { Text("Price: {price}") }
        else { Text("Connecting…").muted }
    }
}
```

Each event name is kept apart, so `ticks.last("price")` is the last price
and not the last event of any kind; `ticks.last()` is the last unnamed
message. `ticks.messages` holds the messages in order — the last 500, or
as many as `keep:` says. A message whose data parses as JSON arrives as a
value.

The browser reconnects a stream on its own and resumes from the last event
id the server sent. Pass `credentials: true` to send the site's cookies
with the request.
