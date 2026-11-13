---
chapter: 25
kind: "recipe"
route: "guide/realtime"
group: "data"
blurb: "Hold a connection open — a socket, server-sent events, a channel every tab hears, a line to another reader's page, a worker — and read the network itself."
description: "WebSockets, server-sent events, broadcast channels, WebRTC peers and workers: opening, matching, sending, options, reconnection and typed messages."
---
# 25. Real-time

This chapter shows you how to keep a page in touch with the world while it
is open: a chat that updates as messages arrive, prices pushed from a
server, a cart that stays in step across tabs, two readers talking page to
page, and heavy work moved off the page's thread. Every connection is
declared like a `resource`, read with a `match`, and closed for you when the
page that opened it leaves, so a route change can never leak one.

In this chapter you will:

- open a WebSocket and send and receive typed messages;
- listen to server-sent events;
- talk to the site's other tabs;
- connect two readers' pages directly over WebRTC;
- run a script in a worker beside the page;
- send a last message as the page unloads;
- show what the network is doing.

## Before you start

You should know how a `resource` is read with `match` and its arms
([Fetching data](../learn/15-data.md#show-a-request-with-resource)), and how `state` and `effect` work
([State and reactivity](../learn/07-state-and-reactivity.md)). A connection is read
the same way a resource is.

There are five kinds of connection. Pick the one that fits the job:

| Declaration | Opened with | Use it for |
|---|---|---|
| `socket` | `ws(url, …)` | Two-way messages with a server: chat, live editing, notifications |
| `stream` | `sse(url, …)` | One-way messages from a server: prices, progress, a log |
| `channel` | `broadcast(name)` | Messages between tabs of your site in one browser |
| `peer` | `rtc(signal:, …)` | A direct line to another reader's page, with no server in the middle |
| `worker` | `spawn(url, …)` | A script of your site running beside the page, off its thread |

## Open a socket

To hold a two-way connection to a server, declare a `socket` with `ws(…)`
and `match` on it. This chat page shows a spinner while the socket
connects, the messages once it is open, and why it stopped when it closes:

```wf
page Chat(path: "/chat", title: "Chat", description: "Talk to the team.") {
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

The page opens the socket when it is drawn and closes it when the reader
leaves. Inside the socket's block, `on message(m) { }` handles each message
that arrives and `on open { }` runs each time the line opens — the first
time and after every reconnection.

A socket looks after itself:

- **It reconnects.** When the line drops, it waits and tries again, doubling
  the wait each time up to a limit. A server that closes the line normally
  (code `1000`) is not reconnected to.
- **It keeps itself alive.** With `heartbeat:`, it sends a ping that often
  and treats a missing answer as a dead line, which it closes and reopens.
- **It holds what you send while it is down.** `chat.send(…)` while the
  socket is reconnecting queues the message and sends it when the line
  opens again.
- **It reads JSON.** A text message that parses as JSON arrives as a value;
  any other text arrives as a string. A value you send is sent as JSON,
  unless it is already a string.

A `match` on a socket takes the arms `connecting`, `open`, `closed(c)` and
`error(e)`, and an `else` for the rest. `closed(c)` is handed the close,
with `c.code` and `c.reason`; `error(e)` is handed the failure, with
`e.message`.

### Tune a socket

Pass options after the address to change how a socket connects and
reconnects:

| Option | Does | Default |
|---|---|---|
| `protocols:` | The subprotocols to ask for: `["v2"]` | none |
| `heartbeat:` | Sends a ping this often, and reconnects when no pong comes back: `20.seconds` | off |
| `ping:`, `pong:` | The heartbeat's messages | `"ping"`, `"pong"` |
| `reconnect:` | `false` to stay closed after the server closes the line | `true` |
| `delay:`, `max:` | The first wait before reconnecting, in milliseconds, and the longest; the wait doubles each time | `500`, `30000` |
| `queue:` | `false` to drop, rather than hold, what is sent while the socket is down | `true` |
| `binaryType:` | `"blob"` or `"arraybuffer"` | `"blob"` |
| `keep:` | How many messages `.messages` holds, the newest; the oldest go first | `500` |

### Type the messages

To have the compiler check what you send and type what you receive, name a
type for each direction inside the socket's block with `send` and
`receive`:

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
`Outgoing`, and each message in `chat.messages` is an `Incoming`, so
`m.text` is a `String` and `m.txet` is an error.

## Listen to server-sent events

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

## Talk to the site's other tabs

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

## Connect two readers' pages

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

## Read a connection's state and messages

Every connection is a *handle*: a value whose members follow the
connection. A `match` reads its state, and text, an `if` or an `effect`
reads what arrived:

| | `socket` | `stream` | `channel` | `peer` |
|---|---|---|---|---|
| `.state` | `connecting`, `open`, `closed`, `error` — the arms of a `match` | the same | `open` or `closed` | the same as a socket |
| `.messages` | the messages, in order: the last 500, or `keep:` | the same | — | the same |
| `.last(kind)` | the last message, or the last whose `type` is `kind`, among those `.messages` holds | the last under an event's name | `.last`: the last posted | as a socket |
| `.error` | the failure, which the `error(e)` arm is handed | the same | — | the same |
| `.closure` | the close, which `closed(c)` is handed: `.code`, `.reason` | — | — | the same |
| Sending | `.send(value)` — queued while the line is down | — | `.post(value)` | `.send(value)` — queued until it opens |
| `.close()` | closes it early | the same | the same | the same |
| `.signal(m)` | — | — | — | hands it what the other side signalled |

A connection left open all day would otherwise hold every message it was
ever sent, so `.messages` keeps the newest 500 and lets the oldest go.
`keep:` on the connection says another number — `ws(url, keep: 50)`,
`sse(url, keep: 2000)` — and `keep: 0` keeps none, for a connection whose
`on message` handler stores what it needs itself.

A store may hold any of these too. It opens the connection when the store
is first read and closes it with the store; a page reads it as
`Lang.css.state`, and the store's actions read it by its name.

## Run a script in a worker

To keep slow work — a language service, a parser, anything that would make
typing stutter — off the page's own thread, declare a `worker` with
`spawn(…)`. A worker runs a script of your site beside the page and is held
like a socket: owned by what opened it, and stopped when that leaves. A
store holds one for the whole app:

```wf
store Lang {
    state problems: [Map] = []
    state restarts = 0
    worker css = spawn("/workers/css.js", restart: 1, timeout: 5000) {
        on message(m) { if m.type == "diagnostics" { problems = m.items } }
        on restart(e) { restarts = restarts + 1 }
    }
    action complete(text: String, at: Number) {
        return await css.request("complete", { text: text, at: at }, { supersede: true })
    }
    action check(text: String) { css.send({ method: "check", params: { text: text } }) }
}

page Editor(path: "/", title: "Editor", description: "A worker beside the page.") {
    Heading("Editor").h1
    match Lang.css {
        open     { Text(if Lang.css.pending > 0 { "Checking…" } else { "Checked" }) }
        error(e) { Alert("The CSS service stopped: {e.message}").danger }
        else     { Text("Stopped") }
    }
    Text("{Lang.problems.length} problems, {Lang.restarts} restarts")
    Button("Complete") { on click { Lang.complete("a { col", 7) } }
    Button("Check") { on click { Lang.check("a { col") } }
}
```

The worker's side is a plain script under `public/` — the one place,
besides a project script, where you write JavaScript by hand:

```js
// public/workers/css.js
self.onmessage = ({ data }) => {
  const { id, method, params, cancel } = data;
  if (cancel !== undefined) return;                       // a newer request took over
  if (method === "complete") self.postMessage({ id, result: complete(params) });
  if (method === "check") self.postMessage({ type: "diagnostics", items: check(params.text) });
};
```

The page and the worker talk in three ways:

- **Ask and wait: `.request(method, params)`.** It posts
  `{ id, method, params }` and returns a promise of what the worker posts
  back as `{ id, result }`. A reply of `{ id, error }` rejects it, with the
  error's `message` and its `data`. A request not answered within
  `timeout:` fails — 10 seconds when unset, or `{ timeout }` on the
  request. With `{ supersede: true }`, a newer request of the same method
  fails the older one, and the worker is posted `{ cancel: id }`.
- **Tell: `.send(value)`.** It posts the value and expects nothing.
- **Listen: messages with no `id`.** What the worker posts without an `id`
  is a message: `on message(m)` hears it, and `.last(kind)` is the last of
  a kind, by its `type` or its `method`.

`.pending` is how many requests are out. `.state` is `open`, `error` (a
`match`'s `error(e)` arm) or `closed`.

A worker that crashes — an error nothing in it caught — fails every request
that was out and is started again, up to `restart:` times, running
`on restart(e)` each time. Past that it is `error`, and `on error(e)` runs.
`.restart()` starts it again by hand, and `.close()` stops it.

Pass `type: .module` to start a module worker. The script is a file of the
site, so it is asked for under the base path, and a name `public/` does not
have is a `D01`.

## Send a message as the page unloads

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

## Show what the network is doing

To change what a page does on a slow line or none at all, read `network`.
Like `viewport`, it is a browser value that keeps itself current:

```wf
page Inbox(path: "/inbox", title: "Inbox", description: "Your messages.") {
    Heading("Inbox").h1
    if !network.online { Alert("You are offline — changes are queued.").warning }
    if network.saveData { Text("Data saver is on.").muted }
}
```

| Member | What it is |
|---|---|
| `network.online` | Whether the browser has a connection |
| `network.effectiveType` | How fast it is: `4g`, `3g`, … |
| `network.saveData` | Whether the reader asked to save data |
| `network.downlink` | The estimated bandwidth, in megabits per second |
| `network.queued` | How many writes wait for the connection, when the site works offline ([Offline](offline.md#keep-writes-made-offline)) |

## Troubleshooting

- **The socket never leaves `connecting`.** The server is not answering at
  that address. Check the scheme: a page served over HTTPS must use
  `wss://`, not `ws://`.
- **The socket closes and does not come back.** The server closed it with
  code `1000`, or you set `reconnect: false`. A normal close is final; any
  other close reconnects.
- **The socket reconnects every few seconds.** With `heartbeat:` set, the
  server must answer each `ping:` with the `pong:` text. A server that does
  not answer looks dead, and the socket reopens.
- **`stream.last("name")` is always null.** The event name is not in
  `events:`. Only the names listed there are listened to.
- **The other tab does not hear a `channel` post.** A tab does not hear its
  own posts, and both tabs must be on the same origin and use the same
  channel name.
- **Two peers on different networks never open.** Give `ice:` a STUN server.
  With none, only readers on the same network can connect.
- **A worker request fails after 10 seconds.** The worker did not post back
  an answer with the request's `id`. Raise `timeout:` for slow work, or
  check the script answers every method it is sent.

## Next

- [Offline](offline.md): keep the site working when the network is gone.
- [Fetching data](../learn/15-data.md): fetch and cache what a page reads.
