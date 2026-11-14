# Built-ins

> Built-in functions, browser values like viewport, query, now and network, string, list and number methods, the built-in types, and browser globals.

The page: https://webfluent.monzeromer.dev/reference/built-ins

This page lists everything a program can call or read without declaring
it: the functions the language adds, the browser values it keeps current,
the methods of strings, lists and numbers, the built-in types, and the
browser's own globals. [Expressions](https://webfluent.monzeromer.dev/reference/expressions.md) explains how to use
them; this page is for looking one up.

Each function and value has an entry in the same order: what it is, its
signature, its parameters and what it returns, where it may be written,
and an example. An example of one line is a fragment of a page; the names
it reads are assumed to be declared.

## Functions

The functions, in alphabetical order:

| Function | Does |
|---|---|
| [`after(interval) { }`](#after) | Runs a block once, after a delay |
| [`ago(date)`](#ago) | The distance from now, in words |
| [`animate(element, name, duration)`](#animate) | Plays an animation on an element handle |
| [`beacon(url, data)`](#beacon) | Sends a small post that survives the page unloading |
| [`broadcast(name)`](#broadcast) | Opens a channel the site's other tabs hear |
| [`every(interval) { }`](#every) | Runs a block repeatedly |
| [`fetch(url, options)`](#fetch) | Sends a request through the same engine as `api` |
| [`format(value, .style, option)`](#format) | A number, date or money as text, in the locale |
| [`keyIs(event, combination)`](#keyis) | Whether a key event is a shortcut |
| [`keyLabel(combination)`](#keylabel) | A shortcut as this system writes it |
| [`log(x)`](#log) | Prints a value to the console |
| [`navigate(path)`](#navigate) | Moves to a route |
| [`optimistic(holder, change)`](#optimistic) | Shows a change now, and takes it back if the action throws |
| [`replayAnimation(element, name)`](#replayanimation) | Plays an element's animation again |
| [`rtc(signal: …)`](#rtc) | Opens a data channel to another reader's page |
| [`sanitize(html)`](#sanitize) | Markup through an allow-list |
| [`setLocale(locale)`](#setlocale) | Switches the locale, and the direction with it |
| [`setTheme(choice)`](#settheme) | Switches between light, dark and the system's choice |
| [`spawn(path)`](#spawn) | Starts a script of `public/` in a worker |
| [`sse(url)`](#sse) | Opens a stream of server-sent events |
| [`t("key", values)`](#t) | A translation |
| [`uuid()`](#uuid) | A fresh `Uuid` |
| [`ws(url)`](#ws) | Opens a WebSocket |

Every name in this list but `spawn` belongs to the language: a `state`,
prop, parameter, loop item or `const` with one of these names is an
`E118`. `spawn` means something only after `worker name =`.

### `after`

Runs a block once, after a delay, unless its scope leaves first.

Signature: `after(interval) { statements }`

| Parameter | Type | Meaning |
|---|---|---|
| `interval` | `Number` (milliseconds) or `Duration` | How long to wait |

Returns nothing. Where: a render block — a page, a component, a branch, a
list item. The timer is cancelled when the block that holds it leaves.
See [Timers](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#timers).

```wf
after(3.seconds) { banner = null }
```

### `ago`

The distance between a date and now, in words, in the reader's locale.

Signature: `ago(date)`, `ago(date, moment)` → `String`

| Parameter | Type | Meaning |
|---|---|---|
| `date` | `Date`, `DateTime` or an ISO 8601 string | The moment to describe |
| `moment` | the same, optional | What to measure from, in place of now |

Returns `now` under 45 seconds, then `5 minutes ago`, `yesterday`,
`in 2 weeks`, and so on. An unreadable date gives `""`. Where: anywhere. It
is the same as `format(date, .relative)`.

```wf
Text("Posted {ago(post.published)}").muted
```

### `animate`

Plays a named animation on an element you hold a handle on.

Signature: `animate(element, name, duration)` → a handle

| Parameter | Type | Meaning |
|---|---|---|
| `element` | a `ref:` handle | The element to animate |
| `name` | `String` | A built-in animation (`"pulse"`, `"shake"`, …) or one an `animation` declares |
| `duration` | `String` such as `"400ms"` | How long it plays |

Returns a handle with `play()`, `cancel()` and `finished` (a promise). It
plays as soon as it is made. Where: handlers and actions. See
[Motion](https://webfluent.monzeromer.dev/recipes/drive-an-animation.md).

```wf
Card(ref: panel) { on mouseenter { animate(panel, "pulse", "400ms") } }
```

### `beacon`

Sends a small `POST` that the browser delivers even while the page is
unloading.

Signature: `beacon(url, data)` → `Bool`

| Parameter | Type | Meaning |
|---|---|---|
| `url` | `String` | Where to send it |
| `data` | `Map` or `String` | The body; a map is sent as JSON |

Returns whether the browser accepted the send. Where: anywhere, usually a
handler. See [Send a message as the page unloads](https://webfluent.monzeromer.dev/recipes/send-on-unload.md).

```wf
beacon("/analytics", { event: "checkout", value: total })
```

### `broadcast`

Opens a channel that every tab and window of the site on this browser
hears.

Signature: `channel name = broadcast(channelName) { on message(m) { … } }`

| Parameter | Type | Meaning |
|---|---|---|
| `channelName` | `String` | The channel; tabs that open the same name hear each other |

Returns a channel handle with `.post(value)`, `.messages`, `.last(kind)`,
`.state` and `.close()`. Where: only after `channel name =`, in a page, a
component or a store. The channel closes when its scope leaves. See
[Talk to the site's other tabs](https://webfluent.monzeromer.dev/recipes/talk-between-tabs.md).

```wf
channel cart = broadcast("cart") { on message(m) { Cart.merge(m) } }
```

### `every`

Runs a block repeatedly until its scope leaves.

Signature: `every(interval) { statements }`

| Parameter | Type | Meaning |
|---|---|---|
| `interval` | `Number` (milliseconds) or `Duration` | The time between runs |

Returns nothing. Where: a render block. The timer stops when its page,
branch or list item leaves. See
[Timers](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#timers).

```wf
every(1.seconds) { elapsed = elapsed + 1 }
```

### `fetch`

Sends a request through the same engine an `api` uses: retries, caching
and cancelling come with it.

Signature: `fetch(url, options)` → the parsed response body

| Parameter | Type | Meaning |
|---|---|---|
| `url` | `String` | The address; a value spliced into it is encoded |
| `options` | `Map`, optional | `method`, `headers`, `body`; a map given as `body` is sent as JSON |

Returns the parsed JSON body, and throws when the response fails. Where:
`await fetch(…)` in an action or a handler, or `resource name = fetch(…)`,
which adds `on:` and the other options of a resource. `window.fetch` is the
browser's own, untouched. See
[Fetch on demand with `await fetch`](https://webfluent.monzeromer.dev/learn/15-data.md#fetch-on-demand-with-await-fetch).

```wf
let rows = await fetch("/api/rows?page={page}", { method: "GET" })
```

### `format`

Writes a number, a date or money as text, in the reader's locale.

Signature: `format(value)`, `format(value, .style)`, `format(value, .style, option)`, `format(value, "pattern")` → `String`

| Parameter | Type | Meaning |
|---|---|---|
| `value` | `Number`, `Money`, `Date`, `DateTime` or an ISO string | What to write |
| `style` | `.number` (default), `.integer`, `.decimal`, `.currency`, `.percent`, `.compact`, `.date`, `.time`, `.datetime`, `.relative` | How to write it |
| `option` | depends on the style | Places for `.decimal` (2) and `.percent` (0); a currency code for `.currency` (`"USD"`); `"short"`, `"medium"`, `"long"` or `"full"` for the date styles |
| `pattern` | `String` | A date pattern of `yyyy yy MMMM MMM MM M dd d EEEE EEE HH H hh h mm ss a` |

Returns the text. The locale is the i18n locale when the project has one,
and the document's language otherwise; a change of locale redraws it. A
`Money` value needs no style. In a string, `{value:.style(option)}` is the
same call. Where: anywhere. See
[`format` and `ago`](https://webfluent.monzeromer.dev/reference/expressions.md#format-and-ago).

```wf
Text(format(price, .currency, "EUR"))
```

### `keyIs`

Whether a key event is a given combination — for a table of shortcuts held
as data.

Signature: `keyIs(event, combination)` → `Bool`

| Parameter | Type | Meaning |
|---|---|---|
| `event` | a key event | The event a `keydown` handler receives |
| `combination` | `String` | Modifiers and a key, as `on key("…")` writes them: `"mod+s"`, `"shift+ArrowDown"` |

Returns `true` when the event is that combination. `mod` is ⌘ on a Mac and
Ctrl elsewhere. Where: handlers. See [Keys](https://webfluent.monzeromer.dev/learn/08-events.md#keys).

```wf
Input(bind: draft, label: "Note") { on keydown(e) { if keyIs(e, "mod+s") { save() } } }
```

### `keyLabel`

A combination as this system writes it, for a menu or a tooltip.

Signature: `keyLabel(combination)` → `String`

| Parameter | Type | Meaning |
|---|---|---|
| `combination` | `String` | As `keyIs` takes it |

Returns `⇧⌘P` on a Mac and `Ctrl+Shift+P` elsewhere for
`"mod+shift+p"`. Where: anywhere. A `Menu.Item`'s or a `Tooltip`'s
`shortcut:` is written this way for you.

```wf
Text("Search: {keyLabel("mod+k")}").muted
```

### `log`

Prints a value to the browser's console.

Signature: `log(value)`

| Parameter | Type | Meaning |
|---|---|---|
| `value` | any | The value to print. For several, splice them into one string, `log("{a} {b}")`, or call `console.log(a, b)` |

Returns nothing. Where: actions, handlers, effects and timers. A `Secret`
passed to it is a `T12`.

```wf
log("saved {items.length} items")
```

### `navigate`

Moves to a route, as a click on a `Link` would.

Signature: `navigate(path)`

| Parameter | Type | Meaning |
|---|---|---|
| `path` | `String` | A site-relative path; a value spliced into it is encoded |

Returns nothing. In a single-page build the router draws the new page; in
a static build it loads the page's own HTML file. A literal path no page
matches is an `R01`, and a scheme a browser would run is refused. Where:
actions and handlers. See
[Links and navigation](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md#links-and-navigation).

```wf
Button("Open") { on click { navigate("/orders/{order.id}") } }
```

### `optimistic`

Shows a change at once, before the server agrees. If anything later in the
action throws, the change is taken back.

Signature: `optimistic(holder, change)`

| Parameter | Type | Meaning |
|---|---|---|
| `holder` | a state, or a store's state | What to change |
| `change` | a lambda from the old value to the new one | The change to show |

Returns a handle with `rollback()`. Where: actions. See
[Show a change before the server agrees](https://webfluent.monzeromer.dev/learn/15-data.md#show-a-change-before-the-server-agrees).

```wf
optimistic(Todos.items, items => items.filter(i => i.id != id))
```

### `replayAnimation`

Plays again an animation the element already has, such as its enter
animation.

Signature: `replayAnimation(element, name)`

| Parameter | Type | Meaning |
|---|---|---|
| `element` | an element, such as `e.currentTarget` | The element to animate |
| `name` | `String` | The animation, such as `"fadeIn"` |

Returns nothing. Where: handlers. See
[Motion](https://webfluent.monzeromer.dev/recipes/drive-an-animation.md).

```wf
Card.outlined.fadeIn { on mouseenter(e) { replayAnimation(e.currentTarget, "fadeIn") } }
```

### `rtc`

Opens a WebRTC data channel straight to another reader's page.

Signature: `peer name = rtc(signal: send, initiator: Bool, ice: [servers]) { on message(m) { … } }`

| Option | Type | Meaning |
|---|---|---|
| `signal` | a lambda, required | How what this side must tell the other reaches it |
| `initiator` | `Bool` | Whether this side makes the offer; one side must |
| `ice` | `[Map]` | The servers that help readers on different networks connect; none by default |

Returns a peer handle with `.send(value)`, `.signal(message)` (hand it what
the other side sent), `.state`, `.messages` and `.close()`. Where: only after
`peer name =`. See
[Connect two readers' pages](https://webfluent.monzeromer.dev/recipes/connect-two-readers.md).

```wf
peer link = rtc(signal: m => lobby.post(m), initiator: query.host == "1") { on message(m) { heard = m.text } }
```

### `sanitize`

Passes markup through an allow-list of elements and attributes, for
`Unsafe.Html`.

Signature: `sanitize(html)` → `String`

| Parameter | Type | Meaning |
|---|---|---|
| `html` | `String` | Markup from outside the project |

Returns the markup without scripts, styles, frames, forms, `on*`
attributes, or an `href` or `src` a browser would run. It runs at build
time and in the browser from the same list. Where: anywhere. See
[Show markup you did not write](https://webfluent.monzeromer.dev/about/security.md#one-door-for-markup).

```wf
Unsafe.Html(sanitize(post.body))
```

### `setLocale`

Switches the page to another locale, and its direction with it.

Signature: `setLocale(locale)`

| Parameter | Type | Meaning |
|---|---|---|
| `locale` | `String` | A locale `i18n.locales` lists, such as `"ar"`; `-u-nu-arab` or `-u-nu-latn` names the digits |

Returns nothing. Every `t(…)`, `format` and `ago` redraws, and an RTL
locale sets `dir="rtl"`. A literal locale the config does not list is an
`I04`. Where: actions, handlers and effects. See
[i18n](https://webfluent.monzeromer.dev/recipes/translate-a-site.md#switch-the-locale).

```wf
Button("العربية") { on click { setLocale("ar") } }
```

### `setTheme`

Switches between the light theme, the dark one, and whatever the system
asks for, and keeps the choice across visits.

Signature: `setTheme(choice)`

| Parameter | Type | Meaning |
|---|---|---|
| `choice` | `String` | `"light"`, `"dark"` or `"system"` |

Returns nothing. The browser value `theme` reads the choice. Any other
number of arguments is a `T10`. Where: actions and handlers. See
[Dark mode](https://webfluent.monzeromer.dev/learn/13-styling.md#dark-mode).

```wf
Button("Dark") { on click { setTheme("dark") } }
```

### `spawn`

Starts a script of `public/` in a worker beside the page.

Signature: `worker name = spawn(path, restart: n, timeout: ms, type: .module) { on message(m) { … } }`

| Parameter | Type | Meaning |
|---|---|---|
| `path` | `String` | The script's site-relative address; a missing file is a `D01` |
| `restart` | `Number` | How many times a crash starts it again |
| `timeout` | `Number` | How long a request waits, in milliseconds (10 s unset) |
| `type` | `.module` | Starts a module worker |

Returns a worker handle with `.request(method, params, options)`,
`.send(value)`, `.last(kind)`, `.pending`, `.state` and `.restart()`.
Where: only after `worker name =`, in a page, a component or a store. See
[Run a script in a worker](https://webfluent.monzeromer.dev/recipes/run-a-worker.md).

```wf
worker css = spawn("/workers/css.js", restart: 1) { on message(m) { problems = m.items } }
```

### `sse`

Opens a stream of server-sent events, which resumes where it was after a
drop.

Signature: `stream name = sse(url, events: [names])`

| Parameter | Type | Meaning |
|---|---|---|
| `url` | `String` | The stream's address |
| `events` | `[String]` | The named events to listen for |

Returns a stream handle with `.messages`, `.last(kind)`, `.state`,
`.error` and `.close()`. Where: only after `stream name =`. See
[Listen to server-sent events](https://webfluent.monzeromer.dev/recipes/server-sent-events.md).

```wf
stream ticks = sse("/events", events: ["price", "status"])
```

### `t`

A translation of a key, in the reader's locale.

Signature: `t(key)`, `t(key, values)` → `String`

| Parameter | Type | Meaning |
|---|---|---|
| `key` | `String` | A key of the translation files |
| `values` | `Map`, optional | Values for the message's `{placeholders}`; `count` picks a plural form |

Returns the message. A literal key no locale has is an `I01`; a value the
message does not use, or a placeholder the call does not pass, is an
`I02`. Where: anywhere. See [`t("key")`](https://webfluent.monzeromer.dev/recipes/translate-a-site.md#translate-text-with-t).

```wf
Text(t("cart.items", { count: Cart.count }))
```

### `uuid`

A fresh random identifier.

Signature: `uuid()` → `Uuid`

It takes no parameters. Where: anywhere, usually an action that makes a
record.

```wf
items = items.concat([Todo(id: uuid(), title: draft)])
```

### `ws`

Opens a WebSocket, which reconnects with backoff and keeps itself alive.

Signature: `socket name = ws(url, protocols: [names], heartbeat: duration) { send T  receive T  on message(m) { … } }`

| Parameter | Type | Meaning |
|---|---|---|
| `url` | `String` | A `ws:` or `wss:` address |
| `protocols` | `[String]` | The subprotocols to offer |
| `heartbeat` | `Duration` | How often to ping a quiet line |

Returns a socket handle with `.send(value)`, `.messages`, `.last(kind)`,
`.state`, `.error`, `.closure` and `.close()`. Where: only after
`socket name =`. See [Open a socket](https://webfluent.monzeromer.dev/recipes/open-a-socket.md).

```wf
socket chat = ws("wss://example.com/chat", heartbeat: 20.seconds) { on message(m) { Feed.add(m) } }
```

## Browser values

The language keeps these values current and lets every page, component and
store read them. Each is a name the language owns: declaring a `state`,
prop or parameter with one of these names is an `E118`.

| Value | Holds |
|---|---|
| [`viewport`](#viewport) | The window's size and breakpoints |
| [`query`](#query) | The address's query string |
| [`hash`](#hash) | The address's fragment |
| [`theme`](#theme) | The reader's light or dark choice |
| [`now`](#now) | The current moment |
| [`network`](#network) | The connection's state |
| [`update`](#update) | A new version of an offline site |
| [`locale`, `dir`](#locale-and-dir) | The reader's language and its direction |
| [`params`](#params) | The route's parameters |

### `viewport`

The window's size, and which breakpoints it is past.

| Field | Type | Meaning |
|---|---|---|
| `.width`, `.height` | `Number` | The size in CSS pixels |
| `.sm`, `.md`, `.lg`, `.xl` | `Bool` | Whether the width is at least that breakpoint's token |

It follows `matchMedia`, so a breakpoint field changes once when its answer
changes, not on every pixel of a resize.

```wf
if viewport.md { Sidebar { Sidebar.Item(to: "/", icon: "home") { Text("Home") } } }
```

### `query`

The address's query string, as a map from each key to its value.

A key the address lacks reads as `null`, so `query.tab ?? "all"` gives a
default.

```wf
Text("Showing {query.tab ?? "all"}")
```

### `hash`

The address's fragment, without the `#`. It is `""` when there is none.

```wf
if hash == "pricing" { Text("You followed a link to pricing.") }
```

### `theme`

The reader's choice: `"light"`, `"dark"` or `"system"`. `setTheme` changes
it.

```wf
Button("Light", outlined: theme != "light") { on click { setTheme("light") } }
```

### `now`

The current moment, as a `DateTime` that keeps itself current — every
minute by default.

Signature: `now`, `now(every: interval)` → `DateTime`

Read it where it is shown or in a `derived` value. A `state` takes it once,
so `state opened = now` keeps the moment the page opened (`X06`). See
[`now`](https://webfluent.monzeromer.dev/learn/12-types.md#now).

```wf
Text("It is {now(every: 1.seconds).time()}")
```

### `network`

The state of the reader's connection.

| Field | Type | Meaning |
|---|---|---|
| `.online` | `Bool` | Whether the browser has a connection |
| `.effectiveType` | `String` | `"4g"`, `"3g"`, and so on |
| `.saveData` | `Bool` | Whether the reader asked to save data |
| `.downlink` | `Number` | The estimated bandwidth, in megabits a second |
| `.queued` | `Number` | Writes waiting for the connection, with `offline.sync` |

```wf
if !network.online { Alert("You are offline — changes are queued.").warning }
```

### `update`

A new version of a site that has `offline` in its config.

| Field | Type | Meaning |
|---|---|---|
| `.available` | `Bool` | Whether a new version has installed and is waiting |
| `.apply()` | — | Takes the new version and reloads the page once |

```wf
if update.available { Button("Reload for the new version") { on click { update.apply() } } }
```

### `locale` and `dir`

The reader's current locale (`"en"`, `"ar"`) and its direction (`"ltr"` or
`"rtl"`). Both follow `setLocale`.

```wf
Text("Reading in {locale}, {dir}")
```

### `params`

The current route's parameters, by name. A page also receives each one as
a typed parameter of its own, which is usually the better way to read it.

```wf
Text("Order {params.id}")
```

## String methods

A string has the browser's methods and five of WebFluent's own. A method
it has neither of is a `T16`, with the nearest one it has.

WebFluent's own:

| Method | Returns |
|---|---|
| `capitalize()` | The text with its first letter in upper case |
| `truncate(n)` | The first `n` characters, followed by `…` when the text was longer |
| `dedent()` | The text without the indentation its lines share, and without blank first and last lines |
| `lines()` | A list of its lines, however they end |
| `words()` | A list of its words, split on runs of whitespace |

The browser's: `length`, `trim()`, `trimStart()`, `trimEnd()`,
`toLowerCase()`, `toUpperCase()`, `toLocaleLowerCase()`,
`toLocaleUpperCase()`, `includes(s)`, `startsWith(s)`, `endsWith(s)`,
`indexOf(s)`, `lastIndexOf(s)`, `search(re)`, `slice(a, b)`,
`substring(a, b)`, `substr(a, n)`, `charAt(i)`, `charCodeAt(i)`,
`codePointAt(i)`, `at(i)`, `replace(a, b)`, `replaceAll(a, b)`,
`split(sep)`, `match(re)`, `matchAll(re)`, `concat(s)`, `padStart(n, c)`,
`padEnd(n, c)`, `repeat(n)`, `normalize()`, `localeCompare(other)`,
`isWellFormed()`, `toWellFormed()`, `toString()`, `valueOf()`.

`contains(s)` is accepted for `includes(s)`, and `toUpper()` and
`toLower()` for `toUpperCase()` and `toLowerCase()`.

```wf
derived preview = post.body.dedent().lines().first() ?? ""
```

## List methods

A list has the browser's array methods and seven of WebFluent's own.

WebFluent's own:

| Method | Returns |
|---|---|
| `sortBy(f)` | A sorted copy, ordered by what `f` returns for each item |
| `groupBy(f)` | A map from what `f` returns, as a string, to the items that returned it |
| `unique()` | A copy without repeats; records are compared by their contents |
| `take(n)` | The first `n` items |
| `first()`, `last()` | The first or last item, or `null` when the list is empty |
| `sum()` | The total of a list of numbers |
| `remove(i)` | The list without the item at index `i`; out of range, an unchanged copy |

The browser's: `length`, `map(f)`, `filter(f)`, `find(f)`, `findIndex(f)`,
`findLast(f)`, `findLastIndex(f)`, `some(f)`, `every(f)`, `includes(x)`,
`indexOf(x)`, `lastIndexOf(x)`, `join(sep)`, `slice(a, b)`,
`concat(other)`, `reverse()`, `sort(f)`, `reduce(f, init)`,
`reduceRight(f, init)`, `forEach(f)`, `flat()`, `flatMap(f)`, `fill(x)`,
`copyWithin(t, a, b)`, `entries()`, `keys()`, `values()`, `at(i)`,
`with(i, x)`, `toReversed()`, `toSorted(f)`, `toSpliced(…)`, `toString()`,
`toLocaleString()`; and, in an action or a handler, `push(x)`, `pop()`,
`shift()`, `unshift(x)` and `splice(i, n)`. `contains(x)` is accepted for
`includes(x)`.

How the methods treat a state:

- `sort` and `reverse` read in an expression work on a copy, so reading a
  sorted list never changes it.
- `push(x)`, `remove(i)` and `sort(f)` as a statement give the state a new
  list, so whatever reads it repaints and a `persist` writes it.

```wf
derived leaders = players.sortBy(p => -p.score).take(3)
```

Every one of WebFluent's own helpers also runs at build time, so the
pre-rendered page shows its result. A member something declares wins over
a helper of its name: a store's own `action first(n)` or
`action preview(path)` is the store's, called with its arguments, and a
worker's `css.last(kind)` is the worker's.

## Number methods

`toFixed(n)`, `toPrecision(n)`, `toExponential(n)`, `toLocaleString()`,
`toString()`, `valueOf()`.

To convert, use `Number("12")` and `String(n)`, or `"{n}"`. To round, use
`Math.round`, `Math.floor`, `Math.ceil`, `Math.max` and the rest of
`Math`. For text a reader sees, prefer [`format`](#format).

```wf
derived share = (part / whole * 100).toFixed(1)
```

## The built-in types

Eleven types come with the language. Each is a plain JSON value at run
time.

| Type | Written | Methods and fields |
|---|---|---|
| `Date` | `@2026-03-14` | `.year() .month() .day() .weekday()`, `.plus(days: 3)`, `.minus(…)`, `.isBefore(d) .isAfter(d) .isSame(d)`, `.until(d)`, `.startOfDay() .startOfWeek() .startOfMonth() .endOfDay()`, `.native()` |
| `Time` | `@09:30` | `.hour() .minute() .second()`, `.plus(minutes: 30)`, the comparisons |
| `DateTime` | `@2026-03-14T09:30Z` | the above, `.date() .time()`, `.inZone("Europe/Berlin")` |
| `Duration` | `90.minutes`, `3.days`, `250.ms` | `.days() .hours() .minutes() .seconds() .ms()`, `.plus(d)` |
| `Money` | `€12.99`, `$9.99`, `£4.50` | `.plus(m) .minus(m) .times(n) .convert(rate, "USD")`, `.amount`, `.currency` |
| `Url` | a string | `.host() .path() .query()`, `.with(query: { page: 2 })` |
| `Email` | a string | `.domain()` |
| `Color` | `#0F766E` | `.mix(other, 0.2) .lighten(n) .darken(n) .alpha(0.5) .contrast(other)` |
| `Uuid` | `uuid()` | a string's |
| `File` | from `FileUpload` | `.preview()`, `.name`, `.size`, `.type` |
| `Secret` | — | none: it may not be shown, spliced, logged or persisted |

`.plus` and `.minus` take `years`, `months`, `weeks`, `days`, `hours`,
`minutes`, `seconds` and `ms`. A duration is written with `ms`, `seconds`,
`minutes`, `hours`, `days` or `weeks`. What the language computes is a call
(`due.year()`); what the value already carries is a field
(`price.amount`). [Types](https://webfluent.monzeromer.dev/learn/12-types.md#the-types-the-language-brings-with-it)
has the rest.

```wf
derived checkout = arrival.plus(days: nights)
```

## Browser globals

These names compile to themselves, with no prefix: `window`, `document`,
`console`, `navigator`, `location`, `history`, `screen`, `performance`,
`crypto`, `globalThis`, `localStorage`, `sessionStorage`, `JSON`, `Math`,
`Date`, `Intl`, `Symbol`, `Reflect`, `setTimeout`, `setInterval`,
`clearTimeout`, `clearInterval`, `requestAnimationFrame`,
`cancelAnimationFrame`, `queueMicrotask`, `structuredClone`, `matchMedia`,
`getComputedStyle`, `parseInt`, `parseFloat`, `Array`, `Object`, `String`,
`Number`, `Boolean`, `Promise`, `Error`, `Map`, `Set`, `RegExp`,
`Infinity`, `NaN`, `undefined`, `encodeURIComponent`,
`decodeURIComponent`, `encodeURI`, `decodeURI`, `atob`, `btoa`, `alert`,
`confirm`, `prompt`, `URL`, `URLSearchParams`, `FormData`, `Blob`, `File`,
`FileReader`, `Event`, `CustomEvent`, `AbortController`, `TextEncoder`,
`TextDecoder`, `Notification`.

The checker types each of them as `Any`. A name the program declares
itself wins over a global, and unlike the language's own names, a global
may be the name of a state, a prop or a parameter.

```wf
localStorage.setItem("last-seen", now.date())
```

## Next

[Design tokens](https://webfluent.monzeromer.dev/reference/tokens.md).
