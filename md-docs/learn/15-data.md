---
chapter: 15
kind: "tutorial"
route: "/learn/15-data"
from: ["/docs/guide/data"]
group: "build"
blurb: "Get data onto a page: describe a service once, show a request's states, ask on demand, and read files at build time."
description: "Describe an API once with retries, caching and typed errors; show a resource with match; ask with http in an action; update optimistically; read data files."
learned: ["Show a read's three cases — `.loading`, `.failed(e)`, `.loaded(value)` — with `resource` and `match`", "Tell a refresh (`refreshing`, the value stays) from a first load (`.loading`)", "Describe a service once with `api`, typed, retried, cached and cancelled", "Ask on demand with `http` in an action, handle its result, and roll back an `optimistic` change", "Read a JSON file at build time with `data`, and render a page per item with `paths:`"]
---
# 15. Fetching data

This chapter is about getting data onto a WebFluent page and sending data
back: from a request the page makes when it shows, from a request an action
makes on demand, and from a file the build reads.

You will be able to:

- show a read's `.loading`, `.failed(e)` and `.loaded(value)` cases with `resource` and
  `match`;
- refresh data without the page going blank;
- describe a service once with `api`, so every call is typed, retried,
  cached and cancelled for you;
- keep a service's contract from its OpenAPI file, as files `wf api` writes;
- ask on demand from an action, and show a change before the server
  agrees;
- read a JSON file at build time and render one page per item in it.

## Before you start

You should know `state` and `action`
([State and reactivity](07-state-and-reactivity.md)), `match`
([Control flow](09-control-flow.md)) and how to declare a `type`
([Types](12-types.md)).

This chapter does not cover:

- a connection the page holds open — a socket, a stream, a channel or a
  peer — which is [Open a socket](../recipes/open-a-socket.md);
- working without a network, which is [Make a site work offline](../recipes/work-offline.md);
- values fixed for each build, which are
  [Environments and secrets](../recipes/environments-and-secrets.md).

## Choose how to get data

| You want… | Use | Section |
|---|---|---|
| Data the page shows as soon as it opens | `resource` | [Show a request with resource](#show-a-request-with-resource) |
| One place that describes a service and its endpoints | `api` | [Describe a service with api](#describe-a-service-with-api) |
| A request made when something happens: a save, a delete, a step in a flow | `http.post(…)` in an action | [Ask on demand with http](#ask-on-demand-with-http) |
| Content that is known at build time | `data` | [Read a file at build time with data](#read-a-file-at-build-time-with-data) |

## How the pieces fit

An app that talks to a service has five parts, each in its own place, and
each knows only the one below it:

| Part | What it is | Where it lives |
|---|---|---|
| The contract | What the service has: its endpoints, their parameters, answers and error statuses, and the types they carry | `api Backend.people { … }`, written by hand or by `wf api sync` from the service's OpenAPI spec |
| The service | Where it is and how it is reached: the base, the timeout, retries, middleware, what a write makes stale | `api Backend(base: …) { … }` |
| The data layer | The reads the app keeps and the writes it makes, and what each failure means here | a `store` holding `resource`s and actions |
| The view | What the reader sees: one arm for each case of a read, one for each way a write can fail | a `page` or `component` that `match`es |
| The test | The view and the data layer against a `fake` of the contract | `tests/`, run by `wf test` |

A small page may put a `resource` straight in the page and skip the store;
the rest stays the same. The parts below go from the bottom up: a read,
the service and its contract, writes, then files read at build time. The
recipes put them together:
[Show the server's field errors on a form](../recipes/form-with-server-errors.md),
[Refresh a list after a write](../recipes/refresh-after-a-write.md),
[Test a page with a fake service](../recipes/test-with-a-fake.md) and
[Call a service from its OpenAPI spec](../recipes/service-from-openapi.md).

## Show a request with `resource`

To fetch data while a page shows, declare a `resource` and show its states
with `match`. A *resource* is a request the page makes, and one value that
says where it is: `.loading`, `.failed(e)` or `.loaded(value)`.

This page lists recent deploys, with a spinner while they load and an alert
if the request fails:

```wf
type Deploy { id: String, status: String, startedAt: String }

page Deploys(path: "/deploys", title: "Deploys", description: "Recent deploys.") {
    resource deploys: [Deploy] = http.get("/api/deploys")

    Heading("Deploys").h1
    match deploys {
        .loading { Spinner }
        .failed(e) { Alert("Could not load: {e.message}").danger }
        .loaded(rows) {
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

> [!NOTE]
> **Coming from React?** There, a fetch is a `useEffect` that sets
> `loading`, `error` and `data` states by hand, or a `useQuery`. A
> `resource` is the request and its state together: no effect, no flags to
> keep in step, and a `match` with one arm per case. See
> [Coming from React](../coming-from/react.md).

How it works:

- `resource name: T = http.get(url)` starts the request when the page renders.
- The annotation (`: [Deploy]`) types the value. A read of `http.get` must
  have one (`N11`); a read of an endpoint takes the endpoint's.
- A read is one value in one of three cases, and a `match` over it has an
  arm for each, or an `else`:

  | Case | When |
  |---|---|
  | `.loading` | nothing to show yet |
  | `.failed(e)` | the first load failed; `e` is an `HttpError` — or the endpoint's error enum, for one declared `-> T ! E` |
  | `.loaded(value, refreshing, stale)` | something to show; `refreshing` while it is asked again; `stale` the failure of a refresh, else `null` |

  An arm names as many of its case's places as it uses: `.loaded(rows)`.
  One with no `.failed` arm and no `else` is `T21`; one that leaves out
  another case is `T15`.
- The pre-rendered page — the HTML the build writes for a static site —
  shows the `.loading` arm. The live page fetches and replaces it.
- A failed response (one that is not `ok`) is `.status(code, problem)`,
  whose `e.message` is `HTTP 500`, or whatever the status was.

### Read one fact of a resource anywhere

Without a `match`, a resource has these, each worked out from its one
value, so they cannot disagree. Each is live wherever you read it — a
label, a splice, an attribute — with no `derived` in between:

| Value | What it is |
|---|---|
| `name.value` | The value, while it is `.loaded`; else `null` (`T?`) |
| `name.isLoading` · `name.isLoaded` · `name.isFailed` | Which case it is in |
| `name.isRefreshing` | `true` while a refresh runs ([Refresh without clearing the page](#refresh-without-clearing-the-page)) |
| `name.error` | What went wrong: the first load's failure, or the last refresh's (`E?`) |
| `name.pending` | `true` while any request of it runs, the first load included |
| `name.reload()` | Makes the request again |
| `name.invalidate()` | Drops what the cache holds for it, and asks again |
| `name.cancel()` | Aborts the request that is running |

So `Button(if plan.isLoaded { "Request approval" } else { "Deploy" })`
changes its label when the plan arrives, and `if let .failed(e) = plan {
Alert(e.message).danger }` shows one case without a `match`.

### Refresh without clearing the page

These ask for a resource again:

- an argument, or a part of the address, a body or a header, that reads
  state and changes;
- the `refresh:` option of `http.get(…)` or of an endpoint's call:
  `.onReconnect` (when the network returns — what a read does when it says
  nothing), `.onFocus` (when the reader comes back to the tab),
  `.every(30.seconds)`, `.never`, or a list of them;
- `name.reload()` and `name.invalidate()`.

Asking again for the request already shown is a *refresh*. The page stays
as it is while a refresh runs: the read stays `.loaded`, with its value and
`refreshing` true, and the answer is drawn in place.

This page polls a list of servers every 30 seconds, and shows a small
spinner during each refresh:

```wf
type Server { id: String, name: String, status: String }

page Servers(path: "/servers", title: "Servers", description: "Every server, kept current.") {
    state stage = "test"
    resource servers: [Server] = http.get("/api/servers?env={stage}", refresh: .every(30.seconds))

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

> [!NOTE]
> **Coming from TanStack Query?** `isLoading` is `name.isLoading`, true
> only while there is nothing to show; `isFetching` is `name.pending`; and
> `name.isRefreshing` is a fetch while data is on screen. The three cases
> are its `status`, made one value. See
> [Coming from React](../coming-from/react.md).

What a refresh keeps:

- **The arm.** The `.loaded` arm's bindings are live. Text, attributes,
  `if`, `if let`, a keyed `for … by` and component props inside the arm
  follow the new value. The arm is drawn again only when the case changes.
- **The rows.** A row whose key is still in the list keeps its nodes, its
  focus and its components' state. An input keeps what was typed.
- **Unchanged data.** The parts of an answer equal to what was shown are
  kept as the same objects. An item is matched by its `id`, by its value, or
  by its place. An answer that changed nothing redraws nothing.

What happens when a request fails, or changes:

- A refresh that fails stays `.loaded`, with the value, and its failure is
  `stale` until the next answer, so the arm can say the value is out of
  date.
- A first load that fails is `.failed(e)`, and its `reload()` is a first
  load again.
- A request that changed is a first load again, shown as `.loading`. A
  request changes when an argument that reads state now names something
  else — another environment, another page — because the value on screen
  would be the wrong one. What the last request brought goes with it:
  `name.value` is `null` (and a paged resource's `name.items` empty) until
  the answer arrives, and stays so if it fails, so a `derived` value over it
  never reads the other request's answer. A paged read's next page is a
  refresh instead ([Load results a page at a time](#load-results-a-page-at-a-time)).
- `placeholder: .previous` keeps the last value shown instead, `.loaded`
  and refreshing, until the new answer arrives — a search whose results
  stay while the next ones load.

Pressing **Production** above changes `stage`, so the list shows its
`.loading` arm and then the production servers.

### Send options with a request

The verb is the method — `http.get`, `http.post`, `http.put`,
`http.patch`, `http.delete` — and options follow the URL, by name:

- `headers:` — a map;
- `body:` — what is sent, as JSON with the content type set.

The answer is read as JSON and is the type the resource declares.

This page searches with a `POST` and an authorization header:

```wf
type Hit { id: String, title: String }

page Search(path: "/search", title: "Search", description: "Search the catalogue.") {
    state q = ""
    state token = "demo-token"
    resource hits: [Hit] = http.post("/api/search",
        headers: { Authorization: "Bearer {token}" },
        body: { query: q, limit: 20 })

    Heading("Search").h1
    Input(bind: q, label: "Search")
    match hits {
        .loading { Spinner.sm }
        .failed(e) { Text(e.message).danger }
        .loaded(list) { Text("{list.length} hits") }
    }
}
```

### Fetch again when state changes

A URL that reads state is fetched again whenever that state changes. Use it
for a search box that fetches as the reader types, or a detail page that
follows its route parameter:

```wf
type Post { slug: String, title: String, body: String }

page PostPage(path: "/posts/:slug", title: "Post", description: "One post.", slug: String) {
    resource post: Post = http.get("/api/posts/{slug}")
    match post {
        .loading { Skeleton }
        .failed(e) { Alert("Not found").warning }
        .loaded(p) {
            Heading(p.title).h1
            Markdown(p.body)
        }
    }
}
```

A value spliced into the address is URL-encoded for you where it stands for
a value.

To fetch less often than the reader types, derive the URL from a slower
value. This page keeps what the reader types in `draft`, and searches only
when they press Enter:

```wf
type Hit { id: String, title: String }

page Lookup(path: "/lookup", title: "Lookup", description: "Search when you press Enter.") {
    state draft = ""
    state submitted = ""
    resource hits: [Hit] = http.get("/api/search?q={submitted}")

    Heading("Lookup").h1
    Input(bind: draft, label: "Search") { on key("Enter") { submitted = draft } }
    match hits {
        .loading { Spinner.sm }
        .failed(e) { Text(e.message).danger }
        .loaded(list) { Text("{list.length} results for {submitted}") }
    }
}
```

## Describe a service with `api`

To call a service from many places, describe it once with `api`: its
address, how it is reached and what endpoints it has. Every call site is
then typed, cached and cancellable.

This service has four endpoints, and the page shows a list of users with a
different alert for each kind of failure:

```wf
type User { id: String, name: String }
type NewUser { name: String }

/// Why adding someone can be refused.
enum CreateUserError {
    unprocessableContent(problem: Problem) on 422
    http(error: HttpError)
}

api Backend(base: "/api/v1") {
    credentials: .sameOrigin      // the session cookie goes with every call
    timeout: 10.seconds
    retry: .backoff(times: 3, on: [.network, .timeout, .status5xx])

    get    users(page: Number = 1, q: String?) at "users"  -> [User]!
    get    user(id: String) at "users/{id}"      -> User!
    post   createUser(body: NewUser) at "createUser"            -> User ! CreateUserError
    delete removeUser(id: String) at "users/{id}" -> !
}

page Users(path: "/users", title: "Users", description: "Everyone.") {
    state page: Number = 1
    resource rows = Backend.users(page: page, cache: .swr(60.seconds), refresh: .onFocus)

    Heading("Users").h1
    match rows {
        .loading { Skeleton(height: "20px") }
        .failed(e) {
            match e {
                .offline(_) { Alert("You are offline.").warning }
                .timeout { Alert("The server took too long.").warning }
                else     { Alert(e.message).danger }
            }
            Button("Try again") { on click { rows.reload() } }
        }
        .loaded(list) { for u in list by u.id { Text(u.name) } }
    }
}
```

- A parameter the path names goes in the path. Every other parameter goes in
  the query string.
- `->` says what a success returns and how a call fails: `-> [User]!` is a
  list of users or an `HttpError`, `-> User ! CreateUserError` a user or a
  case of that error enum, `-> !` nothing but may fail ([Say how a call
  fails](#say-how-a-call-fails)). A `resource` over the endpoint holds what
  a success returns, so `u.nam` is `T05` before the page ever runs.
- An endpoint the service does not have is `T06`, and an argument it does
  not take is `T10`.

### Pass every request through middleware

What every call of a service carries — a token, the reader's language, a
request ID — and what it does when the server says a token expired, is
*middleware*: steps each request passes through, in the order written in
the service's `middleware [ … ]`. Each step is handed the request and the
rest of the chain, `next`, and returns what `next` returns.

This service refreshes the token on a `401`, sends it as a bearer token,
sends the reader's language, and tags each request with an ID:

```wf
type User { id: String, name: String }
type Token { token: String }

store Session {
    state accessToken = ""
    state language = "en"
    action refresh() -> ! {
        let fresh: Token = try await http.post("/api/token")
        accessToken = fresh.token
    }
}

api Backend(base: env.PUBLIC_API) {     // its default in the config's `env`
    middleware [refreshOn401(Session.refresh), bearer(Session.accessToken), header("Accept-Language", Session.language), requestId]

    get me at "me" -> User!
}
```

The runtime's steps:

| Step | What it does |
|---|---|
| `bearer(token)` | `Authorization: Bearer …`, read when the request reaches it; nothing when the token is empty |
| `header(name, value)` | One header, read when the request reaches it |
| `refreshOn401(action)` | On a `401`, runs the action and sends the request again through the steps after it. Requests that hear `401` at the same time share one run of the action, since a running action is joined. Put it before `bearer`, so the request sent again carries the new token |
| `requestId` | A fresh `X-Request-Id` for each request |
| `log` | Each request, and how it ended, in the console |

A step of your own is a `middleware` declaration. Its two names are the
request — a `Request`: `url`, `method`, `headers` (a `[String: String]`) and
`body` — and the rest of the chain:

```wf
type Me { name: String }

/// Every request, timed.
middleware timed(req, next) {
    let started = Date.now()
    let answer = await next(req)
    log("{req.method} {req.url} took {Date.now() - started} ms")
    return answer
}

/// What every request sends about the client.
middleware client(req, next) {
    return await next(Request(url: req.url, method: req.method, headers: req.headers.with("X-Client", "web"), body: req.body))
}

api Accounts(base: "/accounts") {
    middleware [client, timed]
    get me at "me" -> Me!
}
```

What `next` returns is a result: `.ok(json)` with the answer before it is
decoded, or `.err(e)` with an `HttpError`. A step may look at it, call
`next` again, or answer without the network. A step that throws fails the
call with `.transport`.

### What every call does for you

| Feature | What it does |
|---|---|
| Abort | A call is aborted when its scope leaves, when an argument that reads state changes, on `.cancel()`, and on timeout |
| Errors | an `HttpError`: `.offline`, `.timeout(after)`, `.cancelled`, `.transport(message)`, `.decode(at, message)`, `.status(code, problem)` — each with `.message` |
| Decoding | What a call or a `resource` declares it returns is held to what arrives, in every build: a server that breaks its contract fails as `.decode(at, message)` |
| Retry | Exponential backoff with jitter, per error kind, honouring `Retry-After` — for a request that is safe to send twice, a `POST` or `PATCH` only when its endpoint is `idempotent` |
| Cache | What the server's `Cache-Control` allows (RFC 9111): fresh for its `max-age`, stale within `stale-while-revalidate`, revalidated with `ETag`, never under `no-store`, kept apart by `Vary` and the credentials; nothing without caching headers. `cache: .maxAge(60.seconds)`, `.swr(…)`, `.none`, `.forever` override it. It holds the 256 answers read most recently |
| Dedupe | Two readers of one address make one request, cached or not |
| Refetch | `refresh: .onFocus`, `.onReconnect`, `.every(30.seconds)` |
| Pages | An endpoint that says how it pages gives a read `.items`, `.loadMore()` and `.hasMore` ([Load results a page at a time](#load-results-a-page-at-a-time)) |
| Progress | A `File` parameter makes `Backend.avatar.progress` a number from 0 to 1 |

Beside each endpoint you also get `Backend.users.invalidate()`,
`.prefetch(args)`, `.url(args)` and `.key(args)`. `invalidate()` drops what
the cache holds, and every resource over the endpoint on the page asks again,
so a list redraws after the write that changed it.

### Groups, and what a write makes stale

A service's endpoints can be split into groups by what they are about. A
group is reached through its service and sends what its service sends: its
`base`, `timeout`, `retry` and `middleware`. A group that sets its own is
`N08`.

A write can say which reads it changes with `invalidates`. Each name is a
`get` of the same group, and each argument is one of the write's own
parameters (`N05`). When the write succeeds, every resource on the page over
one of those reads asks again, and the answer is drawn in place:

```wf
type Todo { id: String, title: String }
type NewTodo { title: String }
type Rename { title: String }

api Backend(base: "/api") { }

api Backend.todos {
    get   list() at "todos" -> [Todo]!
    get   byId(id: String) at "todos/{id}" -> Todo!
    post  create(body: NewTodo) at "todos" -> Todo!
    invalidates [list]
    patch rename(id: String, body: Rename) at "todos/{id}" -> Todo!
    idempotent
        invalidates [list, byId(id)]
}

store Drafts {
    action add(title: String) {
        let undo = Backend.todos.list.update(list => list.concat([Todo(id: "new", title: title)]))
        if let .err(_) = await Backend.todos.create(body: NewTodo(title: title)) { undo() }
    }
}

page Todos(path: "/todos", title: "Todos", description: "The list.") {
    resource todos = Backend.todos.list()
    Heading("Todos").h1
    Button("Add bread") {
        on click { Backend.todos.create(body: NewTodo(title: "Bread")) }
    }
    match todos {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for todo in list by todo.id { Text(todo.title) } }
    }
}
```

To show a change before the server agrees, call `.update(change)` on a read.
It changes what every resource over that read shows, and returns a function
that takes the change back. `Drafts.add` in the example above shows the new
todo at once and takes it back if the server refuses it.

A `resource` must be over a read; a resource over a write is `N02`. A call
awaited on its own, with nothing reading its result, is `N04`, because a
failure would go unheard.

### Read the service from an OpenAPI file

When the server publishes an OpenAPI specification (3.0 or 3.1, JSON or
YAML), let it write the contract. `wf api` writes it as files you can read,
review and diff:

```bash
wf api add openapi.json --name Backend
wf api sync
wf api check          # in CI: fails when the files are behind the spec
```

`add` records the service in `webfluent.app.json`, writes
`src/api/backend/service.wf` once, and writes the contract into
`src/api/backend/generated/`:

```text
src/api/backend/
├── service.wf          yours: where the service is and how it is reached
└── generated/          written by `wf api sync`; never edited
    ├── types.wf        every schema: types and enums
    ├── people.wf       api Backend.people { … }, one file a tag
    └── default.wf      the operations with no tag, when there are any
```

```json
{ "apis": { "Backend": { "spec": "openapi.json", "out": "src/api/backend" } } }
```

Every generated file opens with
`// Code generated by `wf api sync` from openapi.json. DO NOT EDIT.`, and
`add` marks the folder `linguist-generated` in `.gitattributes`, so a review
collapses the diff. `sync` rewrites the folder whole, removing the file of a
tag that is gone, and refuses to touch a file in it that does not open with
the marker: it was not written by `sync`. `wf check` warns (`N07`) when the
spec has changed since the files were written. `wf api list` shows every
service, its groups and their endpoints; the compiler never fetches, so the
spec is a file in the project.

For a prototype, name the file with `from` and the compiler reads the same
contract without writing it down:

```wf
api Backend from "openapi.json" (base: env.PUBLIC_API)
```

One reader serves both, so the program is the same either way, and
`wf api eject Backend` turns `from` into files. A missing spec is `E111`.

What a generated group looks like:

```wf
// Code generated by `wf api sync` from openapi.json. DO NOT EDIT.

/// Who is on the team.
api Backend.people {
    /// People whose name matches `q`, a page at a time.
    get listPeople(q: String?, after: String?) at "people" -> PersonPage!
    post createPerson(body: NewPerson) at "people" -> Person ! CreatePersonError
    get getPerson(id: String) at "people/{id}" -> Person ! GetPersonError
    delete deletePerson(id: String) at "people/{id}" -> ! DeletePersonError
}
```

How an operation becomes an endpoint:

- **Its group is its first tag**, camelCased (`Pet Store` is `petStore`);
  one with no tag is in `default`. Call it as `Backend.people.listPeople(…)`.
- **Its name is its `operationId`**, made a name only where it must be
  (`list-users` is `listUsers`). One with none is named from its method and
  path (`getUsersId`), with a warning to give it one. Two of one name stop
  the read, naming both.
- **Its parameters go where the spec puts them**: the path, the query, a
  header, a cookie. One whose name is not a name the program can write is
  camelCased, and keeps the server's name where it goes:
  `pageSize: Number = 20 in query "page-size"`,
  `sid: String in cookie "session_id"`. A schema's `default` is the
  parameter's.
- **Its body and its answer are read by media type**: JSON first, then text
  (`String`, `as: .text`), then bytes (`File`, `as: .blob`). A
  `multipart/form-data` body with one file is that file and the rest of its
  fields. A body or an answer the language cannot send or read (XML) stops
  the read with the operation named; `wf api sync --skip NAME` leaves it
  out, and the config remembers.
- **Every error status it declares is a case of its error enum**, named for
  the operation (`CreatePersonError`) and the status's reason phrase:

  ```wf
  enum CreatePersonError {
      conflict(problem: Problem) on 409
      unprocessableContent(problem: ValidationProblem) on 422
      http(error: HttpError)
  }
  ```

  A `4XX` or `5XX` response is `clientError` or `serverError`, written
  `on 4XX`: every status of its class no other case names. `default` is not
  a case; it is what `.http(.status(code, problem))` already holds. An
  operation with no error statuses is `-> T!`.

How each schema becomes a type:

| The schema | What the program gets |
|---|---|
| An object: `{ "type": "object", "properties": … }` | A `type` with those fields; a field the schema does not `require` is `T?`; a field named as no program can write is quoted, `"first-name": String?` |
| An object or a string with cases written in place | A record or an enum named by its place: a request body `CreatePersonBody`, a field `PersonAddress`, an item `PersonTagsItem` |
| An `allOf`: `[{ "$ref": "…/EnvSummary" }, { "properties": … }]` | `type ServerDetail = EnvSummary { … }`: it extends the first record it names, and every other part adds fields of its own |
| A string with cases: `{ "type": "string", "enum": ["prod", "test"] }` | `enum Env { prod, test }`, whose cases are the strings the server sends |
| A string with a case no name can spell: `"operate.flags"` | `String`, wherever the schema is named |
| `additionalProperties` with a schema | `[String: T]` |
| A shape the schema does not say — `{}`, an object with no fields, a `oneOf` of several types | `Json`: matched or decoded where it is read, never `Any` |
| Anything else: a string, a number, a list, another schema's `$ref`, a `oneOf` of one type and `null` | That type, wherever the schema is named |

A value that may be null becomes its type made optional, however the
document says so:

- OpenAPI 3.1's `"type": ["string", "null"]` is `String?`;
- a `oneOf` of `{ "type": "null" }` and a `$ref` is that type's `?`;
- 3.0's `"nullable": true` is the same, beside a `type`, an `allOf` or a
  `$ref`;
- a record or an enum whose own schema allows null is `T?` wherever it is
  named.

A string's `format` gives the language's type for what the server sends:

| `format` | In a response, and in every record only responses reach |
|---|---|
| `date-time` | `DateTime` |
| `date` | `Date` |
| `uuid` | `Uuid` |
| `email` | `Email` |
| `uri` | `Url` |
| `time` | `String`, everywhere: JSON Schema's carries an offset (`09:30:00Z`) that `Time` does not read |

What the program sends — a parameter, a request body, and every record one
of them reaches, except for the fields marked `readOnly` — keeps `String`.
So a value the program holds as a `String`, such as a route parameter or
what an input holds, can still be sent.

Customise beside the generated files, never inside them:

- A `type` or an `enum` the program declares itself wins over the
  specification's of the same name.
- An endpoint the spec lacks goes in a group of the project's own:
  `api Backend.extras { … }`.
- What a write makes stale and how a read pages go in the service's own
  file, each naming an endpoint as `group.endpoint`, and are checked as if
  written on it (`N05`, `N14`):

  ```wf
  api Backend(base: env.PUBLIC_API) {
      invalidations {
          people.createPerson invalidates [people.listPeople]
          people.deletePerson(id) invalidates [people.listPeople, people.getPerson(id)]
      }
      pagination {
          people.listPeople pages .cursor(param: after, next: p => p.nextCursor, items: p => p.items)
      }
  }
  ```

  A team that owns its spec may say the same there instead, with
  `x-invalidates: [listPeople, "getPerson(id)"]` and
  `x-pagination: { style: cursor, param: after, next: nextCursor, items: items }`
  on the operation.

The editor reads the spec as the build does, so a call of one of its
endpoints is known in hover, completion and the problems list.

### Service settings

These go in the service's parentheses or at the top of its block, before
the endpoints:

| Setting | Values | Default |
|---|---|---|
| `base:` | The address every path is relative to: `"/api/v1"`, `env.PUBLIC_API` | — |
| `timeout:` | A duration after which a call gives up with `.timeout`: `10.seconds` | `30.seconds` |
| `retry:` | A count (`3`), `.never`, or `.backoff(times: 3, on: [.network, .timeout, .status5xx], delay: 300, max: 30000, jitter: true)` | no retries |
| `cache:` | Overrides what the server says: `.maxAge(5.minutes)`, `.swr(60.seconds)` (serve what is held, revalidate behind it), `.forever`, `.none` | what the server's `Cache-Control` says |
| `credentials:` | `.omit`, `.sameOrigin`, `.include` — whether cookies go with a call | the browser's (`.sameOrigin`) |
| `mode:` | `.cors`, `.noCors`, `.sameOrigin` | `.cors` |
| `redirect:` | `.follow`, `.error`, `.manual` | `.follow` |
| `referrer:` | A referrer policy: `.noReferrer`, `.origin`, … | the page's |
| `middleware [ … ]` | The steps every request passes through, in order ([Pass every request through middleware](#pass-every-request-through-middleware)) | none |

How `retry` works:

- Its `on:` takes `.network`, `.timeout`, `.status5xx`, `.status4xx`, and
  single codes in the form `.status429` or `.status503`.
- Without `on:`, it retries network failures, timeouts, 5xx responses and
  429.
- The wait doubles from `delay` (in milliseconds) up to `max`, with jitter.
  A `Retry-After` from the server wins.
- An aborted call is never retried.
- Only a request that is safe to send twice is retried: `GET`, `HEAD`,
  `PUT` and `DELETE`. A `POST` or `PATCH` that timed out may have reached
  the server, and a second one could be a second order, so it fails instead
  — unless its endpoint is marked `idempotent`: `post order(body: Order) at "order" ->
  Order! idempotent`. That sends one `Idempotency-Key` with every try of a
  call, so a server that keeps the key does the order once, and the call is
  retried, and kept to replay offline, like a `PUT`. A `retry:` on a call of
  any other write is `N09`.

### Declare endpoints

Each line of a service, after its settings, is an endpoint:

```text
get users(page: Number = 1, q: String?) at "users" -> [User]!
get user(id: String) at "users/{id}" -> User ! LookupError
post createUser(body: NewUser) at "createUser" -> User ! CreateUserError
put rename(id: String, body: Rename) at "users/{id}" -> User!
patch settings(body: Settings) at "settings" -> Settings!
delete removeUser(id: String) at "users/{id}" -> !
get report(id: String) at "reports/{id}" -> Json! as: .blob
```

- **The verb** is `get`, `post`, `put`, `patch`, `delete`, `head` or
  `options`.
- **The path.** `at` is required: where the server answers the endpoint,
  below the service's `base`.
- **Path parameters.** A parameter the path names (`{id}`) goes in the path,
  and every `{name}` must be one of the endpoint's parameters (`N01`).
  One that is empty (`null` or `""`) is not sent at all, since `users/`
  would be the whole collection, not the user; nor is one that is `.` or
  `..`, which the browser reads as this folder or the one above. The call
  ends as `.cancelled`, and a resource over it stays `.loading` until the
  value arrives.
- **The body.** A parameter called `body` is the request body; a map is
  sent as JSON. A `File` parameter makes the call a multipart upload.
- **Query parameters.** Every other parameter goes in the query string, and
  one that is `null` is left out.
- **Headers and cookies.** `token: String in header "X-Token"` sends a
  parameter as a header, and `theme: String? in cookie` as a cookie.
- **The first parameter** may be passed without its name:
  `Backend.user("7")`.
- **What comes back.** `-> T!` is a success's `T`, or an `HttpError`;
  `-> T ! E` names the failures in an error enum `E`, each status a case
  with its typed body; `-> !` returns nothing, as `204 No Content` does, and
  may fail. Under `wf serve` the body that arrives is held to `T`, and a
  declared status's body to its case's type. What crosses is a type the
  program names: `Map` and `Any` there are `N11`.
- **Other responses.** `as:` says how to read a response that is not JSON:
  `.text`, `.blob`, `.arrayBuffer` or `.none`. `errorAs:` does the same for
  a failure's body. `cache:` sets a cache policy for that endpoint alone.

A call passes the endpoint's parameters by name, and one with no default
that may not be null must be given; one it leaves out that has a default
is sent with that default. Called in an action, it is a promise of
a result: `await` it, then `match` it or pass its failure up with `try`
([Say how a call fails](#say-how-a-call-fails)). It may add `cache:`,
`headers:`, `timeout:` and `retry:`, and in a `resource` also `refresh:`
(`.onFocus`, `.onReconnect`, `.every(30.seconds)`, `.never`) and
`placeholder:`.

### Say how a call fails

An endpoint that says how it fails returns a *result* rather than throwing:
`-> T ! E` is a `T`, or an error of the *error enum* `E`, and `-> T!` is a
`T` or an `HttpError`. An error enum names each status the service answers
with as a case of its own, carrying the body that status sends, and keeps
one case, `http(error: HttpError)`, for everything else:

```wf
type User { id: String, name: String }
type NewUser { name: String }
type FieldError { field: String, message: String }
type ValidationProblem = Problem { errors: [FieldError] }

/// Why adding someone can be refused.
enum CreateUserError {
    unprocessableContent(problem: ValidationProblem) on 422
    conflict(problem: Problem) on 409
    http(error: HttpError)
}

api Backend(base: "/api") {
    post createUser(body: NewUser) at "users" -> User ! CreateUserError
    get ping() at "ping" -> !
}

store People {
    state said = ""

    /// 1. Every case handled where the call is made.
    action submit(name: String) {
        match await Backend.createUser(body: NewUser(name: name)) {
            .ok(u) { said = "Added {u.name}" }
            .err(.unprocessableContent(p)) { said = p.errors.map(e => e.message).join(", ") }
            .err(.conflict(_)) { said = "Already there" }
            .err(.http(.status(503, _))) { said = "The service is busy" }
            .err(.http(e)) { said = e.message }
        }
    }

    /// 2. The failure passed up, as the action's own.
    action invite(name: String) -> User ! CreateUserError {
        let user = try await Backend.createUser(body: NewUser(name: name))
        return user
    }
}

page Team(path: "/people", title: "People", description: "Adding someone, and why it can fail.") {
    use People
    state name = ""
    Heading("People").h1
    Input(bind: name, label: "Name")
    Button("Add") { on click { People.submit(name) } }
    Button("Invite") { on click { People.invite(name) } }
    Text(People.said)
    // 3. The last outcome of an action, read where it is shown.
    if let .err(.unprocessableContent(p)) = People.invite.result {
        Alert(p.title ?? "Check the form").danger
    }
}
```

- **An error enum.** Each case but `http` names its status, `on 422`, and
  carries one value, the body of that status. An enum that names a status
  is held to that shape (`N12`).
- **`HttpError`** is what goes wrong between the page and the server:
  `.offline(queued)`, `.timeout(after)`, `.cancelled`, `.transport(message)`,
  `.decode(at, message)` and `.status(code, problem)`. Every one has
  `.message`, for showing. `queued` is true for a write kept to send when
  the connection returns ([Keep writes made offline](../recipes/keep-writes-made-offline.md)).
- **`Problem`** is a server's account of a failure as RFC 9457 has it —
  `type`, `title`, `status`, `detail`, `instance`, `extensions` — and a
  project extends it like any record. A status the enum does not name
  carries it as its `problem` when the server sends
  `application/problem+json`.
- **A result** is `.ok(value)` (`.ok` when there is none) or `.err(error)`.
  A `match` on one gives every case an arm, or an `else`; an arm may look
  inside a case, `.err(.http(.status(503, _)))`, where `_` takes anything
  and a literal takes that value. `match` runs in an action or a handler
  too, and `if let .err(e) = outcome { … }` takes one case.
- **`try`** gives a result's value, or returns its error from the action as
  the action's own, which must say what it fails with (`N03`). The same
  error passes up as it is, an `HttpError` becomes an error enum's `.http`,
  and another error enum passes up through its `http` case once its other
  cases are handled. In a handler, `try` reports a failure instead.
- **An action that may fail** returns its value, which is `.ok` of it;
  `return .err(e)` hands back a failure of its own. Its `.result` is its
  last outcome, or `null` before it has run.
- **A `resource`** over such an endpoint holds its value, as it always has.

### Load results a page at a time

How an endpoint pages is part of its contract, so you say it once, on the
endpoint, with `pages`. A read of it then holds every page loaded so far.
This page shows items and a **Load more** button while there are more:

```wf
type Item { id: String, name: String }

api Catalog(base: "/api") {
    get items(page: Number = 1) at "items" -> [Item]!
        pages .number(param: page)
}

page Items(path: "/items", title: "Items", description: "Everything, a page at a time.") {
    resource list = Catalog.items()

    Heading("Items").h1
    for item in list.items by item.id { Text(item.name) }
    if list.hasMore {
        Button("Load more", loading: list.pending) { on click { list.loadMore() } }
    }
}
```

- `.items` is every item loaded so far, in order — the same list as the
  read's `.value` and its `.loaded(items)` — so `item.name` is checked
  against `Item`.
- `.loadMore()` asks for the next page, as a refresh: what is shown stays
  while it loads. It does nothing while a request runs or when there is no
  next page.
- `.hasMore` is whether there is a next page.
- `reload()`, `refresh:` and an `invalidates` ask again for every page
  loaded, in order, and show them together. A call with other arguments
  starts again from the first page.

The read keeps the page number itself; nothing on the page holds it. There
are three ways an endpoint can page:

| Written | Asks for the next page with | Stops when |
|---|---|---|
| `pages .number(param: page)` | the parameter, one more each time | a page comes back empty |
| `pages .cursor(param: after, next: p => p.nextCursor)` | the cursor the last page gave | `next:` gives `null` |
| `pages .link` | the address the answer's `Link` header names `rel="next"` (RFC 8288) | no `Link` names one |

`param:` names one of the endpoint's parameters that the first call need
not give: it has a default, or may be `null`. When a page is not a list
itself, `items:` says where its items are:

```wf
type Post { id: String, title: String }
type FeedPage { posts: [Post], nextCursor: String? }

api Social(base: "/api") {
    get feed(after: String?) at "feed" -> FeedPage!
        pages .cursor(param: after, next: p => p.nextCursor, items: p => p.posts)
}

page Feed(path: "/feed", title: "Feed", description: "What is new, a page at a time.") {
    resource feed = Social.feed()

    Heading("Feed").h1
    for post in feed.items by post.id { Text(post.title) }
    if feed.hasMore {
        Button("More", loading: feed.pending) { on click { feed.loadMore() } }
    }
}
```

A numbered pager, `Pagination(bind: page)`, is a different thing: each
page is a read of its own, `Catalog.items(page: page)` with no `pages`, and
going back to one is a cache hit when the server allows caching.

### Read an answer that arrives a line at a time

An endpoint that answers one JSON value a line — a log tail, a build's
output, NDJSON — is marked `streamed`, and says the list its lines make:

```wf
type LogLine { at: String, text: String }

api Deploys(base: "/api") {
    get tail(id: String) at "deploys/{id}/log" -> [LogLine]! streamed
}

store Follow {
    state last = ""
    action watch(id: String) {
        for await line in Deploys.tail(id: id) { last = line.text }
    }
}

page Log(path: "/log", title: "Log", description: "A deploy's log, as it is written.") {
    use Follow
    resource lines = Deploys.tail(id: "7")

    Heading("Deploy 7").h1
    match lines {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(all) { for line, i in all by i { Text("{line.at} {line.text}") } }
    }
    Button("Follow") { on click { Follow.watch("7") } }
    Text(Follow.last)
}
```

- A `resource` over it is `.loaded` from the first line, and its list
  grows as the rest arrive; `.pending` is true until the answer ends. A
  refresh gathers the lines again and shows them when they are all there.
- `await Deploys.tail(id: …)` is every line, as one result.
- `for await line in Deploys.tail(id: …)`, in an action or a handler,
  runs its body for each line as it arrives; `for await line, i in …`
  counts them.
- Each line is held to the item's type, `LogLine` here; one that is not
  fails as `.decode(at, message)`, naming the line.

`pages` and `streamed` are a read's: a write is sent once (`N14`).

### Call an API on another origin

A page on `https://example.com` that calls `https://api.example.com` makes a
*cross-origin request*. The API must answer with CORS headers —
`Access-Control-Allow-Origin`, and `Access-Control-Allow-Credentials` if
cookies go with the call — or the browser refuses the response.

The same applies while you develop. `wf serve` answers on `localhost:3000`
and has no proxy, so an API on `localhost:8080` must allow that origin.

The alternative is to serve the API and the site from one origin behind a
reverse proxy, where `base: "/api"` needs no CORS at all
([Deploying](../recipes/api-on-the-same-origin.md)).

If the page's build ships a content security policy, list the API's origin
in `meta.connect` too ([Security](../about/security.md)).

### Show a change before the server agrees

To make an edit appear at once instead of after the round trip, call
`optimistic(holder, change)` before the request. If anything later in the
action throws, the change is taken back: the action rolls itself back.

```wf
action rename(id: String, name: String) {
    optimistic(Todos.items, items => items.map(i => if i.id == id { { ...i, title: name } } else { i }))
    await Backend.updateUser(id: id, body: { name: name })
    Backend.users.invalidate()
}
```

The new title shows at once. If `updateUser` fails, the old title comes
back.

## Ask on demand with `http`

For a request made when something happens — a save, a delete, a step in a
flow — call the built-in service `http` in an action: `http.get(url)`,
`http.post(url, body: …)`, `http.put`, `http.patch`, `http.delete`. Its
answer is a *result*, `.ok(value)` or `.err(e)` with an `HttpError`, so a
failure is a value you handle rather than an exception you catch: pass it
up with `try`, or `match` it where the call is made.

This page loads a list of todos, adds one, and toggles one, each with a
request:

```wf
type Todo { id: String, title: String, done: Bool = false }
type NewTodo { title: String }
type Change { done: Bool }

page Todos(path: "/todos", title: "Todos", description: "Todos saved to a server.") {
    state todos: [Todo] = []
    state draft = ""
    state error = ""

    action load() -> ! {
        todos = try await http.get("/api/todos")
    }

    action add() {
        if draft.trim() == "" { return }
        let saved: Todo! = await http.post("/api/todos", body: NewTodo(title: draft))
        match saved {
            .ok(created) {
                todos.push(created)
                draft = ""
            }
            .err(e) { error = "Could not save: {e.message}" }
        }
    }

    action toggle(id: String) -> ! {
        let found = todos.find(x => x.id == id)
        if let todo = found {
            let updated: Todo = try await http.patch("/api/todos/{id}", body: Change(done: !todo.done))
            todos = todos.map(x => if x.id == id { updated } else { x })
        }
    }

    load()
    Heading("Todos").h1
    Input(bind: draft, label: "New todo") { on key("Enter") { add() } }
    Button("Add", disabled: add.pending).primary { on click { add() } }
    if error != "" { Alert(error).danger }
    for todo in todos by todo.id {
        Checkbox(checked: todo.done, label: todo.title) { on change { toggle(todo.id) } }
    }
}
```

- **What comes back is named where it goes**: `todos = try await
  http.get(…)` is a `[Todo]` because `todos` is, `let updated: Todo` names
  it, and `let saved: Todo!` names a whole result. A `get` whose answer goes
  nowhere that says what it is is `N11`; a write whose answer nothing reads
  needs no type.
- **`try`** in `load` and `toggle` gives the answer, or ends the action with
  its failure as the action's own: each says what it fails with, `-> !` —
  nothing, or an `HttpError`. `add` instead matches the result and shows
  the failure.
- `body:` is what is sent, as JSON; `headers:`, `timeout:`, `retry:` and
  `cache:` go beside it, and `as: .text` reads the answer as text.
- `load()` at the top of the page is set-up code: it runs once, when the
  page renders.
- `todos.push(created)` changes the state in place. The compiler turns it
  into a new list, so everything that reads `todos` repaints.
- `name.pending` is `true` while the action runs, so **Add** is disabled
  during a save.
- `http` goes through the same engine as an `api` call. `fetch` is the
  browser's own, `window.fetch`, with the raw response.

## Read a file at build time with `data`

To use content that is known before the site is built — posts, a price
list, a team page — read it with `data`. `data name = "file.json"` reads a
JSON file from the project directory or `src/` at build time, and makes its
value a constant.

A `data` constant:

- is in scope in every page and component;
- costs nothing at run time, because its value is in the bundle;
- is written into the pre-rendered page.

This blog lists every post in `posts.json`, newest first:

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page Blog(path: "/", title: "Blog", description: "Every post.") {
    derived sorted = posts.sortBy(p => p.date).reverse()

    Heading("Blog").h1
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

Annotate it (`data posts: [Post]`) and the checker knows every field —
and the build holds the file to the type when it reads it. A record that is
not a `Post` stops the build with `T33`, naming the file and the place:
`` `posts.json` is not what `data posts` says it is: $[3].date is "March 5",
not a Date ``.

### Render a page for each item

To give each post its own HTML file in a static build, list the values of
the route parameter in `paths:`. The build renders the page once per value,
each with its own title and description, and lists each in the sitemap:

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

The build writes `posts/hello/index.html` and `posts/second/index.html`.

- `paths:` must be known at build time: a `data` file, a `const` or a
  literal.
- For a route with several parameters, each value is a map naming them:
  `paths: pairs.map(x => { year: x.y, slug: x.s })`.
- A route not in `paths:` still works in a single-page app. Only the static
  build needs the list.

## Lists of data: a table, a tree, a search

Three built-ins draw rows from a list you hand them, each row with a block
you write for it:

```wf
type Deploy { id: String, service: String, status: String }
type Node { path: String, name: String, children: [Node] = [] }

page Ops(path: "/", title: "Ops", description: "Deploys, files and a search.") {
    state deploys: [Deploy] = [Deploy(id: "1", service: "web", status: "live")]
    state chosen: [String] = []
    state files: [Node] = [Node(path: "src", name: "src", children: [Node(path: "src/App.wf", name: "App.wf")])]
    state file = ""
    state who = ""
    Heading("Ops").h1
    DataTable(rows: deploys, key: d => d.id, bind: chosen, label: "Deploys").multiple {
        empty { Text("No deploys yet") }
        DataTable.Column("Service", value: d => d.service).sortable
        DataTable.Column("Status") { cell(d) { Badge(d.status) } }
    }
    Tree(items: files, children: f => f.children, key: f => f.path, bind: file, label: "Files") {
        row(f) { Text(f.name) }
    }
    Combobox(bind: who, options: ["Ada", "Alan", "Grace"], text: p => p, label: "Owner")
}
```

- **`DataTable`** declares its columns once. A `.sortable` column's header
  sorts the rows and says which way (`aria-sort`); `cell(d) { … }` draws a
  cell; `empty { … }` shows with no rows; `bind:` chooses rows by `key:`.
- **`Tree`** walks `children:` from `items:`. The arrow keys open, close and
  move through it as a file tree does; `bind:` holds the chosen item's key.
- **`Combobox`** offers the `options:` that match what is typed, or asks
  `search: q => …` once the typing rests; `option(o) { … }` draws an offer.

Named blocks (`empty`, `row`, `cell`, `option`) come before an element's
children. Each of these is drawn once the page runs; the static paint holds
its place.

## Where data lands in the pre-rendered page

A static build writes each page's HTML before any script runs. What each
source of data puts in that HTML:

| Source | In the pre-rendered HTML |
|---|---|
| `data`, `const`, `env` | Fully painted |
| `state` initial values, and `derived` values over them | Painted |
| `resource` | The `.loading` arm; the live page fetches and replaces it |
| `http` in an action | Nothing until the action runs |

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| The browser console says a response was blocked by CORS | The API is on another origin and does not allow yours. Add CORS headers to the API, or serve both from one origin ([Call an API on another origin](#call-an-api-on-another-origin)). |
| A request gets a `404` under `wf serve`, with a line in the terminal | `wf serve` has no file or page for that address, and no proxy. Point `base:` at the real API. |
| A `.parse` error naming a place, such as `$[1].name is missing` | Under `wf serve`, a response is checked against its declared type. The server sent another shape: fix the server or the `type`. |
| A resource stays `.loading` | A path parameter is `null` or `""`, so the call is not sent. It goes out when the value arrives. |
| The page goes back to `.loading` when state changes | The address now names something else, so it is a first load. A refresh of the same address keeps the value; `placeholder: .previous` keeps it for a new address too. |
| `T05` on a field of the data | The declared type has no such field. Fix the name, or add the field to the `type`. |
| `T06` or `T10` on an endpoint call | The service has no such endpoint, or the endpoint takes no such argument. |
| A post's page is missing from a static build | Its value is not in `paths:`. |

## Quiz

```quiz
A resource's `match` has a `.loading` arm and a `.loaded` arm, and no other. What happens?
- The runtime draws an error message for you when the request fails -- The runtime draws only the arms you write.
+ The build stops with T21 -- A failed first load is `.failed(e)`, which no arm takes, so the place would stay empty. Add `.failed(e) { … }` or an `else`.
- The `.loading` arm shows until a retry succeeds -- A first load that fails is `.failed(e)`, not `.loading`.
```

```quiz
A resource is making its very first request. Which is true?
+ `pending` is true and `isRefreshing` is false -- `pending` covers every request of it, the first load included; `isRefreshing` only a request made again while a value is shown.
- `isRefreshing` is true and `pending` is false -- It is the other way round: there is no value yet to refresh.
- Both are true -- `isRefreshing` is true only while the read is `.loaded`, and on a first load it is `.loading`.
```

```quiz
A resource with `refresh: .every(30.seconds)` asks again while its list is on screen. What does its `match` show while that refresh runs?
- The `.loading` arm, then the new list -- Only a first load, or a request whose address changed, shows `.loading`.
- An empty list until the answer arrives -- The value stays while a refresh runs; `name.value` is never cleared by one.
+ The `.loaded` arm with the old list, then the answer drawn in place -- A refresh stays `.loaded`, with `refreshing` true; rows whose key is still there keep their nodes.
```

```quiz
The address is `http.get("/api/servers?env={stage}")`, and a button sets `stage` from `"test"` to `"prod"`. What shows next?
+ The `.loading` arm, then the production servers -- The request now names something else, so it is a first load; the test servers would be the wrong data.
- The test servers, with `refreshing` true, until the production servers arrive -- That is a refresh, which asks again for the same request; this one changed. `placeholder: .previous` would ask for it.
- Nothing changes until the reader reloads the page -- An address that reads state is fetched again whenever that state changes.
```

## Common mistakes

**A `match` with no `error` arm.** When the request fails, no arm draws, and
the reader is not told:

```wf expect=T21
page P(path: "/", title: "Users", description: "Everyone.") {
    resource users = http.get("/api/users")
    Heading("Users").h1
    match users {
        .loading { Spinner }
        .loaded(list) { Text("{list.length} users") }
    }
}
```

Add `error(e) { Alert(e.message).danger }`, or an `else`
([T21](../errors/T21.md)).

**A field the declared type does not have.** Once a resource is typed, every
field read from it is checked against the `type`:

```wf expect=T05
type User { id: String, name: String }

page P(path: "/", title: "Users", description: "Everyone.") {
    resource users: [User] = http.get("/api/users")
    Heading("Users").h1
    match users {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for u in list by u.id { Text(u.fullName) } }
    }
}
```

Use the field the type declares, `u.name`, or add `fullName` to the `type`
if the server sends it ([T05](../errors/T05.md)).

**Reading `.value` as if it had arrived.** Before the first answer,
`name.value` is `null`:

```wf expect=T04
type User { id: String, name: String }

page P(path: "/", title: "Users", description: "Everyone.") {
    resource users: [User] = http.get("/api/users")
    Heading("Users").h1
    Text("{users.value.length} users")
}
```

Read it through `?.` with a fallback, `{users.value?.length ?? 0}`, or show
it inside the `.loaded` arm, where the binding is never null
([T04](../errors/T04.md)).

Going further: [Open a socket](../recipes/open-a-socket.md) keeps a
connection open for data the server pushes.

## Next

[Testing](16-testing.md).
