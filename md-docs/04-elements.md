# 4. Elements

Everything a page shows is an element: a built-in such as `Button`, or a
component you declare. Every element is written the same way.

## The call shape

```text
Name(positional, prop: value, prop: value).flag.flag { block }
```

- **`Name`** — PascalCase. A built-in (`Card`), a part of one (`Card.Header`),
  or a component of the project (`UserCard`, `Panel.Footer`).
- **One positional argument**, first, unnamed — the element's own thing: a
  button's label, a heading's text, an icon's name. Every other value is passed
  by name. A second positional argument is an error (`Card("Laptop", 999)` —
  name it: `Card("Laptop", price: 999)`).
- **Named props** `name: value`, in any order, any expression as the value.
- **Flags** `.name`, written tight against the closing parenthesis (or the
  name, when there are no parentheses): `Button("Save").primary.lg`,
  `Spacer.sm`, `Card.elevated { … }`.
- **The block** `{ … }`, in a fixed order: `style { }` · `transition { }` ·
  `on … { }` handlers · slot fills · children.

```wf
page P(path: "/") {
    Button("Save", disabled: false).primary.lg {
        style { min-width: 120px }
        transition { background: 150ms ease-out }
        on click { log("saved") }
        Icon("check")
    }
}
```

## Props, cases and flags

A prop has a type the compiler knows: `String`, `Number`, `Bool`, a state
binding, a path, or an **enum** — a fixed set of cases. An enum prop takes a
case written with a leading dot:

```wf
page P(path: "/") {
    Row(gap: .md, align: .center, justify: .between) {
        Text("left")
        Text("right")
    }
}
```

A **flag** is shorthand for two things: a `Bool` prop set to `true`, or a
case of exactly one of the element's enum props:

```wf
page P(path: "/") {
    Button("Danger").danger            // tone: .danger
    Button("Wide").full.outlined       // width: .full, outlined: true
    Input(bind: q).search.required     // type: .search, required: true
    state q = ""
}
```

The two spellings are equivalent; `.md` on a `Row` is `gap: .md` because
`gap` is the only prop of `Row` with an `md` case. When two props share a
case name the flag is ambiguous and the compiler asks you to name the prop.
A flag the element does not declare is an error, with the list of what it
takes — there is no shared vocabulary of modifiers to guess from.

Every element also takes the universal motion props (`animate:`, `exit:`,
`delay:`, `stagger:`, `speed:`, `duration:`, `easing:`) and their flags
(`.fadeIn`, `.fast`), `class:` for your own stylesheet classes, and `ref:`
(chapter 5). See the [reference](19-components-reference.md) for every
element's props.

## Attributes

Named arguments that are not props are HTML attributes, checked against the
families the element accepts: the global attributes (`id`, `title`, `role`,
`tabindex`, `lang`, …), `aria-*`, `data-*`, and per element the input, anchor,
media, form and table attributes. A hyphenated name is written as-is:

```wf
page P(path: "/") {
    Button("More", aria-expanded: "false", aria-controls: "panel", data-track: "more")
    Input(bind: name, id: "name", autocomplete: "name", maxlength: 40)
    state name = ""
}
```

An attribute value that reads state is kept in step with it; `aria-*` keeps a
`false` value (it means something), other attributes drop it.

## Parts

Some built-ins are made of parts, written under their owner's name:

```wf
page P(path: "/") {
    Card.elevated {
        Card.Header { Heading("Title").h3 }
        Card.Body { Text("Body") }
        Card.Footer { Button("OK").primary }
    }
    Table(caption: "Regions") {
        Table.Head { Table.Row { Table.Cell("Region").header  Table.Cell("Latency").header } }
        Table.Body { Table.Row { Table.Cell("fra1")  Table.Cell("34 ms") } }
    }
    Select(bind: region, label: "Region") {
        Select.Option("Frankfurt", value: "fra1")
        Select.Option("Iowa", value: "iad1")
    }
    Tabs {
        Tabs.Page("One") { Text("first") }
        Tabs.Page("Two") { Text("second") }
    }
    state region = "fra1"
}
```

A part is only valid directly inside its owner; `Card.Body` outside a `Card`
is an error. Your own components may declare parts too (chapter 8).

## Text and headings

```wf
page P(path: "/") {
    Heading("Title").h1                       // a real <h1>; .h2 is the default
    Text("Plain, {count} live").bold.muted   // an inline run of text
    Paragraph("Prose, as a <p>.")
    Code("let x = 1").block
    Blockquote("A quotation.")
    Markdown("**Bold** and a [link](/docs)")  // chapter 16
    state count = 3
}
```

Headings form an outline the compiler checks: one `h1` per page, no skipped
levels (`A11`, `A12`).

## Layout elements

```wf
page P(path: "/") {
    Container {                                  // centred, max-width
        Row(gap: .md, align: .center) {          // horizontal flex
            Column(span: 8) { Text("main") }     // 12-column grid child
            Column(span: 4) { Text("aside") }
        }
        Grid(columns: 3, gap: .lg) { Card { Text("a") }  Card { Text("b") }  Card { Text("c") } }
        Stack(gap: .sm) { Text("one")  Text("two") }   // vertical flex
        Spacer.lg
        Divider
    }
    Header { Text("a page region") }
    Section { Text("another") }
    Footer { Text("the last") }
}
```

`Header`, `Section` and `Footer` are real landmarks (`<header>`, `<section>`,
`<footer>`); use them for page regions so the page has an outline a screen
reader and a search engine can follow.

## What the compiler checks on every element

- The name is a built-in, a part in its owner, or a declared component.
- The positional argument is one the element takes; there is at most one.
- Every named prop exists, with a value of its type; an enum prop's case exists.
- Every flag resolves to one prop.
- Handlers name a DOM event or one of the component's declared events.
- Slot fills name a declared slot.
- An `Icon("name")` or `icon:` names one of the icons the runtime draws
  ([the list](19-components-reference.md#icons)); any other name shows as
  the word.
- Accessibility: an `Image` has `alt:`, an `IconButton` has `label:`, a control
  has a label, a `Modal` a title, a `Table` a `Table.Head` (`A01`–`A10`).

## Next

[State and reactivity](05-state-and-reactivity.md).
