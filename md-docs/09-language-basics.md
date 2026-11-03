# 9. Language basics

<!--
route: guide/language-basics
group: concepts
blurb: The shape of every .wf file: the thirteen kinds of declaration, naming, comments, the two kinds of body, the two layouts and how strings are written.
description: Declarations, naming, comments, render and imperative bodies, the braced and the indented layout, and how to write strings in WebFluent.
-->

A WebFluent program is a set of declarations: pages, components, stores and
the rest, each a keyword followed by a name and a body. Learn the thirteen
kinds and the two kinds of body, and you can read any `.wf` file.

In this chapter:

- the thirteen declarations, and the chapter that covers each
- how names are written
- comments, including the doc comments the editor shows
- render blocks and imperative blocks, and what goes in each
- the braced `.wf` layout and the indented `.wfx` layout
- the three ways to write a string, and how a string splices in values

## Declarations

A `.wf` file holds top-level declarations and nothing else. You cannot put an
element or a statement loose in a file; it belongs in the body of a
declaration. This file uses all thirteen kinds:

```wf
page Home(path: "/", title: "Home", description: "The front page.") { Heading("Welcome").h1 }
component Chip(_ label: String, tone: Tone = .calm) { Badge(label) }
store Cart { state items = []  derived count = items.length }
theme Brand { color-primary: #0F766E  radius-md: 14px }
app { Navbar { Text("Shop").heading }  Router }
type Todo { id: String, title: String, done: Bool = false }
enum Tone { calm, loud }
api Backend(base: "/api/v1") { get todos() -> [Todo] }
const API = "/api/v1"
data posts = "posts.json"
image hero = "hero.jpg"
animation Pulse { from { opacity: 1 } to { opacity: 0.5 } }
test "chip shows its label" { Chip("Beta")  expect "Beta" }
```

Each keyword declares one kind of thing:

| Declaration | What it is | Chapter |
|---|---|---|
| `page` | A route and what it shows | [6](10-pages-and-routing.md) |
| `component` | A reusable element with typed props, events, slots and parts | [11](15-components.md) |
| `store` | State shared across pages | [12](16-stores.md) |
| `theme` | Design tokens: colours, spacing, radii, fonts | [15](19-styling.md) |
| `app` | The shell every page renders inside, with the `Router` | [6](10-pages-and-routing.md#the-app-shell) |
| `type`, `enum` | Records and enumerations the type checker enforces | [13](17-types.md) |
| `api` | An HTTP service: its address, its endpoints and what they return | [17](24-data.md#describe-a-service-with-api) |
| `const` | A value read everywhere | [14](18-expressions.md#constants) |
| `data` | A JSON file read at build time, as a constant | [17](24-data.md#read-a-file-at-build-time-with-data) |
| `image` | A picture the build resizes and hands to `Image` | [28](21-media.md) |
| `animation` | Keyframes | [20](20-motion.md#declare-your-own-keyframes) |
| `test` | A render held to expectations and a snapshot | [24](29-testing.md) |

Two other kinds of file under `src/` join the program. A `.css` file is a
stylesheet the build bundles. A `.js` file is a plain browser script, and its
top-level functions are in scope by name in your `.wf` code
([JavaScript interop](39-javascript-interop.md)).

Inside a body you write statements: `state`, `derived`, `effect`, `action`,
`persist`, `resource`, `validate`, `socket`, `if`, `for`, `match` and more.
The [grammar](47-grammar.md) lists every keyword and where it may appear.

### One program from many files

All the files under `src/` merge into one program. A component declared in
`components/Chip.wf` is usable from every page without an import. The
compiler reads `App.wf` (or `App.wfx`) first, then the other files in name
order, so a build is the same on every machine.

## Naming

Names follow fixed conventions, so you can tell what a word is from how it
is written:

- Declaration keywords are lowercase: `page`, `component`, `store`.
- The names they declare, and every element, are **PascalCase**: `Home`,
  `UserCard`, `Button`, `Table.Row`.
- State, props, actions, variables and flags are **camelCase**: `count`,
  `isOpen`, `save`, `.primary`.
- Enum cases start with a dot: `.calm`, `.md`.
- Design tokens start with a dollar sign: `$primary`, `$spacing-md`.

The compiler resolves every name. A name it does not know — a misspelled
element, a flag the element does not take, a case the enum does not have — is
an error with a hint, never a silent no-op.

## Comments

WebFluent has three kinds of comment. A `//` comment runs to the end of the
line, a `/* … */` comment may span lines, and a `///` comment documents the
declaration that follows it:

```wf
// A line comment.
/* A block
   comment. */
/// A person's initials, in a circle.
component Avatar(_ initials: String) { Text(initials).bold }
```

A `///` comment above a declaration, a prop or an event is its documentation.
The editor shows it on hover, and `wf docs` prints it in the component
gallery. Above a `state`, `persist`, `derived` or `action`, it documents that
member, and the editor shows it wherever the member is used.

## Bodies

A body is one of two kinds of block, and each holds different statements:

- A **render block** is the body of a page, a component or `app`. It holds
  elements, and the statements that shape them: `state`, `derived`, `if`,
  `for` and the like.
- An **imperative block** is the body of a handler or an action. It holds
  statements that do something: assignments, calls, `let`, `if`, `for`,
  `try`.

The two blocks have separate grammars. In this page, the `Text` is an element
in the render block, and the assignment sits inside an `on click` handler:

```wf
page Counter(path: "/", title: "Counter", description: "A number that goes up.") {
    state clicks = 0
    Heading("Counter").h1
    Text("Clicked {clicks} times")
    Button("Add one") {
        on click { clicks = clicks + 1 }
    }
}
```

> [!WARNING]
> An assignment written loose in a page body, such as `clicks = clicks + 1`
> next to the `Text`, is error `E105`. Its hint says where the line belongs:
> *"Code that does something goes in `on click { … }`, an `action` or an
> `effect`"*.

### Separating statements

Statements need no separator. You usually put one per line. To put two on one
line, separate them with `;` or with nothing at all; the house style is two
spaces, because it reads well:

```wf
page Score(path: "/", title: "Score", description: "Two teams, two numbers.") {
    state home = 1  state away = 2
    Heading("Score").h1
    Text("{home} – {away}")
    Button("Both score") { on click { home = home + 1; away = away + 1 } }
}
```

## Two layouts: `.wf` and `.wfx`

You can write the same grammar with braces or with indentation. A `.wf` file
opens a block with `{` and closes it with `}`. A `.wfx` file opens a block
with a line indented deeper than the one before it, and closes it when the
indentation returns.

This page is written in the braced layout:

```wf
page Home(path: "/", title: "Home", description: "A panel that opens and closes.") {
    state open = true
    Heading("Home").h1
    Row(gap: .sm) {
        style {
            padding: 6px 0
            &:hover { background: $surface }
        }
        on click { open = !open }
        Text("Details").bold
    }
    if open {
        Spinner
    } else {
        Text("Closed")
    }
}
```

The same page in the indented layout:

```wfx
page Home(path: "/", title: "Home", description: "A panel that opens and closes.")
    state open = true
    Heading("Home").h1
    Row(gap: .sm)
        style
            padding: 6px 0
            &:hover
                background: $surface
        on click { open = !open }
        Text("Details").bold
    if open
        Spinner
    else
        Text("Closed")
```

The indented layout follows these rules:

- A line indented deeper than the line before it opens a block. Indent with
  the same spaces or tabs as the lines around it; four spaces per level is
  the house style, and what `wf fmt` writes.
- A line whose indentation lines up with no enclosing block is error `E003`.
  The first line of a file is not indented.
- You may still write braces yourself, and everything inside them is laid out
  freely: `on click { open = !open }` on one line, or a map literal or an
  argument list across several lines. Inside parentheses and brackets,
  layout is free too.
- An empty block is written `{ }`.
- Blank lines and comment lines do not count.

Both layouts compile to the same output and every tool reads both, so a
project may mix them. `wf fmt --to wfx` converts a project to the indented
layout and `wf fmt --to wf` converts it back, without changing what any file
says. This guide writes `.wf`; every example has a `.wfx` spelling.

## Strings

A string is written in double quotes and splices in a value with `{expr}`:

```wf
page Greeting(path: "/", title: "Greeting", description: "A greeting that knows your age.") {
    state user = { name: "Sam", age: 41 }
    Heading("Hello, {user.name}!").h1
    Text("You are {user.age * 12} months old.")
    Text("A literal brace: \{ and \}")
    Text("A literal dollar: ${user.age}")     // `$` alone is plain text: "$41"
}
```

Spliced text is live: when `user` changes, the text changes with it. A splice
opens with a name, a `(` or a `[`, and closes on the same line; anything
else, such as `{key: value}` in prose, is plain text.

A splice may also say how to show its value: `{total:.currency}`,
`{when:.date(long)}`. This is
[formatting](18-expressions.md#showing-a-value-a-particular-way), written
where the value is read.

### Three ways to write one

A plain `"…"` string reads the escapes `\n`, `\t`, `\r`, `\"`, `\\`, `\{` and
`\}`, and splices `{…}`. For text that a plain string makes you escape over
and over, there are two other forms:

```wf
// Raw: no escapes, no splices. The text is what is written.
const SAMPLE = #"page Home(path: "/") { state n = 0 }"#

// As many # as the text needs, when it holds a `"#` of its own.
const TRICKY = ##"a raw string ends with "# — like that"##

// Block: over as many lines as it likes, escapes and splices on, and
// the indentation the source gave it removed.
const name = "Ada"
const LETTER = """
    Dear {name},
      thank you.
    """                       // "Dear Ada,\n  thank you."
```

Use each form for a different kind of text:

- A **raw string** (`#"…"#`) suits a sample of code, a regular expression or
  a Windows path: anything full of quotes, braces or backslashes.
- A **block string** (`"""…"""`) suits a paragraph, a template or a query.
  The closing `"""` sets how far the text was indented, and the compiler
  removes that much from every line.

## Quick reference

| Syntax | Meaning |
|---|---|
| `page Name(path: "/…") { … }` | A declaration: keyword, name, arguments, body |
| `// …`, `/* … */` | Comments |
| `/// …` | Documentation for the declaration, prop, event or member below |
| `PascalCase`, `camelCase` | Declared names and elements; state, props, actions and flags |
| `.case`, `$token` | An enum case; a design token |
| `on click { … }`, `action save() { … }` | Imperative blocks, where assignments go |
| `a = 1; b = 2` or `a = 1  b = 2` | Two statements on one line |
| `.wf`, `.wfx` | Braced layout; indented layout ([grammar](47-grammar.md)) |
| `"Hi, {name}"` | A string with a live splice |
| `"{total:.currency}"` | A splice with a format |
| `#"…"#`, `"""…"""` | A raw string; a block string |

## Next

[Pages and routing](10-pages-and-routing.md).
