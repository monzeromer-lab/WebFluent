# 40. Built-ins

<!--
route: guide/built-ins
group: reference
blurb: Every function, browser value, method and global the language gives a program, in one place.
description: Built-in functions, browser values like viewport, query, now and network, string and list helpers, the built-in types, and globals.
-->

## Functions

| Function | Does | Where |
|---|---|---|
| `log(x, …)` | Prints to the console | anywhere |
| `navigate(path)` | Moves to a route; in a static build, a page load | actions, handlers |
| `format(value, .style, option)` | A number, date or money as text, in the locale | anywhere ([chapter 14](14-expressions.md#format-and-ago)) |
| `ago(date)` | "3 minutes ago", "in 2 days" | anywhere |
| `t("key", { name: value })` | A translation, with placeholders and plurals | anywhere ([chapter 21](21-i18n.md)) |
| `setLocale("ar")` | Switches the locale, and the direction with it | actions, handlers, effects |
| `setTheme("dark")` | `"light"`, `"dark"` or `"system"`, kept across visits | actions, handlers |
| `uuid()` | A fresh `Uuid` | anywhere |
| `sanitize(html)` | Markup through an allow-list, for `Unsafe.Html` | anywhere ([chapter 23](23-security.md#markup-you-did-not-write)) |
| `fetch(url, options)` | A request through the same engine as `api`; `await` it | actions ([chapter 17](17-data.md#await-fetch-in-an-action)) |
| `optimistic(holder, change)` | Shows a change now, takes it back if the action throws | actions ([chapter 17](17-data.md#showing-a-change-before-the-server-agrees)) |
| `beacon(url, data)` | A small post that survives the page unloading | anywhere ([chapter 18](18-realtime.md)) |
| `animate(element, name, duration)` | Plays an animation on a handle | handlers ([chapter 20](20-motion.md#driving-one-yourself)) |
| `replayAnimation(element, name)` | Plays an element's own animation again | handlers |
| `every(interval) { }`, `after(interval) { }` | A repeating and a one-off timer, stopped when their scope leaves | render blocks ([chapter 8](08-state-and-reactivity.md#timers)) |
| `ws(url)`, `sse(url)`, `broadcast(name)`, `rtc(signal: …)` | Open a socket, a stream, a channel, a peer — after `socket x =`, `stream x =`, `channel x =`, `peer x =` | render blocks ([chapter 18](18-realtime.md)) |

## Browser values

Always in scope and kept current; a `state` of the same name shadows one.

| Name | Holds |
|---|---|
| `viewport` | `.width`, `.height`, `.sm`, `.md`, `.lg`, `.xl` |
| `query` | the query string, as a map |
| `hash` | the fragment, without `#` |
| `theme` | `"light"`, `"dark"` or `"system"` |
| `now` | the current `DateTime`, every minute; `now(every: 1.seconds)` |
| `network` | `.online`, `.effectiveType`, `.saveData`, `.downlink`, `.queued` |
| `update` | `.available`, `.apply()` — a new version of an offline site |
| `locale`, `dir` | the current locale and direction, with i18n configured |
| `params` | the route's parameters, by name |

## String methods

`length`, `trim()`, `trimStart()`, `trimEnd()`, `toLowerCase()`,
`toUpperCase()`, `includes(s)`, `startsWith(s)`, `endsWith(s)`,
`indexOf(s)`, `lastIndexOf(s)`, `slice(a, b)`, `substring(a, b)`,
`charAt(i)`, `at(i)`, `replace(a, b)`, `replaceAll(a, b)`, `split(sep)`,
`padStart(n, c)`, `padEnd(n, c)`, `repeat(n)`, `normalize()`,
`localeCompare(other)`, `match(re)`, `search(re)` — the browser's — and
WebFluent's own:

| Method | Gives |
|---|---|
| `capitalize()` | The first letter in upper case |
| `truncate(n)` | At most `n` characters, with `…` when cut |
| `dedent()` | The text without the indentation its lines share |
| `lines()` | A list of its lines |
| `words()` | A list of its words, split on whitespace |

## List methods

`length`, `map(f)`, `filter(f)`, `find(f)`, `findIndex(f)`, `some(f)`,
`every(f)`, `includes(x)`, `indexOf(x)`, `reduce(f, init)`, `join(sep)`,
`slice(a, b)`, `concat(other)`, `reverse()`, `sort(f)`, `flatMap(f)` — the
browser's; `push(x)` in an action — and WebFluent's own:

| Method | Gives |
|---|---|
| `sortBy(f)` | A sorted copy, by what `f` returns |
| `groupBy(f)` | A map from what `f` returns to the items that returned it |
| `unique()` | A copy without repeats |
| `take(n)` | The first `n` items |
| `first()`, `last()` | The first or last item, or `null` |
| `sum()` | The total of a list of numbers |

Every helper runs at build time too, so a static page shows its result.

## Number methods

`toFixed(n)`, `toString()`. Convert with `Number("12")` and `String(n)`, or
`"{n}"`; round with `Math.round`, `Math.floor`, `Math.max` and the rest.

## The built-in types

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
`minutes`, `seconds` and `ms`. Durations are written with `ms`, `seconds`,
`minutes`, `hours`, `days` and `weeks`. [Chapter 13](13-types.md#the-types-the-language-brings-with-it)
has the rest.

## Browser globals

These compile to themselves, with no prefix: `window`, `document`,
`console`, `navigator`, `location`, `localStorage`, `sessionStorage`,
`JSON`, `Math`, `Date`, `setTimeout`, `setInterval`, `clearTimeout`,
`clearInterval`, `parseInt`, `parseFloat`, `Array`, `Object`, `String`,
`Number`, `Boolean`, `Promise`, `Error`, `Map`, `Set`, `RegExp`, `Infinity`,
`NaN`, `undefined`, `encodeURIComponent`, `decodeURIComponent`,
`encodeURI`, `decodeURI`, `atob`, `btoa`, `alert`, `confirm`, `prompt`,
`requestAnimationFrame`, `cancelAnimationFrame`, `history`, `screen`, `performance`, `crypto`, `globalThis`, `Intl`, `Symbol`, `Reflect`, `URL`, `URLSearchParams`, `FormData`, `Blob`, `File`, `FileReader`, `Event`, `CustomEvent`, `AbortController`, `TextEncoder`, `TextDecoder`, `Notification`, `matchMedia`, `getComputedStyle`, `structuredClone`, `queueMicrotask`. The checker
types them as `Any`, and a name the program declares itself wins over them.

## Next

[Design tokens](41-design-tokens.md).
