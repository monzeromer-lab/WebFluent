# Fetching data

> Show a request with resource and match, ask with http in an action, describe a service once with api, handle each failure, and read data files.

The page: https://webfluent.monzeromer.dev/learn/15-data

This chapter is about getting data onto a WebFluent page and sending data
back. It starts with a single request and grows to a whole service.

You will be able to:

- show a request's `.loading`, `.failed(e)` and `.loaded(value)` cases with
  `resource` and `match`;
- refresh data without the page going blank;
- ask from an action with `http`, and handle what comes back;
- describe a service once with `api`, and say how each call fails;
- make a write refresh what it changed, page through long answers, and
  test a page without a server;
- read a JSON file at build time and render one page per item in it.

Every option this chapter mentions, and the ones it leaves out, is in
[Services and requests](https://webfluent.monzeromer.dev/reference/requests.md).

## Before you start

You should know `state` and `action`
([State and reactivity](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md)), `match`
([Control flow](https://webfluent.monzeromer.dev/learn/09-control-flow.md)) and how to declare a `type`
([Types](https://webfluent.monzeromer.dev/learn/12-types.md)).

This chapter does not cover:

- a connection the page holds open — a socket, a stream, a channel or a
  peer — which is [Open a socket](https://webfluent.monzeromer.dev/recipes/open-a-socket.md);
- values fixed for each build, which are
  [Environments and secrets](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md).

## Choose how to get data

| You want… | Use | Section |
|---|---|---|
| Content known before the site is built | `data` | [Read a file at build time](#read-a-file-at-build-time-with-data) |
| Data the page shows as soon as it opens | `resource` over `http.get(…)` | [Show a request](#show-a-request-with-resource) |
| A request made when something happens: a save, a delete | `http.post(…)` in an action | [Ask on demand](#ask-on-demand-with-http) |
| One place that describes a service, used from many pages | `api` | [Describe a service](#describe-a-service-with-api) |

Start with `http`. Move to `api` when the same service is called from more
than one place: the address, the types and the failures are then written
once.

## Show a request with `resource`

A *resource* is a request the page makes when it shows, and one value that
says where the request is: `.loading`, `.failed(e)` or `.loaded(value)`.
Show it with `match`.

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
            for d in rows by d.id { Text("{d.id}: {d.status}, {ago(d.startedAt)}") }
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
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

How it works:

- `resource name: T = http.get(url)` starts the request when the page
  renders, and cancels it when the page leaves.
- The annotation, `: [Deploy]`, says what the answer is. A read of
  `http.get` must have one (`N11`). The answer is checked against it in
  every build: a server that sends another shape fails as
  `.decode(at, message)`, `$[1].status is missing`.
- The three cases:

  | Case | When |
  |---|---|
  | `.loading` | nothing to show yet |
  | `.failed(e)` | the first load failed; `e` is an `HttpError` |
  | `.loaded(value, refreshing, stale)` | something to show; `refreshing` while it is asked again; `stale` the failure of a refresh, else `null` |

  An arm names as many of its case's places as it uses: `.loaded(rows)`.
  A `match` with no `.failed` arm and no `else` is `T21`; one that leaves
  out another case is `T15`.
- `e.message` is something you can show. For a failed status it is the
  server's own words when its body has them — its `detail`, else its
  `title` — and `HTTP 500` when it says nothing.
- A static build's pre-rendered page shows the `.loading` arm.

### Read one fact of a resource anywhere

Without a `match`, a resource has members worked out from its one value.
Each is live wherever you read it:

| Member | What it is |
|---|---|
| `name.value` | The value while it is `.loaded`, else `null` |
| `name.isLoading` · `name.isLoaded` · `name.isFailed` | Which case it is in |
| `name.isRefreshing` | `true` while a refresh runs |
| `name.error` | The first load's failure, or the last refresh's, else `null` |
| `name.pending` | `true` while any request of it runs, the first load included |
| `name.reload()` | Asks again |

So `Button(if plan.isLoaded { "Request approval" } else { "Deploy" })`
changes its label when the plan arrives, and `if let .failed(e) = plan {
Alert(e.message).danger }` shows one case without a `match`.

### Fetch again when state changes

An address that reads state is asked again whenever that state changes. A
detail page follows its route parameter this way:

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

A value spliced into the address is URL-encoded where it stands for a
value. The same holds for a body or a header that reads state. To ask less
often than the reader types, splice a slower value, as
[Debounce a search](https://webfluent.monzeromer.dev/recipes/debounce-a-search.md) does.

## Refresh without clearing the page

Asking again for the request already shown is a *refresh*: `name.reload()`
asks, and so does `refresh:` on the request — `.onReconnect` (what a read
does unless it says otherwise), `.onFocus`, `.every(30.seconds)` or
`.never`. While a refresh runs, the read stays `.loaded` with its value,
and `refreshing` is true. This page polls every 30 seconds:

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

The `.loaded` arm is drawn again only when the case changes; inside it,
the new value is drawn in place. A row whose key is still in the list keeps
its nodes, its focus and what was typed into it, and an answer that changed
nothing redraws nothing.

When a request fails or changes:

- A refresh that fails stays `.loaded` with the old value, and its failure
  is `stale` until the next answer.
- A first load that fails is `.failed(e)`.
- A request whose address now names something else is a first load again,
  shown as `.loading`, because the old value would be the wrong one.
  Pressing **Production** above does this. `placeholder: .previous` keeps
  the last value shown instead, refreshing, until the new answer arrives —
  for a search whose results stay while the next ones load.

## Ask on demand with `http`

For a request made when something happens — a save, a delete, a step in a
flow — call `http` in an action: `http.get(url)`, `http.post(url, body: …)`,
`http.put`, `http.patch`, `http.delete`. Its answer is a *result*:
`.ok(value)`, or `.err(e)` with an `HttpError`. A failure is a value you
handle, not an exception you catch.

This page loads todos and adds one, each with a request:

```wf
type Todo { id: String, title: String, done: Bool = false }
type NewTodo { title: String }

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

    load()
    Heading("Todos").h1
    Input(bind: draft, label: "New todo") { on key("Enter") { add() } }
    Button("Add", disabled: add.pending).primary { on click { add() } }
    if error != "" { Alert(error).danger }
    for todo in todos by todo.id { Text(todo.title) }
}
```

- **The type is named where the answer goes.** `todos = try await
  http.get(…)` is a `[Todo]` because `todos` is; `let saved: Todo!` names a
  whole result. A `get` whose answer goes nowhere that says what it is is
  `N11`.
- **`match`** gives each case an arm, as `add` does. **`try`** gives the
  value, or ends the action with the failure as its own, so `load` says
  what it fails with: `-> !` is nothing, or an `HttpError`.
- **Options** follow the address by name: `body:` (sent as JSON),
  `headers:`, `timeout:`, `retry:`, `cache:`, `as: .text`.
- `load()` at the top of the page runs once, when the page renders, and
  `add.pending` is `true` while `add` runs.

## Describe a service with `api`

When a service is called from more than one place, describe it once with
`api`: where it is, how it is reached, and its endpoints. Every call is then
typed, and the day the server changes its contract the build says so.

```wf
type User { id: String, name: String }

api Backend(base: "/api/v1") {
    timeout: 10.seconds
    retry: .backoff(times: 3, on: [.network, .timeout, .status5xx])

    get    users(page: Number = 1, q: String?) at "users" -> [User]!
    get    user(id: String) at "users/{id}" -> User!
    delete removeUser(id: String) at "users/{id}" -> !
}

page Users(path: "/users", title: "Users", description: "Everyone.") {
    state page: Number = 1
    resource rows = Backend.users(page: page, refresh: .onFocus)

    Heading("Users").h1
    match rows {
        .loading { Skeleton(height: "20px") }
        .failed(e) {
            match e {
                .offline(_) { Alert("Could not reach the server.").warning }
                .timeout(_) { Alert("The server took too long.").warning }
                else { Alert(e.message).danger }
            }
            Button("Try again") { on click { rows.reload() } }
        }
        .loaded(list) { for u in list by u.id { Text(u.name) } }
    }
    Button("Next page") { on click { page = page + 1 } }
}
```

A `resource` over an endpoint takes the endpoint's type, so `u.nam` is
`T05`. An endpoint the service does not have is `T06`. Every call goes through the engine `http` uses. It cancels a call when its
page leaves, gives up after the `timeout:` (30 seconds unset), retries a
request that is safe to send twice, keeps a `GET` answer as long as the
server's `Cache-Control` allows, and makes one request for two readers of
one address. The settings and their rules are in
[Services and requests](https://webfluent.monzeromer.dev/reference/requests.md).

### Declare endpoints

Each line after the settings is an endpoint:

- **The verb** is the method: `get`, `post`, `put`, `patch`, `delete`.
- **`at`** is where the server answers, below `base`. Every `{name}` in it
  is one of the endpoint's parameters (`N01`).
- **Parameters** the path names go in the path. One called `body` is the
  request body. The rest go in the query string, or where `in header "…"`
  or `in cookie` says.
- **A call** passes them by name, the first without its name if you like:
  `Backend.user("7")`.
- **`->`** says what a success returns and how a call fails: `-> T!` is a
  `T` or an `HttpError`, `-> T ! E` a `T` or a case of the error enum `E`,
  and `-> !` nothing, but it may fail.
- A path parameter that is `null`, `""`, `.` or `..` is not sent, since
  `users/` would be the whole collection. The call ends `.cancelled`, and a
  read over it stays `.loading` until the value arrives.

A service's endpoints may be split into groups — `api Backend.todos { … }`,
called as `Backend.todos.list()`. A group sends what its service sends: its
`base`, `timeout`, `retry` and `middleware`.

## Say how a call fails

An endpoint that names an *error enum* turns each status the service
answers with into a case of its own, carrying that status's body. One case,
`http(error: HttpError)`, holds everything else:

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
}

store People {
    state said = ""

    /// Every case handled where the call is made.
    action submit(name: String) {
        match await Backend.createUser(body: NewUser(name: name)) {
            .ok(u) { said = "Added {u.name}" }
            .err(.unprocessableContent(p)) { said = p.errors.map(e => e.message).join(", ") }
            .err(.conflict(_)) { said = "Already there" }
            .err(.http(.status(503, _))) { said = "The service is busy" }
            .err(.http(e)) { said = e.message }
        }
    }

    /// The failure passed up, as the action's own.
    action invite(name: String) -> User ! CreateUserError {
        let user = try await Backend.createUser(body: NewUser(name: name))
        return user
    }
}
```

- **A case** names its status, `on 422` (or a class, `on 4XX`), and
  carries one value, the body. The `http` case is required, once (`N12`).
- **A pattern** may look inside a case, `.err(.http(.status(503, _)))`.
- **`try`** passes a failure up as the action's own, which must be able to
  hold it (`N03`): an `HttpError` goes into an error enum's `http` case.
- **`.result`** is an action's last outcome, `null` before it has run:
  `if let .err(.conflict(p)) = People.invite.result { … }` shows it.
- **A `resource`** over such an endpoint has the enum in its `.failed(e)`.

`HttpError` is what goes wrong between the page and the server:

| Case | When |
|---|---|
| `.offline(queued)` | the request did not reach a server: no network, a server down, a CORS refusal; `queued` when a write was kept to send later |
| `.timeout(limit)` | no answer within the timeout |
| `.cancelled` | the page left, the request changed, or `.cancel()` |
| `.transport(message)` | anything else that stopped it, such as a middleware that threw |
| `.decode(at, message)` | the answer is not the declared type |
| `.status(code, problem)` | the server answered with an error status; `problem` is its RFC 9457 body, if it sent one |

Each has `.message`, for showing. `Problem` is RFC 9457's record — `type`,
`title`, `status`, `detail`, `instance`, `extensions` — and a project
extends it like any record, as `ValidationProblem` does.

## Change data

A write is a call made from an action or a handler, never a `resource`: a
resource over a write is `N02`. A write's result must be heard — matched,
or passed up with `try`. One awaited on its own is `N04`.

After a write, the lists that show what it changed are out of date.
`invalidates` names them, on the write. When the write succeeds, every
resource on the page over one of them is refreshed, drawn in place:

```wf
type Todo { id: String, title: String }
type NewTodo { title: String }

api Backend(base: "/api") { }

api Backend.todos {
    get   list() at "todos" -> [Todo]!
    get   byId(id: String) at "todos/{id}" -> Todo!
    post  create(body: NewTodo) at "todos" -> Todo!
        invalidates [list]
    delete remove(id: String) at "todos/{id}" -> !
        invalidates [list, byId(id)]
}

store Drafts {
    state said = ""
    action add(title: String) {
        let undo = Backend.todos.list.update(list => list.concat([Todo(id: "new", title: title)]))
        if let .err(e) = await Backend.todos.create(body: NewTodo(title: title)) {
            undo()
            said = e.message
        }
    }
}

page Todos(path: "/todos", title: "Todos", description: "The list.") {
    use Drafts
    resource todos = Backend.todos.list()
    Heading("Todos").h1
    Button("Add bread") { on click { Drafts.add("Bread") } }
    Text(Drafts.said)
    match todos {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for todo in list by todo.id { Text(todo.title) } }
    }
}
```

- Each name in `invalidates` is a read of the same group, and each argument
  is one of the write's own parameters (`N05`).
- `Backend.todos.list.update(change)` changes what every read of `list`
  shows, at once, and returns what takes it back. `Drafts.add` shows the
  new todo before the server answers, and takes it back on a refusal.

### Show a change before the server agrees

`optimistic(holder, change)` does the same for a state you hold yourself.
If anything later in the action fails — a `try` passing a failure up, or a
throw — the change is taken back:

```wf
type Todo { id: String, title: String }
type Rename { title: String }

api Backend(base: "/api") {
    patch rename(id: String, body: Rename) at "todos/{id}" -> Todo!
}

store Todos {
    state items: [Todo] = []

    action rename(id: String, title: String) -> ! {
        optimistic(items, list => list.map(x => if x.id == id { Todo(id: x.id, title: title) } else { x }))
        try await Backend.rename(id: id, body: Rename(title: title))
    }
}
```

The new title shows at once. If `rename` fails, `try` passes the failure
up, and the old title comes back.

## Long answers

### Load results a page at a time

How an endpoint pages is part of its contract, so you say it once, on the
endpoint, with `pages`. A read of it then holds every page loaded so far:

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

- `.items` is every item loaded so far, in order.
- `.loadMore()` asks for the next page as a refresh: what is shown stays.
- `.hasMore` is whether there is a next page.
- `param:` names the parameter the next page goes in; `items:` says where
  a page's items are, when the page is not a list itself.

There are three ways to page:

| Written | Asks for the next page with | Stops when |
|---|---|---|
| `pages .number(param: page)` | the parameter, one more each time | a page comes back empty |
| `pages .cursor(param: after, next: …)` | the cursor the last page gave | `next:` gives `null` |
| `pages .link` | the `Link` header's `rel="next"` (RFC 8288) | no `Link` names one |

### Read an answer that arrives a line at a time

An endpoint that answers one JSON value a line — a log tail, a build's
output — is marked `streamed`, and declares the list its lines make:

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
```

A `resource` over it is `.loaded` from the first line, and its list grows
as the rest arrive. `for await line in …` runs its body for each line as it
arrives. Each line is held to `LogLine`. `pages` and `streamed` are a
read's: a write is sent once (`N14`).

## Credentials and every request

A session belongs in an httpOnly cookie, which no script on the page can
read. `credentials: .sameOrigin` (or `.include`, for a service on another
origin) sends it with every call, and the page holds no token at all.

### Pass every request through middleware

What every call carries — a token, the reader's language, a request ID —
is *middleware*: steps each request passes through, in the order written in
`middleware [ … ]`. This service refreshes the token on a `401`, sends it
as a bearer token, and tags each request:

```wf
type User { id: String, name: String }
type Token { token: String }

store Session {
    state accessToken = ""
    action refresh() -> ! {
        let fresh: Token = try await http.post("/api/token")
        accessToken = fresh.token
    }
}

api Backend(base: "/api") {
    middleware [refreshOn401(Session.refresh), bearer(Session.accessToken), requestId]

    get me() at "me" -> User!
}
```

`refreshOn401` goes before `bearer`, so the request sent again carries the
new token. Requests that hear `401` at the same time share one run of the
action. The other built-in steps are `header(name, value)` and `log`.

A step of your own is a `middleware timed(req, next) { … }` declaration
that returns what `await next(req)` returns.

### Call an API on another origin

A page on `https://example.com` that calls `https://api.example.com` makes
a *cross-origin request*. The API must answer with CORS headers —
`Access-Control-Allow-Origin`, and `Access-Control-Allow-Credentials` if
cookies go with the call — or the browser refuses the answer. The page is
not told why, so the call fails as `.offline`, and the console names CORS.

`wf serve` has no proxy, so an API on `localhost:8080` must allow
`localhost:3000` too. One origin for both, behind a reverse proxy, needs no
CORS ([Serve the API on the same origin](https://webfluent.monzeromer.dev/recipes/api-on-the-same-origin.md)).
A build that ships a content security policy lists the API's origin in
`meta.connect` ([Security](https://webfluent.monzeromer.dev/about/security.md)).

## Read the service from an OpenAPI file

When the server publishes an OpenAPI spec (3.0 or 3.1, JSON or YAML), let
it write the contract:

```bash
wf api add openapi.json --name Backend
wf api sync           # after the spec changes
wf api check          # in CI: fails when the files are behind the spec
```

`add` records the service in `webfluent.app.json`, writes
`src/api/backend/service.wf` once for you to edit, and writes the contract
into `src/api/backend/generated/`, which `sync` rewrites whole. Each tag is
a group, each operation an endpoint named by its
`operationId`, each schema a `type` or an `enum`, and each operation's
error statuses the cases of an error enum. `wf check` warns `N07` when the
files are behind the spec. Customise beside them, never inside: a `type`
you declare wins over the spec's of the same name, an endpoint the spec
lacks goes in a group of your own, and what a write makes stale and how a
read pages go in `service.wf`:

```wf
type Person { id: String, name: String }
type PersonPage { items: [Person], nextCursor: String? }

api Backend(base: "/api") {
    invalidations {
        people.deletePerson(id) invalidates [people.listPeople, people.getPerson(id)]
    }
    pagination {
        people.listPeople pages .cursor(param: after, next: p => p.nextCursor, items: p => p.items)
    }
}

// generated/people.wf, written by `wf api sync`
api Backend.people {
    get listPeople(after: String?) at "people" -> PersonPage!
    get getPerson(id: String) at "people/{id}" -> Person!
    delete deletePerson(id: String) at "people/{id}" -> !
}
```

For a prototype, `api Backend from "openapi.json" (base: "/api")` reads the
same contract without writing files, and `wf api eject Backend` turns it
into files later. [Call a service from its OpenAPI spec](https://webfluent.monzeromer.dev/recipes/service-from-openapi.md)
walks through one.

## Test it without a server

A page that reads a service has nothing to show in a test until something
answers. A `fake` answers a group's endpoints in place of the network:

```wf
type Person { id: String, name: String }

api Backend(base: "/api") { }

api Backend.people {
    get list() at "people" -> [Person]!
}

page People(path: "/", title: "People", description: "The team.") {
    resource people = Backend.people.list()
    Heading("People").h1
    match people {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for p in list by p.id { Text(p.name) } }
    }
}

fake TwoPeople for Backend.people {
    list() { .ok([Person(id: "1", name: "Ada"), Person(id: "2", name: "Grace")]) }
}

test "lists the team"(fakes: [TwoPeople]) {
    People
    expect "Grace"
}
```

- A fake is held to the group's contract (`N06`); one that answers
  `.err(.status(503, null))` tests a failure.
- A test that names `fakes:` runs in a browser and sends nothing. A call no
  fake answers fails the test.
- `fake Examples for Backend.people from examples` answers every endpoint
  it does not answer itself with the spec's example of a success.
- `wf verify --fakes tests/fakes.wf` loads every page of the site with the
  fakes answering. A build leaves fakes out.

[Testing](https://webfluent.monzeromer.dev/learn/16-testing.md) covers tests;
[Test a page with a fake service](https://webfluent.monzeromer.dev/recipes/test-with-a-fake.md) tests a
write.

## Keep writes made offline

With `offline` in the config, the build writes a service worker, and
`"sync": true` keeps a write that failed for want of a network, to send when
the connection returns:

```json
{ "offline": { "precache": ["/"], "sync": true } }
```

- Only a write safe to send twice is kept: a `PUT`, a `DELETE`, or a
  `POST` or `PATCH` whose endpoint is marked `idempotent`. Another `POST`
  may have reached the server, and sent again would be a second order, so
  it fails `.offline(queued: false)`.
- A kept write's call fails `.offline(queued: true)`: no answer came, and
  the write waits, so the page should not show it as done.
- `network.queued` counts what waits. Once a kept write is sent, what its
  endpoint `invalidates` is asked again.

See [Keep writes made offline](https://webfluent.monzeromer.dev/recipes/keep-writes-made-offline.md) and
[Make a site work offline](https://webfluent.monzeromer.dev/recipes/work-offline.md).

## Read a file at build time with `data`

For content known before the site is built — posts, a price list, a team
page — `data name: T = "file.json"` reads a JSON file from the project
directory or `src/` at build time. Its value is a constant, in scope in
every page and component, and written into the pre-rendered page:

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

The build holds the file to the type. A record that is not a `Post` stops
the build with `T33`, naming the file and the place:
`$[3].date is "March 5", not a Date`.

### Render a page for each item

To give each post its own HTML file in a static build, list the values of
the route parameter in `paths:`:

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page PostPage(path: "/posts/:slug", slug: String, title: "Post", description: "A post.", paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    if let p = post {
        Heading(p.title).h1
        Markdown(p.body)
    } else {
        Alert("No such post").warning
    }
}
```

The build writes `posts/hello/index.html` and `posts/second/index.html`,
and lists each in the sitemap. `paths:` must be known at build time: a
`data` file, a `const` or a literal. For a route with several parameters,
each value is a map naming them. A single-page app answers every slug; only
the static build needs the list.

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| The console says a response was blocked by CORS; the call fails `.offline` | The API is on another origin and does not allow yours ([Call an API on another origin](#call-an-api-on-another-origin)). |
| `.decode(at, message)`, such as `$[1].name is missing` | The server sent another shape than the declared type. Fix the server or the `type`. |
| A resource stays `.loading` | A path parameter is `null` or `""`, so the call is not sent. It goes out when the value arrives. |
| The page goes back to `.loading` when state changes | The address now names something else, so it is a first load. `placeholder: .previous` keeps the last value. |
| A list does not show what a write added | The write does not name the read in `invalidates`, or it failed. |
| A post's page is missing from a static build | Its value is not in `paths:`. |

## Quiz

```quiz
A resource with `refresh: .every(30.seconds)` asks again while its list is on screen. What does its `match` show while that refresh runs?
- The `.loading` arm, then the new list -- Only a first load, or a request whose address changed, shows `.loading`.
- An empty list until the answer arrives -- The value stays while a refresh runs.
+ The `.loaded` arm with the old list, then the answer drawn in place -- A refresh stays `.loaded`, with `refreshing` true; rows whose key is still there keep their nodes.
```

```quiz
`post order(body: Order) at "orders" -> Order!` times out. What does the engine do, with `retry: .backoff(times: 3)` on the service?
- It sends the order again, up to three times -- A `POST` that timed out may have reached the server; sent again it could be a second order.
+ It fails with `.timeout`, and sends nothing again -- Only a request safe to send twice is retried. Mark the endpoint `idempotent` and it is retried with one `Idempotency-Key`.
- It keeps the order to send when the network returns -- A timeout is not the network going away, and a `POST` that is not `idempotent` is never kept.
```

## Common mistakes

**A `match` with no `.failed` arm.** When the request fails, no arm draws,
and the reader is not told:

```wf expect=T21
type User { id: String, name: String }

page P(path: "/", title: "Users", description: "Everyone.") {
    resource users: [User] = http.get("/api/users")
    Heading("Users").h1
    match users {
        .loading { Spinner }
        .loaded(list) { Text("{list.length} users") }
    }
}
```

Add `.failed(e) { Alert(e.message).danger }`, or an `else`
([T21](https://webfluent.monzeromer.dev/errors/T21.md)).

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
([T04](https://webfluent.monzeromer.dev/errors/T04.md)).

## Next

[Testing](https://webfluent.monzeromer.dev/learn/16-testing.md).
