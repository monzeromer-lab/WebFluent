---
kind: "recipe"
route: "/recipes/test-a-page"
group: "testing"
description: "Write tests under tests/ that render a component or a page, type and click through it like a reader, and check what it shows; run them with wf test."
outputs: ["website", "app", "desktop"]
---
# Test a page

A test renders a component or a whole page, acts on it the way a reader
does — typing, clicking, pressing keys — and checks the text the page
shows. This shopping list has a component that counts, a page that adds
items, and three tests:

**Needs:** a `tests/` folder beside `src/`, and a Chrome or Chromium on
the machine for the tests that click (or `WF_CHROME` naming one). The block
holds three files; the comment above each says where it goes.

```wf
// src/components/Count.wf
component Count(_ n: Number) {
    Text(if n == 1 { "1 thing to buy" } else { "{n} things to buy" })
}

// src/pages/Shopping.wf
page Shopping(path: "/shopping", title: "Shopping", description: "What to buy.") {
    state items: [String] = ["Milk"]
    state draft = ""

    Heading("Shopping").h1
    Count(items.length)
    Form {
        on submit(e) {
            e.preventDefault()
            if draft != "" { items = items.concat([draft]) }
            draft = ""
        }
        Input(bind: draft, label: "Item")
        Button("Add", type: .submit).primary
    }
    for item in items by item { Text(item) }
}

// tests/shopping.wf
test "one thing reads in the singular" {
    Count(1)
    expect "1 thing to buy"
    expect not "things"
}

test "adding an item lists it and counts it" {
    Shopping
    expect "1 thing to buy"
    type "Bread" into "Item"
    click "Add"
    expect "Bread"
    expect "2 things to buy"
}

test "an empty item is not added" {
    Shopping
    click "Add"
    expect "1 thing to buy"
}
```

Run them:

```bash
wf test
```

```text
  new   tests/shopping.wf — one thing reads in the singular (snapshot written)
  ok    tests/shopping.wf — adding an item lists it and counts it (in a browser)
  ok    tests/shopping.wf — an empty item is not added (in a browser)
3 passed, 0 failed, 1 snapshot(s) written
```

## How it works

- `wf test` runs every `test "…" { }` under `tests/`. A build ignores
  them, so they never reach the site.
- A test's body is drawn like a page's: `Count(1)` places the component,
  and `Shopping`, a page with no route parameter, is placed by its name.
  The project's components, stores, types and constants are all in reach.
- `expect "…"` checks the text a reader would see, with white space
  collapsed the way a browser lays it out, never the markup.
  `expect not "…"` checks that it is absent.
- A test with a step that acts — `click`, `type … into`, `press` — is
  built into a real page and run in headless Chrome, so the handler runs,
  the state changes and the page draws again. `type "Bread" into "Item"`
  finds the field by its label, and `click "Add"` the button by its text:
  by what a reader sees, so a test also catches a control that lost its
  name.
- A test that only looks needs no browser. It is rendered and compared
  with a snapshot, `tests/__snapshots__/shopping/<test name>.html`, written
  the first time it runs. When a change is meant, `wf test --update`
  writes the snapshots again; commit them with the code.
- A handler that throws fails the test, so a broken click is caught even
  when the text happens to match.

## Variations

- **Seed the state**: `test "…"(data: { items: ["a", "b"] }) { … }` starts
  the body's state, or a store by its name, with that data.
- **Close a dialog**: `press "Escape"` closes the open `Modal` or `Dialog`;
  `press "Enter" in "Search"` presses a key in one named field.
- **One file**: `wf test tests/shopping.wf` runs only that file's tests.
- **Every page in a browser**: `wf verify` loads each route of the built
  site and fails on a script error, a failed request or a page that drew
  nothing ([Testing](../learn/16-testing.md#check-every-page-with-wf-verify)).
- **In CI**: run `wf check`, `wf test` and `wf build` in that order
  ([Install wf on a build machine](install-wf-in-ci.md)).

A key step names one key, as the browser names it (`"Enter"`, `"Escape"`,
`"a"`); it does not hold a modifier, so a shortcut such as Ctrl+K is
checked by calling what it runs.
