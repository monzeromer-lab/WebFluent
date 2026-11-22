### A connection the page holds open

A socket, a stream of events, a channel every tab hears, or a peer — a
direct line to another reader's page. Each is closed when the page that
opened it leaves, so a route change cannot leak one.

```wf
socket chat = ws("wss://example.com/chat", protocols: ["v2"], heartbeat: 20.seconds) {
    send    Outgoing                  // typed both ways
    receive Incoming
    on message(m) { Feed.add(m) }
}

match chat {
    connecting { Spinner.sm }
    open       { for m in chat.messages by m.id { Bubble(m) } }
    closed(c)  { Alert("Disconnected ({c.code})").warning }
    error(e)   { Alert(e.message).danger }
}
Button("Send") { on click { chat.send(Outgoing(text: draft))  draft = "" } }

stream ticks = sse("/events", events: ["price", "status"])
effect { if let p = ticks.last("price") { price = p } }

channel cart = broadcast("cart") { on message(m) { Cart.merge(m) } }
Button("Sync") { on click { cart.post({ items: Cart.count }) } }

beacon("/analytics", { event: "checkout", value: total })   // survives unload
```

A socket reconnects with backoff, keeps itself alive with a heartbeat, and
holds what was sent while it was down. A stream resumes where it was.

Each is a handle, not a signal: `.state` (what a `match` reads),
`.messages` (the newest 500; `keep: n` on the connection says how many),
`.last(kind)`, `.error`, `.closure` (`.code`, `.reason`),
`.close()`, and `.send(v)` on a socket or a peer, `.post(v)` on a channel.
Each is typed by what it declares — `send Outgoing` is what `.send` takes,
`receive Incoming` what `.messages` holds and `on message(m)` is given —
and one that declares neither sends and receives `Json`, which is matched
or decoded rather than read as a record (`T28`).
A store may hold any of them too: opened when the store is first read,
closed with it, read as `Lang.css.state` and in its actions as `css`.

#### A worker

`worker` runs a script of `public/` beside the page — a language service, a
parser — held like a socket:

```wf
type Problem { line: Number, message: String }
type CssMessage { type: String, items: [Problem] = [] }

store Lang {
    state problems: [Problem] = []
    worker css = spawn("/workers/css.js", restart: 1, timeout: 5000) {
        receive CssMessage
        on message(m) { if m.type == "diagnostics" { problems = m.items } }
        on restart(e) { log("css restarted: {e.message}") }
    }
    action complete(text: String, at: Number) -> Json {
        return await css.request("complete", { text: text, at: at }, { supersede: true })
    }
}
```

`.request(method, params, { timeout, supersede })` posts `{ id, method,
params }` and is a promise of the `{ id, result }` the worker posts back
(or rejects with its `{ id, error }`, a timeout — 10 s unset — or a newer
request of the method with `supersede: true`, which also posts
`{ cancel: id }`). `.send(v)` posts and expects nothing. What it posts
with no `id` is a message: `on message`, `.last(kind)` by its `type` or
`method`. `.pending` counts what is out; `.state` is `open`, `error` or
`closed`. A crash fails what was out and starts it again `restart:` times
(`on restart(e)`), then is `error` (`on error(e)`); `.restart()` starts it
again by hand. `type: .module` for a module worker. The script is a file of
the site (under the base path); a missing one is a `D01`.

#### A peer

`peer` opens a WebRTC data channel straight to another page. The language
supplies the channel, not a server: what the two sides must tell each other
to connect — an offer, an answer, the routes each can be reached by — goes
out through `signal:`, over whatever the app already has, and what the other
side sent is handed to `link.signal(m)`. One side is the `initiator`.

```wf
type Wave { text: String }

page Room(path: "/room", title: "Room", description: "Two readers, one line.") {
    state heard = ""
    channel lobby = broadcast("room") { on message(m) { link.signal(m) } }
    peer link = rtc(signal: m => lobby.post(m), initiator: query.host == "1",
                    ice: [{ urls: "stun:stun.example.com:3478" }]) {
        send Wave
        receive Wave
        on message(m) { heard = m.text }
    }

    Heading("Room").h1
    match link {
        connecting { Text("Waiting for the other side…").muted }
        open       { Button("Wave") { on click { link.send(Wave(text: "hello")) } } }
        closed(c)  { Text("They left.") }
        error(e)   { Alert(e.message).danger }
    }
    Text(heard)
}
```

A `broadcast` channel signals between two tabs of one browser; between two
readers the signal rides the app's own socket or `api`. `ice:` names the
servers that help two readers behind different networks find each other —
none by default, which connects readers on the same network only. A page
that leaves closes its peer, and the other side reads `closed` at once.

