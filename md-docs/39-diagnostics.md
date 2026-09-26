# 39. Diagnostics

<!--
route: guide/diagnostics
group: reference
blurb: Every error and warning the compiler reports — what it means, a program that draws it, the message, and the fix.
description: Every WebFluent error and warning — structural errors and codes T, A, S, P, U and V — each with a program that draws it and the fix.
-->

`wf build`, `wf serve` and your editor report the same findings. Each names
the file, the line and the column, says what is wrong in a sentence, and
usually offers a fix on the line under it:

```text
Error: [T04] `user.email` may be null, so `.toUpperCase()` may fail at src/pages/Profile.wf:12:10
  Unwrap it first: `if let x = value { … }`, `value ?? fallback`, …
```

- **Errors** stop the build: the program cannot mean what it says.
- **Warnings** print and the build goes on: it means something, probably
  not what you wanted. Nothing is a warning that the compiler could have
  decided for itself.

The code in brackets — `T04` — is the thing to search this page for. Every
example below is compiled by the test suite and draws exactly the code it
is filed under.

## Structural errors

Errors without a code, because each is a reference to something that does
not exist, or code where there must be none:

| You wrote | The compiler says | Fix |
|---|---|---|
| `Buton("Save")` | unknown component `Buton`: no `component Buton` is declared | Check the spelling, or declare it |
| `Button("Save").huge` | Button has no flag or enum case `huge`, then every flag it takes | Use a listed flag |
| `Row(gap: .huge)` | `gap` on Row has no case `.huge`, then the cases | Use a listed case |
| `Card("Laptop", 999)` | Only the first argument may be positional | Name the others: `Card("Laptop", price: 999)` |
| `page P(layout: Shell)` where `Shell` has no `slot` | `Shell` declares no default slot, so the page has nowhere to go | Add `slot` and place `children` |
| two `component Card2` | duplicate component `Card2` | Rename or remove one |
| `Chip("a") { on chnage { … } }` | `Chip` declares no event `chnage` | Handle a declared event, or a DOM event |
| `Panel { footer { … } }` | `Panel` declares no slot `footer` | Fill a declared slot, or declare `slot footer` |
| `n = n + 1` loose in a page | `n` is not an element or a statement a render block can hold | Put it in `on click { }`, an `action` or an `effect` |
| `Button("Save", onclick: "save()")` | `onclick` on Button would put script in an attribute | `Button("Save") { on click { save() } }` |
| `Link("Go", to: "javascript:…")` | `to` on Link names the `javascript` scheme, which a browser runs | A relative URL, or `http`, `https`, `mailto`, `tel`, `sms`, `ftp` |
| `env.STRIPE_SECRET` in a page | `env.STRIPE_SECRET` is not public, and a page reads what is in the bundle | Rename it `PUBLIC_…` or list it in `public_env` — only if anyone may read it ([chapter 30](30-environments.md)) |

A parse error — a missing brace, a stray character — names what it expected
and what it found, at the line it stopped.

## Warnings without a code

| You wrote | The compiler says |
|---|---|
| `Button("Save", colour: "red")` | Button has no prop `colour`; it is written to the element as an attribute. A typo does nothing on screen. `aria-*`, `data-*` and the global attributes (`id`, `role`, `title`, …) are expected and draw nothing. |
| `Icon("rocket")` | `rocket` is not an icon the runtime draws; it will show as the word — and the list of icons |
| a key in `webfluent.app.json` nothing reads | `…` is not a setting, and nothing reads it — did you mean `…`? (**4.1**) |

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
Error: [T01] `count` is `String`, but `Number` is wanted at src/App.wf:4:34
  Convert it: `Number(value)`
```

**Fix:** Give the state a value of its type, or convert: `Number(text)`, `"{n}"`. If the state really holds either, declare it `Any`.

### T02 — A case the enum does not have

```wf expect T02
enum Tone { calm, loud }
page P(path: "/", title: "T", description: "D") {
    state tone: Tone = .quiet
    Heading("Tone").h1
}
```

```text
Error: [T02] `Tone` has no case `.quiet` at src/App.wf:3:17
  `Tone` takes .calm, .loud
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
Error: [T04] `user.email` may be null, so `.toUpperCase()` may fail at src/App.wf:5:10
  Unwrap it first: `if let x = value { … }`, `value ?? fallback`, `value?.{method}()`, or a check for `!= null`
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
Error: [T05] `User` has no field `nmae` at src/App.wf:4:18
  Its fields are `name`
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
Error: [T06] `Cart` has no member `count` at src/App.wf:4:15
  Its members are `items`
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
Error: [T07] `if` reads `items` as a condition, but a list is always true at src/App.wf:4:5
  Ask about its length: `items.length > 0`
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
Error: [T08] `for` loops over a list, but `total` is `Number` at src/App.wf:4:5
  Give it a list, or `.split(…)` a string first
```

**Fix:** Loop over a list — `for n in 1..=total` for a range of numbers.

### T09 — An emit that does not match its event

```wf expect T09
component Stepper(_ start: Number) {
    event change(value: Number)
    Button("+") { on click { emit change(1, 2) } }
}
```

```text
Error: [T09] `emit change` passes 2 arguments, but the event takes 1 at src/App.wf:3:30
  `event change(value: Number)`
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
Error: [T10] `add` takes 1 argument, but 2 are given at src/App.wf:5:30
```

**Fix:** Pass the arguments the action, endpoint or function takes.

### T11 — A match on something that cannot be matched

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
Error: [T11] `match` needs a resource or an enum, but `n` is `Number` at src/App.wf:4:5
  Use `if` for a condition; `match` chooses among a resource's states or an enum's cases
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
Error: [T12] `token` is a `Secret`, and `Text` would show it at src/App.wf:4:10
  A secret must not reach the page; send it as a value, or show one field of what it unlocks
```

**Fix:** Do not show, splice, log or persist a `Secret`. Send it in a request — or keep it on the server.

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
Warning [A01]: Image missing "alt" attribute at src/App.wf:3:5
  Add alt text: Image(src: "...", alt: "Description of image")
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
Warning [A02]: IconButton missing accessible label at src/App.wf:3:5
  Add a label: IconButton(icon: "close", label: "Close dialog")
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
Warning [A03]: Input missing "label" or "placeholder" attribute at src/App.wf:4:5
  Add a label: Input(label: "Username").text
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
Warning [A04]: Checkbox missing "label" attribute at src/App.wf:4:5
  Add a label: Checkbox(bind: value, label: "Description")
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
Warning [A05]: Button has no text content at src/App.wf:3:5
  Add text: Button("Save").primary
```

**Fix:** Give it text, or an `aria-label:` when it shows only an icon.

### A06 — A link with no text

```wf expect A06
page P(path: "/", title: "T", description: "D") {
    Heading("Links").h1
    Link(to: "/about")
}
```

```text
Warning [A06]: Link has no text content at src/App.wf:3:5
  Add text: Link("About", to: "/about")
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
Warning [A07]: Heading has empty text content at src/App.wf:3:5
  Headings should have meaningful text
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
Warning [A08]: Modal missing "title" attribute at src/App.wf:4:5
  Add a title: Modal(visible: state, title: "Dialog Title")
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
Warning [A09]: Video missing "captions" at src/App.wf:3:5
  Add captions: Video(src: "...", captions: "/captions.en.vtt")
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
Warning [A10]: Table missing header row (Thead) at src/App.wf:3:5
  Add a header: Table { Thead { Tcell("Column Name") } ... }
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
Warning [A11]: Heading level skips from h1 to h4 at src/App.wf:3:5
  Use h2 instead, or add the missing intermediate headings
```

**Fix:** Use the next level down (`h2` after `h1`). Size text with `style { font-size }`, not with the heading level.

### A12 — A page without exactly one h1

```wf expect A12
page P(path: "/", title: "T", description: "D") {
    Heading("Section").h2
}
```

```text
Warning [A12]: Page has no h1 heading at src/App.wf:1:1
  Add a main heading: Heading("Page Title").h1
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
Warning [A13]: body text on the page background has a contrast ratio of 1.92:1, below the 4.5:1 WCAG AA minimum at src/App.wf:3:1
  Darken --color-text or lighten --color-background until they clear 4.5:1
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
Warning [A14]: role "tablist" requires children with role "tab", but holds a Button at src/App.wf:3:5
  Give each child role: "tab", or use the built-in that owns this structure
```

**Fix:** Give each child the role the parent requires (`role: "tab"`), or use the built-in that owns the structure — `Tabs`.

### A15 — An aria-label that does not contain the visible text

```wf expect A15
page P(path: "/", title: "T", description: "D") {
    Heading("Save").h1
    Button("Save", aria-label: "Submit the form")
}
```

```text
Warning [A15]: Button shows "save" but its aria-label says "Submit the form" at src/App.wf:3:5
  Start the aria-label with the visible text, so what a user says matches what they see
```

**Fix:** Start the `aria-label` with the visible words: `aria-label: "Save the draft"`.

## Search and sharing

Warnings. What a search engine or a link preview would act on.

### S01 — A page with no title

```wf expect S01
page P(path: "/", description: "D") {
    Heading("No title").h1
}
```

```text
Warning [S01]: Page P has no title at src/App.wf:1:1
  Add one: page Name(path: "/", title: "What this page is")
```

**Fix:** `page P(path: "/", title: "What this page is")`.

### S02 — A page with no description

```wf expect S02
page P(path: "/", title: "T") {
    Heading("No description").h1
}
```

```text
Warning [S02]: Page P has no description at src/App.wf:1:1
  Add one: page Name(path: "/", title: "…", description: "A sentence a search result can show")
```

**Fix:** Add a `description:` of about 150 characters: what a reader gets from the page.

### S03 — A description too long for a search result

```wf expect S03
page P(path: "/", title: "T", description: "A description that goes on for far longer than any search result will ever show, because it keeps adding clauses, qualifications and asides until the snippet is cut mid-sentence.") {
    Heading("Long").h1
}
```

```text
Warning [S03]: Page P's description is 178 characters; a search result shows about 160 at src/App.wf:1:1
  Shorten it, or accept that it will be cut mid-sentence
```

**Fix:** Shorten it to about 160 characters.

### S04 — Two pages on one route

```wf expect S04
page A(path: "/about", title: "A", description: "D") { Heading("A").h1 }
page B(path: "/about", title: "B", description: "D") { Heading("B").h1 }
```

```text
Warning [S04]: Pages A and B both claim the route /about at src/App.wf:2:1
  Give each page its own path
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
Warning [P01]: `items` is at version 2, and nothing brings version 1 forward at src/App.wf:2:5
  A reader who last visited then loses what they had. Add `migrate 1 -> 2`
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
Warning [P02]: `migrate 1 -> 2` on `items` is past the declared version at src/App.wf:2:5
  A step above `version:` never runs; raise the version or drop the step
```

**Fix:** Raise `version:` to the step's target, or drop the step.

### P03 — A persisted value in a route-scoped store

```wf expect P03
store Filters(scope: .route) {
    persist query = ""
}
```

```text
Warning [P03]: `query` is persisted in a store scoped to the route at src/App.wf:2:5
  The route change drops the store and the next read builds it again from storage, so the value comes straight back. Use `state` for what the route owns, or widen the scope for what outlives it
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
Warning [U01]: `unused` is declared but never read at src/App.wf:2:5
  Nothing reads the state; remove it, or name it `_unused` to keep it
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
Warning [U02]: `doubled` is declared but never read at src/App.wf:3:5
  Nothing reads the derived value; remove it, or name it `_doubled` to keep it
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
Warning [U03]: `Orphan` is declared but never placed at src/App.wf:1:1
  Place it in a page, name it as a layout, or remove it
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
Warning [U04]: `Cart.coupon` is declared but never read at src/App.wf:3:5
  Nothing reads the state, inside the store or as `Cart.coupon`; remove it, or name it `_coupon` to keep it
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
Warning [U05]: `reset` is declared but never read at src/App.wf:3:5
  Nothing reads the action; remove it, or name it `_reset` to keep it
```

**Fix:** Call it, remove it, or name it `_reset`.

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
Warning [V01]: nothing in scope declares `primary`, so it reads as nothing — a flag is written `.primary` at src/App.wf:3:12
```

**Fix:** Write the flag with its dot — `Button("Save").primary` — or declare the name.

### V03 — Markup put in as markup

```wf expect V03
page P(path: "/", title: "T", description: "D") {
    state body = "<b>Hi</b>"
    Heading("Markup").h1
    Unsafe.Html(body)
}
```

```text
Warning [V03]: `Unsafe.Html` puts markup in as markup at src/App.wf:4:5
  Anything the project did not write itself belongs in `sanitize(…)` first
```

**Fix:** Wrap markup from outside the project in `sanitize(…)`: `Unsafe.Html(sanitize(body))`.

### V02 — A flag no stylesheet styles

A flag on one of your components whose class no stylesheet — the engine's
or one of the project's `.css` files — defines, so it changes nothing on
screen. The built-ins cannot draw it: the registry and the engine's
stylesheet are held to each other. **Fix:** define the class in a `.css`
file under `src/`, or drop the flag.

## Next

[Built-ins](40-built-ins.md).
