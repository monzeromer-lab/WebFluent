# Fetching data

> Describe an API once with retries, caching and typed errors; show a resource with match; ask with http in an action; update optimistically; read data files.

The page: https://webfluent.monzeromer.dev/learn/15-data

This chapter is about getting data onto a WebFluent page and sending data
back: from a request the page makes when it shows, from a request an action
makes on demand, and from a file the build reads.

You will be able to:

- show a request's loading, error and ready states with `resource` and
  `match`;
- refresh data without the page going blank;
- describe a service once with `api`, so every call is typed, retried,
  cached and cancelled for you;
- read a service's surface from an OpenAPI file;
- ask on demand from an action, and show a change before the server
  agrees;
- read a JSON file at build time and render one page per item in it.

## Before you start

You should know `state` and `action`
([State and reactivity](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md)), `match`
([Control flow](https://webfluent.monzeromer.dev/learn/09-control-flow.md)) and how to declare a `type`
([Types](https://webfluent.monzeromer.dev/learn/12-types.md)).

This chapter does not cover:

- a connection the page holds open — a socket, a stream, a channel or a
  peer — which is [Open a socket](https://webfluent.monzeromer.dev/recipes/open-a-socket.md);
- working without a network, which is [Make a site work offline](https://webfluent.monzeromer.dev/recipes/work-offline.md);
- values fixed for each build, which are
  [Environments and secrets](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md).

## Choose how to get data

| You want… | Use | Section |
|---|---|---|
| Data the page shows as soon as it opens | `resource` | [Show a request with resource](#show-a-request-with-resource) |
| One place that describes a service and its endpoints | `api` | [Describe a service with api](#describe-a-service-with-api) |
| A request made when something happens: a save, a delete, a step in a flow | `http.post(…)` in an action | [Ask on demand with http](#ask-on-demand-with-http) |
| Content that is known at build time | `data` | [Read a file at build time with data](#read-a-file-at-build-time-with-data) |

## Show a request with `resource`

To fetch data while a page shows, declare a `resource` and show its states
with `match`. A *resource* is a request the page makes, and a value that
says where the request is: loading, failed or ready.

This page lists recent deploys, with a spinner while they load and an alert
if the request fails:

```wf
type Deploy { id: String, status: String, startedAt: String }

page Deploys(path: "/deploys", title: "Deploys", description: "Recent deploys.") {
    resource deploys: [Deploy] = http.get("/api/deploys")

    Heading("Deploys").h1
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

> [!NOTE]
> **Coming from React?** There, a fetch is a `useEffect` that sets
> `loading`, `error` and `data` states by hand, or a `useQuery`. A
> `resource` is the request and its state together: no effect, no flags to
> keep in step, and a `match` with one arm per state. See
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

How it works:

- `resource name: T = http.get(url)` starts the request when the page renders.
- The annotation (`: [Deploy]`) types the value. Without one, the value is
  `Any`.
- A `match` over a resource has three arms — `loading`, `error(e)` and
  `ready(v)` — and may have an `else`. A resource `match` with no `error`
  arm and no `else` is `T21`.
- The pre-rendered page — the HTML the build writes for a static site —
  shows the `loading` arm. The live page fetches and replaces it.
- A failed response (one that is not `ok`) is an error whose `e.message` is
  `HTTP 500`, or whatever the status was.

### Read a resource's state anywhere

A resource has these values. Each is live wherever you read it — a label, a
splice, an attribute — with no `derived` in between:

| Value | What it is |
|---|---|
| `name.state` | `"loading"`, `"ready"` or `"error"` |
| `name.data` | The last value, or `null` before the first (`T?`) |
| `name.error` | The last error |
| `name.refreshing` | `true` while a refresh runs ([Refresh without clearing the page](#refresh-without-clearing-the-page)) |
| `name.pending` | `true` while any request of it runs, the first load included |
| `name.reload()` | Makes the request again |
| `name.invalidate()` | Drops what the cache holds for it, and asks again |
| `name.cancel()` | Aborts the request that is running |

So `Button(if plan.data != null { "Request approval" } else { "Deploy" })`
changes its label when the plan arrives.

### Refresh without clearing the page

These ask for a resource again:

- the `on:` option of `http.get(…)` or of an endpoint's call: `.focus` (when
  the reader comes back to the tab), `.reconnect` (when the network
  returns), or `.interval(30.seconds)`;
- `name.reload()` and `name.invalidate()`.

Asking again for the request already shown is a *refresh*. The page stays
as it is while a refresh runs: `name.state` stays `"ready"`, the data stays,
and the answer is drawn in place.

This page polls a list of servers every 30 seconds, and shows a small
spinner during each refresh:

```wf
type Server { id: String, name: String, status: String }

page Servers(path: "/servers", title: "Servers", description: "Every server, kept current.") {
    state stage = "test"
    resource servers: [Server] = http.get("/api/servers?env={stage}", on: .interval(30.seconds))

    Heading("Servers").h1
    Button("Production") { on click { stage = "prod" } }
    match servers {
        loading { Skeleton(height: "20px") }
        error(e) { Alert(e.message).danger }
        ready(list) {
            if servers.error != null { Alert("Could not refresh; this is what was there.").warning }
            if servers.refreshing { Spinner.sm }
            for s in list by s.id { Text("{s.name}: {s.status}") }
        }
    }
}
```

> [!NOTE]
> **Coming from TanStack Query?** `isLoading` is `name.state == "loading"`,
> true only while there is nothing to show; `isFetching` is `name.pending`;
> and `name.refreshing` is a fetch while data is on screen. See
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

What a refresh keeps:

- **The arm.** The `ready` arm's binding is live. Text, attributes, `if`,
  `if let`, a keyed `for … by` and component props inside the arm follow the
  new data. The arm is drawn again only when the arm changes — loading,
  ready, error.
- **The rows.** A row whose key is still in the list keeps its nodes, its
  focus and its components' state. An input keeps what was typed.
- **Unchanged data.** The parts of an answer equal to what was shown are
  kept as the same objects. An item is matched by its `id`, by its value, or
  by its place. An answer that changed nothing redraws nothing.

What happens when a request fails, or changes:

- A refresh that fails keeps the data and `"ready"`, and sets `name.error`
  until the next answer, so the arm can say the data is stale.
- A first load that fails is `"error"`, and its `reload()` is a first load
  again.
- A request that changed is a first load again, shown as `loading`. A
  request changes when an address or an argument that reads state now names
  something else — another environment, another page — because the data on
  screen would be the wrong data. What the last request brought goes with
  it: `name.data` is `null` (and a paged resource's `name.items` empty)
  until the answer arrives, and stays so if it fails, so a `derived` value
  over it never reads the other request's answer. With `paginate:`, the
  next page is a refresh instead.

Pressing **Production** above changes `stage`, so the list shows its
`loading` arm and then the production servers.

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
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} hits") }
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
        loading { Skeleton }
        error(e) { Alert("Not found").warning }
        ready(p) {
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
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} results for {submitted}") }
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

    get    users(page: Number = 1, q: String?)  -> [User]!
    get    user(id: String) at "users/:id"      -> User!
    post   createUser(body: NewUser)            -> User ! CreateUserError
    delete removeUser(id: String) at "users/:id" -> !
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

- A parameter the path names goes in the path. Every other parameter goes in
  the query string.
- `->` says what a success returns and how a call fails: `-> [User]!` is a
  list of users or an `HttpError`, `-> User ! CreateUserError` a user or a
  case of that error enum, `-> !` nothing but may fail ([Say how a call
  fails](#say-how-a-call-fails)). A `resource` over the endpoint holds what
  a success returns, so `u.nam` is `T05` before the page ever runs.
- An endpoint the service does not have is `T06`, and an argument it does
  not take is `T10`.

### Add headers and hooks to every call

To send the same headers with every call, add a `headers { }` block. Each
value is read at the moment of the request, not when the service is
declared, so a token refreshed a moment ago is the one that is sent.

This service sends a bearer token and the reader's language, tags each
request with an ID, logs each response, and refreshes the token on a `401`:

```wf
type User { id: String, name: String }

store Session {
    state token = ""
    state language = "en"
    action refresh() -> ! { token = try await http.get("/api/token", as: .text) }
}

api Backend(base: env.PUBLIC_API ?? "/api/v1") {
    headers {
        Authorization: "Bearer {Session.token}"
        Accept-Language: Session.language
    }

    on request(r)  { r.headers["X-Request-Id"] = uuid() }
    on response(r) { log("{r.status} {r.url}") }
    on error(e)    { if e.status == 401 { await Session.refresh()  return "retry" } }

    get me -> User!
}
```

The three hooks run around every call the service makes:

| Hook | When it runs | What it gets | What returning does |
|---|---|---|---|
| `on request(r)` | Before the request is sent, after `headers { }` | `r.url`, `r.method`, `r.headers`, `r.body` — change them in place | The changed request is what goes out |
| `on response(r)` | After a reply arrives, before it is decoded | `r.status`, `r.headers`, `r.url` | Nothing; it is for recording |
| `on error(e)` | After every retry the policy allows has failed | The error, as [What every call does for you](#what-every-call-does-for-you) describes it | `return "retry"` runs the call once more, which is how a `401` refreshes a token and carries on. Anything else lets the error through. |

A hook may `await`. A hook that throws fails the call it was watching.

### What every call does for you

| Feature | What it does |
|---|---|
| Abort | A call is aborted when its scope leaves, when an argument that reads state changes, on `.cancel()`, and on timeout |
| Errors | `.offline`, `.timeout`, `.aborted`, `.parse`, `.network`, `.status(code, body)` — each with `.message`, `.status`, `.body` and `.headers` to read |
| Retry | Exponential backoff with jitter, per error kind, honouring `Retry-After` — for a request that is safe to send twice |
| Cache | `cache: .swr(60.seconds)`, `.none`, `.forever` — keyed by method, address, body and credentials, revalidated with `ETag`, and holding the 256 answers read most recently |
| Dedupe | Two readers of one address make one request, cached or not |
| Refetch | `on: .focus`, `.reconnect`, `.interval(30.seconds)` |
| Pagination | `paginate: .page` gives `.items`, `.loadMore()` and `.hasMore` |
| Progress | A `File` parameter makes `Backend.avatar.progress` a number from 0 to 1 |

Beside each endpoint you also get `Backend.users.invalidate()`,
`.prefetch(args)`, `.url(args)` and `.key(args)`. `invalidate()` drops what
the cache holds, and every resource over the endpoint on the page asks again,
so a list redraws after the write that changed it.

### Read the service from an OpenAPI file

To take every endpoint from a specification instead of writing them,
name the file with `from`:

```wf
api Backend from "openapi.json" (base: env.PUBLIC_API)
```

The build reads every endpoint, parameter, response type and error shape
from the file, and each named schema becomes something the program can
name. The day the server changes its contract, the build says so. A missing
`openapi.json` is `E111`.

How each schema becomes a type:

| The schema | What the program gets |
|---|---|
| An object: `{ "type": "object", "properties": … }` | A `type` with those fields; a field the schema does not `require` is `T?` |
| An `allOf`: `[{ "$ref": "…/EnvSummary" }, { "properties": … }]` | `type ServerDetail = EnvSummary { … }`: it extends the first record it names, and every other part adds fields of its own |
| A string with cases: `{ "type": "string", "enum": ["prod", "test"] }` | `enum Env { prod, test }`, whose cases are the strings the server sends |
| A string with a case no name can spell: `"operate.flags"` | `String`, wherever the schema is named |
| Anything else: a string, a number, a list, another schema's `$ref`, a `oneOf` of one type and `null` | That type, wherever the schema is named; a `oneOf` of several types is `Any` |

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

Also:

- A parameter or a request body may be a `$ref` into `components`, as a
  shared `limit` often is.
- A body is required when the document says so.
- A `type` or an `enum` the program declares itself wins over the
  specification's of the same name.

### Service settings

These go in the service's parentheses or at the top of its block, before
the endpoints:

| Setting | Values | Default |
|---|---|---|
| `base:` | The address every path is relative to: `"/api/v1"`, `env.PUBLIC_API` | — |
| `timeout:` | A duration after which a call gives up with `.timeout`: `10.seconds` | none |
| `retry:` | A count (`3`), `.never`, or `.backoff(times: 3, on: [.network, .timeout, .status5xx], delay: 300, max: 30000, jitter: true)` | no retries |
| `cache:` | `.swr(60.seconds)` (serve what is held, revalidate behind it), `.cache(5.minutes)`, `.forever`, `.none` | `.none` |
| `credentials:` | `.omit`, `.sameOrigin`, `.include` — whether cookies go with a call | the browser's (`.sameOrigin`) |
| `mode:` | `.cors`, `.noCors`, `.sameOrigin` | `.cors` |
| `redirect:` | `.follow`, `.error`, `.manual` | `.follow` |
| `referrer:` | A referrer policy: `.noReferrer`, `.origin`, … | the page's |
| `headers { }` | Headers every call carries, each read at the moment of the call | — |

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
  the server, and a second one could be a second order, so it fails instead.

### Declare endpoints

Each line of a service, after its settings, is an endpoint:

```text
get users(page: Number = 1, q: String?) -> [User]!
get user(id: String) at "users/:id" -> User ! LookupError
post createUser(body: NewUser) -> User ! CreateUserError
put rename(id: String, body: Rename) at "users/:id" -> User!
patch settings(body: Settings) -> Settings!
delete removeUser(id: String) at "users/:id" -> !
get report(id: String) at "reports/:id" -> Json! as: .blob
```

- **The verb** is `get`, `post`, `put`, `patch`, `delete`, `head` or
  `options`.
- **Path parameters.** A parameter the path names (`:id`) goes in the path.
  One that is empty (`null` or `""`) is not sent at all, since `users/`
  would be the whole collection, not the user; nor is one that is `.` or
  `..`, which the browser reads as this folder or the one above. The call
  ends as `.aborted`, and a resource over it stays `loading` until the value
  arrives.
- **The body.** A parameter called `body` is the request body; a map is
  sent as JSON. A `File` parameter makes the call a multipart upload.
- **Query parameters.** Every other parameter goes in the query string, and
  one that is `null` is left out.
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
that may not be null must be given. Called in an action, it is a promise of
a result: `await` it, then `match` it or pass its failure up with `try`
([Say how a call fails](#say-how-a-call-fails)). It may add `cache:`,
`headers:`, `timeout:` and `retry:`, and in a `resource` also `on:` (`.focus`, `.reconnect`,
`.interval(30.seconds)`) and `paginate:`.

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
  `.offline`, `.timeout(after)`, `.cancelled`, `.transport(message)`,
  `.decode(at, message)` and `.status(code, problem)`. Every one has
  `.message`, for showing.
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

To show a long list in pages, call the endpoint with `paginate: .page`.
This page shows items and a **Load more** button while there are more:

```wf
type Item { id: String, name: String }

api Catalog(base: "/api") {
    get items(page: Number = 1) -> [Item]!
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

- `.items` gathers every page loaded so far. It is a list of what a page
  holds — the answer, when the endpoint returns a list, or its `items`
  field — so `item.name` is checked against `Item`.
- `.loadMore()` moves `page` on by one and fetches the next page, as a
  refresh.
- `.hasMore` turns false when a page comes back empty.

### Call an API on another origin

A page on `https://example.com` that calls `https://api.example.com` makes a
*cross-origin request*. The API must answer with CORS headers —
`Access-Control-Allow-Origin`, and `Access-Control-Allow-Credentials` if
cookies go with the call — or the browser refuses the response.

The same applies while you develop. `wf serve` answers on `localhost:3000`
and has no proxy, so an API on `localhost:8080` must allow that origin.

The alternative is to serve the API and the site from one origin behind a
reverse proxy, where `base: "/api"` needs no CORS at all
([Deploying](https://webfluent.monzeromer.dev/recipes/api-on-the-same-origin.md)).

If the page's build ships a content security policy, list the API's origin
in `meta.connect` too ([Security](https://webfluent.monzeromer.dev/about/security.md)).

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

Annotate it (`data posts: [Post]`) and the checker knows every field.

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
| `resource` | The `loading` arm; the live page fetches and replaces it |
| `http` in an action | Nothing until the action runs |

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| The browser console says a response was blocked by CORS | The API is on another origin and does not allow yours. Add CORS headers to the API, or serve both from one origin ([Call an API on another origin](#call-an-api-on-another-origin)). |
| A request gets a `404` under `wf serve`, with a line in the terminal | `wf serve` has no file or page for that address, and no proxy. Point `base:` at the real API. |
| A `.parse` error naming a place, such as `$[1].name is missing` | Under `wf serve`, a response is checked against its declared type. The server sent another shape: fix the server or the `type`. |
| A resource stays `loading` | A path parameter is `null` or `""`, so the call is not sent. It goes out when the value arrives. |
| The page goes back to `loading` when state changes | The address now names something else, so it is a first load. A refresh of the same address keeps the data. |
| `T05` on a field of the data | The declared type has no such field. Fix the name, or add the field to the `type`. |
| `T06` or `T10` on an endpoint call | The service has no such endpoint, or the endpoint takes no such argument. |
| A post's page is missing from a static build | Its value is not in `paths:`. |

## Quiz

```quiz
A resource's `match` has a `loading` arm and a `ready` arm, and no other. What does the reader see when the request fails?
- An error message the runtime draws for you -- The runtime draws only the arms you write; the build warns about the missing one with T21.
+ Nothing where the `match` is, and the build warns with T21 -- No arm matches `"error"`, so the place stays empty. Add `error(e) { … }` or an `else`.
- The `loading` arm, until a retry succeeds -- A first load that fails is `"error"`, not `"loading"`.
```

```quiz
A resource is making its very first request. Which is true?
+ `pending` is true and `refreshing` is false -- `pending` covers every request of it, the first load included; `refreshing` only a request made again while data is shown.
- `refreshing` is true and `pending` is false -- It is the other way round: there is no data yet to refresh.
- Both are true -- `refreshing` is true only while data is on screen, and on a first load there is none.
```

```quiz
A resource with `on: .interval(30.seconds)` asks again while its list is on screen. What does its `match` show while that refresh runs?
- The `loading` arm, then the new list -- Only a first load, or a request whose address changed, shows `loading`.
- An empty list until the answer arrives -- The data stays while a refresh runs; `name.data` is never cleared by one.
+ The `ready` arm with the old list, then the answer drawn in place -- A refresh keeps `"ready"` and the data; rows whose key is still there keep their nodes.
```

```quiz
The address is `http.get("/api/servers?env={stage}")`, and a button sets `stage` from `"test"` to `"prod"`. What shows next?
+ The `loading` arm, then the production servers -- The request now names something else, so it is a first load; the test servers would be the wrong data.
- The test servers, with `refreshing` true, until the production servers arrive -- That is a refresh, which asks again for the same request; this one changed.
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
        loading { Spinner }
        ready(list) { Text("{list.length} users") }
    }
}
```

Add `error(e) { Alert(e.message).danger }`, or an `else`
([T21](https://webfluent.monzeromer.dev/errors/T21.md)).

**A field the declared type does not have.** Once a resource is typed, every
field read from it is checked against the `type`:

```wf expect=T05
type User { id: String, name: String }

page P(path: "/", title: "Users", description: "Everyone.") {
    resource users: [User] = http.get("/api/users")
    Heading("Users").h1
    match users {
        loading { Spinner }
        error(e) { Alert(e.message).danger }
        ready(list) { for u in list by u.id { Text(u.fullName) } }
    }
}
```

Use the field the type declares, `u.name`, or add `fullName` to the `type`
if the server sends it ([T05](https://webfluent.monzeromer.dev/errors/T05.md)).

**Reading `.data` as if it had arrived.** Before the first answer,
`name.data` is `null`:

```wf expect=T04
type User { id: String, name: String }

page P(path: "/", title: "Users", description: "Everyone.") {
    resource users: [User] = http.get("/api/users")
    Heading("Users").h1
    Text("{users.data.length} users")
}
```

Read it through `?.` with a fallback, `{users.data?.length ?? 0}`, or show
it inside the `ready` arm, where the binding is never null
([T04](https://webfluent.monzeromer.dev/errors/T04.md)).

Going further: [Open a socket](https://webfluent.monzeromer.dev/recipes/open-a-socket.md) keeps a
connection open for data the server pushes.

## Next

[Testing](https://webfluent.monzeromer.dev/learn/16-testing.md).
