# Build a budget app that works offline

> An advanced WebFluent tutorial: a budget tracker with Money, a persist migration, validation, shortcuts, Arabic and English, and offline support.

The page: https://webfluent.monzeromer.dev/recipes/build-a-budget-app

**What you will build.** A personal budget tracker that lives entirely in
the reader's browser: money in and money out, a budget per category, what
is left of it this month, six months of bars, two languages, a CSV export
and import, and a page that keeps working offline. There is no server and
no account. The finished project is
[`examples/tutorials/hard-pocket`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/hard-pocket).

**Time:** a couple of hours.

## Before you start

You need `wf`, Chrome and a copy of the repository
(see [Before you start](https://webfluent.monzeromer.dev/learn/03-tutorial-todos.md#before-you-start) in the Todos tutorial). This tutorial assumes
the Todos tutorial, and each step links the chapter that goes deeper. Like
Field Notes, it shows the part of each file that teaches something new, so
work in the finished project:

```bash
cd WebFluent/examples/tutorials/hard-pocket
wf serve
```

## Step 1: Configure the project

The project's `webfluent.app.json`:

```json
{
  "name": "pocket",
  "version": "1.0.0",
  "author": "WebFluent tutorials",
  "theme": { "name": "Pocket", "dark": "PocketNight" },
  "build": {
    "output": "./build",
    "output_type": "spa",
    "csp": true
  },
  "meta": {
    "title": "Pocket",
    "description": "A budget that lives in your browser: income, spending and what is left, month by month.",
    "lang": "en"
  },
  "i18n": {
    "default_locale": "en",
    "locales": ["en", "ar"],
    "dir": "src/translations"
  },
  "offline": {
    "precache": ["/", "/entries", "/categories", "/settings"],
    "fallback": "/"
  }
}
```

It adds three things to the Todos config:

- `i18n` names the languages and the folder their messages live in
  ([Internationalization](https://webfluent.monzeromer.dev/recipes.md)).
- `offline` makes `wf build` write a service worker that stores the four
  pages, so a second visit opens with no network ([Offline](https://webfluent.monzeromer.dev/recipes.md)).
- `theme` names two themes, light and dark, and the build ships both.

**Check it:** `wf build` prints a line that starts `Offline: sw.js`, then
`2 locales` in its summary.

## Step 2: Describe money

The ledger's types use four of the types the language brings:

```wf
// src/types.wf
/// Which way the money went.
enum Kind { income, expense }

/// Something money is spent on, with what the reader means to spend on it
/// in a month.
type Category {
    id: String
    name: String
    color: Color
    budget: Money
}

/// One line of the ledger: money in, or money out.
type Entry {
    id: Uuid
    date: Date
    amount: Money
    kind: Kind
    category: String
    note: String = ""
}
```

`Money`, `Date`, `Color` and `Uuid` are each a plain JSON value at run time
([Types](https://webfluent.monzeromer.dev/learn/12-types.md)). Money is kept in minor units, `{ amount: 1299,
currency: "USD" }`, so a sum never drifts by a fraction of a cent. You
write it `$12.99` and show it with `format(m)`. What the language works out
is a call — `a.plus(b)`, `a.minus(b)`, `ONE.times(12.5)` — and what the
value carries is a field: `m.amount`. Two constants give every sum a
starting point:

```wf
// src/types.wf
/// Nothing yet: the starting point every sum adds to.
const ZERO: Money = $0.00

/// One dollar — `ONE.times(12.5)` is $12.50, rounded to the cent.
const ONE: Money = $1.00
```

**Check it:** `wf check` prints `No problems in pocket.` Change `$0.00` to
`$0.001`, and the check refuses it: money has two decimals in dollars.
Change it back.

## Step 3: Add up the ledger

Every figure on every page is a `derived` value of the `Ledger` store:

```wf
// src/stores/Ledger.wf
    derived today = now.date()
    derived newestFirst = entries.sortBy(e => e.date).reverse()

    derived income = sum(entries.filter(e => e.kind == .income))
    derived spent = sum(entries.filter(e => e.kind == .expense))
    derived balance = income.minus(spent)

    derived thisMonth = entries.filter(e => e.date.startOfMonth().isSame(today.startOfMonth()))
    derived monthIncome = sum(thisMonth.filter(e => e.kind == .income))
    derived monthSpent = sum(thisMonth.filter(e => e.kind == .expense))
    derived monthBudget = categories.reduce((s, c) => s.plus(c.budget), ZERO)
    derived budgetLeft = monthBudget.minus(monthSpent)
```

Add an entry, and the balance, this month's spending, each budget bar and
the chart all follow, with no code written to keep them in step
([Stores](https://webfluent.monzeromer.dev/learn/11-stores.md)). `now` is the browser's clock as a value, so
`today` turns over at midnight. `sum` is an action that only returns a
value, which a derived value may call:

```wf
// src/stores/Ledger.wf
    // The money in a list of entries, added up.
    action sum(list: [Entry]) {
        return list.reduce((total, e) => total.plus(e.amount), ZERO)
    }
```

**Check it:** the dashboard shows a balance, this month's income and
spending, and what is left of the budget. Each is a sum of the sample
ledger.

## Step 4: Change the stored shape without losing data

What a `persist` keeps outlives the build that wrote it: a reader who
visited last month has last month's shape in their browser. The entries
declare which version of the shape this build writes, and how to bring an
older one forward:

```wf
// src/stores/Ledger.wf
    // Version 1 of Pocket kept an amount as a plain, signed number of
    // dollars (`-52.4` was money out). Version 2 keeps a `Money` and a
    // `Kind`, so `migrate 1 -> 2` turns what a returning reader's browser
    // holds into the new shape instead of throwing it away.
    persist entries: [Entry] = SAMPLE_ENTRIES.map(s => Entry(
            id: s.id,
            date: now.date().minus(days: s.ago),
            amount: s.amount,
            kind: s.kind,
            category: s.category,
            note: s.note,
        )) {
        version: 2
        migrate 1 -> 2 {
            old.map(e => Entry(
                    id: e.id,
                    date: e.date,
                    amount: ONE.times(Math.abs(e.amount)),
                    kind: if e.amount < 0 { .expense } else { .income },
                    category: e.category,
                    note: e.note ?? "",
                ))
        }
    }
```

- `version: 2` says which shape this build writes.
- `migrate 1 -> 2` brings an older value forward. Its body receives the
  stored value as `old`.
- If you raise the version with no migration, the build warns (`P01`). If
  you change the shape without raising the version, it warns too (`D04`),
  from the shapes it keeps in `.wf-cache/` ([Stores](https://webfluent.monzeromer.dev/learn/11-stores.md)).
- A first visit opens on a sample ledger dated from today, so the
  dashboard is never empty.

**Check it:** `wf check --deny-warnings` passes. Delete the `migrate`
block and check again: the check reports `P01`. Put the block back.

## Step 5: Fit the shell to the screen

The shell draws a `Sidebar` when `viewport.lg` holds, and a top bar with a
drawer when it does not. This is the drawer:

```wf
// src/App.wf
            if Ui.menuOpen {
                Host(tag: "div", class: "scrim") { on click { Ui.closeAll() } }
                Stack(gap: .md, class: "drawer", animate: .slideRight, exit: .fadeOut) {
                    on click { Ui.menuOpen = false }
                    Brand
                    NavLinks
                }
            }
```

- `viewport` is the window's size as a value. It follows `matchMedia`, so
  the switch happens once, as the window crosses the breakpoint
  ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md)).
- The layout itself — grids that are one column on a phone — uses
  responsive values, `Grid(columns: { base: 1, lg: 2 })`, which need no
  script.
- A second store, `Ui`, holds what the interface is doing — the dialog,
  the menu, the notices — apart from what the ledger holds.

**Check it:** on a wide window, the sidebar shows. Narrow the window: a top
bar replaces it, and its menu button opens the drawer, which a click on the
dimmed page behind it closes.

## Step 6: Draw a dashboard from elements

The built-in `Chart` draws at build time, from data the build can see. This
ledger exists only in the reader's browser, so the chart is made of plain
elements whose heights follow the store:

```wf
// src/components/MonthChart.wf
/// Six months of money in and money out, as pairs of bars.
///
/// The built-in `Chart` draws at build time, from data the build can see.
/// This ledger only exists in the reader's browser, so the chart is made of
/// plain elements whose heights follow the store: `height: {share}%` is a
/// style value that reads state, and repaints when it changes.
component MonthChart(months: [MonthTotal], top: Number) {
    Stack(gap: .sm) {
        Row(class: "chart", align: .end, role: "img", aria-label: t("chart.label")) {
            for m in months by m.month {
                Stack(gap: .xs, align: .center, class: "chart__month") {
                    Row(gap: .xs, align: .end, class: "chart__bars") {
                        Host(tag: "span", class: "chart__bar chart__bar--in") {
                            style { height: {m.income.amount / top * 100}% }
                        }
                        Host(tag: "span", class: "chart__bar chart__bar--out") {
                            style { height: {m.spent.amount / top * 100}% }
                        }
                    }
```

- A `{…}` in a style value reads state, and the property follows it
  ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md)).
- The chart is one `role="img"` with a label, so a screen reader hears what
  it shows rather than twelve unnamed boxes
  ([Accessibility](https://webfluent.monzeromer.dev/about/accessibility.md)).

The figures above the chart are one component, placed four times:

```wf
// src/pages/Dashboard.wf
    Grid(columns: { base: 2, xl: 4 }, gap: .md) {
        KpiCard(t("kpi.balance"), value: Ledger.balance, note: t("kpi.balanceNote"))
        KpiCard(t("kpi.income"), value: Ledger.monthIncome, note: t("kpi.thisMonth"), lean: .good)
        KpiCard(t("kpi.spent"), value: Ledger.monthSpent, note: t("kpi.thisMonth"), lean: .bad)
        KpiCard(t("kpi.left"), value: Ledger.budgetLeft, note: t("kpi.leftNote", { budget: format(Ledger.monthBudget) }),
            lean: if Ledger.budgetLeft.amount < 0 { .bad } else { .good })
    }
```

`lean` is an enum prop. A component writes its enum props to its root
element, here as `data-lean="good"`, and `pocket.css` colours the card by
that attribute ([Components](https://webfluent.monzeromer.dev/learn/10-components.md)).

**Check it:** the dashboard shows four cards and six pairs of bars. Add an
expense in step 9's dialog, and this month's bar and the **Spent** card
grow at once.

## Step 7: Keep the filters in the address

The Entries page reads its filters from the address:

```wf
// src/pages/Entries.wf
    // The filters live in the address — `/entries?month=2026-09&category=dining` —
    // so they survive a reload, a shared link and the back button.
    derived month = query.month ?? ""
    derived category = query.category ?? ""
    derived shown = Ledger.newestFirst.filter(e => (month == "" || e.date.slice(0, 7) == month) && (category == "" || e.category == category))
```

Choosing a month calls `navigate("/entries?month=…")`. The address
changes, `query` follows it, and so does the list.

An entry also has a page of its own at `/entries/:id`, where `id` is a
typed parameter. The page looks the entry up with
`Ledger.entries.find(…)` and draws it inside `if let e = entry`. `if let`
binds the value only when it is there, which the checker requires before
the page reads its fields ([Pages and routing](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md)).

**Check it:** on `/entries`, pick a month. The address gains `?month=…`
and the list narrows. Press the browser's back button: the filter goes.

## Step 8: Use one form for new and for edit

`EntryForm` adds an entry or, given one, changes it:

```wf
// src/components/EntryForm.wf
/// The form for a new entry — or, given one, for changing it. It checks what
/// the reader typed, writes it to the ledger, and says so with `saved`.
component EntryForm(entry: Entry? = null) {
    use Ledger
    event saved()

    state kind: Kind = entry?.kind ?? .expense
    state amount: Number? = if let e = entry { e.amount.amount / 100 } else { null }
    state date: Date? = entry?.date ?? Ledger.today
    state category = entry?.category ?? "groceries"
    state note = entry?.note ?? ""

    validate amount {
        required
        min(0.01)
    }
    // Not in the future. With no message of its own, the rule shows the
    // translation of `form.max`; `Ledger.today` is read each time it checks.
    validate date {
        required
        max(Ledger.today)
    }
    validate note { maxLength(60) }
```

- A `validate` block sits beside the state it guards. The control bound to
  that state shows what the rules say — the message, `aria-invalid` and the
  link between them — with nothing more written ([Forms](https://webfluent.monzeromer.dev/learn/14-forms.md)).
- A rule with no message of its own shows the translation of
  `form.<rule>`, so `en.json` and `ar.json` each say "that date has not
  happened yet" once.
- The form says it is done with an event, `saved`. The caller decides what
  that means: close the dialog, or go back to the entry.

The submit button follows the form:

```wf
// src/components/EntryForm.wf
        Row(justify: .end, gap: .sm) {
            Button(if entry != null { t("action.save") } else { t("action.add") }, type: .submit, disabled: !form.valid).primary
        }
```

`Form(bind: form)` hands the component a handle on the form. `form.valid`
is whether every rule holds, and the button follows it as the reader
types.

**Check it:** in the new-entry dialog, type `0` as the amount and leave the
field: it says *Enter an amount above zero*, and **Add entry** stays
disabled. Type `12.50`, and the button turns on.

## Step 9: Add a dialog and a shortcut

The app listens for two keys at the top of `src/App.wf`:

```wf
// src/App.wf
    // `n` opens a new entry from anywhere. A key with no modifier is left to
    // the field the reader is typing in, so an `n` in a note is just a letter.
    on key("n") { Ui.compose() }
    on key("Escape") { Ui.closeAll() }
```

`on key` at the top of the app listens on the document for as long as the
page shows ([Events](https://webfluent.monzeromer.dev/learn/08-events.md)). The dialog is a `Modal` whose `title:`
is its accessible name. It keeps focus inside while it is open, and Escape
closes it:

```wf
// src/App.wf
    Modal(visible: Ui.composing, title: t("compose.title")) {
        if Ui.composing {
            EntryForm {
                on saved {
                    Ui.composing = false
                    Ui.notify(t("toast.added"))
                }
            }
        }
    }
```

**Check it:** press `n` anywhere outside a field, and the new-entry dialog
opens with focus inside it. Press Escape, and it closes. Add an entry, and
the dialog closes and a notice says it was added.

## Step 10: Speak two languages

Every word on screen is `t("key")`, read from `src/translations/en.json`
or `ar.json`. A message with plural forms is several keys:

```json
{
  "entries.zero": "No entries",
  "entries.one": "{count} entry",
  "entries.other": "{count} entries"
}
```

- `t("entries", { count: n })` picks the form the language's rules call
  for. English needs two forms and a zero; Arabic has six
  ([Internationalization](https://webfluent.monzeromer.dev/recipes.md)).
- Switching to Arabic sets `dir="rtl"` on the page, and the sidebar, the
  rows and the grids mirror themselves.
- `format(m)` and dates follow the language too.
- The reader's choice is a `persist` in a small `Prefs` store, and an
  `effect` in the app shell applies it on every visit.

**Check it:** on **Settings**, choose Arabic. The page reads right to
left, and every label is in Arabic. Reload: it stays in Arabic.

If a key shows on screen instead of its words, it is missing from a
translation file; `wf check` reports it as `I01`, and `wf translations`
lists what each locale lacks.

## Step 11: Export and import the data

The Settings page writes the ledger out as a CSV file:

```wf
// src/pages/Settings.wf
    action exportCsv() {
        let lines = Ledger.newestFirst.map(e => csvLine(e))
        saveTextFile("pocket-{Ledger.today}.csv", [CSV_HEADER, ...lines].join("\n"), "text/csv")
    }
```

`saveTextFile` is not WebFluent. It is a function in `src/files.js`, a
plain browser script that the build links on every page. Its JSDoc types
the call, so `saveTextFile(1)` is an error where it is written
([JavaScript interop](https://webfluent.monzeromer.dev/recipes.md)).

Importing reads a file from a `FileUpload` with `await f.text()`, parses
each line and merges what is new. Then it tells every other open tab:

```wf
// src/pages/Settings.wf
        let added = Ledger.merge(parsed)
        news.post({ count: added })
```

`news` is `channel news = broadcast("pocket")`, a channel every tab of the
site hears. The app shell listens on it and shows a notice
([Realtime](https://webfluent.monzeromer.dev/recipes.md)). The ledger itself needs no channel: `persist`
already keeps every tab in step.

**Check it:** on **Settings**, export the ledger, and the browser saves
`pocket-<today's date>.csv`. Open a second tab, import the file in the
first: the second tab shows a notice of how many entries were added.

## Step 12: Test it, and run the checks

A test drives the real form by the labels a reader sees — the English
ones, since a test reads the project's default locale
([Testing](https://webfluent.monzeromer.dev/learn/16-testing.md)):

```wf
// tests/pocket.wf
test "an entry added through the form moves the balance" {
    use Ledger
    Text("Balance {format(Ledger.balance)}")
    EntryForm

    expect "Balance $16,533.56"
    click "Money in"
    type "250" into "Amount"
    click "Add entry"
    expect "Balance $16,783.56"
```

Pocket has eight tests: the cards and rows draw what they should, a blank
amount is refused, `n` opens the dialog and Escape closes it, and removing
an entry takes it out of every total. Run them and the checks:

```bash
wf test
wf check --deny-warnings
wf build
wf verify
```

**Check it:** `wf test` reports `8 passed`, and `wf verify` ends with `0
problem(s)`. `build/` is a single-page app with a service worker, `sw.js`,
beside it. Put it on a host that answers every path with `index.html`
([Deploying](https://webfluent.monzeromer.dev/recipes.md)), and the second visit works with no network.

## What you built

- A ledger typed with `Money`, `Date`, `Color` and `Uuid`
  ([Types](https://webfluent.monzeromer.dev/learn/12-types.md)).
- Totals that are all `derived` values of one store, and a `persist`
  migration that keeps returning readers' data ([Stores](https://webfluent.monzeromer.dev/learn/11-stores.md)).
- A shell that changes with the window, and responsive grids
  ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md)).
- A form that validates itself, a dialog and keyboard shortcuts
  ([Forms](https://webfluent.monzeromer.dev/learn/14-forms.md), [Events](https://webfluent.monzeromer.dev/learn/08-events.md)).
- Two languages, plural forms and right-to-left layout
  ([Internationalization](https://webfluent.monzeromer.dev/recipes.md)).
- A script of your own, and a channel between tabs
  ([JavaScript interop](https://webfluent.monzeromer.dev/recipes.md),
  [Realtime](https://webfluent.monzeromer.dev/recipes.md)).
- A site that works offline ([Offline](https://webfluent.monzeromer.dev/recipes.md)).
