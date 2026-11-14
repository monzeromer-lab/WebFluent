# Testing

> How to write wf tests: render tests with snapshots, seeded stores, tests that click and type in headless Chrome, wf verify, and running them in CI.

The page: https://webfluent.monzeromer.dev/learn/16-testing

This chapter is about testing a WebFluent project: render a component and
check what it shows, click and type through a page in a real browser, and
load every page of the built site to catch errors before a reader does.

You write tests in WebFluent itself, as `test "name" { … }` declarations. A
test renders something — a component, a fragment of a page, a whole page —
and holds it to what it must and must not show. A test that clicks or types
runs in a headless browser. `wf verify` then loads every page of the built
site in one.

## Before you start

- A project with at least one component or page to test.
- For a test that clicks, types or presses a key, and for `wf verify`:
  Chrome or Chromium on the machine, or the `WF_CHROME` environment
  variable pointing at one. A test that only looks needs neither.

## Write a test that looks

To check what a component shows, place it in a test and say what the text
must and must not contain. Put tests in `tests/*.wf`, or beside the code in
`src/`. A test has the project's components, stores, types and constants at
hand.

This file declares a price tag and three tests of it:

```wf
component PriceTag(_ amount: Number, sale: Bool = false) {
    Row(gap: .sm, align: .center) {
        Text(format(amount, .currency)).bold
        if sale { Badge("Sale").danger }
    }
}

test "a price tag shows the amount as currency" {
    PriceTag(1234.5)
    expect "$1,234.50"
    expect not "Sale"
}

test "a sale tag carries the badge" {
    PriceTag(10, sale: true)
    expect "Sale"
}

test "a list renders each row"(data: { rows: [{ name: "Ada" }, { name: "Grace" }] }) {
    for r in rows { Text(r.name) }
    expect "Ada"
    expect "Grace"
}
```

Run `wf test`, and each test prints a line; all three pass. The parts of a
test that looks:

- `expect "text"` — the text must appear in what the page shows.
  `expect not "text"` — it must not.
- `data: { … }` — values the body reads, by name. A `state` in the body
  starts with its initial value.
- `t("key")` — reads the project's translations, in its default locale.

### How `expect` reads the page

`expect` reads the text a reader sees, never the markup: a class name in
the HTML does not count. It reads the text laid out as a browser lays it
out:

- runs of white space are one space;
- a block, or anything a `Row`, `Stack` or `Grid` lays out, is set apart
  from what is beside it;
- inline elements that touch read as touching, so
  `Text { Code("live") Code(": ") Code("[Deploy]") }` reads
  `live: [Deploy]`.

The line breaks and indentation of the snapshot are not text. A render and
a browser read the same page the same way.

In a browser, a page may still be moving — a row on its way out, a fetch on
its way in. `expect` reads the page again until it holds, for up to a
second and a half.

### Snapshots

Each render is also compared to a *snapshot*, the HTML saved in
`tests/__snapshots__/<file>/<test>.html`. `wf test` writes a snapshot when
it is missing and fails, with a diff, when a render no longer matches it.
After a change you meant, run `wf test --update` to rewrite them.

## Write a test that acts

To check what a click, a keystroke or typed text does, add steps that act.
A click cannot be rendered — it runs a handler, which runs an action, which
changes a store, which repaints — so a test with an action in it is
compiled into a real page, served, and run in a headless browser. It uses
the same compiler, the same runtime and the same output a reader gets.

This test types an incident, reports it, and checks the count goes up:

```wf
store IncidentStore {
    state titles: [String] = ["Disk full", "Login slow"]
    derived open = titles.length
    action report(title: String) { titles = titles.concat([title]) }
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

The steps run **in the order you write them**. That order is the test:
what a click did is visible only in the `expect` that follows it.

| Step | What it does |
|---|---|
| `expect "text"` · `expect not "text"` | What the page must, or must not, show |
| `click "Save"` | Clicks whatever carries that name: a button or a link, a control, a tab, a menu's item, an option, a row of a tree |
| `type "Ada" into "Name"` | Types into the control that label names |
| `press "Enter"` · `press "Escape" in "Search"` | A key, on the focused element or a named one; `Escape` closes an open `Modal` or `Dialog` as a real one would |

A step finds what it acts on the way a reader does: by the name on it. A
button's name is its text; a control's is its label, its placeholder or its
`aria-label`. The accessibility checks hold a page to the same names
([Accessibility](https://webfluent.monzeromer.dev/about/accessibility.md)), so a test that passes here is a
page somebody can use.

> [!NOTE]
> **Coming from Testing Library?** There is no `getByTestId` and no
> `waitFor`. A step finds an element only by the name a reader sees, so a
> control with no label cannot be clicked or typed into, and `expect` waits
> on its own, for up to a second and a half. See
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

> [!IMPORTANT]
> **A handler that throws fails the test**, even when every `expect` would
> have passed. A click that quietly did nothing is the bug, not a pass.

A test that acts takes no snapshot: its expects are the test.

### The page a test that acts runs on

The page is drawn as the built site is:

- it links the project's scripts under `src/`, so a click that calls one
  runs it;
- it bundles the project's stylesheets under `src/` after the theme's rules
  and before the rules `style { }` blocks compile to;
- it links the stylesheets `meta.stylesheets` names;
- it serves `public/` beside it.

So a rule that hides a word, or one that names a font or an image under
`public/`, behaves the same in a test as on the site. A test that only
looks renders without any of them.

### Test a portal

A `Portal` draws its block into a node that `into:` names, and only the
running page draws it. So a test whose body holds a portal — or places a
component that holds one — is compiled into a page and run in the browser
too, with no snapshot:

```wf
test "a portal draws into the element a handle names" {
    Stack(ref: spot)
    Portal(into: spot) { Text("in") }
    expect "in"
}
```

## Seed data and stores

To start a test from a known state, name it in `data:`. A key that is a
value's name supplies that value; a key that is a store's name seeds that
store's state. Seeding works the same in a test that only looks and in one
that clicks.

This test fills the cart with two of one item and checks the summary:

```wf
type CartItem { id: String, qty: Number }

store Cart {
    state items: [CartItem] = []
    derived count = items.map(i => i.qty).sum()
}

component CartSummary {
    use Cart
    Text("{Cart.count} items")
}

test "the cart shows what is in it"(data: { Cart: { items: [{ id: "a", qty: 2 }] } }) {
    CartSummary
    expect "2 items"
}
```

## Test a whole page

To click through a page as a reader does, place the page in a test by its
name, as you would a component. This works for a page whose route takes no
parameter. Its state, its stores and its handlers all run:

```wf
test "a task is added with Enter" {
    Tasks
    type "Walk the dog" into "New task"
    press "Enter" in "New task"
    expect "Walk the dog"
}
```

`press "Enter"` in a form's text field submits the form, as a real key
does. The `spa` and `static` projects that `wf init` writes carry tests
like this under `tests/`; run `wf test` in a new project to see them pass.

## Run the tests

To run every test under `tests/`, and any declared in `src/`:

```bash
wf test
```

To run one file's tests, in the project it belongs to:

```bash
wf test tests/cart.wf
```

To rewrite the snapshots after a change you meant:

```bash
wf test --update
```

`wf test` prints a line per test and exits non-zero when one fails, so it
works as a CI step as it is. A build never includes tests.

## Check every page with `wf verify`

To catch what only a running page shows, build the site and load every
page of it:

```bash
wf build && wf verify
```

`wf verify` serves the build and opens every route in headless Chrome. It
fails on:

- an uncaught exception or a `console.error`;
- a request that failed, or an image that did not load;
- a page that rendered no text.

For each route it prints the first contentful paint, the size and the
requests. It also lists the built-ins no page drew — a fact about
coverage, not a failure. Its options:

- `--budget 400` fails a route that takes more than 400 ms to paint;
- `--json` prints the report for a pipeline;
- `--returning-visitor` visits every route again with storage holding what
  the previous build's pages kept.

[CLI](https://webfluent.monzeromer.dev/reference/cli.md#wf-verify) has the details.

## Run the checks in CI

To run the formatter, the build, the tests and `wf verify` on every push,
add a workflow such as this one to `.github/workflows/check.yml`:

```yaml
name: Check
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
      - run: echo "$HOME/.webfluent/bin" >> "$GITHUB_PATH"
      - run: wf fmt --check
      - run: wf build
      - run: wf test
      - run: wf verify --budget 1000
```

GitHub's Ubuntu runners come with Chrome, so the tests that act and
`wf verify` run there as they do on your machine.

## Choose what to test

The type checker and the accessibility lints already catch a large class
of mistakes. Tests are for the behaviour they cannot see:

- **Components** with more than one state: each branch a prop can take.
- **Stores**, through what a component shows of them, seeded with `data:`.
- **Flows** a reader depends on — signing in, adding to a cart, submitting
  a form — as tests that act.
- **Every page**, with `wf verify`, on every build.

## Troubleshooting

**A step cannot find what it names.** The test fails and prints what the
page showed instead:

```text
  FAIL  tests/cart.wf — the count follows what is added
        nothing to click called "Publish"
        the page showed:
        Nothing here
```

Check that the name matches the button's text or the control's label,
placeholder or `aria-label` exactly, and that an earlier step has drawn it.

**A test that acts says there is no browser.** Install Chrome or Chromium,
or set `WF_CHROME` to the path of one.

**A snapshot test fails after a change you meant.** Run `wf test --update`
and commit the new snapshots.

## Common mistakes

A step finds what it acts on by its accessible name, so the mistakes that
break a test are the ones the accessibility lints already report.

**A field with no label.** `type "…" into "…"` names a control by its label,
placeholder or `aria-label`, and this one has none:

```wf expect=A03
page Tasks(path: "/", title: "Tasks", description: "What to do.") {
    state draft = ""
    Heading("Tasks").h1
    Input(bind: draft)
    Text("Next: {draft}")
}
```

Give it `label: "New task"`, and the test types `into "New task"`
([A03](https://webfluent.monzeromer.dev/errors/A03.md)).

**A button with no text.** `click "…"` finds a button by its text, and an
empty one has nothing to find:

```wf expect=A05
page Tasks(path: "/", title: "Tasks", description: "What to do.") {
    state count = 0
    Heading("Tasks").h1
    Text("{count} done")
    Button { on click { count = count + 1 } }
}
```

Write its text, `Button("Mark done")`, or give it an `aria-label:`
([A05](https://webfluent.monzeromer.dev/errors/A05.md)).

**An icon button with no label.** An `IconButton` shows no text, so its
`label:` is the only name a step can use:

```wf expect=A02
page Tasks(path: "/", title: "Tasks", description: "What to do.") {
    state count = 0
    Heading("Tasks").h1
    Text("{count} done")
    IconButton(icon: "plus") { on click { count = count + 1 } }
}
```

Add `label: "Add one"`, and click it with `click "Add one"`
([A02](https://webfluent.monzeromer.dev/errors/A02.md)).

Going further: [Security](https://webfluent.monzeromer.dev/about/security.md) covers what the compiler
guarantees, what it cannot, and the decisions left to you: markup,
secrets, storage, headers and a deployment checklist.

## Next

[PDF and slides](https://webfluent.monzeromer.dev/learn/17-pdf-and-slides.md).
