### API services

An `api` declaration describes an HTTP service once — its base URL, endpoints, types and errors — so every call is typed.

`api` is where a service lives: its address, how it is reached, and what it
has. Every call site is then typed, cached and cancellable, and the day the
server changes its contract the build says so.

```wf
type User { id: String, name: String }

api Backend(base: env.PUBLIC_API) {     // its default in the config's `env`
    timeout: 10.seconds                // 30 s when it says nothing
    retry: .backoff(times: 3, on: [.network, .timeout, .status5xx])
    credentials: .sameOrigin           // .omit · .sameOrigin · .include — the
                                       // session cookie, which is where a
                                       // session belongs; `persist` is not
    middleware [requestId, log]        // and refreshOn401(Session.refresh), bearer(Session.token), …

    get    users(page: Number = 1, q: String?) at "users"   -> [User]!
    get    user(id: String) at "users/{id}"       -> User!
    post   createUser(body: NewUser) at "createUser"             -> User!
    put    order(body: Order) at "orders" -> Order! idempotent
    delete removeUser(id: String) at "users/{id}" -> !
    post   avatar(id: String, file: File) at "users/{id}/avatar" -> Url!
}
```

**Where an endpoint is.** `at` is required: the path the server answers it
at, below `base`. A parameter the path names goes there — `at
"users/{id}"`, `{name}` as OpenAPI writes it — and every `{name}` is one of
the endpoint's parameters (`N01`). Another goes in the query string — of a
write too: what a write sends is its `body` parameter — or where it says:
`get me(token: String in header
"X-Token", theme: String? in cookie) at "me" -> User!`. A call may pass the
first parameter without its name: `Backend.user("7")`.

**Groups.** A service's endpoints may be split by what they are about. A
group is reached through its service and sends what the service sends —
its `base`, `timeout`, `retry` and `middleware`, none of its own (`N08`):

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

store Todos {
    action add(title: String) {
        let undo = Backend.todos.list.update(list => list.concat([Todo(id: "new", title: title)]))
        if let .err(_) = await Backend.todos.create(body: NewTodo(title: title)) { undo() }
    }
}
```

**What a write makes stale.** `invalidates [list, byId(id)]` names the reads
of the same group that a write changes, each with the write's own
parameters as its arguments (`N05`): once the write succeeds, every
`resource` on the page over one of them asks again — a refresh, drawn in
place. A read's `.update(change)` shows a change before the server agrees,
in every resource over it, and returns what takes it back — `Todos.add`
above undoes its change when the server refuses.

A `resource` is over a read (`N02` for a write); a write's result that
nothing reads is `N04`, a failure no one hears.

**Middleware.** What every request passes through, in the order written:
each step is handed the request and the rest of the chain, `next`, and
returns what `next` returns — a result, `.ok(json)` or `.err(httpError)`.
The runtime's: `bearer(token)` (`Authorization: Bearer …`, when there is
one), `header(name, value)`, `refreshOn401(action)` (on a `401`, run the
action — once for every request that heard it at the same time, since a
running action is joined — and send the request again through what follows
it, so it goes before `bearer`), `requestId` (`X-Request-Id`) and `log`. A
project writes its own:

```wf
/// Every request, timed.
middleware timed(req, next) {           // req: Request · next: (Request) -> answer
    let started = Date.now()
    let answer = await next(req)
    log("{req.method} {req.url} took {Date.now() - started} ms")
    return answer
}
```

`Request` is `{ url, method, headers: [String: String], body: Json? }`; a
step changes what it hands on — `next(Request(url: req.url, method:
req.method, headers: req.headers.with("X-Client", "web"), body: req.body))`
— or answers without the network. 6.x's `headers { }` and `on request`,
`on response`, `on error` hooks are `N08`.

**Retries and `idempotent`.** A retry policy applies to what is safe to
send twice — `GET`, `HEAD`, `OPTIONS`, `PUT`, `DELETE` — with backoff, jitter
and `Retry-After`. A `POST` or `PATCH` is retried, or kept to replay
offline, only when its endpoint is marked `idempotent`, which sends one
`Idempotency-Key` on every try of a call; `retry:` on any other write is
`N09`.

**Caching.** A `GET` is kept as an HTTP cache keeps it (RFC 9111, a private
cache): only when the server's `Cache-Control` or `Expires` allows it, fresh
for its `max-age` (less its `Age`), served stale and asked again behind it
within `stale-while-revalidate`, asked again with its `ETag` or
`Last-Modified` after, never under `no-store`, every time under `no-cache`,
and kept apart by `Vary` and by the credentials it was asked with. With no
caching headers nothing is kept. An endpoint or a call may override the
server: `cache: .maxAge(60.seconds)`, `.swr(60.seconds)`, `.forever`,
`.none`. Identical `GET`s in flight are always joined.

**Every answer is checked**, in every build: what an endpoint or a
`resource` declares it returns is held to what arrives, and a server that
does not keep its contract fails as `.decode(at, message)` — `$[1].name is
missing` — under `wf serve` and deployed alike.

Or let the service's OpenAPI spec (3.0 or 3.1, JSON or YAML) write the
contract. `wf api add openapi.json --name Backend` records it in the config
(`"apis": { "Backend": { "spec": "openapi.json", "out": "src/api/backend" } }`),
writes `src/api/backend/service.wf` once — yours: the base, the timeout,
middleware — and `src/api/backend/generated/`: `types.wf` and one file a tag,
each opening with a `// Code generated by …` comment naming `wf api sync`
and the spec, DO NOT EDIT. `wf api sync` rewrites that folder whole and never touches a file
in it without the marker; `wf api check` fails when it is behind the spec
(for CI), and `wf check` warns `N07`. `api Backend from "openapi.json"
(base: env.PUBLIC_API)` reads the same contract inside the compiler, for a
prototype — one reader serves both, and `wf api eject Backend` turns it into
files. The editor and `wf render` read it too.

```wf
// generated/people.wf
api Backend.people {
    get listPeople(q: String?, after: String?, pageSize: Number = 20 in query "page-size") at "people" -> PersonPage!
    post createPerson(body: NewPerson) at "people" -> Person ! CreatePersonError
}

// generated/types.wf
enum CreatePersonError {
    conflict(problem: Problem) on 409
    unprocessableContent(problem: ValidationProblem) on 422
    clientError(problem: Problem?) on 4XX      // every other 4xx
    http(error: HttpError)
}
```

An operation is in the group its first tag names (`default` with none),
named by its `operationId` (`list-users` → `listUsers`; none, a name from
the method and path, with a warning; two alike stop the read). A parameter
goes where the spec puts it, under the server's name — `in query "page-size"`,
`in header "X-Tenant"`, `in cookie "session_id"` — and takes its `default`.
A body and an answer are read by media type (JSON; text is `String` with
`as: .text`; bytes `File` with `as: .blob`; a form with one file is that
file and the rest); one the language cannot send or read (XML) stops the
read, unless `wf api sync --skip NAME` leaves it out. Every error status is
a case of the operation's error enum, named by its reason phrase; `default`
is what `.http(.status(…))` holds.

Each named schema becomes something the program can name: an object a
`type`; an `allOf` a `type ServerDetail = EnvSummary { … }` that extends the
first record it names; a string with cases (`"enum": ["prod", "test"]`) an
`enum Env { prod, test }`, or `String` when a case is not a name
(`"operate.flags"`); an object or cases written in place a record or an enum
named by its place (`CreatePersonBody`, `PersonRole`); `additionalProperties`
`[String: T]`; a shape the spec does not say (`{}`, a `oneOf` of several
types) `Json`; anything else the type it reads as. A field named as no
program can write is quoted: `"first-name": String?`. A value that may be
null — 3.1's `"type": ["string", "null"]`, a `oneOf` of `{ "type": "null" }`
and a `$ref`, 3.0's `nullable: true` beside anything — is `T?`. A string's
`format` is the language's type in what the server sends — `date-time` →
`DateTime`, `date` → `Date`, `uuid` → `Uuid`, `email` → `Email`, `uri` →
`Url` — and stays `String` in what the program sends (a parameter, a body, a
record one reaches, but for its `readOnly` fields) and for `time`.

Customise beside the generated files: a `type` or `enum` the program
declares wins over the spec's; an endpoint the spec lacks goes in a group of
your own; and what a write makes stale and how a read pages go in the
service's own block, each named `group.endpoint` and checked as if written
on it (`N05`, `N14`) — or as `x-invalidates` and `x-pagination` in a spec
you own:

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

An endpoint is called like anything else, and a `resource` over one is
typed by what it returns:

```wf
resource users = Backend.users(page: n, cache: .swr(60.seconds), refresh: .onFocus)

match users {
    .loading { Skeleton }
    .failed(e) {
        match e {
            .offline { Alert("You are offline.").warning }
            .timeout { Alert("The server took too long.").warning }
            else     { Alert(e.message).danger }
        }
        Button("Try again") { on click { users.reload() } }
    }
    .loaded(list) { for u in list by u.id { Text(u.name) } }
}
```

What comes with it, at every call site:

| | |
|---|---|
| Abort | on scope leave, on an argument that reads state changing, on `.cancel()`, on timeout |
| Errors | a call's `.err` is an `HttpError` — `.offline(queued)`, `.timeout(limit)`, `.cancelled`, `.transport(message)`, `.decode(at, message)`, `.status(code, problem)` — or, for `-> T ! E`, a case of the error enum, each declared status with its body (held to its type under `wf serve`); a read's `.failed(e)` holds the same |
| Retry | exponential backoff with jitter, per error kind, honouring `Retry-After` — for a request safe to send twice (`GET`, `HEAD`, `OPTIONS`, `PUT`, `DELETE`), and a `POST` or `PATCH` only when its endpoint is `idempotent` (one `Idempotency-Key` on every try) |
| Cache | what the server's `Cache-Control` allows (RFC 9111): `max-age`, `stale-while-revalidate`, `no-store`, `no-cache`, `Vary`, `ETag` revalidation and `304` served from what is held; nothing without caching headers; `.maxAge(d)`, `.swr(d)`, `.none`, `.forever` override it |
| Dedupe | two readers of one address make one request, cached or not; each keeps its own abort |
| Refetch | an argument that reads state changing; `refresh: .onReconnect` (unset), `.onFocus`, `.every(30.seconds)`, `.never` — a refresh: the value stays on the page while it runs, and the answer is drawn in place (see Data Fetching) |
| Pages | the endpoint's `pages .number(…)`, `.cursor(…)` or `.link`: a read holds every page's items, with `.loadMore()` and `.hasMore` (see Pages) |
| Progress | a `File` parameter makes `Backend.avatar.progress` a `Number` signal |
| Bodies | JSON, text, a `File` (sent as a form), `FormData`, `Blob`, `ArrayBuffer`, `URLSearchParams` |
| Responses | JSON, `as: .text`, `.blob`, `.arrayBuffer`, `204` as no value; a `streamed` endpoint's lines as they arrive (see Pages) |
| Elsewhere | `Backend.users.invalidate()` — every resource over it on the page asks again —, `.prefetch(args)`, `.key(args)`, `.url(args)` |
| Empty path parameters | a call whose path parameter is `null`, `""`, `.` or `..` is not sent (`users/` would be the collection, not the user); it ends `.cancelled`, and a read over it stays `.loading` until the value arrives |

A call is a promise of a result: `await` it in an action or a handler, then
`match` it or pass its failure up with `try`. A parameter with no default
that may not be null must be given (`C01`), and a resource's annotation is
held to what its endpoint returns. An endpoint that does not say how it
fails (`-> T` with no `!`), `errors { }` and `fetch(…)` are `N08`, each with
its fix; `Any` and `Map` in a contract are `N11`.

**Saying how a call fails.** An endpoint declared `-> T ! E` or `-> T!`
returns a *result* — `.ok(value)` or `.err(error)` — and never throws. An
*error enum* names each status the service answers with, carrying its
body, and keeps `http(error: HttpError)` for the rest (`N12` otherwise):

```wf
type User { id: String, name: String }
type NewUser { name: String }
type FieldError { field: String, message: String }
type ValidationProblem = Problem { errors: [FieldError] }

enum CreateUserError {
    unprocessableContent(problem: ValidationProblem) on 422
    http(error: HttpError)
}

api Backend(base: "/api") {
    post createUser(body: NewUser) at "users" -> User ! CreateUserError
}

store People {
    state said = ""
    action submit(name: String) {                    // every case, here
        match await Backend.createUser(body: NewUser(name: name)) {
            .ok(u) { said = "Added {u.name}" }
            .err(.unprocessableContent(p)) { said = p.title ?? "Check the form" }
            .err(.http(.status(503, _))) { said = "Busy" }
            .err(.http(e)) { said = e.message }
        }
    }
    action invite(name: String) -> User ! CreateUserError {   // passed up
        let user = try await Backend.createUser(body: NewUser(name: name))
        return user                                  // `.ok(user)`
    }
}
// `if let .err(.unprocessableContent(p)) = People.invite.result { … }`
```

`HttpError` is `.offline(queued)`, `.timeout(limit)`, `.cancelled`,
`.transport(message)`, `.decode(at, message)` or `.status(code, problem)`,
each with `.message`; `Problem` is RFC 9457's (`type`, `title`, `status`,
`detail`, `instance`, `extensions`). `try` gives a result's value or returns
its error from the action, which must declare one that can hold it
(`N03`): the same error as it is, an `HttpError` into an error enum's
`.http`, another error enum through its `http` case. In a handler `try`
reports the failure. An action declaring `-> T ! E` returns its value
(`.ok` of it) or `return .err(e)`; `-> ! E` and `-> !` return nothing but
may fail. Its `.result` is its last outcome, `null` before it has run.
Patterns: `_`, a literal, a case inside a case (`.err(.http(.status(503,
_)))`), in a `match` — which runs in an action or a handler too — and in
`if let .case(…) = value`.

`http` is the built-in service for a request no contract describes:
`http.get(url)`, `http.post(url, body: value)`, `http.put`, `http.patch`,
`http.delete`, each a promise of `T!`, `T` named where the answer goes —
`let rows: [Row] = try await http.get(url)`, `let got: [Row]! = await
http.get(url)`, `resource rows: [Row] = http.get(url)` — and `N11` for a
`get` with nowhere that says (a write whose answer nothing reads needs no
type; `as: .text` is a `String`). `fetch` is gone (`N08`), and `window.fetch` is
not in the browser table (`T30`): a project script reaches it.

**Pages and lines.** How an endpoint answers more than once is part of
its contract. One that pages says how, and a read of it holds every page's
items, in order — `.items` (the same list as `.value`), `.hasMore`, and
`.loadMore()` for the next page, asked as a refresh with what is shown left
in place. A refresh asks again for every page gathered, and shows them
together; another request starts from the first page. The read writes no
state of the page's.

```wf
type Post { id: String, title: String }
type FeedPage { posts: [Post], nextCursor: String? }
type LogLine { at: String, text: String }

api Social(base: "/api") {
    get feed(after: String?) at "feed" -> FeedPage!
        pages .cursor(param: after, next: p => p.nextCursor, items: p => p.posts)
    get list(page: Number = 1) at "posts" -> [Post]!
        pages .number(param: page)                // ends at the first empty page
    get search(q: String) at "search" -> [Post]!
        pages .link                               // the Link header's rel="next" (RFC 8288)
    get tail(id: String) at "logs/{id}" -> [LogLine]! streamed
}

store Logs {
    state seen: [String] = []
    action follow(id: String) {
        for await line, i in Social.tail(id: id) { seen = seen.concat(["{i}: {line.text}"]) }
    }
}

page Feed(path: "/feed", title: "Feed", description: "What is new.") {
    resource feed = Social.feed()                 // a paged read of Post
    resource tail = Social.tail(id: "deploy-7")   // .loaded from the first line
    Heading("Feed").h1
    for p in feed.items by p.id { Text(p.title) }
    if feed.hasMore { Button("More", loading: feed.pending) { on click { feed.loadMore() } } }
    Text("{tail.value?.length ?? 0} lines so far")
}
```

The parameter a page goes in is one of the endpoint's that the first call
need not give (a default, or `T?`); a page that is not a list says where
its items are (`items:`); `.number` stops at an empty page, `.cursor` when
`next:` gives null, `.link` when no `Link` names `rel="next"`. A
`streamed` endpoint answers one JSON value a line, each held to the item
type: a read over it is `.loaded` from its first line and grows; awaited
whole it is the full list; `for await line in …` in an action or a handler
walks the lines as they arrive. Each is a `get` (`N14`); 6.x's `paginate:`
on a call is `N08`.

