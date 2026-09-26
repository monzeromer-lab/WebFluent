# WebFluent 3 — Types

> **Status: a design note, kept for its reasoning.** The current, tested description is the guide — [`md-docs/`](../md-docs/README.md), here 13-types.md. Where the two differ, the guide is right.

> Version 3.0
> Author: Monzer Omer
> Date: 2026-09-20

WebFluent 3 has a gradual, structural type checker. It runs in `wf build`
and in the language server, after semantic validation and before lowering;
its findings are errors. It changes no output: a program that declares no
types checks as it always did, because everything it cannot resolve is
`Any`, and `Any` agrees with everything. Every annotation narrows what the
checker can say, and every finding comes with a hint.

---

## 1. Types

| Type | Written | Values |
|------|---------|--------|
| `String` | `String` | `"text"`, `"with {interpolation}"` |
| `Number` | `Number` (`Int`, `Float` are the same) | `42`, `3.14` |
| `Bool` | `Bool` | `true`, `false` |
| `null` | — | `null` |
| `[T]` | `[String]`, `[Todo]` | `[a, b]` |
| `Map` | `Map` | `{ key: value }` — an object with unknown keys |
| `T?` | `String?`, `Todo?` | a `T` or `null` |
| a record | the `type`'s name | `Todo(id: "a", title: "b")` |
| an enum | the `enum`'s name | `.case` |
| a function | — | an `action`, a lambda, a store's action |
| a resource | — | `resource x = fetch(…)` |
| a store | — | `use Todos` |
| a token | — | `$surface` |
| a regex | — | `/…/flags` |
| a shape | — | `{ name: "" }` — a map literal, with the fields it was written with |
| `Any` | `Any` | anything; the type of what is not known |

A `Map` is assignable to any record and a record to `Map`: JSON that came
from `fetch` fits a `type` it was not declared as.

### Declaring

```wf
enum Tone { neutral, info, danger }

type Todo {
    id: String
    title: String
    done: Bool = false
    tone: Tone = .neutral
    note: String?
}
```

A field with a default may be left out of a construction; a field of type
`T?` defaults to `null`. Records are structural: `Todo(id: "a", title: "b")`
is a `Todo`, and so is `{ id: "a", title: "b" }` given where a `Todo` is
wanted — its fields are checked against the record's (`T01` on a wrong
one).

`type Admin = User { role: String }` extends a record: an `Admin` has every
field of `User` and then its own. `enum Status { idle, failed(reason:
String) }` gives a case a payload: `.failed("x")` is checked like a call
(`T10` on the count, `T01` on a part), `.failed` alone is `T02` where a
`Status` is wanted, and `match s { .failed(r) { } }` binds `r: String`
(`T10` when the names do not cover the payload).

A map literal is a **shape**: `{ name: "", age: 0 }` is `{ name: String,
age: Number }`, reading `form.nam` is `T05` with the fields it was written
with, and where a shape is wanted a literal is checked field by field. Two
shapes join to every field either has, a field only one has as `T?`, so a
`for` over `[{ id: 1 }, { id: 2, other: true }]` reads `r.other` as `Bool?`.
A shape fits a `Map`, a record, and any shape it agrees with on their
shared fields; `{}` and a literal with a spread are plain `Map`s. `const
NAME = value` is typed by its value or its annotation; `env` is a `Map`.

---

## 2. Where Types Come From

**Declared.** `state x: T = …`, a component's props (`_ name: String`,
`active: Bool = false`), an action's parameters, a `type`'s fields, an
`enum`'s cases, a page's route parameters (`slug: String`), an event's
parameters (`event toggle(id: String)`), `resource x: [Deploy] = fetch(…)`,
`let x: T = …`.

**Inferred from a value.** A literal; a list of literals (`[1, 2]` is
`[Number]`, `[]` is `[Any]`); a record construction; a `derived` from its
expression; a state from its initial value — a state that starts as `null`
is `Any?`, a value that may be null and will hold something later; a `for`
binding from the list's element type; a `match` arm's binding (`error(e)`
is `Any`, `ready(v)` is the resource's type); an `if let` binding from the
unwrapped value; a lambda's parameters from what the method expects
(`items.filter(t => …)` gives `t` the item type).

**From the registry.** The type of each built-in prop; the type a `bind:`
needs (`Checkbox` and `Switch` bind a `Bool`, `Slider` a `Number`, `Input`
a `String` or a `Number` by its `type`); the parameter of a DOM event
handler (`Any`).

**From the built-in method tables.** On a `String`: `toLowerCase`,
`toUpperCase`, `trim`, `replace`, `slice`, `substring`, `charAt`, `padStart`,
`padEnd`, `repeat` → `String`; `indexOf`, `length`, `charCodeAt` → `Number`;
`includes`, `startsWith`, `endsWith` → `Bool`; `split` → `[String]`. On a
`Number`: `toFixed`, `toString` → `String`. On a `[T]`: `map` → `[U]`
from the lambda; `filter`, `slice`, `concat`, `reverse`, `sort` → `[T]`;
`find` → `T?`; `some`, `every`, `includes` → `Bool`; `findIndex`, `indexOf`,
`push`, `sum` → `Number`; `join` → `String`; `reduce` → the initial value's
type. Anything else is `Any`.

**From stores.** `use Todos` binds `Todos` as a store; `Todos.items` has the
type of the store's state, `Todos.add` the signature of its action.

Browser globals (`window`, `document`, `localStorage`, `JSON`, `Math`,
`Date`, `fetch`, `console`, …) are `Any`.

---

## 3. Inference

Inference is bidirectional: an expression is checked against the type the
context wants when there is one, and its type is read off it otherwise.

- `.case` on its own is a case whose enum is not yet known; given where a
  `Tone` is wanted it becomes a `Tone`, and `T02` is reported if `Tone` has
  no such case.
- `a ?? b` is the type of `a` without its `null`, joined with `b`.
- `if c { a } else { b }` and `match v { … }` as values are the type both
  arms fit: the same type, `T?` when one arm is `null`, else `Any`.
- `await e` on a resource is its value; on anything else, the thing itself.
- `-`, `*`, `/`, `%` are `Number`; `+` is a `String` when either side is
  one, a `Number` when both are; comparisons and `&&`, `||`, `!` are `Bool`.
- `x.field` on a record is the field's type; on a `Map` or `Any`, `Any`.
- `f(args)` on a declared action checks the arity and each argument; the
  result is the action's return type.
- `a?.b` is `b`'s type made optional, and the rest of the chain with it;
  `if let x = v { a } else { b }` binds `x` to `v` without its null in `a`.
- `/…/` is a regex; `re.test(s)` is `Bool`, `re.exec(s)` and `s.match(re)`
  a `[String]?`, `s.replace(re, …)` a `String`, `s.split(re)` a `[String]`.
- `[...a, b]` is a list of the join of the parts; `{ ...m, k: v }` is a
  `Map`; `a..b` is `[Number]`.
- `let { a, b } = m` and `let [x, y] = l` bind the fields and items.
- `format(value, .style, option)` and `ago(date)` are `String`; a style
  the runtime lacks is `T02`, a style that is neither a case nor a pattern
  `T01`.
- A scoped slot's values are checked against its declaration (`T01`), and a
  fill's names take the declared types in order; a part is checked as the
  component it is.
- `ref: name` binds `name` as the element (`Any`); `Form(bind: f)` binds
  `f` as `{ valid: Bool, values: Map, reset: () → null, submit: () → null }`;
  `action.pending` is `Bool` and any other field of an action is `T05`.
- `every(ms)` and `after(ms)` take a `Number` (`T01`); `setTheme(x)` a
  `String` (`T01`, `T10`); `viewport` is `{ width: Number, height: Number,
  sm: Bool, md: Bool, lg: Bool, xl: Bool }`, `query` a `Map`, `hash` and
  `theme` `String`s.
- `list.sortBy(f)`, `unique()`, `take(n)`, `flatMap(f)` keep the item type;
  `groupBy(f)` is a `Map`; `first()`/`last()` are `T?`; `sum()` a `Number`;
  `capitalize()` and `truncate(n)` are `String`s.

### Narrowing

A value of type `T?` must be unwrapped before a field or method is read
off it (`T04`). Within its branch, each of these proves a name not null:

```wf
if let n = t.note { Text(n.length) }
if sel != null { Text(sel.title) }
if sel { Text(sel.title) }
if sel && t.done { Text(sel.title) }
derived label = if sel != null { sel.title } else { "none" }
Text(t.note ?? "none")
```

Narrowing is flow-sensitive within a block: it holds for the branch the
condition guards, not after it.

---

## 4. Checks

| Code | When | Example |
|------|------|---------|
| `T01` | A value of the wrong type given to a prop, a state, a field, a parameter, an assignment, an argument, or a `bind:` | `` `count` of `C` is `String`, but `Number` is wanted `` |
| `T02` | A case the enum does not have | `` `Tone` has no case `.angry` `` |
| `T04` | A field or method read off a value that may be `null` | `` `sel` may be null, so `.title` may fail `` |
| `T05` | A field or method a record does not have | `` `Todo` has no field `name` `` |
| `T06` | A member or action a store does not have | `` `S` has no action `reset` `` |
| `T07` | A list, a record or an action used as a condition (`if`, `show`, `disabled:`), which is always true | `` `if` reads `items` as a condition, but a list is always true `` |
| `T08` | A `for` over something that is not a list | `` `for` loops over a list, but `name` is `String` `` |
| `T09` | An `emit` whose arguments do not match the event's | `` `emit pick` passes 2 arguments, but the event takes 1 `` |
| `T10` | A call with the wrong number of arguments | `` `add` takes 2 arguments, but 1 is given `` |
| `T11` | A `match` on something that is neither a resource nor an enum, or an arm the value cannot take | `` `match` needs a resource or an enum, but `n` is `Number` `` |

Hints name the fix: a conversion (`Number(value)`, `"{value}"`), the cases
an enum takes, the fields a record has, the unwrapping forms of §3.

What the checker does not do: it does not check the body of a lambda
against a return type, does not track a name's type across assignments
(`state x = 1` stays a `Number`; assigning a string to it is `T01`), and
does not read the schema of a `fetch` — annotate the resource to type its
value.

---

## 5. In the Editor

The language server shows a name's inferred type on hover (`state draft:
String`, `derived remaining: Number`, `Todos.add: action(String)`)
and offers, after `.`, the fields of a record, the cases of an enum where
one is wanted, and the methods of a `String`, a `Number` or a list. A type
error is a diagnostic at the statement or argument it belongs to.

---

## 6. Implementation

`src/sema/types.rs`. `check(program, file_of) -> TypeInfo` walks every
declaration with a scope stack; `declare_hoisted` binds a body's state,
resources, derived values and actions before checking it (the compiler
resolves them regardless of order); `infer(expr, expected)` is the
bidirectional core; `TypeInfo::bindings` records the type and span of every
binding for the language server. Unit tests live beside it, one per rule and
per inference.
