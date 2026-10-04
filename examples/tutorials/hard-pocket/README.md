# Pocket — the hard tutorial

A personal budget tracker that lives entirely in the reader's browser: money
in, money out, a budget per category, and what is left of it this month. No
server, no account — the ledger is `persist`ed, opens on a sample of five
months of a plausible life, and stays in step across every open tab.

It is the finished project of the third tutorial in the guide's
[Tutorials chapter](../../../md-docs/35-tutorials.md#hard-pocket), which
builds it step by step.

What it teaches:

- the types the language brings with it — `Money`, `Date`, `Color`, `Uuid` —
  in records and an enum, and their methods: `.plus`, `.minus`, `.times`,
  `.startOfMonth()`, `.isSame()`, `format(money)` (`src/types.wf`)
- a store whose `persist`ed list carries a `version:` and a
  `migrate 1 -> 2` step, so a reader of the old shape (signed numbers of
  dollars) keeps their data (`src/stores/Ledger.wf`)
- derived totals all the way down: balance, this month, spent per category,
  six months of totals, averages — and actions used as plain functions
- five routes with a typed `:id` parameter, filters kept in the address
  (`/entries?month=2026-09&category=dining`), a sidebar on a wide screen and
  a drawer on a narrow one chosen with `viewport`
- a form with `validate` blocks, `Form(bind:)`, radios, a `DatePicker`, a
  `Select` of the categories, used for both "new" and "edit"
  (`src/components/EntryForm.wf`)
- a dashboard drawn from plain elements whose `style { height: {…}% }`
  follows the store — the built-in `Chart` only draws data the build can see
- English and Arabic, right-to-left included, with plural forms
  (`t("entries", { count: n })`) and money and dates that follow the locale
- `on key("n")` for a new entry from anywhere, `Escape` to close, a `Modal`
- a `broadcast` channel that tells every tab "imported 3 entries"
- CSV export through a two-line project script (`src/files.js`) and CSV
  import from a `FileUpload`, under the content security policy the build ships
- a service worker (`offline` in the config) and an "update available" banner
- a theme and its night version, and the project's own stylesheet
- tests that look and tests that act (`tests/pocket.wf`)

```bash
wf serve      # http://localhost:3000, rebuilt on save
wf test       # eight tests; four click and type, in a headless Chrome
wf build      # build/, ready for any static host
wf verify     # load every route of build/ in a headless Chrome
```

Open `/?lang=ar` for the Arabic version, or choose it in Settings. Press `N`
anywhere for a new entry. Settings → *Start again* puts the sample ledger back.

The service worker only runs from the built site (`wf serve` removes it on
purpose): serve `build/` with any static server that answers unknown paths
with `index.html`, since every route is the same single page.

## Why the download is a script

Saving the CSV is three lines of DOM work — make a `Blob`, point a link at
it, click the link — which reads best as plain JavaScript. `src/files.js`
declares `saveTextFile(name, text)` with a JSDoc comment, so the `.wf` that
calls it is checked like any other call ([JavaScript
interop](../../../md-docs/32-javascript-interop.md)).
