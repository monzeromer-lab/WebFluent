### The network as a value

Like `viewport` and `theme`, read where it matters:

```wf
if !network.online { Alert("You are offline — changes are queued.").warning }
Image(hero, alt: "…", loading: if network.saveData { "lazy" } else { "eager" })
```

`network.online`, `.effectiveType` (`4g`, `3g`, …), `.saveData`, `.downlink`,
and `.queued` — the writes waiting for the connection (see Offline).

