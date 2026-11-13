---
kind: "recipe"
route: "/recipes/offer-a-new-version"
group: "reach"
description: "The worker is versioned by a hash of everything the build wrote, so any deploy that changes a byte is a new version."
from: ["/docs/guide/offline#offer-a-new-version"]
---
# Offer a new version

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
