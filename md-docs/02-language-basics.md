# 2. Language basics

## A file is a list of declarations

Every `.wf` file holds top-level declarations and nothing else — no loose
elements, no statements outside a body. There are eleven kinds:

```wf
page Home(path: "/", title: "Home", description: "The front page.") { Text("hi") }
component Chip(_ label: String, tone: Tone = .calm) { Badge(label) }
store Cart { state items = []  derived count = items.length }
theme Brand { color-primary: #0F766E  radius-md: 14px }
app { Navbar { Text("Shop").heading }  Router }
type Todo { id: String, title: String, done: Bool = false }
enum Tone { calm, loud }
const API = "/api/v1"
data posts = "posts.json"
animation Pulse { from { opacity: 1 } to { opacity: 0.5 } }
test "chip shows its label" { Chip("Beta")  expect "Beta" }
```

| Declaration | What it is | Chapter |
|---|---|---|
| `page` | A route and what it shows | 3 |
| `component` | A reusable element with typed props, events, slots and parts | 8 |
| `store` | State shared across pages | 9 |
| `theme` | Design tokens: colours, spacing, radii, fonts | 12 |
| `app` | The shell every page renders inside, with the `Router` | 3 |
| `type`, `enum` | Records and enumerations the checker enforces | 10 |
| `const`, `data` | A value read everywhere; a JSON file as one | 10 |
| `animation` | Keyframes | 13 |
| `test` | A render held to expectations and a snapshot | 18 |

All the files under `src/` merge into one program, so a component declared in
`components/Chip.wf` is usable from every page. `App.wf` is read first; the
rest in name order.

## Naming

- Declaration keywords are lowercase: `page`, `component`, `store`.
- The names they declare, and every element, are **PascalCase**: `Home`,
  `UserCard`, `Button`, `Table.Row`.
- State, props, actions, variables and flags are **camelCase**: `count`,
  `isOpen`, `save`, `.primary`.
- Enum cases are written with a leading dot: `.calm`, `.md`.
- Design tokens are written with a dollar: `$primary`, `$spacing-md`.
- A `///` comment above a declaration, a prop or an event is its
  documentation, shown on hover in the editor and in `wf docs`.

Nothing resolves silently. A name the compiler does not know — a misspelled
element, a flag the element lacks, a case the enum lacks — is an error with a
hint, never a no-op.

## Comments

```wf
// A line comment.
/* A block
   comment. */
/// A doc comment: attaches to the declaration that follows it.
component Avatar(_ initials: String) { Text(initials).bold }
```

## Bodies

A declaration's body is a **render block**: elements, and the statements that
shape them (`state`, `derived`, `if`, `for`, …). A handler or an action's body
is an **imperative block**: statements that do something (assignments, calls,
`let`, `if`, `for`, `try`). The two are separate grammars, and the compiler
tells you when a statement is in the wrong kind of block:

```wf
page P(path: "/") {
    state n = 0
    Text("{n}")                          // an element: fine in a render block
    Button("+") {
        on click { n = n + 1 }           // an assignment: fine in a handler
    }
}
```

Writing `n = n + 1` loose inside the page body is an error: *"Code that does
something goes in `on click { … }`, an `action` or an `effect`"*.

Statements on one line are separated by two spaces or a `;`:

```wf
page P(path: "/") {
    state a = 1  state b = 2
    Button("x") { on click { a = a + 1; b = b + 1 } }
}
```

## Two layouts: `.wf` and `.wfx`

The same grammar can be written with braces or with indentation. A `.wf` file
opens a block with `{` and closes it with `}`; a `.wfx` file opens a block with
a line indented deeper than the one before it and closes it with a dedent.

```wf
page Home(path: "/") {
    state open = true
    Row(gap: .sm) {
        style {
            padding: 6px 0
            &:hover { background: $surface-hover }
        }
        on click { open = !open }
        Text("a").bold
    }
    if open {
        Spinner
    } else {
        Text("b")
    }
}
```

The same in `.wfx`:

```wfx
page Home(path: "/")
    state open = true
    Row(gap: .sm)
        style
            padding: 6px 0
            &:hover
                background: $surface-hover
        on click { open = !open }
        Text("a").bold
    if open
        Spinner
    else
        Text("b")
```

Rules of the indented layout:

- Four spaces (or one tab) per level, consistently; a line that lines up with
  no enclosing block is an error.
- Braces you write yourself are still allowed, and everything inside them is
  free-form: `on click { open = !open }` on one line, a map literal across
  several, an argument list across several.
- An empty block is written `{ }`.
- Blank lines and comment lines count for nothing.

Both layouts compile to the same output, are read by every tool, and a project
may mix them. `wf fmt --to wfx` converts a project one way and `--to wf` the
other, losslessly. This guide writes `.wf`; every example has an `.wfx`
spelling.

## Strings

Strings are double-quoted and interpolate with `{expr}`:

```wf
page P(path: "/") {
    state user = { name: "Sam", age: 41 }
    Text("Hello, {user.name}! You are {user.age * 12} months old.")
    Text("A literal brace: \{ and \}")
    Text("A literal dollar: ${user.age}")     // `$` alone is plain text
}
```

Interpolated text is live: when `user` changes, the text changes.

## Next

[Pages and routing](03-pages-and-routing.md).
