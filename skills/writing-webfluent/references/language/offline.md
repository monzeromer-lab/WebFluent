### Offline

Name `offline` in the config and `wf build` writes a service worker, `sw.js`,
that stores the site for when the network is gone:

```json
{ "offline": {
    "precache": ["/", "/docs/*"],
    "fallback": "/offline",
    "cache": { "/api/*": "network-first" },
    "sync": true
} }
```

- `precache` — the routes a first visit stores, as globs (default `"/"`),
  each with its own chunk and sheet, and the shell with them. A stored route
  loads with no network; a navigation goes to the network first when there is
  one, so a page that changed is not served stale.
- `fallback` — a page's `path`, shown for a route that was not stored.
- `cache` — how a path the build did not write is fetched: `network-first`,
  `cache-first`, `stale-while-revalidate` or `network-only`. Anything not
  named passes through untouched.
- `sync` — a write safe to send twice (a `PUT` or a `DELETE`, through
  `http` or an `api`) that fails for want of a network is kept and sent, in
  order, when the connection returns — by Background Sync where the browser
  has it, so a closed page still sends it. A `POST` or a `PATCH` is kept only
  when its endpoint is marked `idempotent`: any other one whose answer was
  lost may have been taken, and sent again would be a second order, so it
  fails as `.offline(queued: false)`. A kept write's call
  fails `.offline(queued: true)` — no answer came, and the write waits — so
  the page says so rather than taking it as done. Once it is sent, what its
  endpoint `invalidates` asks again. A server that answers has the write;
  one that refuses it puts it in `network.refused` — `{ method, url,
  status, at }`, never sent again — until `network.clearRefused()`; one
  that is down or asks to wait keeps it for later. A `File` body is
  not kept. Only a write the network failed is kept — the browser offline,
  or the site's own server unreachable — and never a credential: an `api`'s
  middleware (`bearer(…)`) adds them again when it is sent, and a call that carries
  its own credential header is not kept. `WF.clearQueued()` forgets them.
  A site under a `base_path` keeps its stores, writes and `persist` values
  apart from another on the origin.

The worker is versioned by a hash of everything the build wrote, so a
deploy that changes a byte is a new version. It installs and waits:

```wf
page Home(path: "/", title: "Home", description: "The front page.") {
    if update.available {
        Button("A new version is ready — reload") { on click { update.apply() } }
    }
    if network.queued > 0 {
        Text("{network.queued} change(s) will be sent when you are back online").muted
    }
    for r in network.refused by r.at {
        Alert("The server refused {r.method} {r.url} ({r.status})").danger
    }
    Heading("Home").h1
}
```

`update.apply()` takes the new version and reloads the page once — not on
the first install, and not twice for one update. Under `wf serve` the
worker takes itself away, so an edit is never hidden behind its own cache.

