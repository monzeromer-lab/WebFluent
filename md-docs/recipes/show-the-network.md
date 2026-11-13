---
kind: "recipe"
route: "/recipes/show-the-network"
group: "realtime"
description: "To change what a page does on a slow line or none at all, read `network`."
from: ["/docs/guide/realtime#show-what-the-network-is-doing"]
---
# Show what the network is doing

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
| `network.queued` | How many writes wait for the connection, when the site works offline ([Offline](keep-writes-made-offline.md)) |
