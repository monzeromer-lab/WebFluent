# 39. Diagnostics

<!--
route: guide/diagnostics
group: reference
blurb: Every error and warning the compiler reports — what it means, a program that draws it, the message, and the fix.
description: Every WebFluent error and warning — structural errors and codes T, A, S, P, U and V — each with a program that draws it and the fix.
-->

`wf build`, `wf serve` and your editor report the same findings, from
one pipeline. Each starts with its code and what is wrong, then the place in
the `file:line:col` form editors and CI read, the line of source with the
part that is wrong underlined, what to do about it, and a link to its entry
here:

```text
error[T05]: `User` has no field `nmae`
 --> src/pages/Profile.wf:3:18
  |
3 |     Heading(user.nmae).h1
  |                  ^^^^
  = help: Its fields are `id`, `name`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t05
```

- **Errors** stop the build: the program cannot mean what it says.
- **Warnings** print and the build goes on: it means something, probably
  not what you wanted. Nothing is a warning that the compiler could have
  decided for itself.

Every stage runs whatever the one before it found, and every file is read
even when another does not parse, so one build shows every mistake once —
then one line sums them up: `error: the build stopped: 2 errors, 5
warnings`. A build that stops on its findings exits with `1`; one that
could not run at all — a file it could not read, a config it could not
load — with `2`.

The code — `T05` — is the thing to search this page for. The families:

| | |
|---|---|
| `E` | syntax and structure: what cannot be read, or refers to nothing |
| `T` | types |
| `C` | components and their props |
| `R` | routes and navigation |
| `X` | state and reactivity |
| `D` | data, assets and what is kept in the browser |
| `A` | accessibility |
| `S` | search and sharing |
| `P` | persisted values |
| `U` | declared and never used |
| `V` | vocabulary |

Every example below is compiled by the test suite, draws exactly the code
it is filed under, and shows what the compiler prints for it.

## Syntax and structure

Errors, except where an entry says otherwise.

### E001 — Text the compiler cannot read

```wf expect E001
page P(path: "/", title: "T", description: "D") {
    Heading("Hello).h1
}
```

```text
error[E001]: Unterminated string literal
 --> src/App.wf:2:13
  |
2 |     Heading("Hello).h1
  |             ^
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e001
```

**Fix:** Close the string, comment or splice the message names. A
character that begins nothing (`|` alone, a stray `@`) is usually a typo
for the one beside it on the keyboard.

### E002 — A syntax error

```wf expect E002
page P(path: "/", title: "T", description: "D") {
    Heading("Hello").h1
    Text("a" Text("b")
}
```

```text
error[E002]: Expected `,` between arguments, got `Text`
 --> src/App.wf:3:14
  |
3 |     Text("a" Text("b")
  |              ^^^^
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e002
```

**Fix:** The message names what was expected and what was found, at the
place the parser stopped. One file can report several: after an error the
parser picks up again at the next declaration.

### E003 — Indentation that lines up with no block

In a `.wfx` file, a line that comes back out to an indentation no enclosing
block has:

```text
page Home(path: "/")
    Row
        Text("a")
      Text("b")
```

**Fix:** Line the line up with the block it belongs to.

### E004 — A file in the WebFluent 2 grammar

```wf expect E004
Page Home (path: "/") {
    Text("Hi")
}
```

```text
error[E004]: `Page` is a WebFluent 2 declaration; this is WebFluent 3
 --> src/App.wf:1:1
  |
1 | Page Home (path: "/") {
  | ^^^^
  = help: Run `wf migrate` to convert the project to the current grammar
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e004
```

**Fix:** Run `wf migrate` once; it rewrites the project in place.

### E005 — A file the formatter will not change

`wf fmt` checks that its result has the same tokens as the file. When it
would not — something only the formatter's reading of the file would
change — the file is left as it is and named. **Fix:** format the lines it
names by hand; please report the file, since it is a bug in `wf fmt`.

### E101 — A component nothing declares

```wf expect E101
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Buton("Save")
}
```

```text
error[E101]: unknown component `Buton`: no `component Buton` is declared
 --> src/App.wf:3:5
  |
3 |     Buton("Save")
  |     ^^^^^
  = help: declare `component Buton { … }` or check the spelling
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e101
```

**Fix:** Check the spelling, or declare it with `component Buton { … }`.

### E102 — A name declared twice

```wf expect E102
page P(path: "/", title: "T", description: "D") {
    state count = 0
    derived count = 1
    Heading("Count {count}").h1
}
```

```text
error[E102]: `count` is declared twice: as a state at line 2, and as a `derived` here
 --> src/App.wf:3:5
  |
3 |     derived count = 1
  |     ^^^^^^^
  = help: A name means one thing — rename one of the two `count`s
  = note: `count` is first declared here at src/App.wf:2:5
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e102
```

**Fix:** Rename one of the two. The finding points at the second and names
where the first is.

### E103 — A flag, case, part, event or slot the component does not have

```wf expect E103
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button("Save").huge
}
```

```text
error[E103]: Button has no flag or enum case `huge`
 --> src/App.wf:3:19
  |
3 |     Button("Save").huge
  |                   ^^^^^
  = help: Its flags are .bounce, .button, .collapse, .danger, .disabled, .expand, .fadeIn, .fadeOut, .fast, .full, .info, .lg, .md, .normal, .outlined, .pill, .primary, .pulse, .reset, .rounded, .scaleIn, .scaleOut, .secondary, .shake, .slideDown, .slideLeft, .slideRight, .slideUp, .slow, .sm, .spin, .submit, .success, .warning
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e103
```

**Fix:** Use one the message lists — it names every flag, case, event or
slot the component takes.

### E104 — A layout with nowhere to put the page

```wf expect E104
component Shell { Text("frame") }
page P(path: "/", title: "T", description: "D", layout: Shell) {
    Heading("Home").h1
}
```

```text
error[E104]: `Shell` declares no default slot, so the page has nowhere to go
 --> src/App.wf:2:57
  |
2 | page P(path: "/", title: "T", description: "D", layout: Shell) {
  |                                                         ^^^^^
  = help: Add `slot` to the component and place `children` where the page belongs
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e104
```

**Fix:** Add `slot` to the component and place `children` where the page
belongs.

### E105 — A statement a render block cannot hold

```wf expect E105
page P(path: "/", title: "T", description: "D") {
    state n = 0
    Heading("Count").h1
    n = n + 1
}
```

```text
error[E105]: `n` is not an element or a statement a render block can hold
 --> src/App.wf:4:5
  |
4 |     n = n + 1
  |     ^
  = help: Code that does something goes in `on click { … }`, an `action` or an `effect`; an element's name is capitalised
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e105
```

**Fix:** Put it in `on click { }`, an `action` or an `effect`.

### E106 — Script in an attribute

```wf expect E106
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button("Save", onclick: "save()")
}
```

```text
error[E106]: `onclick` on Button would put script in an attribute
 --> src/App.wf:3:20
  |
3 |     Button("Save", onclick: "save()")
  |                    ^^^^^^^
  = help: Write the handler instead: `Button(…) { on click { … } }`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e106
```

**Fix:** `Button("Save") { on click { save() } }`.

### E107 — A URL a browser would run

```wf expect E107
page P(path: "/", title: "T", description: "D") {
    Heading("Go").h1
    Link("Go", to: "javascript:alert(1)")
}
```

```text
error[E107]: `to` on Link names the `javascript` scheme, which a browser runs
 --> src/App.wf:3:16
  |
3 |     Link("Go", to: "javascript:alert(1)")
  |                ^^
  = help: A URL here may be relative, or name http, https, mailto, tel, sms or ftp
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e107
```

**Fix:** A relative URL, or `http`, `https`, `mailto`, `tel`, `sms`,
`ftp`.

### E108 — An `env` name that is not public

`env.STRIPE_SECRET` read in a page: `env.STRIPE_SECRET is not public, and a
page reads what is in the bundle`. **Fix:** rename it `PUBLIC_…` or list it
in `public_env` — only if anyone may read it ([chapter
30](30-environments.md)).

### E109 — An element this output cannot draw

A `Button`, an `Input`, a handler or a `Router` in a project whose
`output_type` is `pdf` or `slides`. **Fix:** draw it as text, or build the
page as a site.

### E110 — A project script that cannot be a plain script

A `.js` file under `src/` with an `import` or `export`, one a file in
`public/` at the same address would replace, or one declaring a name the
program, the language or the browser already has. **Fix:** take off the
`import`/`export` (a top-level `function` is global as it is), keep one of
the two files, or rename the clashing name.

### E111 — A setting that cannot work

A `meta.scripts` module with no `as`, a `build.elements` naming a component
that does not exist, an `offline.fallback` no page has, an `openapi.json`
that is not there. **Fix:** what the message says; it names the setting.

### E112 — A setting nothing reads

Warning. A key in `webfluent.app.json` that is not a setting, with the
nearest one that is: `` `build.minfy` is not a setting, and nothing reads it
— did you mean `minify`? `` **Fix:** correct the key.

### E113 — An argument that cannot be positional

```wf expect E113
component Card(_ name: String, price: Number = 0) { Text(name) }
page P(path: "/", title: "T", description: "D") {
    Heading("Shop").h1
    Card("Laptop", 999)
}
```

```text
error[E113]: Only the first argument may be positional
 --> src/App.wf:4:20
  |
4 |     Card("Laptop", 999)
  |                    ^^^
  = help: Name the others: `Button("Save", tone: .primary)`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e113
```

**Fix:** Name the others: `Card("Laptop", price: 999)`.

### E114 — `emit` outside a component

```wf expect E114
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button("Save") { on click { emit saved() } }
}
```

```text
error[E114]: `emit` fires a component's event; a page has none
 --> src/App.wf:3:33
  |
3 |     Button("Save") { on click { emit saved() } }
  |                                 ^^^^
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e114
```

**Fix:** Fire the event from the component that declares it; a page calls
an action instead.

### E115 — A project script the compiler could not read

Warning. A `.js` file under `src/` the compiler's scanner could not read —
a string that never closes, say. The file is still linked, but its names
are not in scope for the program. **Fix:** what the message says, at the
line it names.

### E116 — A tag that cannot be a custom element's

```wf expect E116
page P(path: "/", title: "T", description: "D") {
    Heading("Pricing").h1
    Element("pricingtable")
}
```

```text
error[E116]: `pricingtable` is not a custom element's tag
 --> src/App.wf:3:5
  |
3 |     Element("pricingtable")
  |     ^^^^^^^
  = help: A custom element's tag is lower case with a hyphen: `Element("stripe-pricing-table", …)`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#e116
```

**Fix:** A custom element's tag is lower case with a hyphen:
`Element("pricing-table")`.

### E901 — The compiler wrote JavaScript a browser would refuse

The build reads back every script it writes; this one would not run. It is
a bug in WebFluent, not in your program. **Fix:** please report it, with
the source that produced it.

### E902 — Output its own security policy refuses

The build holds every page it writes to the content security policy it
ships beside it; something on this page — an inline script, a `style=`, a
script from an origin the policy never named — would be blocked. **Fix:**
what the message says; a library's origin goes in `meta.scripts`.

## Components

### C01 — A required prop or field left out

```wf expect C01
type Todo { id: String, title: String, done: Bool = false }
page P(path: "/", title: "T", description: "D") {
    state todo = Todo(title: "Write the docs")
    Heading(todo.title).h1
}
```

```text
error[C01]: `Todo(…)` leaves out `id` of `Todo`, which has no default
 --> src/App.wf:3:18
  |
3 |     state todo = Todo(title: "Write the docs")
  |                  ^^^^
  = help: Give it, or declare a default in the type: `field: Type = value`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#c01
```

**Fix:** Give it, or declare a default in the type (`id: String = ""`) or
on the prop.

### C02 — A prop the component does not declare

```wf expect C02
component Badge2(_ label: String) { Text(label) }
page P(path: "/", title: "T", description: "D") {
    Heading("Badges").h1
    Badge2("New", colour: "red")
}
```

```text
error[C02]: `Badge2` declares no prop `colour`
 --> src/App.wf:4:19
  |
4 |     Badge2("New", colour: "red")
  |                   ^^^^^^
  = help: It is passed anyway, but nothing in the component reads it
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#c02
```

**Fix:** Correct the spelling, or declare the prop on the component.

### C04 — An attribute a built-in does not declare

Warning.

```wf expect C04
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button("Save", colour: "red")
}
```

```text
warning[C04]: Button has no prop `colour`; it is written to the element as an attribute
 --> src/App.wf:3:20
  |
3 |     Button("Save", colour: "red")
  |                    ^^^^^^
  = help: A typo here does nothing on screen; check the component's props
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#c04
```

**Fix:** Check the prop's spelling — `wf registry` lists what the built-in
takes. `aria-*`, `data-*` and the global attributes (`id`, `role`,
`title`, …) are expected and draw nothing.

### C05 — A positional argument a built-in does not take

Warning.

```wf expect C05
page P(path: "/", title: "T", description: "D") {
    Heading("Rule").h1
    Divider("thin")
}
```

```text
warning[C05]: Divider takes no positional argument
 --> src/App.wf:3:13
  |
3 |     Divider("thin")
  |             ^
  = help: Name it: the registry lists the props it takes
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#c05
```

**Fix:** Name it, with one of the props the built-in takes.

### C06 — A positional argument bound by order

Warning.

```wf expect C06
component Tag2(label: String) { Text(label) }
page P(path: "/", title: "T", description: "D") {
    Heading("Tags").h1
    Tag2("new")
}
```

```text
warning[C06]: `Tag2` declares no positional prop; the argument binds to `label`
 --> src/App.wf:4:10
  |
4 |     Tag2("new")
  |          ^
  = help: Mark the prop: `component Tag2(_ label: …)`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#c06
```

**Fix:** Mark the prop the call means: `component Tag2(_ label: String)`.

## State and reactivity

### X01 — An assignment to something that cannot change

```wf expect X01
const LIMIT = 3
page P(path: "/", title: "T", description: "D") {
    Heading("Limit {LIMIT}").h1
    Button("More") { on click { LIMIT = 4 } }
}
```

```text
error[X01]: `LIMIT` is a `const`, and nothing may assign to it
 --> src/App.wf:4:33
  |
4 |     Button("More") { on click { LIMIT = 4 } }
  |                                 ^^^^^
  = help: Declare it `state` if it changes; a constant is the same on every page
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#x01
```

**Fix:** Hold what changes in a `state`. A `derived` value changes when
what it reads does; a prop is its caller's (`emit` an event so the caller
changes it); a route parameter is the address's (`navigate`); a loop
variable is one pass's copy (change the item through its list).

## Routes

### R01 — A route to nothing

A `Route` whose `page:` names no declared page. **Fix:** name a page that
exists.

## Data and assets

### D05 — An asset from another origin with no integrity hash

Warning. A font, stylesheet or script in `meta.fonts`, `meta.stylesheets`
or `meta.scripts` from another origin, with no hash in `meta.integrity`.
**Fix:** add its `sha384-…` hash to `meta.integrity` (the CDN usually
publishes it). Google Fonts is exempt: its stylesheet differs per browser.

## Types

Errors. The type checker's findings; a program that declares no types raises none of them, since what it cannot resolve is `Any`.

### T01 — A value of the wrong type

```wf expect T01
page P(path: "/", title: "T", description: "D") {
    state count: Number = 0
    Heading("Counter").h1
    Button("Reset") { on click { count = "zero" } }
}
```

```text
error[T01]: `count` is `String`, but `Number` is wanted
 --> src/App.wf:4:34
  |
4 |     Button("Reset") { on click { count = "zero" } }
  |                                  ^^^^^^^^^^^^^^
  = help: Convert it: `Number(value)`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t01
```

**Fix:** Give the state a value of its type, or convert: `Number(text)`, `"{n}"`. If the state really holds either, declare it `Any`.

### T02 — A case the enum does not have

```wf expect T02
enum Tone { calm, loud }
page P(path: "/", title: "T", description: "D") {
    state tone: Tone = .quiet
    Heading("Tone").h1
}
```

```text
error[T02]: `Tone` has no case `.quiet`
 --> src/App.wf:3:17
  |
3 |     state tone: Tone = .quiet
  |                 ^^^^
  = help: `Tone` takes .calm, .loud
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t02
```

**Fix:** Use one of the cases the message lists, or add the case to the `enum`.

### T04 — A value that may be null, read as if it were not

```wf expect T04
type User { name: String, email: String? }
page P(path: "/", title: "T", description: "D") {
    state user = User(name: "Ada", email: null)
    Heading("Profile").h1
    Text(user.email.toUpperCase())
}
```

```text
error[T04]: `user.email` may be null, so `.toUpperCase()` may fail
 --> src/App.wf:5:10
  |
5 |     Text(user.email.toUpperCase())
  |          ^^^^^^^^^^
  = help: Unwrap it first: `if let x = value { … }`, `value ?? fallback`, `value?.{method}()`, or a check for `!= null`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t04
```

**Fix:** Unwrap it: `if let e = user.email { e.toUpperCase() }`, `user.email?.toUpperCase()`, or `(user.email ?? "").toUpperCase()`.

### T05 — A field or method that does not exist

```wf expect T05
type User { name: String }
page P(path: "/", title: "T", description: "D") {
    state user = User(name: "Ada")
    Heading(user.nmae).h1
}
```

```text
error[T05]: `User` has no field `nmae`
 --> src/App.wf:4:18
  |
4 |     Heading(user.nmae).h1
  |                  ^^^^
  = help: Its fields are `name`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t05
```

**Fix:** Correct the spelling to one of the fields listed, or add the field to the `type`.

### T06 — A member a store or service does not have

```wf expect T06
store Cart { state items = [] }
page P(path: "/", title: "T", description: "D") {
    use Cart
    Heading("{Cart.count} items").h1
}
```

```text
error[T06]: `Cart` has no member `count`
 --> src/App.wf:4:15
  |
4 |     Heading("{Cart.count} items").h1
  |               ^^^^
  = help: Its members are `items`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t06
```

**Fix:** Use a member the store declares, or declare it: `derived count = items.length`. For a service, the endpoint must be declared in its `api`.

### T07 — A condition that is always true

```wf expect T07
page P(path: "/", title: "T", description: "D") {
    state items = ["a"]
    Heading("List").h1
    if items { Text("There are items") }
}
```

```text
error[T07]: `if` reads `items` as a condition, but a list is always true
 --> src/App.wf:4:5
  |
4 |     if items { Text("There are items") }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: Ask about its length: `items.length > 0`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t07
```

**Fix:** Ask the question you mean: `items.length > 0`, `user != null`.

### T08 — A loop over something that is not a list

```wf expect T08
page P(path: "/", title: "T", description: "D") {
    state total = 3
    Heading("Loop").h1
    for n in total { Text("{n}") }
}
```

```text
error[T08]: `for` loops over a list, but `total` is `Number`
 --> src/App.wf:4:5
  |
4 |     for n in total { Text("{n}") }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: Give it a list, or `.split(…)` a string first
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t08
```

**Fix:** Loop over a list — `for n in 1..=total` for a range of numbers.

### T09 — An emit that does not match its event

```wf expect T09
component Stepper(_ start: Number) {
    event change(value: Number)
    Button("+") { on click { emit change(1, 2) } }
}
```

```text
error[T09]: `emit change` passes 2 arguments, but the event takes 1
 --> src/App.wf:3:30
  |
3 |     Button("+") { on click { emit change(1, 2) } }
  |                              ^^^^^^^^^^^^^^^^^
  = help: `event change(value: Number)`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t09
```

**Fix:** Pass what the `event` declares, in order, or change the declaration.

### T10 — A call with the wrong arguments

```wf expect T10
page P(path: "/", title: "T", description: "D") {
    state count = 0
    action add(by: Number) { count = count + by }
    Heading("Count").h1
    Button("+") { on click { add(1, 2) } }
}
```

```text
error[T10]: `add` takes 1 argument, but 2 are given
 --> src/App.wf:5:30
  |
5 |     Button("+") { on click { add(1, 2) } }
  |                              ^^^
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t10
```

**Fix:** Pass the arguments the action, endpoint or function takes.

### T11 — A match on something that cannot be matched

```wf expect T11
page P(path: "/", title: "T", description: "D") {
    state n = 1
    Heading("Match").h1
    match n {
        loading { Spinner }
        else { Text("?") }
    }
}
```

```text
error[T11]: `match` needs a resource or an enum, but `n` is `Number`
 --> src/App.wf:4:5
  |
4 |     match n {
  |     ^^^^^^^^^
  = help: Use `if` for a condition; `match` chooses among a resource's states or an enum's cases
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t11
```

**Fix:** Use `if` for a condition; `match` takes a resource, a connection or an enum.

### T12 — A secret where it would escape

```wf expect T12
page P(path: "/", title: "T", description: "D") {
    state token: Secret = ""
    Heading("Token").h1
    Text(token)
}
```

```text
error[T12]: `token` is a `Secret`, and `Text` would show it
 --> src/App.wf:4:10
  |
4 |     Text(token)
  |          ^^^^^
  = help: A secret must not reach the page; send it as a value, or show one field of what it unlocks
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t12
```

**Fix:** Do not show, splice, log or persist a `Secret`. Send it in a request — or keep it on the server.

### T13 — A name nothing declares

```wf expect T13
page P(path: "/", title: "T", description: "D") {
    state count = 0
    Heading("Counter").h1
    Button("+") { on click { cuont = count + 1 } }
}
```

```text
error[T13]: nothing declares `cuont`
 --> src/App.wf:4:30
  |
4 |     Button("+") { on click { cuont = count + 1 } }
  |                              ^^^^^
  = help: Declare it — a `state`, a `const`, an `action`, a prop — or check the spelling. In the browser it would be a ReferenceError
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t13
```

**Fix:** Correct the spelling, or declare the name. The browser's own
globals (`window`, `navigator`, `Math` …) and the language's values
(`viewport`, `query`, `now` …) need no declaration; a call to a function
nothing declares — `uid()` for `uuid()` — is reported the same way. A
template rendered with data (`wf render`) reads its data's keys by name, so
it is not held to this.

### T14 — A comparison that is always the same

```wf expect T14
page P(path: "/", title: "T", description: "D") {
    state count = 0
    Heading("Count").h1
    if count == "0" { Text("Nothing yet") }
}
```

```text
error[T14]: `count` is `Number` and `"0"` is `String`, which are never equal, so this is always false
 --> src/App.wf:4:8
  |
4 |     if count == "0" { Text("Nothing yet") }
  |        ^^^^^
  = help: Convert one side: `Number(text) == n`, or `"{n}" == text`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t14
```

**Fix:** Compare values of one type: `count == 0`, or convert one side
(`Number(text) == n`). Against an enum, use a case it has.

### T15 — A `match` that misses a case, or has one twice

```wf expect T15
enum Status { draft, review, live }
page P(path: "/", title: "T", description: "D") {
    state status: Status = .draft
    Heading("Status").h1
    match status {
        .draft { Text("Draft") }
        .live { Text("Live") }
    }
}
```

```text
error[T15]: this `match` on `status` has no arm for `.review`, and no `else`
 --> src/App.wf:5:5
  |
5 |     match status {
  |     ^^^^^
  = help: Add an arm for each, or `else { … }` for the rest
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t15
```

**Fix:** Give every case its arm, or add `else { … }` for the rest. A
`match` expression that covers every case needs no `else`:
`derived label = match status { .draft { "Draft" } .review { "In review" } .live { "Live" } }`.

### T16 — A method a number, a string or a list does not have

```wf expect T16
page P(path: "/", title: "T", description: "D") {
    state names = ["Ada", "Grace"]
    Heading("People").h1
    Text(names.joined(", "))
}
```

```text
error[T16]: a `[String]` has no method `joined`
 --> src/App.wf:4:16
  |
4 |     Text(names.joined(", "))
  |                ^^^^^^
  = help: Did you mean `join`?
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t16
```

**Fix:** The method the message suggests — here `join`.

### T17 — An async result used before it is awaited

```wf expect T17
page P(path: "/", title: "T", description: "D") {
    state saved = false
    action save() { saved = true }
    Heading("Save").h1
    Button("Save", disabled: save.pending) { on click { save() } }
}
```

```text
error[T17]: `save` awaits nothing, so `.pending` is always false
 --> src/App.wf:5:30
  |
5 |     Button("Save", disabled: save.pending) { on click { save() } }
  |                              ^^^^
  = help: `.pending` is true while a call of an async action runs; an action that awaits nothing finishes before the page repaints
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t17
```

**Fix:** `.pending` is for an action that awaits — drop it here. An async
action's result is read with `await` inside an action or a handler; a value
that arrives over the network is a `resource`, which `derived` values and
elements can read.

### T18 — Arithmetic on something that is not a number

```wf expect T18
page P(path: "/", title: "T", description: "D") {
    state price = "12"
    Heading("Total {price * 2}").h1
}
```

```text
error[T18]: `*` takes numbers, but `price` is `String`
 --> src/App.wf:3:27
  |
3 |     Heading("Total {price * 2}").h1
  |                           ^
  = help: Convert it: `Number(value)`; `+` joins text when one side is a string
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t18
```

**Fix:** Hold numbers as numbers (`state price = 12`), or convert:
`Number(price) * 2`.

### T19 — A value that may be null, shown as text

Warning.

```wf expect T19
type User { name: String, nickname: String? }
page P(path: "/", title: "T", description: "D") {
    state user = User(name: "Ada", nickname: null)
    Heading("Hello").h1
    Text("Also known as {user.nickname}")
}
```

```text
warning[T19]: `user.nickname` may be null, and in text it would show as `null`
 --> src/App.wf:5:26
  |
5 |     Text("Also known as {user.nickname}")
  |                          ^^^^^^^^^^^^^
  = help: Say what to show instead: `{user.nickname ?? ""}`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t19
```

**Fix:** Say what to show instead: `{user.nickname ?? "nothing yet"}`, or
show the line only when there is one: `if let n = user.nickname { … }`.

### T21 — A resource `match` with no `error` arm

Warning.

```wf expect T21
page P(path: "/", title: "T", description: "D") {
    resource users = fetch("/api/users")
    Heading("Users").h1
    match users {
        loading { Spinner }
        ready(list) { Text("{list.length} users") }
    }
}
```

```text
warning[T21]: this `match` on `users` has no `error` arm, so a failed request shows nothing
 --> src/App.wf:4:5
  |
4 |     match users {
  |     ^^^^^
  = help: Add `error(e) { Alert(e.message).danger }`, or `else { … }`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t21
```

**Fix:** `error(e) { Alert(e.message).danger }`, or an `else`.

## Accessibility

Warnings. What makes a page unusable with a screen reader, a keyboard or a voice.

### A01 — An image with no alt text

```wf expect A01
page P(path: "/", title: "T", description: "D") {
    Heading("Photo").h1
    Image(src: "/team.jpg")
}
```

```text
warning[A01]: Image missing "alt" attribute
 --> src/App.wf:3:5
  |
3 |     Image(src: "/team.jpg")
  |     ^^^^^
  = help: Add alt text: Image(src: "...", alt: "Description of image")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a01
```

**Fix:** `Image(src: "/team.jpg", alt: "The team at the launch")`. A purely decorative image takes `alt: ""`.

### A02 — An icon button with no name

```wf expect A02
page P(path: "/", title: "T", description: "D") {
    Heading("Close").h1
    IconButton(icon: "close")
}
```

```text
warning[A02]: IconButton missing accessible label
 --> src/App.wf:3:5
  |
3 |     IconButton(icon: "close")
  |     ^^^^^^^^^^
  = help: Add a label: IconButton(icon: "close", label: "Close dialog")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a02
```

**Fix:** `IconButton(icon: "close", label: "Close")` — the label is read aloud, never shown.

### A03 — An input with no label

```wf expect A03
page P(path: "/", title: "T", description: "D") {
    state name = ""
    Heading("Name").h1
    Input(bind: name).text
}
```

```text
warning[A03]: Input missing "label" or "placeholder" attribute
 --> src/App.wf:4:5
  |
4 |     Input(bind: name).text
  |     ^^^^^
  = help: Add a label: Input(label: "Username").text
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a03
```

**Fix:** `Input(bind: name, label: "Name")`. A placeholder alone disappears as the reader types.

### A04 — A control with no label

```wf expect A04
page P(path: "/", title: "T", description: "D") {
    state agree = false
    Heading("Terms").h1
    Checkbox(bind: agree)
}
```

```text
warning[A04]: Checkbox missing "label" attribute
 --> src/App.wf:4:5
  |
4 |     Checkbox(bind: agree)
  |     ^^^^^^^^
  = help: Add a label: Checkbox(bind: value, label: "Description")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a04
```

**Fix:** `Checkbox(bind: agree, label: "I agree to the terms")`.

### A05 — A button with no text

```wf expect A05
page P(path: "/", title: "T", description: "D") {
    Heading("Actions").h1
    Button { Icon("check") }
}
```

```text
warning[A05]: Button has no text content
 --> src/App.wf:3:5
  |
3 |     Button { Icon("check") }
  |     ^^^^^^
  = help: Add text: Button("Save").primary
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a05
```

**Fix:** Give it text, or an `aria-label:` when it shows only an icon.

### A06 — A link with no text

```wf expect A06
page P(path: "/", title: "T", description: "D") {
    Heading("Links").h1
    Link(to: "/about")
}
```

```text
warning[A06]: Link has no text content
 --> src/App.wf:3:5
  |
3 |     Link(to: "/about")
  |     ^^^^
  = help: Add text: Link("About", to: "/about")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a06
```

**Fix:** Give it text that names where it goes — not "click here".

### A07 — An empty heading

```wf expect A07
page P(path: "/", title: "T", description: "D") {
    Heading("Title").h1
    Heading("").h2
}
```

```text
warning[A07]: Heading has empty text content
 --> src/App.wf:3:5
  |
3 |     Heading("").h2
  |     ^^^^^^^
  = help: Headings should have meaningful text
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a07
```

**Fix:** Give the heading text, or remove it; style text with `style { }` rather than an empty heading.

### A08 — A dialog with no title

```wf expect A08
page P(path: "/", title: "T", description: "D") {
    state open = false
    Heading("Modal").h1
    Modal(visible: open) { Text("Hello") }
}
```

```text
warning[A08]: Modal missing "title" attribute
 --> src/App.wf:4:5
  |
4 |     Modal(visible: open) { Text("Hello") }
  |     ^^^^^
  = help: Add a title: Modal(visible: state, title: "Dialog Title")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a08
```

**Fix:** `Modal(visible: open, title: "Delete this?")` — the title is its accessible name.

### A09 — Media nobody can follow without sound

```wf expect A09
page P(path: "/", title: "T", description: "D") {
    Heading("Demo").h1
    Video(src: "/demo.mp4").controls
}
```

```text
warning[A09]: Video missing "captions"
 --> src/App.wf:3:5
  |
3 |     Video(src: "/demo.mp4").controls
  |     ^^^^^
  = help: Add captions: Video(src: "...", captions: "/captions.en.vtt")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a09
```

**Fix:** `Video(src: …, captions: "/demo.en.vtt").controls`; an `Audio` takes `transcript:`.

### A10 — A table with no header row

```wf expect A10
page P(path: "/", title: "T", description: "D") {
    Heading("Table").h1
    Table(caption: "Prices") { Table.Body { Table.Row { Table.Cell("Pen")  Table.Cell("2") } } }
}
```

```text
warning[A10]: Table missing header row (Thead)
 --> src/App.wf:3:5
  |
3 |     Table(caption: "Prices") { Table.Body { Table.Row { Table.Cell("Pen")  Table.Cell("2") } } }
  |     ^^^^^
  = help: Add a header: Table { Thead { Tcell("Column Name") } ... }
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a10
```

**Fix:** Put the first row in `Table.Head`, so its cells are `<th scope="col">`.

### A11 — A skipped heading level

```wf expect A11
page P(path: "/", title: "T", description: "D") {
    Heading("Title").h1
    Heading("Deep").h4
}
```

```text
warning[A11]: Heading level skips from h1 to h4
 --> src/App.wf:3:5
  |
3 |     Heading("Deep").h4
  |     ^^^^^^^
  = help: Use h2 instead, or add the missing intermediate headings
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a11
```

**Fix:** Use the next level down (`h2` after `h1`). Size text with `style { font-size }`, not with the heading level.

### A12 — A page without exactly one h1

```wf expect A12
page P(path: "/", title: "T", description: "D") {
    Heading("Section").h2
}
```

```text
warning[A12]: Page has no h1 heading
 --> src/App.wf:1:1
  |
1 | page P(path: "/", title: "T", description: "D") {
  | ^^^^
  = help: Add a main heading: Heading("Page Title").h1
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a12
```

**Fix:** One `Heading(…).h1` per page, naming what the page is. A layout's `h1` counts for the page it frames.

### A13 — Colours without enough contrast

```wf expect A13
theme Faint {
    color-text: #BBBBBB
    color-background: #FFFFFF
}
page P(path: "/", title: "T", description: "D") {
    Heading("Contrast").h1
}
```

```text
warning[A13]: text on a card or surface has a contrast ratio of 1.83:1, below the 4.5:1 WCAG AA minimum
 --> src/App.wf:2:1
  |
2 |     color-text: #BBBBBB
  | ^
  = help: Darken --color-text or lighten --color-surface until they clear 4.5:1
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a13
```

**Fix:** Darken the text or lighten the background until the ratio clears 4.5:1 (3:1 for large text).

### A14 — A role whose children are the wrong kind

```wf expect A14
page P(path: "/", title: "T", description: "D") {
    Heading("Tabs").h1
    Row(role: "tablist") {
        Button("One")
        Button("Two")
    }
}
```

```text
warning[A14]: role "tablist" requires children with role "tab", but holds a Button
 --> src/App.wf:3:5
  |
3 |     Row(role: "tablist") {
  |     ^^^
  = help: Give each child role: "tab", or use the built-in that owns this structure
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a14
```

**Fix:** Give each child the role the parent requires (`role: "tab"`), or use the built-in that owns the structure — `Tabs`.

### A15 — An aria-label that does not contain the visible text

```wf expect A15
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button("Save", aria-label: "Submit the form")
}
```

```text
warning[A15]: Button shows "save" but its aria-label says "Submit the form"
 --> src/App.wf:3:5
  |
3 |     Button("Save", aria-label: "Submit the form")
  |     ^^^^^^
  = help: Start the aria-label with the visible text, so what a user says matches what they see
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#a15
```

**Fix:** Start the `aria-label` with the visible words: `aria-label: "Save the draft"`.

## Search and sharing

Warnings. What a search engine or a link preview would act on.

### S01 — A page with no title

```wf expect S01
page P(path: "/", description: "D") {
    Heading("No title").h1
}
```

```text
warning[S01]: Page P has no title
 --> src/App.wf:1:1
  |
1 | page P(path: "/", description: "D") {
  | ^^^^
  = help: Add one: page Name(path: "/", title: "What this page is")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#s01
```

**Fix:** `page P(path: "/", title: "What this page is")`.

### S02 — A page with no description

```wf expect S02
page P(path: "/", title: "T") {
    Heading("No description").h1
}
```

```text
warning[S02]: Page P has no description
 --> src/App.wf:1:1
  |
1 | page P(path: "/", title: "T") {
  | ^^^^
  = help: Add one: page Name(path: "/", title: "…", description: "A sentence a search result can show")
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#s02
```

**Fix:** Add a `description:` of about 150 characters: what a reader gets from the page.

### S03 — A description too long for a search result

```wf expect S03
page P(path: "/", title: "T", description: "A description that goes on for far longer than any search result will ever show, because it keeps adding clauses, qualifications and asides until the snippet is cut mid-sentence.") {
    Heading("Long").h1
}
```

```text
warning[S03]: Page P's description is 178 characters; a search result shows about 160
 --> src/App.wf:1:1
  |
1 | page P(path: "/", title: "T", description: "A description that goes on for far longer than any search result will ever show, because it keeps adding clauses, qualifications and asides until the snippet is cut mid-sentence.") {
  | ^^^^
  = help: Shorten it, or accept that it will be cut mid-sentence
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#s03
```

**Fix:** Shorten it to about 160 characters.

### S04 — Two pages on one route

```wf expect S04
page A(path: "/about", title: "A", description: "D") { Heading("A").h1 }
page B(path: "/about", title: "B", description: "D") { Heading("B").h1 }
```

```text
error[S04]: Pages A and B both claim the route /about
 --> src/App.wf:2:1
  |
2 | page B(path: "/about", title: "B", description: "D") { Heading("B").h1 }
  | ^^^^
  = help: Give each page its own path
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#s04
```

**Fix:** Give each page its own `path:`. Only one of them would ever render.

## What is kept in the browser

Warnings. `persist` values across releases.

### P01 — A persisted value with no way forward

```wf expect P01
store Cart {
    persist items: [String] = [] {
        version: 2
    }
}
```

```text
warning[P01]: `items` is at version 2, and nothing brings version 1 forward
 --> src/App.wf:2:5
  |
2 |     persist items: [String] = [] {
  |     ^^^^^^^
  = help: A reader who last visited then loses what they had. Add `migrate 1 -> 2`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#p01
```

**Fix:** Add a step for each older version: `migrate 1 -> 2 { old.map(…) }`.

### P02 — A migration that never runs

```wf expect P02
store Cart {
    persist items: [String] = [] {
        version: 1
        migrate 1 -> 2 { old }
    }
}
```

```text
warning[P02]: `migrate 1 -> 2` on `items` is past the declared version
 --> src/App.wf:2:5
  |
2 |     persist items: [String] = [] {
  |     ^^^^^^^
  = help: A step above `version:` never runs; raise the version or drop the step
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#p02
```

**Fix:** Raise `version:` to the step's target, or drop the step.

### P03 — A persisted value in a route-scoped store

```wf expect P03
store Filters(scope: .route) {
    persist query = ""
}
```

```text
warning[P03]: `query` is persisted in a store scoped to the route
 --> src/App.wf:2:5
  |
2 |     persist query = ""
  |     ^^^^^^^
  = help: The route change drops the store and the next read builds it again from storage, so the value comes straight back. Use `state` for what the route owns, or widen the scope for what outlives it
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#p03
```

**Fix:** Keep route-scoped data in `state`, or move the value to a store with a wider scope.

## Declared and never read

Warnings. A name beginning `_` is taken as unused on purpose.

### U01 — A state nothing reads

```wf expect U01
page P(path: "/", title: "T", description: "D") {
    state unused = 0
    Heading("Unused").h1
}
```

```text
warning[U01]: `unused` is declared but never read
 --> src/App.wf:2:5
  |
2 |     state unused = 0
  |     ^^^^^
  = help: Nothing reads the state; remove it, or name it `_unused` to keep it
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#u01
```

**Fix:** Remove it, read it, or name it `_unused` to keep it on purpose.

### U02 — A derived value nothing reads

```wf expect U02
page P(path: "/", title: "T", description: "D") {
    state n = 1
    derived doubled = n * 2
    Heading("{n}").h1
}
```

```text
warning[U02]: `doubled` is declared but never read
 --> src/App.wf:3:5
  |
3 |     derived doubled = n * 2
  |     ^^^^^^^
  = help: Nothing reads the derived value; remove it, or name it `_doubled` to keep it
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#u02
```

**Fix:** Remove it or read it; a leading `_` keeps it.

### U03 — A component nothing places

```wf expect U03
component Orphan { Text("Nobody places me") }
page P(path: "/", title: "T", description: "D") {
    Heading("Home").h1
}
```

```text
warning[U03]: `Orphan` is declared but never placed
 --> src/App.wf:1:1
  |
1 | component Orphan { Text("Nobody places me") }
  | ^^^^^^^^^
  = help: Place it in a page, name it as a layout, or remove it
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#u03
```

**Fix:** Place it, name it as a `layout:`, publish it in `build.elements`, or remove it.

### U04 — A store member nothing reads

```wf expect U04
store Cart {
    state items = []
    state coupon = ""
}
page P(path: "/", title: "T", description: "D") {
    use Cart
    Heading("{Cart.items.length}").h1
}
```

```text
warning[U04]: `Cart.coupon` is declared but never read
 --> src/App.wf:3:5
  |
3 |     state coupon = ""
  |     ^^^^^
  = help: Nothing reads the state, inside the store or as `Cart.coupon`; remove it, or name it `_coupon` to keep it
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#u04
```

**Fix:** Remove the member, or name it with a leading `_`.

### U05 — An action nothing calls

```wf expect U05
page P(path: "/", title: "T", description: "D") {
    state n = 0
    action reset() { n = 0 }
    Heading("{n}").h1
}
```

```text
warning[U05]: `reset` is declared but never read
 --> src/App.wf:3:5
  |
3 |     action reset() { n = 0 }
  |     ^^^^^^
  = help: Nothing reads the action; remove it, or name it `_reset` to keep it
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#u05
```

**Fix:** Call it, remove it, or name it `_reset`.

### U10 — An `else` no value reaches

```wf expect U10
enum Tone { calm, loud }
page P(path: "/", title: "T", description: "D") {
    state tone: Tone = .calm
    Heading("Tone").h1
    match tone {
        .calm { Text("Calm") }
        .loud { Text("Loud") }
        else { Text("?") }
    }
}
```

```text
warning[U10]: every case of `Tone` has its own arm, so this `match`'s `else` is never reached
 --> src/App.wf:5:5
  |
5 |     match tone {
  |     ^^^^^
  = help: Remove the `else`
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#u10
```

**Fix:** Remove the `else`.

## Vocabulary

Warnings.

### V01 — A bare word that means nothing

```wf expect V01
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button(primary)
}
```

```text
warning[V01]: nothing in scope declares `primary`, so it reads as nothing — a flag is written `.primary`
 --> src/App.wf:3:12
  |
3 |     Button(primary)
  |            ^^^^^^^
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#v01
```

**Fix:** Write the flag with its dot — `Button("Save").primary` — or declare the name. The word is also a name nothing declares, so the build stops on it as `T13`; the editor shows both.

### V03 — Markup put in as markup

```wf expect V03
page P(path: "/", title: "T", description: "D") {
    state body = "<b>Hi</b>"
    Heading("Markup").h1
    Unsafe.Html(body)
}
```

```text
warning[V03]: `Unsafe.Html` puts markup in as markup
 --> src/App.wf:4:5
  |
4 |     Unsafe.Html(body)
  |     ^^^^^^^^^^^
  = help: Anything the project did not write itself belongs in `sanitize(…)` first
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#v03
```

**Fix:** Wrap markup from outside the project in `sanitize(…)`: `Unsafe.Html(sanitize(body))`.

### V02 — A flag no stylesheet styles

A flag on one of your components whose class no stylesheet — the engine's
or one of the project's `.css` files — defines, so it changes nothing on
screen. The built-ins cannot draw it: the registry and the engine's
stylesheet are held to each other. **Fix:** define the class in a `.css`
file under `src/`, or drop the flag.

### V04 — A class that is the engine's

```wf expect V04
page P(path: "/", title: "T", description: "D") {
    Heading("Classes").h1
    Card(class: "wf-btn") { Text("x") }
}
```

```text
warning[V04]: `class:` names `wf-btn`, one of the engine's own classes
 --> src/App.wf:3:5
  |
3 |     Card(class: "wf-btn") { Text("x") }
  |     ^^^^
  = help: `wf-` classes are the built-ins': one added here brings another built-in's rules with it. Name a class of your own, or use the flag that sets the look
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#v04
```

**Fix:** Name a class of your own and style it in a `.css` file under `src/`, or use the flag that sets the look you wanted (`.primary`, `.elevated`).

### V08 — An icon the runtime does not draw

```wf expect V08
page P(path: "/", title: "T", description: "D") {
    Heading("Launch").h1
    Icon("rocket")
}
```

```text
warning[V08]: `rocket` is not an icon the runtime draws; it will show as the word
 --> src/App.wf:3:10
  |
3 |     Icon("rocket")
  |          ^
  = help: The icons: close, menu, search, home, user, settings, check, plus, minus, edit, trash, star, heart, mail, bell, download, upload, eye, link, calendar, filter, chevron-down, chevron-right, chevron-left, info, warning, arrow-left, arrow-right, logout, copy, sun, moon
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#v08
```

**Fix:** Use one of the built-in icons the message lists.

### V09 — An event the element does not fire

```wf expect V09
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button("Save") { on clik { log("saved") } }
}
```

```text
warning[V09]: `clik` is not an event Button fires
 --> src/App.wf:3:22
  |
3 |     Button("Save") { on clik { log("saved") } }
  |                      ^^
  = help: A handler names a DOM event (`click`, `input`, `change`, `submit`, `keydown`, `pointerdown`, `scroll`, …) or an event the element declares
  = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#v09
```

**Fix:** Name a DOM event (`click`, `input`, `change`, `submit`, `keydown`,
…) or one the component declares.

## Next

[Built-ins](40-built-ins.md).
