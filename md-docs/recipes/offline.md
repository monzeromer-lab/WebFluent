---
chapter: 26
kind: "recipe"
route: "guide/offline"
group: "data"
blurb: "Turn on a service worker the build writes for you: the site works with no network, writes wait for the connection, and new versions are offered, not forced."
description: "Offline support: precaching routes, a fallback page, cache policies, writes kept for later, the update flow, testing it, and its limits."
---
# 26. Offline

This chapter shows you how to make your site work with no network. You add
one `offline` section to the config, and the build writes a *service
worker* — a script the browser runs beside your site, which answers the
site's requests from a store on the reader's machine. Stored pages then load
offline, writes made offline are sent when the connection returns, and a new
deploy is offered to the reader rather than swapped in under them. You do
not write the worker or name any cache by hand.

## Before you start

- **Serve the site over HTTPS**, or from `localhost`. Browsers run service
  workers only from a secure origin.
- **Serve `sw.js` fresh.** The page registers the worker with
  `updateViaCache: "none"`, so the browser asks for it on every visit — but
  a CDN in front can still hold a copy. Give `sw.js` the header
  `Cache-Control: no-cache` ([Deploying](deploying.md#set-cache-headers)).
- **Deploy everything in `build/` together.** The worker's version is a
  hash of every file the build wrote, so deploying half a build confuses it.

## Turn on offline support

To store the site for when the network is gone, add `offline` to
`webfluent.app.json`. The build then writes a service worker, `sw.js`,
beside the pages:

```json
{ "offline": {
    "precache": ["/", "/docs/*"],
    "fallback": "/offline",
    "cache": { "/api/*": "network-first" },
    "sync": true
} }
```

| Key | What it does |
|---|---|
| `precache` | The routes a first visit stores, as globs. `"/"` by default. Each route is stored with its own chunk and stylesheet, and the shell with them. |
| `fallback` | A page's `path`, shown for a route that was not stored. |
| `cache` | How a path the build did not write — an API, say — is fetched: `network-first`, `cache-first`, `stale-while-revalidate` or `network-only`. A path not named passes through untouched. |
| `sync` | Keep a write made with no network and send it later ([Keep writes made offline](#keep-writes-made-offline)). |

A stored route loads with no network at all. When there is a network, a
navigation still goes to it first, so a page that changed is never served
stale while the server is there to ask.

The `fallback` is an ordinary page of your site. This one tells the reader
what happened:

```wf
page Offline(path: "/offline", title: "Offline", description: "Shown when a page was not stored.") {
    Heading("You are offline").h1
    Text("This page was not saved for offline use. It will load when you are back online.")
    Link("Go to the home page", to: "/")
}
```

## Keep writes made offline

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

## Offer a new version

The worker is versioned by a hash of everything the build wrote, so any
deploy that changes a byte is a new version. The new version installs beside
the old one and waits. `update.available` turns true when it is ready, and
`update.apply()` takes it. This home page offers a reload and shows the
queue:

```wf
page Home(path: "/", title: "Home", description: "The front page.") {
    if update.available {
        Button("A new version is ready — reload") { on click { update.apply() } }
    }
    if network.queued > 0 {
        Text("{network.queued} change(s) will be sent when you are back online").muted
    }
    Heading("Home").h1
}
```

`update.apply()` takes the new version and reloads the page once. It never
reloads on the first install, which only takes over a page that already
works, and never twice for one update. The old version's store is deleted
when the new one takes over.

## What the worker keeps

What a `cache` entry fetches is kept in the worker's runtime store, the 200
most recently fetched answers, for the next reader of the device — so an
answer that is one reader's is never kept: one the server marks
`Cache-Control: no-store` or `private`, one asked for with an
`Authorization` header, and one that varies by `Cookie` or `Authorization`.
Mark what is personal `private` and it stays off the device.

Two sites on one origin under different `base_path`s keep apart: each has
its own stores, kept writes and `persist` values (`wf:/shop:cart`), and
neither deletes, serves or sends the other's. A site at the root keeps the
names it always had, and one under a base path reads a value kept under the
old name once and keeps it under the new one.

> [!NOTE]
> Under `wf serve` the worker removes itself. A page answered from its own
> store would hide every edit you make.

## Test it offline

Because `wf serve` removes the worker on purpose, test the built output with
a plain static server. Build the site and serve `build/`:

```bash
wf build
python3 -m http.server -d build
```

Then, in the browser:

1. Open the site. In the developer tools, open
   **Application → Service workers** and check the worker is activated.
2. In the **Network** panel, tick **Offline** and reload. Stored routes
   load; every other route shows the `fallback` page.
3. Make a write, then untick **Offline**. The write goes out, and
   `network.queued` drops to zero.

`wf verify` runs against the built output too, and the repository's own
browser tests hold the whole flow to a real Chrome.

## Troubleshooting

- **An upload made offline fails.** A `File` body is never kept: the call
  fails as it would without `sync`. Ask the reader to try again online.
- **A write arrives at the server minutes late.** That is the design: a kept
  write is sent once it can be. Make the server's endpoints safe to receive
  late, and idempotent where a retry could repeat one.
- **The API the site needs does not work offline.** What `cache` does not
  name passes straight through to the network. Give the API a
  `network-first` or `stale-while-revalidate` entry.
- **A page that loaded offline yesterday has to be fetched again.** The
  browser decides how much it keeps, and can evict the store under storage
  pressure. The next visit with a network stores it again.
- **Readers keep getting the old version.** Something in front of the site
  caches `sw.js`. Serve it with `Cache-Control: no-cache`.
- **Nothing is stored when you test with `wf serve`.** The dev server
  removes the worker. Test the output of `wf build` with a static server.

## Next

[Content](content.md): markdown, pages written as .md files, code on the page, and what the build writes for search engines and link previews.
