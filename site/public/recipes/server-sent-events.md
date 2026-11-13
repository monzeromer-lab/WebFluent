# Listen to server-sent events

> Receive a one-way flow of events from a server — prices, progress, a log — with a `stream` opened by `sse(…)`, and show the latest of each kind.

The page: https://webfluent.monzeromer.dev/recipes/server-sent-events

A `stream` receives a one-way flow of events from a server — prices,
progress, a log — over server-sent events. Use it when the server talks and
the page only listens; for two-way messages, [open a socket](https://webfluent.monzeromer.dev/recipes/open-a-socket.md).
This page shows the latest price as it arrives:

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

## How it works

- **`stream ticks = sse("/events", events: ["price"])`** opens the stream
  when the page is drawn and closes it when the reader leaves. `events:`
  names the event types to listen for; only those names are listened to.
- **`ticks.last("price")`** is the latest event of that name. Each event
  name is kept apart, so it is the last price and not the last event of any
  kind; `ticks.last()` is the last unnamed message. The `effect` copies it
  into `price` each time a new one arrives.
- **`match ticks`** reads the stream's state, with the same arms as a
  socket: `connecting`, `open`, `closed(c)`, `error(e)`, or an `else`.

`ticks.messages` holds the messages in order — the last 500, or as many as
`keep:` says. A message whose data parses as JSON arrives as a value.

The browser reconnects a stream on its own and resumes from the last event
id the server sent. Pass `credentials: true` to send the site's cookies
with the request.
