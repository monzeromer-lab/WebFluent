## CLI

```bash
wf init <name> -t spa|static|pdf|slides [--locales en,ar] # Create a project (writes AGENTS.md); --locales: start translated, first is default
wf build [-d DIR] [--stats] [--cache] [--studio] [--format F] [--deny-warnings] [--allow-output-outside] # Compile (--stats: print output sizes; --cache: rebuild changed pages only; --studio: write nodes.json for editors)
wf check [-d DIR] [--format F] [--deny-warnings] [--allow-output-outside] # Report all diagnostics; write nothing
wf fix [-d DIR] [--code C] [--dry-run]   # Apply known fixes (all, or one code's)
wf config schema [--json] | get PATH | set PATH VALUE | unset PATH # List settings; read or edit webfluent.app.json
wf commands [--json]                     # List commands and their options as data
wf assets [--unused] [--plan /a.png] [--json] # List public/ files: kind, size, dimensions, uses; preview image processing
wf theme [--json]                        # List design tokens, their uses and contrast pairs
wf motion [--json]                       # List animations, easings, speeds and route transitions
wf rules [--json]                        # List validation rules and the states they guard
wf scope FILE:LINE[:COL] [--data D] [--json] # List the names in scope at a position, with types
wf translations [--missing L] [--unused] [--plurals] [--json] # Report missing, unused and plural translation keys
wf context [--open FILE] [--guide full] [--json] # Print the context an agent gets (private env withheld); --guide full: whole reference
wf components [--json]                   # List components (built-in and project): props, events, slots, parts
wf api add SPEC [--name N] [--out DIR] | sync [N] [--skip OP] | check [N] | eject N [--out DIR] | list [--json] # Generate and sync an API contract from an OpenAPI spec
wf explain [CODE] [--json]               # Explain a diagnostic code: meaning, example, fix
wf serve [-d DIR] [--host ADDR] [--allow-output-outside] # Dev server on localhost:3000 with live reload and an error overlay (--host 0.0.0.0 for a phone)
wf start [-d DIR] [--port N] [--host ADDR] [--proxy /api=https://HOST/api] # Serve a build in production on $PORT; --proxy forwards a path to an API
wf inspect ROUTE|URL [--do STEP]… [--mobile] [--screenshot [FILE]] [--proxy …] [--fakes FILE] [--json] # Print a page as text (landmarks, headings, controls, errors); --do acts first
wf generate page|component|store|block <name> [-d DIR] # Scaffold a page, component, store or block (sign-in, pricing, dashboard, settings)
wf fmt [path] [--check] [--stdout]       # Format sources (--check: fail if anything would change)
wf fmt --to wfx|wf [path] [--stdout]     # Convert a project between .wf and .wfx
wf test [path] [--update]                # Run the tests under tests/
wf docs [-d DIR] [-o OUT]                # Generate a documentation site for the project's components
wf migrate [path] [--check] [--stdout] [--wfx] # Migrate WebFluent 2 → 3, 3 → 4, 4 → 4.2, 6 → 7 and 7 → 8
wf verify [path] [--json] [--budget MS] [--returning-visitor] [--fakes FILE] [--allow-output-outside] # Load every page in headless Chrome: errors, failed requests, paint timing
wf audit [path] [--json]                 # Report what the project trusts: markup, origins, storage, env, CSP, dependencies
wf render TEMPLATE [--data D] [-f html|html-fragment|pdf|slides] [-o OUT] [--page P] [--lang L] [--theme T] [--token NAME=VALUE] # Render a template with JSON data (see Template Engine)
wf registry [--json]                     # List every built-in: props, cases, flags, events, slots, parts
wf types [path] [--json] [--holes]       # List the project's declarations; --holes: where a type is unknown
```

`wf verify` builds nothing — it loads what `wf build` already wrote. It
starts a headless Chrome, serves the output, opens every route the
project has, and fails on an uncaught exception, a `console.error`, a
file that did not arrive, an image that failed, or a page that rendered
no text. It prints the first-contentful-paint, the node count and the
bytes each route fetched, says which built-ins no page drew, and takes
`--budget MS` to fail a route that paints too slowly. It names every
control a pointer aims at that is drawn under 24×24 (WCAG 2.5.8), leaving
out a link inside a line of text and a box its label makes larger — a
warning, never a failure. `--json` is the same report for a pipeline. It needs a Chrome or Chromium on the machine
(or `WF_CHROME` pointing at one) and says so plainly when there is none.

```
  16 route(s) in http://127.0.0.1:39785
    ok   /                                    236ms     613 nodes    447.0 kB   11 req
    ok   /app/deployments                      68ms     657 nodes    360.6 kB   10 req

  16 page(s), 0 problem(s)
```

`wf start` serves what `wf build` wrote, as a host would: on `$PORT`
(else 3000, on every interface), with the headers the build's `_headers`
names (its policy, its frame rule), the `.gz` file beside each text file to
a client that takes gzip, a single-page build's shell for any route a
navigation asks for and a static build's `404.html` for one it never wrote,
and a 404 — never the shell — for a missing script, file or API call.
`--proxy /api=http://localhost:9000/api` forwards every request under
`/api` to another server, the prefix replaced by the URL, so a page calls
its own origin (`api Backend(base: "/api")`) and the API's address is
chosen where the site starts, not when it is built: nothing to add to the
policy, no CORS. An `https://` API is reached over TLS, trusted as this
machine trusts it: its certificate store, plus what `SSL_CERT_FILE` or
`NODE_EXTRA_CA_CERTS` names (a private or development authority).

```json
{ "scripts": { "build": "wf build", "start": "wf start --proxy /api=$API_BASE_URL/api" } }
```

`wf inspect /pokemon/pikachu` opens one page of the build (or a URL of a
server already running) in a headless Chrome and prints it as a reader
meets it, a line a thing: its landmarks, headings, links and where they
go, buttons and their states (`[pressed]`, `[current=page]`), fields and
their values, tables (the first rows), lists, pictures and text — then
anything that threw, logged an error or failed to arrive. `--mobile` lays
it out as a phone, `--screenshot [FILE]` also saves a PNG (`--full-page`
the whole page), `--proxy` serves the build as `wf start` does, and
`--json` is the same for a tool. `--do` plays a step first, as a test
writes it — `--do 'type "pika" into "Search"' --do 'click "Name"'`, then
the page as it stands after; `--do 'expect "Pikachu"'` reports a problem
when the text is not there. It is how to check a page, and how it behaves:
one command, a few hundred words, rather than a browser script.

```
/ — "Dashboard" (en) at 1280×800
  link "Skip to content" → #wf-main
  navigation "Main"
    link "Dashboard" → / [current=page]
    link "Tasks" → /tasks
    button "Log in"
  main
    h1 Dashboard
    "Welcome to your dashboard."
    checkbox "Learn WebFluent"
no problems: nothing threw, logged an error or failed to arrive
```

`wf serve` puts a **stores** button in the corner of every page it serves:
what each store that has been built is holding, a log of every action with
the state on each side of it, and a click to put a store back to before one
ran.

`wf serve` answers a request that is not a page (a call to an API, a file)
and that the build has no file for with a `404` and a line in the terminal —
only a navigation gets the shell. Its builds are development builds: an
error enum's body is held to its declared type, as every build holds a
response (a mismatch is a `.decode` error naming the place, `$[1].name is
missing`); a route no page answers shows a 404 box; a `for` item that throws
shows where (the rest of the list draws — on a deployed page it leaves a
gap); an effect that feeds itself stops after 100 runs with a warning.

`wf verify` visits `:param` routes at their `paths:` values, or at a
placeholder (`/user/1`); fails a page whose router drew nothing into
`<main>`; and with `--returning-visitor` visits every route again with
storage holding what the previous build's pages kept.

`wf serve` builds first, then watches `src/`, `public/` and the config
(`dev.hot_reload`, on by default): a save rebuilds, and every page it
serves carries `/__wf/dev.js`, which reloads the page when a build lands
and draws the error of a build that failed over the page — with the
diagnostics — until the next build passes. `/__wf/status` is what it
polls.

`wf test` renders every `test "name" { … }` declared under `tests/` (or in
`src/`) through the template engine, with the project's components,
stores, types and constants at hand, and holds the render to what it
expects and to a snapshot in `tests/__snapshots__/<file>/<name>.html`
(written when missing, rewritten with `--update`). A build ignores tests.
`expect` reads the text a reader sees, never the markup, laid out as a
browser lays it out — white space collapsed, blocks (and what a `Row`,
`Stack` or `Grid` lays out) apart, inline elements that touch touching, so
`Text { Code("live") Code(": ") }` reads `live:` rendered and in a browser
alike; `t("key")` reads
the project's default locale; `data: { … }` (held to the type of each
state it seeds, `T33`) seeds the body's state and,
by a store's name, the store — in a render and in a browser alike; a
page with no route parameter is placed by its name, as a component is
(`Tasks`, then `click "Add"`), so a test can click through a whole page;
and `wf test tests/cart.wf` runs one file's tests in the project it
belongs to. The `spa` and `static` starters come with tests of their own.

```wf
test "greets by name" {
    Greeting("Sam")
    expect "Hello, Sam"
    expect not "Hello, world"
}
test "lists the data"(data: { items: ["a", "b"] }) {
    state open = true
    for it in items { if open { Text(it) } }
    expect "b"
}
test "reporting an incident opens one more" {
    use IncidentStore
    state draft = ""
    Text("{IncidentStore.open} open")
    Input(bind: draft, label: "What happened")
    Button("Report") { on click { IncidentStore.report(draft) } }

    expect "2 open"
    type "Queue backing up" into "What happened"
    click "Report"
    expect "3 open"
}
```

A test with a step that **acts** is compiled into a real page, served and
run in a headless Chrome (or `WF_CHROME`), because a click runs a handler,
which runs an action, which repaints; so is one that looks at a `Portal`
(in its body or a component it places), whose block only the running page
draws. Its page is drawn as the built one is: the project's scripts, its
stylesheets under `src/`, those `meta.stylesheets` links, and `public/`
served beside it. A test that only looks needs no browser and is compared
to its snapshot as well. The steps run in the order written, everything is found
by the name a reader sees (a button's text, a control's label, placeholder
or `aria-label`) among what a reader can reach — never a control in a
closed dialog, a hidden branch or under `inert`; two a reader could mean
fail the step, saying so — `expect` reads an open dialog (an `ask`) and
the toasts as well as the page, and a handler that throws fails the test.

| Step | What it does |
|---|---|
| `expect "text"` · `expect not "text"` | What the page must, or must not, show |
| `expect focus "Beta"` | What has the focus is called that (its `aria-label`, label, placeholder or text) |
| `click "Save"` | Clicks whatever carries that name, and puts the focus on it as a real click does |
| `type "Ada" into "Name"` | Types into the control that label names |
| `press "Enter"` · `press "Escape" in "Search"` · `press "mod+k"` | A key, on the focused element or a named one, with the modifiers a combination names (`mod` is ⌘ on a Mac, Ctrl elsewhere); `Escape` closes an open `Modal` |
| `expect called Backend.users.create` · `expect not called …` | Whether the page has asked that endpoint |

A page that reads a service is tested against a **fake**: what answers a
group's endpoints in place of the network, held to the group's contract
(`N06`) — every endpoint it answers is one the group has, every name it
binds one of that endpoint's parameters (a default when the call left it
out), every answer the result the endpoint declares. A test names its
fakes — one beside it, or in any file under `tests/` (`tests/fakes.wf`) —
and runs in a browser; nothing is sent, a write's `invalidates` asks
its reads again, and a call to an endpoint no fake answers fails the test,
naming it. A paged endpoint is answered a page at a time, a `streamed` one
with the whole list.

```wf
type User { id: String, name: String }
type NewUser { name: String }
enum CreateUserError {
    unprocessableContent(problem: Problem) on 422
    http(error: HttpError)
}
api Backend(base: "/api") { }
api Backend.users {
    get list(page: Number = 1) at "users" -> [User]!
    post create(body: NewUser) at "users" -> User ! CreateUserError
    invalidates [list]
    delete remove(id: String) at "users/{id}" -> !
}

page Users(path: "/users", title: "Users", description: "Who is here.") {
    state name = ""
    resource users = Backend.users.list()
    Heading("Users").h1
    match users {
        .loaded(list) { for u in list by u.id { Text(u.name) } }
        else { Spinner }
    }
    Input(bind: name, label: "Name")
    Button("Add") { on click { Backend.users.create(body: NewUser(name: name)) } }
}

fake OneUser for Backend.users {
    list(page) { .ok([User(id: "1", name: "Ada")]) }
    create(body) {
        if body.name == "" { .err(.unprocessableContent(Problem(title: "A name is needed"))) } else { .ok(User(id: "2", name: body.name)) }
    }
    remove(id) { .err(.status(500, null)) }
}

test "adds a user"(fakes: [OneUser]) {
    Users
    expect "Ada"
    type "Grace" into "Name"
    click "Add"
    expect called Backend.users.create
}
```

With a contract from an OpenAPI spec, `fake Examples for Backend.users
from examples` answers every endpoint it does not answer itself with the
spec's example of a success, held to the endpoint's result.

`wf verify --fakes tests/fakes.wf` builds the site with that file's fakes
(and the `const`s and types beside them) answering its endpoints, aside
from `build/`, and loads every page of
that — for a site whose API is elsewhere. A build leaves fakes out.

`wf docs` writes `docs/`: a site documenting the project's components
as the official reference documents the built-ins, in its design (the
Blueprint theme, its fonts, light and dark). A page a component: its
`///` comment, its header, every `example` of it drawn live — in a frame
of the project's own build, its theme, stylesheets and scripts, as tall as
what it drew — beside its source, then its props (type, what it is when
left out, each prop's `///`), events, slots, parts, and what places it.
A component with no example is drawn from its props, each one it needs
given a value of its type (`title: String` is `"Title"`). The types, the
stores and the pages have a page each. Serve the folder to read it; a
second run clears what the first wrote, and nothing else.

`example` shows one way to use a component — what `wf docs` draws, live,
beside its source. It names the component, says what it shows, and its
body is a page's: elements, `state`, handlers, the project's stores. It
is checked as a page is (`E101` for a component nothing declares, `T01`
for a wrong value), formatted by `wf fmt`, and left out of every build. A
`///` above it is the words above its frame; `fakes:` answers the
endpoints it calls, as a test's do. An example only draws: a step
(`expect`, `click`) is a test's.

```wf
/// A person, at a glance.
component UserCard(_ name: String, role: String, active: Bool = true) {
    Card { Text(name).bold  Text(role).muted  if active { Badge("Active").success } }
}

/// The usual case.
example UserCard "A designer" {
    UserCard("Sam", role: "Designer")
}

example UserCard "Away" {
    state here = false
    UserCard("Ada", role: "Engineer", active: here)
    Button("Back") { on click { here = true } }
}
```

Keep them beside the component, or in a file of their own under `src/`
(`src/components/UserCard.examples.wf`).


`wf types --json` lists what the project's scripts declare beside its own
declarations — each function's parameters and return as its JSDoc types
them — so a tool knows what a call site may pass to either.

`wf registry --json` and `wf types --json` are the registry and a project's
declarations as a tool reads them — the studio's inspector offers a dropdown
for an enum prop, a toggle for a flag, a handler editor for an event and a
drop target for a slot from these, and a call site of the project's own
`UserCard` reads like one of `Button`.

<!-- core -->
