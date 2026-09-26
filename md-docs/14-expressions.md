# 14. Expressions

<!--
route: guide/expressions
group: basics
blurb: The whole expression language: literals, interpolation, operators, values that choose, calls, methods, formatting and await.
description: Literals, interpolation and formatted splices, operators and precedence, if and match as values, lambdas, methods, format and ago, regexes, tokens and await.
-->

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

Strings are double-quoted and read the escapes `\n`, `\t`, `\r`, `\\`,
`\"`, `\{` and `\}`; a raw string `#"…"#` and a block string `"""…"""` are in
[Language basics](05-language-basics.md#three-ways-to-write-one). Lists and
map literals may span lines and take a trailing comma.

The language's own types have literals of their own
([Types](13-types.md#the-types-the-language-brings-with-it)):

```wf
page Typed(path: "/", title: "Typed", description: "Literals of the built-in types.") {
    state due: Date = @2026-03-14             // a Date
    state opens: Time = @09:30                // a Time
    state at: DateTime = @2026-03-14T09:30Z   // a DateTime
    state wait: Duration = 90.minutes         // a Duration: ms, seconds, minutes, hours, days, weeks
    state price: Money = €12.99               // Money, in minor units; $ and £ too
    state brand: Color = #0F766E              // a Color
    Heading("Typed").h1
    Text("{due} {opens} {at} {wait.minutes()} {format(price)} {brand}")
}
```

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

### Showing a value a particular way

A splice may say how to show its value: `{value:.style}`, or
`{value:.style(option)}`, which is `format(value, .style, option)` written
where it is read.

```wf
page Receipt(path: "/") {
    state total = 1234.5
    state count = 1235
    state when = "2026-03-05T10:00:00Z"
    Text("Total: {total:.currency}")            // Total: $1,234.50
    Text("{count:.integer} items")              // 1,235 items
    Text("Shipped {when:.date(long)}")          // Shipped March 5, 2026
    Text("Down {0.256:.percent(1)}")            // — see below
}
```

The styles and their options are `format`'s, listed under
[`format` and `ago`](#format-and-ago). The value is an expression like any
other, so `{order.total:.currency}` and `{sum(rows):.compact}` both work —
but a splice has to open with a name, a `(` or a `[`, so a bare number
takes a name first (`derived ratio = 0.256`, then `{ratio:.percent(1)}`).

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
they are the same object, so compare ids. Precedence, strongest first: `!`
and unary `-`, then `* / %`, `+ -`, comparisons, equality, `&&`, `||`, `??`,
and ranges last ([Grammar](43-grammar.md#expressions)).

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

`length`, `trim()`, `trimStart()`, `trimEnd()`, `toLowerCase()`,
`toUpperCase()`, `capitalize()`, `truncate(n)`, `dedent()`, `lines()`,
`words()`, `includes(s)`, `startsWith(s)`, `endsWith(s)`, `indexOf(s)`,
`lastIndexOf(s)`, `slice(a, b)`, `substring(a, b)`, `charAt(i)`, `at(i)`,
`replace(a, b)`, `replaceAll(a, b)`, `split(sep)`, `padStart(n, c)`,
`padEnd(n, c)`, `repeat(n)`, `normalize()`, `localeCompare(other)`,
`match(re)`, `search(re)`.

`dedent()` takes the common indentation off a block of text — what a
string carried in from a file or an API keeps and a reader does not want.
`lines()` and `words()` split it: by line ending, and by runs of
whitespace.

### List methods

`length`, `map(f)`, `filter(f)`, `find(f)`, `findIndex(f)`, `some(f)`,
`every(f)`, `includes(x)`, `indexOf(x)`, `reduce(f, init)`, `join(sep)`,
`slice(a, b)`, `concat(other)`, `reverse()`, `sort(f)`, `sortBy(f)`,
`groupBy(f)`, `unique()`, `take(n)`, `first()`, `last()`, `sum()`,
`flatMap(f)`, and, in an imperative block, `push(x)`.

`sortBy`, `groupBy`, `unique`, `take`, `first`, `last`, `sum`, `capitalize`,
`truncate`, `dedent`, `lines` and `words` are WebFluent's own; the rest are
the browser's.

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

`$name` names a design token from the theme ([chapter 15](15-styling.md)).
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
        rows = await fetch("/api/rows")
        let answer = await fetch("/api/note", { method: "GET" })
        note = answer.text ?? ""
    }
    Button("Load", disabled: load.pending) { on click { load() } }
    Text("{rows.length} rows, note: {note}")
}
```

`fetch(url, options)` returns the parsed JSON body (or throws on a failed
response); `options` is a map with `method`, `headers`, `body` (a map is
sent as JSON). [Chapter 17](17-data.md) has `resource`, the declarative
form.

## Constants

```wf
const MAX_ROWS = 50
const API = env.PUBLIC_API_URL ?? "/api"

page Consts(path: "/") {
    Text("Up to {MAX_ROWS} rows from {API}")
}
```

`const` declares a project-wide value, read by name everywhere and inlined
into the bundle. It may carry a type: `const PAGE_SIZE: Number = 20`.
`env.NAME` reads a value fixed at build time — from the config's `env` map,
a `.env` file, or the shell — and only a public name may be read from a page;
[Environments](30-environments.md) has the rules.

## Built-in functions

These are the whole of what the language adds to what the browser already
has. Everything else with parentheses after it is a method, an action, a
component or one of the browser's own globals.

| Name | Does |
|---|---|
| `log(x, …)` | Prints to the console |
| `navigate(path)` | Changes the route ([chapter 6](06-pages-and-routing.md#links-and-navigation)) |
| `format(v, .style, opt)`, `ago(d)` | [Above](#format-and-ago) |
| `t("key", args)`, `setLocale("ar")` | Translates, and switches locale ([chapter 21](21-i18n.md)) |
| `setTheme("dark")` | `"dark"`, `"light"` or `"system"`, kept across visits ([chapter 15](15-styling.md#dark-mode)) |
| `every(ms) { }`, `after(ms) { }` | Timers ([chapter 8](08-state-and-reactivity.md#timers)) |
| `animate(el, name, ms)`, `replayAnimation(el, name)` | Plays one by hand ([chapter 20](20-motion.md#driving-one-yourself)) |
| `optimistic(holder, change)` | Shows a change before the server agrees ([chapter 17](17-data.md#showing-a-change-before-the-server-agrees)) |
| `beacon(url, data)` | A send that outlives the page ([chapter 18](18-realtime.md)) |
| `sanitize(html)` | Markup, through an allow-list ([chapter 23](23-security.md#markup-you-did-not-write)) |
| `uuid()` | A fresh `Uuid` ([chapter 13](13-types.md#the-types-the-language-brings-with-it)) |
| `fetch(url, options)` | A request, through the same engine as `api` ([Data](17-data.md#await-fetch-in-an-action)) |
| `ws(…)`, `sse(…)`, `broadcast(…)`, `rtc(…)` | Open a socket, a stream, a channel or a peer, after `socket x =` and the rest ([Real-time](18-realtime.md)) |

The browser's own globals need no prefix and compile to themselves:
`window`, `document`, `console`, `localStorage`, `sessionStorage`, `JSON`,
`Math`, `Date`, `setTimeout`, `setInterval`, `clearTimeout`,
`clearInterval`, `parseInt`, `parseFloat`, `Array`, `Object`, `String`,
`Number`, `Boolean`, `Promise`, `Error`, `Map`, `Set`, `RegExp`,
`Infinity`, `NaN`, `undefined`, `encodeURIComponent`, `decodeURIComponent`,
`encodeURI`, `decodeURI`, `atob`, `btoa`, `fetch`, `alert`, `confirm`,
`prompt`, `requestAnimationFrame`, `cancelAnimationFrame`, `navigator`,
`location` and the other platform names [Built-ins](40-built-ins.md#browser-globals)
lists.

## Next

[Styling](15-styling.md).
