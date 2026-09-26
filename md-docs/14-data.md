# 14. Data

Ways to get data onto a page: `api` for a service described once and called
everywhere, `resource` for something fetched while the page shows, `await
fetch` inside an action for something fetched on demand, and `data` for a
file read at build time. Plus `const` and `env` for values that are the
same for the whole build, and `socket`, `stream` and `channel` for a
connection the page holds open.

## `resource`

A resource is a request the page makes and a `match` shows:

```wf
type Deploy { id: String, status: String, startedAt: String }

page Deploys(path: "/", title: "Deploys", description: "Recent deploys.") {
    resource deploys: [Deploy] = fetch("/api/deploys")
    match deploys {
        loading { Spinner }
        error(e) { Alert("Could not load: {e.message}").danger }
        ready(rows) {
            if rows.length == 0 { Text("No deploys yet.").muted }
            for d in rows by d.id {
                Row(justify: .between) {
                    Text(d.id).bold
                    Badge(d.status, tone: if d.status == "ok" { .success } else { .danger })
                    Text(ago(d.startedAt)).muted.sm
                }
            }
        }
    }
    Button("Refresh") { on click { deploys.reload() } }
}
```

- `resource name = fetch(url)` starts the request when the page renders.
  The annotation (`: [Deploy]`) types the value; without it the value is
  `Any`.
- `match` over a resource has three arms — `loading`, `error(e)`, `ready(v)`
  — and `else`. The static paint renders the `loading` arm.
- `name.reload()` makes the request again; `name.state` is `"loading"`,
  `"ready"` or `"error"`; `name.data` the last value (`T?`), `name.error`
  the last error.
- A failed response (`!ok`) is an error with `e.message` of `HTTP 500`.

### Options

```wf
page Search(path: "/") {
    state q = ""
    state token = "abc"
    resource hits = fetch("/api/search?q={q}", headers: { Authorization: "Bearer {token}" })
    resource created = fetch("/api/items", method: "POST", body: { name: q })
    Input(bind: q, label: "Search")
    match hits {
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} hits") }
    }
}
```

Options are named after the URL: `method:`, `headers:` (a map), `body:` (a
map, sent as JSON with the content type set). The body of a response is
parsed as JSON.

### A reactive URL

A URL that reads state is re-fetched whenever that state changes — a
search box that fetches as you type, or a detail page that follows its
parameter:

```wf
type Post { slug: String, title: String, body: String }

page PostPage(path: "/posts/:slug", slug: String) {
    resource post: Post = fetch("/api/posts/{slug}")
    match post {
        loading { Skeleton }
        error(e) { Alert("Not found").warning }
        ready(p) {
            Heading(p.title).h1
            Markdown(p.body)
        }
    }
}
```

Debounce a fast-changing URL by deriving it from a slower value, or by
keeping the query in state and copying it to a `submitted` state on Enter.

## `api`: a service, described once

An address, how it is reached, and what it has — in one place, so every
call site is typed, cached and cancellable:

```wf
type User { id: String, name: String }

api Backend(base: "/api/v1") {
    credentials: .sameOrigin      // the session cookie goes with every call
    timeout: 10.seconds
    retry: .backoff(times: 3, on: [.network, .timeout, .status5xx])

    get    users(page: Number = 1, q: String?) -> [User]
    get    user(id: String) at "users/:id"     -> User
    post   createUser(body: Map)               -> User
        errors { 422 -> Map }
    delete removeUser(id: String) at "users/:id"
}

page Users(path: "/users", title: "Users", description: "Everyone.") {
    state page: Number = 1
    resource rows = Backend.users(page: page, cache: .swr(60.seconds), on: .focus)

    Heading("Users").h1
    match rows {
        loading { Skeleton(height: "20px") }
        error(e) {
            match e {
                .offline { Alert("You are offline.").warning }
                .timeout { Alert("The server took too long.").warning }
                else     { Alert(e.message).danger }
            }
            Button("Try again") { on click { rows.reload() } }
        }
        ready(list) { for u in list by u.id { Text(u.name) } }
    }
}
```

A parameter the path names goes in the path; every other goes in the query.
`->` says what comes back, and a `resource` over the endpoint is typed by
it — `u.nam` is `T05`, before the page ever runs.

### Headers, and what happens around a call

A `headers { }` block names what every call carries. Each value is read
**at the moment of the request**, not when the service is declared — so a
token that has just been refreshed is the one that is sent:

```wf
api Backend(base: env.PUBLIC_API ?? "/api/v1") {
    headers {
        Authorization: "Bearer {Session.token}"
        Accept-Language: Session.language
    }

    on request(r)  { r.headers["X-Request-Id"] = uuid() }
    on response(r) { Metrics.record(r.status) }
    on error(e)    { if e.status == 401 { await Session.refresh()  return "retry" } }

    get me() -> User
}
```

The three hooks run around every call the service makes:

| Hook | When it runs | What it gets | What returning does |
|---|---|---|---|
| `on request(r)` | Before the request is sent, after `headers { }` | `r.url`, `r.method`, `r.headers`, `r.body` — change them in place | The changed request is what goes out |
| `on response(r)` | After a reply arrives, before it is decoded | `r.status`, `r.headers`, `r.url` | Nothing; it is for recording |
| `on error(e)` | After every retry the policy allows has failed | The error, as the table below describes it | `return "retry"` runs the call once more — which is how a 401 refreshes a token and carries on; anything else lets the error through |

A hook may `await`. One that throws fails the call it was watching.

### What each call gets

| | |
|---|---|
| Abort | when the scope leaves, when an argument that reads state changes, on `.cancel()`, on timeout |
| Errors | `.offline`, `.timeout`, `.aborted`, `.parse`, `.network`, `.status(code, body)` — with `.message`, `.status`, `.body` and `.headers` to read |
| Retry | exponential backoff with jitter, per error kind, honouring `Retry-After` |
| Cache | `cache: .swr(60.seconds)`, `.none`, `.forever` — keyed by method, address and body, revalidated with `ETag` |
| Dedupe | two readers of one address make one request |
| Refetch | `on: .focus`, `.reconnect`, `.interval(30.seconds)` |
| Pagination | `paginate: .page` gives `.items`, `.loadMore()` and `.hasMore` |
| Progress | a `File` parameter makes `Backend.avatar.progress` a number from 0 to 1 |

And beside each endpoint: `Backend.users.invalidate()`, `.prefetch(args)`,
`.url(args)`, `.key(args)`.

### From a specification

```wf
api Backend from "openapi.json" (base: env.PUBLIC_API)
```

Every endpoint, parameter, response type and error shape is read from the
file at build time, and each named schema becomes a `type` the program can
name. The day the server changes its contract, the build says so.

### Showing a change before the server agrees

```wf
action rename(id: String, name: String) {
    optimistic(Todos.items, items => items.map(i => if i.id == id { { ...i, title: name } } else { i }))
    await Backend.updateUser(id, { name: name })
    Backend.users.invalidate()
}
```

The change shows at once. If anything later in the action throws, what was
shown is taken back.

## A connection the page holds open

A socket, a stream of server-sent events, or a channel every tab of the
origin hears. Each is closed when the page that opened it leaves.

```wf
page Chat(path: "/chat", title: "Chat", description: "Talk.") {
    state draft = ""
    socket chat = ws("wss://example.com/chat", heartbeat: 20.seconds) {
        on message(m) { log(m) }
    }

    Heading("Chat").h1
    match chat {
        connecting { Spinner.sm }
        open       { for m in chat.messages by m.id { Text(m.text) } }
        closed(c)  { Alert("Disconnected ({c.code})").warning }
        error(e)   { Alert(e.message).danger }
    }
    Input(bind: draft, label: "Message").text
    Button("Send").primary { on click { chat.send({ text: draft })  draft = "" } }
}
```

A socket reconnects with backoff, keeps itself alive with a heartbeat, and
holds what was sent while it was down.

```wf
stream ticks = sse("/events", events: ["price"])
channel cart = broadcast("cart") { on message(m) { Cart.merge(m) } }
beacon("/analytics", { event: "checkout" })     // survives the page unloading
```

`ticks` and `cart` are handles too — a `match` reads the state, and an
`effect` reads what arrived:

```wf
effect { if let p = ticks.last("price") { price = p } }
Button("Sync") { on click { cart.post({ items: Cart.count }) } }
```

What each handle holds:

| | `socket` | `stream` | `channel` |
|---|---|---|---|
| `.state` | `connecting` `open` `closed` `error` — the arms of a `match` | the same | `open` or `closed` |
| `.messages` | every message, in order | every message, in order | — |
| `.last(kind)` | the last message, or the last of a kind | the last under an event's name | the last posted |
| `.error` | the failure, which the `error(e)` arm is handed | the same | — |
| `.closure` | the close, which `closed(c)` is handed: `.code`, `.reason` | — | — |
| Sending | `.send(value)` — queued while the line is down | — | `.post(value)` |
| `.close()` | closes it early | the same | the same |

A `match` over one takes `connecting`, `open`, `closed(c)` and `error(e)`,
and an `else` for the rest. The page closing closes the connection, so a
route change cannot leak one.

## The network as a value

```wf
if !network.online { Alert("You are offline — changes are queued.").warning }
```

`network.online`, `.effectiveType` (`4g`, `3g`, …), `.saveData`, `.downlink`
— live, so a page can say what it does on a slow line or none at all — and
`.queued`, the writes waiting for the connection when the site works offline.

## Working offline

Name `offline` in `webfluent.app.json` and the build writes a service worker,
`sw.js`, that stores the site for when the network is gone:

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
| `precache` | The routes a first visit stores, as globs — `"/"` by default. Each is stored with its own chunk and sheet, and the shell with them. |
| `fallback` | A page's `path`, shown for a route that was not stored. |
| `cache` | How a path the build did not write is fetched: `network-first`, `cache-first`, `stale-while-revalidate` or `network-only`. A path not named passes through. |
| `sync` | A write made with no network is kept and sent later (below). |

A stored route loads with no network at all. When there is one, a
navigation still goes to it first, so a page that changed is never served
stale while the server is there to ask.

### A write made offline

With `sync`, a write — any method but `GET` and `HEAD`, through `fetch` or
an `api` — that fails because the network is gone is kept instead of
thrown, and the call resolves with nothing, so what the page showed stays
shown. The writes are sent in order when the connection returns: by
Background Sync where the browser has it, so a closed tab still sends them,
and by the page otherwise. A server that answers has the write, even if it
refuses it; one that is down or asks to wait keeps it for later. A `File`
body is not kept, and its call fails as before.

`network.queued` is how many wait.

### A new version

The worker is versioned by a hash of everything the build wrote, so any
deploy that changes a byte is a new version. It installs beside the old one
and waits for the page to take it:

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

`update.apply()` takes it and reloads the page once: never on the first
install, which only takes over a page that already works, and never twice
for one update. The old version's store is deleted when the new one takes
over.

Under `wf serve` the worker takes itself away: a page answered from its own
store would hide every edit you make.

## `await fetch` in an action

For a request made on demand — a save, a delete, a step in a flow —
`fetch` returns the parsed body and throws on a failed response:

```wf
type Todo { id: String, title: String, done: Bool = false }

page Todos(path: "/") {
    state todos: [Todo] = []
    state draft = ""
    state error = ""

    action load() {
        let r: [Todo] = await fetch("/api/todos")
        todos = r
    }

    action add() {
        if draft.trim() == "" { return }
        try {
            let created: Todo = await fetch("/api/todos", { method: "POST", body: { title: draft } })
            todos.push(created)
            draft = ""
        } catch e {
            error = "Could not save: {e.message}"
        }
    }

    action toggle(id: String) {
        let t = todos.find(x => x.id == id)
        if let todo = t {
            let updated: Todo = await fetch("/api/todos/{id}", { method: "PATCH", body: { done: !todo.done } })
            todos = todos.map(x => if x.id == id { updated } else { x })
        }
    }

    load()
    Input(bind: draft, label: "New todo") { on key("Enter") { add() } }
    Button("Add", disabled: add.pending).primary { on click { add() } }
    if error != "" { Alert(error).danger }
    for t in todos by t.id {
        Checkbox(checked: t.done, label: t.title) { on change { toggle(t.id) } }
    }
}
```

The second argument is a map: `method`, `headers`, `body`. `name.pending`
is true while the action runs. The browser's own `fetch`, for a response
you want raw, is `window.fetch`.

## `data`: a file at build time

`data name = "file.json"` reads a JSON file (from the project root or
`src/`) at build time and makes its value a constant. It is in every page
and component, costs nothing at run time, and is painted into the static
build:

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page Blog(path: "/", title: "Blog", description: "Every post.") {
    derived sorted = posts.sortBy(p => p.date).reverse()
    for p in sorted by p.slug {
        Card {
            Link(p.title, to: "/posts/{p.slug}")
            Text(format(p.date, .date)).muted.sm
        }
    }
}
```

```json
[
  { "slug": "hello", "title": "Hello", "date": "2026-01-10", "body": "# Hi\nFirst post." },
  { "slug": "second", "title": "Second", "date": "2026-02-02", "body": "More." }
]
```

Annotate it (`data posts: [Post]`) and the checker knows every field.

### Static paths

A `:param` page is rendered once per value in `paths:` by a static build —
one HTML file per post, each with its own title and description:

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page PostPage(path: "/posts/:slug", slug: String, title: "Post", description: "A post.", paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    head { meta(property: "og:title", content: post?.title ?? "Post") }
    if let p = post {
        Heading(p.title).h1
        Markdown(p.body)
    } else {
        Alert("No such post").warning
    }
}
```

`paths:` must be knowable at build time — a `data` file, a `const`, a
literal. For a route with several parameters, each value is a map naming
them: `paths: pairs.map(x => { year: x.y, slug: x.s })`. Routes not in
`paths:` still work in the SPA; only the static build needs the list.

## `const` and `env`

```wf
const PAGE_SIZE = 25
const API = env.PUBLIC_API_URL ?? "/api"
const FEATURES = { billing: env.PUBLIC_BILLING == "on", beta: false }

page Rows(path: "/") {
    resource rows = fetch("{API}/rows?limit={PAGE_SIZE}")
    if FEATURES.billing { Link("Billing", to: "/billing") }
    match rows {
        loading { Spinner }
        error(e) { Text(e.message).danger }
        ready(r) { Text("{r.length} rows") }
    }
}
```

`env.NAME` reads the build environment — the shell's variables, a `.env`
file in the project root and the config's own `env` map — and is `null`
when unset.

Everything a page reads is **inlined into the bundle**, so the compiler
only lets a page, a component, a store or an `api` read a name that says
it may be read: one beginning `PUBLIC_`, or one the config's `public_env`
lists. Any other name there is a compile error, not a warning
([chapter 19](19-security.md#env-and-what-ends-up-in-the-bundle)). Build
scripts and the config itself may read any name.

## Where data lands in the static paint

| Source | In the pre-rendered HTML |
|---|---|
| `data`, `const`, `env` | Fully painted |
| `state` initial values, `derived` over them | Painted |
| `resource` | The `loading` arm; the live page fetches and replaces it |
| `await fetch` | Nothing until the action runs |

## Next

[Internationalisation](15-i18n.md).
