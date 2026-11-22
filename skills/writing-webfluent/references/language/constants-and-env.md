### Constants and `env`

```wf
const API = "/api/v1"
const PAGE_SIZE: Number = 20
resource rows: [Row] = http.get("{API}/rows?limit={PAGE_SIZE}")
Text(env.PUBLIC_APP_NAME)
```

`const` declares a top-level value read everywhere; `env.X` reads the
`"env": { "PUBLIC_APP_NAME": "…" }` map of `webfluent.app.json`, the
shell's variables and a `.env` file, fixed at build time. A page, a
component, a store or an `api` may only read a name that begins `PUBLIC_`
or that `public_env` lists — everything it reads is in the bundle, so any
other name there is a compile error. `env` is a record of exactly the
names the build knows, each typed by its value (`String` for text), so
`env.PUBLIC_APII` is `T05` with the nearest name; a name that may be unset
is given a default in the config's `env`. `env` is the language's name for
these settings and nothing else's: a `state`, prop, parameter, loop item or
`const` called `env` is `E118` (call it `target` or `stage`).

The language's other values are typed the same way: `query` is `[String:
String]` (`query.tab ?? "all"`), a page's `params` a record of its own
route parameters, `locale` and `dir` `String`s, and a form handle's
`errors` and `touched` `[String: String]` and `[String: Bool]`, by field.

