# 7.0 verification — findings

What was checked by hand, in the built-in browser, against the lab
(`verify/lab7`, served with `verify/server.mjs`), and what came of it.
Each finding: what was done, what happened, what should have, the cause,
the fix, the proof.

## Part A

### Checked and right

- **Cursor paging** (`pages .cursor`): Load more twice gathers 3, then 6,
  then 8 people; `hasMore` goes false; each page asks with `after=` the
  last id.
- **A refresh** (`people.reload()` with 8 gathered) asks again for all
  three pages; during it the read is `.loaded` with `refreshing` true and
  `pending` true; the row's input is the same node afterwards and keeps
  the focus.
- **A refresh that fails** (500 on every try) stays `.loaded`, `stale`
  holding the error; a `GET` is retried twice with growing gaps (178 ms,
  436 ms).
- **A changed search** with `placeholder: .previous` keeps the last list,
  refreshing, then shows the new one; one request per change.
- **Lookup**: 404 lands in `.notFound` with the server's detail; a found
  person's `joinedAt` shows as a long date.
- **Writes** (after fix 2): an optimistic `update` hides the row at once
  and `undo()` brings it back on failure; `rename` (`PUT`) and `remove`
  (`DELETE`) each ask for the list again, as `invalidates` says.
- **Create's results**: an empty form is stopped before anything is sent
  ("This is required"); 409 → `.conflict` with the server's detail; 418 →
  `.clientError` (`on 4XX`); 500 → `.http`, and the `POST` is sent once,
  never retried; 201 → `.ok`, and `invalidates [list]` asks for the list
  again.

- **Transport**: middleware put `Authorization`, `X-Client` and a fresh
  `X-Request-Id` on every try; an `idempotent` `POST` that got 503 twice was
  sent three times with one `Idempotency-Key`; a plain `POST` once; three
  reads at once whose token had expired made one `/auth/refresh` and three
  retries with the new token; a 5 s read against a 3 s timeout ends as
  `.timeout` after two retries (10 s in all); leaving a page aborts its
  read (the server sees the client go); caching follows the headers
  (`max-age` served without asking, `no-store` and no headers always ask,
  `ETag` revalidates with `If-None-Match` and takes the 304, then the new
  version); an answer of the wrong shape fails, in a deployed build, as
  `$.items[0].role is "pilot", which is none of .engineer, .designer,
  .manager`.
- **Paging and streams**: number paging stops at the empty page, `Link`
  paging at the last `rel="next"`; a refresh asks again for every page
  gathered; a streamed read grows a line at a time; `for await` walks the
  lines in an action.
- **Offline** (service worker on): a `PUT` whose connection drops is tried
  three times, then kept ("Kept, to send when the connection returns"), and
  reached the server once, with its change, when it answered again; a
  `POST` is not kept.
- **Fakes**: `wf test` passes the lab's three tests, and fails the two
  meant to fail, saying why — text not shown, and `No fake answers
  Backend.people.byId`.
- **`wf render`** paints a read's `.loading`, `.loaded` and `.failed` arms
  from the data, and its members agree.

- **A contract `wf api` wrote**, from a YAML spec, called in the browser:
  cursor paging from `x-pagination`; the `X-Trace` header and the
  `lab_session` cookie parameters sent; 409 into `.conflict`, 404 into
  `.notFound`; a `text/plain` answer as `String`; an `image/png` answer as
  a 70-byte file; a multipart upload (after fix 6).

### Found and fixed

1. **`form.apply` did nothing for a control no `validate` block guards.**
   A 422's field errors applied to `Input(bind: email, name: "email")`
   showed no message, set no `aria-invalid`, and moved no focus — though
   the docs and the recipe say it shows each message under its field.
   *Cause:* `apply` looked only in the fields that `validate` registers.
   *Fix:* `apply` finds any control of the form by `name`, shows the
   message beneath it with the field plumbing (`role="alert"`,
   `aria-invalid`, `aria-describedby`), focuses the first, and clears it
   when the reader edits the control or the form is reset.
   *Proof:* the lab shows "Use your work address" under Email, focused,
   `aria-invalid="true"`, and it goes when the address is changed.

2. **An endpoint named like a list helper, in a group, was compiled as the
   helper.** `Backend.people.remove(id: p.id)` compiled to
   `Backend.people = WF.removeAt(Backend.people, …)`: no request was sent,
   the group was overwritten (every later call of it, such as `rename`,
   failed), and an optimistic removal never came back.
   *Cause:* the list-helper table took `remove` on anything held like a
   store member, and codegen did not know `Backend.people` is a group —
   the plan's B12 was fixed for a service's own endpoints, not a group's.
   *Fix:* codegen knows every service and group, and a method called on
   one is an endpoint call, whatever its name.
   *Proof:* the lab: remove shows the row gone at once, the `DELETE` is
   sent (retried twice on 500, as a safe write is), the row comes back with
   "still here"; rename sends `PUT` and the list is asked again.

3. **An error thrown in a busy button's handler vanished.** A `Button`'s
   `on click` that calls an async action is wrapped to show it busy; that
   wrapper waited on the work with `Promise.allSettled`, which counts as
   handling a rejection, so nothing reached the console, `wf serve`'s
   overlay or a test. (Found while finding 2 was breaking every later call
   silently.) *Fix:* after the work settles, each rejection is thrown again
   on its own turn, where every uncaught error is reported. *Proof:* a
   handler that throws, in the lab, now shows `Uncaught Error: …` in the
   console.

4. **`wf render` did not hold a read's data to its endpoint's type.**
   `{ "users": { "loaded": [{ "id": 1 }] } }` for `resource users =
   Backend.users()` (whose endpoint answers `[User]`) was drawn as an empty
   line; only a read written with its own annotation was checked.
   *Fix:* a read with no annotation is held to what its endpoint answers
   (a paged read whose items take a function to find is left unchecked).
   *Proof:* the render now says `T33: … users.loaded[0].id is 1, not a
   String`, and the right data still draws.

5. **A generated endpoint with no error enum and a setting did not parse.**
   `wf api add` on a spec with a text answer wrote `get version at
   "version" -> String! as: .text`, and the parser took `as` as the error
   type after `!`, so the generated file did not parse (`E002`) and `add`
   stopped. The oxn spec passed only because each of its endpoints named an
   error enum. *Fix:* a setting word followed by `:` after `!` ends the
   type. *Proof:* `wf api add` on the lab's YAML spec writes all three
   files, and they parse in the compiler and in tree-sitter.

6. **`FileUpload(bind: files)` bound nothing** (without `.dropzone`): the
   chosen files never reached the state, so an upload had nothing to send.
   *Fix:* the input writes `Array.from(files)` to the bound state on
   `change`, beside any `on change` the call writes.
   *Proof:* the lab uploads a picture as `multipart/form-data`, the file
   under `picture` and the rest of the body beside it.

7. **An endpoint parameter's default was never sent.** `pageSize: Number
   = 2 in query "page-size"` sent no `page-size`, so the server's own
   default (3) applied; the same for `page: Number = 1`.
   *Fix:* each endpoint carries its literal defaults, and a call that
   leaves a parameter out sends its default.
   *Proof:* the lab's first request is `GET /api/people?page-size=2`, and
   the next page `?after=grace&page-size=2`.

8. **`wf api check` listed its changes in the past tense** ("added …")
   though it writes nothing. *Fix:* it says what `sync` would do
   ("add", "rewrite", "remove").

9. **`wf migrate` refused the 6.2.0 site**, whose `build.output` is outside
   the project (`../docs`): its check ran a build's path rules. *Fix:* the
   migration's check allows an output outside, since it writes no output.

10. **`wf migrate` made the 6.2.0 site worse**: writing `Json` for every
    `Map` turned 21 `T24`s into about 398 errors, mostly `T28` (a `Json`
    read as a record). *Fix:* the `Any`/`Map` → `Json` step is applied
    file by file and kept only where it lowers the project's errors;
    elsewhere the `T24` stays for a person, who names a record type.
    *Proof:* a fresh copy of the 6.2.0 site migrates to 24 errors left
    (19 of them `T24`, each a `Map` that wants a record); halyard-review
    leaves 1, hard-pocket 6, each a real decision.

11. **`wf render` drew `else` for a loaded read matched as `.loaded(v)` and
    `else`.** The template engine knew a `match` was over a read only by
    its arms naming `.loading`; with `.loaded` and `else` it read the
    read's value as an enum and found no arm. (Found writing the test for
    fix 4.) *Fix:* a value the data gave as a read is matched as one,
    whatever the arms. *Proof:* `tests/hands_on.rs`.

12. **A write kept offline lost what the middleware added.** A `PUT`
    behind `middleware [bearer(Session.token)]` that failed for want of a
    network was kept, then replayed with no `Authorization`, so the server
    refused it: only 6.x's `on request` hook was registered to make a kept
    write up again, and 7.0 removed that hook. (Found reading the offline
    recipe, which still named the hook.) *Fix:* a service with middleware
    registers it for replay: the chain runs to where the network would be,
    and the request that reaches it is sent, with the token of that moment.
    *Proof:* `tests/js/offline_queue.test.mjs`.

13. **`let x: T = try await http.get(…)` was not held to `T`.** The docs
    say an answer is checked against the type named where it goes; a
    `resource`'s was, a `let`'s was not, so `[]` reached `.ok` as a `Rows`
    and the action crashed reading `r.items`. (Found when fix 3 stopped
    hiding the crash in the guide's own example.) *Fix:* a `let`'s declared
    type is the shape its awaited `http` call checks the answer against
    (`T!` and `T ! E` are `T`'s). *Proof:* `tests/hands_on.rs`; the
    guide's example now lands in `.err`.

Each fix has a regression test: `tests/hands_on.rs` (2, 4, 5, 6, 7, 9, 10,
11) and `tests/js/hands_on.test.mjs` (1, 3, 7).

### Migration of the 6.2.0 code

Every example and the site, from `git archive v6.2.0`. Most check clean
after the migration; what is left is what a person decides (a `Map` that
wants a record type, a `Json` read as a record, an `Any` list, a value
that may be null passed where it may not).

### Noted, not changed

- An `HttpError`'s message for a status is `HTTP 500` even when the body is
  a Problem with a `title`; a page that wants the title matches
  `.status(code, problem)`.
- A `POST` whose connection is reset before any answer reached the server
  twice: the browser itself repeats a request on a reused connection that
  closes (a raw `fetch` does the same; same request id, same millisecond).
  Not the runtime's retry, which sends a `POST` once.
- `.timeout(after)`: a binding named `after` is `E118` (it is the
  language's timer); the docs' spelling of the payload is a name to
  avoid in code.

## Part B

### Checked and right

- **A crawl of every route** (`verify/crawl.mjs`: 588 routes, each at
  desktop, phone and dark, in headless Chrome against `docs/` served as
  GitHub Pages serves it): no console error, failed request, broken link,
  broken image, empty section or cell, unhighlighted code block, raw
  Markdown, duplicate id, or content wider than a 375 px screen. Every page
  has one `h1`; the Arabic pages are `lang="ar" dir="rtl"`.
- **The CLI page** names every command, subcommand and flag
  `wf commands --json` lists.
- **The live examples and the Playground** load the WebAssembly compiler,
  compile in the browser, and report 7.0's findings (`T23`, `T15`, `T20`).
- **Search** finds the 7.0 subjects (middleware, `idempotent`, fakes,
  OpenAPI).
- **Stated numbers**: 83 icons, 255 built-ins, and a static page's
  `app.js` of 1.9 kB gzipped are still true.

### Found and fixed

14. **Seven links to `/reference#modal` and the like led nowhere**: the
    components reference became a page per built-in, and the guide's links
    into the old one-page reference kept their anchors. *Fix:* the
    generator sends `components.md#<built-in>` to `/reference/<built-in>`,
    and `#icons` to the icon list in the recipe.
15. **The changelog showed 7.0.0 as "Latest"** before it is released (it
    has no date until it is tagged), so a docs deploy before the tag would
    have announced it. *Fix:* the latest is the newest release with a date;
    one without says "Not released yet" (and "next" in the list), and shows
    every section.
16. **The 7.0 release notes had one heading twice** ("A write is sent again
    only when that is safe"); the second is gone, and the notes now list
    the fixes above.
17. **Error pages showed backticks literally** in their lede (`` `POST` ``),
    and 246 pages (224 error pages, 22 recipes) put Markdown in their
    search snippet, six of them past 160 characters (`S03`). *Fix:* the
    lede is drawn as Markdown; a page's `description:` is written as a
    search result shows it — no marks, cut at a sentence or a word.
18. **Words 6.x used, left in the docs**: the data chapter's "Reading
    `.data`" (now `.value`); the offline recipe and the agent guide naming
    an `on request` hook (now middleware); the grammar page's `api` line
    (now with `middleware`, groups, `invalidations`, `pagination`,
    `middleware` and `fake` declarations); the config page had no `apis`
    section; the performance page and the agent guide said the runtime has
    44 modules (58).
19. **A build said "6 warning(s)" for six `V11` notes**, which are
    information. *Fix:* only warnings are counted.

20. **Your own `~/Code/halyard` (6.x) left 66 errors after `wf migrate`**
    (it is one of the browser suite's projects, so the suite reported
    "halyard: build failed"). 27 were `T18`, text joined to a number with
    `+` (`"+" + f.add`), which 7.0 refuses but the step did not rewrite:
    `T18` was not among the codes it applies, and the finding carried no
    fix. *Fix:* `T18` on a plain string and a value carries the fix
    (`"+{f.add}"`), and the step applies it. *Proof:* a copy of halyard
    migrates to 47 left; `tests/hands_on.rs`. What is left is a person's:
    lambdas with untyped parameters (`T32`), actions that return records
    without saying which (`T26`), `RegExp(…)` and `Object.assign` (`T30`),
    `Any` and `Map` read as records. The project itself was not touched.

### Noted, not changed

- At about 800 px wide the home page's example squeezes its file name
  under the "Open in the Playground" button; desktop and phone are right.
- `wf check` refuses a project whose output is outside it unless it is
  given `--allow-output-outside`, though it writes nothing — as the docs
  say.
