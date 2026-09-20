# WebFluent 3 — The Grammar

> Version 3.0
> Author: Monzer Omer
> Date: 2026-09-20

This is the design record and the reference for the grammar WebFluent 3
reads. It replaces the surface syntax specified in `SPEC.md` (WebFluent 2),
whose semantics — pages, components, stores, reactivity, styling, the
built-in library — carry over unchanged under the new spelling. `wf migrate`
converts a project from the old grammar to this one; `TYPES.md` specifies
the type checker that this grammar makes possible.

---

## 1. Why

The grammar of WebFluent 2 grew by accretion and ended up breaking its own
rules:

- A bare word in an argument list was told from an expression by a
  vocabulary lookup: `Button("x", huge)` did nothing, silently.
- One `{ }` block meant five things — children, `style`, `transition`,
  `on:click`, and a `Button`'s click shorthand.
- The first positional argument meant something different per component;
  named arguments no component declared fell through to HTML attributes.
- Style values were strings in which a bare word was either a design token
  or a state variable, depending on a lookup.
- Routes were declared twice: `Page X (path: …)` and `Route(path:, page:)`.
- Components of a project had no typed props, no variants, no events and no
  slots, so they could not be checked, completed or documented like the
  built-ins.
- The built-in vocabulary lived in twenty-four places.

WebFluent 3 has **one lexeme per meaning**. The built-ins are described once,
in a registry that the compiler, the linters, the language server and the
editors all read.

---

## 2. Lexical Structure

- Source is UTF-8. Statements are separated by newlines; `;` separates two
  on one line.
- Comments: `// …` to the end of the line, `/* … */`. `/// …` before a
  declaration, a prop, a field, an event or a slot is its documentation,
  shown on hover.
- Identifiers: a letter or `_`, then letters, digits, `_`. Declarations,
  components and parts are Capitalised (`UserCard`, `Table.Row`); state,
  props, actions and flags are lowercase (`draft`, `.primary`).
- There are **no reserved keyword tokens**. `page`, `state`, `on`, `type`
  and the rest are keywords by position, so `type: .submit` is a prop and
  `on = !on` is an assignment.
- Literals: `"string"` with `{expr}` interpolation and `\"`, `\\`, `\n`,
  `\{` escapes; numbers `42`, `3.14`; `true`, `false`, `null`.
- `$name` is a design token, greedy over hyphens: `$surface-hover`.
- `.name` before a case is an enum case; after `)` or an element's name it
  is a flag; after a value it is member access.
- Inside a `style { }` or a `theme { }` block, after `name:` the lexer is in
  **style-value mode**: the value is raw CSS text to the end of the line,
  to `;`, or to `}` at parenthesis depth zero. A line that ends with `,` or
  an open parenthesis continues. `//` starts a comment only after
  whitespace, so `url(http://…)` survives. `{expr}` is a splice, re-lexed as
  an expression. A line that starts with `&`, `@`, `.`, `:`, `[` or `>` is a
  nested rule's selector.

---

## 3. Files and Declarations

A file is a list of declarations, in any order, across any number of files
under `src/`:

```
Declaration = Page | Component | Store | Theme | App | Type | Enum ;
```

### 3.1 `page`

```wf
page Home(path: "/", title: "Home", description: "What it is", layout: AppShell(crumb: "Home")) {
    …
}
page Post(path: "/posts/:slug", slug: String) { … }
```

Header keys: `path` (required), `title`, `description`, `image`, `type`,
`redirect`, `noindex: true | false`, `guard: expr`, `layout: Component(args)`,
and route parameters as `name: Type`. A page **owns its route**; the `app`'s
`Router` takes no children. Routes are matched by specificity — static
segments before `:params`, `*` last — not by source order.

`layout:` names a component with a default `slot`; the page's body renders
where that component writes `children`.

### 3.2 `component`

```wf
/// A person, at a glance.
component UserCard(_ name: String, role: String, active: Bool = false) {
    event select(id: String)
    slot trailing
    slot
    …
}
```

Props are declared in the header: `_ name: Type` is the **one positional
prop** and must be first; the rest are named, with an optional default. A
`Bool` prop is set by a flag at the call site (`.active`); a prop whose type
is an `enum` is set by `prop: .case`, or by `.case` when only one prop has
that case.

The body starts with declarations — `event name(params)`, `slot` (the
default slot, filled by the caller's children) and `slot name` (a named
slot) — then `state`, `derived`, `action`, `effect`, `use`, `resource`, and
elements. The slot's content is placed by writing its name: `children` for
the default slot, `trailing` for a named one.

### 3.3 `store`

```wf
store Todos {
    state items: [Todo] = []
    derived remaining = items.filter(t => !t.done).length
    action add(title: String) { items = items.concat([Todo(id: uid(), title: title)]) }
    effect { localStorage.setItem("todos", JSON.stringify(items)) }
}
```

A page or component reaches a store with `use Todos`, then reads
`Todos.items` and calls `Todos.add(x)`. `Todos.load(id)` at the top of a
page's body is set-up code, run once when the page renders.

### 3.4 `theme`

```wf
theme Brand {
    color-primary: #8B5CF6
    surface-raised: #131519
    radius-md: 6px
}
```

A theme is a list of `token-name: value` with raw CSS values. Every token
becomes a custom property; one the baseline does not have is the project's
own, reachable as `$surface-raised`.

### 3.5 `app`

```wf
app {
    Navbar(brand: "Acme") { Navbar.Links { Link("Home", to: "/") } }
    Router(transition: .fade, duration: "200ms")
}
```

The application shell: what surrounds every page. `Router` is where the
current page renders.

### 3.6 `type` and `enum`

```wf
enum Tone { neutral, info, danger }
type Todo { id: String, title: String, done: Bool = false, tone: Tone = .neutral }
```

A `type` is a record; `Todo(id: "a", title: "b")` constructs one. An `enum`
is a closed set of cases written `.case`. See `TYPES.md`.

Types: `String`, `Number`, `Bool`, `Map`, `Any`, `[T]` (a list), `T?`
(optional), and a declared name.

---

## 4. Elements

```
Element = Name ("." Part)? ("(" Args ")")? ("." Flag)* Block? ;
Args    = (Positional ("," Named)* | Named ("," Named)*)? ;
Named   = name ":" Expression ;
Flag    = "." name ;
```

- **Name**: a built-in (`Button`), a part of one (`Table.Row`, `Card.Header`,
  `Select.Option`, `Tabs.Page`) or a component of the project (`UserCard`).
  A part is written directly after its owner's name.
- **Arguments**: at most one positional, first — the component's positional
  prop (`Button("Save")`, `Heading("Title")`, `Icon("home")`) — then named
  props. An enum prop takes a case: `Row(gap: .md, align: .center)`.
- **Flags** come tight after the parentheses or the name:
  `Button("Save").primary.lg`, `Spacer.sm`, `Input(bind: q).text`. A flag is
  a `Bool` prop or a case that exactly one enum prop of the component has;
  a case two props share is written by name (`tone: .info`). A flag takes
  no arguments. A word nothing declares is an error, with the flags the
  component takes.
- **Universal props** every element takes: the motion props (`animate:`,
  `exit:`, `speed`, `duration:`, `delay:`, `easing:`, `stagger:` — and an
  animation name as a flag, `.fadeIn`), `class:`, `id:`, `role:`,
  `aria-*:`, `data-*:`, and the HTML attributes of the element's tag. A named
  argument a built-in does not declare is a warning and is written to the
  element as an attribute.

### 4.1 The block

An element's block has a fixed order, parser-enforced:

```
Block = "{" Style? Transition? Handler* Fill* Child* "}" ;
```

1. `style { … }` — see §7.
2. `transition { property: duration easing … }`.
3. `on event(param?) { statements }` — handlers, see §5.
4. `name { children }` — fills of the component's named slots.
5. Children: elements and the statements of §6.

A lowercase word followed by `{` is a clause or a fill; a Capitalised word is
a child. Loose statements — a call, an assignment — inside a render block
are errors: what does something goes in a handler, an `action` or an
`effect`.

```wf
Button("Save").primary {
    style { min-width: 8rem }
    transition { background: 150ms ease-out }
    on click { save() }
}
Panel("Keys") {
    trailing { Badge("Beta") }
    Text("Rotate every 90 days.")
}
```

---

## 5. Handlers and Events

```
Handler = "on" name ("(" param ")")? "{" ImperativeStatement* "}" ;
```

`on click { … }` runs the block; `on click(e) { … }` binds the event. The
event names an element takes are the DOM's (`click`, `input`, `change`,
`submit`, `keydown`, `mouseenter`, …); on a component of the project they are
also the events it declares:

```wf
component TodoRow(_ label: String) {
    event toggle(id: String)
    Checkbox(label: label) { on change { emit toggle(label) } }
}
TodoRow("Milk") { on toggle(id) { Todos.toggle(id) } }
```

`emit name(args)` fires a declared event; its arguments are checked against
the declaration.

---

## 6. Statements

### 6.1 In render blocks (pages, components, stores, element children)

| Statement | Form |
|-----------|------|
| State | `state name = value`, `state name: Type = value` |
| Derived | `derived name = expression` |
| Action | `action name(param: Type, …) { … }` |
| Effect | `effect { … }` |
| Store | `use Name` |
| Resource | `resource name = fetch(url, options?)` |
| Set-up | `Store.load(id)` at the top level of a page or component |
| Conditional | `if cond { } else if cond { } else { }` |
| Unwrap | `if let x = expr { }` — `x` is the non-null value; `if let x { }` unwraps the name itself |
| Loop | `for item in list { }`, `for item, index in list { }`, `for item in list by item.id { }` |
| Show | `show cond { }` — stays in the DOM, toggles display |
| Match | `match value { arm { } … }` |
| Slot | `children`, or a named slot's name |

A `match` on a resource has the arms `loading { }`, `error(e) { }`,
`ready(v) { }` and `else { }`; a `match` on an enum has `.case { }` arms and
`else { }`.

```wf
resource posts = fetch("/api/posts")
match posts {
    loading { Spinner }
    error(e) { Alert(e.message).danger }
    ready(list) { for p in list by p.id { PostCard(p) } }
}
```

### 6.2 In imperative blocks (handlers, actions, effects)

| Statement | Form |
|-----------|------|
| Local | `let name = value` — a local signal |
| Assignment | `name = value`, `a.b = value`, `a[i] = value` |
| Call | `save()`, `Todos.add(x)`, `console.log(x)` |
| Navigate | `navigate("/path")` |
| Log | `log(value)` |
| Emit | `emit name(args)` |
| Return | `return`, `return value` |
| Conditional | `if … { } else { }` |

`await` may appear in any expression of an action or a handler and makes it
asynchronous: `let r = await fetch(url)`.

---

## 7. Style Values

```wf
Card {
    style {
        padding: 1rem 2rem
        background: $surface
        --accent: {todo.color}
        width: {pct}%
        content: "→"
        &:hover { background: $surface-hover }
        @media (max-width: 768px) { padding: 0.5rem }
    }
}
```

- A value is **CSS as written** — no quotes, any function, any unit, to the
  end of the line or `;`.
- `$token` is a design token of the theme, validated, compiled to
  `var(--token)`. A token of a group may be written short where the
  property implies the group: `padding: $md` for `$spacing-md`,
  `color: $primary` for `$color-primary`, when the theme declares no
  `$md`/`$primary` of its own.
- `{expr}` is a live value; a value that is one splice is the expression
  itself, a value with more text around it is an interpolation.
- A quoted string is CSS's own string (`content`, `font-family`).
- A nested rule keeps CSS nesting's spelling: `&:hover { }`, `&::after { }`,
  `& > .x { }`, `@media (…) { }`. One level deep.
- `--name: value` declares a custom property on the element.

---

## 8. Expressions

Precedence, lowest first: `??` · `||` · `&&` · `== !=` · `< > <= >=` ·
`+ -` · `* / %` · unary `! -` and `await` · postfix `.member`, `.method(args)`,
`[index]` · primary.

Primaries: literals; `null`; a name; `f(args)`; `Record(field: value, …)`;
`.case`; `$token`; `[a, b]`; `{ key: value }`; `(expr)`; `x => expr` and
`(a, b) => expr`; `if c { a } else { b }`; `match v { .case { a } else { b } }`.

`a ?? b` is `b` when `a` is null. `if` and `match` are values when written in
expression position. Strings interpolate with `{expr}` and stay live.

Browser globals compile as written: `window`, `document`, `localStorage`,
`JSON`, `Math`, `Date`, `fetch`, `setTimeout`, `console`, …

---

## 9. Diagnostics

The grammar makes these errors decidable, and they are errors, not
warnings: an unknown component, part, prop, flag, case, event, slot or
layout; an ambiguous flag; a positional argument that is not first; a
`match` arm the value cannot take; an `emit` of an undeclared event; a loose
statement in a render block; a clause out of order; a file in the grammar
of WebFluent 2 (with the pointer to `wf migrate`). The type checker's
findings (`T01`–`T11`) are listed in `TYPES.md`.

---

## 10. Grammar Summary (EBNF)

```ebnf
File          = Declaration* ;
Declaration   = Doc? ( Page | Component | Store | Theme | App | TypeDecl | EnumDecl ) ;
Doc           = ("///" text NEWLINE)+ ;

Page          = "page" NAME "(" PageAttr ("," PageAttr)* ")" RenderBlock ;
PageAttr      = ("path" | "title" | "description" | "image" | "type" | "redirect") ":" STRING
              | "noindex" ":" BOOL | "guard" ":" Expression
              | "layout" ":" NAME ("(" Args ")")? | NAME ":" Type ;
Component     = "component" NAME ("(" PropDecl ("," PropDecl)* ")")? "{" MemberDecl* RenderStmt* "}" ;
PropDecl      = Doc? "_"? NAME ":" Type ("=" Expression)? ;
MemberDecl    = Doc? ( "event" NAME "(" (NAME ":" Type ("," NAME ":" Type)*)? ")" | "slot" NAME? ) ;
Store         = "store" NAME RenderBlock ;
Theme         = "theme" NAME "{" (TOKEN_NAME ":" STYLE_VALUE)* "}" ;
App           = "app" RenderBlock ;
TypeDecl      = "type" NAME "{" (Doc? NAME ":" Type ("=" Expression)? ","?)* "}" ;
EnumDecl      = "enum" NAME "{" NAME ("," NAME)* ","? "}" ;
Type          = ("String" | "Number" | "Bool" | "Map" | "Any" | NAME | "[" Type "]") "?"? ;

RenderBlock   = "{" RenderStmt* "}" ;
RenderStmt    = Element | State | Derived | Action | Effect | Use | Resource | SetupCall
              | If | For | Show | Match | SlotUse ;
State         = "state" NAME (":" Type)? "=" Expression ;
Derived       = "derived" NAME "=" Expression ;
Action        = "action" NAME "(" (NAME ":" Type ("," NAME ":" Type)*)? ")" ImperativeBlock ;
Effect        = "effect" ImperativeBlock ;
Use           = "use" NAME ;
Resource      = "resource" NAME "=" "fetch" "(" Expression ("," Expression)? ")" ;
SetupCall     = (NAME ".")? NAME "(" (Expression ("," Expression)*)? ")" ;
If            = "if" ( "let" NAME ("=" Expression)? | Expression ) RenderBlock
                ("else" "if" Expression RenderBlock)* ("else" RenderBlock)? ;
For           = "for" NAME ("," NAME)? "in" Expression ("by" Expression)? RenderBlock ;
Show          = "show" Expression RenderBlock ;
Match         = "match" Expression "{" Arm+ "}" ;
Arm           = ( "loading" | "error" "(" NAME ")" | "ready" "(" NAME ")" | "." NAME | "else" ) RenderBlock ;
SlotUse       = "children" | NAME ;

Element       = NAME ("." NAME)? ("(" Args ")")? ("." NAME)* ElementBlock? ;
Args          = ( Expression ("," NamedArg)* | NamedArg ("," NamedArg)* )? ;
NamedArg      = ARG_NAME ":" Expression ;
ElementBlock  = "{" StyleBlock? TransitionBlock? Handler* Fill* RenderStmt* "}" ;
StyleBlock    = "style" "{" (StyleDecl | NestedRule)* "}" ;
StyleDecl     = PROPERTY ":" STYLE_VALUE (";" | NEWLINE) ;
NestedRule    = SELECTOR "{" StyleDecl* "}" ;
TransitionBlock = "transition" "{" (PROPERTY ":" DURATION EASING?)+ "}" ;
Handler       = "on" NAME ("(" NAME ")")? ImperativeBlock ;
Fill          = NAME RenderBlock ;

ImperativeBlock = "{" ImperativeStmt* "}" ;
ImperativeStmt  = "let" NAME (":" Type)? "=" Expression
                | Target "=" Expression | Expression
                | "navigate" "(" Expression ")" | "log" "(" Expression ")"
                | "emit" NAME ("(" (Expression ("," Expression)*)? ")")?
                | "return" Expression?
                | "if" Expression ImperativeBlock ("else" "if" Expression ImperativeBlock)* ("else" ImperativeBlock)? ;
Target        = NAME | Expression "." NAME | Expression "[" Expression "]" ;

Expression    = Coalesce ;
Coalesce      = Or ("??" Or)* ;
Or            = And ("||" And)* ;
And           = Equality ("&&" Equality)* ;
Equality      = Comparison (("==" | "!=") Comparison)* ;
Comparison    = Additive (("<" | ">" | "<=" | ">=") Additive)* ;
Additive      = Multiplicative (("+" | "-") Multiplicative)* ;
Multiplicative = Unary (("*" | "/" | "%") Unary)* ;
Unary         = ("!" | "-" | "await") Unary | Postfix ;
Postfix       = Primary ( "." NAME ("(" ExprList ")")? | "[" Expression "]" )* ;
Primary       = STRING | NUMBER | BOOL | "null" | "$" TOKEN | "." NAME
              | NAME | NAME "(" ExprList ")" | NAME "(" NamedArg ("," NamedArg)* ")"
              | "[" ExprList "]" | "{" (KEY ":" Expression ("," KEY ":" Expression)*)? "}"
              | "(" Expression ")" | NAME "=>" Expression | "(" NAME ("," NAME)* ")" "=>" Expression
              | "if" Expression "{" Expression "}" "else" ("{" Expression "}" | IfExpr)
              | "match" Expression "{" (("." NAME | "else") "{" Expression "}")+ "}" ;
ExprList      = (Expression ("," Expression)*)? ;
```

---

## 11. Rules the Parser Encodes

- `.word` after `)` or a name is a flag in a render block; in an imperative
  block, elements are errors and `.` is member access.
- `.Word` directly after a name is a part.
- `if let NAME` is followed by `=` (a binding) or `{` (the name itself).
- `by` is contextual, read only in a `for` head.
- `on`, `let`, `slot`, `event`, `emit`, `match`, `resource` and `children`
  are reserved in the position of a statement; anywhere else they are names.
- `.` IDENT in primary position is an enum case.
- A style value that is exactly one splice is the expression itself;
  `--accent: {expr}px` is an interpolation.
- `$primary - 1` needs the spaces: a token is greedy over hyphens.
- `Todo(id: …)` with a Capitalised name and named arguments is a record;
  `f(a, b)` is a call.

---

## 12. Migration from WebFluent 2

`wf migrate [path] [--check] [--stdout]` rewrites every `.wf` under `src/`
in place and reports what needed a decision. It is a change of spelling:
the migrated project builds to the same output. The rules, in outline:

| WebFluent 2 | WebFluent 3 |
|-------------|-------------|
| `Page X (path: "/")` · `Component X ()` · `Store`/`App`/`Theme` | `page X(path: "/")` · `component X` · lowercase keywords |
| `a?: String` · `List` | `a: String?` · `[Any]` |
| `Button("x", primary, large)` | `Button("x").primary.lg` |
| `Input(text, …)` · `Heading("x", h1)` | `Input(…).text` · `Heading("x").h1` |
| `gap: md` · `active: "prefix"` | `gap: .md` · `.prefix` |
| `Thead`/`Tbody`/`Trow`/`Tcell` · `Option("v", "L")` · `TabPage` | `Table.Head`/`Body`/`Row`/`Cell` · `Select.Option("L", value: "v")` · `Tabs.Page` |
| `on:click { }` · a Button block of actions | `on click { }` · `on click(event) { }` when the body reads it |
| `style { padding: "2rem" background: surface }` | `style { padding: 2rem; background: $surface }` |
| `hover { }` in a style block | `&:hover { }` |
| `if c, animate(fadeIn, fadeOut) {` | `if c {` with `exit: .fadeOut` and `.fadeIn` on the branch's roots |
| `fetch x from url { loading {} error(e) {} success {} }` | `resource x = fetch(url)` + `match x { loading {} error(e) {} ready(x) {} }` |
| `Router { Route(path:, page:) }` | `Router` — pages own their paths |
| `state x` inside an action | `let x` |
