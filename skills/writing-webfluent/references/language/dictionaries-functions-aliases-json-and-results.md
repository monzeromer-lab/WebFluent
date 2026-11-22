### Dictionaries, functions, aliases, JSON and results

```wf
type Scores = [String: Number]               // an alias: another name for a type
type Settings { theme: String, columns: Number }
enum Region { north, south }

store Prefs {
    state scores: Scores = { ada: 3, bo: 5 }  // a dictionary: keys of one type, values of another
    state prices: [Region: Number] = { north: 10, south: 20 }
    state raw: Json = Json.null              // JSON as it arrives
    action total() -> Number {               // what an action returns
        return scores.values().sum()
    }
}

component Label(_ region: Region, describe: (Region) -> String) {   // a function type
    Text(describe(region))
}

page Restore(path: "/", title: "Restore", description: "Settings decoded from text.") {
    use Prefs
    state text = "{\"theme\": \"dark\", \"columns\": 3}"
    derived byTheme = (a: Settings, b: Settings) => a.theme.localeCompare(b.theme)
    Heading("Restore").h1
    match Settings.decode(JSON.parse(text)) {   // a result: Settings ! DecodeError
        .ok(s) { Text("{s.theme}, {s.columns} columns") }
        .err(e) { Alert("{e.at} {e.message}").danger }   // `$.columns` · `is "3", not a Number`
    }
    Text("{Prefs.total()} points; Ada has {Prefs.scores.ada ?? 0}")
    Label(.south, describe: r => "Region {r}")
}
```

- **`[K: V]`** — keys of `String` or an enum, values of `V`. Written as a
  map literal where one is wanted; reading a key (`d["k"]`, `d.k`) gives a
  `V?`. Methods: `keys()`, `values()`, `entries()` (each `{ key, value }`),
  `has(k)`, `with(k, v)` and `without(k)` (copies).
- **`(A, B) -> R`** — a function; `()` is nothing (`() -> ()`). A lambda
  handed to one is held to its parameters and its result.
- **`type Name = T`** — an alias, read as `T` (`type Admin = User { … }`
  with a block still extends a record).
- **`action f() -> T`** — what it returns; every `return` is held to it and
  callers see `T`. A lambda's parameter may say its type: `(a: User) => …`.
- **`Json`** — an enum built into the language: `.object(fields: [String:
  Json])`, `.array(items: [Json])`, `.string(value)`, `.number(value)`,
  `.bool(value)`, `.null`. A value is written `Json.null`,
  `Json.string("x")` — never `.null`, since JSON is a plain value at run
  time — and read by `match`. Any plain data fits where `Json` is wanted.
- **`T.decode(json)`** — on every declared record, enum and alias: the value
  checked against the type as the page runs, giving `T ! DecodeError`.
  `DecodeError` is `{ at: String, message: String }`. A field the JSON
  leaves out that declares a default is given it, as a record built in the
  program is — here, in an answer an endpoint or a `resource` declares, a
  `data` file, a template's data and a test's `data:` alike.
- **`T ! E`** — a result: `.ok(value)` or `.err(error)`, matched like an
  enum. `T!` is a `T` or a request's failure; `-> ! E` and `-> !` declare
  an action that returns nothing but may fail.

