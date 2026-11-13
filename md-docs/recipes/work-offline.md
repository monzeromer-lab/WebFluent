---
kind: "recipe"
route: "/recipes/work-offline"
group: "reach"
description: "To store the site for when the network is gone, add `offline` to `webfluent.app.json`."
from: ["/docs/guide/offline#before-you-start", "/docs/guide/offline#turn-on-offline-support", "/docs/guide/offline#what-the-worker-keeps", "/docs/guide/offline#test-it-offline", "/docs/guide/offline#troubleshooting"]
---
# Make a site work offline

## Before you start

- **Serve the site over HTTPS**, or from `localhost`. Browsers run service
  workers only from a secure origin.
- **Serve `sw.js` fresh.** The page registers the worker with
  `updateViaCache: "none"`, so the browser asks for it on every visit — but
  a CDN in front can still hold a copy. Give `sw.js` the header
  `Cache-Control: no-cache` ([Deploying](cache-and-security-headers.md#set-cache-headers)).
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
| `sync` | Keep a write made with no network and send it later ([Keep writes made offline](keep-writes-made-offline.md)). |

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
