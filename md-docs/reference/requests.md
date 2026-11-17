---
chapter: 46
kind: "reference"
route: "/reference/requests"
group: "reference"
blurb: "Every setting of a service, the endpoint grammar, call options, a read's members, HttpError, caching, retries, middleware, fakes and the N codes."
description: "Lookup tables for api, endpoints, http calls and resources: settings, options, members, HttpError, caching, retry, middleware, fakes and codes."
---
# Services and requests

Lookup tables for `api`, `http` and `resource`. [Fetching data](../learn/15-data.md)
teaches them in order.

## Service settings

What `api Backend(…) { … }` takes, in its parentheses or at the top of its block, before the endpoints; a group (`api Backend.users { … }`) takes none of them.

| Setting | Values | Unset |
|---|---|---|
| `base:` | The address every path is below: `"/api/v1"`, `env.PUBLIC_API` | — |
| `timeout:` | A duration, after which a call fails `.timeout(limit)`: `10.seconds` | `30.seconds` |
| `retry:` | A count (`3`), `.never`, or `.backoff(times: 3, on: […], delay: 300, max: 30000, jitter: true)` ([Retries](#retries)) | no retries |
| `cache:` | `.maxAge(d)`, `.swr(d)`, `.forever`, `.none`, for every `GET` of the service ([Caching](#caching)) | what the server's headers allow |
| `credentials:` | `.omit`, `.sameOrigin`, `.include` — whether cookies go with a call | `.sameOrigin` |
| `mode:` | `.cors`, `.noCors`, `.sameOrigin` | `.cors` |
| `redirect:` | `.follow`, `.error`, `.manual` | `.follow` |
| `referrer:` | A referrer policy: `.noReferrer`, `.origin`, … | the page's |
| `middleware [ … ]` | The steps every request passes through, in order ([Middleware](#middleware)) | none |
| `invalidations { … }` | `people.deletePerson(id) invalidates [people.listPeople, people.getPerson(id)]`, one line a write, checked as if written on the endpoint (`N05`) | — |
| `pagination { … }` | `people.listPeople pages .cursor(…)`, one line a read, checked as if written on the endpoint (`N14`) | — |
| `from "openapi.json"` | `api Backend from "openapi.json" (base: …)`: the contract read from a spec | — |

## Endpoint grammar

One line an endpoint: `verb name(parameters) at "path" -> result`, then its modifiers.

| Part | Written | Means |
|---|---|---|
| Verb | `get` `post` `put` `patch` `delete` `head` `options` | The HTTP method |
| Path | `at "users/{id}"` | Required; below `base`. Every `{name}` is a parameter (`N01`) |
| Path parameter | `id: String`, named in the path | Filled in, URL-encoded. `null`, `""`, `.` or `..` is not sent: the call ends `.cancelled` |
| Body | `body: NewUser` | The request body, as JSON |
| File | `avatar: File` | The call is sent as a form; `Backend.x.progress` follows the upload |
| Query parameter | any other parameter; `pageSize: Number in query "page-size"` | The query string under its own name, or the one written; `null` is left out |
| Header parameter | `token: String in header "X-Token"` | Sent as that header |
| Cookie parameter | `theme: String? in cookie`, `sid: String in cookie "session_id"` | Set as that cookie |
| Default | `page: Number = 1` | Sent when the call leaves it out |
| Optional | `q: String?` | May be left out; a parameter with neither must be given (`C01`) |
| Result | `-> User!` | A `User`, or an `HttpError` |
| Result with an error enum | `-> User ! CreateUserError` | A `User`, or a case of the enum (below) |
| No value | `-> !` · `-> ! E` | Nothing (a `204`), but it may fail |
| `idempotent` | after the result | Safe to send twice: one `Idempotency-Key` on every try; retried, and kept offline, as a `PUT` is |
| `invalidates [list, byId(id)]` | on a write | Reads of the same group a success makes stale; arguments are the write's parameters (`N05`) |
| `pages .number(param: page)` | on a read | Next page: the parameter, one more; ends at an empty page |
| `pages .cursor(param: after, next: p => p.next, items: p => p.items)` | on a read | Next page: the cursor `next:` gives; ends at `null` |
| `pages .link` | on a read | Next page: the `Link` header's `rel="next"` (RFC 8288) |
| `streamed` | on a `get` returning a list | One JSON value a line, each held to the item type |
| `as: .text` · `.blob` · `.arrayBuffer` · `.none` | after the result | How a success's body is read; JSON unset |
| `errorAs:` | as `as:` | How a failure's body is read |
| `cache:` | `.maxAge(d)` · `.swr(d)` · `.forever` · `.none` | This endpoint's cache policy |

The cases of an error enum, what an endpoint's `-> T ! E` names (`N12`).

| Case | Written | Holds |
|---|---|---|
| A status | `conflict(problem: Problem) on 409` | That status's body, held to its type |
| A class | `clientError(problem: Problem?) on 4XX` | Every status of the class no other case names |
| The rest | `http(error: HttpError)` | Required, once: every other failure |

## Call options

Named arguments a call takes beside the endpoint's parameters; `http.get(url, …)` takes the same.

| Option | Values | Where |
|---|---|---|
| `body:` | What is sent, as JSON (a `File`, `FormData`, `Blob`, `ArrayBuffer`, `URLSearchParams` as it is) | `http.post`/`put`/`patch` |
| `headers:` | A map: `{ "X-Api-Key": key }` | any call |
| `timeout:` | A duration | any call |
| `retry:` | As the service setting (`N09` on a write that is not `idempotent`) | any call |
| `cache:` | `.maxAge(d)`, `.swr(d)`, `.forever`, `.none` | any call |
| `as:` | `.text`, `.blob`, `.arrayBuffer` | any call |
| `refresh:` | `.onReconnect` (unset), `.onFocus`, `.every(30.seconds)`, `.never`, or a list | a `resource` |
| `placeholder:` | `.previous`: the last value stays shown, refreshing, while another request loads | a `resource` |

## A read's cases and members

What `resource name = …` is. A `match` over it takes each case.

| Case | When |
|---|---|
| `.loading` | Nothing to show: a first load, or a request whose address changed |
| `.failed(e)` | The first load failed; `e` is an `HttpError`, or the endpoint's error enum |
| `.loaded(value, refreshing, stale)` | Something to show; `refreshing` while it is asked again; `stale` a refresh's failure, else `null` |

| Member | What it is |
|---|---|
| `name.value` | The value while `.loaded`, else `null` |
| `name.isLoading` · `isLoaded` · `isFailed` | Which case it is in |
| `name.isRefreshing` | `.loaded` and asking again |
| `name.error` | The first load's failure, or the last refresh's, else `null` |
| `name.pending` | Any request of it running, the first load included |
| `name.reload()` | Asks again: a refresh when a value is shown |
| `name.invalidate()` | Drops what the cache holds for it, and asks again |
| `name.cancel()` | Aborts the request that is running |
| `name.items` | A paged read: every item loaded so far, in order |
| `name.hasMore` | A paged read: whether there is a next page |
| `name.loadMore()` | A paged read: the next page, as a refresh; nothing while a request runs |

## A call's members

What an endpoint and an action carry, beside being called.

| Member | Of | What it is |
|---|---|---|
| `Backend.users.invalidate()` | an endpoint | Drops its cache; every read of it on the page asks again. With arguments, one call's |
| `Backend.users.update(change)` | a read | Changes what every read of it shows, at once; returns what takes it back (`N02` on a write) |
| `Backend.users.prefetch(args)` | an endpoint | Asks now, into the cache |
| `Backend.users.url(args)` | an endpoint | The address a call would ask |
| `Backend.users.key(args)` | an endpoint | The key its cache holds a call's answer under |
| `Backend.avatar.progress` | an endpoint with a `File` | The upload's progress, 0 to 1 |
| `save.pending` | an action | `true` while a call of it runs |
| `save.result` | an action that may fail | Its last outcome, `.ok(v)` or `.err(e)`; `null` before it has run |

## HttpError

What goes wrong between the page and the server. Every case has `.message`.

| Case | When | Its `.message` |
|---|---|---|
| `.offline(queued)` | The request reached no server: no network, a server down, a CORS refusal; `queued` is `true` when a write was kept to send later | the browser's words, or says so |
| `.timeout(limit)` | No answer within `limit`, a `Duration` | says so |
| `.cancelled` | The scope left, the request changed, `.cancel()`, or an empty path parameter | says so |
| `.transport(message)` | Anything else that stopped it: a middleware that threw | `message` |
| `.decode(at, message)` | The answer is not the declared type, or not JSON | names the address and the place, `$[1].name is missing` |
| `.status(code, problem)` | An error status; `problem` is the body when it is `application/problem+json`, else `null` | the body's `detail`, else its `title`, else `HTTP 503` |

`Problem`, RFC 9457's account of a failure; a project extends it like any record, `type ValidationProblem = Problem { errors: [FieldError] }`.

| Field | Type |
|---|---|
| `type` | `Url?` |
| `title` | `String?` |
| `status` | `Number?` |
| `detail` | `String?` |
| `instance` | `Url?` |
| `extensions` | `Json` |

## Caching

A `GET` is kept as a private HTTP cache keeps it (RFC 9111), unless a `cache:` policy overrides the server.

| Server says | What is kept |
|---|---|
| No caching headers | Nothing |
| `Cache-Control: max-age=n` | Fresh for `n` seconds, less its `Age` |
| `Expires` | Fresh until then, when there is no `max-age` |
| `stale-while-revalidate=n` | Served stale for `n` more seconds, asked again behind it |
| `no-cache` | Kept, and asked again before every use |
| `no-store` | Nothing |
| `ETag` · `Last-Modified` | Asked again with `If-None-Match` · `If-Modified-Since`; a `304` serves what is held |
| `Vary` | Kept apart by those request headers; `Vary: *` never reused |
| — | Kept apart by the credentials a request carries; at most 256 answers, the one read least recently dropped first |

| Policy | Means |
|---|---|
| `.maxAge(d)` | Fresh for `d`, whatever the server says |
| `.swr(d)` | Served at once; once older than `d`, asked again behind it |
| `.forever` | The first answer, for good |
| `.none` | Nothing kept; every call asks |

Two readers of one address make one request, cached or not; each keeps its own abort.

## Retries

What `retry:` does.

| Rule | |
|---|---|
| Methods retried | `GET`, `HEAD`, `OPTIONS`, `PUT`, `DELETE`; a `POST` or `PATCH` only when its endpoint is `idempotent` |
| Failures retried, unset `on:` | `.network`, `.timeout`, `.status5xx`, `.status429` |
| `on:` takes | `.network`, `.timeout`, `.status5xx`, `.status4xx`, one status (`.status503`) |
| Never retried | A cancelled call |
| Wait | `delay` (300 ms) doubled each try, up to `max` (30 s), times a jitter of 0.5–1 (`jitter: false` for none) |
| `Retry-After` | The server's wait, in seconds or as a date, wins |

## Middleware

A step is handed the request and the rest of the chain, `next`, and returns a result: `.ok(json)` or `.err(httpError)`.

| Step | What it does |
|---|---|
| `bearer(token)` | `Authorization: Bearer …`, read as the request passes; nothing when empty |
| `header(name, value)` | One header, read as the request passes |
| `refreshOn401(action)` | On a `401`, runs the action once for every request that heard it, then sends the request again through the steps after it; put it before `bearer` |
| `requestId` | A fresh `X-Request-Id` |
| `log` | Each request and how it ended, in the console |
| `middleware name(req, next) { … }` | A step of the project's own; one that throws fails the call `.transport` |

| `Request` field | Type |
|---|---|
| `url` | `String` |
| `method` | `String` |
| `headers` | `[String: String]` |
| `body` | `Json?` |

## Fakes

`fake Name for Backend.group { endpoint(params) { .ok(…) } }`, as `N06` holds it.

| Rule | |
|---|---|
| Endpoints | Each one it answers is one the group has, answered once |
| Parameters | Each name it binds is one of that endpoint's; one the call left out is its default |
| Answers | `.ok(v)` of the endpoint's result, or `.err(e)` of its error: an `HttpError` case or a case of its error enum |
| `from examples` | Every endpoint it does not answer is answered with the spec's example of a success |
| A test's `fakes: [A, B]` | Each names a fake; no two answer one endpoint |
| A call no fake answers | Fails the test, naming the endpoint |
| A write's `invalidates` | Asks its reads again, as it does live |
| Pages · `streamed` | A paged endpoint is answered a page at a time; a streamed one with the whole list |
| `expect called Backend.group.endpoint` | Names an endpoint the group has |
| Builds | Leave fakes out; `wf verify --fakes FILE` builds a copy with them |

## The codes

| Code | What it means |
|---|---|
| [N01](../errors/N01.md) | A path's `{name}` that no parameter fills, or a parameter in the path and in a header or cookie |
| [N02](../errors/N02.md) | A `resource` over a write, or `.update` on one |
| [N03](../errors/N03.md) | `try` in an action that declares no error, or one that cannot hold it |
| [N04](../errors/N04.md) | A call's result awaited on its own, so a failure is never heard |
| [N05](../errors/N05.md) | An `invalidates` naming no read of the group, or an argument that is not the write's |
| [N06](../errors/N06.md) | A fake that does not match its group's contract |
| [N07](../errors/N07.md) | A service's `generated/` files behind its spec |
| [N08](../errors/N08.md) | A 6.x form: `fetch(…)`, `-> T`, `errors { }`, `loading`/`error`/`ready` arms, `headers { }`, `paginate:` |
| [N09](../errors/N09.md) | `retry:` on a `POST` or `PATCH` that is not `idempotent` |
| [N11](../errors/N11.md) | `Any` or `Map` in a contract, or an `http.get` with no type for its answer |
| [N12](../errors/N12.md) | An error enum without one `http(error: HttpError)` case, or a case with no status |
| [N14](../errors/N14.md) | A `pages` or `streamed` an endpoint cannot have, or `for await` over a call that is not streamed |
| [T17](../errors/T17.md) | An async result used before it is awaited, or `await` where it cannot run |
| [T21](../errors/T21.md) | A `match` over a read with no `.failed` arm and no `else` |
