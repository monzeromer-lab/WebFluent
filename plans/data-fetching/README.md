# Data fetching: a redesign

Written 2026-11-15, against 6.2.0 (commit 39767cde). Nothing here is built
yet: this is the plan to approve first.

## In short

Today a WebFluent page can get data four ways — `resource x = fetch(…)`,
`resource x = Backend.ep(…)`, `await fetch(…)` in an action, `await
Backend.ep(…)` in an action — and each one behaves a little differently.
Underneath, one good engine (`http.js`) does timeouts, retries, a cache,
ETags and typed errors. On top of it, the language surface is uneven: the
same word means two things, some options follow state and some do not, the
cache is off unless you ask, invalidating a cache does not refresh what is
on the screen, errors are typed in the docs but not in the checker, and a
few calls quietly never send (section 2 lists nineteen defects).

The plan keeps what works (the engine, the `match` over loading / error /
ready, refreshes that keep the page, abort on leaving) and changes the
shape around it:

1. **Two kinds of request, named for what they do.** A *read* is declared
   (`resource`), shared, cached and refreshed for you. A *write* is called
   from an action, and says which reads it makes stale. That is the line
   TanStack Query, RTK Query, Remix and tRPC all draw.
2. **Writes refresh the reads they change, without being told.** After
   `await Backend.createUser(…)`, every `resource` on screen that reads
   `users` refreshes, in place. The rule is the REST path by default and a
   `changes:` clause when the path is not enough.
3. **One error type the checker knows.** `HttpError` is a real enum —
   `.offline`, `.timeout`, `.aborted`, `.network`, `.parse(at)`,
   `.status(code, body)` — and an endpoint's `errors { 422 -> FormErrors }`
   types `body`. Actions get Rust's `let … else` so a failure is a value
   when you want it to be.
4. **One `fetch`, one meaning.** `fetch(…)` is the same request in a
   `resource` and in an action, with the same options, and everything it
   reads — address, body, headers — is followed. The browser's is
   `window.fetch`, as today.
5. **Requests you can test.** `wf test` gains `respond` steps (a fake
   server, by endpoint or by address), and a template render or a test's
   `data:` can hand a resource its answer.
6. **A new command, `wf api`,** that writes a service's WebFluent from an
   OpenAPI file, one file per tag plus a file of shared types, and writes
   it again safely when the spec changes.

Section 8 breaks it into nine stages. Stage 1 is bug fixes that are worth
doing whatever is decided about the rest.

---

## 1. How it works today

### The four ways in

```wf
type User { id: String, name: String }

api Backend(base: "/api") {
    get  users(page: Number = 1) -> [User]
    get  user(id: String) at "users/:id" -> User
    post createUser(body: Map) -> User
        errors { 422 -> Map }
}

page Users(path: "/users", title: "Users", description: "Everyone.") {
    state page = 1
    resource rows = Backend.users(page: page)          // 1. a read, over an endpoint
    resource stats = fetch("/api/stats")               // 2. a read, over an address
    state draft = ""

    action add() {
        let u = await Backend.createUser(body: { name: draft })   // 3. a call, in an action
        let ping = await fetch("/api/ping")                        // 4. a fetch, in an action
        rows.reload()
    }
    …
}
```

### What each compiles to

| Written | Compiled | Runtime |
|---|---|---|
| `api Backend(…) { … }` | `WF.api({ name, base, endpoints: { users: { method, path } } })` | `api.js` → one callable per endpoint, each through `send` |
| `resource r = Backend.ep(args)` | `WF.resource(Backend.ep, { call: true, args: () => ({…}) })` | `net.js` `resource` → re-runs when `args()` changes |
| `resource r = fetch(url, opts)` | `WF.resource(() => url, { …opts })` | the same, re-runs when `url()` changes |
| `await fetch(url, opts)` | `WF.request(url, opts)` | `net.js` `request` → `send`, cache `.none` |
| `await Backend.ep(args)` | `Backend.ep({…})` | `send` |
| `match r { loading … error(e) … ready(v) … }` | `WF.match(parent, () => r.state(), …)` | `match.js`, keyed on the arm |

`send` (`http.js`) is the engine: timeout, an abort that follows the
caller's signal, retries with backoff and `Retry-After`, a 256-entry cache
keyed by method, address, body and credentials, `ETag` / `If-Modified-Since`
revalidation, joining a request already in flight, upload progress over
XHR, the `on request` / `on response` / `on error` hooks, the offline write
queue, and — under `wf serve` — holding a response to its declared type.

`resource` (`net.js`) is a good piece of work: a first load is `loading`, a
refresh keeps `ready` and the data, a refresh that fails keeps the data and
sets `error`, an answer is reconciled into what was shown so keyed rows keep
their nodes, a superseded answer never lands, and leaving the page aborts
the request. None of that should be lost.

---

## 2. What is wrong

Each problem below was checked against the code and, where marked
**(probe)**, by compiling a small project with `wf 6.0` and reading the
output.

### 2a. Bugs: things that do not do what the docs say

**B1. A positional argument to an endpoint is always called `id`.** (probe)

```wf
api Backend(base: "/api") { get user(userId: String) at "users/:userId" -> User }
resource one = Backend.user(uid)
```

compiles to `WF.resource(Backend.user, { call: true, args: () => ({ id:
_uid() }) })` (`src/web/codegen/js/net.rs`, `resource_call`: "A single
positional argument is the one the path names" — it is hard-coded to
`id`). The path wants `userId`, finds it empty, and the call ends as
`.aborted` — so the resource stays `loading` for ever. `wf check` says
nothing.

**B2. A resource's `body:` and `headers:` are read once.** (probe)

```wf
state q = ""
resource hits = fetch("/api/search", method: "POST", body: { query: q })
```

compiles to `WF.resource("/api/search", { method: "POST", body: ({ query:
_q() }) })`. `_q()` is called once, when the page draws; typing into the
search box never asks again. Only the address is followed. The tutorial's
"Send options with a request" example is exactly this page.

**B3. `.status(code, body)` cannot be matched.** (probe) The reference says
an error is `.status(code, body)`, and the runtime makes it so
(`netError` builds `["status", code, body]`), but

```wf
error(e) { match e { .status(code, body) { Text("status {code}") }  else { … } } }
```

is `T13: nothing declares code`. The checker does not know the error's
cases, so only bare `.offline` / `.timeout` arms work.

**B4. `errors { 422 -> T }` does nothing.** The parser keeps it
(`Endpoint.errors`), but `emit_api` never writes it, and the checker does
not type `e.body` with it. It is documentation that looks like a contract.

**B5. "Two readers of one address make one request" — only with a cache
policy.** `requestOptions` gives a `fetch(…)` resource `cache: { kind:
"none" }`, and `send` only joins a request in flight through a cache entry,
which `.none` never makes. Two `resource x = fetch("/api/same")` on one page
make two requests; so do a page and a component asking for the same user.

**B6. `Backend.users.invalidate()` does not refresh the screen.** It
deletes cache entries (`http.js` `invalidate`). A `resource` showing users
does not hear about it; it only asks again on `reload()`, its own
`invalidate()`, `on:` or a change of what it reads. The tutorial's
optimistic example ends with `Backend.users.invalidate()` and nothing on the
page changes.

**B7. An endpoint with no `at` is requested at its name.** `post
createUser(body: Map)` is `POST /api/createUser`. That is rarely a real
route; it is a silent default that only shows up as a 404.

**B8. Dead code.** `wfFetch` (the WebFluent 2 `fetch x from url { loading
… success … }` runtime) and `fetchOptions` in `net.js` are reachable from
nothing the current grammar writes; `("fetch", "wfFetch")` is still in the
runtime's name table (`src/web/runtime/mod.rs:150`).

The rest were found by reading the code (file and line given); they follow
from it directly but were not each run.

**B9. The editor never reads the spec.** `api Backend from "openapi.json"`
is expanded only by `read_project_collecting_via`
(`src/project/sources.rs:186`). The language server, the WebAssembly
compiler and the template engine never call `openapi::expand`, so in the
editor a spec-backed service has no endpoints: `Backend.getUser(…)` is a
false `T06`, and hover, completion and go-to-definition see nothing.

**B10. The OpenAPI reader breaks path parameters that are not one word.**
`/users/{user_id}` becomes the path `users/:user_id` and the parameter
`userId` (`openapi.rs:716` camel-cases the prop, `:727` keeps the path) —
they never meet, so the call is never sent. `{user-id}` does not match the
runtime's `:name` pattern at all.

**B11. The OpenAPI reader drops what does not fit, silently.**
- A parameter's `in` is never read: header and cookie parameters are sent
  in the query string.
- A request body's content type is dropped: multipart and form bodies are
  sent as JSON.
- A response's media type is dropped: a `text/plain` or binary answer has
  no `as:` and fails as `.parse`.
- A response that is a `$ref`, and the `default`, `4XX`, `5XX` keys, are
  dropped; so is an inline object's shape (it becomes `Map`).
- `serde_json` runs without `preserve_order`, so "the first content type"
  and "the first 2xx" mean the alphabetically first.
- Tags, security schemes, servers, `deprecated` and examples are ignored;
  an operation name is never checked to be a name (`getV1Users.json`), and
  two endpoints of one name are not reported.

**B12. An endpoint named like a list helper is rewritten.** `method_to_js`
(`src/web/codegen/js/expr.rs:59`) turns `x.remove(i)` into a splice, and it
does so for a service too: `Backend.remove(id: x)` does not call the
endpoint. So do `filter`, `first`, `contains`, `split` and the rest.

**B13. Endpoint calls are typed as if they were already awaited.**
`Backend.user(id: x)` without `await` checks as `User`, not as a promise,
so `T17` never fires for an endpoint (`check/sema/types/method.rs:137`). A
missing required parameter is not reported. An endpoint with no `->`
returns `Null` to the checker, where the docs say `Any`. A resource's
annotation is never compared with its endpoint's return type: `resource u:
[User] = Backend.user(…)` passes. `rows.relaod()` passes too: a resource's
methods are not checked.

**B14. Per-call `headers:` and `retry:` go into the query string.** The
checker allows them on a call; the runtime's list of call options
(`api.js:52`) does not have them, so they are sent as `?headers=[object
Object]`.

**B15. A `File` parameter must be called `file`.** The build records the
parameter's name (`fileField`), but `api.js:66` only looks at `given.file`;
`post avatar(id: String, image: File)` sends the image as query text.

**B16. Two uploads at once are one upload.** An async action called again
with the same arguments joins the first call; the arguments are compared
by `JSON.stringify`, and every `File` stringifies to `{}`
(`runtime/modules/busy.js:22`).

**B17. A write kept offline returns nothing, typed as something.** The call
resolves `undefined` while the checker says `User`; and when it is sent
later, a `4xx` deletes it with nothing told to the page (`offline.js:173`).

**B18. A page and a store encode a `fetch` address differently.** A page's
`await fetch("/x/{id}")` URL-encodes `id`; a store's does not
(`expr.rs:683` against `:760`).

**B19. Two copies of a dead grammar.** The WebFluent 1 `fetch x from url {
loading … success … }` node is still matched in the scope builder, the
checker, the static paint, the printer and the linters, for a parser only
`wf migrate` runs.

And three gaps, not bugs:

- A `match` over a resource always paints `loading` in the static page
  and in a template render (`ssg.rs:821`, `template/mod.rs:1479`): a PDF can
  never show fetched data, even when the data is handed in.
- No browser suite exercises an `api`, an endpoint, `optimistic` or `await
  fetch`; no project in `examples/` uses `api` or `resource`; `openapi.rs`
  has no unit tests of its own.
- `wf verify` fails every page whose API is not reachable, with no way to
  answer for it.

### 2b. Design problems

**P1. Two homes for server data.** A page can hold a list as a `resource`
(read-only, refreshed, cached) or as a `state` filled by a `load()` action
(writable, never refreshed). The tutorial teaches both, and the second is
what anyone who needs to *change* the list ends up with:

```wf
store Todos {
    state items: [Todo] = []
    action load() { items = await fetch("/api/todos") }
    action add(t: String) {
        let created = await fetch("/api/todos", method: "POST", body: { title: t })
        items.push(created)
    }
}
```

That is a hand-written cache: no refresh on focus, no dedupe, no stale
marker, and the page and the server drift. Every library that has made data
fetching pleasant (TanStack Query, SWR, RTK Query, Apollo, Remix) has the
same answer: the server's data lives in one cache, reads subscribe to it,
writes tell it what changed. WebFluent has the cache and the reads — it is
missing the writes.

**P2. `fetch` means three things.** In a `resource` it is a declarative
read whose options are syntax (`parse_resource` looks for the word). In an
action it is `WF.request`, a one-off call that returns parsed JSON and
throws. In a project script it is the browser's. A `resource` cannot be
anything else: `resource n = Stats.compute(range)` (an async store action)
or `resource n = loadFromScript(id)` is a parse error, because the right
side must be `fetch(…)` or a capitalised call.

**P3. Options are spread over five places, with different rules.** A
policy may be written on the service (`timeout:`), on the endpoint
(`cache:`), on the call (`cache:`, `retry:`), on the resource (`on:`,
`paginate:`) or on `fetch(…)`; which ones follow state differs (B2), and
`on:` / `paginate:` are only allowed in a resource, which the checker does
not say.

**P4. Errors are typed in prose only.** `e` in `error(e)` is a `NetError`
whose case names are checked but whose payloads bind nothing (B3), and in
`catch e` it is `Any`, so `e.mesage` passes there. `.error` is `Any?` in
one table of the checker and `NetError?` in another. Nothing distinguishes "the network is down" from "the server said
no" from "the server said something we could not read" at the type level,
which is the distinction libcurl, reqwest and httpx all make first.

**P5. Every failure in an action is an exception.** A form that wants to
show a `422`'s field errors writes `try { … } catch e { if e.status == 422 {
form.apply(e.body) } else { … } }`. There is no way to get the outcome as a
value, the way `Result` does in Rust or `{ data, error }` does in
openapi-fetch.

**P6. Pagination is spelled with a borrowed word.** `paginate: .page`
uses the enum case `.page` as *the name of the argument*, needs a `state
page` the runtime then writes to, knows only page numbers, and learns that
the list ended by asking for an empty page. Cursor APIs and `Link` headers
— most real APIs — cannot be expressed.

**P7. Retrying after a `401` returns a magic string.** `on error(e) { …
return "retry" }`: any other string, or a typo, silently does not retry.

**P8. Optimistic updates hold the wrong thing.** `optimistic(holder, f)`
takes a `state`; a `resource`'s data cannot be its holder, so an optimistic
change forces P1's hand-written cache.

**P9. Nothing to test with.** `wf test` cannot fake a response, so a test
of a page that fetches sees the `loading` arm, or the real network. A
template render cannot hand a resource its answer.

**P10. The OpenAPI import is all or nothing, and invisible.** `api Backend
from "openapi.json"` builds the whole surface inside the compiler — and
not even the editor sees it (B9). You
cannot read the endpoint list in the repository, review what a spec change
did to it in a diff, split it by area, or fix one endpoint the spec gets
wrong without forking the spec.

### 2c. What is good and stays

- `match` over `loading` / `error(e)` / `ready(v)`, and `.state`, `.data`,
  `.error`, `.refreshing`, `.pending` read anywhere.
- First load vs refresh, reconciled answers, abort on leaving, a superseded
  answer dropped.
- The engine: timeout, backoff with jitter and `Retry-After`, ETags, cache
  keyed by credentials, the offline queue, upload progress.
- An `api` block as the one description of a service; `from
  "openapi.json"`.
- Connections (`socket`, `stream`, `channel`, `peer`, `worker`) — a
  different job, out of scope except where noted.

---

## 3. What others do, and what fits WebFluent

### JavaScript and TypeScript (weighted most)

| Design | The idea | Take it? |
|---|---|---|
| `fetch` + `AbortController` | A request is a promise; cancellation is a signal you pass. Transport failure rejects, an HTTP error status does not. | Keep `window.fetch` raw. Our abort-by-scope is already better than a hand-passed signal. |
| async / await | Sequential code over promises. | Already ours in actions. |
| **TanStack Query** | *Queries* (reads, keyed, cached, deduped, refetched on focus/reconnect, stale-while-revalidate by default) vs *mutations* (writes, `onMutate` / `onError` / `onSettled`), and `invalidateQueries(key)` refetches the active queries under a key prefix. `isLoading` vs `isFetching`. Infinite queries take `getNextPageParam`. | **Yes — the core of the plan.** We already have the query half; we add mutations that invalidate, and stale-while-revalidate as the default. |
| SWR | The key *is* the URL; `mutate(key)` revalidates everywhere it is read; dedupe within a window. | Yes: a read's key is its request, and a write names keys. |
| **RTK Query** | Endpoints declared once as `query` or `mutation`; `providesTags` / `invalidatesTags` connect them, so a mutation refetches the right queries with no call-site code. | **Yes — closest to our `api` block.** `changes:` on a write is `invalidatesTags`; the path is the default tag. |
| Apollo | A normalised cache by `id`: a mutation's returned object updates every query holding it. | Partly: `reconcile` already matches by `id`. A full normalised cache is too much; invalidate-and-refresh is enough (open question 4). |
| tRPC | End-to-end types: the client is the server's types. Query vs mutation is in the procedure. | The idea that a call site is typed from the contract — we have it through `api`/OpenAPI. |
| Remix / React Router loaders + actions | Reads run before render; after any action, every loader on the page revalidates. Forms work without JS. | The "after a write, refresh what is shown" rule is the simple fallback (`changes: .all`). We have no server, so loaders stay out. |
| Next.js `fetch(…, { next: { tags } })` + `revalidateTag` | Tags on reads, invalidated by name. | Same as RTK's tags. |
| React Suspense / `use(promise)` | Reading a pending value suspends to the nearest boundary. | No: `match` is explicit and reads better in a page; Suspense's implicit boundaries are what people find hardest. |
| openapi-typescript / **openapi-fetch** | Types only, generated from the spec; calls are `client.GET("/users/{id}", …)` and return `{ data, error }` with `error` typed per status. | Yes: errors as values typed per status (`let … else`, `HttpError.status(code, body)`). |
| **orval** | Generates a client from OpenAPI, `tags-split` mode writes one file per tag plus a schemas folder, and can generate MSW mocks from examples. | **Yes — the shape of `wf api`**, and mocks for `wf test` from the spec's `example`s. |

### Rust

| Design | The idea | Take it? |
|---|---|---|
| reqwest | A client with defaults (base, timeout, headers); a builder per request; a non-2xx is *not* an error until you call `.error_for_status()`; `.json::<T>()` decodes to a type. | The client-with-defaults is our `api`. A decode failure is its own error (`.parse`). |
| `Result<T, E>` and `?` | Errors are values; `?` passes them up; `match` takes them apart. | Yes: `await call` keeps throwing (that is our `?`); `let x = await call else e { … }` gives the value form. |
| `async` / `.await`, cancellation by drop | Dropping a future cancels it. | Ours: leaving a scope cancels its reads. |
| serde | The type *is* the decoder; a missing field is an error at the boundary. | Yes: the declared type checks the answer (dev builds today; a `strict` option for production). |
| progenitor | Generates a typed client from OpenAPI, with `ResponseValue<T>` and `Error<E>` typed per status. | Same lesson as openapi-fetch. |

### Python

| Design | The idea | Take it? |
|---|---|---|
| requests / httpx | `Session`/`Client` with `base_url`, auth, hooks; `raise_for_status()`; httpx has a **default timeout** (5 s) because "no timeout" hangs. | Yes: a default timeout (30 s) when a service says none. Event hooks = our `on request`/`on response`. |
| pydantic | Validate at the boundary; a bad payload is a typed error naming the field (`users.1.name: field required`). | Our dev-mode `.parse` message is already this. |
| openapi-python-client | One module per tag (`api/users/get_user.py`), `models/`, sync and async variants, and a `_detailed` variant returning status + parsed body. Regenerates by overwriting the package. | Module-per-tag, yes. Overwrite-by-default, no: we detect hand edits. |

### C

| Design | The idea | Take it? |
|---|---|---|
| libcurl | Everything explicit: an easy handle per transfer, a multi handle for many; `CURLcode` for *transport* errors kept apart from the HTTP *status* (`CURLOPT_FAILONERROR` merges them, on request). | The lesson: transport error, HTTP status and decoding are three different things. `HttpError` keeps them apart. Explicitness we do not copy: our defaults should be right. |

### The principles, as they apply here

1. **Reads and writes are different.** A read is declared where it is shown
   and has a lifetime; a write is a step in an action. (TanStack, RTK,
   Remix, tRPC.)
2. **A read is known by its request.** Same method, address, body and
   credentials → same read, one request, one cached answer. (SWR,
   TanStack.)
3. **Writes say what they change; the cache does the rest.** (RTK tags,
   Next tags, Remix revalidation.)
4. **Show what you have, then refresh it.** Stale-while-revalidate is the
   default, not an option. (TanStack, SWR.)
5. **Errors are values with kinds**: transport, status (with a typed body),
   decoding. Throwing is a convenience on top. (Rust, libcurl,
   openapi-fetch.)
6. **The contract types every call**, and is checked at the boundary.
   (serde, pydantic, tRPC.)
7. **Generated code is reviewable code.** One file per area, shared types
   apart, and regeneration never eats a hand edit. (orval,
   openapi-python-client, progenitor.)
8. **A fake server is part of testing**, keyed the way the real calls are.
   (MSW, orval mocks.)
9. **Sensible defaults**: a timeout, dedupe, refetch on reconnect.

What does *not* fit: Suspense-style implicit loading (WebFluent is explicit
by design and pre-renders the `loading` arm); a client-side normalised
entity cache (too much machinery for the gain); loaders that run on a
server (WebFluent builds static files).

---

## 4. The design

### 4.1 The vocabulary, in one table

| | Today | Proposed |
|---|---|---|
| A read shown on a page | `resource r = fetch(…)` / `= Backend.ep(…)` | the same, and `= any async call` |
| A write | `await fetch(…)` / `await Backend.ep(…)` in an action | the same; it also refreshes the reads it changes |
| The value of a failure | `try { } catch e { }` | that, or `let x = await call else e { … }` |
| An error | `Any` at check time | `HttpError`, typed per endpoint |
| Which reads a write changes | you call `reload()` by hand | the path, or `changes:` |
| Pages of a list | `paginate: .page` + a `state page` | `paginate: .page(page)`, `.cursor(…)`, `.link` |
| Fake answers in a test | — | `respond Backend.users with [...]` |
| A service from OpenAPI | `api B from "x.json"` | that, or `wf api` writing the files |

### 4.2 Reads: `resource`

The syntax stays. Three things change.

**Any async value can be a resource.** The right side is any expression that
makes a request or returns a promise: `fetch(…)`, an endpoint, an async
store action, a project script's function. What the expression reads is
followed, exactly as a `derived` follows what it reads.

```wf
resource hits  = fetch("/api/search", method: "POST", body: { query: q })   // q followed (fixes B2)
resource user  = Backend.user(userId)                                       // the path's own name (fixes B1)
resource stats = Metrics.summarise(range)                                   // an async store action
resource tiles = loadTiles(viewport.width)                                  // a script function returning a Promise
```

A read over something that is not an HTTP request (the last two) has the
states, the refresh and the abort-on-leave, but no cache key: it is not
shared.

**Every HTTP read is shared and kept.** Its *key* is its request: method,
address, body, and the credentials it sends. Two reads with one key make one
request and hold one answer — on one page, or across a page and a component,
or across pages. A read that comes back to a key already held (going back to
a page, a tab switch) shows what is held at once, as `ready`, and refreshes
behind it. That is TanStack's default and it is what makes going back
instant.

Defaults, each overridable per service, endpoint or call with the existing
`cache:` option:

| | Default | Meaning |
|---|---|---|
| fresh for | `0` | A new reader always refreshes (but shows what is held while it does). |
| kept for | `5.minutes` after the last reader leaves | Then the answer is dropped. |
| `cache: .swr(60.seconds)` | — | Fresh for 60 s: a new reader within that does not ask again. |
| `cache: .none` | — | Not shared, not kept: every reader asks. |

Writes (`POST`, `PUT`, `PATCH`, `DELETE`) are never cached or shared.

**Refetch defaults.** A read refreshes when the network comes back
(`.reconnect`) unless it says `on: .never`; focus stays opt-in (`on:
.focus`) because it surprises people on pages with forms. Open question 3.

Nothing else about `resource` changes: `match`, `.state`, `.data`,
`.error`, `.refreshing`, `.pending`, `.reload()`, `.cancel()`.

`resource.invalidate()` is renamed in meaning, not in name: it marks the
read stale *for every reader of the key* and refreshes the ones on screen.

### 4.3 Writes: calls in actions

The syntax stays: a write is `await` of an endpoint or of `fetch(…, method:
"POST")` in an action or a handler. What is new is what happens after it
succeeds: **every read it changes is marked stale, and the ones on screen
refresh in place** (a refresh, so the page keeps its data and its focus).

Which reads a write changes:

1. **By its path (the default).** A write to `users/:id` changes every read
   whose path is `users` or starts with `users/` — the collection and
   every member — on the same service. A write to `users` changes the same.
   For `fetch`, the same rule on the address, without the query string.

   | Write | Reads it refreshes |
   |---|---|
   | `POST users` | `GET users`, `GET users?page=2`, `GET users/7` |
   | `PATCH users/7` | `GET users`, `GET users/7`, `GET users/7/keys` |
   | `DELETE users/7/keys/3` | `GET users/7/keys…` and up: `users/7`, `users` |

   (Up the path, because a parent usually embeds a count or a summary of its
   children; across siblings, never.)

2. **By `changes:`, when the path is not enough.** On the endpoint:

   ```wf
   api Backend(base: "/api") {
       get  users(page: Number = 1) -> [User]
       get  user(id: String) at "users/:id" -> User
       get  stats() -> Stats
       post invite(email: Email) at "invites" -> Invite
           changes: [users, stats]          // endpoint names, every argument
       post rename(id: String, name: String) at "users/:id/name"
           changes: [user(id), users]       // one argument's read, and all of users
       post ping() at "ping"
           changes: .nothing                // a write that changes nothing we read
   }
   ```

   `changes: .all` refreshes every read of the service — Remix's rule, for
   a service whose paths say nothing.

3. **By hand.** `Backend.users.invalidate()` and `Backend.invalidate()`
   keep working and now refresh the screen (fixes B6).

A write's result can also be put straight into the reads, when the server
returns the new object and a refresh would be a wasted request:

```wf
action rename(id: String, name: String) {
    let u = await Backend.rename(id: id, name: name)
    Backend.user.put(id: id, value: u)       // the read of user(id) now holds u; no request
}
```

`.put(args, value)` is TanStack's `setQueryData`. Rarely needed; listed so
the optimistic story (4.8) has a name to stand on.

### 4.4 Errors: `HttpError`

A built-in enum the checker knows, with the runtime's existing shape (an
array whose first item is the case, so `match` already reads it):

```wf
enum HttpError {
    offline                              // no network, or our own server unreachable
    timeout
    aborted
    network(message: String)             // another origin refused, CORS, DNS
    parse(at: String)                    // not JSON, or not the declared type: "$[1].name is missing"
    status(code: Number, body: E)        // the server answered, and it was not 2xx
}
```

and, on every `HttpError`, the fields `.message`, `.status` (0 when there is
no status) and `.headers`.

`E` — the type of `body` — comes from the endpoint: the union of its
`errors { … }` types, or `Any` when it declares none. So:

```wf
api Backend(base: "/api") {
    post signUp(body: SignUp) at "users" -> User
        errors { 422 -> FieldErrors, 409 -> Conflict }
}

error(e) {
    match e {
        .offline           { Alert("You are offline.").warning }
        .status(422, body) { Text("{body.fields.length} fields need a look") }   // body: FieldErrors
        .status(409, body) { Text(body.existing) }                               // body: Conflict
        .status(code, _)   { Alert("The server said {code}.").danger }
        else               { Alert(e.message).danger }
    }
}
```

That needs one small language addition: **a literal in a case pattern**
(`.status(422, body)`), which narrows `body` to that status's type. Without
it, `.status(code, body)` binds `body` as the union.

`e` in a resource's `error(e)` arm and in `catch e` around a request is
`HttpError` (a `catch` around other code stays `Any`). `e.mesage` is `T05`.

### 4.5 A failure as a value: `let … else`

`await call` still throws — that is WebFluent's `?`, and most actions want
it. For the actions that want the failure in hand:

```wf
action submit() {
    let user = await Backend.signUp(body: draft) else e {
        match e {
            .status(422, body) { form.apply(body) }
            else { toast(e.message, tone: .danger) }
        }
        return
    }
    navigate("/users/{user.id}")
}
```

`let x = await call else e { … }`: when the call fails, the block runs with
`e: HttpError`, and the block must leave (`return`, or end the action) —
the same rule as Rust's `let … else`, so `user` is never unset below it.
The checker reports a block that can fall through (new `N03`).

### 4.6 One `fetch`

`fetch(url, options)` is the same thing in a `resource` and in an action:
the engine's request, with the options below, every one of which follows
state in a `resource`. It takes a type to check the answer by — `resource
r: [Row] = fetch(…)` as today, and in an action `let r: [Row] = await
fetch(…)`, which today is checked statically and, once this lands, held to
the shape under `wf serve` too.

| Option | Values |
|---|---|
| `method:` | `"GET"` … |
| `headers:` | a map |
| `body:` | a map (JSON), a string, a `File`, `FormData`, `Blob`, … |
| `query:` | a map, appended to the address (new — today one writes `?a={a}`) |
| `as:` | `.json` (default), `.text`, `.blob`, `.arrayBuffer`, `.none` |
| `timeout:`, `retry:`, `cache:` | as on a service |
| `on:` | in a `resource`: `.focus`, `.reconnect`, `.interval(d)`, `.never` |
| `changes:` | on a write: the addresses it changes, when the path rule is not right |

The browser's own stays `window.fetch`.

### 4.7 Loading and refresh

Unchanged, because it is the part that works. Two additions:

- **`placeholder:`** on a read: what `ready` shows while a *new* request
  loads, instead of `loading` — usually the previous page's rows
  (`placeholder: .previous`), TanStack's `keepPreviousData`. For a search
  box or a pager this removes the flash of the spinner between pages.
- **The static paint.** A read whose answer is handed in (a template render,
  a test's `data:`, see 4.12) paints its `ready` arm; every other read paints
  `loading`, as today.

### 4.8 Optimistic updates

`optimistic(holder, change)` accepts a read as its holder, and its change is
applied to the read's data for every reader of the key:

```wf
action toggle(t: Todo) {
    optimistic(Backend.todos, list => list.map(x => if x.id == t.id { { ...x, done: !x.done } } else { x }))
    await Backend.updateTodo(id: t.id, body: { done: !t.done })
}
```

If the write throws, the change is taken back (as today). If it succeeds,
the path rule refreshes `todos` and the server's answer replaces the
guess. No state is copied out of the cache, so P1 has no reason to exist.

`optimistic(Backend.todos, …)` changes every argument's read of the
endpoint; `optimistic(Backend.todo(id: t.id), …)` one.

### 4.9 Pagination

`paginate:` takes one of three cases, each naming how the next page is
asked for, and the resource no longer writes to the page's state:

```wf
resource list = Catalog.items(paginate: .page(by: "page"))                 // ?page=1, 2, 3 … until an empty page
resource feed = Social.posts(paginate: .cursor(by: "after", next: r => r.nextCursor, items: r => r.posts))
resource hits = GitHub.search(q: q, paginate: .link)                        // the Link header's rel="next"
```

`.items`, `.loadMore()` and `.hasMore` stay. `.hasMore` is false when
`next:` gives `null` or the `Link` header has no `next` — no empty request
at the end. For a numbered pager (`Pagination(bind: page)`) there is no
`paginate:` at all: the page number is an argument and each page is its own
read, which with 4.2's sharing makes going back a page instant.

### 4.10 Streaming

A response read as it arrives (NDJSON, a log tail, a model's tokens) is an
endpoint returning `Stream<T>`:

```wf
api Logs(base: "/api") { get tail(id: String) at "deploys/:id/log" -> Stream<LogLine> }

resource lines = Logs.tail(deployId)
for l in lines.items by l.seq { Text(l.text) }
```

A stream read's `.items` grows as lines arrive, its state is `ready` from
the first line, and leaving the page aborts it. (Today `streamLines` exists
as `Backend.ep.lines(args)`, untyped and undocumented outside a comment.)
`socket` and `stream … = sse(…)` stay as they are.

### 4.11 Auth

`headers { Authorization: "Bearer {Session.token}" }` stays (read at each
call). Two changes:

- `on error(e)` says `retry`, a statement, instead of `return "retry"`:

  ```wf
  on error(e) { if e.status == 401 { await Session.refresh()  retry } }
  ```

  `retry` outside an `on error` hook is a compile error. Concurrent `401`s
  already share one refresh, since an async action called twice with the
  same arguments is one call (`sameCall`) — the docs should say so.
- Logging out calls `Backend.invalidate()`, which now drops what every
  reader holds, so the next user never sees the last one's answers. (The
  cache is already keyed by credential headers; a cookie cannot be read,
  which is why this matters.)

### 4.12 Static paint, templates and tests

**Handing a read its answer.** In `wf render` and in a test's `data:`, a
key named like a resource is that resource's answer, and its `match` paints
`ready`:

```wf
test "lists the users"(data: { rows: [{ id: "1", name: "Ada" }] }) {
    Users
    expect "Ada"
}
```

**A fake server.** A new test step, `respond`, answers a request the page
makes, by endpoint or by address, before the page loads:

```wf
test "shows a 422 as field errors" {
    respond Backend.users with [{ id: "1", name: "Ada" }]
    respond Backend.signUp status 422 with { fields: ["email"] }
    respond "GET /api/stats" with { count: 3 }
    respond Backend.slow after 2.seconds with []
    respond Backend.flaky offline

    SignUp
    type "ada@" into "Email"
    click "Join"
    expect "1 fields need a look"
}
```

A request the test does not answer fails the test, naming it (`GET
/api/me was not answered`), so a test never reaches the real network.
`expect requested "POST /api/users"` checks that a write went out. The
answers are checked against the endpoint's declared type, so a fake
cannot drift from the contract. With a spec (`wf api`), `respond
Backend.users` with no `with` uses the spec's `example`.

The same table serves `wf verify --respond tests/responses.wf`, so a site
whose API is elsewhere can still be verified.

### 4.13 Offline

`offline.sync` still queues failed writes and sends them when the network
returns. Three changes (B17):

- a queued write's `changes:` are applied when it is finally sent, so the
  reads catch up;
- the call's type says it may be queued: under `offline.sync` a write's
  result is `T?`, `null` when kept for later, so a page cannot read a
  field of an answer that never came;
- a kept write the server refuses later is reported to the page —
  `network.refused`, a list of what was refused and why — instead of
  vanishing.

---

## 5. The new command: `wf api`

### What it is for

`api Backend from "openapi.json"` is a good start and a poor team tool:
the endpoints and types exist only inside the compiler (P10). `wf api`
writes the same surface as WebFluent files you commit, read, review in a
diff, and keep in step with the spec.

### The commands

```bash
wf api add openapi.json --name Backend [--out src/api/backend] [--base env.PUBLIC_API]
wf api sync [Backend]            # write the files again from the spec
wf api check [Backend]           # fail when the files are behind the spec (for CI)
wf api eject Backend             # turn `api Backend from "…"` into files
wf api list [--json]             # services, their specs, endpoints by tag
```

`add` records the service in `webfluent.app.json`:

```json
{ "apis": { "Backend": { "spec": "openapi.json", "out": "src/api/backend", "base": "env.PUBLIC_API" } } }
```

### What it writes

For a spec with tags `users` and `billing`:

```text
src/api/backend/
├── service.wf        api Backend(base: …) { settings } — written once, then yours
├── types.wf          every components.schemas type and enum
├── users.wf          endpoints tagged users
├── billing.wf        endpoints tagged billing
└── untagged.wf       endpoints with no tag (only when there are some)
```

`types.wf`:

```wf
// Written by `wf api sync` from openapi.json (sha256 4be1…). Do not edit:
// declare a type of the same name elsewhere to replace one.

/// A person with an account.
type User {
    id: Uuid
    name: String
    email: Email
    createdAt: DateTime
}

enum Plan { free, team, enterprise }
```

`users.wf`:

```wf
// Written by `wf api sync` from openapi.json (sha256 4be1…). Do not edit.

api Backend.users {
    /// Every user, a page at a time.
    get list(page: Number = 1, q: String?) at "users" -> [User]
    get get(id: Uuid) at "users/:id" -> User
    post create(body: NewUser) at "users" -> User
        errors { 422 -> ValidationErrors }
    delete remove(id: Uuid) at "users/:id"
}
```

Called as `Backend.users.list(page: 2)`.

`service.wf`, written once and never again:

```wf
// Written by `wf api add`. Yours to edit: `wf api sync` never touches it.
api Backend(base: env.PUBLIC_API ?? "https://api.example.com/v1") {
    timeout: 30.seconds
}
```

### The language change it needs: groups

Today an `api` is one block. Splitting it by file needs **groups**: `api
Backend.users { … }` declares endpoints in the group `users` of the service
`Backend`, which is declared once elsewhere with its settings. A group has
no settings of its own; it shares the service's base, headers, hooks and
cache. A group without its service is an error (`E1xx`), and two groups of
one name are `E102`. Hand-written services may use groups too.

(The other way — one flat `Backend` spread over files — would make names
from different tags collide; groups follow the spec's own structure and
read well at the call site.)

### Names

- **Group**: the tag, camelCased (`User Accounts` → `userAccounts`).
  An endpoint with several tags goes under its first.
- **Endpoint**: the `operationId`, camelCased, with the tag's name taken off
  the front or the back when what is left is still a word and unique in the
  group (`listUsers` in `users` → `list`; `getUserById` → `getById`).
  Without an `operationId`: the method and the path's last fixed segment
  (`get users/{id}/keys` → `getKeys`).
- **A name the language owns** (`get`, `delete`, `match`…) is allowed as an
  endpoint name, since it is always called after a dot; one that is not a
  WebFluent name at all gets a `_` and a comment.
- **Types**: the schema's name, PascalCased. A name both the spec and the
  program declare: the program's wins, as with `from` today.

`--names operation-id|path|keep` changes the rule for a project that wants
another; it is recorded in the config.

### Regenerating safely

1. Every file it writes starts with the header above, carrying the hash of
   the spec it came from.
2. `.wf-api.json` (beside `.wf-sizes.json`, committed) records, per file,
   the hash of what was written.
3. `wf api sync` writes a file only when it is unchanged since it was
   written. A file someone edited is not overwritten: `sync` stops, names
   it, and shows the difference; `--force` overwrites, `--keep` leaves it
   and writes the rest.
4. Changes belong outside the generated files: declare a `type` of the same
   name to replace one, or a group of your own (`api Backend.extras { … }`)
   for an endpoint the spec lacks. `wf api sync` never touches a file it did
   not write.
5. Output is deterministic — tags, endpoints and fields in the spec's
   order, formatted by `wf fmt` — so a spec change shows as the smallest
   diff.
6. `wf api check` fails (exit 1) when `sync` would change a file, and `wf
   check` reports `N07` (a warning) when a recorded spec's hash no longer
   matches, so a spec bump without a sync is caught in CI.

### How it relates to `from`

Both use the one OpenAPI reader (`src/…/openapi.rs`), so a spec means the
same thing either way: `wf api sync` prints the declarations the reader
already builds for `from`. `from` stays — for a prototype it is one line —
and `wf api eject Backend` turns it into files in one step. The docs teach
`from` first and `wf api` for a project that will live.

### What the reader needs for it

First, B9–B11: the reader runs everywhere a project is read (the build, the
language server, the WebAssembly compiler, the template engine), keeps the
file's order, reads each parameter's `in` (a header parameter becomes a
header, a cookie parameter is reported), keeps a body's and a response's
media type (`as:`, multipart), resolves response `$ref`s and `default` /
`4XX`, keeps inline objects as records named after their place
(`CreateUserBody`), and reports a name it cannot use and two endpoints of
one name. Then, new:

- **Tags and operationIds** (for groups and names).
- **YAML** specs (most are YAML). Open question 6.
- **Security schemes**: an `apiKey` or `http bearer` scheme becomes a
  commented `headers { }` line in `service.wf`, for the author to point at
  their token.
- **`servers[0].url`** as the default `base`.
- **Examples**, kept for `respond` (4.12).

---

## 6. What changes in the runtime

| Module | Change |
|---|---|
| `http.js` | The cache becomes the *read store*: an entry has readers (subscribers), a stale flag and a last-read time. `invalidate(match)` marks entries stale and notifies their readers. Entries without readers are dropped after `kept for`. Joining in flight works with any cache policy but `.none`. Default timeout 30 s. |
| `net.js` `resource` | Subscribes to its key's entry: a `refresh` from the store reloads it as a refresh. Accepts any thunk returning a promise (the "any async value" form); options become one thunk, so body and headers are followed. `placeholder:`, the new `paginate:` cases, stream reads. `wfFetch` and `fetchOptions` removed. |
| `api.js` | Endpoints carry their path's *changes* (computed at build time, emitted as data) and call `invalidate` after a successful write. Groups: `Backend.users` is an object of endpoints sharing the service's config. `.put(args, value)`. The `errors` table is emitted, so the dev-mode shape check covers failure bodies. |
| `net.js` `optimistic` | A holder may be an endpoint or an endpoint call: the change goes to every matching entry. |
| new `mock.js` (tests only) | The `respond` table, consulted by `send` before the network when `wf test` builds the page. Never in a production build. |

The checker learns `HttpError` (a built-in enum), literal case patterns,
`let … else`, `Stream<T>`, groups, `changes:`, and types `.put`, `.items`.

The compiler emits each write's change set: from `changes:`, else from the
path rule, resolved at build time where the path is static.

---

## 7. Migration and diagnostics

### `wf migrate` (6 → 7)

Rewrites, file by file, printing each:

| Before | After |
|---|---|
| `paginate: .page` | `paginate: .page(by: "page")`, and a note that the `state page` it wrote to is no longer written |
| `on error(e) { … return "retry" }` | `… retry` |
| `post createUser(…)` with no `at` | `post createUser(…) at "createUser"` — making today's silent default visible, with a note to check it |
| `optimistic(state, …)` over a `state` filled by a load action | unchanged, with a note pointing at 4.8 |

Behaviour that changes with no rewrite, which `migrate` states:

- a `resource`'s `body:` and `headers:` are now followed (B2);
- reads are shared and kept by default — a page that relied on two reads of
  one address making two requests (rare) says `cache: .none`;
- a successful write refreshes the reads its path covers — a write that must
  not says `changes: .nothing`;
- a request without a `timeout:` now gives up after 30 s.

### New diagnostics, family `N` (network)

| Code | Severity | What |
|---|---|---|
| `N01` | error | An endpoint call whose positional argument cannot be matched to a path parameter (B1's case, caught at build time) |
| `N02` | warning | A `resource` over a write (`post`, `put`, `patch`, `delete`): it would send on every change of what it reads |
| `N03` | error | A `let … else` block that can fall through |
| `N04` | warning | An `await` of a read endpoint whose result nothing uses |
| `N05` | error | `changes:` naming an endpoint the service does not have, or an argument it does not take |
| `N06` | error | `retry` outside an `on error` hook |
| `N07` | warning | A service written by `wf api` whose spec has changed since (`wf api sync`) |
| `N08` | warning | An endpoint with no `at` whose name is not a plausible path (`createUser`) |

`T21` stays; `T11` learns `HttpError`.

---

## 8. Stages

Each stage builds, passes the full suite, and ships the docs for what it
adds. "Rust tests" means unit tests beside the code and `tests/*.rs`;
"browser" means a `tests/browser/*.mjs` suite run in headless Chrome.

**Stage 1 — Fix what is broken (no new syntax).** In three commits:

- *Calls that never send or send the wrong thing*: B1 (match a positional
  argument to the path's parameter; `N01` when there is none), B2 (a
  `fetch` resource's options are one thunk), B12 (an endpoint is never a
  list helper), B14 (per-call `headers:`/`retry:` reach the request), B15
  (a `File` parameter of any name), B16 (a `File` argument is compared by
  identity), B18.
- *The checker*: B3 (`.status(code, body)` binds), B4 (emit and type
  `errors`), B13 (an endpoint call is a promise, `T17`, missing required
  parameters, an annotation held to the endpoint, a resource's methods
  checked), the error type made one.
- *The cache and the dead code*: B5 (join in flight for every read), B6
  (an endpoint's `invalidate()` reloads the resources over it), B7 (`N08`),
  B8 and B19 (remove `wfFetch`, `fetchOptions`, the name-table entry and the
  WebFluent 1 arms outside `migrate`).

B9–B11 belong to stage 7, where the reader is rewritten; B17 to stage 4.
*Tests*: codegen tests for B1, B2, B4, B12, B14, B15; checker tests for B3
and B13; a new browser suite `tests/browser/fetch.mjs` against a small local
server that counts and records requests (B2 asks again as you type, B5 one
request for two readers, B6 the screen refreshes, B14 the header arrives,
B15 the form arrives) — the first browser suite to exercise `api` at all.
*Delivers*: the tutorial's examples do what they say.

**Stage 2 — `HttpError` and `let … else`.**
The built-in enum, typed `error(e)` and `catch e`, literal case patterns,
`errors { }` typing `body`, `let … else` with `N03`.
*Tests*: `check/sema/types/tests.rs` cases for each pattern; parser and
printer round-trips; a browser test of a 422 shown through `let … else`.

**Stage 3 — The read store.**
Shared reads by key, `kept for`, stale-while-revalidate by default,
reconnect refetch default, `placeholder:`, default timeout, any async value
as a resource.
*Tests*: `http.js` unit tests in the runtime's JS test harness for
subscribe/notify/drop; browser: back-navigation shows held data with no
spinner, two components one request, a script promise as a resource.

**Stage 4 — Writes that refresh.**
The path rule, `changes:`, `.put`, `optimistic` over a read, the change set
emitted at build time, the offline queue applying changes.
*Tests*: Rust tests for the path rule (a table of write/read pairs); browser:
create → list refreshes, rename → detail and list refresh, optimistic
rollback on a 500.

**Stage 5 — Pagination and streams.**
`.page(by:)`, `.cursor(…)`, `.link`, `Stream<T>` reads.
*Tests*: browser suites against a local server serving each style.

**Stage 6 — Testing.**
`respond`, `expect requested`, data-seeded reads in `wf test` and `wf
render`, the unanswered-request failure.
*Tests*: `wf test` runs over a fixture project; the template engine's tests
for seeded reads.

**Stage 7 — Groups and `wf api`.**
`api Backend.users { }`; the reader fixed (B9–B11: run by the language
server, the WebAssembly compiler and the template engine too; parameters by
`in`; media types; order kept) and taught tags, operationIds, YAML,
security, servers, examples; `wf api add|sync|check|eject|list`;
`.wf-api.json`; `N07`. The reader gets unit tests of its own.
*Tests*: golden files — three real specs (a small one, Petstore, and a large
one such as GitHub's or Stripe's subset) in `tests/fixtures/openapi/`, their
generated folders checked in and compared; a test that the generated
project builds and that `from` and generated produce the same `wf types
--json`; regeneration tests (hand edit detected, `--force`, `--keep`,
deterministic output).

**Stage 8 — Migration.**
The `wf migrate` 6 → 7 step, `N02`, `N04`, `N05`, `N06`.
*Tests*: `tools/migrate` before/after fixtures; each diagnostic in
`diagnostics` tests and an `md-docs/errors/N0x.md` entry.

**Stage 9 — Docs and site.**
Rewrite `md-docs/learn/15-data.md` around reads and writes; new recipes
(a form with server errors, a list that updates after a create, testing a
page that fetches, generating a client with `wf api`); update `AGENTS.md`'s
Data Fetching and service sections, `coming-from/react.md`'s TanStack
mapping, and the Arabic pages that mirror them.

Stages 2–6 depend on 1; 7 is independent of 2–6 and can run beside them;
8 and 9 come last.

---

## 9. Open questions, each with a recommendation

1. **Keep the word `resource`, or rename it to `query`?** *Keep
   `resource`.* It is taught everywhere, and `query` is already a browser
   value (`query.tab`). The read/write split is in what you do, not in a new
   keyword.
2. **Should writes refresh by path with no opt-in?** *Yes.* It is right for
   REST services and costs one GET when it is wrong; `changes: .nothing`
   and `cache: .none` are the escapes, and `migrate` says it changed.
3. **Refetch on focus by default?** *No — reconnect only.* Focus refetches
   surprise people with half-filled forms over a list; it stays one word
   away (`on: .focus`).
4. **A normalised cache (Apollo)?** *No.* Invalidate-and-refresh plus
   `reconcile` keeping rows by `id` gives the same screen for far less
   machinery.
5. **Should `await` of a read in an action be aborted when its page
   leaves?** *Reads yes, writes no.* A save must finish; a lookup for a
   page that is gone need not.
6. **YAML specs: a dependency, or our own reader?** *Our own small YAML
   reader for the subset OpenAPI uses*, matching the project's habit of
   owning its formats (the compressor, the PDF engine); fall back to
   telling the user to convert with a one-line command if a spec uses
   anchors or tags we do not read.
7. **May `wf api add` take a URL?** *No.* The compiler never fetches; the
   docs show `curl -o openapi.json …` first.
8. **Production response checking?** *Off by default, `strict: true` on a
   service to turn it on* (pydantic-style `.parse` errors), since it costs
   time on every answer.
9. **Where is the default timeout set?** *30 s, in the engine, overridable
   on the service.* Higher than httpx's 5 s because uploads go through the
   same engine.
10. **Literal patterns in `match` beyond errors?** *Yes, generally* — any
    case payload may be matched against a literal — since it is the same
    checker work and makes the feature unsurprising.

---

## Appendix: where things are

| What | Where |
|---|---|
| Grammar for `resource`, `api`, endpoints | `src/syntax/parser/v2.rs` (`parse_resource`, `parse_api`, `parse_endpoint`) |
| AST | `src/syntax/parser/ast.rs` (`ResourceDecl`, `ApiDecl`, `Endpoint`, legacy `FetchDecl`) |
| OpenAPI reader | `src/project/openapi.rs` (`expand`, called from `src/project/sources.rs:186` only) |
| Checker | `src/check/sema/types/{method,expr,stmt,members,names,call}.rs`, `src/check/sema/mod.rs` |
| Codegen | `src/web/codegen/js/net.rs` (resources, connections), `store.rs` (`emit_api`), `expr.rs` (`fetch` and endpoint calls), `flow.rs` (`sameCall`, `attempt`) |
| Static paint, templates, tests | `src/web/codegen/ssg.rs:821`, `src/template/mod.rs:1479`, `wf test` steps in `ast.rs:349` and `testing.rs` |
| Existing tests | `tests/language_4_0.rs` (services, OpenAPI), `tests/store_resource.rs`, `tests/offline.rs`; browser `store_resource.mjs`, `offline.mjs`, `leaks.mjs`, `busy.mjs` |
| Runtime | `src/web/runtime/modules/http.js` (engine), `net.js` (`resource`, `request`, `optimistic`), `api.js` (services, connections), `match.js`, `offline.js` |
| Tutorial | `md-docs/learn/15-data.md`; recipes `page-through-results`, `load-more`, `sign-in-with-a-session-cookie`, `api-on-the-same-origin`, `debounce-a-search` |
