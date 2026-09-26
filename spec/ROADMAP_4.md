# WebFluent 4 — the plan

> Approved in outline 2026-09-24; the decisions are recorded at the end.
> Nothing here is implemented yet.
> Author: Monzer Omer · Date: 2026-09-24
>
> Every `wf-next` block is **proposed** syntax: it does not compile today, which
> is why those blocks are not fenced as `wf` — the documentation test holds
> those to the grammar that exists.

Thirteen things were raised. This plan answers each one with a design, the
evidence that it is a real problem today, the files it touches, and what it
breaks. At the end: the order to do them in, and the decisions already taken.

**This is WebFluent 4.** There are no users to protect, so nothing is kept for
compatibility's sake: where the old shape is wrong it goes, and `wf migrate`
carries the existing projects across. Legacy paths that survived the 3.0 cut —
the `fetch … from` block, `Route` children, the retired modifier words — are
deleted rather than maintained.

The rule I applied throughout: **a gap the compiler can close belongs in the
compiler, not in a cookbook recipe.** Most of what follows is the language
learning something it currently leaves to the author.

---

## What I measured first

Everything below was run against the tree as it stands, not recalled.

| Measurement | Result |
|---|---|
| A page with one heading and one paragraph | `app.js` **63,617 B** (18,310 B gzipped); the page's own code is **111 B** of it |
| What that page ships without using | the carousel, markdown, the syntax highlighter, 32 icons, i18n, fetch, `resource`, toast, dialog, menu, drawer, tabs, form fields, `persist`, key handling, `Intl` formatting, animations, keyed lists |
| Its stylesheet | **11,128 B**, 79 `.wf-*` selectors, 2,164 B of design tokens — for a heading and a paragraph |
| Building the documentation site | **12** `flex-direction: row` overrides and **5** `!important`s written for no reason but to undo the engine's own responsive rules |
| Interop escape hatches | none: no `import`, no custom elements, no module surface; `window.X` and a `head { script }` are the whole story |
| Types the checker knows | String, Number, Bool, Null, Token, Regex, List, Map, Shape, Record, Enum, Case, Optional, Func, Resource, Store — **no Date, Time, Duration, Money, File, Url, Email, Color** |
| Networking | `resource` and `request` are 30 lines between them: JSON only, no abort, no cache, no retry, no timeout, no status, no body on an error, `credentials`/`mode`/`signal` silently dropped |
| Multi-line input | there is no `Textarea`. There is no `Audio` |

Three defects found while reading, each reproduced:

1. **A store that reads a store declared later is constructed first** and its
   `derived` reads `undefined`. Declaration order silently decides correctness.
2. **`Button("x", onclick: value)`** compiles to an `onclick` attribute — an
   inline handler written from data. Any `on*` name does. CSP is off by default.
3. **`href:`, `src:`, `to:` and `navigate()` take any string**, `javascript:`
   included. No scheme is ever checked, in any backend.

---

## The shape of the plan

Four of the thirteen are foundations the others stand on:

```
     ┌──────────────────────────────────────────────┐
     │  0. Runtime modules   1. Strings             │  foundations
     └───────────────┬──────────────────────────────┘
                     │
     ┌───────────────▼──────────────────────────────┐
     │  2. The missing types (dates, money, url, …) │  unlocks 3,4,5,9
     └───────────────┬──────────────────────────────┘
                     │
   ┌─────────────────┼─────────────────┬─────────────────┐
   ▼                 ▼                 ▼                 ▼
 3. Networking   4. Validation     5. Media          9. Security
   │  (the whole      │                 │                 │
   │   browser stack) │                 │                 │
   └─────────────────┴────────┬────────┴─────────────────┘
                              ▼
        6. Responsive    7. Motion    8. Stores    10. Interop
                              │
                              ▼  4.1
                 11. Offline      12. Peer-to-peer
```

Runtime modularisation goes **first**, not last: every phase after it adds
runtime code, and doing it afterwards means paying to un-bloat twice. Networking
grew, by decision, into the largest phase in the plan — the browser's whole
network stack, typed — with offline and peer-to-peer held back to 4.1.

---

## 0. The build ships what the program uses  ·  (item 13)  ·  XL

### Today

`src/runtime/runtime.js` is one 2,555-line IIFE assigned to `const WF`, embedded
whole with `include_str!` and emitted at the top of `app.js`. Nothing removes
any of it. The numbers are in the table above: 18.3 KB gzipped for a page that
uses `el` and `text`.

The compiler already knows exactly which features a program reaches — it decides
which emitters to run. It simply never tells the bundler.

### Proposal

Split the runtime into **feature modules with a declared dependency graph**, and
have codegen assemble only the reachable set.

```
src/runtime/
├── core.js         signal, effect, computed, el, text, scoped, onCleanup   (always)
├── control.js      when, each, show, match
├── router.js       router, navigate, params, activeLink, page, loadPage
├── store.js        store, persist
├── net.js          resource, request, api client                          (phase 3)
├── forms.js        field, form, validation                                (phase 4)
├── motion.js       animate, FLIP, view transitions                        (phase 7)
├── widgets/        dialog, menu, drawer, tabs, toast, carousel, tooltip
├── text/           markdown, highlight, format, ago
├── i18n.js         locales, t
└── icons/<name>.js one file per icon
```

- A new `src/runtime/manifest.rs` names each module, its exports and its
  dependencies; a test asserts every `WF.*` the codegen can emit is exported by
  exactly one module, and that no module references a symbol outside its
  dependencies.
- `JsCodegen` gains a `used: HashSet<Feature>` populated as it emits (it already
  branches on every one of these), and the bundle is the closure of that set,
  concatenated in dependency order.
- Icons: only the icons the program names (the registry already knows them).
- Tokens: `:root` carries only the tokens the program's CSS references.
- **`wf build --stats`**: every module with its raw and gzipped size, what pulled
  it in, the per-page chunks, the CSS by section, and the ten largest things in
  the bundle. The number is reported, never enforced — a build does not fail on
  size, it tells you where the size went.
- `build.budget` is advisory: exceed it and the build prints a warning with the
  diff against the last recorded figure. Sizes go in the release notes so a
  regression is visible in review.

**Target** (aimed at, not enforced): the heading-and-paragraph page under **4 KB
gzipped**, down from 18.3 KB, and the documentation site at half its current
JavaScript. Optimisation is the standing instruction: if a byte can be left out,
leave it out.

### Touches
`src/runtime/*` (split), `src/runtime/manifest.rs` (new), `src/codegen/js.rs`
(feature tracking), `src/cli/build.rs`, `src/themes/prune.rs` (token pruning),
`tests/js/*` (each module loadable alone), and a new `tests/bundle_size.rs` that
records the sizes of a minimal project and prints a comparison — reporting, not
gating.

### Breaks
Nothing in the language. `window.WF` keeps the same surface, but a page that
reached a function the program never used will now find it missing — documented,
and `build.runtime: "full"` restores the old behaviour for anyone hand-writing
scripts against it.

---

## 1. Strings that do not fight the author  ·  (item 1)  ·  S

### Today

- Escapes: `\n \t \r \\ \" \{ \}` — and `\{` is carried through the compiler as
  **U+FFFE/U+FFFF sentinels**, unescaped again in the template engine. A private
  hack that leaks between layers.
- No raw string. Every code sample in the documentation escapes every brace:
  `"page Home(path: \"/\") \{\n    state count = 0\n\}"`. The site's generator
  exists partly to do that escaping.
- A multi-line literal works, but keeps the source's indentation verbatim.
- No formatting in interpolation: `"{format(total, .currency)}"` every time.

### Proposal

```wf-next
// Raw: no escapes, no interpolation. Any number of #.
const SAMPLE = #"page Home(path: "/") { state n = 0 }"#

// Block: common indentation stripped, interpolation on.
const NOTE = """
    Dear {name},
      thank you.
    """                      // → "Dear Ada,\n  thank you."

// Formatted interpolation, sugar for format(value, .style, option)
Text("Total: {total:.currency}")
Text("Shipped {when:.date(long)} — {count:.integer} items")
```

- The sentinels go: `parse_interpolated` returns `StringPart::Literal` with the
  real brace, and the template engine's `interpolate_string` loses its
  `\u{FFFE}` pass.
- `text.dedent()`, `.lines()`, `.words()` join the string helpers; the method
  table in `sema::types` and the registry gains what is missing (`trimStart`,
  `trimEnd`, `at`, `normalize`, `localeCompare`).

### Touches
`src/lexer/v2.rs` (raw and block forms), `src/parser/v2.rs` (`parse_interpolated`,
format specs), `src/template/mod.rs` (drop the sentinel pass),
`src/sema/types.rs`, `src/codegen/highlight.rs` (colour the new forms), the
tree-sitter grammars, `scripts/site-from-guide.py` (stop escaping).

### Breaks
Nothing. `\{` keeps working.

---

## 2. The types the language is missing  ·  (items 4, 6)  ·  L

### Today

A date is a `String` that `format` re-parses on every render. `DatePicker(bind:)`
binds a `String`. There is no way to add a day, compare two dates, hold money
without float drift, or say "this is a URL" — and because nothing says that,
nothing can check it (see §9).

### Proposal

Eight types, each with a literal, checker support, methods, and `format`
integration. Each is a **plain JSON value at run time** — no wrapper objects, so
it crosses `fetch`, `persist`, the static paint and the template engine
unchanged — and each has an escape to the platform's own object, so the language
is never the thing standing in your way:

```wf-next
state due: Date = @2026-03-14

due.plus(days: 3)              // the language's own arithmetic
due.native                     // a real `Date`, to hand to any JS library
DateTime(from: someJsDate)     // and back again
Chart.setRange(from.native, to.native)
```

A value arriving from `external` JavaScript as a platform `Date` is accepted
wherever a `Date` is wanted and normalised at the boundary. The carrier is
boring on purpose; the escape hatch is one word.

| Type | Run time | Literal | Methods |
|---|---|---|---|
| `Date` | `"2026-03-14"` | `@2026-03-14` | `.year .month .day .weekday`, `.plus(days:months:)`, `.minus(…)`, `.isBefore(d)`, `.startOfWeek`, `.format(.date)` |
| `Time` | `"09:30"` | `@09:30` | `.hour .minute`, `.plus(minutes:)` |
| `DateTime` | ISO 8601 | `@2026-03-14T09:30Z` | the above, `.date`, `.time`, `.inZone("Europe/Berlin")` |
| `Duration` | milliseconds | `3.days`, `90.minutes`, `250.ms` | `.days .hours`, arithmetic with the three above |
| `Money` | `{ amount: 1299, currency: "EUR" }` (minor units) | `€12.99`, `Money(1299, "EUR")` | `+ - *`, `.convert(rate)`, `format(m)` needs no option |
| `Url` | string | `@url("https://…")` or inferred from a literal | `.host .path .query`, `.with(query:)` |
| `Email` | string | inferred from a literal | `.domain` |
| `Color` | string | `#0F766E`, `$primary` | `.mix(other, 0.2)`, `.contrast(other)`, `.alpha(0.5)` |

Plus two the compiler needs for other phases:

- **`File`** — what `FileUpload` yields: `.name .size .type`, `.preview` (an
  object URL that is revoked on scope leave).
- **`Secret`** — a `String` that cannot be `persist`ed, logged, interpolated into
  a URL, or rendered. A compile error at each. (§9.)

Every one is **refinable**: `Number(0...100)`, `String(minLength: 8)`,
`Date(after: @2026-01-01)`. A refinement is what validation reads in §4 and what
`T01` reports on a bad assignment.

```wf-next
type Booking {
    id: Uuid
    guest: Email
    from: Date
    nights: Number(1...30)
    total: Money
}

derived checkout = booking.from.plus(days: booking.nights)
derived late = now.date.isAfter(checkout)
Text("Due {booking.total} on {checkout:.date(long)}")
```

`now` joins `viewport`/`query`/`hash`/`theme` as a browser value — a `DateTime`
signal, updated on a cadence the page asks for (`now(every: 1.minutes)`), so a
relative timestamp ticks without a hand-written timer.

### Touches
`src/sema/types.rs` (the `Type` enum, inference, refinements), `src/parser/v2.rs`
(literals), `src/lexer/v2.rs` (`@`-literals, `3.days`), `src/codegen/format.rs`
(build-time formatting for the new types), `src/runtime/text/format.js` and a new
`src/runtime/temporal.js`, `src/registry/builtins.rs` (`DatePicker`, `Input`,
`Slider` bind the right type).

### Breaks
`DatePicker(bind: s)` where `s` is a `String` becomes `T01`. A migration
rewrites `state d = "2026-01-01"` bound to a `DatePicker` into `state d: Date =
@2026-01-01`.

---

## 3. The browser's network stack, typed  ·  (item 9)  ·  XXL

### Today

`resource` and `request` are the whole of it. Verbatim: JSON only; a failed
response throws `new Error("HTTP 500")` and the body is discarded; no abort (a
superseded request runs to completion and its answer is dropped by a counter); no
timeout; no retry; no cache; no dedupe; no base URL; no default headers; and
`credentials`, `mode`, `cache` and `signal` passed in options are silently
ignored. Two resources on one URL make two requests. Leaving a page cancels
nothing. HTTP is the only protocol the language has heard of.

The decision taken is that **everything the browser can do over a network, the
language should be able to say** — and say with types: what goes out, what comes
back, and what can go wrong.

### 3a. HTTP, in full

An **`api` declaration**: one place where a service is described, so every call
site is typed, cached and cancellable.

```wf-next
api Backend(base: env.PUBLIC_API ?? "/api/v1") {
    headers { Authorization: "Bearer {Session.token}" }
    timeout: 10.seconds
    retry: .backoff(times: 3, jitter: true, on: [.network, .timeout, .status(5xx)])
    credentials: .sameOrigin          // .omit · .sameOrigin · .include
    mode: .cors                       // .cors · .sameOrigin · .noCors
    redirect: .follow                 // .follow · .error · .manual
    referrer: .sameOrigin

    on request(r)  { r.headers["X-Request-Id"] = uuid() }
    on response(r) { Metrics.record(r.status, r.duration) }
    on error(e)    { if e.status == 401 { await Session.refresh()  retry once } }

    get    users(page: Number = 1, q: String?)  -> [User]
    get    user(id: String)                     -> User
    post   createUser(body: NewUser)            -> User
        errors { 422 -> ValidationErrors, 409 -> Conflict }
    patch  updateUser(id: String, body: Map)    -> User
    delete removeUser(id: String)
    post   avatar(id: String, file: File)       -> Url        // multipart, progress
    get    export(from: Date, to: Date)         -> File       // blob download
    get    logs(since: DateTime)                -> Stream<LogLine>
}
```

Or import the whole surface from a specification:

```wf-next
api Backend from "openapi.json" (base: env.PUBLIC_API)
```

Every endpoint, request body, response type and error shape is generated from the
spec and checked by the compiler, so the day the server changes its contract the
build says so.

**What the HTTP layer covers.** The test is: anything `fetch` and `XMLHttpRequest`
can do between them, the language can say.

| | |
|---|---|
| Methods | all, including `HEAD` and `OPTIONS`, with typed path parameters and typed query parameters |
| Request bodies | JSON, text, `FormData`, multipart with files, URL-encoded, `Blob`, `ArrayBuffer`, a `ReadableStream` |
| Response kinds | JSON, text, `File`/blob, `ArrayBuffer`, streamed lines or chunks, `204` as no value |
| Headers | typed on the way out, readable on the way in (`r.headers.link` for pagination, `retry-after` honoured by the retry policy) |
| Errors | a typed union: `.offline`, `.timeout`, `.aborted`, `.parse`, `.status(code, body)` — with the **body decoded to the declared error type**, matched with `match` |
| Progress | `Backend.avatar.progress` is a `Number` signal, up and down, for uploads and large downloads |
| Abort | on scope leave, on a URL that reads state changing, on `.cancel()`, on timeout — an `AbortController` per request, owned by the scope the runtime already tracks |
| Retry | exponential backoff with jitter, per error kind, honouring `Retry-After`, capped |
| Cache | `.swr(60.seconds)`, `.none`, `.forever`, keyed by URL and body; `invalidate()`, `prefetch()`, and request **dedupe** so two readers of one URL make one request |
| Revalidation | `ETag`/`If-None-Match` and `Last-Modified`, with `304` served from cache |
| Range | resumable downloads and partial reads |
| Refetch | `on: .focus`, `.reconnect`, `.interval(30.seconds)`, `.never` |
| Pagination | `.page`, `.cursor` or `Link`-header, giving `.items`, `.loadMore()`, `.hasMore` |
| Mutations | `optimistic` with automatic rollback, then `invalidate()` |
| Build time | a `get` with statically known arguments is **resolved during `wf build`** and baked into the static paint — a static site with live types |
| Escape hatch | `window.fetch` remains, untouched and unwrapped |

```wf-next
resource users = Backend.users(page: n, cache: .swr(60.seconds), on: .focus)

match users {
    loading { Skeleton }
    error(e) {
        match e {
            .offline        { Alert("You are offline.").warning }
            .timeout        { Alert("The server took too long.").warning }
            .status(422, v) { for m in v.messages by m { Text(m).danger } }
            else            { Alert(e.message).danger }
        }
        Button("Try again") { on click { users.reload() } }
    }
    ready(list) { for u in list by u.id { Text(u.name) } }
}

action rename(id: String, name: String) {
    optimistic Backend.users where u.id == id { u.name = name }   // rolled back on failure
    await Backend.updateUser(id, { name: name })
    Backend.users.invalidate()
}
```

### 3b. WebSocket

```wf-next
socket chat = ws("wss://example.com/chat", protocols: ["v2"]) {
    send    Outgoing                      // typed both ways
    receive Incoming
    reconnect: .backoff(max: 30.seconds)
    heartbeat: 20.seconds                 // ping, and reconnect on a missed pong
    queue: .whileClosed                   // sends made while down are flushed on open
}

match chat {
    connecting { Spinner.sm }
    open       { for m in chat.messages by m.id { Bubble(m) } }
    closed(c)  { Alert("Disconnected ({c.code})").warning }
    error(e)   { Alert(e.message).danger }
}

Button("Send") { on click { chat.send(Outgoing(text: draft))  draft = "" } }
```

Binary frames, close codes, subprotocols, and disposal when the page that opened
it leaves — the ownership scope closes the socket, so a route change cannot leak
one.

### 3c. Server-sent events

```wf-next
stream ticks = sse("/events", events: ["price", "status"], resume: true)
match ticks {
    open { Text("Live") }
    else { Text("Reconnecting…") }
}
effect { if let p = ticks.last("price") { price = p.value } }
```

Named events, `Last-Event-ID` resume across reconnects, the server's `retry:`
hint honoured.

### 3d. Beacon and cross-tab

```wf-next
beacon("/analytics", { event: "checkout", value: total })   // survives unload

channel cart = broadcast("cart")        // every tab of this origin
cart.post({ items: Cart.count })
on message(m) { Cart.merge(m) }
```

`BroadcastChannel` is also what gives stores real cross-tab synchronisation
(§8), replacing the `storage`-event approach that was never written.

### 3e. The network as a value

Like `viewport` and `theme`, a live value the page can read:

```wf-next
if !network.online { Alert("You are offline — changes are queued.").warning }
Image(hero, alt: "…", quality: if network.saveData { .low } else { .high })
```

`network.online`, `.effectiveType` (`4g`, `3g`, …), `.saveData`, `.downlink`.

### 3f. Offline and peer-to-peer  ·  **4.1**

Both are self-contained, both are large, neither blocks anything else, so they
follow the rest:

- **Service worker and offline.** `wf build` emits one: precache the shell and
  the routes named in the config, runtime caching policies per route, an offline
  fallback page, **background sync** that queues mutations made offline and
  replays them on reconnect, and a versioned update flow with a prompt. This
  changes the deployment story (cache versioning, update prompts, the
  double-refresh problem), which is exactly why it is its own release.
- **WebRTC data channels.** `peer link = rtc(signal: Backend.signal)` with typed
  messages, the same `match` shape as a socket, ICE configuration, and signalling
  left to the author's own transport — the language supplies the channel, not a
  server.

#### Offline, as built (4.1)

Opted into in the config; nothing changes for a project that does not name it.

```json
{ "offline": {
    "precache": ["/", "/docs/*"],
    "fallback": "/offline",
    "cache": { "/api/*": "network-first", "/media/*": "cache-first" },
    "sync": true
} }
```

- **`sw.js`**, written by `wf build` last, versioned by a hash of everything
  else it wrote — so every deploy that changes a byte is a new version, and
  one that changes nothing is not. It stores the shell (`index.html`,
  `app.js`, `styles.css`) and each route `precache` names (a glob per
  route; default `/`) with the page's own chunk and sheet, and the
  `fallback` page. A navigation goes to the network first and falls back to
  what is stored, then to `fallback`; a stored asset is served from the
  store; a path `cache` names follows its policy — `network-first`,
  `cache-first`, `stale-while-revalidate` or `network-only`; everything
  else passes through untouched. Activating a version deletes the stores of
  the versions before it.
- **The update flow.** A new version installs and waits; the page reads
  `update.available` and calls `update.apply()`, which activates it and
  reloads **once** — not on the first install, not twice on one update: the
  double-refresh problem is the page reloading on every `controllerchange`.
- **`sync: true`.** A write — any method but `GET` and `HEAD`, through the
  request engine — that fails because the network is gone is queued in
  IndexedDB rather than thrown, and the call resolves with nothing:
  optimistic changes stay shown. `network.queued` is how many wait. They are
  sent in order when the connection returns — by the worker's Background
  Sync where the browser has it, so a closed page still sends them, and by
  the page otherwise — and a write the server refuses is dropped, not
  retried forever. A body that cannot be stored (a `File`) is not queued.
- **Under `wf serve`** the worker unregisters itself: a page kept from the
  network by its own cache would hide every edit.


### Touches
New `ast::ApiDecl`, `SocketDecl`, `StreamDecl`, `ChannelDecl` and parser support;
`src/sema/types.rs` (endpoint signatures, the error union, `Stream<T>`);
new `src/codegen/api.rs`; new `src/openapi/` (spec → endpoints, behind a crate);
new runtime modules `net/http.js`, `net/cache.js`, `net/socket.js`, `net/sse.js`,
`net/channel.js`, and in 4.1 `net/sw.js` and `net/rtc.js`; `src/codegen/ssg.rs`
(build-time resolution of static `get`s); `src/cli/build.rs` (service worker
emission).

### Breaks
`await fetch(…)` keeps working. `resource(url)` keeps working and gains abort,
timeout and typed errors. The `fetch … from` block of the original grammar is
deleted.

---

## 4. Validation the compiler knows about  ·  (item 2)  ·  L

### Today

`form.valid` is the browser's own constraint validation; `error:` is a string the
author computes. Every rule is a hand-written `derived` and every message a
hand-written string — the cookbook's sign-up form is eleven lines of `derived`
before it renders anything. No cross-field rule, no async rule, no "show it after
they leave the field", no translation, and `Textarea` does not exist.

### Proposal

A **`validate` block**, next to the state it guards, reusing the accessible error
plumbing that already exists (`role="alert"`, `aria-invalid`, `aria-describedby`).

```wf-next
page Signup(path: "/join") {
    state email = ""
    state password = ""
    state confirm = ""

    validate email {
        required                                  // messages default, and translate
        email
        async "That address is taken" { !(await Backend.emailTaken(email)) }
    }
    validate password {
        required
        minLength(8) "Use at least 8 characters"
        pattern(/[0-9]/) "Include a number"
    }
    validate confirm {
        matches(password) "The two passwords differ"
    }

    Form(bind: form, show: .onBlur) {          // .onBlur (default) · .onSubmit · .live
        on submit { Backend.signUp(email, password) }
        Input(bind: email, label: "Email")     // error, aria-invalid, focus order: automatic
        Input(bind: password, label: "Password").password
        Input(bind: confirm, label: "Confirm").password
        Button("Join", type: .submit, disabled: !form.valid || form.pending).primary
    }
}
```

- Rules: `required`, `email`, `url`, `minLength`, `maxLength`, `min`, `max`,
  `pattern`, `matches`, `oneOf`, `custom { expr }`, `async { … }`. Each takes an
  optional message; messages go through `t()` when the project has i18n.
- A **refined type validates itself**: `state age: Number(18...120)` needs no
  `validate` block for the range, and a `type` used as a form's shape
  (`Form(of: Signup)`) validates every field from the record's declaration.
- `form.errors` is a map, `form.touched` a set, `form.submit()`/`reset()` as now.
  On a failed submit, focus moves to the first invalid control (WCAG 3.3.1).
- Server errors merge in: `form.apply(e.body.errors)` after a 422.
- **`Textarea(bind:, rows:, maxLength:)`** joins the registry, with a character
  counter when `maxLength` is set.

### Touches
`ast::ValidateDecl` + parser; `src/sema/` (rules checked against the bound
type — `minLength` on a `Number` is an error); `src/codegen/js.rs`; new
`src/runtime/forms.js`; `src/registry/builtins.rs` (`Textarea`, `Form(show:)`);
`src/linter/accessibility.rs` (a validated field without a label is a warning).

### Breaks
Nothing; `error:` stays for hand-rolled cases.

---

## 5. Media that the build actually processes  ·  (items 6, 7)  ·  L

### Today

`public/` is copied byte for byte. There is no image pipeline of any kind: no
resize, no format conversion, no `srcset`, no placeholder, no aspect-ratio box,
no crop. `Image` takes `src alt width height loading shape`. There is no `Audio`.
`Video` has no `poster`, `muted`, `loop` or captions. The PDF backend draws a
grey rectangle where an image should be.

### Proposal

**A build-time asset pipeline**, with the declaration form `data` already
established:

```wf-next
image hero = "media/hero.jpg"        // read at build time: dimensions, colour, hash

page Home(path: "/") {
    Image(hero, alt: "The team at the launch",
          sizes: "(max-width: 768px) 100vw, 1200px",
          crop: .focal(0.5, 0.25),
          placeholder: .blur)
}
```

The build emits `hero.a1b2c3.avif/webp/jpg` at the widths `sizes` implies, writes
a `<picture>` with `srcset`, sets `width`/`height` from the real file (so layout
never shifts), and inlines a 20-byte blurred placeholder as a data URI. A
content hash in the name makes the file immutable-cacheable.

- `Video(src:, poster:, muted:, loop:, playsinline:, captions: "en.vtt")` —
  captions become a `<track>`, and a `Video` without one is an `A09` warning.
- **`Audio(src:, transcript:)`**.
- `Image` accepts a `Url` for remote images and leaves them alone (no pipeline),
  but still sets the aspect ratio when given `width`/`height`.
- PDF and slides embed real images (the placeholder rectangle goes).
- `media.formats`, `media.widths`, `media.quality` in the config; `media.pipeline:
  false` turns the whole thing off and copies as today.

This phase takes real dependencies: `image` for decode and encode, plus an AVIF
encoder. That is settled — the standing rule is now **use a crate where it earns
its place**, while the hand-written PDF writer and gzip encoder stay as they are.
Decoding and re-encoding four image formats is not work worth doing by hand, and
the same judgement lets the OpenAPI parser (§3) and a proper CSS parser for the
style pipeline come from crates too. Each addition is named in the release notes
with what it replaces.

### Touches
New `src/media/` (probe, resize, encode, cache in `.wf-cache/`), `src/data.rs`
(the `image` declaration), `src/registry/builtins.rs`, `src/codegen/ssg.rs` and
`js.rs` (`<picture>`), `src/codegen/pdf.rs` and `slides.rs` (embedding),
`Cargo.toml`.

### Breaks
Nothing; existing `Image(src: "/x.png")` keeps working untouched.

---

## 6. Responsive without JavaScript  ·  (item 12)  ·  M

### Today

Two mechanisms, both wrong in a different way:

1. **`viewport.md`** is a `resize` listener in the runtime. The static paint
   cannot know the width, so `if viewport.md { … }` paints one branch and swaps
   after hydration — a visible jump, and a re-render on every resize event
   (unthrottled).
2. **The engine's own media rules**, ten of which are `!important`: below 768px
   every `Row` becomes a column and every `Grid` one column, whatever the author
   wrote. Building the documentation site took **12 `flex-direction: row`
   overrides and 5 `!important`s** to undo them.

### Proposal

**Responsive values on any prop**, compiled to classes and media queries, with no
JavaScript and no layout shift:

```wf-next
Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg }) { … }
Row(direction: { base: .column, md: .row }) { … }
Text("Title") { style { font-size: { base: 24px, lg: 34px } } }
```

- Breakpoint names are **tokens** (`screen-sm/md/lg/xl`), so a theme can move
  them; the runtime's hard-coded 640/768/1024/1280 goes.
- `@md { … }` in a style block is sugar for the token's media query; `@container
  (min-width: 30rem) { … }` becomes first-class, and `Container(query: .inline)`
  declares the containment.
- **The `!important` reflow rules are deleted.** A `Row` stays a row unless asked:
  `Row(direction: { base: .column, md: .row })`, or the shorthand `.stacks`.
  That is a breaking change and the reason this is 4.0 — but it is the difference
  between the engine helping and the engine fighting.
- `viewport` remains for genuinely behavioural cases (a drawer instead of a
  sidebar) and gains an SSR-honest story: both branches paint, CSS hides one,
  JavaScript takes over on hydration — so no jump.
- Fluid type tokens (`clamp()`) for the display sizes.

### Touches
`src/parser/v2.rs` (a map value where a prop is expected), `src/sema/mod.rs`
(responsive values checked per breakpoint), `src/codegen/scoped_css.rs` (class
generation), `src/themes/components.rs` + `structural.rs` (delete the
`!important` block), `src/runtime/core.js` (viewport as a media-query listener,
not a resize listener).

### Breaks
Layouts that relied on the automatic phone reflow. `wf migrate` adds `.stacks`
to every `Row` and `Grid` in a project so the old behaviour is explicit.

---

## 7. Motion with a real timing model  ·  (item 11)  ·  M

### Today

Twelve CSS keyframe classes, added and removed around `animationend` with a
`setTimeout` fallback. A FLIP transform for keyed list moves. View Transitions
for routes. That is all: no spring, no interruption (an element fading out that
comes back has no handling), no scroll-driven animation, no shared element, no
sequencing, no animating to `height: auto`, no animating a *number*.

### Proposal

Move the engine to the **Web Animations API**, which gives interruption and
composition for free, and add what the declarative layer is missing:

```wf-next
Card(animate: .slideUp, easing: .spring(stiffness: 180, damping: 20)) { … }

// Enters when it scrolls into view, once
Section(animate: .fadeIn, on: .enterView) { … }

// The same element across a route change
Image(post.cover, alt: post.title, shared: "cover-{post.id}")

// Height to auto, which CSS cannot do
show details { Stack(animate: .expand) { … } }

// A number that counts
Text("{revenue:.currency}") { animate value: 600ms }

sequence {                         // orchestration
    step { Heading("Welcome").h1 animate: .fadeIn }
    step(after: 120ms) { Text("…") animate: .slideUp }
}
```

- `$ease-spring` and friends become theme tokens; `motion` in the config sets
  global defaults (`motion: { duration: 180ms, easing: $ease-standard }`).
- Reduced motion keeps its guarantee: everything above collapses to an instant
  state change, by construction.
- The escape hatch `replayAnimation(node, name)` is replaced by a typed
  `animation.play(node)`/`.cancel()` handle.

### Touches
New `src/runtime/motion.js` (WAAPI, FLIP, spring solver, IntersectionObserver
binding, View Transitions for shared elements), `src/registry/builtins.rs`
(universal props gain `on:`, `shared:`), `src/parser/v2.rs` (`sequence`),
`src/themes/tokens.rs` (easing tokens).

### Breaks
Custom `animation Name { }` keyframes keep working. The class names
(`wf-animate-fadeIn`) are no longer applied, so a stylesheet targeting them
breaks — rare, and named in the migration notes.

---

## 8. Stores: lifetime, location, and a way to see them  ·  (item 5)  ·  M

### Today

Every store is constructed eagerly at module scope, in declaration order, whether
or not anything uses it. **A store whose `derived` reads a store declared later
reads `undefined`** — reproduced above. `use Store` is a scoping hint only; the
store is a global either way. `persist` is `localStorage` + `JSON`, with no
versioning, no migration, no expiry, no cross-tab sync, and no way to say "this
is sensitive, never write it down" — the cookbook's own session example stores a
token where any script can read it. There is no way to inspect a store at run
time, and nothing documents where stores live or when they are created.

### Proposal

```wf-next
store Cart(scope: .app) {                 // .app (default) · .session · .route
    persist items: [Item] = [] (in: .local, version: 2) {
        migrate 1 -> 2 { old.map(i => Item(id: i.id, qty: i.count)) }
    }
    state note: Secret = ""               // never persisted, never logged
    derived total = items.sum(i => i.price)
}
```

- **Lazy construction**: a store is built on first read. Dependency order stops
  mattering, and an unused store costs nothing (it also drops out of the bundle,
  §0).
- **Scopes**: `.route` resets when the route leaves — the natural home for "this
  page's filters"; `.session` is per tab.
- **Persistence policy**: storage choice, a version with a `migrate` block,
  cross-tab sync through the `storage` event, and a compile error on persisting a
  `Secret`.
- **Devtools** in `wf serve`: an action log with before/after state, a snapshot
  tree, and time travel, in the existing dev overlay.
- **Documentation**: a rewritten chapter 9 saying plainly when a store is built,
  where it may be declared, how testing one works (`wf test` with a seeded
  store), and when a page's `state` is the better answer.

### Touches
`src/codegen/js.rs` (`emit_store`, lazy accessor, dependency-order fix),
`src/runtime/store.js`, `src/sema/mod.rs` (scope and `Secret` checks),
`src/cli/serve.rs` (devtools panel), `md-docs/09-stores.md`.

### Breaks
A store with a side effect in a `derived` that used to run at boot now runs on
first read. Lazy is the right default; `store X(eager: true)` opts out.

---

## 9. Security, as a subject the language takes seriously  ·  (item 8)  ·  M

### Today

The good: text goes in as `textContent`, Markdown escapes HTML, the output
satisfies `script-src 'self'` with no inline handlers, the CSP builder widens
itself for declared fonts, and every built page I inspected has zero inline
styles.

The holes, each reproduced:

1. `Button("x", onclick: expr)` — **any `on*` attribute is writable from data**.
2. `href:`, `src:`, `to:` and `navigate()` **accept `javascript:`**, in every
   backend, unchecked.
3. `persist token = "…"` is the documented way to keep a session — in
   `localStorage`, readable by any script that gets in.
4. CSP is **off by default**, and `frame-ancestors` in a `<meta>` tag is ignored
   by browsers, so the policy silently under-delivers there.
5. No SRI on external fonts or stylesheets; no `rel="noopener"`; no way to render
   trusted HTML safely (no sanitiser) and no way to render it unsafely either.

### Proposal

- **Typed URLs close hole 2**: `href:`, `src:`, `to:` take a `Url`; a literal is
  checked at build time, a runtime value is passed through a scheme allow-list
  (`http`, `https`, `mailto`, `tel`, and relative). `javascript:` and `data:` are
  refused — `data:` allowed for `Image` only, behind `Unsafe.dataUrl(…)`.
- **`on*` as an attribute becomes a compile error** with a hint pointing at
  `on click { }`.
- **`Secret`** (§2) cannot be persisted, logged, put in a URL or rendered; the
  auth chapter teaches httpOnly cookies, with `persist` documented as unsuitable
  for tokens and the cookbook example rewritten.
- **CSP on by default** for `wf init`, `frame-ancestors` moved to `_headers`
  where it works, SRI hashes for declared external assets, `rel="noopener
  noreferrer"` on any link with `target`, and a **build-time verifier** that
  parses the emitted output and asserts it satisfies the emitted policy — a test,
  not a promise.
- **`Unsafe.html(string)`**: the one way to inject markup, named so it greps, and
  a warning at every use. A sanitiser (allow-list, no script, no event handlers,
  no `javascript:`) ships as `sanitize(html)`.
- **`wf audit`**: reports every `Unsafe.*`, every external origin, every
  persisted value, the CSP verdict and the dependency list.
- **A security chapter** in the guide (a new chapter 19; the reference moves to
  20): the threat model, what the compiler guarantees, what it cannot, auth
  patterns, secrets in `env` (and what `env` inlines into the bundle — a footgun
  today), uploads, CSP, headers, and a deployment checklist.
- **`env` splits in two.** `env.PUBLIC_*` (or an explicit `"public"` list in the
  config) is inlined into the bundle as today; every other name is a **compile
  error in client code** and reaches only the template engine, build-time
  expressions and the static paint. An API key in `env` stops being a silent leak
  and becomes a message naming the file and the line.

### Touches
`src/sema/mod.rs` (URL and `on*` checks), `src/runtime/core.js` (scheme guard),
`src/codegen/html.rs` + `seo.rs` (SRI, `_headers`), new `src/cli/audit.rs`,
`src/config/project.rs` (CSP default), `md-docs/` (new chapter).

### Breaks
`href: someString` needs `Url` or an explicit `Url(s)`; `onclick:` stops
compiling. Both are what you want to break.

---

## 10. Interop, in both directions  ·  (items 3, 10)  ·  L

### Today

Nothing. No `import`, no module surface, no custom-element support, no way to
hand a node to a library with a lifetime contract. The only paths are `window.X`
in an expression and a `<script src>` in `head { }`. A WebFluent component cannot
be used from React, Vue or plain HTML at all; the template engine (server-rendered
HTML or PDF) is the only embedding story.

### Proposal — three mechanisms

**(a) `external`: a typed import.**

```wf-next
external Chart from "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/+esm" {
    fn Chart(canvas: Any, config: Map) -> ChartHandle
    type ChartHandle { update(data: Map), destroy() }
}
```

Emits a real ESM `import`, checked at every call site, with the URL going into
the CSP and an SRI hash where the host supports it. A bare specifier
(`external x from "d3"`) resolves from `node_modules` when one exists, so the
npm ecosystem is reachable without adopting a bundler.

**(b) `Host`: a node with a lifetime.**

```wf-next
Host(mount: (node) => Chart.Chart(node, config),
     update: (h) => h.update(data),          // runs when `data` changes
     cleanup: (h) => h.destroy())
```

The mount runs once, `update` re-runs on the state it reads, and `cleanup` is
tied to the ownership scope the runtime already has — the pattern people write
by hand today (`ref` + `effect` + `cleanup`), made declarative, typed and
impossible to leak.

**(c) Custom elements, in and out.**

```wf-next
// Use someone's
external element Stripe("stripe-pricing-table") {
    prop publishableKey: String
    event ready()
}
Stripe(publishableKey: key) { on ready { … } }

// Publish yours
// webfluent.app.json → "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] }
```

`output_type: "elements"` emits `<price-tag>` and `<rating>` as standards-based
custom elements with the runtime inlined and attributes mapped to props — usable
from React, Vue, Svelte, Rails, WordPress or a plain page. **That is the answer
to item 10**, and it is the honest one: the shared interface between frameworks
is the platform, not an adapter per framework.

### Touches
New `ast::ExternalDecl`; `src/sema/` (external types); `src/codegen/js.rs` (ESM
emission, `Host`); new `src/codegen/elements.rs`; `src/config/project.rs`;
`src/cli/build.rs`.

### Breaks
Nothing. ESM output for a project using `external` requires
`<script type="module">`, which the build writes.

---

## Order and what each phase delivers

**WebFluent 4.0**

| # | Phase | Items | Size | Ships |
|---|---|---|---|---|
| 0 | Runtime modules and `--stats` | 13 | XL | only what the program uses; the size report |
| 1 | Strings | 1 | S | raw and block strings, formatted interpolation |
| 2 | The missing types | 4, 6 | L | dates, durations, money, url, email, colour, file, secret — each with a `.native` escape |
| 3 | Networking | 9 | XXL | `api` + OpenAPI import, typed HTTP in full, WebSocket, SSE, beacon, cross-tab, `network` |
| 4 | Validation and forms | 2 | L | `validate` blocks, refined types, `Textarea`, focus management |
| 5 | Media | 6, 7 | L | the image pipeline, `Audio`, captions, images in PDF |
| 6 | Responsive | 12 | M | responsive props, breakpoint tokens, container queries, no `!important` |
| 7 | Motion | 11 | M | WAAPI, spring, scroll-driven, shared elements, `sequence` |
| 8 | Stores | 5 | M | lazy, scoped, versioned persistence, cross-tab, devtools, the chapter |
| 9 | Security | 8 | M | typed URLs, `Unsafe`, split `env`, `wf audit`, CSP by default, the chapter |
| 10 | Interop | 3, 10 | L | `external`, `Host`, custom elements in **and out** |

**WebFluent 4.1** — self-contained, large, and blocking nothing:

| # | Phase | Items | Size | Ships |
|---|---|---|---|---|
| 11 | Offline | 9 | L | service worker, precache, runtime caching, offline page, background sync, update flow |
| 12 | Peer-to-peer | 9 | L | WebRTC data channels with typed messages |

Phase 0 goes first because every phase after it adds runtime code, and
un-bloating afterwards means paying twice. Phase 2 comes before 3, 4, 5 and 9
because each of them is smaller once the types exist: the network layer types its
payloads, validation reads refinements, media has a `File`, and security closes
the URL hole by typing it rather than by checking strings.

Every phase carries what this project already expects: tests that fail without
it, a `md-docs` chapter or section written from the same source as the site,
`wf migrate` rules for anything that breaks, and a release-note entry with the
before-and-after sizes.

---

## Decisions taken (2026-09-24)

| # | Question | Decision |
|---|---|---|
| 1 | Version and breakage | **Straight to 4.0.** No compatibility layer, no 3.5 staging. Legacy paths are deleted, not maintained; `wf migrate` carries existing projects. |
| 2 | Networking scope | **Everything the browser can do**, typed — not only REST. Full HTTP, WebSocket, SSE, beacon, cross-tab, the `network` value in 4.0; service worker/offline and WebRTC in 4.1. |
| 3 | Where API types come from | **Hand-written endpoints and OpenAPI import.** A spec generates the whole surface so a server change is a build error. |
| 4 | Dates | **Give the developer the option**: a plain ISO/number carrier that crosses every boundary, plus `.native` to the platform object and coercion back, so the language never blocks a library. |
| 5 | Validation | **Both** — `validate` blocks for form rules and messages, refined types (`Number(18...120)`) so a record validates itself. |
| 6 | Image codecs | **Take the dependencies.** |
| 7 | Dependencies generally | **Use a crate where it earns its place** — image codecs, OpenAPI/JSON-Schema, a real CSS parser. The hand-written PDF writer and gzip encoder stay. |
| 8 | Custom-element output | **Yes, in 4.0**: `build.output_type: "elements"`. |
| 9 | `env` secrets | **Split public and server.** A non-public `env` name in client code is a compile error. |
| 10 | Size budget | **Never fails a build.** Optimise as hard as possible, report with `wf build --stats`, track the figures in the release notes. |

---

## Progress log

Appended as work lands, so this file alone is enough to pick the work back up.

| Date | Phase | State |
|------|-------|-------|
| 2026-09-24 | — | Plan approved, decisions recorded. |
| 2026-09-24 | 0. Runtime modules | **Done.** `src/runtime/runtime.js` (2,556 lines) is 28 feature modules under `src/runtime/modules/`, concatenated inside one closure by `runtime::assemble`. What a build carries is read from the bundle it just wrote (`WF.<name>` + attribute triggers), closed over the manifest's dependencies; the icon table keeps only the glyphs the program names. `el`'s `markdown`/`highlight`/`data-icon` branches became registered hooks so those modules are droppable; `i18nInstance` moved to core to break the `format`↔`i18n` cycle; `__reg` became a hook so `store` no longer pins the studio's debug module. New: `build.runtime: "auto" \| "full"`, `wf build --stats`, `tests/bundle_size.rs` (reports, never gates), `src/runtime/modules/manifest.json` (written from Rust, read by `tests/js/runtime.mjs`). Also: `:root` keeps only the tokens something in the output names (25 of 67 dropped on a minimal page — 0.5 kB, measured and reported rather than assumed to matter), `build.budget` warns (never fails) when a gzipped output goes over, and `.wf-sizes.json` beside the project lets `--stats` show what moved since the last build. Measured: the heading-and-paragraph page is **3.6 kB gzipped**, against the plan's target of 4 kB and the 18.3 kB it was; the whole runtime is 107.2 kB, of which that page carries 22.4 kB, the `spa` template 24.9 kB and the documentation site 73.4 kB. |
| 2026-09-25 | 2. The missing types | **Done.** Eleven types the language brings with it — `Date`, `Time`, `DateTime`, `Duration`, `Money`, `Url`, `Email`, `Color`, `Uuid`, `File`, `Secret` — each a plain JSON value at run time, with `@2026-03-14`, `@09:30`, `3.days`, `€12.99` and `#0F766E` as literals and `.native()` as the way out to a platform object. `Type::Scalar` in the checker, resolved through `World::resolve`, so a project's own `type Color { … }` takes the name back. Arithmetic is routed by method name to polymorphic runtime helpers (`src/runtime/modules/scalars.js`), and every one of them has a build-time twin in `static_eval` — the static paint shows `2026-03-19`, not a blank that fills in on hydration; a node test pins the two to the same answers, including that a month lands on a day that exists. Also: named call arguments (`due.plus(days: 3)`), refinements (`Number(1..=30)`, `String(minLength: 8)`, `Date(after: @2026-01-01)`) checked at every literal, `format(money)` with no style or code, `now` as a browser value with a cadence, and `Secret` refused by the compiler wherever it would escape (`T12`: shown, spliced into text, logged, persisted). As the plan said it would, `DatePicker(bind:)` now wants a `Date?` — an empty picker holds nothing, and the runtime reads `""` from the input as `null` — and `wf migrate` rewrites a v2 `state when = ""` bound to one into `state when: Date? = null`, which the migration corpus pins. Deviation from the plan's table, recorded on purpose: what the language *computes* is a call (`due.year()`, `url.host()`) and only what the carrier *already has* is a field (`money.amount`, `file.size`) — so dispatch needs no type-directed codegen, and a record with its own `plus` is never taken over. |
| 2026-09-25 | 6. Responsive | **Done.** Any layout prop takes one value per breakpoint — `Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg })`, `Row(direction: { base: .column, md: .row })`, `Column(span: { … })` — compiled to one class and one media query per step in `scoped_css`, so the right layout is in the **first paint** with no JavaScript and no resize listener, and the checker holds each step to the prop's own type (`at \`md\`` in the message). The steps are the `screen-*` design tokens, resolved at build time because a media query cannot read a custom property: move `screen-md` in a theme and every query moves, `@md { … }` sugar included. **The `!important` reflow rules are deleted.** A `Row` stays a row and a `Grid` keeps its columns at every width; `.stacks` is how a layout asks to reflow, and `wf migrate` adds it to every `Row`, `Grid` and `Column` of a v2 project so the old behaviour carries over written down (32 expected-output files regenerated, and the `v2_syntax` pair updated to the idiomatic spelling). `viewport` now listens through `matchMedia` — one event when a breakpoint is crossed, where a `resize` listener fired on every pixel of a drag — while the width stays the answer, so a page and a stylesheet never disagree about what `md` means. Fluid type tokens (`clamp()` on `font-size-lg` upward) were already in place from an earlier release. |
| 2026-09-25 | 5. Media | **Done.** A build-time image pipeline, and the first dependency taken under the standing rule: `image` (decode/resize/encode), because decoding four formats and a mis-read progressive JPEG is not work worth doing by hand. The hand-written PDF writer and gzip encoder stay — and the gzip encoder gained `zlib()`, so an image embedded in a PDF and a file served precompressed go through the same deflate. `image hero = "hero.jpg"` reads the file at build time — real size, average colour, content hash — and writes it again at every width in `media.widths` smaller than the original (never up-scaled), cached in `.wf-cache/media/`. `Image(hero, alt:, sizes:, placeholder: .blur)` is a `<picture>`: a `<source>` per format with a `srcset`, `width`/`height` from the file so **nothing shifts**, and a sixteen-pixel-wide blurred copy inlined as a data URI (`.color` and `.none` too). The name is a value: `hero.width`, `.height`, `.color`, `.src`, `.srcset`. The runtime `WF.picture` and the SSG renderer are twins, so hydration repaints nothing. `media.formats/widths/quality/pipeline` in the config; `pipeline: false` copies as `public/` always did. `Video` gained `poster`, `muted`, `loop`, `playsinline` and `captions` (a `<track>`), `Audio` is new with `transcript` (a link beneath the player), and `A09` now covers both. **PDF and slides embed the real picture** — an `/XObject` per image, FlateDecode, written once however many times it is drawn; the grey `[Image]` rectangle is gone from both. **Deferred:** AVIF (the encoder is a much heavier dependency than the decoder, and WebP already covers every browser still shipped) and `crop: .focal(x, y)` (the pipeline resizes but does not yet crop). |
| 2026-09-25 | 4. Validation | **Done.** A `validate` block beside the state it guards: `required`, `email`, `url`, `minLength`, `maxLength`, `min`, `max`, `pattern`, `matches(other)` (which follows what it names), `oneOf`, `custom "…" { expr }` and `async "…" { await … }` — each with an optional message, falling back to the rule's own, which a project's translations replace by naming `form.required` and so on. Every rule but `required` passes an empty value, so a blank optional field is one message rather than two; an `async` rule is asked only once the rest pass, and once per value. The control bound to a validated state **shows the message without being told to** — no `error:` to write — through the `role="alert"` / `aria-invalid` / `aria-describedby` plumbing that already existed, and marks itself touched on blur. `Form(bind:)` now gives `valid`, `errors`, `touched`, `pending`, `values`, `reset()`, `submit()` and `apply(serverErrors)` (a 422's messages, each standing until its value changes), and `Form(show:)` says when a message appears (`.onBlur` default, `.onSubmit`, `.live`). A failed submit shows every message, moves focus to the first field that has one, announces it, and does not run the handler. A rule is checked against what it guards (`minLength` on a `Number` is `T01`; an unknown rule and an unknown state are `T05`), and **a refined type validates itself** — `state nights: Number(1..=30)` contributes its own `min`/`max`. New: `Textarea` in the registry, the lexer, the HTML map, the stylesheet and the A04 label lint, with a polite character counter when `maxLength` is set. The old `form()` and the new one are one function, so the browser's own constraint validation still answers for every control no block guards. |
| 2026-09-25 | 3. Networking (3a–3e) | **Done**, except the two noted below. One request engine (`src/runtime/modules/http.js`) under `api`, `resource` and `await fetch`: typed errors (`.offline`, `.timeout`, `.aborted`, `.parse`, `.network`, `.status(code, body)` — an array so `match` reads it as an enum, carrying `message`/`status`/`body`/`headers` so it reads as a record, **with the body decoded to the declared type** rather than discarded), timeout, abort on scope leave and on a changed argument, retry with backoff + jitter honouring `Retry-After`, a cache with SWR/forever/none keyed by method+address+body, request dedupe, `ETag`/`If-None-Match` revalidation with `304` served from what is held, upload progress through XHR, every body kind and response kind, and streamed lines. `api Backend(base:) { headers { } timeout: retry: on request/response/error … get users(page: Number) -> [User] errors { 422 -> E } }` compiles to one object of callable endpoints, each with `.invalidate()`, `.prefetch()`, `.progress`, `.url()`, `.key()`; `api B from "openapi.json"` reads the whole surface from a spec at build time, turning each named schema into a `type` the checker enforces (`src/openapi.rs`). `resource rows = Backend.users(page: n, cache: .swr(60.seconds), on: .focus, paginate: .page)` is typed by the endpoint's return, refetches on focus/reconnect/interval, and gives `.items`/`.loadMore()`/`.hasMore`. `socket`/`stream`/`channel` statements with `on message(m)`, matched by state (`connecting`/`open`/`closed(c)`/`error(e)`), all closed by the scope; `beacon(url, data)`; `network` as a live value. `optimistic(holder, change)` in an action, with the action rolling itself back on a throw (`WF.attempt`). **Deferred, with reasons:** build-time resolution of a statically known `get` — it would make `wf build` do network I/O, which is a hermetic-build decision worth taking on its own; and `Stream<T>` as an endpoint's declared return — the runtime reads streamed lines today (`.lines()`), but the type has no `match` shape yet. 3f (service worker, WebRTC) stays 4.1 as planned. |
| 2026-09-25 | 7. Motion | **Done.** The engine is the **Web Animations API** (`src/runtime/modules/motion.js`): an animation that is interrupted gives way to the one that replaced it (the engine cancels only its own, by a `wf-` id, so a page's own `el.animate` is left alone), `finished` is a promise the runtime waits on rather than a `setTimeout` guessing the duration, and a spring is sampled into the `linear()` easing the platform runs — it overshoots and settles, which no cubic Bézier can say. New in the language: `on: .enterView` (an IntersectionObserver, played once, and the **mount class is dropped** so the first paint does not play it early), `shared:` (a `view-transition-name`, carried across a route change), `.expand`/`.collapse` (measured to the content's height, which CSS cannot animate to; `show` opens the box itself, since `display: contents` has no height), `count:` on a number (`Text(format(revenue, .currency), count: "600ms")` — the first value is written, every one after it counted to, each step formatted the way the first was), and `sequence { step { … } step(after: "120ms") { … } }`, which is lowered **in the parser** to a `delay:` on each step's elements, so no new AST node, no walker arm, and what every linter and backend sees is the program you would have written by hand. `easing:` became an enum — `.standard`, `.spring`, `.ease`, `.easeIn`, `.easeOut`, `.easeInOut`, `.linear`, `.bouncy`, `.smooth` — each resolving to a design token (`$ease-standard`, `$ease-spring`), so a theme retunes every animation at once and the runtime resolves the `var()` before handing it to WAAPI. `motion: { duration, easing }` in the config writes those two tokens. The fake DOM gained the Web Animations API (`tests/js/dom.mjs`), and `tests/js/motion.test.mjs` is thirteen tests of what it bought: interruption, leaving a page's own animation alone, the sampled spring, the resolved token, the theme's duration, expanding to the measured height, the counting number, waiting to be scrolled to, the handle, and reduced motion collapsing all of it. **Kept on purpose, against the plan's "Breaks" note:** an element that stands still still plays its enter animation as the `wf-animate-*` class, because a static page should animate before its script has loaded — so nothing that targets those class names breaks, and `sequence`'s delays are compiled into stylesheet rules (one per distinct delay, shared) rather than markers only JavaScript reads. Measured: `motion` is 15.2 kB before minifying, and nothing reaches it that did not reach the old engine — a branch, a list, a `match` or a router — so the heading-and-paragraph page is still **3.6 kB gzipped** and the documentation site's `app.js` 45.1 kB. **Deferred:** scroll-*driven* animation (a timeline tied to scroll position, not a one-shot on entry) — `animation-timeline` has no fallback worth shipping yet, and `on: .enterView` covers what was asked for. |
| 2026-09-25 | 8. Stores | **Done.** A store is **built the first time something reads it**: `WF.store(name, define, options)` takes the whole definition as a thunk behind a proxy, so the order stores are declared in cannot matter — the reproduced bug, a `derived` reading a store declared below it and getting `undefined`, is gone by construction — and a store nothing reads costs nothing. `store X(eager: true)` is the opt-out for set-up that has to run at boot. **Scopes:** `store X(scope: .app | .session | .route)`; `.route` is dropped when the route changes (the router calls `dropRouteStores()` through a `typeof` check, so a site with no stores carries none of that code), and `.session` sends what it keeps to the tab's own storage. **Persistence became a policy**, and the same one at the top of a page, a component or a store: `persist items: [Item] = [] { in: .local  version: 2  sync: false  migrate 1 -> 2 { old.map(…) } }`. A version is a promise — a value an older build wrote is brought forward one step at a time, each step reading it as `old` — and a gap in the chain (`P01`) or a step past the declared version (`P02`) is a warning that says the value would be discarded; a value a *newer* build wrote is left alone, since this build cannot know what it means. Cross-tab sync through the `storage` event is **on by default** for `.local`, which is shared between tabs by definition, with `sync: false` as the way out. `persist` of a `Secret` was already `T12`. One implementation, so a page's `persist` and a store's cannot drift — and it moved **out** of `core.js` into a `keep` module, so a program that persists nothing no longer carries any of it: the heading-and-paragraph page is **3.5 kB gzipped**, down from the 3.6 kB it was before this phase. **Devtools:** a `stores` button in the `wf serve` overlay showing the tree, an action log with the state on each side of every call, and a click to put a store back — over `WF.watchStores` / `storeSnapshot` / `restoreStore`, which record nothing while nothing is watching. **Testing:** `wf test` now seeds stores — a store's initial state and its derived values are there without being asked, and `data: { Cart: { items: [...] } }` stands in for what an action would have put there, with the derived values following it; before this a `Cart.count` in a test rendered as nothing, because the template engine reads data and nobody had put the stores in it. Chapter 9 rewritten: when a store is built, where it may be declared, the scopes, the policy, the devtools, testing, and when a page's `state` is the better answer. **Deviation from the plan, on purpose:** the policy is a block rather than `persist x = v (in: .local, version: 2)`. A parenthesised group after the value is ambiguous with a call — `persist x = fallback (in: .local)` — and the `migrate` block had to exist anyway, so the two are one block. Known and accepted: the documentation site now carries the `store` module (6.5 kB) because chapter 9's prose names `WF.storeSnapshot` and friends, and the bundle scanner cannot tell a name being described from a name being called — the failure mode §0 documented, and the docs site is the one project that talks about the runtime. 14 node tests in `tests/js/store.test.mjs` (the fake DOM gained both storage areas and a `storage` event) and 8 in `tests/language_4_0.rs`. |
| 2026-09-25 | 9. Security | **Done.** All five holes closed, each with the reproduction turned into a test. **(1) `on*` as an attribute is a compile error** pointing at `on click { }` — the shape, not a list, so an attribute nobody has thought of is refused too. **(2) A URL a browser would run is refused**: `href`, `src`, `to`, `poster`, `action`, `cite` and the rest take `http`, `https`, `mailto`, `tel`, `sms`, `ftp` or a relative URL, and nothing else. A literal is refused where it is written; anything else goes through `WF.safeUrl` at the moment it is used, in every backend — the twin of `codegen::url`, with the control characters and whitespace a browser ignores stripped first, so `java\tscript:` and ` JavaScript:` are caught as well. `navigate()` is guarded at both ends. A `target:` brings `rel="noopener noreferrer"` with it. **(3) Tokens**: the auth examples in the guide and in AGENTS.md now use `credentials: .sameOrigin` and an httpOnly cookie, and chapter 19 says plainly that neither storage area is a place for a session; `persist token: Secret` was already `T12`. **(4) CSP is on for `wf init`**, `frame-ancestors` moved to `_headers` where a browser honours it, and the build **reads its own output back and holds it to the policy it ships** (`codegen::csp`) — an inline `<script>` or `<style>`, a `style=` attribute, an `on*` attribute, or a script or stylesheet from an origin the policy never named stops the build. A `<script>` whose type is not JavaScript is data and is not reported. One case is widened rather than refused — a `style { }` value that reads state has nowhere else to go, so `style-src` gains `'unsafe-inline'` and the build names the pages that caused it. The check found two real defects on its first run: the static paint wrote `style="display:none"` for a false `show` (now a `.wf-hidden` class) and the published docs output still carries whole page trees from an older site layout (flagged as a separate task; the check now speaks only for the HTML this build wrote). `meta.integrity` emits SRI with `crossorigin`, and a declared external asset without a hash draws a warning — the build never fetches the file, because a build that does network I/O is a different decision. **(5) `Unsafe.Html(markup)`** is the one door for markup, drawing a `V03` at every use that says whether it went through `sanitize`; `sanitize(html)` is an allow-list of elements and attributes with every `href`/`src` through the scheme check, written twice — `codegen::sanitize` and `modules/sanitize.js`, held to the same bytes by a shared case table (`tests/sanitize-cases.json`) — so a pre-rendered article body is sanitised markup rather than an empty box. **`env` split in two**: a name is public when it begins `PUBLIC_` or `public_env` lists it, and any other name read from a page, a component, a store or an `api` block stops the build at the file and line that reads it; the values still reach `wf render`, which runs on a server. The scan is over the lexer's tokens, not the text, so a chapter that writes *about* `env.KEY` is not a page reading it — while a splice inside a string still is. **`wf audit`** prints every `Unsafe.*` and whether it is sanitised, every other origin, everything kept on the reader's machine and where, every `env` name and whether it is public, the policy this build ships and what the compiler depends on; `--json` for a tool, and it never fails a build. **Chapter 19 is Security** (the reference moved to 20, the cookbook to 21): the threat model, what the compiler guarantees, what it cannot, markup, sessions, `env`, uploads, CSP, other origins, `wf audit` and a deployment checklist. **Deviation, on purpose:** the part is `Unsafe.Html`, not `Unsafe.html` — every other part in the language is capitalised, and a lowercase one would be the single exception in a language whose premise is one lexeme, one meaning; `Unsafe` is still what a review greps for. 5 node tests in `tests/js/security.test.mjs` and 7 in `tests/language_4_0.rs`, beside the unit tests in `codegen::url`, `codegen::csp`, `codegen::sanitize` and `linter::secrets`. |
| 2026-09-25 | 10. Interop | **Done.** All three mechanisms. **(a) `external`** is a typed import: `external Chart from "…" { integrity: "sha384-…"  fn Chart(canvas: Any, config: Map) -> ChartHandle  type ChartHandle { update(data: Map)  destroy()  width: Number } }`. The compiler cannot read the other side, so the declaration **is** the contract. The imports go in **`externals.js`**, a module of their own rather than the bundle — a module has its own scope and a split build's page chunks are classic scripts, so each import is bound to a global there and the bundle stays what it was; a module script and a deferred classic script run in document order, so it has finished before `app.js` reads any of it. The origin joins `script-src`, and `integrity:` is emitted as `<link rel="modulepreload" integrity>` — the one place the platform lets subresource integrity reach an ESM import, since `import` takes no such attribute. **(b) `Host`** is a node with a lifetime: `Host(tag: "canvas", mount:, update:, cleanup:)` → `WF.attach`. The mount runs once, the update again on the state it reads, and the cleanup is `onCleanup` on the scope that owns the element, so the thing people forget is the thing they cannot. A library that throws on mount takes itself out and the page stays. `tag:` is an allow-list, because the value is written into the document. **(c) Custom elements, both directions.** `external element Stripe("stripe-pricing-table") { prop publishableKey: String  event ready() }` is placed like a component, its props the attributes a framework would write (`publishable-key`, following state) and its events DOM events the page hears. `output_type: "elements"` with `build.elements` publishes the project's own: `PriceTag` is `<price-tag>`, a one-word component is prefixed so its tag has the hyphen a custom element needs, and the build writes `elements.js`, `styles.css` and a page listing the tags. An attribute is a prop — a `Number` read as a number, a `Map` or list as JSON (`WF.jsonAttr`), a `Bool` true when present and false when `="false"` — read again whenever it changes; a declared event is a bubbling `CustomEvent`; and `WF.scoped` means what the component made goes when the element leaves the document. A `build.elements` name the project does not declare stops the build, and `U03` no longer reports a published component as unplaced. Throughout: `wf types --json` lists the externals beside a project's own declarations, the LSP hovers and outlines them, the tree-sitter grammars and the Zed queries know `external`, `Host` and `Unsafe`, and the `.wf` ↔ `.wfx` round trip is byte-identical. **Deviation, recorded:** the roadmap's `external x from "d3"` resolving out of `node_modules` is not implemented — a bare specifier is emitted as written and left to the page's import map or the host to resolve, because reading `node_modules` would make the compiler a module resolver and a build depend on an installed tree. 4 node tests in `tests/js/interop.test.mjs` — including one that builds a real elements project with the compiler and runs what it wrote — and 5 in `tests/language_4_0.rs`. Found while testing, flagged as a separate task rather than fixed here: a page `state` named `key`, `value`, `event` or `e` compiles to a bare identifier instead of a signal read. |
| 2026-09-25 | — | **`wf verify`**, and every example swept. The browser check stopped being a thing I did by hand and became a command: it starts a headless Chrome, serves the build, opens every route, and fails on an uncaught exception, a `console.error`, a file that did not arrive, an image that failed or a page with no text — reporting the first paint, the node count and the bytes each route fetched, with `--budget MS` to gate the paint and `--json` for a pipeline. The DevTools Protocol client is written here (`src/browser/ws.rs`, 200 lines of handshake and frame): it talks to a process the build started, on the loopback address, so it needs no TLS, no compression and no fragmentation, and a dependency would be larger than the problem — the same judgement that keeps the PDF writer and the gzip encoder. `wf registry --json` now carries each built-in's tag and root class, so a tool reading a built page knows which element is which without guessing. `just browser` sweeps every example through `wf verify`: the five HTML fixtures, both `wf init` templates, the documentation site and Halyard — **124 pages, zero problems**, and **every one of the 52 browser-renderable built-ins drawn**. Getting there closed two gaps it found: `Textarea` (new in this release) and `Toast` had never been drawn in a browser by anything, and the gallery fixture referenced three images it did not ship. Also found by the real corpus: the `env` lint reported `indexOf`, `length`, `filter` and `concat` as leaked secrets, because a method call is not a value read and a `state env` shadows the build's — a security warning that cries wolf is worse than none. |
| 2026-09-25 | — | **A test that acts.** `wf test` could only render, so nothing in the language could exercise a button: a click runs a handler, which runs an action, which changes a store, which repaints. A test with an interaction is now compiled into a real page, served and run in a headless browser — the same compiler, the same runtime, the same output a reader gets, thrown away afterwards. Four steps: `expect` (and `expect not`), `click "Save"`, `type "Ada" into "Name"`, `press "Escape" in "Search"`, run **in the order written**, because what a click did is only visible in the expect that follows it. Everything is found the way a reader finds it — a button's text, a control's label, its placeholder, its `aria-label` — which is the same name the accessibility checks hold a page to, so a test that passes is a page somebody can use. A handler that threw fails the test even where the expects would have passed. `TestDecl.expects` became `TestDecl.steps` so the two kinds interleave; `acts()` is what routes a test to the browser or the renderer, and `type` is told from a `type` declaration by what follows the word. It found a real bug in the first fixture it was pointed at: `IncidentStore.resolve` changed a field of an item in place, so the list stayed the same list and nothing repainted — the classic mutation-without-assignment, invisible to every test that only renders. It also found one in the CDP client: events arriving while a request was in flight were being read past and dropped, so a handler that threw was never heard. 4 tests in `tests/language_4_0.rs` pin the vocabulary, and the dashboard fixture now carries three tests — one that looks, two that act — which `just browser` runs. |
| 2026-09-24 | 1. Strings | **Done.** The U+FFFE/U+FFFF sentinels are gone: the v2 lexer hands the parser the spelling between the delimiters and the parser resolves escapes, dedents a block form and splits splices in one pass, so `\{` is a brace from the moment it is understood. This fixed a live defect — the published documentation site was painting `￾` and `￿` in inline code (440 lines of the built site changed, every one of them a brace). Added: raw strings `#"…"#` (any number of hashes), block strings `"""…"""`, formatted splices `{total:.currency}` / `{when:.date(long)}`, and `dedent()`, `lines()`, `words()` beside `trimStart`, `trimEnd`, `at`, `normalize`, `localeCompare` in the checker's method table. The template engine's second interpolation pass is gone with them — the parser is the only authority on what a splice is. `wf fmt`, the `.wfx` round trip, both highlighters (pinned to each other), the tree-sitter grammars and the Zed queries all know the new forms; `scripts/site-from-guide.py` writes code samples as raw strings instead of escaping every quote and brace. |
| 2026-09-25 | — | **Every phase of the plan is done.** Measured at the end: a heading-and-paragraph page is **3.7 kB gzipped** (2 of 35 runtime modules, 22.3 kB of 200.6 kB before minifying), against the 18.3 kB it was at the start and the 4 kB the plan asked for; the documentation site's `app.js` is 47.8 kB gzipped, and it builds with a Content-Security-Policy it satisfies. The suite: 22 Rust test binaries, 166 node tests, clippy with `-D warnings`, `cargo fmt --check`, `just zed-check`, and both editor grammars regenerated. Left for 4.1, as planned: the service worker and WebRTC (§3f), and the studio integration. Deferred with reasons inside their own entries: AVIF and focal crop (§5), build-time resolution of a static `get` and `Stream<T>` (§3), scroll-driven animation (§7), and `node_modules` resolution for a bare `external` specifier (§10). The work surfaced defects of its own, and they are fixed here: the published `docs/` output carried 62 files from an older site layout, which are removed and the tree now matches a fresh build exactly; a page `state` named `key`, `value`, `event`, `e` or `params` compiled to a bare identifier; and `Input(bind: Store.member)` compiled to a control with no binding at all. |
| 2026-09-25 | — | **Run in a browser**, which nothing above had been. The pane serves and scripts the page but never paints, and that turned out to be the most useful thing about it: it is a page with **no animation frames**, and three pieces of the engine were waiting on one. A route change with `Router(transition:)` never completed, because the View Transitions callback needs a captured frame; a `show` with `.expand` stayed shut, because the filled animation pinned the height it never left; and `count:` never moved, because it drives on `requestAnimationFrame`. Each is a real failure in a background tab or a prerender, and each had the same cause — the DOM's correctness was resting on a piece of decoration. Fixed by the rule the engine already applies to reduced motion: **the end state is the DOM's, and the animation is how it gets there, not whether.** `play` races `finished` against the clock, `expand` sets the end state before it animates (`fill: none`), `countTo` writes the final value on a timer, and the router draws the page when the transition does not begin. Also found and fixed in the browser: `bind:` to a store member was a silent no-op, and the new `skipTransition` guard left an unhandled rejection on every route change. Verified against a minified release build: the box opens, the number arrives at `$1,284.00`, an `.app` store survives a route change while a `.route` store resets, the persisted value carries its version envelope, the injected `<script>` never runs, `rel="noopener noreferrer"` is there, and no promise goes unanswered. New: `P03` warns that a `.route` store which persists reads its value straight back. Released as **4.0.0** — both crates bumped, `RELEASE_NOTES.md` written with the breaking changes first, and `wf migrate` carried from "2 → 3" to "2 → 3, then 3 → 4": it fills `public_env` with the `env` names the pages already read, names every `on*` attribute and hostile URL literal with its file and line, and states the changes that need no edit. |
| 2026-09-25 | — | **The documentation audited against the compiler, which found six defects in it.** Every list a reader relies on is now checked mechanically against the source that defines it: the 13 CLI commands, the 41 diagnostic codes, the 67 config keys, the 149 runtime exports, the 66 built-ins, the 24 parts, the 32 icons, the 9 easings, the 14 animation names, the 11 validation rules, the 7 refinements, the 10 format styles, the 12 universal props, the 12 DOM events, the 41 scalar methods and — newly pinned by a test — all **71 design tokens**, thirteen of which the styling chapter had never named. Every internal link resolves. `check_strictly` (the semantic and type checks, with warnings as failures) now runs over **AGENTS.md, the README, the release notes and the specs**, not only the guide, and gained a rule that every `env.NAME` a guide shows must begin `PUBLIC_` — a reader copying `env.API` got a compile error the page they copied it from did not have. That is how the rest were found: `uid()` and `uuid()` were written in four examples and **neither existed**, so `uuid()` is now the language's — the `Uuid` scalar had no way to make one — and it works in a store action too, where `optimistic`, `beacon` and `sanitize` had also been emitting bare calls. `truncate(text, n)` is a method, not a function. `Navbar(brand:)` is not a prop. And holding AGENTS.md to the checker found `form.pending`, `form.errors`, `form.touched` and `form.apply(…)` reported as `T05`: the checker's idea of a form handle was missing half of what the runtime gives. Four compiler bugs came out of writing the missing `api` documentation and then running it: an `api` block's `on request`/`on response`/`on error` **compiled to empty functions** (the body was generated into a string that was thrown away); a handler's parameter in a hook or a `socket`/`stream`/`channel` compiled to `_m()`, a signal nobody made; `match` over a connection called the handle as a function and took the page's first paint with it; and `if let` inside an action, a handler or an `effect` read the bound name as a signal. Two checker gaps with them: `T06` held a store to its members only where the body wrote `use`, and never inside another store or an `api` block, and `U04` could not see the reads in a `headers { }` or a hook. The guide gained the `api` hooks table, the connection-handle table, `replayAnimation`, the complete built-in-function list with the browser globals beside it, and the full token table; Halyard builds unchanged under the stricter checks, and the rebuilt documentation site's **90 pages verify clean in a browser**. |
| 2026-09-26 | — | **After the release: what the two copies of everything had let drift.** Merged 3.2.1 into v4 — its `removeAt` fix had been dropped by the runtime split while `codegen/js.rs` kept emitting `WF.removeAt`, and is ported into `modules/helpers.js`. A store's action and a page's handler were compiled from two method tables that had drifted: inside an action, `remove`, `contains`, `toUpper`, `toLower` and all 39 scalar methods compiled to methods the value does not have, and threw on first use. There is one table now, `method_to_js`, read by both; `tests/store_method_parity.rs` holds the emitters to each other. `items = items.remove(i)` on a page `state` set the list to what `set` returns, which is nothing; it evaluates to the new list now. The static paint knew `toUpperCase` but not `toUpper`, and resolved `t(…)` only as the whole of a text — spliced, added or held in a prop, a translation painted empty until the script ran; the evaluator answers `t` wherever it is written. The documentation site: the hand-written rail never learned chapter 19, Security, so the page was reachable from no navigation and the two chapters after it were numbered one short; the Styling chapter was behind `md-docs/` by five rows of the token table; the breadcrumb and prev/next titles were English beside a translated rail; the header read 3.0. Two tests now hold the site to the guide — every chapter in the rail and both translation tables, and every generated page current with what `scripts/site-from-guide.py` writes. Released: 4.0.0 tagged, binaries published, and the crate on crates.io, with `editors/`, `docs/`, `site/` and `tests/` excluded from it (15.6 MB, which crates.io kept timing out on, to 3.3 MB). The `t(…)` static-paint fix and the site changes landed after 4.0.0 was published. |

---

## What I deliberately left out

- **A component library expansion.** More built-ins is not what was asked and
  would make phase 0 harder.
- **SSR with a server runtime.** The template engine covers server-rendered HTML;
  a live server runtime is a different product.
- **A plugin system.** `external` (§10) covers the real need — reaching other
  people's code — without inventing a plugin API to maintain.
- **Anything about the studio**, which has its own plan.
