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

### Requests, the rest by hand

Checked and right, in the browser against the lab: `refresh:
.every(2.seconds)` asks every 2 s exactly; `.onFocus` asks once, that read
alone; two readers of one address make one request; `Retry-After: 1` is
waited (1003 ms); a write kept offline that the server refuses on its
replay lands in `network.refused`, and `network.clearRefused()` empties it;
the replay carries the middleware's headers (fix 12); after a rebuild the
page offers the new version (`update.available`) and `update.apply()` takes
it with one reload.

32. **`reload()` did not ask while a kept answer was fresh** — a server's
    `max-age`, or `cache: .swr(…)` — though the docs say it asks now; a
    refresh button did nothing for a minute. *Fix:* a refresh (`reload()`,
    `refresh:`) goes to the server whatever the freshness, sending the
    `ETag` it holds; the value stays shown, `refreshing`, until the answer.
33. **Only one variant of a `Vary` answer was kept**: English, then Arabic,
    then English asked three times within the English answer's `max-age`.
    *Fix:* each variant is kept under its own key. *Proof:* the lab asks
    once for each language; `tests/js/hands_on.test.mjs`.

### Fakes, `wf api` and the language server, by hand

Checked and right: a fake `from examples` answers with the spec's example
and its own answers win; `wf verify --fakes` builds and loads the site with
the fakes, naming each endpoint none answers; `N06` names each way a fake
breaks its contract; `wf api list`; `sync` rewrites an edited generated
file and leaves one without the marker alone (saying why); `check` and
`N07` catch an edit; an XML answer stops `sync` (`E111`) until `--skip`
names it; `eject` turns `api … from` into files that build.

34. **A wrong example in the spec read as a mistake in the fake**: `T02
    Role has no case .wizard` at the `fake` line, naming no example. *Fix:*
    an example is JSON, held to the endpoint's answer as data is: `T33 the
    spec's example of getPerson is not what Lab.people.getPerson answers:
    $.role is "wizard", which is none of …`.
35. **"No fake answers X: name one in the test's `fakes:`"** under `wf verify
    --fakes`, where there is no test; it now names both places.
36. **Messages still said `Any` and `Map`**, which 7.0 took out of the
    language: an empty list was `[Any]`, an open map `Map`. They are `[]` and
    `{…}`.
37. **The language server knew nothing of groups**: hover on
    `Lab.people.listPeople` was empty, completion after `Lab.people.` offered
    nothing, and signature help looked for a service called `people` — for a
    group written by hand and one read from a spec alike. *Fix:* the owner is
    the service and the group together; hover shows the endpoint (its verb
    as the language writes it, `get`).

### Migration, by hand

A 6.x project written with every row of the plan's migration tables.

38. **Four rows promised a rewrite the step did not make**: `errors { 404 ->
    Problem404 }` (now an `enum UserError { notFound(…) on 404 … }` and `->
    User ! UserError`), `fetch(url, method: "POST", body: …)` in an action
    (now `try await http.post(url, body: …)`), `paginate: .page` (now `pages
    .number(param: page)` on the endpoint, the call losing the argument), and
    `users.state == "ready"` (now `users.isLoaded`).
39. **A `fetch` inside `try { } catch` became `try await http.get(…)`**,
    which passes a failure up out of the action, so the `catch` that used to
    handle it never ran — a silent change of behaviour. *Fix:* inside a `try`
    block the step does not write `try`; the finding is left for a person,
    with a hint to `match` the result.
    *Proof (38, 39):* `tests/hands_on.rs` migrates
    `tests/fixtures/migrate7/messy-6.wf`; what is left is the `on request`
    hook, an empty `{}` state and that `catch`, each a person's decision.

### halyard, migrated

`~/Code/halyard` (on its branch `webfluent-7`, not committed): `wf migrate`
then 47 decisions by hand — record types for its data (`Incident`,
`OutLine`, `CodeProblem`, `LogRow`, …), dictionaries (`[String: String]`),
regex literals, a project script that asks the browser whether a pattern
is valid (in place of a hand-written scanner 6.x needed for want of
`try`), and a local `dir` that read the language's text direction. Every
page then drew pixel for pixel as the 6.2 build did. The hand-made command
palette, toast stack, menus, popover and drawer became `CommandPalette`,
`toast(…)`, `Menu`, `Popover` and `Sheet`, drawn in the design's colours
through its theme's baseline names.

40. **`ref:` on a component call bound nothing**: it was passed as a prop
    the component does not take, so `Menu(anchor: launcher)` opened in the
    corner. It is now a handle on the component's root, as on a built-in.
41. **A structural project's `Popover` was the browser's white, page-wide
    box**: its surface was in the full sheet only, though a `Menu`'s is in
    both. It is in both.
42. **Toasts were set in the browser's serif** in a project whose font is
    set on `body`: the stack lives outside it. It takes `--font-family`.

### The guide, read for 7.0

Three readers went through Learn 1–17, the data, forms, testing and offline
recipes and the about pages, running every example against the compiler.
What they found, now fixed:

43. **Chapter 12 still taught WebFluent 6's types**: a *gradual* checker,
    `Any` and `Map` in the type table and the quick reference, mixed `if`
    arms that are `Any`, `fetch` returning `Any`, `groupBy` giving `Map`.
    Rewritten for 7.0's strict checker (`T23`, `T24`, `T25`, `T27`, `T16`,
    `N11`); chapter 10's prop types and the introduction's "types are
    optional" too.
44. **"Under `wf serve` … a `.parse` error"** in chapters 12 and 15: an
    answer is checked in every build, and fails as `.decode(at, message)`.
45. **6.x wording**: chapter 15's "no `error` arm" (`.failed`), chapter 7's
    `fetch` among the globals, "This guide describes WebFluent 6", the
    static-paint table's `loading` arm and `await fetch`, the installer's
    `v6.2.0` and `wf 6.0.0`.
46. **Examples that taught a bug**: the shelf tutorial's and the upload
    example's bare `await` (`N04`: a failure went unheard and `done`/`sent`
    turned true anyway; now `try`), chapter 15's `optimistic` example (could
    not roll back — now `-> !` and `try`), chapter 11's socket (`T28`: now
    typed with `send`/`receive`), chapter 14's `form.values` (`T19`),
    `-> Json! as: .blob` (`File!`), `Text { Text(…) Kbd(…) }` (a `<p>` in a
    `<p>`; now a `Row`).
47. **Facts that were wrong**: "thirteen kinds of declaration" (fifteen —
    `middleware` and `fake`); the `T33` message quoted for a seed; the
    feedback built-ins left out `AlertDialog`, `Sheet`, `ConsentSettings`;
    the notebook recipe's `ThemeToggle` "two icon buttons" (one); "sixteen"
    browser suites; a snapshot's file name; the `1.9 kB` page (one page,
    no router); the sign-in recipe's "Needs" (the starter's `Profile.wf`
    reads the store it replaces) and its "in a `try`"; the load-more
    recipe's cursor (a cursor needs `items:`; so did the hint the parser
    gives and the comment `wf api` writes in `service.wf`).
48. **`optimistic(holder, change)` gave its change no types**, so every
    example of it was `T32`. *Fix (checker):* the change is a function from
    what the holder holds to the same, and its parameter is typed by it.
49. **`form.reset()` reset the controls on the page but not the states
    bound to them**, so the page and the program disagreed after it.
    *Fix (runtime):* each control says what it holds after the reset.
50. **The `spa` starter's own `ThemeToggle` hid the built-in**, so a new
    project's first build printed a `V11` note. The starter's switch is
    `DarkModeSwitch`.

### Noted, not changed

- `Text` inside a `Text` renders a `<p>` inside a `<p>`, which a browser
  splits. The guide's one such example is now a `Row`; making an inner
  `Text` a `<span>` touches all three renderers and is left for after the
  release.

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

## Part C — after the first report

### The search dialog

21. **Its contents filled 500 of its 640 px**, leaving an empty band on the
    right: the site sized the `Modal`, but a built-in `Modal`'s content was
    fixed at 500 px whatever the dialog's size. *Fix (engine):* a `Modal` and
    a `Dialog` are as wide as their `<dialog>`, and the content fills it, so a
    `class:` or `style { }` on one sizes what it shows.
22. **The arrow keys did nothing** in a list marked `listbox`, and Enter
    always opened the first result. *Fix:* the field is a `combobox` whose
    `aria-activedescendant` follows the arrows, Enter opens the one marked,
    the pointer marks what it rests on.
23. **Two words found nothing unless written together** (`sticky header`);
    now every word is matched, in any order.
24. **Results stopped at 12** without saying so; now 40 show, scrolling, and
    the count of every match is said (`role="status"`).
25. **Arabic found nothing**: the index had no Arabic page. The Arabic
    chapters and sheet are indexed with their Arabic words and their own
    anchors, and each result's text takes its own direction (`dir="auto"`).
26. **`/` opened the search and typed itself into it**, so `/button` found
    nothing; **"Nothing matches"** came with "0 results" under it; at phone
    width **the kinds wrapped** to two lines; **the field drew two focus
    rings**. Each fixed.

### The built-ins and the theme

27. **Built-ins wrote fixed colours where the theme should have spoken**:
    labels on a tone were `#fff`/`#000` (a light primary made its buttons
    unreadable), alerts were fixed light pastels (glaring in a dark theme),
    the focus ring a fixed blue, the rating star and highlight fixed
    yellows, backdrops a fixed black, and a plain toast had white text and
    no background. *Fix:* `color-on-primary` … `color-on-info` and
    `color-backdrop` join the baseline; every tone's label, the toggles,
    pagination, stepper, calendar and chip draw from them; alerts, focus
    rings and highlights are mixed from the tone and the page's colours. A13
    checks each label against its tone. *Proof:* a dark theme with a light
    primary, in the browser: every tone, alert, toggle, page, focus ring,
    modal and toast follows it; `tests/hands_on.rs`.

### The leftovers

28. **A failed status said `HTTP 500`** when the server sent a problem; its
    `detail`, else its `title`, is now the message.
29. **`.timeout(after)` in the docs could not be written** (`after` is the
    language's timer, `E118`); the payload is `limit`.
30. **At 1024–1279 px the header's places ran under the search**, and the
    home page's example squeezed its file name and code. The search is its
    icon there, and the hero sits side by side from 1280 px.
31. **`wf check` stopped** on an output outside the project, where its docs
    say it reports it: it now says a build would stop, and checks the rest.

### Noted, not changed

