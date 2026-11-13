---
kind: "recipe"
route: "/recipes/keep-writes-made-offline"
group: "reach"
description: "To keep the reader's changes when the network drops, set `\"sync\": true`."
from: ["/docs/guide/offline#keep-writes-made-offline"]
---
# Keep writes made offline

To keep the reader's changes when the network drops, set `"sync": true`.
A write — any method but `GET` and `HEAD`, made through `fetch` or an
`api` — that fails because the network is gone is then kept instead of
thrown. The call resolves with nothing, so what the page showed stays shown.

Only a write that failed for want of a network is kept: the browser says it
is offline, or the site's own server could not be reached. A request to
another origin that is refused while the browser is online — a CORS
refusal, a closed port — fails as it would without `sync`.

A credential is never stored with a kept write. What an `api`'s
`on request` hook adds — an `Authorization` header from the session — is
added again when the write is sent, by the hook as it is then: a write made
before a logout goes out with no one's token, never the old one. A write
whose call carries a credential header itself (`headers: { "Authorization":
token }`) cannot be kept without it, so it is not kept, and its call fails
as it would without `sync`.

When the connection returns, the kept writes are sent in order:

- by Background Sync, where the browser has it, so a closed tab still sends
  them;
- by the page otherwise, the next time it is open.

A server that answers has the write, even if it refuses it. A server that is
down, or asks the client to wait, keeps the write waiting for later. A
`File` body is not kept, and its call fails as it would without `sync`.

A write whose service adds credentials with its hook is sent by a page that
has that service open, so the worker leaves it — and the writes after it,
to keep them in order — for the next such page.

To forget every kept write — when the reader logs out, say — call
`WF.clearQueued()` from one of the project's scripts.

`network.queued` is how many writes are waiting. Show it so the reader knows
their changes are safe:

```wf
page Notes(path: "/notes", title: "Notes", description: "Notes that save offline.") {
    Heading("Notes").h1
    if network.queued > 0 {
        Text("{network.queued} change(s) will be sent when you are back online").muted
    }
}
```
