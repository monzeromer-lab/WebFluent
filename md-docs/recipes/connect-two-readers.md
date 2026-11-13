---
kind: "recipe"
route: "/recipes/connect-two-readers"
group: "realtime"
description: "To send messages straight from one reader's page to another's, with no server relaying them, declare a `peer` with `rtc(…)`."
from: ["/docs/guide/realtime#connect-two-readers-pages"]
---
# Connect two readers' pages

To send messages straight from one reader's page to another's, with no
server relaying them, declare a `peer` with `rtc(…)`. It opens a WebRTC
data channel between the two pages.

The language supplies the channel, not a server. Before two pages can
connect, they must tell each other how — an offer, an answer, and the
network routes each can be reached by. This exchange is called
*signalling*. Your page sends its side out through the `signal:` function,
over whatever connection the app already has, and hands what the other side
sent to `link.signal(m)`. One side is the `initiator`.

This page signals over a `broadcast` channel, so it connects two tabs of one
browser. Open `/room?host=1` in one tab and `/room` in another:

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

Between two readers on different machines, the signal rides the app's own
socket or `api` instead of a `broadcast` channel.

| Option | Does |
|---|---|
| `signal:` | Required. A function handed each message the other side must receive. |
| `initiator:` | Whether this side makes the offer. Exactly one side does. |
| `ice:` | The servers that help two readers on different networks find each other: `[{ urls: "stun:…" }]`. None by default, which connects readers on the same network only. |
| `ordered:` | `false` for a channel that may deliver messages out of order. |

A peer is matched like a socket. `link.send(value)` sends a message, and
holds it until the channel opens. When a page leaves, it closes its peer,
and the other side reads `closed` at once rather than when the connection
times out.
