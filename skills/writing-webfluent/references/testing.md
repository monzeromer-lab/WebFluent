# Testing and verifying

Four commands, each catching what the one before it can't:

| Command | Catches |
|---|---|
| `wf check` | Everything the compiler can see: types, names, props, routes, accessibility, translations, styles |
| `wf test` | Behaviour: what a component or page shows in each state, and what a click or a key does |
| `wf build` | That the project compiles, and with `--stats`, what it weighs |
| `wf verify` | What only a running page shows: a thrown error, a `console.error`, a request that failed, an image that didn't load, a page with no text |

Don't say something works until `wf test` and `wf verify` say so.

## Where tests live

- **Files.** `tests/<feature>.wf`, one file per feature or component.
  Snapshots go to `tests/__snapshots__/<file>/<name>.html` and are
  committed.
- **Names.** A test's name is a sentence about behaviour: "a blank task is
  not added", "a closed issue offers no close button".
- **What the build ignores.** A build ignores tests.

## A test that looks

It places a component (or a page with no route parameter) and says what the
reader must see, and what they must not. It needs no browser.

```wf
/// One order line.
type Line { name: String, qty: Number, price: Money }

/// A line of an order: what, how many, and the total.
component OrderLine(_ line: Line) {
    Row(justify: .between) {
        Text("{line.qty} × {line.name}")
        Text(format(line.price.times(line.qty)))
        if line.qty == 0 { Badge("Removed").warning }
    }
}

test "a line shows its total"(data: { line: { name: "Tea", qty: 3, price: { amount: 450, currency: "USD" } } }) {
    OrderLine(line)
    expect "3 × Tea"
    expect "$13.50"
    expect not "Removed"
}

test "a line of none says it was removed"(data: { line: { name: "Tea", qty: 0, price: { amount: 450, currency: "USD" } } }) {
    OrderLine(line)
    expect "Removed"
}
```

- **`expect` reads text the way a reader does.** White space is collapsed and
  blocks are laid out apart, so `expect "3 × Tea"` matches across inline
  elements. `t("key")` reads the default locale.
- **`data:` seeds values by name, and stores by the store's name**:
  `data: { Cart: { items: […] } }`. Seed with the JSON a server or `persist`
  would hold: a `Money` is `{ amount, currency }`, a `Date` is
  `"2026-03-14"`, an enum case is its name.
- **Snapshots.** Every looking test is also compared to its snapshot. Write
  new ones with `wf test --update`, then **read them** before you commit.

## A test that acts

A test with a step that acts (`click`, `type … into`, `press`) is compiled
into a real page and run in headless Chrome. The steps run in order, and
everything is found by the name a reader sees: a button's text, a control's
label, placeholder or `aria-label`. A step acts only on what a reader can
reach — a control in a closed dialog or a hidden branch is passed over — and
two a reader could mean fail the step, saying so. `expect` reads an open
dialog (an `ask`) and the toasts as well as the page.

```wf
/// The cart: what is in it, and how many.
store Basket {
    state lines: [Line] = []
    derived count = lines.map(l => l.qty).sum()

    action add(name: String) {
        let text = name.trim()
        if text == "" { return }
        lines = [...lines, Line(name: text, qty: 1, price: $4.50)]
    }
}

page Shop(path: "/shop", title: "Shop", description: "Add things to the basket.") {
    use Basket
    state draft = ""
    Heading("Shop").h1
    Input(bind: draft, label: "Item")
    Button("Add to basket") { on click { Basket.add(draft)  draft = "" } }
    Text("{Basket.count} in the basket")
    for line in Basket.lines by line.name { OrderLine(line) }
}

test "an added item is counted" {
    Shop
    expect "0 in the basket"
    type "Tea" into "Item"
    click "Add to basket"
    expect "1 in the basket"
    expect "1 × Tea"
}

test "a blank item is not added" {
    Shop
    type "   " into "Item"
    click "Add to basket"
    expect "0 in the basket"
}

test "a seeded basket counts what it holds"(data: { Basket: { lines: [{ name: "Tea", qty: 2, price: { amount: 450, currency: "USD" } }] } }) {
    Shop
    expect "2 in the basket"
}
```

| Step | What it does |
|---|---|
| `expect "text"` · `expect not "text"` | What the page must, or must not, show |
| `expect focus "Beta"` | What has the focus is called that (its `aria-label`, label, placeholder or text) |
| `click "Save"` | Clicks whatever carries that name, and puts the focus on it |
| `type "Ada" into "Name"` | Types into the control that label names |
| `press "Enter"` · `press "Escape" in "Search"` · `press "mod+k"` | A key on the focused element, or on a named one, with the modifiers a combination names (`mod` is ⌘ on a Mac, Ctrl elsewhere). Enter in a form's field submits it; Escape closes an open `Modal` or `Dialog` |
| `expect called Backend.users.create` · `expect not called …` | Whether the page has asked that endpoint (a test with `fakes:`) |

- **Name things so a test can find them.**
  - A control needs a label, and the page needs it anyway.
  - A row's action is named after its row (`IconButton(icon: "trash", label: "Remove {item.title}")`),
    so `click "Remove Tea"` finds exactly one.
  - A step whose name matches nothing fails and prints what the page showed.
- **Pages and harnesses.**
  - A page with no route parameter is placed by its name, as a component is.
  - For a page with a parameter, build a small harness in the test body:
    `use Store`, a `state`, the controls, the component under test.
- **What the test page includes.** The test page has the project's scripts,
  stylesheets and `public/`, as the built site does.
- **Native code.** A bridge function to native code (Tauri) answers
  harmlessly outside the app, so pages that call it are still testable. See
  [tauri-apps.md](tauri-apps.md).

## A page that reads a service: fakes

A test never reaches the network. A `fake` answers a group's endpoints in
its place, held to the group's contract (`N06`): every endpoint it answers
is one the group has, every name it binds one of the endpoint's parameters,
every answer the result the endpoint declares (`.ok(…)` or `.err(…)`). A
test names its fakes with `fakes:` — declared beside it or in any file
under `tests/` — and runs in Chrome. A write's
`invalidates` asks its reads again, and a call to an endpoint no fake
answers fails the test, naming it.

```wf
/// One member of the team.
type Member { id: String, name: String }
/// What adding a member sends.
type NewMember { name: String }

api Team(base: "/api") { }

api Team.members {
    get list() at "members" -> [Member]!
    post add(body: NewMember) at "members" -> Member!
        invalidates [list]
}

page Members(path: "/members", title: "Members", description: "Who is on the team.") {
    state name = ""
    state failed = ""
    resource members = Team.members.list()

    action add() {
        match await Team.members.add(body: NewMember(name: name)) {
            .ok(_) { name = "" }
            .err(_) { failed = "{name} was not added. Try again." }
        }
    }

    Heading("Members").h1
    match members {
        .loading { Spinner }
        .failed(_) { Alert("The team could not be loaded.").danger }
        .loaded(list) { for m in list by m.id { Text(m.name) } }
    }
    Input(bind: name, label: "Name")
    Button("Add member") { on click { add() } }
    if failed != "" { Alert(failed).danger }
}

fake OneMember for Team.members {
    list() { .ok([Member(id: "1", name: "Ada")]) }
    add(body) { .ok(Member(id: "2", name: body.name)) }
}

fake Down for Team.members {
    list() { .err(.status(503, null)) }
    add(body) { .err(.status(503, null)) }
}

test "the team is listed, and a new member is sent"(fakes: [OneMember]) {
    Members
    expect "Ada"
    type "Grace" into "Name"
    click "Add member"
    expect called Team.members.add
}

test "a server that is down says so"(fakes: [Down]) {
    Members
    expect "The team could not be loaded."
    type "Grace" into "Name"
    click "Add member"
    expect "Grace was not added"
}
```

- **One fake per answer you want to see**: the happy path, an empty list,
  a `422`, a server that is down. Each test names the one it needs.
- **From an OpenAPI spec**, `fake Examples for Backend.users from examples`
  answers every endpoint it doesn't answer itself with the spec's example.
- **A build leaves fakes out.** Keep them under `tests/`.

## What to test

The type checker and the lints already catch a large class of mistakes. Spend
tests on behaviour:
- **Components.** Each branch a prop can take: empty, one, many, error,
  done.
- **Stores.** Test them through what a component shows of them, seeded with
  `data:`.
- **Flows.** What a reader depends on: adding, submitting, filtering,
  removing. Include a blank input and the empty state.
- **Every page** goes through `wf verify`, on every build.

Don't assert on markup, class names or structure. A test reads what a reader
reads, so it survives a restyle.

## Run them

```bash
wf test                    # every test
wf test tests/cart.wf      # one file
wf test --update           # rewrite snapshots after a change you meant
```

A test that acts needs Chrome or Chromium on the machine, or `WF_CHROME`
pointing at one.

## wf verify

```bash
wf build && wf verify
wf verify --budget 1000           # fail a route that paints slower than 1 s
wf verify --returning-visitor     # again, with storage from the last build
wf verify --fakes tests/fakes.wf  # the fakes in that file answer the endpoints
wf verify --json                  # for a pipeline
```

- **What it does.** It serves `build/` and opens every route.
  - A `:param` route is opened at its `paths:` values, or at a placeholder.
  - It fails on an uncaught exception, a `console.error`, a failed request or
    image, or a route that drew nothing in `<main>`.
- **What it lists.** The built-ins no page drew. That is coverage, not a
  failure.
- **A request with no server behind it.** A page that reads a server
  `wf verify` doesn't have fails with a 404. `wf verify --fakes tests/fakes.wf`
  builds the site with that file's fakes answering their endpoints, aside
  from `build/`, and loads every page of that. Never point the `api` at a
  fixture to make it pass.

## In CI

```yaml
- run: curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
- run: echo "$HOME/.webfluent/bin" >> "$GITHUB_PATH"
- run: wf check --deny-warnings --format github
- run: wf fmt --check
- run: wf build
- run: wf test
- run: wf verify --budget 1000
```

GitHub's Ubuntu runners have Chrome. `--format github` turns each finding
into an annotation on the pull request.
