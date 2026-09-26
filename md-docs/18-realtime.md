# 18. Real-time

<!--
route: guide/realtime
group: building
blurb: Sockets, server-sent events, a channel every tab hears, a direct line to another reader's page — and the network itself as a value.
description: WebSockets, server-sent events, broadcast channels and WebRTC peers: opening, matching, sending, options, reconnection and typed messages.
-->

A page can hold a connection open: a **socket** to a server, a **stream** of
server-sent events, a **channel** every tab of the site hears, or a **peer**
— a WebRTC data channel straight to another reader's page. Each is declared
like a `resource`, matched on its state, and closed when the page that
opened it leaves, so a route change can never leak one.

## Four kinds of connection

A socket, a stream of server-sent events, a channel every tab of the
origin hears, or a peer — a line straight to another reader's page. Each
is closed when the page that opened it leaves.

```wf
page Chat(path: "/chat", title: "Chat", description: "Talk.") {
    state draft = ""
    socket chat = ws("wss://example.com/chat", heartbeat: 20.seconds) {
        on message(m) { log(m) }
    }

    Heading("Chat").h1
    match chat {
        connecting { Spinner.sm }
        open       { for m in chat.messages by m.id { Text(m.text) } }
        closed(c)  { Alert("Disconnected ({c.code})").warning }
        error(e)   { Alert(e.message).danger }
    }
    Input(bind: draft, label: "Message").text
    Button("Send").primary { on click { chat.send({ text: draft })  draft = "" } }
}
```

A socket reconnects with backoff, keeps itself alive with a heartbeat, and
holds what was sent while it was down.

```wf
stream ticks = sse("/events", events: ["price"])
channel cart = broadcast("cart") { on message(m) { Cart.merge(m) } }
beacon("/analytics", { event: "checkout" })     // survives the page unloading
```

`ticks` and `cart` are handles too — a `match` reads the state, and an
`effect` reads what arrived:

```wf
effect { if let p = ticks.last("price") { price = p } }
Button("Sync") { on click { cart.post({ items: Cart.count }) } }
```

What each handle holds:

| | `socket` | `stream` | `channel` | `peer` |
|---|---|---|---|---|
| `.state` | `connecting` `open` `closed` `error` — the arms of a `match` | the same | `open` or `closed` | the same as a socket |
| `.messages` | every message, in order | every message, in order | — | every message, in order |
| `.last(kind)` | the last message, or the last of a kind | the last under an event's name | the last posted | as a socket |
| `.error` | the failure, which the `error(e)` arm is handed | the same | — | the same |
| `.closure` | the close, which `closed(c)` is handed: `.code`, `.reason` | — | — | the same |
| Sending | `.send(value)` — queued while the line is down | — | `.post(value)` | `.send(value)` — queued until it opens |
| `.close()` | closes it early | the same | the same | the same |
| `.signal(m)` | — | — | — | hands it what the other side signalled |

A `match` over one takes `connecting`, `open`, `closed(c)` and `error(e)`,
and an `else` for the rest. The page closing closes the connection, so a
route change cannot leak one.

## A peer

`peer` is a WebRTC data channel to another page. The language supplies the
channel, not a server: what the two sides must tell each other to connect —
an offer, an answer, the routes each can be reached by — goes out through
`signal:`, over whatever the app already has, and what the other side sent
is handed back to `link.signal(m)`. One side is the `initiator`.

```wf
page Room(path: "/room", title: "Room", description: "Two readers, one line.") {
    state heard = ""
    channel lobby = broadcast("room") { on message(m) { link.signal(m) } }
    peer link = rtc(signal: m => lobby.post(m), initiator: query.host == "1") {
        on message(m) { heard = m.text }
    }

    Heading("Room").h1
    match link {
        connecting { Text("Waiting for the other side…").muted }
        open       { Button("Wave") { on click { link.send({ text: "hello" }) } } }
        closed(c)  { Text("They left.") }
        error(e)   { Alert(e.message).danger }
    }
    Text(heard)
}
```

A `broadcast` channel signals between two tabs of one browser, as here;
between two readers the signal rides the app's own socket or `api`.

| Option | |
|---|---|
| `signal:` | Required: a function handed each message the other side must receive. |
| `initiator:` | Whether this side offers. Exactly one side does. |
| `ice:` | The servers that help two readers on different networks find each other — `[{ urls: "stun:…" }]`. None by default, which connects readers on the same network only. |
| `ordered:` | `false` for a channel that may deliver out of order. |

A page that leaves closes its peer, and the other side reads `closed` at
once rather than when the connection times out.

## Options

A socket takes, after its address:

| Option | Does | Default |
|---|---|---|
| `protocols:` | Subprotocols to ask for: `["v2"]` | none |
| `heartbeat:` | Sends a ping this often and reconnects when no pong comes back: `20.seconds` | off |
| `ping:`, `pong:` | The heartbeat's messages | `"ping"`, `"pong"` |
| `reconnect:` | `false` to stay closed after the server closes | `true` |
| `delay:`, `max:` | The first wait before reconnecting, in ms, and the longest; it doubles each time | `500`, `30000` |
| `queue:` | `false` to drop, rather than hold, what is sent while the socket is down | `true` |
| `binaryType:` | `"blob"` or `"arraybuffer"` | `"blob"` |

A stream takes `events:` — the event names to listen for, each kept apart so
`ticks.last("price")` is the last price — and `credentials: true` to send
cookies. A stream reconnects on its own and resumes from the last event id
the server sent.

Inside the block, `on message(m) { }` handles what arrives and `on open { }`
runs when the line opens. A socket may also say what it sends and receives:

```wf
type Incoming { id: String, text: String }
type Outgoing { text: String }

page Room(path: "/room", title: "Room", description: "A typed conversation.") {
    state draft = ""
    socket chat = ws("wss://example.com/chat", heartbeat: 20.seconds) {
        send    Outgoing
        receive Incoming
    }
    Heading("Room").h1
    for m in chat.messages by m.id { Text(m.text) }
    Input(bind: draft, label: "Message")
    Button("Send") { on click { chat.send(Outgoing(text: draft))  draft = "" } }
}
```

With `send` and `receive` declared, `chat.send(…)` is checked against
`Outgoing` and each message is an `Incoming`.

## A message that must arrive: `beacon`

```wf
beacon("/analytics", { event: "checkout", value: 42 })
```

`beacon(url, data)` posts a small body with `navigator.sendBeacon`, which the
browser delivers even as the page unloads — for analytics and "the reader
left" signals, where a `fetch` would be cancelled.

## The network as a value

```wf
if !network.online { Alert("You are offline — changes are queued.").warning }
```

`network.online`, `.effectiveType` (`4g`, `3g`, …), `.saveData`, `.downlink`
— live, so a page can say what it does on a slow line or none at all — and
`.queued`, the writes waiting for the connection when the site works offline.


## Next

[Offline](19-offline.md).
