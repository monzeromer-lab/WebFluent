# Show what the network is doing

> Change what a page does on a slow connection or none at all by reading `network`: online, connection speed, data saver and queued writes.

The page: https://webfluent.monzeromer.dev/recipes/show-the-network

`network` tells a page whether it is online, how fast the line is and
whether the reader asked to save data, so the page can change what it does
on a slow line or none at all. This inbox warns when the browser is offline
and when data saver is on:

```wf
page Inbox(path: "/inbox", title: "Inbox", description: "Your messages.") {
    Heading("Inbox").h1
    if !network.online { Alert("You are offline — changes are queued.").warning }
    if network.saveData { Text("Data saver is on.").muted }
}
```

## How it works

Like `viewport`, `network` is a browser value that keeps itself current:
each `if` that reads it is drawn again when the connection changes, with
nothing to subscribe to.

| Member | What it is |
|---|---|
| `network.online` | Whether the browser has a connection |
| `network.effectiveType` | How fast it is: `4g`, `3g`, … |
| `network.saveData` | Whether the reader asked to save data |
| `network.downlink` | The estimated bandwidth, in megabits per second |
| `network.queued` | How many writes wait for the connection, when the site works offline ([Offline](https://webfluent.monzeromer.dev/recipes/keep-writes-made-offline.md)) |
