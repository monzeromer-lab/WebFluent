# Connect two readers' pages

> Send messages straight from one reader's page to another's over a WebRTC data channel, with a `peer` opened by `rtc(…)` and no server relaying them.

The page: https://webfluent.monzeromer.dev/recipes/connect-two-readers

A `peer` opens a WebRTC data channel straight from one reader's page to
another's, so messages travel between them with no server relaying them.
This page signals over a `broadcast` channel, so it connects two tabs of one
browser: open `/room?host=1` in one tab and `/room` in another.

**Needs:** nothing besides the page; open it in two tabs of one browser.

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

## How it works

The language supplies the channel, not a server. Before two pages can
connect, they must tell each other how — an offer, an answer, and the
network routes each can be reached by. This exchange is called
*signalling*.

- **`signal: m => lobby.post(m)`** sends this side's signalling messages out,
  over whatever connection the app already has — here the `lobby`
  [channel](https://webfluent.monzeromer.dev/recipes/talk-between-tabs.md).
- **`link.signal(m)`**, in the lobby's `on message`, hands the peer what the
  other side sent.
- **`initiator: query.host == "1"`** makes the tab opened with `?host=1` the
  side that makes the offer. Exactly one side does.
- **`match link`** reads the peer like a socket: `connecting`, `open`,
  `closed(c)`, `error(e)`.
- **`link.send(value)`** sends a message, and holds it until the channel
  opens; **`on message(m)`** in the peer's block hears the other side's.

When a page leaves, it closes its peer, and the other side reads `closed`
at once rather than when the connection times out.

| Option | Does |
|---|---|
| `signal:` | Required. A function handed each message the other side must receive. |
| `initiator:` | Whether this side makes the offer. Exactly one side does. |
| `ice:` | The servers that help two readers on different networks find each other: `[{ urls: "stun:…" }]`. None by default, which connects readers on the same network only. |
| `ordered:` | `false` for a channel that may deliver messages out of order. |

## Variations

### Connect readers on different machines

Between two readers on different machines, the signal rides the app's own
socket or `api` instead of a `broadcast` channel, and `ice:` names a STUN
server so readers behind different networks can find each other:

```wf
page Call(path: "/call", title: "Call", description: "Two readers, two machines.") {
    state heard = ""
    socket relay = ws("wss://example.com/signal") { on message(m) { link.signal(m) } }
    peer link = rtc(signal: m => relay.send(m), initiator: query.host == "1",
                    ice: [{ urls: "stun:stun.example.com:3478" }]) {
        on message(m) { heard = m.text }
    }

    Heading("Call").h1
    match link {
        open { Button("Wave") { on click { link.send({ text: "hello" }) } } }
        else { Text("Connecting…").muted }
    }
    Text(heard)
}
```
