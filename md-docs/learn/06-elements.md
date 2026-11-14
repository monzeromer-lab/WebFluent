---
chapter: 6
kind: "tutorial"
route: "/learn/06-elements"
from: ["/docs/guide/elements"]
group: "language"
blurb: "Everything on screen is an element, and every element is written the same way: a name, one positional value, named props, flags and a block."
description: "The element call shape: positional, named props, flags and enum cases, the block and its order, attributes, parts, and the families of built-ins."
learned: ["Read and write any element call: `Name(value, prop: value).flag { block }`", "Pick an enum prop's case by name (`gap: .md`) or as a flag (`.primary`)", "Put HTML attributes such as `aria-*`, `data-*` and `id` on an element", "Place parts inside their owner: `Card.Header`, `Table.Row`", "Know which built-in family to reach for, and what the compiler checks on each call"]
---
# 6. Elements

This chapter covers WebFluent's elements: everything a page shows, whether a
built-in such as `Button` or a component you declare. Every element is
written in the same shape, so once you can read one call you can read them
all.

In this chapter:

- the shape of an element call, and the order of its block
- props, enum cases and flags, and the props every element takes
- HTML attributes, and which element of a built-in receives them
- parts, such as `Card.Header`
- the text, heading and layout elements
- the families of built-ins, the HTML they render, and what the compiler
  checks on every element

## The call shape

Every element call has the same five pieces, each optional except the name:

```text
Name(positional, prop: value, prop: value).flag.flag { block }
```

- **`Name`** is PascalCase: a built-in (`Card`), a part of one
  (`Card.Header`), or a component of the project (`UserCard`,
  `Panel.Footer`).
- **One positional argument** comes first, unnamed. It is the element's main
  value: a button's label, a heading's text, an icon's name. You pass every
  other value by name.
- **Named props**, `name: value`, come in any order, and the value may be any
  expression.
- **Flags**, `.name`, are written tight against the closing parenthesis, or
  against the name when there are no parentheses: `Button("Save").primary.lg`,
  `Spacer.sm`, `Card.elevated { … }`. An element or a component called with
  no arguments needs no parentheses: `Spinner` and `Spinner()` are the same
  call, and this book writes the shorter one.
- **The block**, `{ … }`, holds its contents in a fixed order: `style { }`,
  then `transition { }`, then `on … { }` handlers, then slot fills, then
  children.

This button uses all five pieces, with its block in that order:

```wf
page Editor(path: "/", title: "Editor", description: "A save button, fully dressed.") {
    Heading("Editor").h1
    Button("Save", disabled: false).primary.lg {
        style { min-width: 120px }
        transition { background: 150ms ease-out }
        on click { log("saved") }
        Icon("check")
    }
}
```

> [!NOTE]
> **Coming from JSX or a Vue template?** There are no tags and no string
> attributes: `<Button variant="primary" size="lg">Save</Button>` is
> `Button("Save").primary.lg`. A prop's value is an expression, never a
> string to be parsed, and children go in the block. See
> [Coming from React](../coming-from/react.md#the-map).

## Props, cases and flags

Each prop has a type the compiler knows: `String`, `Number`, `Bool`, a state
binding, a path, or an **enum**, which is a fixed set of named cases. You
pass an enum prop a case, written with a leading dot:

```wf
page Toolbar(path: "/", title: "Toolbar", description: "Two labels, spaced apart.") {
    Heading("Toolbar").h1
    Row(gap: .md, align: .center, justify: .between) {
        Text("Drafts")
        Text("3 unsent")
    }
}
```

A **flag** is a short way to write one of two things: a `Bool` prop set to
`true`, or a case of one of the element's enum props. Each comment shows the
long spelling of the flags on its line:

```wf
page Search(path: "/", title: "Search", description: "A search box and two buttons.") {
    state term = ""
    Heading("Search").h1
    Button("Delete").danger                                // tone: .danger
    Button("Continue").full.outlined                       // width: .full, outlined: true
    Input(bind: term, label: "Search").search.required     // type: .search, required: true
}
```

The two spellings mean the same thing. `.md` on a `Row` means `gap: .md`,
because `gap` is the only prop of `Row` with an `md` case. A flag follows
these rules:

- When two enum props of an element share a case name, the flag is
  ambiguous. The compiler reports `E103` and asks you to name the prop.
- A flag the element does not declare is error `E103`, and the message lists
  the flags the element takes. There is no shared vocabulary of modifiers to
  guess from.
- A `Bool` prop may take a condition instead of a flag:
  `Button("Light", outlined: theme != "light")`.

### Universal props

Every element also takes the *universal props*. The reference does not
repeat them on each element:

| Prop | What it does | Read more |
|---|---|---|
| `class:` | Adds classes from your own stylesheets. It adds; it never replaces | [Styling](13-styling.md#classes-and-your-own-stylesheet) |
| `ref:` | Names a handle on the element: `box.focus()` | [State and reactivity](07-state-and-reactivity.md#element-handles) |
| `mount:`, `update:`, `cleanup:` | Hands the element to a script each time it is made, again when what `update` reads changes, and takes it back when it leaves | [Hand an element to a script](../recipes/hand-an-element-to-a-script.md) |
| `animate:` or a flag (`.fadeIn`) | The animation it enters with | [Animate in and out](../recipes/animate-in-and-out.md#animate-an-element-in) |
| `exit:` | The animation it leaves with | [Animate in and out](../recipes/animate-in-and-out.md#animate-an-element-out) |
| `delay:`, `duration:`, `speed:` (`.fast`, `.slow`), `easing:` | How the animation is timed | [Animate in and out](../recipes/animate-in-and-out.md), [Motion defaults](../recipes/motion-defaults.md) |
| `stagger:` | Inside a `for`, each item's delay grows by this much | [Animate a list](../recipes/animate-a-list.md) |
| `on:` | When the animation plays: `.mount` (the default) or `.enterView` | [Animate in and out](../recipes/animate-in-and-out.md#play-an-animation-when-it-is-scrolled-to) |
| `shared:` | A name the element keeps across a route change | [Shared element](../recipes/shared-element.md) |
| `count:` | A number in its text counts up or down to its new value | [Count a number](../recipes/count-a-number.md) |

The [components reference](../reference/components.md) lists every element's
own props.

## Attributes

A named argument that is not a prop becomes an HTML attribute. These
families are declared, so they draw no warning:

- the global attributes: `id`, `title`, `role`, `tabindex`, `lang`, `dir`,
  `hidden`;
- `aria-*` and `data-*`;
- the attributes of the element's kind: input, anchor, media, form and table
  attributes on the elements that take them.

Any other name is still written to the element, with warning `C04`. You write
a hyphenated name as it is:

```wf
page Signup(path: "/", title: "Sign up", description: "A name field and a menu button.") {
    state name = ""
    Heading("Sign up").h1
    Button("More", aria-expanded: "false", aria-controls: "panel", data-track: "more")
    Input(bind: name, label: "Name", id: "name", autocomplete: "name", maxlength: 40)
}
```

### Attribute values

An attribute's value follows these rules:

- A value that reads state is kept in step with it.
- `true` is written as the word: `data-open: true` renders
  `data-open="true"`, so the selector `[data-open="true"]` matches it.
- A boolean attribute such as `hidden` or `disabled` is either present or
  absent.
- `false` removes an attribute, except an `aria-*` one, which keeps the
  string `"false"` because it is a real ARIA state.

The same rules apply to an attribute you write on a call of your own
component, which passes it to the component's root.

### Where a built-in puts them

A built-in places your attributes and `on` handlers on the element it draws:

- On its root, for most built-ins: the scrolling box of a `VirtualList`, the
  handle of a `Splitter`, the box of a `Tabs` or a `Carousel`, the panel of a
  `Sidebar`, the `<nav>` of a `Breadcrumb`, the `<dialog>` of a `Modal`.
- On its input, for a control drawn inside a box (`Switch`, `Checkbox`,
  `Radio`, `Slider`, `DatePicker`, `FileUpload`). The input is the control,
  and its own handlers are already there. `hidden`, `lang` and `dir` belong
  to the whole control, so they stay on its box.

A `role:` or `aria-label:` you write takes the place of the one the built-in
gives itself: a `Splitter`'s `separator`, a `Switch`'s `switch`, a
`Breadcrumb`'s name. A `Sidebar`'s `id:` is the one its toggle points at.

This list box puts `role`, `tabindex` and `aria-activedescendant` on the
`VirtualList`'s scrolling box, and a `role` and `id` on each row:

```wf
page References(path: "/", title: "References", description: "A list box of references.") {
    state refs = [{ id: "a" }, { id: "b" }, { id: "c" }]
    state current = 0
    Heading("References").h1
    VirtualList(rowHeight: 24, label: "References", role: "listbox", tabindex: "0", aria-activedescendant: "ref-{current}") {
        style { height: 240px }
        on keydown(e) { if e.key == "ArrowDown" { current = current + 1 } }
        for r in refs by r.id { Text(r.id, id: "ref-{r.id}", role: "option") }
    }
}
```

The pre-rendered page and the template engine write the same attributes on
the same elements, as far as the build knows their values.

## Parts

Some built-ins are made of parts. You write a part under its owner's name,
inside its owner:

```wf
page Regions(path: "/", title: "Regions", description: "A card, a table, a select and tabs.") {
    state region = "fra1"
    Heading("Regions").h1
    Card.elevated {
        Card.Header { Heading("Frankfurt").h2 }
        Card.Body { Text("Our main region.") }
        Card.Footer { Button("Open").primary }
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
        Tabs.Page("Summary") { Text("Two regions.") }
        Tabs.Page("Details") { Text("fra1 and iad1.") }
    }
}
```

A part belongs inside its owner. `Card.Body` outside a `Card` is error
`C03`. Your own components may declare parts too
([Components](10-components.md)).

A `Tabs` keeps the open tab in a state of yours with `bind:`, holding the
open page's `value:`. Only the open tab is in the Tab order; the arrow keys
move between tabs, the other way round in a right-to-left page:

```wf
page Account(path: "/account", title: "Account", description: "Your profile and billing.") {
    state tab = "profile"
    Heading("Account").h1
    Tabs(bind: tab, label: "Account") {
        Tabs.Page("Profile", value: "profile") { Text("Your name and photo.") }
        Tabs.Page("Billing", value: "billing") { Text("Your plan and invoices.") }
    }
    Text("Open: {tab}")
}
```

Any element of your own can move the same way: `roving: .x` makes a row
one stop for Tab, with the arrow keys inside it
([Events](08-events.md#moving-through-a-group-with-the-arrow-keys)).

## Text and headings

The typography elements cover inline text, headings, paragraphs, code,
quotations and Markdown:

```wf
page Article(path: "/", title: "Article", description: "Every kind of text.") {
    state count = 3
    Heading("Release notes").h1                // a real <h1>; .h2 is the default
    Text("{count} changes").bold.muted         // a run of text
    Paragraph("Prose, as a <p>.")
    Code("let x = 1").block
    Blockquote("A quotation.")
    Markdown("**Bold** and a [link](/docs)")   // Markdown, rendered as HTML
}
```

The compiler checks the outline the headings form: a page needs an `h1`
(`A12`), and a heading level may not be skipped, such as `h2` straight to
`h4` (`A11`).

[Render Markdown](../recipes/render-markdown.md) shows what `Markdown`
draws, and how to show text somebody else wrote.

## Layout elements

The layout elements arrange what is inside them. Each comment says what the
element is:

```wf
page Dashboard(path: "/", title: "Dashboard", description: "Every layout element at once.") {
    Header { Heading("Dashboard").h1 }
    Container {                                  // centred, with a max width
        Row(gap: .md, align: .center) {          // horizontal flex
            Column(span: 8) { Text("main") }     // a child of a 12-column grid
            Column(span: 4) { Text("aside") }
        }
        Grid(columns: 3, gap: .lg) { Card { Text("a") }  Card { Text("b") }  Card { Text("c") } }
        Stack(gap: .sm) { Text("one")  Text("two") }   // vertical flex
        Spacer.lg
        Divider
    }
    Section { Text("another region") }
    Footer { Text("the last region") }
}
```

`Header`, `Section` and `Footer` render real landmarks: `<header>`,
`<section>` and `<footer>`. Use them for the page's regions, so a screen
reader and a search engine can follow the page's outline. Inside a
`Document`, `Header` and `Footer` mean something else: they are drawn in
the margin of every printed page ([PDF and slides](17-pdf-and-slides.md#set-the-paper-headers-and-footers)).

## The families of built-ins

The built-ins fall into these families:

| Family | Elements |
|---|---|
| Layout | `Container`, `Row`, `Column`, `Grid`, `Stack`, `Spacer`, `Divider`, `Header`, `Section`, `Footer`, `VirtualList`, `Splitter` |
| Navigation | `Navbar`, `Sidebar`, `Link`, `Tabs`, `Breadcrumb`, `Menu` |
| Data display | `Card`, `Table`, `List`, `Badge`, `Tag`, `Avatar`, `Tooltip` |
| Forms | `Form`, `Input`, `Textarea`, `Select`, `Checkbox`, `Radio`, `Switch`, `Slider`, `DatePicker`, `FileUpload` ([Forms](14-forms.md)) |
| Feedback | `Alert`, `Toast`, `Modal`, `Dialog`, `Popover`, `Spinner`, `Progress`, `Skeleton` ([Events](08-events.md#overlays-modals-dialogs-toasts-and-menus)) |
| Actions | `Button`, `IconButton`, `ButtonGroup`, `Dropdown` |
| Media | `Image`, `Video`, `Audio`, `Frame`, `Icon`, `Carousel` ([Add a picture](../recipes/add-a-picture.md), [Video and audio](../recipes/video-and-audio.md)) |
| Typography | `Text`, `Heading`, `Paragraph`, `Code`, `Blockquote`, `Markdown`, `Unsafe.Html` |
| Somebody else's code | `Host`, `Element`, `Portal` ([Call your own JavaScript](../recipes/call-your-own-script.md), [Hand an element to a script](../recipes/hand-an-element-to-a-script.md)) |
| Documents and decks | `Document`, `PageBreak`, `TableOfContents`, `Background`, `Watermark`, `Chart`, `QrCode`, `Presentation`, `Slide`, `TitleSlide`, `SectionSlide`, `TwoColumn`, `ImageSlide` ([PDF and slides](17-pdf-and-slides.md)) |
| Routing | `Router` ([Pages](05-pages-and-routing.md#the-app-shell)) |

## What an element renders

Each built-in renders real, semantic HTML. A `Button` is a `<button>`, a
`Link` an `<a>`, a `Heading(…).h2` an `<h2>`, and a `Table` a `<table>` with
`<th scope="col">` cells in its head.

Each also carries a stable class named after it, with a modifier class for
each flag. For example, `Button("Save").primary.lg` renders:

```html
<button class="wf-btn wf-btn--primary wf-btn--large" type="button">Save</button>
```

These classes are part of the public surface and stay stable across
releases, so your own stylesheet may target them
([Styling](13-styling.md#classes-and-your-own-stylesheet)). The reference
names each element's class.

## What the compiler checks on every element

The compiler holds every element call to these rules:

| What it checks | When it fails |
|---|---|
| The name is a built-in, a part inside its owner, or a declared component | `E101`; a part outside its owner is `C03` |
| There is at most one positional argument, and the element takes one | A second is `E113`; one a built-in does not take is `C05` |
| Every named prop exists, with a value of its type, and an enum prop's case exists | A wrong type is `T01`; a missing case is `E103` |
| A named argument that is not a prop is a declared attribute | Any other is written as an attribute, with `C04` |
| Every flag resolves to exactly one prop | `E103` |
| A handler names a DOM event, or one of the component's declared events | `V09` for any other name, such as a misspelled `on clik` |
| A slot fill names a declared slot | `E103` |
| An `Icon("name")` or `icon:` names an icon the runtime draws ([the list](../reference/components.md#icons)) | `V08`; the name shows as a word |
| Accessibility: an `Image` has `alt:`, an `IconButton` a `label:`, a control a label, a `Modal` a title, a `Table` a `Table.Head` | `A01`–`A10` |

## Quick reference

| Syntax | Meaning |
|---|---|
| `Name(value, prop: value)` | An element with one positional and named props |
| `.flag` | A `Bool` prop set to `true`, or an enum prop's case |
| `prop: .case` | An enum prop, named |
| `{ style { } transition { } on … { } fills children }` | The block, in its fixed order |
| `aria-label: "…"`, `data-tone: "…"`, `id: "…"` | HTML attributes |
| `Owner.Part { … }` | A part, inside its owner |
| `class:`, `ref:`, `mount:`, `animate:` | Universal props, on every element |

The [components reference](../reference/components.md) lists each built-in's
props, flags, events, slots and parts; `wf registry --json` prints the same
data for a tool.

## Common mistakes

**A second positional argument.** Only the first argument may go without a
name, so a tone passed after the label does not build:

```wf expect=E113
page Trash(path: "/", title: "Trash", description: "Empty the trash.") {
    Heading("Trash").h1
    Button("Empty the trash", "danger")
}
```

Write the tone as a flag, `Button("Empty the trash").danger`, or name it,
`tone: .danger` ([E113](../errors/E113.md)).

**A flag the element does not have.** Flags are not a shared vocabulary; each
element takes its own, and a word it does not declare stops the build:

```wf expect=E103
page Save(path: "/", title: "Save", description: "A save button.") {
    Heading("Save").h1
    Button("Save").huge
}
```

The finding lists the flags `Button` takes; `.lg` is its largest size
([E103](../errors/E103.md)).

**A part outside its owner.** A part only means something inside the element
it belongs to:

```wf expect=C03
page Profile(path: "/", title: "Profile", description: "A profile card.") {
    Heading("Profile").h1
    Card.Body { Text("Sam, designer") }
}
```

Wrap it in its owner, `Card { Card.Body { … } }`, or use a plain `Stack`
([C03](../errors/C03.md)).

## Next

[State and reactivity](07-state-and-reactivity.md).
