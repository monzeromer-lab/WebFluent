### Built-in types

The language has eleven built-in value types for dates, times, durations, money, URLs, emails, colours, ids, files and secrets.

Eleven types come with the language. Each is a **plain JSON value at run
time** — no wrapper — so it crosses a request, a `persist`, the static
paint and the template engine unchanged, and `.native()` hands a real
platform object to any library that wants one.

| Type | Carried by | Written | What it can do |
|---|---|---|---|
| `Date` | `"2026-03-14"` | `@2026-03-14` | `.year() .month() .day() .weekday()`, `.plus(days: 3)`, `.minus(…)`, `.isBefore(d) .isAfter(d) .isSame(d)`, `.until(d)`, `.startOfDay() .startOfWeek() .startOfMonth() .endOfDay()`, `.native()` |
| `Time` | `"09:30"` | `@09:30` | `.hour() .minute() .second()`, `.plus(minutes: 30)`, the comparisons |
| `DateTime` | ISO 8601 | `@2026-03-14T09:30Z` | the above, `.date() .time()`, `.inZone("Europe/Berlin")` |
| `Duration` | milliseconds | `3.days`, `90.minutes`, `250.ms` | `.days() .hours() .minutes() .seconds() .ms()` |
| `Money` | `{ amount: 1299, currency: "EUR" }` | `€12.99`, `$9.99` | `.plus(m) .minus(m) .times(n) .convert(rate, "USD")`, the fields `.amount` and `.currency`; `format(m)` needs no style |
| `Url` | a string | `"https://example.com"` | `.host() .path() .query()`, `.with(query: { page: 2 })` |
| `Email` | a string | `"ada@example.com"` | `.domain()` |
| `Color` | a string | `#0F766E` | `.mix(other, 0.2) .lighten(n) .darken(n) .alpha(0.5) .contrast(other)` |
| `Uuid` | a string | `uuid()` makes one | the methods of a string |
| `File` | the browser's `File` | what `FileUpload` yields | `.preview()`, the fields `.name .size .type` |
| `Secret` | a string | — | nothing: it may not be shown, logged, persisted or put in text |

Minor units for money, so nothing drifts. Milliseconds for a duration, so
it adds to a moment. Everything the language computes is a **call**;
everything the carrier already has is a **field** — so `price.amount` is a
field and `due.year()` is a call.

```wf
type Booking {
    id: Uuid
    guest: Email
    from: Date
    nights: Number(1..=30)
    total: Money
}

state due: Date = @2026-03-14
derived checkout = due.plus(days: 5)          // 2026-03-19
derived late = now.date().isAfter(checkout)
Text("Due {format(total)} on {checkout:.date(long)}")
```

A literal written where one of these is wanted is read as one and held to
its shape: `state site: Url = "example"` is an error where it is written.
A `Url`, an `Email`, a `Color` and a `Uuid` have one spelling, and fit
wherever a `String` does. A `Date`, a `Time` and a `DateTime` have many, so
one is shown through a splice that says which — `Text("{due}")`,
`Text("{due:.date(long)}")` — and `Text(due)` is `T34`. A `Duration` adds to
a duration and is scaled by a number; `3.seconds + 500` is `T34`
(`500.ms`), and `wait.ms()` is the number it counts.
A `type` the program declares wins over the name — a project with its own
`type Color { … }` means that one.

`now` joins `viewport`, `query`, `hash` and `theme` as a browser value: a
`DateTime` that keeps itself current, every minute by default, or as often
as a page asks — `now(every: 1.seconds)`. Read it where it is shown or in a
`derived` value: a `state` takes it once, so `state clock = now(every:
1.seconds)` stays at the moment the page opened (`X06`). The static paint, a template and a test that only looks show
the moment of the build or render — `SOURCE_DATE_EPOCH`, when set, pins it —
and `locale` and `dir` are the page's (its `lang:`, else the default locale).

**A condition on a type.** Any type may say what its values must be, and
the checker holds every value it can read to it:

```wf
state nights: Number(1..=30) = 12
state pass: String(minLength: 8) = "correcthorse"
state start: Date(after: @2026-01-01) = @2026-03-14
```

`min`, `max`, `minLength`, `maxLength`, `pattern`, `after`, `before`, and a
range for the two ends. A value that arrives at run time is validation's to
refuse, from the same condition.

**A secret is not a string.** `Secret` is refused by the compiler wherever
it would escape: shown by an element, spliced into text (which is how one
ends up in a URL), logged, or kept with `persist`. Each is a `T12`.

A record is built with named fields, `Todo(id: "1", title: "x")`, and one
whose fields all have defaults with none, `Offers()` — a prop's default
too, `offers: Offers = Offers()`; a case is written `.calm`, or with its
enum where nothing says which, `Tone.calm` (`Status.failed("x")` with its
payload). The type checker reads every declared type and infers the
rest (literals, records, lists, the built-in methods of strings, numbers and
lists, store members, lambdas). A value's type is always one the program
can name: a signature says its types, a body's are inferred, and `Any` and
`Map` are not types (`T23`, `T24`) — JSON whose shape is open is `Json`. See
[Compiler diagnostics](#compiler-diagnostics) for what it reports.

A record may extend another, and a case may carry a payload:

```wf
type Admin = User { role: String }                       // every field of User, plus role
enum Status { idle, failed(reason: String), done(count: Number, label: String) }

state s: Status = .idle
action fail() { s = .failed("timed out") }
match s {
    .idle { Text("…") }
    .failed(r) { Text("Failed: {r}") }                   // one name per part of the payload
    .done(n, l) { Text("{n} {l}") }
}
derived note = match s { .failed(r) { r } else { "" } } // one name in a match expression
```

A case with a payload is `["failed", "timed out"]` at run time and a bare
case its name, so `s == .idle` still compares. A map literal has the
shape it was written with: `state form = { name: "", age: 0 }` lets the
checker read `form.name` and refuse `form.nam` (`T05`); a field only some
items of a list have is `T?`; an empty `{}` or a spread says nothing about
the keys.

