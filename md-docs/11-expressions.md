# 11. Expressions

An expression is a value: a prop's value, a condition, the right side of a
`state` or `derived`, the inside of `{…}` in a string. This chapter is the
full expression language.

## Literals

```wf
page Literals(path: "/") {
    state s = "text"                        // String
    state n = 42                            // Number
    state f = 3.14                          // Number
    state b = true                          // Bool
    state nothing: String? = null           // null
    state list = [1, 2, 3]                  // [Number]
    state map = { name: "Sam", age: 30 }    // a shape with those fields
    state re = /^[a-z]+$/i                  // a regex
    state color = $primary                  // a design token
    Text("{s} {n} {f} {b} {list.length} {map.name}")
}
```

Strings are double-quoted; `\"`, `\\`, `\n` and `\t` escape. Lists and map
literals may span lines and take a trailing comma.

## Interpolation

`"{expr}"` puts a value into a string; anything that shows text accepts one:

```wf
page Interp(path: "/") {
    state name = "Sam"
    state n = 3
    derived items = if n == 1 { "item" } else { "items" }
    Text("Hello, {name}: {n} {items}")
    Text("Total: {format(n * 9.99, .currency)}")
    Text("Next: {n + 1}")
}
```

Anything that is an expression goes inside `{…}` — a call, a method chain,
an `if`, even a string of its own: `"{n ?? "none"}"` and
`"{if ok { "yes" } else { "no" }}"` both work, as long as the group closes
on the same line. A brace group that is not an expression (`"{key: value}"`
in prose, a fragment of code) stays text. For anything long, a `derived`
reads better, as `items` above.

## Operators

| Kind | Operators |
|---|---|
| Arithmetic | `+ - * / %`, unary `-` |
| Comparison | `== != < <= > >=` |
| Logic | `&& \|\| !` |
| Null | `??` fallback, `?.` optional chain |
| Range | `a..b` (exclusive), `a..=b` (inclusive) |
| Spread | `[...a, x]`, `{ ...m, key: v }` |
| Grouping | `( )` |

`+` joins strings when either side is a `String`, and adds numbers otherwise.
`==` compares values; two records with the same fields are not `==` unless
they are the same object, so compare ids. Precedence is the usual: `!` and
unary `-`, then `* / %`, `+ -`, comparisons, `&&`, `||`, `??`.

```wf
page Ops(path: "/") {
    state a = 7
    state b = 2
    state user: Map? = null
    Text("{a + b} {a - b} {a * b} {a / b} {a % b}")
    Text(if a > b && b != 0 { "a wins" } else { "b wins" })
    Text(user?.name ?? "guest")
    for n in 1..=3 { Badge("{n}") }
}
```

## Choosing a value

`if` and `match` are expressions when they stand where a value is expected;
each arm's block is its value:

```wf
enum Level { low, mid, high }

page Choose(path: "/") {
    state score = 72
    state level: Level = .mid
    derived grade = if score >= 90 { "A" } else if score >= 70 { "B" } else { "C" }
    derived tone = match level { .low { "calm" } .mid { "steady" } else { "loud" } }
    Text("{grade} {tone}")
    Badge(grade, tone: if score >= 70 { .success } else { .danger })
}
```

A `match` expression always ends with an `else` arm, so it has a value
whatever comes; a `match` statement may list only the arms it shows.

## Fields, calls and methods

```wf
type Todo { id: String, title: String, done: Bool = false }

page Calls(path: "/") {
    state todos: [Todo] = [Todo(id: "1", title: "milk"), Todo(id: "2", title: "eggs", done: true)]
    state q = "M"
    derived open = todos.filter(t => !t.done)
    derived titles = todos.map(t => t.title.capitalize())
    derived hits = todos.filter(t => t.title.toLowerCase().includes(q.toLowerCase()))
    derived first = todos.first()
    derived count = todos.length
    derived summary = titles.join(", ")
    Text("{count} todos, {open.length} open, {hits.length} match: {summary}")
    if let f = first { Text("First: {f.title}") }
}
```

Lambdas are `x => expr`, `(a, b) => expr` or `() => expr` — an expression
body, which may call an action when it needs to do more; a parameter's
type comes from what the method expects.

### String methods

`length`, `trim()`, `toLowerCase()`, `toUpperCase()`, `capitalize()`,
`truncate(n)`, `includes(s)`, `startsWith(s)`, `endsWith(s)`, `indexOf(s)`,
`slice(a, b)`, `substring(a, b)`, `charAt(i)`, `replace(a, b)`, `split(sep)`,
`padStart(n, c)`, `padEnd(n, c)`, `repeat(n)`, `match(re)`.

### List methods

`length`, `map(f)`, `filter(f)`, `find(f)`, `findIndex(f)`, `some(f)`,
`every(f)`, `includes(x)`, `indexOf(x)`, `reduce(f, init)`, `join(sep)`,
`slice(a, b)`, `concat(other)`, `reverse()`, `sort(f)`, `sortBy(f)`,
`groupBy(f)`, `unique()`, `take(n)`, `first()`, `last()`, `sum()`,
`flatMap(f)`, and, in an imperative block, `push(x)`.

`sortBy`, `groupBy`, `unique`, `take`, `first`, `last`, `sum`, `capitalize`
and `truncate` are WebFluent's own; the rest are the browser's.

### Number methods

`toFixed(n)`, `toString()`. `Number("12")` converts a string; `"{n}"` or
`String(n)` converts a number. `Math.round`, `Math.max` and friends are
available as `Math.*`.

## `format` and `ago`

`format(value, style, option)` formats numbers and dates in the reader's
locale (the `locale` of the app, or the browser's):

```wf
page Format(path: "/") {
    state price = 1234.5
    state ratio = 0.256
    state big = 1200000
    state when = "2026-03-14T09:30:00"
    Text(format(price, .currency))                 // $1,234.50
    Text(format(price, .currency, "EUR"))          // €1,234.50
    Text(format(ratio, .percent))                  // 26%
    Text(format(ratio, .percent, 1))               // 25.6%
    Text(format(price, .decimal, 1))               // 1,234.5
    Text(format(big, .integer))                    // 1,200,000
    Text(format(big, .compact))                    // 1.2M
    Text(format(when, .date))                      // Mar 14, 2026
    Text(format(when, .date, "long"))              // March 14, 2026
    Text(format(when, .time))                      // 9:30 AM
    Text(format(when, .datetime))                  // Mar 14, 2026, 9:30 AM
    Text(format(when, "EEEE d MMMM yyyy, HH:mm"))  // Saturday 14 March 2026, 09:30
    Text(ago(when))                                // 3 days ago
    Text(format(when, .relative))                  // the same
}
```

Styles: `.integer`, `.decimal(places)`, `.currency(code)`, `.percent(places)`,
`.compact`, `.date(short|medium|long|full)`, `.time`, `.datetime`,
`.relative`, or a pattern string of `yyyy yy MMMM MMM MM M dd d EEEE EEE HH
H hh h mm ss a`. `ago(date)` gives "just now", "5 minutes ago", "in 2
days". The static paint formats with a built-in table for the common
locales and currencies; the live page uses `Intl` and corrects any
difference.

## Regular expressions

```wf
page Regex(path: "/") {
    state email = ""
    derived valid = /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email)
    derived domain = email.split("@").last() ?? ""
    Input(bind: email, label: "Email")
    if email != "" && !valid { Text("That does not look like an email.").danger.sm }
    Text("Domain: {domain}")
}
```

`/…/flags` is a regex; `re.test(s)` is a `Bool`, `s.match(re)` and
`re.exec(s)` a `[String]?`, `s.replace(re, "x")` a `String`.

## Tokens as values

`$name` names a design token from the theme ([chapter 12](12-styling.md)).
In a style it becomes the CSS variable; as a value it is the variable
reference, so a prop that takes a color takes a token:

```wf
page Tokens(path: "/") {
    state accent = $primary
    Card {
        style { border-left: 4px solid {accent} }
        Text("Accent from a token")
    }
}
```

## `await`

`await` waits for a promise in an action or handler, and makes the action
asynchronous:

```wf
page Await(path: "/") {
    state rows: [Map] = []
    state note = ""
    action load() {
        let r = await fetch("/api/rows")
        rows = r.rows
        let text = await fetch("/api/note", { method: "GET" })
        note = text.note
    }
    Button("Load", disabled: load.pending) { on click { load() } }
    Text("{rows.length} rows, note: {note}")
}
```

`fetch(url, options)` returns the parsed JSON body (or throws on a failed
response); `options` is a map with `method`, `headers`, `body` (a map is
sent as JSON). [Chapter 14](14-data.md) has `resource`, the declarative
form.

## Constants and environment

```wf
const MAX_ROWS = 50
const API = env.API_URL ?? "/api"

page Consts(path: "/") {
    Text("Up to {MAX_ROWS} rows from {API}")
}
```

`const` declares a project-wide value; `env.NAME` reads a variable from the
build environment (and `.env` in the project root). Both are inlined.

## Built-in functions

| Name | Does |
|---|---|
| `log(x, …)` | Prints to the console |
| `navigate(path)` | Changes the route ([chapter 3](03-pages-and-routing.md#navigating)) |
| `format`, `ago` | Above |
| `t("key", args)` | Translates ([chapter 15](15-i18n.md)) |
| `setTheme("dark")` | Switches the theme ([chapter 12](12-styling.md#dark-mode)) |
| `Number(x)`, `String(x)` | Convert |
| `every(ms) { }`, `after(ms) { }` | Timers ([chapter 5](05-state-and-reactivity.md#timers)) |

## Next

[Styling](12-styling.md).
