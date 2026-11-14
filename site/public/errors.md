# Diagnostics

> Every WebFluent error and warning, by code — what it means, a program that draws it, the message the compiler prints, and the fix.

The page: https://webfluent.monzeromer.dev/errors

This chapter is the reference for every finding the compiler reports. Each
finding has a code, such as `T05`, and each code has a page of its own,
listed below; `wf explain T05` prints the same entry in your terminal.

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
  = docs: https://webfluent.monzeromer.dev/errors/T05
```

From the top, a finding shows:

1. its severity, its code and what is wrong;
2. the place, in the `file:line:col` form that editors and CI problem
   matchers read;
3. the line of source, with the part that is wrong underlined;
4. `help:` — what to do about it, when the compiler can say;
5. `docs:` — a link to the code's page.

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
([Command line](https://webfluent.monzeromer.dev/reference/cli.md#wf-check)):

| Format | Output |
|---|---|
| `json` | One document on standard output, with every finding's code, place, hint, related places and fixes. |
| `sarif` | SARIF 2.1.0, for GitHub code scanning. |
| `github` | `::error` annotations, which a GitHub workflow shows on the pull request. |

## Fixes

Where the compiler knows the fix, the finding carries it as edits. Your
editor offers it as a quick fix, `--format json` carries it in `fixes`, and
[`wf fix`](https://webfluent.monzeromer.dev/reference/cli.md#wf-fix) applies it. A fix is offered only where it is
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
([Configuration](https://webfluent.monzeromer.dev/reference/config.md#lints)).

## Allowing one finding

To accept one finding on purpose, write a `// wf-allow(CODE)` comment:

- on a line of its own, it covers the next line of code;
- at the end of a line, it covers that line.

It names codes or families, comma-separated — `// wf-allow(U01, A)`. Only a
finding that [`lints`](https://webfluent.monzeromer.dev/reference/config.md#lints) could lower may be
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
| `R` | routes |
| `X` | state and reactivity |
| `I` | translations |
| `F` | forms and bindings |
| `D` | data and assets |
| `A` | accessibility |
| `S` | search and sharing |
| `P` | what is kept in the browser |
| `U` | declared and never read |
| `V` | vocabulary |

Each code's page has the code and its title as the heading, what it means
and the fix. Most pages also show a program that draws it, the message the
compiler prints for that program, and the program put right. The test
suite compiles every one of those programs, checks that the broken one
draws the code it is filed under and the fixed one draws nothing, and
checks that the message shown is what the compiler prints.

Some codes depend on more than one file can show: the files in `public/`
(`D01`), the config (`E108`, `E109`, `E111`, `E112`), the project's scripts
(`E110`, `E115`), its translations (`I01`–`I05`), the layout of a `.wfx`
file (`E003`) or the compiler's own output (`E901`, `E902`). Their pages
describe the case in words, or show a program that is not compiled and
say what project it needs.

## Trying an example

An example is a whole program. Put it in `src/App.wf` of a project with no
other `.wf` file under `src/` and no `theme` in `webfluent.app.json`. A
project `wf init` made already has a page at `/`, a `type User` and a
`theme Day`; an example pasted beside them draws `S04` (two pages claim
one route) or `E102` (a name declared twice) instead of its own code.

## Every code

<!-- codes: written by scripts/docsite/errors.py -->
### Syntax and structure

Errors, except where an entry says otherwise.

- [E001](https://webfluent.monzeromer.dev/errors/E001.md) — Text the compiler cannot read
- [E002](https://webfluent.monzeromer.dev/errors/E002.md) — A syntax error
- [E003](https://webfluent.monzeromer.dev/errors/E003.md) — Indentation that lines up with no block
- [E004](https://webfluent.monzeromer.dev/errors/E004.md) — A file in the WebFluent 2 grammar
- [E005](https://webfluent.monzeromer.dev/errors/E005.md) — A file the formatter will not change
- [E101](https://webfluent.monzeromer.dev/errors/E101.md) — A component nothing declares
- [E102](https://webfluent.monzeromer.dev/errors/E102.md) — A name declared twice
- [E103](https://webfluent.monzeromer.dev/errors/E103.md) — A flag, case, part, event or slot the component does not have
- [E104](https://webfluent.monzeromer.dev/errors/E104.md) — A layout with nowhere to put the page
- [E105](https://webfluent.monzeromer.dev/errors/E105.md) — A statement a render block cannot hold
- [E106](https://webfluent.monzeromer.dev/errors/E106.md) — Script or a whole style in an attribute
- [E107](https://webfluent.monzeromer.dev/errors/E107.md) — A URL a browser would run
- [E108](https://webfluent.monzeromer.dev/errors/E108.md) — An `env` name that is not public
- [E109](https://webfluent.monzeromer.dev/errors/E109.md) — An element this output cannot draw
- [E110](https://webfluent.monzeromer.dev/errors/E110.md) — A project script that cannot be a plain script
- [E111](https://webfluent.monzeromer.dev/errors/E111.md) — A setting that cannot work
- [E112](https://webfluent.monzeromer.dev/errors/E112.md) — A setting nothing reads
- [E113](https://webfluent.monzeromer.dev/errors/E113.md) — An argument that cannot be positional
- [E114](https://webfluent.monzeromer.dev/errors/E114.md) — `emit` outside a component
- [E115](https://webfluent.monzeromer.dev/errors/E115.md) — A project script the compiler could not read
- [E116](https://webfluent.monzeromer.dev/errors/E116.md) — A tag that cannot be a custom element's
- [E117](https://webfluent.monzeromer.dev/errors/E117.md) — A key combination no keyboard sends
- [E118](https://webfluent.monzeromer.dev/errors/E118.md) — A name the language already has
- [E901](https://webfluent.monzeromer.dev/errors/E901.md) — The compiler wrote JavaScript a browser would refuse
- [E902](https://webfluent.monzeromer.dev/errors/E902.md) — Output its own security policy refuses
- [E119](https://webfluent.monzeromer.dev/errors/E119.md) — A hidden direction control in the source
- [E120](https://webfluent.monzeromer.dev/errors/E120.md) — Something nests too deeply
- [E121](https://webfluent.monzeromer.dev/errors/E121.md) — A render that would do too much

### Types

Errors. The type checker's findings; a program that declares no types raises none of them, since what it cannot resolve is `Any`.

- [T01](https://webfluent.monzeromer.dev/errors/T01.md) — A value of the wrong type
- [T02](https://webfluent.monzeromer.dev/errors/T02.md) — A case the enum does not have
- [T04](https://webfluent.monzeromer.dev/errors/T04.md) — A value that may be null, read as if it were not
- [T05](https://webfluent.monzeromer.dev/errors/T05.md) — A field or method that does not exist
- [T06](https://webfluent.monzeromer.dev/errors/T06.md) — A member a store or service does not have
- [T07](https://webfluent.monzeromer.dev/errors/T07.md) — A condition that is always true
- [T08](https://webfluent.monzeromer.dev/errors/T08.md) — A loop over something that is not a list
- [T09](https://webfluent.monzeromer.dev/errors/T09.md) — An `emit` that does not match its event
- [T10](https://webfluent.monzeromer.dev/errors/T10.md) — A call with the wrong arguments
- [T11](https://webfluent.monzeromer.dev/errors/T11.md) — A `match` on something that cannot be matched
- [T12](https://webfluent.monzeromer.dev/errors/T12.md) — A secret where it would escape
- [T13](https://webfluent.monzeromer.dev/errors/T13.md) — A name nothing declares
- [T16](https://webfluent.monzeromer.dev/errors/T16.md) — A method a number, a string or a list does not have
- [T14](https://webfluent.monzeromer.dev/errors/T14.md) — A comparison that is always the same
- [T17](https://webfluent.monzeromer.dev/errors/T17.md) — An async result used before it is awaited
- [T19](https://webfluent.monzeromer.dev/errors/T19.md) — A value that may be null, shown as text
- [T20](https://webfluent.monzeromer.dev/errors/T20.md) — A list or a record shown as text
- [T18](https://webfluent.monzeromer.dev/errors/T18.md) — Arithmetic on something that is not a number
- [T15](https://webfluent.monzeromer.dev/errors/T15.md) — A `match` that misses a case, or has one twice
- [T22](https://webfluent.monzeromer.dev/errors/T22.md) — A handler that names more values than its event sends
- [T21](https://webfluent.monzeromer.dev/errors/T21.md) — A resource `match` with no `error` arm

### Components

- [C01](https://webfluent.monzeromer.dev/errors/C01.md) — A required prop or field left out
- [C02](https://webfluent.monzeromer.dev/errors/C02.md) — A prop the component does not declare
- [C03](https://webfluent.monzeromer.dev/errors/C03.md) — A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md) — An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md) — A positional argument a built-in does not take
- [C06](https://webfluent.monzeromer.dev/errors/C06.md) — A positional argument bound by order
- [C07](https://webfluent.monzeromer.dev/errors/C07.md) — A drag or a gesture on a component with no one element

### Routes

- [R01](https://webfluent.monzeromer.dev/errors/R01.md) — A route to nothing
- [R02](https://webfluent.monzeromer.dev/errors/R02.md) — A route parameter and a page parameter that do not match
- [R03](https://webfluent.monzeromer.dev/errors/R03.md) — An `app` with no `Router`, or with two
- [R04](https://webfluent.monzeromer.dev/errors/R04.md) — A relative URL on a nested route
- [R05](https://webfluent.monzeromer.dev/errors/R05.md) — A translation of no page, or two in one language

### State and reactivity

- [X01](https://webfluent.monzeromer.dev/errors/X01.md) — An assignment to something that cannot change
- [X02](https://webfluent.monzeromer.dev/errors/X02.md) — An effect that feeds itself
- [X04](https://webfluent.monzeromer.dev/errors/X04.md) — A derived value that changes something
- [X05](https://webfluent.monzeromer.dev/errors/X05.md) — `use` of a store nothing declares
- [X06](https://webfluent.monzeromer.dev/errors/X06.md) — A state that copies a live value once

### Forms and bindings

- [F01](https://webfluent.monzeromer.dev/errors/F01.md) — A `bind:` to something that cannot be written
- [F02](https://webfluent.monzeromer.dev/errors/F02.md) — A `bind:` to a state the control cannot hold
- [F03](https://webfluent.monzeromer.dev/errors/F03.md) — A `validate` block no control can satisfy
- [F04](https://webfluent.monzeromer.dev/errors/F04.md) — A checkbox that shows a state and never changes it
- [F05](https://webfluent.monzeromer.dev/errors/F05.md) — A select whose value is none of its options

### Data and assets

- [D01](https://webfluent.monzeromer.dev/errors/D01.md) — A file the project does not have
- [D02](https://webfluent.monzeromer.dev/errors/D02.md) — A persisted value storage cannot keep
- [D03](https://webfluent.monzeromer.dev/errors/D03.md) — A persisted value every instance shares
- [D04](https://webfluent.monzeromer.dev/errors/D04.md) — A persisted shape changed with no new version
- [D05](https://webfluent.monzeromer.dev/errors/D05.md) — An asset from another origin with no integrity hash

### Translations

These need the project's translation files, so the examples are described
rather than run.

- [I01](https://webfluent.monzeromer.dev/errors/I01.md) — A message no translation has
- [I02](https://webfluent.monzeromer.dev/errors/I02.md) — A message and its call that disagree
- [I03](https://webfluent.monzeromer.dev/errors/I03.md) — A message one locale has and another does not
- [I04](https://webfluent.monzeromer.dev/errors/I04.md) — A locale the project does not have
- [I05](https://webfluent.monzeromer.dev/errors/I05.md) — Words a reader sees that are not translated

### Accessibility

Warnings. What makes a page unusable with a screen reader, a keyboard or a voice.

- [A01](https://webfluent.monzeromer.dev/errors/A01.md) — An image with no alt text
- [A02](https://webfluent.monzeromer.dev/errors/A02.md) — An icon button with no name
- [A03](https://webfluent.monzeromer.dev/errors/A03.md) — An input with no label
- [A04](https://webfluent.monzeromer.dev/errors/A04.md) — A control with no label
- [A05](https://webfluent.monzeromer.dev/errors/A05.md) — A button with no text
- [A06](https://webfluent.monzeromer.dev/errors/A06.md) — A link with no text
- [A07](https://webfluent.monzeromer.dev/errors/A07.md) — An empty heading
- [A08](https://webfluent.monzeromer.dev/errors/A08.md) — A dialog with no title
- [A09](https://webfluent.monzeromer.dev/errors/A09.md) — Media nobody can follow without sound
- [A10](https://webfluent.monzeromer.dev/errors/A10.md) — A table with no header row
- [A11](https://webfluent.monzeromer.dev/errors/A11.md) — A skipped heading level
- [A12](https://webfluent.monzeromer.dev/errors/A12.md) — A page without exactly one h1
- [A13](https://webfluent.monzeromer.dev/errors/A13.md) — Colours without enough contrast
- [A14](https://webfluent.monzeromer.dev/errors/A14.md) — A role whose children are the wrong kind
- [A15](https://webfluent.monzeromer.dev/errors/A15.md) — An `aria-label` that does not contain the visible text
- [A16](https://webfluent.monzeromer.dev/errors/A16.md) — One id on several elements
- [A17](https://webfluent.monzeromer.dev/errors/A17.md) — A frame with no title
- [A18](https://webfluent.monzeromer.dev/errors/A18.md) — A popover with no name

### Search and sharing

Warnings. What a search engine or a link preview would act on.

- [S01](https://webfluent.monzeromer.dev/errors/S01.md) — A page with no title
- [S02](https://webfluent.monzeromer.dev/errors/S02.md) — A page with no description
- [S03](https://webfluent.monzeromer.dev/errors/S03.md) — A description too long for a search result
- [S04](https://webfluent.monzeromer.dev/errors/S04.md) — Two pages on one route

### What is kept in the browser

Warnings. `persist` values across releases.

- [P01](https://webfluent.monzeromer.dev/errors/P01.md) — A persisted value with no way forward
- [P02](https://webfluent.monzeromer.dev/errors/P02.md) — A migration that never runs
- [P03](https://webfluent.monzeromer.dev/errors/P03.md) — A persisted value in a route-scoped store

### Declared and never read

Warnings. A name beginning `_` is taken as unused on purpose.

- [U01](https://webfluent.monzeromer.dev/errors/U01.md) — A state nothing reads
- [U02](https://webfluent.monzeromer.dev/errors/U02.md) — A derived value nothing reads
- [U03](https://webfluent.monzeromer.dev/errors/U03.md) — A component nothing places
- [U04](https://webfluent.monzeromer.dev/errors/U04.md) — A store member nothing reads
- [U05](https://webfluent.monzeromer.dev/errors/U05.md) — An action nothing calls
- [U06](https://webfluent.monzeromer.dev/errors/U06.md) — Code after return
- [U07](https://webfluent.monzeromer.dev/errors/U07.md) — An allow that allows nothing
- [U08](https://webfluent.monzeromer.dev/errors/U08.md) — A condition that is always the same
- [U09](https://webfluent.monzeromer.dev/errors/U09.md) — An unkeyed loop whose items hold state
- [U10](https://webfluent.monzeromer.dev/errors/U10.md) — An `else` no value reaches

### Vocabulary

Warnings.

- [V01](https://webfluent.monzeromer.dev/errors/V01.md) — A bare word that means nothing
- [V02](https://webfluent.monzeromer.dev/errors/V02.md) — A flag no stylesheet styles
- [V03](https://webfluent.monzeromer.dev/errors/V03.md) — Markup put in as markup
- [V04](https://webfluent.monzeromer.dev/errors/V04.md) — A class that is the engine's
- [V05](https://webfluent.monzeromer.dev/errors/V05.md) — A CSS property no browser knows
- [V06](https://webfluent.monzeromer.dev/errors/V06.md) — A design token the theme does not declare
- [V07](https://webfluent.monzeromer.dev/errors/V07.md) — A number where CSS wants a length
- [V08](https://webfluent.monzeromer.dev/errors/V08.md) — An icon the runtime does not draw
- [V09](https://webfluent.monzeromer.dev/errors/V09.md) — An event the element does not fire
- [V12](https://webfluent.monzeromer.dev/errors/V12.md) — A long list that is not windowed
- [V10](https://webfluent.monzeromer.dev/errors/V10.md) — A `Host` tag it does not make
- [V11](https://webfluent.monzeromer.dev/errors/V11.md) — A component named like a built-in

<!-- /codes -->

## Next

[Built-ins](https://webfluent.monzeromer.dev/reference/built-ins.md).
