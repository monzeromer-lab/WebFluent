---
chapter: 43
kind: "error"
route: "guide/diagnostics"
group: "reference"
blurb: "Every error and warning the compiler reports — what it means, a program that draws it, the message, and the fix."
description: "Every WebFluent error and warning, by code — what it means, a program that draws it, the message the compiler prints, and the fix."
---
# 43. Diagnostics

This chapter is the reference for every finding the compiler reports. Each
finding has a code, such as `T05`; search this page for the code to find
its entry, or run `wf explain T05` to print the entry in your terminal.

`wf build`, `wf check`, `wf serve` and your editor report the same findings,
from one pipeline. This chapter first explains how to read a finding and
how to change what one counts as, then lists every code by family.

## Reading a finding

Every finding is printed in the same layout. This one reports a misspelt
field:

```text
error[T05]: `User` has no field `nmae`
 --> src/pages/Profile.wf:3:18
  |
3 |     Heading(user.nmae).h1
  |                  ^^^^
  = help: Its fields are `id`, `name`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t05-a-field-or-method-that-does-not-exist
```

From the top, a finding shows:

1. its severity, its code and what is wrong;
2. the place, in the `file:line:col` form that editors and CI problem
   matchers read;
3. the line of source, with the part that is wrong underlined;
4. `help:` — what to do about it, when the compiler can say;
5. `docs:` — a link to the code's entry in this chapter.

Colour is used only on a terminal, and never when `NO_COLOR` is set.

## Errors and warnings

Every code has a severity:

- An **error** stops the build: the program cannot mean what it says.
- A **warning** prints and the build goes on: the program means something,
  probably not what you wanted. Nothing is a warning that the compiler
  could have decided for itself.

Every stage of the compiler runs on whatever the stage before it found, and
every file is read even when another does not parse — the parser picks up
again at the next declaration. One build therefore shows every mistake
once, then sums them up in one line:
`error: the build stopped: 2 errors, 5 warnings`.

A build that stops on its findings exits with `1`. A build that cannot run
at all — a file it cannot read, a config it cannot load — exits with `2`.
`--deny-warnings` on `wf build` or `wf check` makes a warning stop the build
too.

## Findings for a tool

`--format` on `wf build` and `wf check` writes the same findings for a tool
([Command line](../reference/cli.md#wf-check)):

| Format | Output |
|---|---|
| `json` | One document on standard output, with every finding's code, place, hint, related places and fixes. |
| `sarif` | SARIF 2.1.0, for GitHub code scanning. |
| `github` | `::error` annotations, which a GitHub workflow shows on the pull request. |

## Fixes

Where the compiler knows the fix, the finding carries it as edits. Your
editor offers it as a quick fix, `--format json` carries it in `fixes`, and
[`wf fix`](../reference/cli.md#wf-fix) applies it. A fix is offered only where it is
certain; where it is not, the code's entry says what to do instead.

Each kind of finding has its decided fix:

- **The nearest name**, wherever the code names a set: a field, member,
  method, variable, component, case or flag (`T02`, `T05`, `T06`, `T13`,
  `T16`, `E101`, `E103`), a prop (`C02`, `C04`), a route (`R01`, which also
  rewrites a relative link from the root), a store (`X05`), a locale
  (`I04`), a file of `public/` (`D01`), a CSS property or token (`V05`,
  `V06`), an icon (`V08`) and an event (`V09`).
- **What is missing, written in**: `?.` for a read that may be null
  (`T04`), `(await …)` for a promise read as its value (`T17`), the missing
  props (`C01`), the missing arms of a `match` (`T15`) and an `error` arm
  (`T21`), a `Router` in an `app` with none (`R03`), `bind:` for a box that
  shows a state and never changes it (`F04`), `px` for a bare number where
  CSS wants a length (`V07`), `controls: true` (`A09`), and the `migrate`
  step a version is missing (`P01`) — with the version raised in the same
  change when a persisted shape changed (`D04`).
- **A name, drafted from the code**: `alt: ""` for an image (`A01`); a
  label from what a control binds or what an icon is (`A02`–`A04`), a
  link's from where it leads (`A06`), a dialog's title from what opens it
  (`A08`); the heading level that mends the outline (`A11`); the visible
  text at the start of an `aria-label` (`A15`). With nothing to draft from,
  there is no fix, because an empty name would silence the warning and
  name nothing.
- **What is not used, removed** — a state, derived value, action, store
  member or component (`U01`–`U05`), code after a `return` (`U06`), an
  allow that allows nothing (`U07`) — with *rename to `_name`* as the
  second fix, for what you keep on purpose; and a loop keyed `by item.id`
  when its body reads the item's `id` (`U09`).
- **A change of form**: `derived` for a state that copies a live value
  nothing changes (`X06`), a positional argument named for the one prop it
  can be (`C05`), and the prop marked `_` in the component's declaration
  (`C06`).

A family's findings in one file are fixed together, as one change: the
editor's **Fix all `V07` in this file**, or `wf fix --code V07` for the
whole project.

## Changing what a finding counts as

`lints` in `webfluent.app.json` sets what a finding counts as for the
whole project, by code or by family — `"off"`, `"warn"` or `"error"`. A
code's setting wins over its family's:

```json
{ "lints": { "A11": "off", "U": "error", "R01": "warn" } }
```

You can turn any warning off or raise it to an error. You can lower an
error only when it cannot ship a broken page: `R01`, `C02`, `S04`, `I01`,
`I02` and `I04`. Lowering any other error is refused
([Configuration](../reference/configuration.md#lints)).

## Allowing one finding

To accept one finding on purpose, write a `// wf-allow(CODE)` comment:

- on a line of its own, it covers the next line of code;
- at the end of a line, it covers that line.

It names codes or families, comma-separated — `// wf-allow(U01, A)`. Only a
finding that [`lints`](../reference/configuration.md#lints) could lower may be
allowed. An allow that covers nothing is itself reported (`U07`), so an
allow does not outlive what it was for.

## Explaining a code

`wf explain CODE` prints a code's entry from this chapter, word for word;
`wf explain` with no code lists every code with its severity and title.

```bash
wf explain T05
wf explain
```

## The families

A code's letter is its family:

| Family | What it covers |
|---|---|
| `E` | syntax and structure: what cannot be read, or refers to nothing |
| `T` | types |
| `C` | components and their props |
| `R` | routes and navigation |
| `X` | state and reactivity |
| `I` | translations |
| `F` | forms and bindings |
| `D` | data, assets and what is kept in the browser |
| `A` | accessibility |
| `S` | search and sharing |
| `P` | persisted values |
| `U` | declared and never used |
| `V` | vocabulary |

Each entry below has the same parts: the code and its title as the heading,
what it means, a program that draws it, the message the compiler prints for
that program, and the fix. The test suite compiles every example program,
checks that it draws exactly the code it is filed under, and checks that
the message shown is what the compiler prints.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e001-text-the-compiler-cannot-read
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e002-a-syntax-error
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e004-a-file-in-the-webfluent-2-grammar
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
  = help: did you mean `Button`? Or declare `component Buton { … }`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e101-a-component-nothing-declares
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e102-a-name-declared-twice
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
  = help: Its flags are .bounce, .button, .collapse, .danger, .disabled, .dropInside, .expand, .fadeIn, .fadeOut, .fast, .full, .info, .lg, .md, .normal, .outlined, .pill, .primary, .pulse, .reset, .rounded, .scaleIn, .scaleOut, .secondary, .shake, .slideDown, .slideLeft, .slideRight, .slideUp, .slow, .sm, .spin, .submit, .success, .trapFocus, .warning
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e103-a-flag-case-part-event-or-slot-the-component-does-not-have
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e104-a-layout-with-nowhere-to-put-the-page
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e105-a-statement-a-render-block-cannot-hold
```

**Fix:** Put it in `on click { }`, an `action` or an `effect`.

### E106 — Script or a whole style in an attribute

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e106-script-or-a-whole-style-in-an-attribute
```

**Fix:** `Button("Save") { on click { save() } }`.

`style:` is refused the same way: it would write a value as the
element's whole `style=` attribute, which a value from data can turn
into a box over the whole page or a request to anywhere. A style is a
`style { }` block, one property at a time, where a value that reads
state sets that one property and nothing else.

```wf expect E106
page P(path: "/", title: "T", description: "D") {
    state look = "color: red"
    Heading("Note").h1
    Text("Hi", style: look)
}
```

```text
error[E106]: `style:` on Text would write a value as the element's whole inline style
 --> src/App.wf:4:16
  |
4 |     Text("Hi", style: look)
  |                ^^^^^
  = help: Write a style block instead: `Text(…) { style { color: {tone} } }`, one property at a time
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e106-script-or-a-whole-style-in-an-attribute
```

**Fix:** `Text("Hi") { style { color: {tone} } }`.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e107-a-url-a-browser-would-run
```

**Fix:** A relative URL, or `http`, `https`, `mailto`, `tel`, `sms`,
`ftp`.

A page's `head { }` is held to the same rule, and more: a
`script(src:)` must be written out, since whoever controls a value it
reads would choose the script the page runs; a `meta(http-equiv:
"refresh")` whose address comes from a value is an open redirect, and
an `http-equiv` a value chooses could become one.

```wf expect E107
page P(path: "/", title: "T", description: "D") {
    head { script(src: query.lib) }
    Heading("Home").h1
}
```

```text
error[E107]: `head { script(src: …) }` loads a script a value chooses: whoever controls the value runs code as this site
 --> src/App.wf:2:12
  |
2 |     head { script(src: query.lib) }
  |            ^^^^^^
  = help: Write the address out — `script(src: "/analytics.js")` — and keep what varies in the script's own data
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e107-a-url-a-browser-would-run
```

**Fix:** `head { script(src: "/analytics.js") }`; send a reader on with
`navigate(…)`, which keeps to the site's routes.

### E108 — An `env` name that is not public

`env.STRIPE_SECRET` read in a page: `env.STRIPE_SECRET is not public, and a
page reads what is in the bundle`. **Fix:** rename it `PUBLIC_…` or list it
in `public_env` — only if anyone may read it ([chapter
30](../recipes/environments.md)).

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e113-an-argument-that-cannot-be-positional
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e114-emit-outside-a-component
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e116-a-tag-that-cannot-be-a-custom-element-s
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
script from an origin the policy never named — would be blocked. A page's
own `head { script(src: …) }` or `head { link(rel: "stylesheet", href: …) }`
is held to it too, where it is written: the tag is added on the live page,
and the browser would refuse it there without a word. **Fix:** what the
message says; a library's origin goes in `meta.scripts`, a stylesheet's in
`meta.stylesheets`.

### E117 — A key combination no keyboard sends

```wf expect E117
page P(path: "/", title: "T", description: "D") {
    on key("ctrl+shfit+k") { log("search") }
    Heading("Search").h1
}
```

```text
error[E117]: `on key("ctrl+shfit+k")` names `shfit`, which is not a modifier
 --> src/App.wf:2:12
  |
2 |     on key("ctrl+shfit+k") { log("search") }
  |            ^
  = help: Modifiers `mod` (⌘ on a Mac, Ctrl elsewhere), `ctrl`, `shift`, `alt`, `meta` (`cmd`), then one key: a letter, a digit, or a name like `Enter`, `Escape`, `ArrowDown`, `Tab`, `F2`; a chord is its steps with a space between, `mod+k mod+s`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e117-a-key-combination-no-keyboard-sends
```

**Fix:** Modifiers `ctrl`, `shift`, `alt`, `meta` (or `cmd`), then one key:
a letter, a digit, or a name — `Enter`, `Escape`, `ArrowDown`, `Tab`, `F2`.

### E118 — A name the language already has

```wf expect E118
page Deploy(path: "/", title: "Deploy", description: "Pick a target.") {
    state env = "staging"
    Heading("Deploy to {env}").h1
}
```

```text
error[E118]: `env` is a name the language already has: the build's settings, read as `env.PUBLIC_API`
 --> src/App.wf:2:5
  |
2 |     state env = "staging"
  |     ^^^^^^^^^^^^^^^^^^^^^
  = help: Name it something of your own — `target` or `stage`: inside its scope every `env` would mean this, and the language's would have no name
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#e118-a-name-the-language-already-has
```

A name the language gives a meaning is not a program's to declare — not as
a `state`, `derived`, `persist` or `let`, a prop, a route or action
parameter, a loop item, a lambda's parameter, a binding of `if let`, `match`
or a slot, nor a top-level `const`, `data`, `image`, store or `api`.
Declared, it would hide what the language means by it for as far as its
scope reaches: a `state env` hid the build's settings in its page, and a
loop's `t` hid the translation function inside the loop. The names are:

- `env`, the build's settings; `params`, the route's; `locale` and `dir`,
  the reader's language;
- the values the browser keeps current: `now`, `viewport`, `query`, `hash`,
  `theme`, `network`, `update`;
- the language's functions: `t`, `format`, `ago`, `log`, `navigate`,
  `fetch`, `uuid`, `sanitize`, `setLocale`, `setTheme`, `optimistic`,
  `beacon`, `animate`, `replayAnimation`, `every`, `after`, `ws`, `sse`,
  `broadcast`, `rtc`;
- the words a statement is written with: `state`, `persist`, `derived`,
  `effect`, `action`, `use`, `resource`, `validate`, `socket`, `stream`,
  `channel`, `peer`, `head`, `slot`, `part`, `if`, `else`, `for`, `in`, `by`,
  `show`, `match`, `children`, `emit`, `sequence`, `on`, `style`,
  `transition`, `let`, `return`, `await`, `try`, `catch`;
- at the top level, `WF`, the runtime the compiled pages call;
- in a store, the names every JavaScript object has — `valueOf`,
  `toString`, `constructor`, `hasOwnProperty`, `isPrototypeOf`,
  `propertyIsEnumerable`, `toLocaleString`, `__proto__` and its
  `__define…`/`__lookup…` kin: a store is an object whose members are its
  own, and a `valueOf` member replaced the one JavaScript calls itself, so
  the page threw as soon as it was drawn;
- and, as a prop, `__proto__`, which in the object a call passes sets its
  prototype instead (a prop named `toString` or `valueOf` is fine).

The names a handler brings with it — `event`, `e`, `value`, `key` — and the
browser's globals (`window`, `history`, …) are a program's to use, and so is
any name as a field, a map key or an enum case: `type Box { env: String }`
and `.env` are fine.
**Fix:** Name it something of your own — `state target = "staging"` — and
its reads with it. The message suggests one where there is a usual choice:
`target` or `stage` for `env`, `search` for `query`, `scheme` for `theme`,
`item` or a word for what it holds for `t`.

### E119 — A hidden direction control in the source

Warning. An embedding, override or isolate control — U+202A to U+202E,
U+2066 to U+2069 — is invisible, and it reorders what follows it: a
comment that seems to close, or a string that seems to hold one word, can
hide code that runs (Trojan Source, CVE-2021-42574). The editor draws each
as a mark; the check names each by its line, its column and its Unicode
name. **Fix:** remove it. Arabic beside Latin needs no control: each run
takes the direction of its own letters.

Names may be written in Arabic, or any script: a name starts with a letter
and goes on with letters, digits, `_` and the marks of its letters, as
UAX #31 says (`state عددٌ = 3`), and is one name however its letters were
typed — أ precomposed or ا with a combining hamza (NFC).

### E120 — Something nests too deeply

Error. Brackets inside brackets, blocks inside blocks, a splice inside a
string inside a splice, the links of one chain (`a.b.c`, `1 + 2 + 3`,
each link nesting the value one level deeper) and, in a template,
components that place one another — counted together, more than 100
levels deep. The parser, the checker and the template engine each stop
there and say so, where each would otherwise run out of stack. The
deepest program of this guide, the examples and the site nests 15 levels.
**Fix:** name a part of it — a `let` in an action, a `derived` value, a
component — and use the name. A component that places itself stops when
its data does; one whose data does not end is this error too.

### E121 — A render that would do too much

Error, from the template engine (`wf render`, `Template::render_*`). One
render may take ten million steps — an element drawn, a loop item, a unit
of working a value out — and write 256 MB of markup, and one expression
may make about half a million list items or 32 MB of text. A render past
any of them stops and says where, instead of growing until the server
runs out of memory: two nested `for` loops over ranges of a billion, a
loop over the data inside a loop over the data that is too large, or
`(1..=1000000).map(i => 1..=1000000)`. A build that meets such an
expression at build time leaves it to the page, as it does any value it
cannot work out. **Fix:** draw less — page the data before it reaches the
template, or work the value out on the server and hand it over.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#c01-a-required-prop-or-field-left-out
```

**Fix:** Give it, or declare a default in the type (`id: String = ""`) or
on the prop.

A built-in that cannot be placed without a prop is held to it the same way:
a `Portal` with no `into:` would draw nowhere, and the fix passes
`into: null` for you to name the node.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#c02-a-prop-the-component-does-not-declare
```

**Fix:** Correct the spelling, or declare the prop on the component.

### C03 — A part outside the component it belongs to

```wf expect C03
page P(path: "/", title: "T", description: "D") {
    Heading("Plans").h1
    Select.Option("Pro", value: "pro")
}
```

```text
error[C03]: `Select.Option` is a part of `Select`, and it is not inside one
 --> src/App.wf:3:5
  |
3 |     Select.Option("Pro", value: "pro")
  |     ^^^^^^^^^^^^^
  = help: Place it in a `Select { … }`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#c03-a-part-outside-the-component-it-belongs-to
```

**Fix:** Place the part inside its owner: `Select(bind: plan) {
Select.Option(…) }`.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#c04-an-attribute-a-built-in-does-not-declare
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#c05-a-positional-argument-a-built-in-does-not-take
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#c06-a-positional-argument-bound-by-order
```

**Fix:** Mark the prop the call means: `component Tag2(_ label: String)`.

### C07 — A drag or a gesture on a component with no one element

```wf expect C07
component Rows2(_ name: String) {
    for x in [1, 2] { Text(name) }
}
page P(path: "/", title: "T", description: "D") {
    Heading("Files").h1
    Rows2("a", drag: "a")
}
```

```text
error[C07]: `drag:` on `Rows2` has no element to land on: its body is a `for`
 --> src/App.wf:6:16
  |
6 |     Rows2("a", drag: "a")
  |                ^^^^
  = help: Wrap what `Rows2` draws in one element (a `Stack`, a `Row`), or write `drag:` inside it
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#c07-a-drag-or-a-gesture-on-a-component-with-no-one-element
```

`drag:`, `dragGroup:`, `dropValue:`, `dropAxis:`, `dropInside:` and the
handlers of the gestures — `on receive(d)`, `on dragmove(d)`, `on
longpress`, `on swipe` — written on your component's call belong to the one
element it draws. A body that is an `if`, a `for`, a `show`, a `match` or
several elements has no one element to give them.

**Fix:** Wrap what the component draws in one element — a `Stack`, a `Row`
— or write the drag inside it, on the element it is for.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#x01-an-assignment-to-something-that-cannot-change
```

**Fix:** Hold what changes in a `state`. A `derived` value changes when
what it reads does; a prop is its caller's (`emit` an event so the caller
changes it); a route parameter is the address's (`navigate`); a loop
variable is one pass's copy (change the item through its list).

### X02 — An effect that feeds itself

```wf expect X02
page P(path: "/", title: "T", description: "D") {
    state visits = 0
    effect { visits = visits + 1 }
    Heading("Visits {visits}").h1
}
```

```text
error[X02]: this effect writes `visits`, which it also reads, so each run starts the next and the page never settles
 --> src/App.wf:3:5
  |
3 |     effect { visits = visits + 1 }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: Work the value out with `derived` instead, or write it from the action that changes what it reads
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#x02-an-effect-that-feeds-itself
```

**Fix:** Work a value out with `derived`; change state from the action or
handler that changes what it depends on.

### X04 — A derived value that changes something

```wf expect X04
page P(path: "/", title: "T", description: "D") {
    state count = 0
    action next() {
        count = count + 1
        return count
    }
    derived ticket = next()
    Heading("Ticket {ticket}").h1
}
```

```text
error[X04]: `ticket` is worked out by calling `next(…)`, which assigns `count`
 --> src/App.wf:7:5
  |
7 |     derived ticket = next()
  |     ^^^^^^^^^^^^^^^^^^^^^^^
  = help: A derived value only reads; call the action from a handler, and derive from what it sets
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#x04-a-derived-value-that-changes-something
```

**Fix:** Call the action from a handler; derive from the state it sets.

### X05 — `use` of a store nothing declares

```wf expect X05
store Cart { state items = [] }
page P(path: "/", title: "T", description: "D") {
    use Carts
    Heading("Cart").h1
}
```

```text
error[X05]: `use Carts` names a store nothing declares
 --> src/App.wf:3:5
  |
3 |     use Carts
  |     ^^^
  = help: The stores are `Cart`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#x05-use-of-a-store-nothing-declares
```

**Fix:** Name a store that exists — here `Cart`.

### X06 — A state that copies a live value once

```wf expect X06
page P(path: "/", title: "T", description: "D") {
    state posted = @2026-03-14T09:30:00Z
    state clock = now(every: 1.seconds)
    derived age = posted.until(clock)
    Heading("Posted {age.minutes()} minutes ago").h1
}
```

```text
warning[X06]: `clock` copies `now` once, when the page opens, and never follows it
 --> src/App.wf:3:5
  |
3 |     state clock = now(every: 1.seconds)
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: Nothing assigns `clock`: declare it `derived`, which is worked out again whenever `now` changes
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#x06-a-state-that-copies-a-live-value-once
```

**Fix:** Declare it `derived` — the quick fix does. `derived clock =
now(every: 1.seconds)` is worked out again on every tick, and so is
everything that reads it. `now`, `viewport`, `query`, `hash`, `theme`,
`network` and `update` all keep themselves current; a `state` only takes
the value it starts with. A state the program assigns, binds or
changes in place is a starting value on purpose — `state tab = query.tab
?? "all"` that a click then changes — and draws nothing. Neither does a
live value read inside a lambda, which reads it each time it is called.

## Forms and bindings

### F01 — A `bind:` to something that cannot be written

```wf expect F01
page P(path: "/", title: "T", description: "D") {
    state first = "Ada"
    derived name = first.toUpperCase()
    Heading("Name").h1
    Input(bind: name, label: "Name")
}
```

```text
error[F01]: `bind:` on `Input` writes back to what it names, and `name` is a `derived` value
 --> src/App.wf:5:11
  |
5 |     Input(bind: name, label: "Name")
  |           ^^^^^^^^^^
  = help: Bind a `state` (or a store's state, or a field of a loop's item); show a value with `value:`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#f01-a-bind-to-something-that-cannot-be-written
```

**Fix:** Bind the state the reader edits (`bind: first`); show a value
that is worked out with `value:`.

### F02 — A `bind:` to a state the control cannot hold

```wf expect F02
page P(path: "/", title: "T", description: "D") {
    state qty = 1
    Heading("Order").h1
    Input(bind: qty, label: "Quantity")
}
```

```text
error[F02]: `bind:` on `Input` holds a `String`, but `qty` is `Number`
 --> src/App.wf:4:11
  |
4 |     Input(bind: qty, label: "Quantity")
  |           ^^^^^^^^^
  = help: A number field is `Input(…).number`, which holds a number
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#f02-a-bind-to-a-state-the-control-cannot-hold
```

**Fix:** A number field is `Input(…).number`; a text field holds a
`String`.

### F03 — A `validate` block no control can satisfy

```wf expect F03
page P(path: "/", title: "T", description: "D") {
    state email = ""
    validate email { required  email }
    Heading("Join").h1
    Form { Button("Join", type: .submit) }
}
```

```text
error[F03]: `validate email` guards a state no control binds, so its rules can never be met
 --> src/App.wf:3:5
  |
3 |     validate email { required  email }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: Bind it to the control the reader fills in: `Input(bind: email, …)`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#f03-a-validate-block-no-control-can-satisfy
```

**Fix:** Bind the state to the control the reader fills in:
`Input(bind: email, label: "Email").email`.

### F04 — A checkbox that shows a state and never changes it

Warning.

```wf expect F04
page P(path: "/", title: "T", description: "D") {
    state agree = false
    Heading("Terms").h1
    Checkbox(checked: agree, label: "I agree")
}
```

```text
warning[F04]: this `Checkbox` shows `checked:` but nothing changes it, so a click toggles the box and not the state
 --> src/App.wf:4:5
  |
4 |     Checkbox(checked: agree, label: "I agree")
  |     ^^^^^^^^
  = help: Write `bind:` to keep the box and the state together, or handle `on change`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#f04-a-checkbox-that-shows-a-state-and-never-changes-it
```

**Fix:** `Checkbox(bind: agree, label: "I agree")`.

### F05 — A select whose value is none of its options

Warning.

```wf expect F05
page P(path: "/", title: "T", description: "D") {
    state plan = "basic"
    Heading("Plan").h1
    Select(bind: plan, label: "Plan") {
        Select.Option("Free", value: "free")
        Select.Option("Pro", value: "pro")
    }
}
```

```text
warning[F05]: `plan` starts as `basic`, which is none of this select's options
 --> src/App.wf:4:5
  |
4 |     Select(bind: plan, label: "Plan") {
  |     ^^^^^^
  = help: Start it as one of them (`free`, `pro`), so what is shown is what is held
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#f05-a-select-whose-value-is-none-of-its-options
```

**Fix:** Start the state as one of the options' values.

## Routes

### R01 — A route to nothing

```wf expect R01
page Home(path: "/", title: "T", description: "D") {
    Heading("Home").h1
    Link("About us", to: "/abuot")
}
page About(path: "/about", title: "About", description: "D") {
    Heading("About").h1
}
```

```text
error[R01]: `/abuot` is not a route: no page's `path` matches it
 --> src/App.wf:3:5
  |
3 |     Link("About us", to: "/abuot")
  |     ^^^^
  = help: The nearest is `/about`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#r01-a-route-to-nothing
```

**Fix:** Link to a route a page has; the message names the nearest. A
spliced link is checked by its shape — `"/team/{t.slug}"` must match a route
like `/team/:slug` — a link with no leading `/` is refused (where it leads
would depend on the page it is on), and a link to a file (`/report.pdf`) or
another site is left alone.

### R02 — A route parameter and a page parameter that do not match

```wf expect R02
page Profile(path: "/u/:id", title: "T", description: "D") {
    Heading("Profile").h1
}
```

```text
error[R02]: the route `/u/:id` names `:id`, and the page declares no parameter `id`
 --> src/App.wf:1:1
  |
1 | page Profile(path: "/u/:id", title: "T", description: "D") {
  | ^^^^
  = help: Declare it: `page Profile(path: "/u/:id", id: String)`, then read `id`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#r02-a-route-parameter-and-a-page-parameter-that-do-not-match
```

**Fix:** Declare the parameter the route names — `page Profile(path:
"/u/:id", id: String)` — and read it by name; a declared parameter the route
has no `:name` for is never filled.

### R03 — An `app` with no `Router`, or with two

```wf expect R03
app {
    Navbar { Navbar.Brand { Text("Ledger") } }
}
page Home(path: "/", title: "T", description: "D") {
    Heading("Home").h1
}
```

```text
error[R03]: `app` places no `Router`, so no page is ever drawn
 --> src/App.wf:2:5
  |
2 |     Navbar { Navbar.Brand { Text("Ledger") } }
  |     ^^^^^^
  = help: Put `Router` where the current page belongs: `app { Navbar { … } Router }`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#r03-an-app-with-no-router-or-with-two
```

**Fix:** Place `Router` where the page belongs: `app { Navbar { … }
Router }`. (A program with no `app` at all routes its pages by itself.)

### R04 — A relative URL on a nested route

Warning.

```wf expect R04
page Post(path: "/blog/first", title: "T", description: "D") {
    Heading("First post").h1
    Image(src: "images/cover.png", alt: "")
}
```

```text
warning[R04]: `src: "images/cover.png"` is relative, so on `/blog/first` it is fetched from `/blog/images/cover.png`
 --> src/App.wf:3:5
  |
3 |     Image(src: "images/cover.png", alt: "")
  |     ^^^^^
  = help: Write it from the site's root: `"/images/cover.png"`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#r04-a-relative-url-on-a-nested-route
```

**Fix:** Write it from the site's root: `"/images/cover.png"`.

### R05 — A translation of no page, or two in one language

Error.

```wf expect R05
page Home(path: "/", title: "Home", description: "D") {
    Heading("Home").h1
}

page HomeEn(path: "/en", title: "Home", description: "D", lang: "en", translates: Hom) {
    Heading("Home").h1
}
```

```text
error[R05]: `HomeEn` translates `Hom`, and no page is named `Hom`
 --> src/App.wf:5:1
  |
5 | page HomeEn(path: "/en", title: "Home", description: "D", lang: "en", translates: Hom) {
  | ^^^^
  = help: Did you mean `Home`?
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#r05-a-translation-of-no-page-or-two-in-one-language
```

`translates:` names the page this one is a translation of, and the pages
it joins list each other as `hreflang` alternates, each in its own `lang:`.
Two pages of one group in the same language — or both with no `lang:`, so
both in the site's — would be two alternates for one language. **Fix:**
name a page the program declares, and give each translation the `lang:` it
is written in.

## Data and assets

### D01 — A file the project does not have

Warning. `Image(src: "/img/hero.png")` when `public/img/hero.png` does not
exist (also `poster:`, `captions:`, `transcript:`, and a worker's
`spawn("/workers/css.js")`): the page asks for it and gets a 404. **Fix:** put the file in `public/`, or correct the path.

### D02 — A persisted value storage cannot keep

```wf expect D02
page P(path: "/", title: "T", description: "D") {
    persist pick = (x) => x + 1
    Heading("Pick").h1
}
```

```text
error[D02]: `persist pick` keeps a function, which cannot be written to the browser's storage
 --> src/App.wf:2:5
  |
2 |     persist pick = (x) => x + 1
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = help: Keep what it is made from — a name, an id, the data — and build it again when it is read
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#d02-a-persisted-value-storage-cannot-keep
```

**Fix:** Keep what it is made from — a name, an id, the data — and build
the rest when it is read.

### D03 — A persisted value every instance shares

Warning.

```wf expect D03
component Panel(_ title: String) {
    persist open = false
    Card { Text(title) }
}
page P(path: "/", title: "T", description: "D") {
    Heading("Panels").h1
    Panel("One")
    Panel("Two")
}
```

```text
warning[D03]: `persist open` is in `Panel`, which is placed more than once, so every one of them reads and writes one stored value
 --> src/App.wf:2:5
  |
2 |     persist open = false
  |     ^^^^^^^
  = help: Give each its own: `persist open = false { key: id }`, with `id` a prop that tells them apart
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#d03-a-persisted-value-every-instance-shares
```

**Fix:** Give each instance its own key: `persist open = false { key: title
}`, with a prop that tells them apart.

### D04 — A persisted shape changed with no new version

Warning. The build remembers what each `persist` held (in
`.wf-cache/persist.json`); when that changes — `persist items = []` becomes
`persist items = { open: [], done: [] }` — and `version:` does not, a
returning reader's browser hands the old shape to code written for the new
one. **Fix:** raise `version:` and add `migrate 1 -> 2 { … }` to bring the
old value forward.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t01-a-value-of-the-wrong-type
```

**Fix:** Give the state a value of its type, or convert: `Number(text)`, `"{n}"`. If the state really holds either, declare it `Any`.

A number a built-in cannot be given is one too: a `Carousel(interval:)`
outside 1 to 2147483647 milliseconds (`-5000` used to become a 0 ms
autoplay), a `Column(span:)` that is not a whole number from 1 to 12.

```wf expect T01
page P(path: "/", title: "T", description: "D") {
    Heading("Grid").h1
    Row { Column(span: 13) { Text("Wide") } }
}
```

```text
error[T01]: `span` of Column is 13, but a whole number of the 12 columns, from 1 to 12, is wanted
 --> src/App.wf:3:11
  |
3 |     Row { Column(span: 13) { Text("Wide") } }
  |           ^^^^^^
  = help: `span: 6` is half the row
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t01-a-value-of-the-wrong-type
```

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t02-a-case-the-enum-does-not-have
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t04-a-value-that-may-be-null-read-as-if-it-were-not
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
  = help: Did you mean `name`? Its fields are `name`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t05-a-field-or-method-that-does-not-exist
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
 --> src/App.wf:4:20
  |
4 |     Heading("{Cart.count} items").h1
  |                    ^^^^^
  = help: Its members are `items`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t06-a-member-a-store-or-service-does-not-have
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t07-a-condition-that-is-always-true
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t08-a-loop-over-something-that-is-not-a-list
```

**Fix:** Loop over a list — `for n in 1..=total` for a range of numbers.

### T09 — An `emit` that does not match its event

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t09-an-emit-that-does-not-match-its-event
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t10-a-call-with-the-wrong-arguments
```

**Fix:** Pass the arguments the action, endpoint or function takes.

### T11 — A `match` on something that cannot be matched

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t11-a-match-on-something-that-cannot-be-matched
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t12-a-secret-where-it-would-escape
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
  = help: Did you mean `count`? Declare it — a `state`, a `const`, an `action`, a prop — or check the spelling. In the browser it would be a ReferenceError
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t13-a-name-nothing-declares
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t14-a-comparison-that-is-always-the-same
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t15-a-match-that-misses-a-case-or-has-one-twice
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t16-a-method-a-number-a-string-or-a-list-does-not-have
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t17-an-async-result-used-before-it-is-awaited
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t18-arithmetic-on-something-that-is-not-a-number
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t19-a-value-that-may-be-null-shown-as-text
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t21-a-resource-match-with-no-error-arm
```

**Fix:** `error(e) { Alert(e.message).danger }`, or an `else`.

### T22 — A handler that names more values than its event sends

Error.

```wf expect T22
component Entry(_ key: String) {
    event menu(key: String, x: Number, y: Number)
    Button(key) { on contextmenu(e) { e.preventDefault()  emit menu(key, e.clientX, e.clientY) } }
}

page P(path: "/", title: "T", description: "D") {
    state at = ""
    Heading("Files").h1
    Entry("notes.wf") { on menu(key, x, y, z) { at = "{key} at {x}, {y}" } }
    Text(at)
}
```

```text
error[T22]: `on menu` names 4 values, but `event menu(key: String, x: Number, y: Number)` sends 3 values
 --> src/App.wf:9:44
  |
9 |     Entry("notes.wf") { on menu(key, x, y, z) { at = "{key} at {x}, {y}" } }
  |                                            ^
  = help: Name at most 3, in order: `on menu(key, x, y)`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t22-a-handler-that-names-more-values-than-its-event-sends
```

A handler names the values its event sends, in the order the event
declares them; it may name fewer, never more. A DOM event — `click`,
`input`, `keydown` — sends one value, the event itself, so `on click(e, x)`
is this error too.

**Fix:** Drop the extra names. For a DOM event, name the event alone and
read what it carries from it: `on click(e) { … e.clientX … }`.

### T20 — A list or a record shown as text

Warning.

```wf expect T20
page P(path: "/", title: "T", description: "D") {
    state tags = ["a", "b"]
    Heading("Tags").h1
    Text("Tagged {tags}")
}
```

```text
warning[T20]: `tags` is `[String]`, and in text it shows as its items run together with commas
 --> src/App.wf:4:19
  |
4 |     Text("Tagged {tags}")
  |                   ^^^^
  = help: Show a field of it, or `.join(", ")` a list of text
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#t20-a-list-or-a-record-shown-as-text
```

**Fix:** `{tags.join(", ")}` for a list of text; a field of a record.

## Translations

These need the project's translation files, so the examples are described
rather than run.

### I01 — A message no translation has

`t("nav.blgo")` when no locale's file holds `nav.blgo`: the page shows
`nav.blgo`. **Fix:** the key the message suggests, or add it to the files.

### I02 — A message and its call that disagree

`t("greeting", { nme: user.name })` against `"greeting": "Hello, {name}!"`:
`nme` is not used, and `{name}` shows as written. **Fix:** pass what the
message shows, by the name it shows it.

### I03 — A message one locale has and another does not

Warning. `nav.blog` in `en.json` and not in `ar.json`: a reader in Arabic
sees `nav.blog`. **Fix:** translate it, or remove it from the others.

### I04 — A locale the project does not have

`setLocale("fr")` when `i18n.locales` is `["en", "ar"]`. **Fix:** add the
locale and its file, or name one that exists.

### I05 — Words a reader sees that are not translated

Warning, under `"literals": true`. `Text("{plural(n, "file")} found")`
splices a call, so no table can key it; `derived said = "{n} errors"`
shown as `Text(said)` is not looked up, because only a value whose name
says it is words (`saidText`, `errorsLabel`) is. A reader in Arabic sees
both in English. **Fix:** splice names, `derived filesText = "{n} files"`
then `Text("{filesText} found")`, and name what is words as words.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a01-an-image-with-no-alt-text
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a02-an-icon-button-with-no-name
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a03-an-input-with-no-label
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a04-a-control-with-no-label
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a05-a-button-with-no-text
```

**Fix:** Give it text — `Button("Save")`, or a `Text` in its block beside
the icon, `Button { Icon("check")  Text("Save") }` — or an `aria-label:`
when it shows only an icon. A `Text` anywhere in the block names it — in a
branch, in a component the block places, or reading state; a `Tooltip`'s
words, which show only on hover, do not.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a06-a-link-with-no-text
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a07-an-empty-heading
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a08-a-dialog-with-no-title
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a09-media-nobody-can-follow-without-sound
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
warning[A10]: Table has no header row (`Table.Head`)
 --> src/App.wf:3:5
  |
3 |     Table(caption: "Prices") { Table.Body { Table.Row { Table.Cell("Pen")  Table.Cell("2") } } }
  |     ^^^^^
  = help: Add one: `Table { Table.Head { Table.Row { Table.Cell("Name") } } … }`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a10-a-table-with-no-header-row
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a11-a-skipped-heading-level
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a12-a-page-without-exactly-one-h1
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a13-colours-without-enough-contrast
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a14-a-role-whose-children-are-the-wrong-kind
```

**Fix:** Give each child the role the parent requires (`role: "tab"`), or use the built-in that owns the structure — `Tabs`.

### A15 — An `aria-label` that does not contain the visible text

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a15-an-aria-label-that-does-not-contain-the-visible-text
```

**Fix:** Start the `aria-label` with the visible words: `aria-label: "Save the draft"`.

### A16 — One id on several elements

Warning.

```wf expect A16
page P(path: "/", title: "T", description: "D") {
    state rows = ["a", "b"]
    Heading("Rows").h1
    for r in rows by r { Text(r, id: "row") }
}
```

```text
warning[A16]: `id: "row"` is inside a `for`, so the page has several elements with one id
 --> src/App.wf:4:26
  |
4 |     for r in rows by r { Text(r, id: "row") }
  |                          ^^^^
  = help: Make it unique — `id: "row-{item.id}"` — or drop it; a label's `for` and `aria-*` find one element by id
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a16-one-id-on-several-elements
```

**Fix:** Make it unique — `id: "row-{r}"` — or drop it.

### A17 — A frame with no title

Warning.

```wf expect A17
page P(path: "/", title: "T", description: "D") {
    Heading("Preview").h1
    Frame(src: "https://example.com/")
}
```

```text
warning[A17]: Frame missing "title" attribute
 --> src/App.wf:3:5
  |
3 |     Frame(src: "https://example.com/")
  |     ^^^^^
  = help: Say what it shows: Frame(src: "/preview/", title: "Preview")
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#a17-a-frame-with-no-title
```

**Fix:** `Frame(src: "https://example.com/", title: "Example")` — the title is how a screen reader announces it.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#s01-a-page-with-no-title
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#s02-a-page-with-no-description
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#s03-a-description-too-long-for-a-search-result
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#s04-two-pages-on-one-route
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#p01-a-persisted-value-with-no-way-forward
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#p02-a-migration-that-never-runs
```

**Fix:** Raise `version:` to the step's target, or drop the step.

### P03 — A persisted value in a route-scoped store

```wf expect P03
store Filters(scope: .route) {
    persist search = ""
}
```

```text
warning[P03]: `search` is persisted in a store scoped to the route
 --> src/App.wf:2:5
  |
2 |     persist search = ""
  |     ^^^^^^^
  = help: The route change drops the store and the next read builds it again from storage, so the value comes straight back. Use `state` for what the route owns, or widen the scope for what outlives it
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#p03-a-persisted-value-in-a-route-scoped-store
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u01-a-state-nothing-reads
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u02-a-derived-value-nothing-reads
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u03-a-component-nothing-places
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u04-a-store-member-nothing-reads
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u05-an-action-nothing-calls
```

**Fix:** Call it, remove it, or name it `_reset`.

### U06 — Code after return

```wf expect U06
page P(path: "/", title: "T", description: "D") {
    state n = 0
    action reset() {
        n = 0
        return n
        log("reset")
    }
    Heading("Count {n}").h1
    Button("Reset") { on click { reset() } }
}
```

```text
warning[U06]: this never runs: the `return` above leaves first
 --> src/App.wf:6:9
  |
6 |         log("reset")
  |         ^^^^^^^^^^^^
  = help: Remove it, or move it above the `return`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u06-code-after-return
```

**Fix:** Remove it, or move it above the `return`.

### U07 — An allow that allows nothing

A `// wf-allow(CODE)` comment accepts one finding on purpose: on a line of
its own it covers the next line of code, and at the end of a line it covers
that line. It names codes (`U01`) or families (`U`), comma-separated, and
only what `lints` could lower — a warning, or an error that ships no broken
page. An allow that covers nothing it names is reported, so allows cannot
outlive what they were written for.

```wf expect U07
page P(path: "/", title: "T", description: "D") {
    // wf-allow(U01)
    state shown = 0
    Heading("Count {shown}").h1
}
```

```text
warning[U07]: `wf-allow(U01)` allows nothing: no `U01` is reported on the line it covers
 --> src/App.wf:2:5
  |
2 |     // wf-allow(U01)
  |     ^^^^^^^^^^^^^^^^
  = help: Remove it — what it was written for is gone
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u07-an-allow-that-allows-nothing
```

**Fix:** Remove the allow — what it was written for is gone.

### U08 — A condition that is always the same

```wf expect U08
page P(path: "/", title: "T", description: "D") {
    Heading("Beta").h1
    if false { Text("Coming soon") }
}
```

```text
warning[U08]: `false` is always false, so one branch never runs
 --> src/App.wf:3:8
  |
3 |     if false { Text("Coming soon") }
  |        ^^^^^
  = help: Use the value that decides, or remove the branch that cannot run
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u08-a-condition-that-is-always-the-same
```

**Fix:** Use the value that decides, or remove the branch that cannot run.

### U09 — An unkeyed loop whose items hold state

Warning.

```wf expect U09
page P(path: "/", title: "T", description: "D") {
    state rows = [{ id: 1, name: "a" }]
    Heading("Rows").h1
    for r in rows { Input(bind: r.name, label: "Name") }
}
```

```text
warning[U09]: this `for` has no `by`, and each item holds a `Input`, which starts again whenever the list changes
 --> src/App.wf:4:5
  |
4 |     for r in rows { Input(bind: r.name, label: "Name") }
  |     ^^^
  = help: Key it: `for r in … by r.id`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u09-an-unkeyed-loop-whose-items-hold-state
```

**Fix:** Key it: `for r in rows by r.id`.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#u10-an-else-no-value-reaches
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
  = help: did you mean `.primary`?
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v01-a-bare-word-that-means-nothing
```

**Fix:** Write the flag with its dot — `Button("Save").primary` — or declare the name. The word is also a name nothing declares, so the build stops on it as `T13`; the editor shows both.

A name a branch binds is declared through the whole of its block, however
deep the element that reads it: `if let why = e.reason { Row { Pill(why) } }`,
and a case's payload, `.failed(why) { … }`, draw nothing.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v03-markup-put-in-as-markup
```

**Fix:** Wrap markup from outside the project in `sanitize(…)`: `Unsafe.Html(sanitize(body))`.

A `Frame(srcdoc:)` drawing HTML from a value is named too: that HTML is
a document of its own, scripts and all. Unless the call says what the
frame may do, it may do nothing — the build gives it `sandbox=""` —
and the warning says so. Writing `sandbox:` out is the reviewed case;
`allow-scripts` with `allow-same-origin` undoes the sandbox, and is
named again.

```wf expect V03
page P(path: "/", title: "T", description: "D") {
    state html = "<p>Hi</p>"
    Heading("Preview").h1
    Frame(srcdoc: html, title: "Preview")
}
```

```text
warning[V03]: `Frame(srcdoc:)` draws HTML from a value as a document; it is sandboxed (`sandbox=""`), so nothing in it runs
 --> src/App.wf:4:5
  |
4 |     Frame(srcdoc: html, title: "Preview")
  |     ^^^^^
  = help: Say what it may do with `sandbox:` — `sandbox: ""` keeps it as it is now; `allow-scripts` and `allow-same-origin` together undo the sandbox
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v03-markup-put-in-as-markup
```

**Fix:** `Frame(srcdoc: html, title: "Preview", sandbox: "")`.

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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v04-a-class-that-is-the-engine-s
```

**Fix:** Name a class of your own and style it in a `.css` file under `src/`, or use the flag that sets the look you wanted (`.primary`, `.elevated`).

### V05 — A CSS property no browser knows

Warning.

```wf expect V05
page P(path: "/", title: "T", description: "D") {
    Heading("Hello") {
        style { colr: red }
    }
}
```

```text
warning[V05]: `colr` is not a CSS property, so the browser ignores it
 --> src/App.wf:3:17
  |
3 |         style { colr: red }
  |                 ^^^^
  = help: Did you mean `color`?
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v05-a-css-property-no-browser-knows
```

**Fix:** The property the message suggests — here `color`. A custom
property is written `--name`.

### V06 — A design token the theme does not declare

```wf expect V06
page P(path: "/", title: "T", description: "D") {
    Heading("Hello") {
        style { color: $brnad }
    }
}
```

```text
error[V06]: `$brnad` is no token the theme declares, so it is never set
 --> src/App.wf:3:17
  |
3 |         style { color: $brnad }
  |                 ^^^^^
  = help: Declare it in your `theme { … }`, or use one the theme has
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v06-a-design-token-the-theme-does-not-declare
```

**Fix:** A token the theme has — a short name resolves through the
property (`padding: $lg`, `border: 1px solid $border`) — or declare it in
your `theme { … }`.

### V07 — A number where CSS wants a length

Warning.

```wf expect V07
page P(path: "/", title: "T", description: "D") {
    state pct = 40
    Heading("Progress").h1
    Card { style { width: {pct} } }
}
```

```text
warning[V07]: `width: {pct}` is a number with no unit, which the browser drops
 --> src/App.wf:4:20
  |
4 |     Card { style { width: {pct} } }
  |                    ^^^^^^^^^^^^
  = help: Give it one: `width: {pct}px` (or `%`, `rem`)
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v07-a-number-where-css-wants-a-length
```

**Fix:** Give it a unit: `width: {pct}%`.

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
  = help: The icons: close, menu, search, home, user, settings, check, plus, minus, edit, trash, star, heart, mail, bell, download, upload, eye, link, calendar, filter, chevron-down, chevron-right, chevron-left, info, warning, arrow-left, arrow-right, logout, copy, sun, moon, folder, folder-plus, folder-off, file, file-plus, refresh, alert-circle, clock, dot, diff, book, git-pull-request, pin, git-branch, more-horizontal, window-minimize, window-maximize, outline, help-circle, collapse-all, locate, chevron-up, globe, external-link, panel-down, panel-up, code, monitor, git-commit, play, layers, insert, grid, list, square, pill, image, sparkle, lock, type, pointer, reach, frame, line, pipette, comment, hand, zoom, ruler, eye-off, unlock
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v08-an-icon-the-runtime-does-not-draw
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
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v09-an-event-the-element-does-not-fire
```

**Fix:** Name a DOM event (`click`, `input`, `change`, `submit`, `keydown`,
…) or one the component declares.

### V10 — A `Host` tag it does not make

A `Host` is made of one of a few elements: `div`, `span`, `canvas`, `svg`,
`section`, `figure`, `pre`, `p`, `ul` or `table` — the tag is written into
the document, so it is an allow-list rather than whatever the string says.
Any other tag is a `div`.

```wf expect V10
page P(path: "/", title: "T", description: "D") {
    Heading("Chart").h1
    Host(tag: "video", mount: (n) => n)
}
```

```text
warning[V10]: `Host` makes no `video`, so it is a `div`
 --> src/App.wf:3:10
  |
3 |     Host(tag: "video", mount: (n) => n)
  |          ^^^
  = help: It makes `div`, `span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul`, `table`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v10-a-host-tag-it-does-not-make
```

**Fix:** Make it one of the tags `Host` makes, or use the element the
built-ins offer for it (`Video`, `Audio`, `Image`).

### V11 — A component named like a built-in

A call of a built-in's name draws the built-in, so a component declared
with that name is never drawn: its body, its props, its slots go unused, and
what the page shows is the built-in instead. It happens to a component
written before WebFluent gained an element of the same name — a `Footer`
from WebFluent 2, a `Chart` from before 5.2.

```wf expect V11
component Chart(_ title: String) {
    Text(title)
}

page P(path: "/", title: "T", description: "D") {
    Heading("Sales").h1
    Chart(kind: .bar, data: [3, 5, 2])
}
```

```text
warning[V11]: `Chart` is a built-in component, so a call of that name draws the built-in, never this one
 --> src/App.wf:1:1
  |
1 | component Chart(_ title: String) {
  | ^^^^^^^^^
  = help: Rename it — `MyChart`, `ReportChart` — and its call sites
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v11-a-component-named-like-a-built-in
```

**Fix:** Rename the component and its call sites.

### V12 — A long list that is not windowed

A `VirtualList` draws only the rows in view of the one keyed `for` that is
the whole of its body. With anything else beside the `for`, or a `for` with
no `by` — which has no way to keep a row's nodes as it scrolls — it draws
every row, as a plain scrolling box does, and the length of the list is
what the page pays for.

```wf expect V12
page P(path: "/", title: "T", description: "D") {
    state rows = ["a", "b"]
    Heading("Files").h1
    VirtualList(rowHeight: 32) {
        for r in rows { Text(r) }
    }
}
```

```text
warning[V12]: This `VirtualList` draws every row: its body is not one keyed `for`
 --> src/App.wf:4:5
  |
4 |     VirtualList(rowHeight: 32) {
  |     ^^^^^^^^^^^
  = help: Make its body one `for` with a `by`: `VirtualList(rowHeight: 32) { for row in rows by row.id { … } }`
  = docs: https://webfluent.monzeromer.dev/docs/guide/diagnostics#v12-a-long-list-that-is-not-windowed
```

**Fix:** Give the `for` a `by`, and move anything else — a heading, a
count — outside the `VirtualList`.
## Next

[Built-ins](../reference/built-ins.md).
