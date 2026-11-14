# Grammar

> The WebFluent grammar on one page: keywords, declarations, render and imperative blocks, components, tests, styles, operators, precedence and literals.

The page: https://webfluent.monzeromer.dev/reference/grammar

This page is a map of what the parser accepts: each keyword, each
statement, and where it may be written. The other chapters say what each
form means; this page says where it goes.

How to read the shapes on this page:

- A `text` block shows the shape of a form, not a program to copy.
- `…` stands for more of the same, and `|` separates alternatives.
- A word in lower case that is not a keyword, such as `expr`, `Type` or
  `name`, stands for something you write there.
- *Render block* means a block that holds elements and the statements that
  declare state: a page, a component, the `app`, a test, and every
  element's children. *Imperative block* means a block of statements that
  run: an action, a handler, an effect, a timer or a hook.

## Names

The case of a name says what it is:

| Kind of name | Case | Examples |
|---|---|---|
| Declarations and statements | lower case | `page`, `state`, `for` |
| Elements, and the names declarations introduce | capitalised | `Button`, `UserCard`, `Todo` |
| Everything else | camelCase | `draft`, `isOpen`, `saveAll` |

A keyword has its meaning only where a statement starts. Everywhere else
it is an ordinary word, so every keyword works as a map key
(`{ action: "approve", type: "a" }`), a field name and an enum case.

A name the language already gives a meaning cannot be a program's own: the
words a statement is written with (`state`, `action`, `let`, `if`, `show`,
`match`, `style`, `on`, and the rest), the built-in functions, and the
values `env`, `params`, `locale`, `dir`, `now`, `viewport`, `query`,
`hash`, `theme`, `network` and `update`. Declared as a `state`, a prop, a
parameter, a loop item, a lambda's parameter or a `const`, one is an
`E118`. The names a handler brings (`event`, `e`, `value`, `key`) and the
browser's globals are yours to use. A program's own names may be written in
any script, Arabic included.

## Keywords

Each keyword, and where it starts a statement:

| Keyword | Where | Starts |
|---|---|---|
| `page`, `component`, `store`, `app`, `theme`, `type`, `enum`, `api`, `const`, `data`, `image`, `animation`, `test` | top of a file | a declaration |
| `state` | render block, store | a value that can change |
| `persist` | page, component, store | a value kept across visits |
| `derived` | render block, store | a value worked out from others |
| `effect`, with `cleanup` | render block, store | code that runs when what it reads changes, and its clean-up |
| `action` | render block, store | a named imperative block |
| `use` | render block | access to a store |
| `resource` | render block, store | a request whose state the page shows |
| `validate` | render block | the rules a state must pass |
| `socket`, `stream`, `channel`, `peer`, `worker` | render block, store | a connection the scope holds open |
| `every`, `after` | render block | a repeating or a one-off timer |
| `on key("…")`, `on resize`, and the other page events | top of a page, a component or `app` | a listener on the document or the window |
| `head` | page | tags for the document's head |
| `if`, `else`, `if let` | render or imperative block | a branch |
| `for`, with `in` and `by` | render or imperative block | a loop |
| `show` | render block | a block hidden, not removed, when its condition fails |
| `match` | render block, or a value | one arm, by a resource's state or an enum's case |
| `sequence`, `step` | render block | a timed group of enter animations |
| `slot`, `event`, `part` | component, before its elements | a slot, an event it fires, a part |
| `children` | component | where the caller's block goes |
| `style`, `transition`, `on` | element block | styles, transitions, handlers |
| `let`, `return` | imperative block | a local, and leaving with a value |
| `try`, `catch` | imperative block | error handling |
| `await` | imperative block, or a value | waiting for a promise |
| `emit` | imperative block of a component | firing one of its events |
| `expect`, `click`, `type`, `press` | test | a step a test checks or performs |

## Declarations

These forms are written at the top of a file, and nowhere else:

```text
page Name(path: "/", title: …, description: …, image: …, image_alt: …, type: …,
          noindex: …, lang: "…", translates: Page, layout: Component(…),
          guard: expr, redirect: "/…", paths: expr, param: Type, …) { render block }
component Name(_ positional: Type, prop: Type = default, …) { component body }
store Name(scope: .app | .session | .route, eager: Bool) { store body }
app { render block }
theme Name { token: value … }
type Name { field: Type = default, … }        type Name = Other { … }
enum Name { case, case(part: Type, …), … }
api Name(base: expr, …) { settings  on request|response|error(x) { … }  endpoints }
api Name from "spec.json" (base: expr)
const NAME: Type = expr
data name: Type = "file.json"
image name = "picture.jpg"
animation Name { from { css } 50% { css } to { css } }
test "name"(data: { … }) { render block and test steps }
```

A `///` comment before a declaration, a prop, a field, an event, an
endpoint, or a `state`, `persist`, `derived` or `action` of a page, a
component or a store is its documentation.

A store's body holds `state`, `persist`, `derived`, `effect`, `action`,
`resource` and the connections (`socket`, `stream`, `channel`, `peer`,
`worker`).

## Render blocks

A page, a component, the app, a test and every element's children hold
these statements:

| Statement | Form |
|---|---|
| state | `state name: Type = expr` |
| persisted state | `persist name: Type = expr { in: .local \| .session \| .name  key: expr  version: n  sync: Bool  migrate n -> n+1 { expr } }` — a page, a component or a store |
| derived | `derived name = expr` |
| effect | `effect { imperative  cleanup { imperative } }` |
| action | `action name(param: Type, …) { imperative }` |
| store | `use Store` |
| resource | `resource name: Type = fetch(url, options)` or `= Api.endpoint(args)` |
| validation | `validate name { rule(args) "message"  custom "message" { expr }  async "message" { expr } }` |
| connection | `socket name = ws(url, …) { send T  receive T  on message(m) { … }  on open { … } }`, `stream name = sse(…)`, `channel name = broadcast(…)`, `peer name = rtc(signal: …, …)`, `worker name = spawn(path, …) { on message(m) { … } }` |
| timer | `every(interval) { imperative }`, `after(interval) { imperative }` |
| key | `on key("ctrl+k", e) { imperative }` |
| page event | `on keydown(e) { … }`, `on resize { … }` — and `keyup`, `scroll`, `hashchange`, `popstate`, `online`, `offline`, `message`, `storage`, `visibilitychange`, `blur`, `focus` — at the top of a page, a component or `app` |
| head | `head { meta(…)  link(…)  script(…) }` — a page only |
| conditional | `if cond { … } else if cond { … } else { … }`, `if let x = expr { … }` |
| loop | `for item, index in list by key { … }` |
| visibility | `show cond { … }` |
| choice | `match expr { .case(a) { … }  loading { … }  error(e) { … }  ready(v) { … }  else { … } }` |
| orchestration | `sequence { step(after: "120ms") { … } }` |
| set-up | `Store.action(args)` — a call, run once when the block renders |
| element | `Name(positional, prop: value).flag.flag { block }` |
| slot use | `children`, `slotName`, `slotName(value: expr)` — a component only |

A connection's options may also be written inside its block, one
`name: value` to a line.

## Component bodies

A component's body is a render block, and before its elements it may
declare:

```text
slot                                  the default slot, placed with `children`
slot name                             a named slot
slot name(value: Type, …)             a scoped slot
event name(param: Type, …)            an event it fires with `emit`
part Name(props) { render block }     a part, called as `Component.Name`
```

## Element blocks

An element's block holds these, in this order, each optional:

```text
Name(…).flags {
    style { declarations }
    transition { property: duration easing … }
    on event(e) { imperative }
    slotName(params) { render block }      a slot fill, on a component call
    children
}
```

## Imperative blocks

Actions, handlers, effects, timers and hooks hold these statements:

| Statement | Form |
|---|---|
| assignment | `name = expr`, `a.b = expr`, `list[i] = expr` |
| local | `let name: Type = expr`, `let { a, b } = expr`, `let [x, y] = expr` |
| call | `f(args)`, `Store.action(args)`, `list.push(x)`, `navigate("/…")`, `log(…)` |
| emit | `emit event(args)` — a component only |
| return | `return`, `return expr` |
| conditional | `if … else …`, `if let …` |
| loop | `for x in list { … }`, `for x, i in list { … }`, `for n in 1..=10 { … }` |
| error handling | `try { … } catch e { … }` |
| waiting | `await expr` — makes the action asynchronous |

`let`, `return` and `emit` written in a render block are an error that
says to move them into a handler, an action or an effect.

## Style blocks and themes

A `style` block holds CSS declarations, one to a line:

```text
style {
    property: raw css value               to the end of the line, a `;`, or two spaces and the next declaration
    property: {expr}px $token             a live value, a design token
    --custom-property: value
    &:hover { declarations }              nested rules, as CSS nesting writes them
    @media (…) { declarations }           and `@md { }` for a breakpoint, `@container (…) { }`
}
```

A theme's body is the same declarations, without nesting.

## Test steps

A `test` holds render statements and these steps, run in the order
written:

| Step | Form |
|---|---|
| check | `expect "text"`, `expect not "text"` |
| click | `click "name"` |
| type | `type "text" into "name"` |
| key | `press "Key"`, `press "Key" in "name"` |

## Expressions

The operators, from weakest to strongest binding:

| Level | Operators |
|---|---|
| 1 | `..` `..=` (ranges) |
| 2 | `??` |
| 3 | `\|\|` |
| 4 | `&&` |
| 5 | `==` `!=` (`!==` too) |
| 6 | `<` `<=` `>` `>=` |
| 7 | `+` `-` |
| 8 | `*` `/` `%` |
| 9 | unary `!` `-`, `await` |
| 10 | calls `f(…)`, fields `a.b`, `a?.b`, methods `a.m(…)`, `a?.m(…)`, indexes `a[i]`, `a?.[i]`, flags |

So `1..n + 1` is `1..(n + 1)`, and `a ?? b || c` is `a ?? (b || c)`.
Operators of one level group from the left: `a - b - c` is `(a - b) - c`.

These forms are expressions too:

- `if c { a } else { b }`, which is `null` when there is no `else` and `c`
  fails;
- `if let x = v { a } else { b }`;
- `match v { .case { a } else { b } }`;
- lambdas `x => expr`, `(a, b) => expr` and `() => expr`;
- spreads `...x` in a list, a map or a call's arguments.

A call's named arguments (`due.plus(days: 3)`) are gathered into one map,
passed last. [Expressions](https://webfluent.monzeromer.dev/reference/expressions.md) explains each form.

## Literals

| Kind | Written |
|---|---|
| String | `"text {splice} {value:.style(option)}"`, raw `#"…"#`, block `"""…"""` |
| Number | `42`, `3.14` |
| Bool, null | `true`, `false`, `null` |
| List, map | `[a, b]`, `{ key: value, "Quoted-Key": value, ...other }` |
| Regex | `/pattern/flags` |
| Date, time | `@2026-03-14`, `@09:30`, `@2026-03-14T09:30Z` |
| Duration | `250.ms`, `5.seconds`, `90.minutes`, `2.hours`, `3.days`, `1.weeks` |
| Money | `€12.99`, `$9.99`, `£4.50` |
| Colour | `#0F766E` |
| Enum case | `.name`, `.name(payload)` |
| Design token | `$name` |

## Comments

`// to the end of the line`, `/* across lines */`, and `///` documentation.

## Layout

A `.wf` file writes blocks in braces. A `.wfx` file writes them by
indentation: a line indented deeper than the one before opens a block, and
a dedent closes it. Braces written by hand still work inside a line
([Language basics](https://webfluent.monzeromer.dev/learn/04-language-basics.md#two-layouts-wf-and-wfx)).
Statements need no separator, on one line either: the parser finds where
one ends. Two on one line read best apart by a `;` or two spaces
(`count = 0; log(count)`).

## Next

[Browser support](https://webfluent.monzeromer.dev/reference/browser-support.md): which browsers a WebFluent site targets, and what each newer platform feature falls back to where a browser lacks it.
