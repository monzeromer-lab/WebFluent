# 43. Grammar

<!--
route: guide/grammar
group: reference
blurb: Every keyword and where it may appear, the shapes of declarations, statements and expressions, operators and precedence, and the literals.
description: The WebFluent grammar on one page: declarations, statements, element blocks, components, services, tests, styles, operators, literals.
-->

A compact map of what the parser accepts. The chapters say what each form
means; this page says where it may be written.

## Words

No word is reserved. `page`, `state`, `type` and the rest are keywords only
where a statement starts, so every one of them is also usable as a map key
(`{ action: "approve", type: "a" }`), a field name or a parameter.
Declarations are lowercase; elements and the names declarations introduce
are capitalised; everything else is camelCase.

## Keywords

Where a statement starts, these words have meaning: `page`, `component`,
`store`, `app`, `theme`, `type`, `enum`, `api`, `external`, `const`, `data`,
`image`, `animation` and `test` at the top of a file; `state`, `persist`,
`derived`, `effect`, `cleanup`, `action`, `use`, `resource`, `validate`,
`socket`, `stream`, `channel`, `peer`, `every`, `after`, `on key`, `head`,
`if`, `if let`, `for`, `show`, `match`, `sequence` and `step` in a render
block; `slot`, `event`, `part` and `children` in a component; `let`,
`return`, `try`, `catch`, `await` and `emit` in an action; `expect`, `click`,
`type`, `press` in a test. Everywhere else they are ordinary names.

## Declarations

At the top of a file, and nowhere else:

```text
page Name(path: "/", title: …, description: …, image: …, type: …, noindex: …,
          layout: Component(…), guard: expr, redirect: "/…", paths: expr,
          param: Type, …) { render block }
component Name(_ positional: Type, prop: Type = default, …) { component body }
store Name(scope: .app | .session | .route, eager: Bool) { store body }
app { render block }
theme Name { token: value … }
type Name { field: Type = default, … }        type Name = Other { … }
enum Name { case, case(part: Type, …), … }
api Name(base: expr, …) { settings  headers { … }  on request|response|error(x) { … }  endpoints }
api Name from "spec.json" (base: expr)
external Name from "module" { integrity: "…"  fn f(params) -> Type  type T { method(params) -> Type  field: Type } }
external element Name("tag-name") { prop name: Type  event name(params) }
const NAME: Type = expr
data name: Type = "file.json"
image name = "picture.jpg"
animation Name { from { css } 50% { css } to { css } }
test "name"(data: { … }) { render block and test steps }
```

A `///` comment before a declaration, a prop, a field, an event or an
endpoint is its documentation.

## Render blocks

A page, a component, the app, a test and every element's children:

| Statement | Form |
|---|---|
| state | `state name: Type = expr` |
| persisted state | `persist name: Type = expr { in: .local \| .session  version: n  sync: Bool  migrate n -> n+1 { expr } }` |
| derived | `derived name = expr` |
| effect | `effect { imperative  cleanup { imperative } }` |
| action | `action name(param: Type, …) { imperative }` |
| store | `use Store` |
| resource | `resource name: Type = fetch(url, options)` or `= Api.endpoint(args)` |
| validation | `validate name { rule(args) "message"  custom "message" { expr }  async "message" { expr } }` |
| connection | `socket name = ws(url, …) { send T  receive T  on message(m) { … }  on open { … } }`, `stream name = sse(…)`, `channel name = broadcast(…)`, `peer name = rtc(signal: …, …)` |
| timer | `every(interval) { imperative }`, `after(interval) { imperative }` |
| key | `on key("ctrl+k", e) { imperative }` |
| head | `head { meta(…)  link(…)  script(…) }` — a page only |
| conditional | `if cond { … } else if cond { … } else { … }`, `if let x = expr { … }` |
| loop | `for item, index in list by key { … }` |
| visibility | `show cond { … }` |
| choice | `match expr { .case(a) { … }  loading { … }  error(e) { … }  ready(v) { … }  else { … } }` |
| orchestration | `sequence { step(after: "120ms") { … } }` |
| set-up | `Store.action(args)` — a call, run once |
| element | `Name(positional, prop: value).flag.flag { block }` |
| slot use | `children`, `slotName`, `slotName(value: expr)` — a component only |

## Component bodies

A component's body is a render block, plus:

```text
slot                                  the default slot, placed with `children`
slot name                             a named slot
slot name(value: Type, …)             a scoped slot
event name(param: Type, …)            an event it fires with `emit`
part Name(props) { render block }     a part, called as `Component.Name`
```

## Element blocks

In this order, each optional:

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

Actions, handlers, effects, timers and hooks:

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

## Style blocks and themes

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

In a `test`, beside render statements: `expect "text"`, `expect not "text"`,
`click "name"`, `type "text" into "name"`, `press "Key"`,
`press "Key" in "name"`.

## Expressions

From weakest to strongest binding:

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
| 10 | calls `f(…)`, fields `a.b`, `a?.b`, indexes `a[i]`, `a?.[i]`, flags |

So `1..n + 1` is `1..(n + 1)`, and `a ?? b || c` is `a ?? (b || c)`.

Values that are expressions: `if c { a } else { b }`, `if let x = v { a } else { b }`,
`match v { .case { a } else { b } }`, lambdas `x => expr`, `(a, b) => expr`,
`() => expr`, spreads `...x` in a list or map.

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
indentation: a line indented deeper than the one before opens a block, a
dedent closes it, and braces written by hand still work inside a line
([chapter 5](05-language-basics.md#two-layouts-wf-and-wfx)). Statements need
no separator; on one line, a `;` or two spaces reads best.

## Next

[Troubleshooting](44-troubleshooting.md).
