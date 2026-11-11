# Fixing what the compiler reports

Every finding has a code, a place, a message and usually a hint. Many come
with a fix. The loop:

```bash
wf check                  # every finding, nothing written
wf explain T06            # what a code means, an example, the fix
wf fix --dry-run          # the fixes the compiler knows, as a diff
wf fix --code C02         # apply one code's fixes
wf check --format json    # the same findings for a tool, with `fixes`
```

Read the message: it names what's wrong and, usually, the nearest right
thing ("did you mean `.center`?", "its fields are `id`, `name`"). Fix the
cause, never the symptom.
- **Don't delete what a finding is about** to make it go away.
- **Don't cast to `Any`** to get past a type error.
- **Accepting one finding.** If a finding is understood and must stay,
  write `// wf-allow(CODE)` on the line, with a comment saying why. Only what
  `lints` could lower may be allowed, and an allow that covers nothing is
  `U07`.

Errors stop the build; warnings don't. Treat warnings as work to do:
`wf check --deny-warnings` in CI.

## The ones models hit most

| Code | What it means | The fix |
|---|---|---|
| `T06` | A store (or `api`) member that doesn't exist: `Cart.total` when it declares `sum` | Read the store (`wf types --json`) and use its member; never guess |
| `C02` | A prop your component doesn't declare | Read the component, then add the prop or use the right name |
| `C04` | A named argument a built-in doesn't declare, written as an HTML attribute | Usually a misspelled prop; `wf components --json` lists the real ones |
| `E103` | A flag, case, part, event or slot that doesn't exist (`Button(…).ghost`) | Use one the component takes; the message lists them |
| `T13` | A name nothing declares, or a flag without its dot (`gap: md`) | Declare it, fix the spelling, or write `.md` |
| `V01` | A bare word in an argument nothing in scope declares | Usually a missing dot: `align: .center` |
| `X05` | `use` of a store nothing declares | Name it exactly, with its capital |
| `E002` | A syntax error | See [Syntax](#syntax) below |
| `E105` | A statement where elements go (`n = n + 1` loose in a page) | Put it in an action or an `on` handler |
| `E113` | A second positional argument | Only the first argument may be positional, and only for a prop marked `_` |
| `E118` | A name the language owns, declared as yours: `t`, `env`, `query`, `hash`, `now`, `theme`, `dir`, `params`, `locale`, `show`, `state`… | Rename by what it holds: `todo`, `search`, `stage` |
| `T04` | A value that may be null read as if it weren't (`xs[0]`, `filter(…)[0]`, `user.name` on a `User?`) | `if let x = v { … }`, `v?.name`, `v ?? fallback`, or `xs.find(…)` |
| `T05` | A field a record doesn't have | The message lists the fields |
| `T01` | A value of the wrong type for a prop, state, field or argument | Convert it, or fix the type you declared |
| `T17` | An async result used before it's awaited, or `await` where it can't run | `await` it inside an action or handler |
| `T21` | A resource `match` with no `error` arm | Add `error(e) { … }` |
| `T15` | A `match` missing a case of the enum | Add the arm. Prefer that to adding an `else` |
| `F01` | `bind:` to something that can't be written (derived, prop, const) | Bind a `state`, or have the component `emit` |
| `F02` | `bind:` to a state the control can't hold (a text `Input` on a `Number`, a `DatePicker` on a `String`) | `.number` holds a number; a `DatePicker` binds a `Date?` |
| `X01` | An assignment to something that can't change: a `const`, a `derived`, a prop, a route parameter, a loop item | Assign the state it derives from, or emit an event |
| `X02` | An effect that writes what it reads, so it never settles | Make the value a `derived` |
| `X06` | A `state` that reads `now`/`viewport`/`query`/`theme` once and stays | Make it `derived` |
| `R01` | A link or `navigate()` to a route no page has | Fix the path, or add the page |
| `R02` | A route `:param` with no page parameter of that name | `page P(path: "/x/:id", id: String)` |
| `I01` | `t("key")` that no locale has | Add the key to every `translations/*.json` |
| `I03` | A key one locale has and another lacks | `wf translations --missing <locale>` |
| `E108` | A non-public `env` name read from a page | Name it `PUBLIC_…`, or keep the value on a server |
| `T12` | A `Secret` shown, spliced, logged or persisted | Keep it in Rust or on a server; never in the page |
| `D01` | A `src:` naming a file `public/` doesn't have | Fix the path; `public/x.png` is `/x.png` |
| `V06` | A `$token` the theme doesn't declare | A token `wf theme` lists, or declare it |
| `V05` | A style property CSS doesn't have | The message names the nearest one |
| `V07` | A bare number spliced as a length | Add the unit: `{pct}%` |
| `V08` | An icon the runtime doesn't draw | One of the built-in icons (`wf registry --json`) |
| `V11` | Your component is named like a built-in (`Footer`), so the built-in is drawn | Rename it (`SiteFooter`) |
| `U01`–`U05` | Declared but never read, placed or called | Delete it, or prefix `_` if that is on purpose |
| `U09` | A `for` with no `by` whose items hold state | `for x in xs by x.id` |
| `A01`–`A17` | Accessibility: missing `alt`, label, heading, title, captions… | Add what's missing; the message says what |
| `S01`–`S03` | A page with no title or description, or one too long | Give every page `title:` and `description:` |

`wf explain` prints any code's full entry. `wf explain` with no code lists
them all.

## Syntax

`E002` is the parser's. These are the usual causes:

| Wrong | Right |
|---|---|
| `Button.primary("Save")` | `Button("Save").primary`: flags go after the parentheses |
| `Text("a"), Text("b")` | a new line, or `;`, between two statements |
| `style { padding: 1rem, margin: 0 }` | `style { padding: 1rem; margin: 0 }` |
| `match query.tab { "a" { … } }` | `match` chooses by enum case; use `if … else if …` for strings, or map the string to an enum in a `derived` |
| `Page Home { … }` | `page Home(path: "/") { … }` (`E004`: WebFluent 2) |
| `on:click={…}` | `on click { … }` inside the element's block |
| a `style { }` after the children | `style` comes first in its block, then `transition`, `on …`, slot fills, children |
| `if x { … } else if y { … }` missing a brace | `wf fmt` refuses to format a file it can't parse; the error names the line |

## What the compiler doesn't catch

These pass `wf check` and are still wrong:

- **An assignment to a name the language owns.** Assigning `dir = 1`
  without `let` in an action isn't reported, and it overwrites the
  document's direction. **`let` every local.**
- **A hover rule that reads state.** Nested rules are compiled at build time,
  so `&:hover { background: {tone} }` can't follow state. Use a custom
  property: `--hover: {tone}` and `&:hover { background: var(--hover) }`.
- **A raw colour.** It doesn't follow dark mode. Use a token.
- **Physical sides.** `left`/`right` don't flip in Arabic. Use logical
  properties.
- **A server that answers with a different shape.** Under `wf serve`, a
  response is checked against its declared type and a mismatch is a `.parse`
  error naming the place. In production it isn't checked. Declare the type
  so `wf serve` can check it.
- **Layout by `viewport`.** It works, but the first paint is wrong. Use
  responsive values.

## The wf on PATH may not be the one the docs describe

`wf --version` names a release. A project's `AGENTS.md` describes the
version that made it. If the compiler refuses something the reference shows
(or the reverse), compare `wf --version` with the version at the top of
`AGENTS.md`. Then believe the compiler, and say so to the person you're
working with.
