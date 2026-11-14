# Offer a new version

> When a deploy lands, offer the reader the new version of an offline site: update.available says it is ready, and update.apply() reloads into it.

The page: https://webfluent.monzeromer.dev/recipes/offer-a-new-version

Tell the reader of an offline site that a new version has been deployed,
and let them reload into it when they choose. This needs offline support
turned on ([Make a site work offline](https://webfluent.monzeromer.dev/recipes/work-offline.md)).

This home page offers a reload when a new version is ready, and shows how
many changes are waiting to be sent:

**Needs:** `offline` in `webfluent.app.json` ([Make a site work
offline](https://webfluent.monzeromer.dev/recipes/work-offline.md)) and a host that serves the site over HTTPS. It
claims `/`: paste it into a new project, or replace the starter's
`src/pages/Home.wf` with it.

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

## How it works

- **Every deploy is a version.** The worker is versioned by a hash of
  everything the build wrote, so any deploy that changes a byte is a new
  version.
- **The new version waits.** It installs beside the old one and waits.
  `update.available` turns true when it is ready.
- **`update.apply()` takes it** and reloads the page once. It never reloads
  on the first install, which only takes over a page that already works,
  and never twice for one update. The old version's store is deleted when
  the new one takes over.
- **`network.queued`** is how many writes made offline are waiting
  ([Keep writes made offline](https://webfluent.monzeromer.dev/recipes/keep-writes-made-offline.md)).
