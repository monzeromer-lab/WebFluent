### Expressions

`x => expr` and `(a, b) => expr` are lambdas with one expression as the
body, returning a map with `(x) => { key: value }`; `a ?? b` takes `b` when
`a` is null; `if c { a } else { b }` (with no `else`, `null` when `c` fails), `if let x = v { a } else { b }` and
`match t { .calm { 1 } else { 2 } }` are values; `$token` is a design
token; `.case` is a case of an enum; `a?.b`, `a?.m()`, `a?.[i]` read
through null; `/…/flags` is a regular expression; `...x` spreads a list or
a map; `a..b` and `a..=b` are ranges.

```wf
derived open = incidents.filter(i => !i.resolved)
derived byAge = rows.slice().sort((a, b) => a.age - b.age)
derived label = if let s = selected { s.title } else { "none" }
derived city = user?.address?.city ?? "—"
derived valid = /^[\w.]+@[\w.]+$/.test(email)
derived slug = title.toLowerCase().replace(/[^a-z0-9]+/g, "-")
derived merged = { ...defaults, ...overrides, open: true }
derived all = [...pinned, ...rest]
derived pages = 1..=pageCount
derived byTeam = rows.groupBy(r => r.team)
derived top = rows.sortBy(r => -r.score).take(3)
derived names = rows.map(r => r.name).unique().join(", ")
derived short = description.truncate(80)
```

List helpers: `sortBy(f)`, `groupBy(f)` (a map of lists), `unique()`,
`take(n)`, `first()`, `last()`, `sum()`, `remove(i)` (a copy without the
item at `i`), beside the JavaScript methods (`flatMap`, `findLast`, `at`, …); string helpers: `capitalize()`, `truncate(n)`,
`dedent()` (the common indentation off a block of text), `lines()`,
`words()`, and `match`, `replace`, `replaceAll`, `split`, `search` with a
regex. Every one runs at build time too, so the static paint shows the
result.

A member something declares always wins over a helper of its name: a
store's `action preview(path: String)` or `action first(n: Number)` is called as
`OpenFiles.preview(path)` from a page or another store, and a worker's
`css.last(kind)` is the worker's — never the list or file helper.

