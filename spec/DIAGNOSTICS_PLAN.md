# Diagnostics — the plan

> Approved 2026-10-03, against 4.3.2. Decisions are recorded at the end.
> Author: Monzer Omer · Date: 2026-10-03
>
> Every `wf-next` block is **proposed** behaviour: it does not compile (or
> does not report) today.

WebFluent's promise is *"one lexeme, one meaning, and anything that resolves
to nothing is an error, never a silent no-op."* The registry keeps that
promise for built-ins. Everywhere else it leaks. Three investigations —
the diagnostics plumbing, the checkers, and ~100 broken programs run in a real
Chrome — found that a large share of the mistakes an author makes **compile
cleanly and fail in the browser**, and that the diagnostics we do have are
hard to read, hard to consume, and inconsistent between the build and the
editor.

**The rule:** a mistake the compiler can see is reported where it is
written, once, with a code, a precise span, a hint, and — where there is one
right answer — a fix. A mistake the compiler can *prevent* by generating
different code, it prevents. What only a browser can see, the dev server and
`wf verify` show.

---

## What was measured

Everything below was reproduced; programs are in `/tmp/claude-1000/diag-{a,b,c}`.

**The plumbing.**
- Eight diagnostic shapes and no common one: `Diagnostic` has no severity,
  no code field, no end position and no related locations. Type-checker
  codes are text spliced into the message (`"[T05] …"`). PDF and slides
  errors carry no location at all.
- 43 codes exist. About **165 error sites have none**: every parse,
  structural, security and output-mode error.
- Byte spans exist in the AST and are thrown away. The column is re-found
  by searching the message for its first backticked word
  (`sema/types.rs:3145`).
- One finding often prints 2–4 times (`Heading(user.nmae)` gives three
  identical T05s). Then every error prints again in a summary labelled
  "Codegen Error".
- No source snippet, no underline, no colour. The location comes after the
  message, so `file:line:col:` problem matchers cannot read it.
- The build **stops at the first failing stage**. Parsing stops at the
  first error in the whole project, so one typo in `C.wf` hides every
  mistake in `A.wf` and `B.wf`.
- No machine-readable build output, and one exit code for everything.
- The check pipeline is copied four times (build, LSP, docs test, studio),
  and the copies disagree. The LSP never runs the `env`, script, PDF,
  config, offline or CSP checks.
- The LSP sends no code for T- and structural errors, underlines only the
  first word, and has no related locations, no "unnecessary" tags and no
  doc links. Its one quick fix scrapes "did you mean" from the message, and
  is wrong for flags (`Button(outline)` becomes `Button(.outlined)`).
- Nothing can be configured. There is no allow comment, no lint config and
  no `--deny-warnings`.

**The checks.** About 50 confirmed gaps.
- **Builds that ship a dead page or site:**
  - duplicate `state` or `const` names (a JavaScript `SyntaxError`);
  - `await` inside a `derived`;
  - a `let` reassigned in a store action;
  - `.pending` on an action that is not async;
  - an undeclared component inside a slot fill;
  - a lambda-valued `derived`.
- **Crashes on use:**
  - assigning to a `const`, prop, loop variable or route parameter;
  - a missing required prop (a `layout:` argument too);
  - an unknown method on a number, string or list;
  - indexing an empty list (`todos[0].title`);
  - an `effect` that writes what it reads (stack overflow);
  - `if let` inside a store action.
- **Silent wrong output:**
  - `Input(...).number` and `Select` with numeric values store strings
    (`qty + 1` gives `"31"`);
  - nested and in-place mutation never repaints (`form.name = x`,
    `items[0].done = true`, `.sort()`);
  - a `Button` inside a `Form` submits it;
  - a non-exhaustive or duplicate-arm `match` (the first arm is dropped);
  - comparing a value with a case that does not exist (always false);
  - `"1" == 1` (always false);
  - links and `navigate()` calls to routes that do not exist;
  - `app` with no `Router` (no page ever renders);
  - unencoded URL splices;
  - an `api` path parameter that is never filled (`/users/:id?userId=42`);
  - `validate` on a state no control binds (the submit stays disabled
    forever);
  - `t("missing.key")`;
  - `[object Object]` printed as text;
  - a `persist` shape change that blanks the page for returning visitors;
  - typos in CSS properties and design tokens;
  - `@2026-02-30` accepted as a date.
- **Declared types that narrow nothing:**
  - a `derived` annotation is parsed and discarded;
  - a record's fields are never checked (unknown, missing or mistyped);
  - any case is assignable to any enum;
  - `Func` is assignable to `Func` regardless of arity;
  - an async action is typed as its bare result, not as something to await.

**Tools.**
- `wf serve` and `wf verify` answer a missing `/api/x.json` with the SPA
  shell and status 200, so a missing endpoint is never reported.
- `wf verify` skips `:param` routes.
- `wf test` cannot type into a `Select`.

---

## Part A — The foundations

Everything in Parts B–D lands on these. They come first.

### A1. One diagnostic

```rust
pub struct Diagnostic {
    pub code: Code,               // an enum: T05, R01, E104 … — data, not text
    pub severity: Severity,       // Error · Warning · Info (from the code, then config)
    pub file: String,
    pub span: Span,               // start AND end, bytes + line/col
    pub message: String,
    pub hint: Option<String>,
    pub related: Vec<Related>,    // "first declared here", "the route table", …
    pub fixes: Vec<Fix>,          // each a set of text edits, machine-applicable
    pub tags: Vec<Tag>,           // Unnecessary (U*), Deprecated
}
```

- **One type.** `WebFluentError`, `A11yWarning`, `VocabWarning`, the
  PDF/slides validation errors, script-scan problems and config errors all
  become this struct, so every diagnostic has a location.
- **One code registry** (`src/diagnostics/codes.rs`). Each code carries its
  family, default severity, title, docs anchor and an example. The docs
  test, the LSP's `codeDescription` link, `wf explain` and the JSON output
  all read it.
- **New code families** for what has none today:

  | Family | What | Examples |
  |---|---|---|
  | `E` | syntax and structure | E001 parse errors (numbered by kind), E101 unknown component, E102 duplicate declaration, E103 unknown flag/case/slot/part/event |
  | `T` | types, as today, extended | T14 disjoint comparison, T15 non-exhaustive match, T16 unknown method … |
  | `C` | components and props | C01 missing required prop, C02 unknown prop |
  | `R` | routes and navigation | R01 unknown route, R02 param mismatch, R03 no `Router` |
  | `F` | forms and bindings | F01 bind to something unwritable, F02 bind type mismatch, F03 `validate` on unbound state |
  | `X` | state and reactivity | X01 assign to read-only, X02 effect feeds itself, X03 in-place mutation |
  | `I` | i18n | I01 unknown key, I02 placeholder mismatch, I03 locale gap |
  | `D` | data, assets, persistence | D01 missing asset, D02 not serialisable, D03 shared persist key, D04 shape change |
  | `A` `S` `P` `U` `V` | as today, extended | V05 unknown CSS property, V06 unknown token, U06 unreachable code |

### A2. One pipeline, every finding

- **One entry point.** `check_project(&Project, &Config) -> Vec<Diagnostic>`
  replaces the four copies. The build, the LSP, `wf test`, the docs tests
  and the studio all call it, so the editor shows exactly what the build
  refuses — the config-aware checks included.
- **Every stage runs.** A stage's errors no longer stop the next. Each later
  stage runs on whatever parsed and resolved, and skips only what depends
  on a broken declaration.
- **Each file parses independently**, and one file's parse error does not
  hide another file's findings.
- **Parser recovery.** On an error, the parser resynchronises at the next
  top-level declaration and at the `}` that closes the current block, so
  one file reports several syntax errors, not just the first.
- **De-duplication.** Findings are de-duplicated by (code, file, span)
  centrally, and the triple inference behind repeated T05s is fixed at its
  source (`builtin_args`).

### A3. A renderer people can read

```
error[T05]: `User` has no field `nmae`
  --> src/pages/Profile.wf:5:18
   |
 5 |     Heading(user.nmae).h1
   |                  ^^^^ did you mean `name`?
   |
   = help: its fields are `id`, `name`, `email`
   = docs: https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t05
```

- The location comes first, so editors' and CI's problem matchers read it.
- A source line with an underline across the whole span.
- Colour when stderr is a terminal, honouring `NO_COLOR`.
- **One summary line:** `2 errors, 5 warnings` instead of every error
  printed twice. The `wf serve` overlay gets the structured list, not a
  count.
- **Exit codes:** `1` for diagnostics, `2` for I/O or config failures.

### A4. Machine-readable, and a fast check

- **`wf check`:** every diagnostic with no output written. It is fast enough
  for a pre-commit hook.
- **`wf build --format`** (and `wf check --format`):
  - `human`, the default;
  - `json`, the diagnostic struct;
  - `sarif`, for GitHub code scanning;
  - `github`, `::error file=…` annotations on a PR.
- **`wf explain T05`:** the code's page from the registry, in the terminal.
- `/__wf/status` carries the JSON list.

### A5. The editor gets all of it

- **Every diagnostic in the LSP has:**
  - its `code` and a `codeDescription` link;
  - a range from the real span;
  - `relatedInformation`, e.g. both declarations of a duplicate;
  - `Unnecessary` tags on U codes.
- **Quick fixes from `Diagnostic.fixes`,** never scraped from text. First:
  - V01: append the flag after the parentheses (and fix today's broken
    rewrite);
  - T13, T05, T06, E101, E103: the nearest name;
  - A01: add `alt: ""`;
  - U01–U05: prefix with `_`, or remove;
  - T04: add `?.` or `??`;
  - `onclick:` to `on click { }`;
  - C01: add the missing prop with a placeholder;
  - T15: add the missing arms.

### A6. Configuration

```json
{ "lints": { "A11": "off", "U": "error", "V02": "warn" } }
```

- **Per project:** a `lints` map in `webfluent.app.json`, keyed by code or
  family. Errors that would ship a broken page cannot be lowered: a `T`
  error stays an error.
- **Per line:** `// wf-allow(U01)` on the line above. An allow that no
  longer matches anything is itself a warning (U07), so allows cannot rot.
- **CLI:** `--deny-warnings` for CI.
- **Wired in:** `build.elements` and similar special cases go through this
  instead of today's substring match on message text.

### A7. Docs from the registry

- The diagnostics chapter is generated from the code registry.
- The docs test compares each example's **actual rendered output** with
  what the chapter shows, not just the code.
- Hints that still name pre-lowering, WebFluent 2 forms (A10's
  `Thead { Tcell }`) are corrected.

---

## Part B — Bugs to fix now

These are not missing checks — the compiler generates broken JavaScript.
They ship as a patch release ahead of everything else.

| # | Bug | Fix |
|---|---|---|
| B1 | A `let` in a store action is emitted `const`, so reassigning it is a SyntaxError (blank site) or a TDZ error | emit `let`, track store locals |
| B2 | `if let` inside a store action ignores the binding (ReferenceError) | reuse the page emitter's lowering |
| B3 | `Input(...).number` and a `Select` with numeric values store strings | coerce by the bound state's type (`valueAsNumber`; the option's typed value) |
| B4 | A `Button` inside a `Form` with no `type:` submits it | default `type="button"`; `type: .submit` stays |
| B5 | A lambda-valued `derived` is called as `scale(3)` | emit `_scale()(3)` |
| B6 | Storing a lambda in state calls it (`set(fn)` treats a function as an updater) | `set(() => fn)` for a Func-typed value |
| B7 | `bind:` to a loop item's field (`Input(bind: t.title)`) compiles one-way | lower to a keyed update of the source list, or C-error if the loop has no key |
| B8 | Duplicate `state`, `const`, store or action names ship a SyntaxError or silently replace one another | E102, with the first as related |
| B9 | **Safety net:** the build never parses its own JavaScript | parse every emitted `.js` (the minifier's tokenizer is already a JS lexer); a syntax error is an internal error naming the source construct |

---

## Part C — New checks

Each check lists what it detects, the code and severity, and the information
it needs (all of it already available unless noted).

### C1. Writability (X01, error)

Assigning to a `const`, `derived`, component prop, route parameter, loop
variable or `data` constant. Also `bind:` to any of these, and to a store's
`derived`. The checker already knows every name's kind.

```wf-next
const LIMIT = 3
page P(path: "/") { Button("x") { on click { LIMIT = 4 } } }   // X01: `LIMIT` is a const
```

### C2. Types, tightened (T)

- **T01** for every one of these:
  - a `derived` annotation, which the parser stops discarding;
  - record-constructor fields: wrong type, unknown field (T05), missing
    required field (C01);
  - a map literal given as a record: field check, not blanket acceptance;
  - **case to enum:** only a case the enum has;
  - **`Func` arity and parameter types**, when both sides are known.
- **T16, unknown method.** A method a `Number`, `String` or `List` does not
  have, from a method table of the JavaScript and runtime helpers. It
  replaces today's fall-through to `Any`.
- **T14, disjoint comparison.** `==` or `!=` between types that can never be
  equal (`"1" == 1`, `count == "0"`), and comparison with a case the enum
  does not have.
- **T17, async results.**
  - An async action's result is `Promise<T>`; using it as `T` without
    `await` is an error.
  - `await` outside an action or handler, `await` in a `derived` (with a
    pointer to `resource`), and `.pending` on an action that is not async
    are errors too.
- **T18, operators.** Arithmetic on non-numbers (`"a" - 1`, `list * 2`).
  `+` with a string stays concatenation.
- **T13, handler-only names.** `e`, `value` and `key` are known names only
  inside a handler.
- **Refinements on assignment.** A refined type (`Number(1..=30)`) is
  checked on every assignment of a literal, not only at its declaration.

### C3. Null safety (T04, extended)

- **Indexing yields `T?`.** `todos[0].title` needs `?.` unless a guard
  proves it non-empty. `first()`, `find()` and `match(…)[i]` are already
  optional.
- **`?.` chains** keep each field's own optionality: `sel?.note.length`
  with `note: String?` is T04.
- **Narrowing** in `else` branches and after an early `return`/guard,
  which removes today's false positives
  (`if sel == null { … } else { sel.title }`).
- **Nullable splice (warning T19).** A `T?` spliced into text without `??`,
  because it shows "null" or "undefined".

### C4. `match` (T15)

- **Exhaustiveness.** An enum `match` statement or expression that misses a
  case and has no `else` is an error naming the missing cases. A `match`
  expression that covers every case **no longer needs** `else`.
- **Duplicate arm** (error): today the first is silently dropped.
- **Unreachable `else`** (warning): every case is already covered.
- **Resource `match`:** a duplicate state arm is an error; a missing
  `error` arm is a warning.

### C5. Components (C)

- **C01 (error):** a required prop or positional not passed — on calls,
  `layout:` arguments and record constructors.
- **C02:** an unknown prop on a **user** component is an error
  (**[decide]**; today it is a warning, and the prop is passed anyway).
- **E101 in every position:** an unknown component inside a slot fill or a
  handler (the semantic walk skips both today).
- **C03:** a part used outside its owner (`Select.Option` outside a
  `Select`).

### C6. Routes and navigation (R)

- **R01 (error):** a literal `to:`, `navigate()` or `Link` path that
  matches no route in the table, including a missing `:param` segment and a
  path with no leading slash. Spliced paths are checked by their static
  shape (`"/team/{x}"` must match `/team/:slug`).
- **R02 (error):** a route's `:param` with no page parameter of that name,
  or the reverse.
- **R03 (error):** pages declared but `app` has no `Router`, or has two.
  The same for a `layout:` page with no `app` at all.
- **S04, two pages on one path:** an error (**[decide]**; today a
  warning, and one of the pages can never be reached).
- **R04 (warning):** a relative `src:`/`href:` literal (`"images/x.png"`)
  on a page whose route is nested, since it resolves differently on each
  route.

### C7. State and reactivity (X)

- **X02 (error):** an `effect` whose body writes a signal it also reads,
  directly or through a store action it calls. Today it overflows the stack.
- **X03, in-place mutation of state:**
  - `form.name = x`, `items[0].done = true`, `x.sort()`, `x.splice()`,
    `x.reverse()`, and `x = x.sort()` (the same reference).
  - **[decide]:** the compiler *rewrites* these into immutable updates
    (D2), or rejects them with that rewrite as the fix.
- **X04 (error):** a `derived` that assigns.
- **X05 (error):** `use` of a store that does not exist.

### C8. Forms and bindings (F)

- **F01 (error):** `bind:` to something that is not writable state (C1),
  or to a literal.
- **F02 (error):** a bound state's type that the control cannot hold:
  `Input(bind: list)`, `.email` on a `Number`, `Checkbox` on a `String`
  (the last is already caught).
- **F03 (error):** `validate x` where no control in a `Form` binds `x`, or
  `x` is derived. Today the submit stays disabled forever with a misleading
  "never read".
- **F04 (warning):** `Checkbox(checked: x)` or `Switch(on: x)` with no
  `bind:`, so the box toggles and the state does not.
- **F05 (warning):** a `Select` whose initial bound value is none of its
  option values.

### C9. Secrets (T12, extended)

- A `Secret` escaping through `+` concatenation (`"/a?t=" + token`).
- A `Secret` passed to any browser global (`console.log`,
  `localStorage.setItem`, `fetch` URLs).
- `Secret` loses `.length` and the other string members.

### C10. Data, assets and persistence (D)

- **D01 (warning):** a literal `src:`, `poster:`, `captions:` or
  `transcript:` naming a site-relative file that is not in `public/`.
- **D02 (error):** `persist` of a type that is not JSON-serialisable
  (`File`, a function).
- **D03 (warning):** `persist` inside a component that is placed more than
  once or inside a `for`, since every instance shares one key. It comes with
  a new `key:` option on `persist`.
- **D04 (warning):** a persisted value's type changed since the last build
  without a `version:` bump. Each key's type fingerprint is stored in
  `.wf-cache`. Today returning visitors get a blank page.
- **`api` paths (T10):** every `:name` in an endpoint's `at` path must be a
  parameter of that endpoint, and the reverse
  (`get user(userId) at "users/:id"` today requests
  `/users/:id?userId=42`).

### C11. i18n (I)

- **I01 (error):** a literal `t("key")` that no locale has. Today the
  raw key is shown.
- **I02 (error):** a parameter the message does not use, or a placeholder
  the call does not pass.
- **I03 (warning):** a key present in one locale and missing in another.
- **I04 (error):** a `setLocale("xx")` literal that is not in `locales`.

### C12. Literals, styles and text (V, T)

- **Calendar validity (T01):** `@2026-02-30` and `"2026-13-01"` as a
  `Date`. Money precision (`€12.999`). `on key("…")` combinations are
  parsed at compile time.
- **V05 (warning):** a style property no CSS defines (`colr:`), with the
  nearest suggested.
- **V06 (error):** a `$token` the resolved theme does not declare. The
  theme is already resolved for A13.
- **V07 (warning):** a bare number spliced as a whole length
  (`width: {pct}`), which the browser drops.
- **T20 (warning):** a map or list spliced into text (`[object Object]`);
  `Text(list)` likewise.
- **A16 (warning):** a literal `id:` inside a `for` or in a component placed
  more than once, which duplicates DOM ids.

### C13. Dead and suspicious code (U, warnings)

- **U06:** code after `return`.
- **U08:** a constant condition (`if false`, `if 1`) and a
  self-comparison (`x == x`).
- **U09:** a `for` with no `by` whose body holds state, an input, or a
  stateful component. Items lose their state when the list changes.

---

## Part D — What the compiler handles instead of reporting

Where there is one right answer, the compiler generates it.

- **D1. Typed binding.** `bind:` reads the bound state's type: a `Number`
  is set from `valueAsNumber` (NaN becomes `null`), and a `Select` writes
  back the option's typed `value:` (B3).
- **D2. Immutable updates** (**[decide]**, see C7):
  - `form.name = v` becomes `_form.set({ ..._form(), name: v })`;
  - `items[i].done = v` becomes `items.with(i, { ...items[i], done: v })`;
  - `sort`, `reverse` and `splice` on state become `toSorted`,
    `toReversed` and `toSpliced` followed by `set`.
  
  `persist` then saves these changes too; today it misses them.
- **D3. Buttons in forms** are `type="button"` unless they say `.submit`
  (B4).
- **D4. URL encoding.** A splice inside a path segment or query value of
  `to:`, `navigate()`, `fetch()` or an `api` URL is wrapped in
  `encodeURIComponent`. `"/team/{name}"` with `R&D/Ops` works.
- **D5. Exhaustive `match`** without `else`. When the compiler proves every
  case is covered, the `else` is unnecessary (C4).
- **D6. Bind on a loop field** becomes a keyed update of the source list
  (B7).
- **D7. The output self-check** (B9).

---

## Part E — The dev server, `wf verify` and `wf test`

- **E1.** `wf serve` and `wf verify` answer a request that is not a page
  navigation — a path with a file extension, or `Sec-Fetch-Mode` other than
  `navigate` — with a real 404, and report "a fetch was answered by the HTML
  shell". Today a missing API is invisible.
- **E2.** `wf verify` visits `:param` routes, through `paths:` or a
  placeholder. It flags a page whose `<main>` stays empty while pages exist.
  A `--returning-visitor` pass seeds storage with the previous build's
  persisted values.
- **E3, dev mode only:**
  - a response is checked against its declared type (`resource x: [User]`,
    `api … -> User`) at the `net` boundary; a mismatch takes the `error`
    arm with a `.parse` error naming the path (`$[1].name missing`);
  - a route that matches nothing renders a 404 and warns;
  - an `effect` that re-enters itself warns instead of overflowing.
- **E4.** One bad item in a `for` renders as an error placeholder (in dev)
  instead of blanking the whole list.
- **E5.** An `api` call with an empty path parameter stays `loading`
  instead of requesting the parent collection.
- **E6.** `wf test`:
  - `type … into` works on a `Select`;
  - `press "Escape"` closes a native `<dialog>`.

---

## Order

1. **B: the codegen bugs** (B1–B9), as a patch release. They are bugs
   today, independent of the rest.
2. **A1–A3: one diagnostic, one pipeline, the renderer.** Every check after
   this lands on them.
3. **C1–C4: writability, types, null safety, `match`.** Mostly in
   `sema/types.rs`, and the most crashes per line of work.
4. **C5–C8 and D1–D6: components, routes, reactivity, forms,** and the
   handling that pairs with them.
5. **C9–C13: secrets, data, i18n, literals, dead code.**
6. **A4–A7: JSON/SARIF, `wf check`, `wf explain`, the LSP, configuration,**
   generated docs.
7. **E: dev server, verify, test.**

Each phase ends with every new code documented (the docs test enforces it),
a test per code, and the guide's examples still building.

## What breaks

New errors stop builds that pass today; that is the point, but it is a
break. Each new error ships with its fix where there is one, `wf migrate`
applies the mechanical ones (missing `type:`, `_` prefixes, the V01
rewrite), and `lints` can lower anything below the "ships a broken page"
line while a project catches up.

## Decisions

Taken 2026-10-03.

| | Decided |
|---|---|
| Version | **5.0** — the new errors break builds that pass today. Part B ships first as a 4.3.x patch |
| In-place mutation (C7/D2) | **Rewritten** into immutable updates by the compiler |
| Unknown prop on a user component (C02) | **Error** |
| Two pages on one path (S04) | **Error** |
| Unkeyed `for` (U09) | **Warn** when the body is stateful; no automatic keys |
| Code scheme | Families E, T, C, R, F, X, I, D beside A, S, P, U, V |
| Lowering errors | `lints` can silence warnings and the A/S/P/U/V families, never an error that ships a broken page |

## Progress log

| Phase | State |
|---|---|
| 1 Codegen bugs (B) | released in 4.3.3. B1–B9 in `tests/codegen_fixes.rs`. B3 is `WF.bound(target)` (a number field's number, a select's option value as written, which `<option>` keeps as `_wfValue`). B6 is in the runtime: `signal.set(fn)` stores `fn`; `.update(fn)` is the updater. B7 went through one `bind_target` that every control uses, so a store's member now binds on `Switch`, `Checkbox`, `Radio`, `Slider` and `DatePicker` too (a `Switch` bound to one drew no input); an unkeyed loop writes the item on `input` and tells the list on `change`, so the row is not redrawn under the cursor. B8 is a semantic error for now (no code until A1), over three top-level namespaces — runtime values (store, const, data, image, api), types (type, enum), components — since a component and a `type` of one name are legitimate (Halyard has two) and a call is read by position. B9 is `codegen::jscheck`: closed strings, templates, regexes and comments, matched brackets, no name declared twice in one block — before and after minifying. Also: `wf test`'s `type … into` a `<select>` threw, and now picks the option by its text (E). Left: a lambda prop in the template engine cannot be applied |
| 2 Diagnostic, pipeline, renderer (A1–A3) | done. `src/diagnostics/`: one `Diagnostic` (code, severity, start and end, hint, related, fixes, tags; `Serialize`), the registry `codes.rs` (every code's severity, title and summary; a test holds the guide to it and the source's code literals to it), `check.rs` (`check_project`: scripts, config, `env`, structure, registry, types, lints, output mode — every stage runs; `incomplete` drops what a file that did not parse could answer), `render.rs` (rustc-style, source line and underline, colour only on a terminal and never with `NO_COLOR`). The build, the LSP (codes, `codeDescription` links, full ranges, related places, `Unnecessary` tags), the template engine and the studio all call it; `wf serve`'s overlay and `/__wf/status` carry the structured list. Each file parses on its own and the parser resynchronises at the next declaration (`parse_source_recovering`). New codes for what had none: E001–E005, E101–E116, E901–E902, C02, C04–C06, R01, D05, V08, V09. PDF and slides errors have places. `WebFluentError::Diagnostics`, exit `1` for findings and `2` for a build that could not run; lexer and parser errors box their diagnostic so a `Result` stays small. The guide's diagnostics chapter shows each example's real rendered output (`WF_WRITE_DIAGNOSTICS=1` rewrites it). The type checker inferred each built-in argument three times; once now. Tests: `tests/diagnostics.rs`, `diagnostics::*` unit tests, the docs tests |
| 3 Writability, types, null, match (C1–C4) | done. X01 through a record of what each binding is (a parallel `fixed` per scope, kept through null narrowing) plus the program's constants and each store's derived values and actions. `derived` keeps its annotation (`DerivedDecl.ty`). Records: field types (T01), unknown fields (T05), missing required fields (C01), for `Todo(…)` and for a map written where a record is wanted. A bare case is held to the enum it is given for (T02); a function takes no more arguments than it is handed (`Func` assignability, and T10 for a list method's lambda — JavaScript hands an item, its index and the list, and `reduce` its accumulator too). T16 from method tables of lists, strings, numbers and booleans. T14 for disjoint families and an enum against a missing case. T18 for `- * / %` (and `+` between non-strings) on non-numbers. T17: an async action's type is `Promise<T>` (`Type::Promise`), `await` is allowed only in actions, handlers, timers, hooks and `async` rules — not in a `derived` value or an effect, which compile non-async — and `.pending` on an action that awaits nothing. `event`, `e`, `value`, `key` known only inside a handler. A refined state is checked at every literal assignment. T04: a literal index of a list is `T?` unless an enclosing `xs.length > 0`-shaped condition says otherwise; a `?.` chain keeps each field's own null; `else` and an early `return` narrow. T19 warns on a nullable splice. T15: a `match` statement or expression (read back from its lowering, `match_arm`) with no `else` that misses a case, a case twice, or no `else` on a value of unknown type; a match expression with no `else` lowers to `__exhaustive`, which every backend compiles to `null` (`Type::Never`). U10 for an unreachable `else`, T21 for a resource match with no `error` arm. Real findings in Halyard: an undeclared `key` in `deploys.wf` (a ReferenceError at run time) and unguarded `[0]` reads. The `wf init` SPA template and the migration corpus guarded. Tests: `tests/checks.rs` |
| 4 Components, routes, reactivity, forms (C5–C8, D) | not started |
| 5 Secrets, data, i18n, literals, dead code (C9–C13) | not started |
| 6 JSON, check, explain, LSP, config, docs (A4–A7) | not started |
| 7 Dev server, verify, test (E) | not started |
