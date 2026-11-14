# Open a socket

> Hold a two-way connection to a server with a `socket`: it reconnects, keeps itself alive, and queues what you send while the line is down.

The page: https://webfluent.monzeromer.dev/recipes/open-a-socket

A `socket` holds a two-way connection to a server: chat, live editing,
notifications. You declare it with `ws(…)` and read it with `match`, the
way you read a `resource`. This chat page shows a spinner while the socket
connects, the messages once it is open, and why it stopped when it closes:

**Needs:** a WebSocket server; change `wss://example.com/chat` to yours.

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

## How it works

A connection is read the same way a resource is, so this recipe assumes you
know how a `resource` is read with `match` and its arms
([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md#show-a-request-with-resource)), and how
`state` and `effect` work
([State and reactivity](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md)).

- **`socket chat = ws(…)`** opens the socket when the page is drawn and
  closes it when the reader leaves.
- **`on message(m) { }`**, inside the socket's block, handles each message
  that arrives. `on open { }` runs each time the line opens — the first
  time and after every reconnection.
- **`match chat`** takes the arms `connecting`, `open`, `closed(c)` and
  `error(e)`, and an `else` for the rest. `closed(c)` is handed the close,
  with `c.code` and `c.reason`; `error(e)` is handed the failure, with
  `e.message`.
- **`chat.messages`** holds what arrived, in order, and
  **`chat.send(…)`** sends a value.

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

## Variations

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

## Choose a kind of connection

A socket is one of five kinds of connection. Pick the one that fits the job:

| Declaration | Opened with | Use it for |
|---|---|---|
| `socket` | `ws(url, …)` | Two-way messages with a server: chat, live editing, notifications |
| `stream` | `sse(url, …)` | One-way messages from a server: prices, progress, a log |
| `channel` | `broadcast(name)` | Messages between tabs of your site in one browser |
| `peer` | `rtc(signal:, …)` | A direct line to another reader's page, with no server in the middle |
| `worker` | `spawn(url, …)` | A script of your site running beside the page, off its thread |

## Read a connection's state and messages

Every connection is a *handle*: a value whose members follow the
connection. A `match` reads its state, and text, an `if` or an `effect`
reads what arrived:

| Member | `socket` | `stream` | `channel` | `peer` |
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
