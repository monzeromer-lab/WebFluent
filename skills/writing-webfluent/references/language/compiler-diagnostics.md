## Compiler diagnostics

Every finding the compiler reports has a code, a severity, a location, a hint and, often, a fix.

`wf build`, `wf check`, `wf serve` and the editor report the same findings, from one
pipeline (`diagnostics::check::check_project`). Every stage runs whatever
the one before it found, and every file is read even when another does not
parse (the parser picks up again at the next declaration), so one build
shows every mistake once. Each finding has a code, a severity, a span, a
hint, the other places it concerns and, where there is one, a fix:

```text
error[T05]: `User` has no field `nmae`
 --> src/pages/Profile.wf:3:18
  |
3 |     Heading(user.nmae).h1
  |                  ^^^^
  = help: Its fields are `id`, `name`
  = docs: https://webfluent.monzeromer.dev/errors/T05
```

The place comes first in the `file:line:col` form problem matchers read;
colour only on a terminal (and never with `NO_COLOR`); one summary line at
the end (`error: the build stopped: 2 errors, 5 warnings`). A build that
stops on its findings exits `1`; one that could not run — a file it could
not read, a config it could not load — exits `2`. `wf serve`'s overlay and
`/__wf/status` carry the structured list. Errors stop the build; warnings
do not. The codes live in one registry (`diagnostics::codes`), which the
guide's chapter, the editor's links and the tests read.

`"lints": { "A11": "off", "U": "error", "R01": "warn" }` in the config says
what a finding counts as, by code or family. A warning may be turned off or
raised; an error may be lowered only when it cannot ship a broken page
(`R01`, `C02`, `S04`, `I01`, `I02`, `I04`).

One finding on one line is accepted with `// wf-allow(U01)` (codes or
families, comma-separated): on a line of its own it covers the next line of
code, at the end of a line that line. Only what `lints` could lower may be
allowed, and an allow that covers nothing is `U07`.

`--format json|sarif|github` on `wf build` and `wf check` writes the
findings for a tool (one document on standard output for JSON and SARIF,
with the progress moved to standard error; `::error` annotations for a
GitHub workflow); `--deny-warnings` fails on a warning; `wf explain T05`
prints a code's entry. Where the fix is known it is part of the finding —
the editor's quick fix and the JSON's `fixes`: the nearest name for `T05`,
`T06`, `T16`, `T13`, `E101`, `E103`; `?.` for `T04`; the missing props for
`C01`; the missing arms for `T15`; the flag for `V01`; `alt: ""` for `A01`;
`derived` for `X06`; removing it, or an `_` before it, for `U01`–`U05`; a splice for `T18`
(`"+" + n` → `"+{n}"`).

| Family | What |
|---|---|
| `E` | syntax and structure |
| `T` | types |
| `C` | components and their props |
| `R` | routes and navigation |
| `X` | state and reactivity |
| `F` | forms and bindings |
| `I` | translations |
| `D` | data, assets and what is kept in the browser |
| `N` | requests: results, error enums and `try` |
| `A` `S` `P` `U` `V` | accessibility, search, persisted values, unused, vocabulary |

**Errors — the program cannot mean what it says.**

| Code | What it means |
|---|---|
| `E001` | Text the lexer cannot read: a stray character, a string or comment that never closes |
| `E002` | A syntax error: what was expected, and what was found |
| `E003` | `.wfx` indentation that lines up with no block |
| `E004` | A file in the WebFluent 2 grammar (`wf migrate`) |
| `E005` | A file `wf fmt` will not change: formatting would change what it says, so it is left as it is |
| `E101` | A component nothing declares (an element, a `layout:`, a part) |
| `E102` | A name declared twice in one scope — two stores, a state and a derived — with the first as a related place |
| `E103` | A flag, case, part, event, slot or `emit` the registry or the component's declaration does not know (`Button has no flag or enum case `huge``, with the ones it takes) |
| `E104` | A `layout:` component with no default slot |
| `E105` | A statement a render block cannot hold (`n = n + 1` loose in a page) |
| `E106` | An `on*` attribute: script in an attribute; or `style:`, a value as the element's whole inline style |
| `E107` | A `href`/`src`/`to` literal naming a scheme a browser runs; a `head { script(src:) }` or refresh whose address a value chooses |
| `E108` | A non-public `env` name read from a page |
| `E109` | An element a PDF or a slide deck cannot draw |
| `E110` | A project script that cannot be a plain script (a module, a `public/` clash, a name something else means) |
| `E111` | A setting that cannot work (`meta.scripts` module without `as`, `build.elements`, `offline`, a missing `openapi.json`) |
| `E113` | An argument that cannot be positional |
| `E114` | `emit` outside a component |
| `E116` | `Element`'s tag is not a custom element's |
| `E118` | A name the language already has, declared as the program's own — `state env`, `for t in …`, a prop `query`, a `const format` |
| `E120` | Something nests more than 100 levels deep — brackets, blocks, the links of one chain (`a.b.c`), splices, or in a template components that place one another |
| `E121` | A template render that would do too much: more than ten million elements, loop items and units of evaluation, 256 MB of markup, or an expression that makes more than one evaluation may |
| `E901` | The compiler wrote JavaScript a browser would refuse (it reads back every script it writes — a WebFluent bug) |
| `E902` | A page the content security policy shipped beside it would block |
| `C02` | A prop your component does not declare |
| `S04` | Two pages claim one route |

And what the type checker finds:

| Code | What it means |
|---|---|
| `T01` | A value of the wrong type given to a prop, a state, a field, a parameter or an assignment — `` `count` of `C` is `String`, but `Number` is wanted ``; a `Carousel(interval:)` outside 1–2147483647 ms, a `Column(span:)` not a whole 1–12 |
| `T02` | A case the enum does not have |
| `T04` | A value that may be `null` read as if it were not; `if let`, `??` or a `!= null` check narrows it |
| `T05` | A field or method a record does not have |
| `T06` | A member or action a store does not have |
| `T07` | A list, record or action used as a condition, which is always true |
| `T08` | A `for` over something that is not a list |
| `T09` | An `emit` whose arguments do not match the event's |
| `T10` | A call with the wrong number or kind of arguments |
| `T11` | A `match` on something that is neither a read nor an enum, or with arms of the wrong kind |
| `T12` | A `Secret` where it would escape — shown, spliced into text, joined with `+`, handed to a browser global (`console`, storage), logged, or kept with `persist`. A secret has no string members |
| `T13` | A name, or a function called, that nothing declares — a ReferenceError in the browser. `event` (`e`), `value` and `key` are names only inside a handler. Not applied to a template rendered with data |
| `T14` | A comparison that is always the same: `"1" == 1`, a record against a string, an enum against a case it does not have |
| `T15` | A `match` with no `else` that misses a case of the enum, the result or the read, or one with a case twice. A `match` expression that covers every case needs no `else` |
| `T20` | A list or a record shown as text — a list's items run together, a record shows `[object Object]` |
| `T21` | A `match` over a read with no `.failed` arm and no `else`: a failed request shows nothing |
| `T16` | A method a number, a string or a list does not have, with the nearest one that it does (`joined` → `join`) |
| `T17` | An async action's result used before it is awaited; `await` outside an action, a handler, a timer or a hook (in a `derived` value or an effect); `.pending` on an action that awaits nothing |
| `T18` | Arithmetic on something that is not a number (`"12" * 2`, `items - 1`), or `+` between text and a number (`"Total: " + n` — a splice says how it shows: `"Total: {n}"`) |
| `T22` | A handler that names more values than its event sends — `on click(e, x)`, or `on menu(a, b, c, d)` for an event of three |
| `T26` | An action that returns a value without `-> T`; the fix writes the type it returns |
| `T27` | Two values with no type in common where one value is made of them — an `if`'s or a `match`'s branches, a list's items, the sides of `??`: `if ok { 1 } else { "none" }`. `null` on one side makes it `T?` |
| `T28` | A `Json` value read as if it were something else — `raw.name`, `raw[0]`, `raw + 1`, a method — rather than matched or decoded (`User.decode(raw)`) |
| `T29` | A condition — `if`, `show`, `!`, `&&`, `\|\|` — on something that is not a `Bool` (a `Bool?` is one, its `null` false): `if count` is `count != 0`, `!name` is `name == ""`, `user \|\| x` is `user ?? x`; the fix writes the comparison |
| `T30` | A browser global, or a member of an element, an event or a browser object, the language does not type — `navigator.bluetooth`, `e.kye`; reach it from a project script with JSDoc |
| `T23` | `Any` written as a type, which 7.0 took out of the language; the fix writes `Json` |
| `T24` | `Map` written as a type, which 7.0 took out of the language — a `type` with its fields, `[String: T]`, or `Json`; the fix writes `Json` |
| `T25` | A `state` or `persist` whose first value does not say what it holds — `[]`, `{}`, `null`, open JSON; the fix writes the type the body assigns it |
| `T31` | A project script's function called from `.wf` with no JSDoc, or a JSDoc type the language cannot say (a union, a bare `Array`); the fix writes JSDoc from the call |
| `T32` | A parameter written with no type — an action's, an event's, a slot's — or a lambda's that nothing it is passed to types; the fix writes what every call passes |
| `T33` | Data from outside the program that is not its type, named by its place: a `data` file at build time (`$[3].date is "March 5", not a Date`), a template's data against its page's parameters, a test's `data:` seed against the state it seeds |
| `T34` | A conversion the language does not make: a `Date`, `Time` or `DateTime` where a `String` is wanted (`Text(due)` — `Text("{due:.date(long)}")`), or a `Duration` and a `Number` mixed (`3.seconds + 500` — `500.ms`) |
| `T35` | Two `derived` values — in one body or two stores — that each read the other, or one that reads itself, with none declared |
| `N01` | An endpoint's path naming `{name}` that no parameter fills, or a parameter in the path and in a header or cookie too |
| `N02` | A `resource` over a write, or `.update` on one: a resource and an update are a read's |
| `N03` | `try` in an action that declares no error, or whose error cannot hold the one passed up |
| `N05` | An `invalidates` naming no read of the same group, an `invalidates` on a read, or an argument that is not one of the write's own parameters; a service's `invalidations { }` naming an endpoint no group of it has, or a read of another group |
| `N06` | A `fake` that does not match its group's contract — an endpoint the group does not have, a parameter it does not take, an answer of the wrong type, one endpoint answered twice; a test's `fakes:` naming no fake, or two answering one endpoint; an `expect called` naming no endpoint |
| `N12` | An error enum without exactly one `http(error: HttpError)` case, a case with no status or not one value, or a status twice |
| `N14` | A service's `pagination { }` naming an endpoint no group of it has; a `pages` or `streamed` an endpoint cannot have: a write, a page parameter the endpoint lacks or the first call must give, a cursor with no `next:`, a page that is not a list with no `items:`, a streamed answer that is not a list; `for await` over anything but a streamed endpoint's call |
| `N08` | A form 7.0 removed: `fetch(…)`, which `http.get(…)` replaces; an endpoint that does not say how it fails (`-> T`), which `-> T!` replaces; an `errors { }` block, which an error enum replaces; a read's `loading`, `error(e)` and `ready(v)` arms, which `.loading`, `.failed(e)` and `.loaded(v)` replace; its `.data`, `.refreshing` and `.state`, which `.value`, `.isRefreshing` and `.isLoading`/`.isLoaded`/`.isFailed` replace; and `on:`, which `refresh:` replaces; a service's `headers { }` and `on request`/`on response`/`on error` hooks, which `middleware [ … ]` replaces; an endpoint with no `at`, and a path's `:name`, which `{name}` replaces; and a group's `base:`, `retry:` or `middleware`, which are its service's; and `paginate:` on a call, which the endpoint's `pages` replaces |
| `N11` | `Any` or `Map` where a service meets the program — an endpoint's parameter or answer, a resource's type — or an `http.get` with nowhere that names what it answers; the fix names `Json`, or declare a `type` |
| `X01` | An assignment to something that cannot change: a `const`, a `data` constant, a `derived` value (a store's too), a prop, a route parameter, a loop variable, an action |
| `C01` | A required prop or record field left out — on a call, a `layout:`, a record (`Todo(title: "x")` when `id` has no default), a built-in's (`Portal` without `into:`); a flag gives a `Bool` or an enum prop |
| `C03` | A part outside its owner (`Select.Option` with no `Select` around it) |
| `C07` | `drag:`, `drop…:`, `on receive`, `on dragmove`, `on longpress` or `on swipe` on a component whose body is not one element (an `if`, a `for`, several elements) |
| `X02` | An effect that writes, every run, something it reads — itself or through an action it calls — so it never settles |
| `X04` | A `derived` value that calls an action that assigns state |
| `X05` | `use` of a store nothing declares |
| `F01` | `bind:` to something that cannot be written: a constant, a derived value, a prop, a literal |
| `F02` | `bind:` to a state the control cannot hold (a text `Input` on a `Number` — `.number` holds one) |
| `F03` | `validate x` where no control binds `x`, or `x` is derived: its rules can never pass |
| `R01` | A link or `navigate()` to a path no page's route matches (spliced paths by their shape; a path with no leading `/`) |
| `R02` | A route's `:param` with no page parameter of that name (a parameter with no `:param` is the page's data: a template, left out of a build) |
| `R05` | A `translates:` that names no page (or its own), or two translations of one page in one language |
| `R03` | An `app` with no `Router` (no page is ever drawn), or with two |
| `E117` | An `on key("…")` combination no keyboard sends — an unknown modifier, no key, a key that does not exist, in any step of a chord |
| `D02` | A `persist` of a value storage cannot keep as JSON — a function, a `File`, a promise |
| `I01` | A literal `t("key")` that no locale's translations have, so the page shows the key |
| `I02` | A message and its call that disagree: a value passed the message does not use, or a placeholder the call does not pass |
| `I04` | `setLocale("xx")` naming a locale `i18n.locales` does not list |
| `V06` | A `$token` the resolved theme does not declare, so the property falls back to nothing |

An endpoint a service does not have is `T06`, and an argument it does not
take is `T10`.

There is no value of unknown type in a program that checks: `Any` and
`Map` written as types are `T23` and `T24` (the fix writes `Json`), and a
*name* nothing declares is `T13`. `T04` also covers an item at a
fixed index (`todos[0]`, which is nothing when the list is empty, unless a
condition like `todos.length > 0` encloses it) and a field that may be null
read after a `?.` (`sel?.note.length`). A check for `null` narrows in the
`else` branch and after `if x == null { return }`, and `xs.filter(x => x
!= null)` is a list of what is there (`[T?]` becomes `[T]`). A refined type
(`Number(1..=30)`) is held to every literal assigned to it, and a `derived`
value's annotation to what it works out to.

Where there is one right answer the compiler writes it instead of
reporting: a change inside a state's value — `form.name = v`,
`items[i].done = v`, `items.sort()` as a statement, `t.done = v` on a loop's
item in a handler — compiles to the state given a copy with that place
changed (`WF.setIn`, `WF.mutated`), so what reads it repaints and `persist`
writes it; reading `items.sort(…)` in an expression sorts a copy; a value
spliced into a `to:`, `navigate()`, `http` or resource address is
`encodeURIComponent`ed where it stands for a value (not when it opens the
address, is a whole query, or follows `#`), in the live page and the static
paint alike.

**Warnings.**

| Rule | What it means |
|---|---|
| `A01` | An `Image` with no `alt` (a decorative one takes `alt: ""`) |
| `A02` | An `IconButton` with no accessible name — `label:` is its name, never visible text |
| `A03` | An `Input` with neither `label:` nor `placeholder:` |
| `A04` | A `Checkbox`, `Radio`, `Switch`, `Slider` or `Textarea` with no `label:` |
| `A05` | A `Button` with no text — no label, no `aria-label:`, and no `Text` in its block (a branch or a component it places counts) |
| `A06` | A `Link` with no text |
| `A07` | A `Heading` with no text |
| `A08` | A `Modal`, `Dialog`, `AlertDialog` or `Sheet` with no `title:` (a `Sheet` may take a `label:`), which is its accessible name |
| `A09` | A `Video` with no `controls`, or a `Video`/`Audio` with no `captions:`/`transcript:` |
| `A10` | A `Table` with no header row (`Table.Head`) |
| `A11` | A heading level skipped — `h2` straight to `h4` |
| `A12` | A page with no `h1`, or more than one |
| `A13` | A theme's colour pairing falls below the WCAG AA contrast ratio |
| `A14` | A `role:` that requires particular children (`tablist` → `tab`, `list` → `listitem`, `menu` → `menuitem`, …) holds a control that is not one |
| `A15` | A control's `aria-label` does not contain its visible text — its label, the literals in its block, and what a component in the block renders from its literal props — so what a voice-control user says does not match what they see |
| `S01` | A page has no title |
| `S02` | A page has no description, so its search snippet is written for it |
| `S03` | A description longer than ~160 characters, which a search result truncates |
| `P01` | A `persist` at `version: n` with nothing to bring an older version forward, so what a returning reader had is discarded |
| `P02` | A `migrate` step above the declared `version:`, which never runs |
| `P03` | A `persist` inside a `store(scope: .route)`: the route change drops the store, and the next read builds it again from storage, so the value comes straight back |
| `V01` | A bare word in an argument that nothing in scope declares — a name misspelled, or a flag written without its dot (`did you mean `.center`?`); an `if let` binding or a case's payload is declared through its whole block |
| `V02` | A flag whose class no stylesheet — the engine's or one of the project's `.css` files — defines; the registry keeps this from happening for the built-ins |
| `V03` | An `Unsafe.Html`, and whether what it puts in went through `sanitize`; a `Frame(srcdoc:)` from a value with no `sandbox:` (given `sandbox=""`) |
| `V04` | A `class:` naming one of the built-ins' `wf-*` classes, which brings that built-in's rules with it |
| `U01`–`U02` | A `state` or `derived` value nothing in its page or component reads (an assignment alone does not read it) |
| `U03` | A component nothing places, names as a layout, or reaches as a part |
| `U04` | A store member — state, derived, action — nothing reads, inside the store or as `Store.member` |
| `U05` | An action nothing calls; a name that starts with `_` is understood to be unused on purpose |
| `U07` | A `// wf-allow(CODE)` that covers nothing it names, or names an error no allow may silence |
| `C04` | A named argument a built-in does not declare, written to the element as an attribute |
| `C05` | A positional argument a built-in does not take |
| `C06` | A positional argument bound to a component's first prop, which is not marked `_` |
| `D05` | A font, stylesheet or script from another origin with no `meta.integrity` hash |
| `N04` | A call's result awaited on its own, so a failure is never heard: match it or pass it up with `try` |
| `N09` | `retry:` on a call of a `POST` or `PATCH` its contract does not mark `idempotent`, which is not retried |
| `N07` | A service `wf api` keeps whose `generated/` files are behind its spec: `wf api sync` would change them |
| `D06` | A `meta.scripts` entry marked `"load": "consent"` in a program with no `ConsentBanner` to ask for it, so it never loads |
| `T19` | A value that may be null spliced into text, where it shows as `null` |
| `R04` | A relative `src:`/`href:` literal on a nested route, which resolves against the page's address |
| `F04` | `Checkbox(checked: x)` with no `bind:` and no `on change`: the box toggles, the state does not |
| `F05` | A `Select` whose bound state starts as none of its options |
| `D01` | A site-relative `src:`, `poster:`, `captions:`, `transcript:` or `spawn(…)` naming a file `public/` does not have |
| `D03` | A `persist` in a component placed more than once, or in a loop, which every instance shares; `key:` gives each its own (`persist open = false { key: item.id }`) |
| `D04` | A persisted value's shape changed since the last build with no `version:` raised (the shapes are kept in `.wf-cache/persist.json`) |
| `I03` | A message one locale has and another does not |
| `A16` | A literal `id:` inside a `for`, or in a component placed more than once, which several elements then share |
| `A17` | A `Frame` with no `title:`, which is how a screen reader announces it |
| `A18` | A `Popover` drawn as a dialog (no other `role:`) with no `label:` |
| `U06` | Code after a `return` in the same block |
| `U08` | A condition that is always the same — a literal, or a value compared with itself |
| `U09` | A `for` with no `by` whose items hold state, a control or a stateful component, which starts again when the list changes |
| `V05` | A style property no CSS defines (`colr:`), with the nearest one that it does |
| `V07` | A bare number spliced as a whole length (`width: {pct}`), which the browser drops |
| `U10` | A `match`'s `else` that no value reaches: every case has its arm |
| `E112` | A key in `webfluent.app.json` nothing reads, with the nearest one that is |
| `E115` | A project script the compiler could not read (still linked; its names are not in scope) |
| `V08` | An icon the runtime does not draw |
| `V09` | A handler for an event the element does not fire |
| `V10` | A `Host(tag: …)` it does not make — it makes `div`, `span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul`, `table`; any other is a `div` |
| `V11` | Information, not a warning: a component named like a built-in (`Footer`, `Chart`) takes its place, so every call of the name draws the project's own and the built-in cannot be placed there |
| `X06` | A `state` whose initial value reads `now`, `viewport`, `query`, `hash`, `theme`, `network` or `update` (outside a lambda) and that nothing assigns, binds or changes in place: it holds the value it started with, so `state clock = now(every: 1.seconds)` never ticks. The fix makes it `derived` |
| `I05` | Under `i18n.literals`, shown words no table can translate: text splicing more than a name, or a value shown by a name that does not say it is words |
| `V12` | A `VirtualList` whose body is not one keyed `for`: it draws every row |
| `E119` | A hidden direction control (U+202A–U+202E, U+2066–U+2069) in the source: Trojan Source |

The heading-outline rules (`A11`, `A12`) do not apply to `Presentation` or
`Document` output, where an `h1` per slide or per section is correct.

