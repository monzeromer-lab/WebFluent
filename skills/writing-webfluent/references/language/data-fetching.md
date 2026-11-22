### Data Fetching

```wf
resource users: [User] = http.get("/api/users")
match users {
    .loading { Spinner }
    .failed(err) { Alert("Failed: {err.message}").danger }
    .loaded(users) {
        for user in users by user.id {
            Text(user.name)
        }
    }
}
```

A read is **one value**, in one of three cases, and a `match` takes each:

| Case | When |
|---|---|
| `.loading` | nothing to show yet |
| `.failed(e)` | the first load failed; `e` is what `.err` holds for the same request — an `HttpError`, or the endpoint's error enum (`-> T ! E`) |
| `.loaded(value, refreshing, stale)` | something to show; `refreshing` while it is asked again; `stale` the `E?` of a refresh that failed |

"Loaded, and the last refresh failed" is `.loaded(value, false, e)`;
"loading, with data" cannot be written. An arm binds as many of its case's
places as it wants (`.loaded(list)`), `_` none; a `match` with no `else` covers
all three (`T15`, and `T21` for a missing `.failed`); `if let .failed(e) =
users { … }` takes one case. A read declares its type — `resource users:
[User] = http.get(…)`, `N11` without one — and the `.loaded` binding has
it; one over an endpoint takes the endpoint's. The verb is the method —
`http.get`, `http.post`, `http.put`, `http.patch`, `http.delete` — and
options follow the address: `http.post(url, body: { key: value }, headers:
{ "Authorization": token })`.

To read one fact without a `match`, a read has members worked out from the
one value, which cannot disagree with it: `users.value` (`T?`),
`users.isLoading`, `users.isLoaded`, `users.isFailed`, `users.isRefreshing`,
`users.error` (`E?`: the first load's failure, or a refresh's), and
`users.pending` (any request of it running). Each is live wherever it is
read — `Button(if plan.isLoaded { "Request approval" } else { "Deploy" })`
changes when the plan arrives, with no `derived` in between. 6.x's
`.state`, `.data` and `.refreshing`, and its `loading`, `error(e)` and
`ready(v)` arms, are `N08`, each with its fix.

**Asking again.** Every argument and every part of the address, a body or a
header that reads state is followed: when one changes, the read asks again.
`refresh:` says when else — `.onReconnect` (what a read does when it says
nothing), `.onFocus`, `.every(30.seconds)`, `.never`, or a list (an option
of `http.get(…)`, or of an endpoint's call) — and `users.reload()` and
`users.invalidate()` ask now. Asking again for the request already shown is
a **refresh**, and the page stays as it is while it runs: the read stays
`.loaded`, `refreshing` true, and the answer is drawn in place. The
`.loaded` arm's bindings are live — text, attributes, `if`, `if let`, a
keyed `for … by` and component props inside the arm follow the new value; a
row whose key is still there keeps its nodes, its focus and its components'
state; and the parts of the answer equal to what was shown are kept as the
same objects, so an answer that changed nothing redraws nothing. The arm is
drawn again only when the case changes.

- A refresh that fails stays `.loaded`, with the value, and its failure is
  `stale` until the next answer arrives, so the arm can say the value is
  out of date. A first load that fails is `.failed(e)`.
- A request that changed — an argument that reads state now names something
  else: another environment, another page — is a first load again,
  `.loading`, since the value on screen would be the wrong one;
  `placeholder: .previous` keeps the last value shown, refreshing, until the
  new one arrives. A paged read's next page is a refresh (see Pages).

```wf
type Server { id: String, name: String, status: String }

page Servers(path: "/servers", title: "Servers", description: "Every server, kept current.") {
    state stage = "test"
    resource servers: [Server] = http.get("/api/servers?env={stage}", refresh: .every(30.seconds), placeholder: .previous)
    Heading("Servers").h1
    Button("Production") { on click { stage = "prod" } }
    match servers {
        .loading { Skeleton(height: "20px") }
        .failed(e) { Alert(e.message).danger }
        .loaded(list, refreshing, stale) {
            if stale != null { Alert("Could not refresh; this is what was there.").warning }
            if refreshing { Spinner.sm }
            for s in list by s.id { Text("{s.name}: {s.status}") }
        }
    }
}
```

Map literals support **quoted string keys** for headers and special field names:

```wf
resource data: Created = http.post("/api/users",
    headers: { "Content-Type": "application/json", "X-Api-Key": apiKey },
    body: { action: "create", token: sessionToken })
match data {
    .loaded(created) { Text("Done") }
    else { Spinner }
}
```

Reserved words (`action`, `token`, `error`, `state`, etc.) work as map keys.

