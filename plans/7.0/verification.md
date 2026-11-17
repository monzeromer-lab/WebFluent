# 7.0 verification, before shipping

Two things to prove before `just ship 7.0.0`: that every 7.0 feature works
when a person uses it in a real browser against a real server, and that the
docs site says what is true and shows it properly. The automated suites
already pass; this is the hands-on pass they cannot replace. Nothing ships
until both parts are done and every finding is fixed or written down as
known.

Everything found goes in `plans/7.0/verification-findings.md`: the page or
feature, what was done, what happened, what should have happened, the
cause, the fix, and the proof (a screenshot, a request log, a test).

---

## Part A — The 7.0 features, by hand

### A0. The lab

A project made for this, `verify/lab7/` (kept out of the release; deleted
or moved to `tests/fixtures/` at the end), and a small server written for
it, `verify/server.mjs` (Node, no dependencies):

- **The lab site**: one page per feature area below, each showing what it
  reads, a log of what happened, and buttons that make each case happen.
- **The server** answers the team-directory API the plan's example uses
  (people, pages, writes) and can be told, per request, how to misbehave:
  a status (`401`, `404`, `409`, `422`, `429` with `Retry-After`, `500`,
  `503`), a delay, a dropped connection, an answer of the wrong shape,
  `Cache-Control`/`ETag`/`Vary`, a cursor, a `Link` header, a stream of
  NDJSON lines, server-sent events. It records every request — method,
  path, query, headers (`Authorization`, `Idempotency-Key`,
  `X-Request-Id`, `If-None-Match`, cookies), body, timestamp — at
  `/__log`, so each check reads what was actually sent.
- Served as the built site and the API on one origin, through
  `.claude/launch.json`, and opened in the built-in browser. Each check is
  run twice: under `wf serve` (a dev build) and against `wf build` output
  (a deployed build), because some behaviour differs between them, such as
  decoding answers and the 404 box.

### A1. Results and error enums
- A create that answers 201, 409, 422 with field errors, 500, a network
  drop, a timeout: each lands in its own `match` arm on screen.
- `form.apply` puts each 422 message under its field, moves focus, and is
  announced to a screen reader (`read_page` for the ARIA attributes).
- `try` in an action passes the error up; `-> ! E` and `.result`.
- `on 4XX`: a 404 and a 429 land in `clientError`; a status a case names
  wins over its class.

### A2. A read is one value
- `.loading` → `.loaded`; `.failed(e)` on a first failure.
- A refresh keeps the list on screen (`refreshing` true) and keeps row
  nodes and focus (focus an input in a row, refresh, check it is still
  focused).
- A refresh that fails stays `.loaded` with `stale` set.
- An argument that changes asks again; `placeholder: .previous` keeps the
  old list meanwhile; the `refresh:` options `.every`, `.onFocus` and
  `.onReconnect` fire (watch `/__log`).
- The members `value`, `isLoading`, `pending` and `error` agree with the
  `match`.

### A3. The transport
- Middleware order, from the request log: `bearer`, `header`,
  `requestId`, and the project's own middleware.
- `refreshOn401`: three calls at once that each get a 401 cause one
  refresh and three retries.
- Retries: a `GET` that gets a 503 is retried with backoff and honours
  `Retry-After`; a `POST` is never retried; an `idempotent` write is
  retried with the same `Idempotency-Key` on every try.
- Timeout: no setting is 30 s, `timeout:` changes it, and a timed-out call
  is aborted on the server side (the connection closes).
- Cache: `max-age` is served without a request; `no-store` always asks;
  an `ETag` revalidates with a 304; `Vary` keeps answers apart; with no
  headers nothing is kept; `cache: .swr(…)` serves the old answer and asks
  again.
- Dedupe: two readers of one address make one request.
- Abort: leaving a page cancels what it asked (the server sees the close).
- Decode: an answer of the wrong shape is `.decode(at, message)`, naming
  the place, in a deployed build as well as a dev build.

### A4. Contracts and writes
- Groups: `Backend.people.*`.
- `invalidates`: a create refreshes the list in place; a delete refreshes
  the list and that person's read, with the right argument.
- `.update(…)` shows a change at once, `undo()` puts it back when the
  server answers 500, and a paged read's update changes its items.

### A5. Pages, streams, offline
- Cursor, number and `Link` paging: `loadMore()`, `hasMore`, and a refresh
  that asks again for every gathered page.
- A `streamed` endpoint: lines draw as they arrive (the server sends them
  slowly), and `for await` in an action.
- Offline (service worker): precache, a navigation with the server down,
  a `PUT` queued and sent in order when the server returns, a `POST` that
  is not queued and says `.offline(false)`, `network.refused` for a 4xx on
  replay, and `update.available` / `update.apply()` after a rebuild.

### A6. Fakes
- `wf test` on the lab with fakes: a passing test, a failing one (does it
  say why?), a call to an endpoint no fake answers (named?), `expect
  called`.
- `fake … from examples`.
- `wf verify --fakes`.
- `wf render` with a read given in the data.
- N06 on a deliberately wrong fake.

### A7. OpenAPI and `wf api`
- `add`, `sync`, `check`, `eject` and `list` on Petstore YAML, oxn, the
  team spec, and the 3.0 and 3.1 specs from the reader's tests: the
  generated client calls the lab server in the browser, including
  `in query "page-size"`, a cookie parameter, a header parameter, a
  multipart upload with progress, a text answer and a binary answer.
- Hand-edit a generated file → `sync` refuses; change the spec → `check`
  fails and `wf check` says `N07`; a spec with XML → refused, then
  `--skip`.
- `api … from` in the editor: open the lab in VS Code with the language
  server and check hover, completion and the problems list on a spec
  endpoint. If no editor can be driven here, run the language server's
  requests through a small script instead.

### A8. Strict typing and migration, on real 6.x code
- Check out the 6.2.0 release's `examples/` and `site/` (`git worktree add
  … v6.2.0`), run `wf migrate` on each, then:
  - read what it rewrote, judging whether each change is right;
  - read what it left for a person, judging whether each is real;
  - build and open the result in the browser, comparing it page by page
    with the 6.2.0 build of the same project (screenshots side by side).
- Write a deliberately messy 6.x project (every row of the plan's
  migration tables) and do the same.
- Spot-check that every new code gives the message and the fix its docs
  page says it does: `T23`–`T35`, `N01`–`N14`.

### A9. Things 7.0 must not have broken
- Run every built-in's gallery (`wf docs`) and the showcase examples in
  the browser: no console errors, nothing that looks broken.
- The security suites and the CSP of a built site with an API on another
  origin (`meta.connect`).

Bugs found in Part A are fixed with a regression test (a Rust test, a JS
test or a browser suite) before anything else moves.

---

## Part B — The docs site, by hand

### B0. Build and serve
`wf build -d site --allow-output-outside` into `docs/`, served by the
`docs-static` entry in `.claude/launch.json` (port 4100), and opened in
the built-in browser. The live pieces (playground, live examples) need the
WebAssembly compiler built (`crates/wf-wasm/build.sh`).

### B1. A crawl, for what a machine can see
A script drives the browser over every route the site has — 155 pages,
plus the Arabic ones and the error pages. For each page it records:
- the title and `<h1>`;
- console errors and failed requests;
- links that lead nowhere, and `#anchors` with no target;
- images that fail to load;
- headings with nothing under them, and tables with empty cells;
- code blocks with no highlighting;
- content wider than a phone screen (at 375 px).

It runs at desktop and phone width, in the light and dark themes, and once
right to left. The output is a table of every page and what was wrong.

### B2. Accuracy, against the compiler
What the docs claim, checked against what `wf` does:
- **Reference pages**: built-ins (each component's props, flags, events,
  slots, parts and defaults) against `wf registry --json`; the CLI page
  against `wf <command> --help` for every command and flag; the
  configuration page against `wf config schema --json`; tokens against
  `wf theme --json`; error pages against `src/diagnostics/codes.rs`, each
  example run.
- **Numbers** stated in prose ("83 icons", "44 modules", "1.9 kB",
  "152 error pages", counts of built-ins) against the real values.
- **Words 7.0 changed**: grep the built site for `fetch(`, `ready(`,
  `.data`, `errors {`, `Map`, `Any`, `paginate:`, `headers {`, an endpoint
  with no `at`, `-> T` without `!`, and "6.x" claims that are no longer
  true. Read every hit in context.
- **The version** shown in the header, the changelog and the install
  commands — still 6.2.0 until shipping, which is right, but the
  changelog must not present 7.0 notes as released.
- **Live examples and the playground**: run each live example's code in
  the in-page compiler; compare what renders with what the prose says it
  shows.
- **Recipes**: each "Needs" line is true, and each recipe's output looks
  like its description (run a sample of every group in the playground or a
  scratch project, and every recipe that touches data or 7.0).

### B3. Reading, for what only a person sees
Every page type is read in the browser end to end: Home, each of the 36
Learn chapters, the reference, a sample of recipes from every group (all
data and testing ones), a sample of error pages from every family, the
coming-from pages, the changelog, playground, showcase, 404, the Arabic
pages. For each:
- Does the navigation make sense? Sidebar order, current item, previous
  and next, the "on this page" rail and its highlighting.
- Does search find it?
- Does the layout hold together? Overflow, crowded or missing spacing,
  broken tables, code blocks, callouts, dark mode, phone width, right to
  left.
- Is the text right for 7.0, and does what it says match what a reader
  sees after doing it?

The specific wrongness reported ("items not accurate and seem off") is
traced each time to its cause — the source Markdown, the generator
(`scripts/docsite`), a component, or the compiler — and fixed there, so
the next regeneration does not bring it back.

### B4. Fix and re-verify
Fixes go in the source or the generator, the site is regenerated, and the
failing check is run again in the browser. Where a check can be automated
cheaply (a link, an anchor, a count, a stale word), it becomes a test in
`tests/docs_parse.rs`, so it cannot come back.

---

## Order and checkpoints

1. A0, then A1 to A5: the lab and the request engine, where a bug would
   cost users the most.
2. A6, A7, A8 and A9.
3. B0 and B1: the crawl gives the full list of mechanical problems.
4. B2 and B3, guided by the crawl and by what the user has seen.
5. Fixes, regression tests, regeneration, the full suites, then a short
   report: what was checked, what was found and fixed, and what is left
   and why. Commit and push with approval, then ship with approval.

At each checkpoint the findings so far are shown before going on.
