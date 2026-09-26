# 17. Data

<!--
route: guide/data
group: building
blurb: Getting data onto a page: a service described once, a resource a page shows, a fetch an action makes, and files read at build time.
description: api services with retries, caching and typed errors, resource and match, reactive URLs, await fetch, optimistic updates and data files.
-->

Ways to get data onto a page: `api` for a service described once and called
everywhere, `resource` for something fetched while the page shows, `await
fetch` inside an action for something fetched on demand, and `data` for a
file read at build time. A connection the page holds open — a socket, a
stream, a channel, a peer — is [Real-time](18-realtime.md); working without
a network is [Offline](19-offline.md); values fixed per build are
[Environments](30-environments.md).

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

### Settings

What goes in the service's parentheses or its block, before the endpoints:

| Setting | Values | Default |
|---|---|---|
| `base:` | The address every path is relative to: `"/api/v1"`, `env.PUBLIC_API` | — |
| `timeout:` | A duration after which a call gives up with `.timeout`: `10.seconds` | none |
| `retry:` | A count (`3`), `.never`, or `.backoff(times: 3, on: [.network, .timeout, .status5xx], delay: 300, max: 30000, jitter: true)` | no retries |
| `cache:` | `.swr(60.seconds)` (serve what is held, revalidate behind it), `.cache(5.minutes)`, `.forever`, `.none` | `.none` |
| `credentials:` | `.omit`, `.sameOrigin`, `.include` — whether cookies go with a call | the browser's (`.sameOrigin`) |
| `mode:` | `.cors`, `.noCors`, `.sameOrigin` | `.cors` |
| `redirect:` | `.follow`, `.error`, `.manual` | `.follow` |
| `referrer:` | a referrer policy: `.noReferrer`, `.origin`, … | the page's |
| `headers { }` | Headers every call carries, each read at the moment of the call | — |

`retry`'s `on:` takes `.network`, `.timeout`, `.status5xx`, `.status4xx`,
`.status429` and `.status503`-style single codes; left out, it retries the
network, timeouts, 5xx and 429. The wait doubles from `delay` (ms) up to
`max`, with jitter, and a `Retry-After` from the server wins. An aborted call
is never retried.

### Endpoints

```text
get users(page: Number = 1, q: String?) -> [User]
get user(id: String) at "users/:id" -> User
post createUser(body: Map) -> User
    errors { 422 -> Map }
put rename(id: String, body: Map) at "users/:id" -> User
patch settings(body: Map) -> Map
delete removeUser(id: String) at "users/:id"
get report(id: String) at "reports/:id" as: .blob
```

- The verb is `get`, `post`, `put`, `patch`, `delete`, `head` or `options`.
- A parameter the path names (`:id`) goes in the path; a parameter called
  `body` is the request body (a map is sent as JSON); a `File` parameter
  makes the call a multipart upload; every other parameter goes in the query
  string, and one that is `null` is left out.
- `-> T` is what a success returns; without it the call returns `Any`, and
  `204 No Content` returns nothing.
- `errors { 422 -> T }` types the body of a failure with that status: the
  `.status(code, body)` error carries it decoded.
- `as:` says how to read a response that is not JSON: `.text`, `.blob`,
  `.arrayBuffer`, `.none`. `errorAs:` does the same for a failure's body.
  `cache:` sets a cache policy for that endpoint alone.

A call passes the endpoint's parameters by name, and may add `cache:`,
`retry:` and — in a `resource` — `on:` for when to fetch again
(`.focus`, `.reconnect`, `.interval(30.seconds)`) and `paginate:`.

### Pages of results

```wf
type Item { id: String, name: String }

api Catalog(base: "/api") {
    get items(page: Number = 1) -> [Item]
}

page Items(path: "/items", title: "Items", description: "Everything, a page at a time.") {
    state page = 1
    resource list = Catalog.items(page: page, paginate: .page)
    Heading("Items").h1
    for item in list.items by item.id { Text(item.name) }
    if list.hasMore {
        Button("Load more") { on click { list.loadMore() } }
    }
}
```

`paginate: .page` gathers the pages into `.items`, `loadMore()` moves `page`
on by one, and `.hasMore` turns false when a page comes back empty.

### An API on another origin

A page on `https://example.com` calling `https://api.example.com` is a
cross-origin request: the API must answer with CORS headers
(`Access-Control-Allow-Origin`, and `-Allow-Credentials` if cookies go with
it), or the browser refuses the response. The same applies while you
develop: `wf serve` answers on `localhost:3000` and has no proxy, so an API
on `localhost:8080` must allow that origin. The alternative is to serve the
API and the site from one origin behind a reverse proxy, where `base: "/api"`
needs no CORS at all ([Deploying](29-deploying.md#an-api-on-the-same-origin)).

### Showing a change before the server agrees

```wf
action rename(id: String, name: String) {
    optimistic(Todos.items, items => items.map(i => if i.id == id { { ...i, title: name } } else { i }))
    await Backend.updateUser(id: id, body: { name: name })
    Backend.users.invalidate()
}
```

The change shows at once. If anything later in the action throws, what was
shown is taken back.

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

## Where data lands in the static paint

| Source | In the pre-rendered HTML |
|---|---|
| `data`, `const`, `env` | Fully painted |
| `state` initial values, `derived` over them | Painted |
| `resource` | The `loading` arm; the live page fetches and replaces it |
| `await fetch` | Nothing until the action runs |

## Next

[Real-time](18-realtime.md).
