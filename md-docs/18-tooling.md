# 18. Tooling

One binary, `wf`, does everything: scaffolds, builds, serves, formats,
tests, renders and describes. A language server, `wf-lsp`, brings the same
checks into the editor.

## The commands

| Command | Does |
|---|---|
| `wf init NAME [-t spa\|static\|pdf\|slides]` | Creates a project from a template |
| `wf build [-d DIR] [--stats]` | Compiles `src/` to the output directory; fails on errors, prints warnings. `--stats` reports what it weighs |
| `wf serve [-d DIR]` | Builds, serves on `dev.port` (3000), rebuilds on every save and reloads the browser |
| `wf fmt [PATH] [--check] [--stdout] [--to wf\|wfx]` | Formats source files; converts layouts |
| `wf test [PATH] [--update]` | Runs `test "…" { }` declarations against their expectations and snapshots |
| `wf generate page\|component\|store NAME` | Adds a file with a starter declaration |
| `wf render FILE --data JSON [-f html\|html-fragment\|pdf\|slides] [-o OUT]` | Renders a template with data ([chapter 17](17-outputs.md#templates-rendering-with-data-on-a-server)) |
| `wf migrate [PATH] [--check] [--stdout] [--wfx]` | Rewrites WebFluent 2 sources to 3, then carries the project to 4 |
| `wf docs [-o DIR]` | Writes a component gallery — every built-in and everything the project declares |
| `wf registry [--json]` | Describes every built-in: props, cases, flags, events, slots, parts |
| `wf types [--json]` | Describes what the project declares: enums, types, components, stores, pages, externals |
| `wf audit [PATH] [--json]` | What the project trusts: markup, other origins, storage, `env`, the policy, the dependencies |
| `wf verify [PATH] [--json] [--budget MS]` | Loads every page in a real browser: errors, failed requests, paint timing |

## `wf verify`

A build says what the compiler wrote. This says what a browser does with
it.

```bash
wf build && wf verify
```

It starts a headless Chrome, serves the output, and opens every route the
project has. A route fails on an uncaught exception, a `console.error`, a
file that did not arrive, an image that failed to load, or a page that
rendered no text — the things a compiler cannot see and a reader always
can.

```
  16 route(s) in http://127.0.0.1:39785
    ok   /                                    236ms     613 nodes    447.0 kB   11 req
    ok   /app/deployments                      68ms     657 nodes    360.6 kB   10 req

  16 page(s), 0 problem(s)
```

The numbers are the first contentful paint, the elements in the document
once it settled, the bytes the page fetched and the requests it made.
`--budget 400` fails a route that takes longer than that to paint.
`--json` is the same report for a pipeline.

It also says **which built-ins no page drew**. A component can be in the
registry, in the tests and in the documentation and still be broken in a
page; the classes the pages carried are the list of what actually ran, so
the difference is what nothing has exercised. One project is not expected
to draw them all — it is a fact about coverage, not a failure.

It needs a Chrome or Chromium on the machine, or `WF_CHROME` pointing at
one, and says so plainly when there is none.

## `wf migrate`

Two things, in one command.

**2 → 3** rewrites every `.wf` under `src/` in place, with a note for
anything that needed a decision. It is a change of spelling: the migrated
project builds to what it built before.

**3 → 4** is a change of what the compiler allows, so it runs over the
project rather than the files. It adds every `env` name your pages already
read to `public_env` — preserving what the project did, and printing the
list, because *public* means anyone who opens the site may read the value.
Then it names, with file and line, everything 4 refuses that 3 allowed: an
`on*` attribute, a `javascript:` or `data:` URL. Those have no automatic
rewrite; the old value was a string of JavaScript or a link that ran one.
Finally it states the changes that need no edit: a store is built on first
read rather than at boot, a `persist` value follows the site's other tabs,
and `WF.store`/`WF.host` were renamed.

`--check` writes nothing and tells you what it would do.

## `wf serve`

The development loop. It builds once, serves the output, watches `src/`,
`public/` and the config, rebuilds on a change and reloads every open tab.
A build that fails leaves the last good page up and draws the error over
it — file, line, message and hint — until the next save fixes it. The
overlay is served at `/__wf/dev.js`, the build's state at `/__wf/status`
(JSON), for anything else that wants to know.

## `wf build`

Runs the whole pipeline: parse → semantic checks → type checks → linters →
codegen. Errors stop the build with the file, line, column, message and a
hint; warnings print and the build goes on. The exit code is non-zero on
an error, so a CI step is just `wf build`.

```bash
wf build && rsync -a dist/ user@host:/var/www/site/
```

`--stats` prints what the build weighs — every text file with its gzipped
size, and which of the runtime's feature modules it carries. The runtime a
page ships is the part of it the program reaches: `each` for a `for`,
`router` for a second page, `net` for a `resource`, and, in the icon table,
only the glyphs the program names.

```bash
wf build --stats
```

```text
  What it weighs
    app.js                                15.8 kB  (5.3 kB gzipped)
    styles.css                            19.0 kB  (4.1 kB gzipped)
    total                                 36.3 kB

  Runtime: 4 of 28 modules, 24.9 kB of 107.2 kB (before minifying)
    kept     core pages store icons
    left out motion helpers format form text when match slot each show …
```

A name the build cannot see — an icon named by an API response, or a
hand-written `<script>` that calls `WF` — is the one case this gets wrong;
`"build": { "runtime": "full" }` ships every module.

Each build records what it weighed in `.wf-sizes.json` beside the project,
so the next `--stats` shows what moved. `build.budget` names a gzipped size
an output should stay under; going over prints a warning, never an error:

```json
{ "build": { "budget": { "app.js": "40 kB", "styles.css": "30 kB" } } }
```

## Diagnostics

Every finding carries a code. The families:

Every finding carries its code, the file and the line, and a hint. The
code is the thing to search for, so each one is listed below — none of
them is a range you have to guess your way inside.

**Errors stop a build. Warnings do not**, and nothing is a warning that
the compiler could have decided for itself.

### Types — `T01`–`T12`

Anything the checker cannot resolve is `Any`, which agrees with
everything, so a program that declares no types raises none of these.
Every annotation you add narrows what it can say. Fully covered in
[chapter 10](10-types.md#the-checks).

| Code | What it means |
|---|---|
| `T01` | A value of the wrong type — given to a prop, a state, a field, a parameter or an assignment |
| `T02` | A case the enum does not have |
| `T04` | A value that may be `null`, read as if it were not. `if let`, `??` or a `!= null` check narrows it |
| `T05` | A field or method a record does not have |
| `T06` | A member an object does not have: a store's, or a service's endpoint |
| `T07` | A list, record or action used as a condition, which is always true |
| `T08` | A `for` over something that is not a list |
| `T09` | An `emit` whose arguments do not match the event it names |
| `T10` | A call with the wrong number or kind of arguments |
| `T11` | A `match` on something that is neither a resource nor an enum, or with arms of the wrong kind |
| `T12` | A `Secret` where it would escape — shown, spliced into text, logged, or kept with `persist` |

Two more errors have no code because they cannot be anything else: an
`on*` attribute (script written as an attribute) and an `href`, `src` or
`to` literal naming a scheme a browser runs. Both are covered in
[chapter 19](19-security.md).

### Accessibility — `A01`–`A15`

Warnings. A control is named by `label:`, or by `aria-label:` /
`aria-labelledby:` where its visible label is a separate element. The
rules behind them are in
[spec/ACCESSIBILITY_SPEC.md](../spec/ACCESSIBILITY_SPEC.md).

| Code | What it means | What to do |
|---|---|---|
| `A01` | An `Image` with no `alt` | `Image(src: "…", alt: "What it shows")`. A decorative image takes `alt: ""` |
| `A02` | An `IconButton` with no accessible name | `IconButton(icon: "close", label: "Close")` — the label is never visible text |
| `A03` | An `Input` with neither `label:` nor `placeholder:` | `Input(label: "Username").text`. A placeholder alone is a weaker answer, and disappears as you type |
| `A04` | A `Checkbox`, `Radio`, `Switch`, `Slider` or `Textarea` with no `label:` | `Checkbox(bind: agreed, label: "I agree")` |
| `A05` | A `Button` with no text | Give it text, or an `aria-label:` when its content is an icon |
| `A06` | A `Link` with no text | Same. "Read more" ×12 on a page is legal and unhelpful; name the destination |
| `A07` | A `Heading` with no text | An empty heading breaks the outline a screen reader navigates by |
| `A08` | A `Modal` or `Dialog` with no `title:` | `Modal(visible: open, title: "Confirm")` — it is the dialog's accessible name |
| `A09` | A `Video` with no `controls`, or a `Video`/`Audio` with no `captions:`/`transcript:` | `Video(src: "…", captions: "/clip.en.vtt").controls`. A video nobody can hear is a video nobody can follow |
| `A10` | A `Table` with no header row | Wrap the first row in `Table.Head`, so each cell is a `<th scope="col">` |
| `A11` | A heading level skipped — `h2` straight to `h4` | The outline is how a page is navigated, not how big the text is; style with `style { font-size }` |
| `A12` | A page with no `h1` | Every page needs one, and one is usually enough |
| `A13` | A colour pairing in the theme below the WCAG AA contrast ratio | Only checkable because a `theme` is parsed source. Adjust the token |
| `A14` | A `role:` that requires particular children holding something else — `tablist` → `tab`, `list` → `listitem`, `menu` → `menuitem` | Use the built-in that carries the role, or correct the children |
| `A15` | A control whose `aria-label` does not contain its visible text | What a voice-control user says is what they see. Make the label start with the visible words |

`A11` and `A12` do not apply to `Presentation` or `Document` output,
where an `h1` per slide or per section is correct.

### Search and sharing — `S01`–`S04`

Warnings. [Chapter 3](03-pages-and-routing.md) covers what a page
declares.

| Code | What it means |
|---|---|
| `S01` | A page with no `title:` — it has nothing to show as a search result's link, or in the tab |
| `S02` | A page with no `description:`, so the search snippet is written for you out of the page's text |
| `S03` | A description over about 160 characters, which a search result truncates mid-sentence |
| `S04` | Two pages claiming the same route. Only one of them will ever render |

### What is kept in the browser — `P01`–`P03`

Warnings. [Chapter 9](09-stores.md#keeping-a-value-across-visits) covers
the policy.

| Code | What it means |
|---|---|
| `P01` | A `persist` at `version: n` with no `migrate` step bringing some older version forward. A reader who last visited then loses what they had |
| `P02` | A `migrate` step above the declared `version:`, or with no `version:` at all — it never runs |
| `P03` | A `persist` inside a `store(scope: .route)`. The route change drops the store and the next read builds it again from storage, so the value comes straight back |

### Declared and never read — `U01`–`U05`

Warnings. A name beginning `_` is understood to be unused on purpose and
raises none of these.

| Code | What it means |
|---|---|
| `U01` | A `state` nothing reads. Assigning to it is not reading it |
| `U02` | A `derived` nothing reads |
| `U03` | A component nothing places, names as a `layout:`, or reaches as a part |
| `U04` | A store member — state, derived or action — nothing reads, inside the store or as `Store.member` |
| `U05` | An action nothing calls |

### Vocabulary — `V01`–`V03`

Warnings.

| Code | What it means |
|---|---|
| `V01` | A bare word in argument position that nothing in scope declares — a misspelled name, or a flag written without its dot (`center` where `.center` was meant) |
| `V02` | A flag whose class no stylesheet defines, yours or the engine's |
| `V03` | An `Unsafe.Html`, and whether what it puts in went through `sanitize`. Not a mistake — the one place markup enters, named so a review finds every one |

### Structure

Errors, and they have no codes because each is a reference to something
that does not exist: an unknown component, prop, flag, case, part, event
or slot; a fill for a slot the component does not declare; two positional
arguments; a `layout:` naming a component with no default slot; two pages
or two components with one name; an `emit` of an undeclared event. Each
names what it takes instead.

An `env` name that has not been made public, read from a page, is also an
error — see [chapter 19](19-security.md#env-and-what-ends-up-in-the-bundle).

## `wf fmt`

Formats every `.wf` and `.wfx` under a path (or one file) to the canonical
layout: four-space indents, one statement per line, clause order `style`
→ `transition` → `on` → fills → children, spaces after commas and around
operators. `--check` fails without writing, for CI; `--stdout` prints one
file. `--to wfx` rewrites a `.wf` project in the indented layout (and
renames the files), `--to wf` the other way; both are lossless.

## `wf test`

A test declaration renders a body with the project's components, stores,
types and constants at hand, and holds the result to what it expects:

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

test "a list renders each row"(data: { rows: [{ name: "a" }, { name: "b" }] }) {
    for r in rows { Text(r.name) }
    expect "a"
    expect "b"
}
```

- Tests live in `tests/*.wf` (or beside the code in `src/`).
- `expect "text"` must appear in what the page shows; `expect not "text"`
  must not.
- `data: { … }` supplies values by name; a `state` in the body is seeded
  with its initial value.
- Each render is also compared to `tests/__snapshots__/<file>/<test>.html`,
  written when missing; `--update` rewrites them after an intended change.

`wf test` exits non-zero on a failure, with a diff.

### A test that acts

Everything above only **looks**: the body is rendered and the expects read
the result. A click cannot be rendered — it runs a handler, which runs an
action, which changes a store, which repaints — so a test with an
interaction in it is compiled into a real page, served, and run in a
headless browser. Same compiler, same runtime, same output a reader gets.

```wf
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

| Step | What it does |
|---|---|
| `expect "text"` · `expect not "text"` | What the page must, or must not, show |
| `click "Save"` | Clicks whatever carries that name |
| `type "Ada" into "Name"` | Types into the control that label names |
| `press "Enter"` · `press "Escape" in "Search"` | A key, on the focused element or a named one |

The steps run **in the order written** — that is the whole meaning, since
what a click did is only visible in the expect that follows it.

Everything is found the way a reader finds it: by the name on it. A
button is its text; a control is its label, its placeholder or its
`aria-label`. That is the same name the accessibility checks hold a page
to, so a test that passes here is a page somebody can use — and a test
that cannot find what it asks for says so:

```
  FAIL  tests/cart.wf — the count follows what is added
        nothing to click called "Publish"
        the page showed:
        Nothing here
```

**A handler that throws fails the test**, even where the expects would
have passed — a click that quietly did nothing is the bug, not a pass.
These tests need a Chrome or Chromium on the machine, or `WF_CHROME`
pointing at one; a test that only looks needs neither.

A test that acts takes no snapshot. What it expects is the test.

## `wf generate`

```bash
wf generate page Pricing
```

writes `src/pages/Pricing.wf` with a page declaring `path: "/pricing"`, a
title and an `h1`; `component` and `store` likewise. It is
a starting point, not a framework — edit freely.

## `wf docs`, `wf registry`, `wf types`

`wf docs` writes `docs/index.html`: a gallery of every built-in with its
props, cases, flags, events, slots and parts, and every component, enum,
type, store and page the project declares, with their docs (`///`
comments). `wf registry --json` and `wf types --json` print the same data
for tools — a design tool, a generator, a script:

```bash
wf registry --json | jq '.components[] | select(.name == "Button") | .props[].name'
```

The registry is the compiler's own table: what it prints is exactly what
the checker accepts.

## Editors

`wf-lsp` is the language server, built from `crates/wf-lsp`
(`cargo install --path crates/wf-lsp`, or the binary from a release). It
gives:

- Diagnostics as you type — every check `wf build` runs.
- Completion: components and their props after `(`, flags and parts after
  `.`, enum cases after `:`, events after `on `, `$` tokens, `match` arms,
  slot names, fields of a record, methods of a string or a list.
- Hover: a component's doc and signature, a name's inferred type.
- Go to definition, find references, rename across the project.
- Document symbols and the outline.
- Quick fixes — "Change to `…`" on a misspelt name — and `Extract
  component` on a selection.

**Zed**: the extension is in `editors/zed`; install it with
`zed: install dev extension` pointing at that folder, or from the
extension registry. It brings Tree-sitter highlighting, outline, brackets,
indentation, snippets and the language server (found on `PATH` or
downloaded from the latest release), for `.wf` and `.wfx`.

**VS Code**: `editors/vscode` — TextMate highlighting and the language
server. Open the folder and run `Extensions: Install from VSIX` on the
packaged extension, or symlink it into `~/.vscode/extensions`.

Any other editor with LSP support runs `wf-lsp` over stdio.

## Project layout, as a reminder

```
webfluent.app.json    config
src/                  .wf / .wfx / .md pages, components, stores, themes, .css, translations/
public/               copied to the output root
tests/                wf test declarations and __snapshots__/
dist/                 the build (build.output)
```

## Next

[Security](19-security.md), then the
[components reference](20-components-reference.md) and the
[cookbook](21-cookbook.md).
