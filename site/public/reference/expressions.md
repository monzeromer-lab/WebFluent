# Expressions

> Literals, interpolation and formatted splices, operators and precedence, if and match as values, lambdas, methods, format and ago, regexes, tokens, await.

The page: https://webfluent.monzeromer.dev/reference/expressions

An expression is anything that has a value: a prop's value, a condition,
the right side of a `state` or a `derived`, or the inside of `{…}` in a
string. Remember one thing: wherever WebFluent wants a value, any
expression in this chapter can stand there, and it stays live when it reads
state.

In this chapter:

- literals, including the literals of the language's own types;
- interpolation, and how a splice formats its value;
- operators and their precedence;
- `if` and `match` used as values;
- fields, calls, lambdas and the methods of strings, lists and numbers;
- `format` and `ago`, regular expressions, design tokens as values;
- `await`, constants, and the functions the language adds.

## A first example

This page works out a total from state and shows it. It uses a list
literal, a lambda, a method, arithmetic, an `if` expression and a
formatted splice:

```wf
page Basket(path: "/basket", title: "Basket", description: "What the basket holds.") {
    state prices = [4.5, 12, 3.5]
    state member = true
    derived subtotal = prices.sum()
    derived total = if member { subtotal * 0.9 } else { subtotal }
    Heading("Basket").h1
    Text("{prices.length} items, {total:.currency}")
    Text(if member { "Member price" } else { "Join to save 10%" }).muted
}
```

The page shows `3 items, $18.00` and `Member price`:

- `[4.5, 12, 3.5]` is a list literal, and `prices.sum()` is one of the
  helpers lists have.
- `if member { … } else { … }` is a value: the block that runs gives it.
- `{total:.currency}` is a splice that formats its value as money.

When `member` or `prices` changes, `subtotal`, `total` and both texts
follow.

## Literals

A literal writes a value directly. This page declares one of each of the
everyday kinds:

```wf
page Literals(path: "/literals", title: "Literals", description: "One value of each kind.") {
    state label = "Draft"                         // String
    state count = 42                              // Number
    state ratio = 3.14                            // Number
    state open = true                             // Bool
    state note: String? = null                    // null
    state sizes = [1, 2, 3]                       // [Number]
    state owner = { name: "Sam", age: 30 }        // a map with those fields
    state slug = /^[a-z]+$/i                      // a regular expression
    state accent = $primary                       // a design token
    Heading("Literals").h1
    Text("{label} {count} {ratio} {open} {sizes.length} {owner.name}")
}
```

How each kind behaves:

- **Strings** are double-quoted and read the escapes `\n`, `\t`, `\r`,
  `\\`, `\"`, `\{` and `\}`. A raw string `#"…"#` and a block string
  `"""…"""` are in
  [Language basics](https://webfluent.monzeromer.dev/learn/04-language-basics.md#three-ways-to-write-one).
- **Lists and maps** may span lines and take a trailing comma. A map
  literal has the shape it was written with, so the checker can read
  `owner.name` and refuse a misspelled field.
- **`null`** is the absence of a value. Only a type that ends in `?`
  holds it.

The language's own types have literals too
([Types](https://webfluent.monzeromer.dev/learn/12-types.md#the-types-the-language-brings-with-it)):

```wf
page Typed(path: "/typed", title: "Typed", description: "Literals of the built-in types.") {
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

`"{expr}"` puts the value of an expression into a string. Anything that
shows text accepts one, and the text follows the state it reads:

```wf
page Greeting(path: "/greeting", title: "Greeting", description: "Text built from state.") {
    state name = "Sam"
    state count = 3
    derived noun = if count == 1 { "item" } else { "items" }
    Heading("Hello, {name}").h1
    Text("You have {count} {noun}")
    Text("Total: {format(count * 9.99, .currency)}")
    Text("Next: {count + 1}")
}
```

The page shows `Hello, Sam`, `You have 3 items`, `Total: $29.97` and
`Next: 4`.

Any expression may go inside `{…}`: a call, a method chain, an `if`, even a
string of its own. `"{n ?? "none"}"` and `"{if ok { "yes" } else { "no" }}"`
both work. The rules:

- The group must close on the same line.
- A splice opens with a name, a `(` or a `[`. Anything else in braces —
  `"{key: value}"` in prose, a fragment of code, `"{"` alone — stays text.
- For a long expression, declare a `derived` value and splice its name, as
  `noun` above. It reads better and the checker names it in a message.

### Showing a value a particular way

A splice may say how to show its value: `{value:.style}`, or
`{value:.style(option)}`. It is `format(value, .style, option)` written
where the value is read.

```wf
page Receipt(path: "/receipt", title: "Receipt", description: "An order, formatted.") {
    state total = 1234.5
    state count = 1235
    state shipped = "2026-03-05T10:00:00Z"
    state discount = 0.256
    Heading("Receipt").h1
    Text("Total: {total:.currency}")            // Total: $1,234.50
    Text("{count:.integer} items")              // 1,235 items
    Text("Shipped {shipped:.date(long)}")       // Shipped March 5, 2026
    Text("Saved {discount:.percent(1)}")        // Saved 25.6%
}
```

The styles and their options are those of `format`, listed under
[`format` and `ago`](#format-and-ago). The value is an expression like any
other, so `{order.total:.currency}` and `{rows.sum():.compact}` both work.

> [!WARNING]
> A splice must open with a name, a `(` or a `[`, so `"{0.256:.percent}"`
> shows as written. Give the number a name first — `derived ratio = 0.256`,
> then `{ratio:.percent}` — or wrap it: `{(0.256):.percent}`.

## Operators

The operators, by kind:

| Kind | Operators |
|---|---|
| Arithmetic | `+ - * / %`, unary `-` |
| Comparison | `== != < <= > >=` (`!==` too) |
| Logic | `&& \|\| !` |
| Null | `??` fallback, `?.` optional chain |
| Range | `a..b` (exclusive), `a..=b` (inclusive) |
| Spread | `[...a, x]`, `{ ...m, key: v }` |
| Grouping | `( )` |

This page uses most of them:

```wf
page Scores(path: "/scores", title: "Scores", description: "Two numbers compared.") {
    state home = 7
    state away = 2
    state player: Map? = null
    Heading("Scores").h1
    Text("{home + away} {home - away} {home * away} {home / away} {home % away}")
    Text(if home > away && away != 0 { "Home leads" } else { "Away leads" })
    Text(player?.name ?? "No player picked")
    for round in 1..=3 { Badge("Round {round}") }
}
```

How they behave:

- `+` joins strings when either side is a `String`, and adds numbers
  otherwise. Arithmetic on anything else is a `T18`.
- `==` compares values. Two records with the same fields are not `==`
  unless they are the same object, so compare their ids.
- Against a `null` literal, `==` and `!=` also match a value that is
  absent, such as a query key the address does not have.
- `a ?? b` is `b` when `a` is `null`. `a?.b`, `a?.m()` and `a?.[i]` read
  through a `null` and give `null` instead of failing.
- A range is the whole numbers from `a` up to `b`, without `b` for `..` and
  with it for `..=`.

Precedence, strongest first: `!`, unary `-` and `await`; then `* / %`;
`+ -`; the comparisons; `==` and `!=`; `&&`; `||`; `??`; and ranges last. So
`1..n + 1` is `1..(n + 1)`, and `a ?? b || c` is `a ?? (b || c)`. The full
table is in [Grammar](https://webfluent.monzeromer.dev/reference/grammar.md#expressions).

## Choosing a value

`if` and `match` are expressions when they stand where a value is
expected. Each arm's block is its value:

```wf
enum Level { low, mid, high }

page Grades(path: "/grades", title: "Grades", description: "A score, graded.") {
    state score = 72
    state level: Level = .mid
    derived grade = if score >= 90 { "A" } else if score >= 70 { "B" } else { "C" }
    derived tone = match level { .low { "calm" } .mid { "steady" } else { "loud" } }
    Heading("Grade {grade}").h1
    Text("The room is {tone}")
    Badge(grade, tone: if score >= 70 { .success } else { .danger })
}
```

The rules:

- An `if` with no `else` is `null` when its condition fails.
- `if let x = v { a } else { b }` binds `x` when `v` is not `null`.
- A `match` on an enum, as a value or as a statement, needs an arm for
  every case or an `else` (`T15`). One that covers every case needs no
  `else`.
- A case with a payload binds one name in a `match` expression:
  `match status { .failed(reason) { reason } else { "" } }`.

## Fields, calls and methods

A field is read with `.`, an item with `[i]`, and a method is called with
`.name(…)`. This page filters, maps and joins a list of records:

```wf
type Todo { id: String, title: String, done: Bool = false }

page Todos(path: "/todos", title: "Todos", description: "A list, filtered and joined.") {
    state todos: [Todo] = [Todo(id: "1", title: "milk"), Todo(id: "2", title: "eggs", done: true)]
    state search = "M"
    derived open = todos.filter(todo => !todo.done)
    derived titles = todos.map(todo => todo.title.capitalize())
    derived hits = todos.filter(todo => todo.title.toLowerCase().includes(search.toLowerCase()))
    derived first = todos.first()
    derived summary = titles.join(", ")
    Heading("{todos.length} todos").h1
    Text("{open.length} open, {hits.length} match: {summary}")
    if let f = first { Text("First: {f.title}") }
}
```

A lambda is written `x => expr`, `(a, b) => expr` or `() => expr`. Its body
is one expression; when it must do more, it calls an action. To return a
map, write the parameters in parentheses: `(x) => { key: x }`. A
parameter's type comes from what the method expects, so `todo` above is a
`Todo`.

A member something declares wins over a helper of the same name. A store
may name an action `preview`, `first` or `truncate`: `OpenFiles.preview(path)`
calls that action, with its arguments, from a page or from another store,
and a worker's `css.last(kind)` is the worker's own `last`. The helpers
below are for lists and strings.

### String methods

`length`, `trim()`, `trimStart()`, `trimEnd()`, `toLowerCase()`,
`toUpperCase()`, `capitalize()`, `truncate(n)`, `dedent()`, `lines()`,
`words()`, `includes(s)`, `startsWith(s)`, `endsWith(s)`, `indexOf(s)`,
`lastIndexOf(s)`, `slice(a, b)`, `substring(a, b)`, `charAt(i)`, `at(i)`,
`replace(a, b)`, `replaceAll(a, b)`, `split(sep)`, `padStart(n, c)`,
`padEnd(n, c)`, `repeat(n)`, `normalize()`, `localeCompare(other)`,
`match(re)`, `search(re)`.

`dedent()` takes the common indentation off a block of text — the
indentation a string carried in from a file or an API keeps and a reader
does not want. `lines()` splits a text by line ending, and `words()` by
runs of whitespace.

### List methods

`length`, `map(f)`, `filter(f)`, `find(f)`, `findIndex(f)`, `some(f)`,
`every(f)`, `includes(x)`, `indexOf(x)`, `reduce(f, init)`, `join(sep)`,
`slice(a, b)`, `concat(other)`, `reverse()`, `sort(f)`, `sortBy(f)`,
`groupBy(f)`, `unique()`, `take(n)`, `first()`, `last()`, `sum()`,
`flatMap(f)`, and, in an action or a handler, `push(x)` and `remove(i)`.

`sortBy`, `groupBy`, `unique`, `take`, `first`, `last`, `sum`,
`capitalize`, `truncate`, `dedent`, `lines` and `words` are WebFluent's
own; the rest are the browser's. Every one of them also runs at build time,
so the pre-rendered page shows the result. [Built-ins](https://webfluent.monzeromer.dev/reference/built-ins.md#list-methods)
says what each returns.

On a state, `sort` and `reverse` read in an expression work on a copy, so
reading a sorted list never changes the list. As a statement, `push`,
`remove` and `sort` give the state a new list, so whatever reads it
repaints.

### Number methods

`toFixed(n)`, `toPrecision(n)`, `toString()`. To convert, use
`Number("12")` for a string to a number, and `"{n}"` or `String(n)` for a
number to a string. `Math.round`, `Math.max` and the rest are available as
`Math.*`.

## `format` and `ago`

`format(value, .style, option)` writes a number, a date or money as text in
the reader's locale. The locale is the i18n locale when the project has
one, and the document's language otherwise. This page shows each style:

```wf
page Formats(path: "/formats", title: "Formats", description: "Every style of format.") {
    state price = 1234.5
    state ratio = 0.256
    state visitors = 1200000
    state when = "2026-03-14T09:30:00"
    Heading("Formats").h1
    Text(format(price, .currency))                 // $1,234.50
    Text(format(price, .currency, "EUR"))          // €1,234.50
    Text(format(ratio, .percent))                  // 26%
    Text(format(ratio, .percent, 1))               // 25.6%
    Text(format(price, .decimal, 1))               // 1,234.5
    Text(format(visitors, .integer))               // 1,200,000
    Text(format(visitors, .compact))               // 1.2M
    Text(format(when, .date))                      // Mar 14, 2026
    Text(format(when, .date, "long"))              // March 14, 2026
    Text(format(when, .time))                      // 9:30 AM
    Text(format(when, .datetime))                  // Mar 14, 2026, 9:30 AM
    Text(format(when, "EEEE d MMMM yyyy, HH:mm"))  // Saturday 14 March 2026, 09:30
    Text(ago(when))                                // 3 days ago
    Text(format(when, .relative))                  // the same
}
```

The styles, and the option each takes:

| Style | Gives | Option |
|---|---|---|
| `.number` (the default) | `1,234.5` | — |
| `.integer` | `1,235` | — |
| `.decimal` | `1,234.50` | places, `2` when omitted |
| `.currency` | `$1,234.50` | the currency code, `"USD"` when omitted |
| `.percent` | `26%`, of a fraction | places, `0` when omitted |
| `.compact` | `1.2K`, `1.2M` | — |
| `.date`, `.time`, `.datetime` | `Mar 14, 2026`, `9:30 AM` | `"short"`, `"medium"`, `"long"` or `"full"` |
| `.relative` | `3 days ago` | — |

A pattern string in place of a style writes a date with these letters:
`yyyy yy MMMM MMM MM M dd d EEEE EEE HH H hh h mm ss a`. A `Money` value
needs no style: `format(price)` writes it in its own currency.

`ago(date)` gives the distance from now in words: `now` under 45 seconds,
then `5 minutes ago`, `yesterday`, `in 2 days`. `ago(date, moment)`
measures from a moment of your own.

The pre-rendered page formats with a built-in table of the common locales
and currencies. Once the page's script runs, the browser's `Intl` takes
over and corrects any difference. A change of locale redraws every
formatted text.

## Regular expressions

`/…/flags` is a regular expression. This page checks an address as the
reader types it:

```wf
page Signup(path: "/signup", title: "Sign up", description: "An address, checked as it is typed.") {
    state email = ""
    derived valid = /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email)
    derived domain = email.split("@").last() ?? ""
    Heading("Sign up").h1
    Input(bind: email, label: "Email")
    if email != "" && !valid { Text("That does not look like an email.").danger.sm }
    Text("Domain: {domain}")
}
```

What each use gives:

- `re.test(s)` is a `Bool`.
- `s.match(re)` and `re.exec(s)` are a `[String]?`: the match and its
  groups, or `null`.
- `s.replace(re, "x")` and `s.replaceAll(re, "x")` are a `String`;
  `s.split(re)` is a `[String]`; `s.search(re)` is a `Number`.

## Tokens as values

`$name` names a design token from the theme ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md)). In
a style it becomes the token's CSS variable. As a value it is a reference
to that variable, so a prop or a state that takes a colour takes a token:

```wf
page Callout(path: "/callout", title: "Callout", description: "A border coloured by a token.") {
    state accent = $primary
    Heading("Callout").h1
    Card {
        style { border-left: 4px solid {accent} }
        Text("The border takes its colour from the theme.")
    }
}
```

When the theme changes `color-primary`, the border changes with it.

As a value there is no property to say which group a short name belongs
to, so `$primary` is read as the one group that has it, the colours:
`$color-primary`. A short name several groups have (`$md` is a spacing, a
radius and a shadow) is written in full: `$spacing-md`.

## `await`

`await` waits for a promise inside an action or a handler, and makes the
action asynchronous. While a call of it runs, `name.pending` is `true`:

```wf
page Report(path: "/report", title: "Report", description: "Rows loaded on request.") {
    state rows: [Map] = []
    state note = ""
    action load() {
        rows = await fetch("/api/rows")
        let answer = await fetch("/api/note", { method: "GET" })
        note = answer.text ?? ""
    }
    Heading("Report").h1
    Button("Load", disabled: load.pending) { on click { load() } }
    Text("{rows.length} rows, note: {note}")
}
```

`fetch(url, options)` returns the parsed JSON body, or throws when the
response fails. `options` is a map with `method`, `headers` and `body`; a
map given as `body` is sent as JSON. `await` outside an action, a handler,
a timer or a hook is a `T17`. [Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md) has `resource`, the
declarative way to load data.

## Constants

`const` declares a value for the whole project. Every page, component and
store reads it by name, and the build inlines it into the bundle:

```wf
const MAX_ROWS = 50
const PAGE_SIZE: Number = 20
const API = env.PUBLIC_API_URL ?? "/api"

page Settings(path: "/settings", title: "Settings", description: "The limits this site runs with.") {
    Heading("Settings").h1
    Text("Up to {MAX_ROWS} rows, {PAGE_SIZE} a page, from {API}")
}
```

A constant may carry a type, as `PAGE_SIZE` does. Assigning to one is an
`X01`. `env.NAME` reads a value fixed at build time — from the config's
`env` map, a `.env` file or the shell. A page may read only a public name,
one that begins `PUBLIC_` or that `public_env` lists;
[Environments](https://webfluent.monzeromer.dev/recipes.md) has the rules.

## Built-in functions

These functions are everything the language adds to what the browser
already has. Anything else followed by parentheses is a method, an action,
a component or one of the browser's own globals. [Built-ins](https://webfluent.monzeromer.dev/reference/built-ins.md#functions)
has an entry for each.

| Name | Does |
|---|---|
| `log(x)` | Prints a value to the console |
| `navigate(path)` | Changes the route ([Pages and routing](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md#links-and-navigation)) |
| `format(v, .style, opt)`, `ago(d)` | Formats a value ([above](#format-and-ago)) |
| `t("key", args)`, `setLocale("ar")` | Translates, and switches the locale ([i18n](https://webfluent.monzeromer.dev/recipes.md)) |
| `setTheme("dark")` | `"dark"`, `"light"` or `"system"`, kept across visits ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md#dark-mode)) |
| `keyIs(e, "mod+s")`, `keyLabel("mod+s")` | Tests a key event against a shortcut, and writes a shortcut for this system |
| `every(ms) { }`, `after(ms) { }` | Timers ([State and reactivity](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#timers)) |
| `animate(el, name, ms)`, `replayAnimation(el, name)` | Plays an animation by hand ([Motion](https://webfluent.monzeromer.dev/recipes/drive-an-animation.md)) |
| `optimistic(holder, change)` | Shows a change before the server agrees ([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md#show-a-change-before-the-server-agrees)) |
| `beacon(url, data)` | Sends data in a way that outlives the page ([Real-time](https://webfluent.monzeromer.dev/recipes.md)) |
| `sanitize(html)` | Passes markup through an allow-list ([Security](https://webfluent.monzeromer.dev/about/security.md#show-markup-you-did-not-write)) |
| `uuid()` | Makes a fresh `Uuid` ([Types](https://webfluent.monzeromer.dev/learn/12-types.md#the-types-the-language-brings-with-it)) |
| `fetch(url, options)` | Sends a request, through the same engine as `api` ([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md#fetch-on-demand-with-await-fetch)) |
| `ws(…)`, `sse(…)`, `broadcast(…)`, `rtc(…)`, `spawn(…)` | Open a socket, a stream, a channel, a peer or a worker, after `socket x =` and the rest ([Real-time](https://webfluent.monzeromer.dev/recipes.md)) |

The browser's own globals need no prefix and compile to themselves:
`window`, `document`, `console`, `localStorage`, `sessionStorage`, `JSON`,
`Math`, `Date`, `setTimeout`, `setInterval`, `clearTimeout`,
`clearInterval`, `parseInt`, `parseFloat`, `Array`, `Object`, `String`,
`Number`, `Boolean`, `Promise`, `Error`, `Map`, `Set`, `RegExp`,
`Infinity`, `NaN`, `undefined`, `encodeURIComponent`, `decodeURIComponent`,
`encodeURI`, `decodeURI`, `atob`, `btoa`, `fetch`, `alert`, `confirm`,
`prompt`, `requestAnimationFrame`, `cancelAnimationFrame`, `navigator`,
`location`, and the other platform names that
[Built-ins](https://webfluent.monzeromer.dev/reference/built-ins.md#browser-globals) lists.

## Summary

| Syntax | Meaning |
|---|---|
| `"text {expr}"` | A string with a live value spliced in |
| `"{value:.style(option)}"` | A splice formatted as `format` would |
| `a ?? b`, `a?.b`, `a?.[i]` | A fallback for `null`, and reading through one |
| `a..b`, `a..=b` | A range, without and with its end |
| `[...xs, x]`, `{ ...m, key: v }` | A list or map spread into a new one |
| `if c { a } else { b }` | The value of the arm that runs |
| `match v { .case { a } else { b } }` | The value of the arm that matches |
| `x => expr`, `(a, b) => expr` | A lambda |
| `/pattern/flags` | A regular expression |
| `$name` | A design token |
| `await expr` | The result of a promise, in an action or a handler |
| `const NAME = expr` | A value the whole project reads |

[Grammar](https://webfluent.monzeromer.dev/reference/grammar.md#expressions) has the full precedence table, and
[Built-ins](https://webfluent.monzeromer.dev/reference/built-ins.md) every function, value and method.

## Next

[Styling](https://webfluent.monzeromer.dev/learn/13-styling.md).
