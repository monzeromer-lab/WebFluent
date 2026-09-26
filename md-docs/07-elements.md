# 7. Elements

<!--
route: guide/elements
group: basics
blurb: Everything on screen is an element: a call with one positional value, named props, flags, and a block.
description: The element call shape: positional, named props, flags and enum cases, the block and its order, attributes, parts, and the families of built-ins.
-->

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

Every element also takes the **universal props**, which are not listed on
each one:

| Prop | Does | Chapter |
|---|---|---|
| `class:` | Adds classes from your own stylesheets (it adds, it never replaces) | [15](15-styling.md#classes-and-your-own-stylesheet) |
| `ref:` | Names a handle on the element: `box.focus()` | [8](08-state-and-reactivity.md#element-handles) |
| `animate:` or a flag (`.fadeIn`) | The animation it enters with | [20](20-motion.md) |
| `exit:` | The animation it leaves with | [20](20-motion.md#exit-animations) |
| `delay:`, `duration:`, `speed:` (`.fast`, `.slow`), `easing:` | How the animation is timed | [20](20-motion.md) |
| `stagger:` | Inside a `for`, each item's delay grows by this much | [20](20-motion.md#lists) |
| `on:` | When the animation plays: `.mount` (default) or `.enterView` | [20](20-motion.md#playing-when-it-is-scrolled-to) |
| `shared:` | A name the element keeps across a route change | [20](20-motion.md#the-same-element-across-a-route-change) |
| `count:` | A number in its text counts to its new value | [20](20-motion.md#a-number-that-counts) |

The [components reference](36-components-reference.md) lists every element's
own props.

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

A part belongs inside its owner: `Card.Body` is drawn as a card's body
section, and outside a `Card` it is only a section with no card around it.
Your own components may declare parts too (chapter 11).

## Text and headings

```wf
page P(path: "/") {
    Heading("Title").h1                       // a real <h1>; .h2 is the default
    Text("Plain, {count} live").bold.muted   // an inline run of text
    Paragraph("Prose, as a <p>.")
    Code("let x = 1").block
    Blockquote("A quotation.")
    Markdown("**Bold** and a [link](/docs)")  // chapter 27
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

## The families of built-ins

| Family | Elements |
|---|---|
| Layout | `Container`, `Row`, `Column`, `Grid`, `Stack`, `Spacer`, `Divider`, `Header`, `Section`, `Footer`, `Host` |
| Navigation | `Navbar`, `Sidebar`, `Link`, `Tabs`, `Breadcrumb`, `Menu` |
| Data display | `Card`, `Table`, `List`, `Badge`, `Tag`, `Avatar`, `Tooltip` |
| Forms | `Form`, `Input`, `Textarea`, `Select`, `Checkbox`, `Radio`, `Switch`, `Slider`, `DatePicker`, `FileUpload` ([Forms](16-forms.md)) |
| Feedback | `Alert`, `Toast`, `Modal`, `Dialog`, `Spinner`, `Progress`, `Skeleton` ([Events](09-events.md#overlays-modals-dialogs-toasts-and-menus)) |
| Actions | `Button`, `IconButton`, `ButtonGroup`, `Dropdown` |
| Media | `Image`, `Video`, `Audio`, `Icon`, `Carousel` ([Media](28-media.md)) |
| Typography | `Text`, `Heading`, `Paragraph`, `Code`, `Blockquote`, `Markdown`, `Unsafe.Html` |
| Documents and decks | `Document`, `PageBreak`, `Presentation`, `Slide`, `TitleSlide`, `SectionSlide`, `TwoColumn`, `ImageSlide` ([PDF and slides](33-pdf-and-slides.md)) |
| Routing | `Router` ([Pages](06-pages-and-routing.md#the-app-shell)) |

## What an element renders

Each built-in renders real, semantic HTML — a `Button` is a `<button>`, a
`Link` an `<a>`, a `Heading(…).h2` an `<h2>`, a `Table` a `<table>` with
`<th scope="col">` in its head — and carries a stable class named
`wf-<name>`, with modifier classes for its flags: `Button("Save").primary.lg` is
`<button class="wf-btn wf-btn--primary wf-btn--large">`. The classes are part
of the public surface and stable across releases, so your own stylesheet may
target them ([Styling](15-styling.md#classes-and-your-own-stylesheet)). The
reference names each element's class.

## What the compiler checks on every element

- The name is a built-in, a part in its owner, or a declared component.
- The positional argument is one the element takes; there is at most one.
- Every named prop exists, with a value of its type; an enum prop's case exists.
- Every flag resolves to one prop.
- Handlers name a DOM event or one of the component's declared events.
- Slot fills name a declared slot.
- An `Icon("name")` or `icon:` names one of the icons the runtime draws
  ([the list](36-components-reference.md#icons)); any other name shows as
  the word.
- Accessibility: an `Image` has `alt:`, an `IconButton` has `label:`, a control
  has a label, a `Modal` a title, a `Table` a `Table.Head` (`A01`–`A10`).

## Next

[State and reactivity](08-state-and-reactivity.md).
