# Components reference

> The components reference: every built-in element, its props, flags, events, slots, parts, attribute families and an example.

The page: https://webfluent.monzeromer.dev/reference

Generated from the compiler's registry (`wf registry --json`) by `scripts/docsite/components.py`, with an example of each from `scripts/docsite/examples.py`; edit those, not this file.

Every component is written `Name(positional, prop: value).flag { block }`. A **prop** is passed by name; the one **positional** prop, where there is one, comes first and unnamed. A **flag** is written after the parentheses with a dot: a `Bool` prop, or a case of one of the element's enum props (`Button("x").primary.lg`). An **enum prop** takes a case written `.case` (`Row(gap: .md)`). Every element also takes the universal props and events listed at the end, and the HTML attribute families named on it (`aria-*`, `data-*`, and the global attributes such as `id`, `class`, `title`, `role`, `tabindex`).

## Contents

- **Layout**: [`Container`](https://webfluent.monzeromer.dev/reference/container), [`Row`](https://webfluent.monzeromer.dev/reference/row), [`Column`](https://webfluent.monzeromer.dev/reference/column), [`Grid`](https://webfluent.monzeromer.dev/reference/grid), [`Stack`](https://webfluent.monzeromer.dev/reference/stack), [`Spacer`](https://webfluent.monzeromer.dev/reference/spacer), [`VirtualList`](https://webfluent.monzeromer.dev/reference/virtuallist), [`Splitter`](https://webfluent.monzeromer.dev/reference/splitter), [`Divider`](https://webfluent.monzeromer.dev/reference/divider), [`AppShell`](https://webfluent.monzeromer.dev/reference/appshell), [`Host`](https://webfluent.monzeromer.dev/reference/host), [`Portal`](https://webfluent.monzeromer.dev/reference/portal), [`Element`](https://webfluent.monzeromer.dev/reference/element)
- **Navigation**: [`Navbar`](https://webfluent.monzeromer.dev/reference/navbar), [`Sidebar`](https://webfluent.monzeromer.dev/reference/sidebar), [`Link`](https://webfluent.monzeromer.dev/reference/link), [`Tabs`](https://webfluent.monzeromer.dev/reference/tabs), [`Breadcrumb`](https://webfluent.monzeromer.dev/reference/breadcrumb), [`Menu`](https://webfluent.monzeromer.dev/reference/menu), [`OnThisPage`](https://webfluent.monzeromer.dev/reference/onthispage), [`Pagination`](https://webfluent.monzeromer.dev/reference/pagination), [`Stepper`](https://webfluent.monzeromer.dev/reference/stepper), [`CommandPalette`](https://webfluent.monzeromer.dev/reference/commandpalette)
- **Data display**: [`Card`](https://webfluent.monzeromer.dev/reference/card), [`Table`](https://webfluent.monzeromer.dev/reference/table), [`List`](https://webfluent.monzeromer.dev/reference/list), [`Badge`](https://webfluent.monzeromer.dev/reference/badge), [`Tag`](https://webfluent.monzeromer.dev/reference/tag), [`Avatar`](https://webfluent.monzeromer.dev/reference/avatar), [`Tooltip`](https://webfluent.monzeromer.dev/reference/tooltip), [`Tree`](https://webfluent.monzeromer.dev/reference/tree), [`DataTable`](https://webfluent.monzeromer.dev/reference/datatable), [`Chart`](https://webfluent.monzeromer.dev/reference/chart), [`QrCode`](https://webfluent.monzeromer.dev/reference/qrcode)
- **Form**: [`Input`](https://webfluent.monzeromer.dev/reference/input), [`Select`](https://webfluent.monzeromer.dev/reference/select), [`Checkbox`](https://webfluent.monzeromer.dev/reference/checkbox), [`Radio`](https://webfluent.monzeromer.dev/reference/radio), [`Switch`](https://webfluent.monzeromer.dev/reference/switch), [`Slider`](https://webfluent.monzeromer.dev/reference/slider), [`DatePicker`](https://webfluent.monzeromer.dev/reference/datepicker), [`FileUpload`](https://webfluent.monzeromer.dev/reference/fileupload), [`Form`](https://webfluent.monzeromer.dev/reference/form), [`Textarea`](https://webfluent.monzeromer.dev/reference/textarea), [`Field`](https://webfluent.monzeromer.dev/reference/field), [`Fieldset`](https://webfluent.monzeromer.dev/reference/fieldset), [`InputGroup`](https://webfluent.monzeromer.dev/reference/inputgroup), [`RadioGroup`](https://webfluent.monzeromer.dev/reference/radiogroup), [`NumberInput`](https://webfluent.monzeromer.dev/reference/numberinput), [`Calendar`](https://webfluent.monzeromer.dev/reference/calendar), [`Listbox`](https://webfluent.monzeromer.dev/reference/listbox), [`Combobox`](https://webfluent.monzeromer.dev/reference/combobox)
- **Feedback**: [`Alert`](https://webfluent.monzeromer.dev/reference/alert), [`Toast`](https://webfluent.monzeromer.dev/reference/toast), [`Modal`](https://webfluent.monzeromer.dev/reference/modal), [`Popover`](https://webfluent.monzeromer.dev/reference/popover), [`Dialog`](https://webfluent.monzeromer.dev/reference/dialog), [`AlertDialog`](https://webfluent.monzeromer.dev/reference/alertdialog), [`Sheet`](https://webfluent.monzeromer.dev/reference/sheet), [`Spinner`](https://webfluent.monzeromer.dev/reference/spinner), [`Progress`](https://webfluent.monzeromer.dev/reference/progress), [`Skeleton`](https://webfluent.monzeromer.dev/reference/skeleton)
- **Content**: [`Callout`](https://webfluent.monzeromer.dev/reference/callout), [`Kbd`](https://webfluent.monzeromer.dev/reference/kbd), [`EmptyState`](https://webfluent.monzeromer.dev/reference/emptystate), [`Stat`](https://webfluent.monzeromer.dev/reference/stat), [`CodeBlock`](https://webfluent.monzeromer.dev/reference/codeblock), [`Chip`](https://webfluent.monzeromer.dev/reference/chip), [`VisuallyHidden`](https://webfluent.monzeromer.dev/reference/visuallyhidden), [`Accordion`](https://webfluent.monzeromer.dev/reference/accordion)
- **Actions**: [`CopyButton`](https://webfluent.monzeromer.dev/reference/copybutton), [`ThemeToggle`](https://webfluent.monzeromer.dev/reference/themetoggle), [`LocaleSwitch`](https://webfluent.monzeromer.dev/reference/localeswitch), [`Toggle`](https://webfluent.monzeromer.dev/reference/toggle), [`ToggleGroup`](https://webfluent.monzeromer.dev/reference/togglegroup), [`Button`](https://webfluent.monzeromer.dev/reference/button), [`IconButton`](https://webfluent.monzeromer.dev/reference/iconbutton), [`ButtonGroup`](https://webfluent.monzeromer.dev/reference/buttongroup), [`Dropdown`](https://webfluent.monzeromer.dev/reference/dropdown)
- **Media**: [`Image`](https://webfluent.monzeromer.dev/reference/image), [`Video`](https://webfluent.monzeromer.dev/reference/video), [`Audio`](https://webfluent.monzeromer.dev/reference/audio), [`Frame`](https://webfluent.monzeromer.dev/reference/frame), [`Icon`](https://webfluent.monzeromer.dev/reference/icon), [`Carousel`](https://webfluent.monzeromer.dev/reference/carousel)
- **Typography**: [`Text`](https://webfluent.monzeromer.dev/reference/text), [`Heading`](https://webfluent.monzeromer.dev/reference/heading), [`Code`](https://webfluent.monzeromer.dev/reference/code), [`Markdown`](https://webfluent.monzeromer.dev/reference/markdown), [`Unsafe`](https://webfluent.monzeromer.dev/reference/unsafe), [`Blockquote`](https://webfluent.monzeromer.dev/reference/blockquote)
- **PDF**: [`TableOfContents`](https://webfluent.monzeromer.dev/reference/tableofcontents), [`Document`](https://webfluent.monzeromer.dev/reference/document), [`Section`](https://webfluent.monzeromer.dev/reference/section), [`Paragraph`](https://webfluent.monzeromer.dev/reference/paragraph), [`Header`](https://webfluent.monzeromer.dev/reference/header), [`Footer`](https://webfluent.monzeromer.dev/reference/footer), [`Background`](https://webfluent.monzeromer.dev/reference/background), [`Watermark`](https://webfluent.monzeromer.dev/reference/watermark), [`PageBreak`](https://webfluent.monzeromer.dev/reference/pagebreak)
- **Slides**: [`Presentation`](https://webfluent.monzeromer.dev/reference/presentation), [`Slide`](https://webfluent.monzeromer.dev/reference/slide), [`TitleSlide`](https://webfluent.monzeromer.dev/reference/titleslide), [`SectionSlide`](https://webfluent.monzeromer.dev/reference/sectionslide), [`TwoColumn`](https://webfluent.monzeromer.dev/reference/twocolumn), [`ImageSlide`](https://webfluent.monzeromer.dev/reference/imageslide)
- **Routing**: [`Router`](https://webfluent.monzeromer.dev/reference/router)

## Layout

### Container

Centred wrapper with a maximum width and horizontal padding.

```
Container(fluid: …) { … }
```

```wf
Container { Heading("Pricing").h1 }
Container.fluid { Text("Edge to edge") }
```

Renders with the class `wf-container`.

| Prop | Type | Meaning |
|---|---|---|
| `fluid:` | `Bool` | Full width, no maximum |

**Flags:** `.fluid`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Row

Horizontal flex layout. It stays a row at every width unless `.stacks` or a responsive `direction` says otherwise.

```
Row(gap: …, align: …, justify: …) { … }
```

```wf
Row(gap: .md, align: .center, justify: .between) {
    Text("Left")
    Button("Right")
}
Row(direction: { base: .column, md: .row }) { Text("a")  Text("b") }
```

Renders with the class `wf-row`.

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between children |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Cross-axis alignment |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | Main-axis distribution |
| `direction:` | `.row` `.column` | Which way its children run; takes a value per breakpoint |
| `stacks:` | `Bool` | Reflow on a narrow screen: a Row becomes a column, a Grid one column |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.row` `.column` `.stacks`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Column

A child of the 12-column grid, spanning `span` columns.

```
Column(span: …, stacks: …) { … }
```

```wf
Row {
    Column(span: 8) { Text("Main") }
    Column(span: 4) { Text("Aside") }
}
```

Renders with the class `wf-col`.

| Prop | Type | Meaning |
|---|---|---|
| `span:` | `Number` | How many of the 12 grid columns to span |
| `stacks:` | `Bool` | Reflow on a narrow screen: a Row becomes a column, a Grid one column |

**Flags:** `.stacks`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Grid

CSS grid with a fixed number of columns.

```
Grid(columns: …, gap: …, align: …) { … }
```

```wf
Grid(columns: { base: 1, md: 3 }, gap: .lg) {
    Card { Text("One") }
    Card { Text("Two") }
    Card { Text("Three") }
}
```

Renders with the class `wf-grid`.

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | Number of equal columns |
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between children |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Cross-axis alignment |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | Main-axis distribution |
| `stacks:` | `Bool` | Reflow on a narrow screen: a Row becomes a column, a Grid one column |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.stacks`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Stack

Vertical flex layout.

```
Stack(gap: …, align: …, justify: …) { … }
```

```wf
Stack(gap: .sm) {
    Text("First")
    Text("Second")
}
```

Renders with the class `wf-stack`.

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between children |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Cross-axis alignment |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | Main-axis distribution |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Spacer

Vertical space; `md` when no size is given.

```
Spacer(size: …)
```

```wf
Text("Above")
Spacer.lg
Text("Below")
```

Renders with the class `wf-spacer`.

| Prop | Type | Meaning |
|---|---|---|
| `size:` | `.xs` `.sm` `.md` `.lg` `.xl` | How much space |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl`

Attribute families: `global`, `aria`, `data`.

### VirtualList

A scrolling box that draws only the rows in view of the one keyed `for` inside it — for a tree, search results or a diff of thousands of rows.

```
VirtualList(rowHeight: …, estimate: …, reveal: …) { … }
```

```wf
state rows = (0..10000).map((i) => "row {i}")
state picked = "row 0"
VirtualList(rowHeight: 32, reveal: picked, label: "Rows") {
    style { height: 320px }
    for r in rows by r { Text(r) { style { height: 32px; margin: 0 } } }
}
```

Renders with the class `wf-virtual-list`.

| Prop | Type | Meaning |
|---|---|---|
| `rowHeight:` | `Number` | Every row's height, in px: the fast path. Unset, each row is measured |
| `estimate:` | `Number` | The height a row not yet drawn stands for, in px. Unset, 32 |
| `reveal:` | `Any` | A row's key: when it changes, that row is scrolled into view — the selection, the current match |
| `label:` | `String` | What the list is, for a screen reader |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Splitter

A handle that sets the size of a region beside it: dragged, between `min:` and `max:`; double-clicked or Enter, back to `reset:`; the arrow keys move it.

```
Splitter(bind: …, min: …, max: …)
```

```wf
state side = 240
Row {
    Container { style { width: {side}px } Text("Files") }
    Splitter(bind: side, min: 160, max: 480, reset: 240, label: "Sidebar width")
    Container { Text("Editor") }
}
```

Renders with the class `wf-splitter`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `min:` | `Number` | The smallest size, in px |
| `max:` | `Number` | The largest size, in px |
| `reset:` | `Number` | The size a double-click or Enter goes back to, taken as written: `0` can mean the region's default share |
| `label:` | `String` | What it resizes, for a screen reader |
| `horizontal:` | `Bool` | Between two regions one above the other: it moves up and down |
| `reverse:` | `Bool` | On the start side of what it sizes — a pane on the right, a panel below — so a drag toward the start grows it |

**Flags:** `.horizontal` `.reverse`

Attribute families: `global`, `aria`, `data`.

### Divider

Horizontal rule.

```
Divider
```

```wf
Text("Section one")
Divider
Text("Section two")
```

Renders with the class `wf-divider`.

Attribute families: `global`, `aria`, `data`.

### AppShell

An app's frame: `AppShell.Header` across the top, a `Sidebar` down the start (a drawer on a narrow screen), the `Router`'s page beside it, and an `AppShell.Footer`.

```
AppShell { … }
```

```wf
AppShell {
    AppShell.Header { Text("Ledger").bold  Spacer  ThemeToggle }
    Sidebar { Sidebar.Item(to: "/", icon: "home") { Text("Home") } }
    Router
}
```

Renders with the class `wf-app-shell`.

**Parts:**

- `AppShell.Header { … }` — The bar across the top.
- `AppShell.Footer { … }` — The bar along the bottom.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Host

A bare node handed to somebody else's code, with a lifetime. `mount:` runs with the node each time it is made, `update:` again whenever the state it reads changes, and `cleanup:` when the page, branch or list item it belongs to leaves. Every other element takes the same three.

```
Host(tag: …)
```

```wf
Host(tag: "canvas", mount: (node) => node.getContext("2d"), cleanup: (ctx) => log("gone"))
```

Renders with the class `wf-host`.

| Prop | Type | Meaning |
|---|---|---|
| `tag:` | `String` | The element to make: `div` (the default), `span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul` or `table` |

Attribute families: `global`, `aria`, `data`.

### Portal

Its block, drawn into a node somebody else's code owns: an element a script hands over, or a `ref:` handle. When `into:` names another node the block moves there with its nodes and state; `null` keeps it aside until a node comes back; it goes when the page, branch or list item it is written in leaves. What is inside belongs to where it is written: its handlers, `emit` and lifetimes.

```
Portal(into: …) { … }
```

```wf
Stack(ref: panel)
Portal(into: panel) { Text("Drawn inside the stack") }
```

Renders with the class `wf-portal`.

| Prop | Type | Meaning |
|---|---|---|
| `into:` (required) | `Any` | Where it draws: an element, a `ref:` handle, or `null` to keep it aside; read again whenever what it reads changes |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Element

A custom element a script defines, placed by its tag: `Element("stripe-pricing-table", publishableKey: key) { on ready { … } }`. Every named argument is an attribute, every `on` handler an event it fires, and the block its children.

```
Element(tag) { … }
```

```wf
Element("relative-time", datetime: "2026-03-14T09:30:00Z") { Text("March 14") }
Element("stripe-pricing-table", publishableKey: "pk_test_…") { on ready { log("prices shown") } }
```

| Prop | Type | Meaning |
|---|---|---|
| `tag` (positional) | `String` | The custom element's tag: lower case, with a hyphen, as every custom element's is |

Takes a block of children.

Attribute families: `any`.

## Navigation

### Navbar

Top navigation bar with brand, links and action areas.

```
Navbar { … }
```

```wf
Navbar {
    Navbar.Brand { Link("Acme", to: "/") }
    Navbar.Links { Link("Docs", to: "/docs")  Link("Pricing", to: "/pricing") }
    Navbar.Actions { Button("Sign in").sm }
}
```

Renders with the class `wf-navbar`.

**Parts:**

- `Navbar.Brand { … }` — The brand area, usually a link home.
- `Navbar.Links { … }` — The navigation links; collapses behind a toggle on small screens.
- `Navbar.Actions { … }` — Buttons at the end of the bar.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Sidebar

Side navigation. A `Sidebar.Item` whose `to` matches the current route is marked `.active` and `aria-current="page"`.

```
Sidebar { … }
```

```wf
Sidebar {
    Sidebar.Header { Text("Console").bold }
    Sidebar.Item(to: "/", icon: "home") { Text("Overview") }
    Sidebar.Divider
    Sidebar.Item(to: "/settings", icon: "settings") { Text("Settings") }
}
```

Renders with the class `wf-sidebar`.

**Parts:**

- `Sidebar.Header { … }` — The sidebar's heading area.
- `Sidebar.Item(to: …, icon: …, active: …) { … }` — One navigation entry.
  - `to:` `Path` — Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"`
  - `icon:` `String` — Icon shown before the label
  - `active:` `.prefix` — How the link decides it is the current one
  - flags: `.prefix`
- `Sidebar.Divider` — A rule between groups of items.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Link

Client-side navigation link; its label or its block is the link text.

```
Link(label, to: …, active: …) { … }
```

```wf
Link("About us", to: "/about")
Link("Guide", to: "/docs", active: .prefix)
Link(to: "https://example.com", target: "_blank") { Text("Example") }
```

Renders with the class `wf-link`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The link text |
| `to:` | `Path` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `active:` | `.prefix` | How the link decides it is the current one |

**Flags:** `.prefix`

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `anchor`.

### Tabs

Tab strip switching between its pages: the arrow keys move between the tabs (the other way round in a right-to-left page), Home and End go to the ends.

```
Tabs(bind: …, value: …, orientation: …) { … }
```

```wf
Tabs {
    Tabs.Page("Profile") { Text("Your details") }
    Tabs.Page("Billing") { Text("Your plan") }
}
```

Renders with the class `wf-tabs`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The open tab, read and written: a page's `value:`, or its place from 0 when the pages give none |
| `value:` | `Any` | The open tab, read only: `on change(v)` hears the one the reader picks |
| `orientation:` | `.horizontal` `.vertical` | Which way the tabs run |
| `activation:` | `.automatic` `.manual` | Whether moving the focus opens a tab |
| `label:` | `String` | The tab strip's accessible name |

**Flags:** `.horizontal` `.vertical` `.automatic` `.manual`

**Events of its own:** `on change`

**Parts:**

- `Tabs.Page(label, value: …, disabled: …) { … }` — One tab and its panel.
  - `label:` `String` — The tab's label
  - `value:` `Any` — What the Tabs' `bind:` holds while this tab is open
  - `disabled:` `Bool` — Shown, but not opened: the arrow keys pass over it
  - flags: `.disabled`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Breadcrumb

Breadcrumb trail. An item without `to` is the current page.

```
Breadcrumb { … }
```

```wf
Breadcrumb {
    Breadcrumb.Item(to: "/") { Text("Home") }
    Breadcrumb.Item(to: "/docs") { Text("Docs") }
    Breadcrumb.Item { Text("Components") }
}
```

Renders with the class `wf-breadcrumb`.

**Parts:**

- `Breadcrumb.Item(to: …) { … }` — One step of the trail.
  - `to:` `Path` — Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Menu

A menu above everything: opened by its `trigger:` button, or by `visible:` at an `anchor:` or a point `at:` — a context menu. Escape, a press outside or a choice close it, write `false` back to `visible:` and fire `on close`.

```
Menu(trigger: …, label: …, visible: …) { … }
```

```wf
Menu(trigger: "Options") {
    Menu.Item(shortcut: "F2") { Text("Rename") }
    Menu.Sub(label: "Move to") { Menu.Item { Text("Archive") } }
    Menu.Divider
    Menu.Item(disabled: true) { Text("Delete") }
}
state asked = false
state point = { x: 0, y: 0 }
Menu(visible: asked, at: point) { Menu.Item { Text("Copy") } }
```

Renders with the class `wf-menu`.

| Prop | Type | Meaning |
|---|---|---|
| `trigger:` | `String` | Label of the button that opens the menu |
| `label:` | `String` | Same as `trigger` |
| `visible:` | `State` | State flag that opens and closes it |
| `anchor:` | `Any` | What it is placed against: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }` |
| `at:` | `Any` | A point to place it at, `{ x: e.clientX, y: e.clientY }`: where the pointer asked for a context menu |
| `placement:` | `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room. Unset, `.bottomStart` |
| `offset:` | `Number` | How far from the anchor, in px |

**Flags:** `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd`

**Events of its own:** `on close`

**Parts:**

- `Menu.Item(shortcut: …, checked: …, disabled: …, danger: …) { … }` — One entry of the menu; `on click` is what choosing it does.
  - `shortcut:` `String` — The keys that do the same, `mod+b`, shown at its end as this system writes them (`⌘B`, `Ctrl+B`)
  - `checked:` `Bool` — A check mark: the item turns something on and off
  - `disabled:` `Bool` — Shown, skipped by the arrow keys, and cannot be chosen
  - `danger:` `Bool` — What it does cannot be taken back — Delete, Discard: drawn in the danger colour
  - flags: `.checked` `.disabled` `.danger`
- `Menu.Sub(label: …) { … }` — An entry that opens a menu of its own beside it.
  - `label:` `String` — What the entry says
- `Menu.Divider` — A rule between groups of entries.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### OnThisPage

The page's headings as links, the one being read marked as the reader scrolls: a table of contents beside a long page.

```
OnThisPage(levels: …, label: …)
```

```wf
OnThisPage(levels: 2)
```

Renders with the class `wf-on-this-page`.

| Prop | Type | Meaning |
|---|---|---|
| `levels:` | `Number` | How many levels below the title it lists: `2` is `h2` and `h3` (the default 1, `h2` alone) |
| `label:` | `String` | Its title and name; unset, `On this page` |

**Parts:**

- `OnThisPage.Title(text)` — Its title. Drawn for it.
  - `text:` `String` — The title

Attribute families: `global`, `aria`, `data`.

### Pagination

Pages of a list: Previous, the first and last pages, those around the current one with a gap for the rest, and Next. The current page is `aria-current="page"`; `bind:` holds it, from 1.

```
Pagination(bind: …, pages: …, siblings: …)
```

```wf
state page = 5
Pagination(bind: page, pages: 12, siblings: 1)
```

Renders with the class `wf-pagination`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The current page, from 1 |
| `pages:` (required) | `Number` | How many pages there are |
| `siblings:` | `Number` | How many pages either side of the current one it shows (unset, 1) |
| `label:` | `String` | Its name for a screen reader; unset, `Pagination` |

**Parts:**

- `Pagination.Prev(disabled: …) { … }` — The previous page. Drawn for it.
  - `disabled:` `Bool` — Whether the control is inert
  - flags: `.disabled`
- `Pagination.Next(disabled: …) { … }` — The next page. Drawn for it.
  - `disabled:` `Bool` — Whether the control is inert
  - flags: `.disabled`
- `Pagination.Page(text) { … }` — A page. Drawn for it.
  - `text:` `String` — Its number
- `Pagination.Gap(text) { … }` — The pages left out. Drawn for it.
  - `text:` `String` — An ellipsis

Attribute families: `global`, `aria`, `data`.

### Stepper

Steps taken in order — an account, then billing, then a summary — with each step's state (done, current, an error) and the current step's block shown. `bind:` holds the current step, from 0; `.linear` keeps a step out of reach until the ones before it are passed.

```
Stepper(bind: …, linear: …) { … }
```

```wf
state step = 0
Stepper(bind: step) {
    Stepper.Step("Account") { Text("Your name and email.") }
    Stepper.Step("Billing", done: step > 1) { Text("A card on file.") }
    Stepper.Step("Confirm") { Text("Check and send.") }
}
```

Renders with the class `wf-stepper`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The current step, from 0 |
| `linear:` | `Bool` | A step ahead of the current one cannot be pressed |

**Flags:** `.linear`

**Parts:**

- `Stepper.Step(title, done: …, error: …) { … }` — One step: its title in the list, its block shown while it is the current one.
  - `title:` `String` — Its title
  - `done:` `Bool` — Passed: drawn with a check
  - `error:` `Bool` — Something in it needs fixing
  - flags: `.done` `.error`
- `Stepper.List { … }` — The steps, in order. Drawn for it.
- `Stepper.Item { … }` — A step in the list. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### CommandPalette

A dialog of everything a reader can do or go to, filtered as they type: the arrow keys go through what matches, Enter runs it (`on select`) and closes. `open:` is the state that shows it.

```
CommandPalette(open: …, label: …, placeholder: …) { … }
```

```wf
state palette = false
Button("Commands") { on click { palette = true } }
CommandPalette(open: palette) {
    CommandPalette.Group("Pages") {
        CommandPalette.Item("Home", icon: "home") { on select { navigate("/") } }
    }
    CommandPalette.Group("Actions") {
        CommandPalette.Item("Toggle theme", shortcut: "mod+shift+l") { on select { setTheme(if theme == "dark" { "light" } else { "dark" }) } }
    }
}
```

Renders with the class `wf-command-palette`.

| Prop | Type | Meaning |
|---|---|---|
| `open:` (required) | `State` | The state that shows it, written back when it closes |
| `label:` | `String` | Its name and the field's; unset, `Commands` |
| `placeholder:` | `String` | Shown in the field; unset, `Type a command or search…` |

**Parts:**

- `CommandPalette.Group(title) { … }` — Items under a heading.
  - `title:` `String` — Its heading
- `CommandPalette.Item(label, icon: …, shortcut: …) { … }` — One thing to do or go to; `on select` is what it does.
  - `label:` `String` — Its words, which the typing matches
  - `icon:` `String` — An icon before the words
  - `shortcut:` `String` — Its keys, shown at its end as this system writes them
- `CommandPalette.List { … }` — What matches. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Data display

### Card

Card surface with optional header, body and footer sections.

```
Card(surface: …) { … }
```

```wf
Card.elevated {
    Card.Header { Heading("Pro").h3 }
    Card.Body { Text("Everything in Free, and more.") }
    Card.Footer { Button("Choose Pro").primary }
}
```

Renders with the class `wf-card`.

| Prop | Type | Meaning |
|---|---|---|
| `surface:` | `.flat` `.elevated` `.outlined` | Surface treatment |

**Flags:** `.flat` `.elevated` `.outlined`

**Parts:**

- `Card.Header { … }` — The card's heading area.
- `Card.Body { … }` — The card's content area.
- `Card.Footer { … }` — The card's action area.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Table

Data table. Cells inside `Table.Head` render as `<th scope="col">`; `caption` is the table's accessible name, visually hidden.

```
Table(caption: …) { … }
```

```wf
Table(caption: "Deploys") {
    Table.Head { Table.Row { Table.Cell("Region")  Table.Cell("Status") } }
    Table.Body { Table.Row { Table.Cell("fra1")  Table.Cell("Healthy") } }
}
```

Renders with the class `wf-table`.

| Prop | Type | Meaning |
|---|---|---|
| `caption:` | `String` | Accessible name of the table (rendered visually hidden) |

**Parts:**

- `Table.Head { … }` — Header section; its cells become `<th scope="col">`.
- `Table.Body { … }` — The body of a table: its rows of data.
- `Table.Row { … }` — A row of cells.
- `Table.Cell(content, header: …) { … }` — A cell. `header` makes it a `<th>` anywhere, as `Table.Head` does for its cells.
  - `content:` `Any` — The cell's content
  - `header:` `Bool` — Render as a `<th>`
  - flags: `.header`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### List

List of its children; `ordered` for a numbered list.

```
List(ordered: …) { … }
```

```wf
List {
    List.Item { Text("Milk") }
    List.Item { Text("Eggs") }
}
List.ordered { List.Item { Text("First") } }
```

Renders with the class `wf-list`.

| Prop | Type | Meaning |
|---|---|---|
| `ordered:` | `Bool` | Numbered (`<ol>`) |

**Flags:** `.ordered`

**Parts:**

- `List.Item { … }` — One entry (`<li>`).

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Badge

Small status label.

```
Badge(label, tone: …, pill: …)
```

```wf
Badge("Beta").info
Badge("3 new").danger.pill
```

Renders with the class `wf-badge`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The badge text |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Colour |
| `pill:` | `Bool` | Fully rounded ends |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.pill`

Attribute families: `global`, `aria`, `data`.

### Tag

Compact chip.

```
Tag(label)
```

```wf
Tag("design")
```

Renders with the class `wf-tag`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The tag text |

Attribute families: `global`, `aria`, `data`.

### Avatar

A picture of a person, or their initials.

```
Avatar(src: …, alt: …, initials: …)
```

```wf
Avatar(src: "/ada.jpg", alt: "Ada Lovelace")
Avatar(initials: "AL").primary.lg
```

Renders with the class `wf-avatar`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Image URL |
| `alt:` | `String` | Text alternative for the image |
| `initials:` | `String` | Letters shown when there is no image |
| `size:` | `.sm` `.md` `.lg` | Its size |
| `tone:` | `.primary` | Colour of the initials |

**Flags:** `.sm` `.md` `.lg` `.primary`

Attribute families: `global`, `aria`, `data`.

### Tooltip

Shows `text` above everything when its child is hovered (after `delay:`) or focused from the keyboard.

```
Tooltip(text, shortcut: …, placement: …, delay: …) { … }
```

```wf
Tooltip("Copies the link") { IconButton(icon: "copy", label: "Copy link") }
Tooltip("Save", shortcut: "Ctrl+S", delay: 400, placement: .bottom) { Button("Save") }
```

Renders with the class `wf-tooltip`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The tooltip text |
| `shortcut:` | `String` | The keys that do the same, `mod+s`, shown as a chip as this system writes them |
| `placement:` | `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room. Unset, `.bottomStart` |
| `delay:` | `Number` | How long a hover waits before it shows, in ms. Unset, 500 |

**Flags:** `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Tree

Nested items — files and folders, an outline — as a tree a keyboard can work: Up and Down through what shows, Right to open and go in, Left to close and go out (the other way round right to left), Home and End, a letter. `row(item) { … }` draws each; `bind:` holds the chosen item's key, or a list of keys with `.multiple`.

```
Tree(items: …, children: …, key: …)
```

```wf
state files = [{ path: "src", name: "src", children: [{ path: "src/App.wf", name: "App.wf", children: [] }] }]
state file = ""
Tree(items: files, children: f => f.children, key: f => f.path, bind: file, label: "Files") {
    row(f) { Icon(if f.children.length > 0 { "folder" } else { "file" })  Text(f.name) }
}
```

Renders with the class `wf-tree`.

| Prop | Type | Meaning |
|---|---|---|
| `items:` (required) | `Any` | The items at the top |
| `children:` (required) | `Any` | An item's children: `f => f.children`; none, or an empty list, for a leaf |
| `key:` (required) | `Any` | What tells two items apart: `f => f.path` |
| `bind:` | `State` | The chosen key, or a list of them with `.multiple` |
| `multiple:` | `Bool` | Several at once: Space, Shift with an arrow, Ctrl or ⌘ with A |
| `open:` | `Any` | The keys open when it is drawn |
| `label:` (required) | `String` | What the tree is, for a screen reader |

**Flags:** `.multiple`

Attribute families: `global`, `aria`, `data`.

### DataTable

Rows of records with their columns declared once: a column sorts when it is `.sortable` (its header a button, `aria-sort` saying which way), `cell(r) { … }` draws a cell, `empty { … }` shows when there are no rows, and `bind:` chooses rows (a list with `.multiple`, a box for each and one for all).

```
DataTable(rows: …, key: …, bind: …) { … }
```

```wf
state deploys = [{ id: "1", service: "web", status: "live" }, { id: "2", service: "api", status: "failed" }]
state chosen: [String] = []
DataTable(rows: deploys, key: d => d.id, bind: chosen, label: "Deploys").multiple {
    empty { EmptyState(icon: "list", title: "No deploys yet") }
    DataTable.Column("Service", value: d => d.service).sortable
    DataTable.Column("Status") { cell(d) { Badge(d.status) } }
}
```

Renders with the class `wf-data-table`.

| Prop | Type | Meaning |
|---|---|---|
| `rows:` (required) | `Any` | The records |
| `key:` (required) | `Any` | What tells two rows apart: `r => r.id` |
| `bind:` | `State` | The chosen row's key, or a list of them with `.multiple` |
| `multiple:` | `Bool` | Several rows at once |
| `label:` (required) | `String` | What the table is: its caption, read by a screen reader |

**Flags:** `.multiple`

**Parts:**

- `DataTable.Column(title, value: …, sortable: …)` — One column: its title, its `value:` (what it shows and sorts by), and a `cell(r) { … }` that draws it instead.
  - `title:` `String` — Its header
  - `value:` `Any` — What the column holds for a row: `r => r.service`
  - `sortable:` `Bool` — Its header sorts the rows by its `value:`
  - flags: `.sortable`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Chart

A chart drawn as vector graphics when the page is built: bars, lines, areas, a pie or a donut over a list of records.

```
Chart(kind: …, data: …, x: …)
```

```wf
Chart(kind: .bar, data: [{ month: "Jan", web: 12 }, { month: "Feb", web: 18 }], x: "month", y: "web", unit: "k")
```

Renders with the class `wf-chart`.

| Prop | Type | Meaning |
|---|---|---|
| `kind:` | `.bar` `.line` `.area` `.pie` `.donut` | What it draws |
| `data:` | `Any` | The rows: a list of records, or of numbers |
| `x:` | `String` | The key of each row's label |
| `y:` | `Any` | The key of each row's value, or a list of keys for several series |
| `colors:` | `Any` | The series' colours, in order |
| `width:` | `Number` | Its width in user units (it scales to its box) |
| `height:` | `Number` | Its height in user units |
| `ink:` | `String` | The colour of its labels |
| `grid:` | `String` | The colour of its grid lines |
| `legend:` | `Bool` | Name the series |
| `stacked:` | `Bool` | Stack a bar chart's series |
| `labels:` | `Bool` | Write each bar's or slice's value on it |
| `unit:` | `String` | A unit after each value on the axis: `ms`, `%` |

**Flags:** `.bar` `.line` `.area` `.pie` `.donut` `.legend` `.stacked` `.labels`

Attribute families: `global`, `aria`, `data`.

### QrCode

A QR code for a URL or any text, drawn as vector graphics when the page is built.

```
QrCode(value, color: …, background: …)
```

```wf
QrCode("https://example.com/report/q1")
```

Renders with the class `wf-qr`.

| Prop | Type | Meaning |
|---|---|---|
| `value` (positional) | `String` | What it encodes |
| `color:` | `String` | The colour of the dark modules |
| `background:` | `String` | The colour behind them |

Attribute families: `global`, `aria`, `data`.

## Form

### Input

Text field. With `label`, `hint` or `error` it is a labelled field.

```
Input(bind: …, label: …, hint: …)
```

```wf
state email = ""
Input(bind: email, label: "Email", hint: "We never share it").email.required
```

Renders with the class `wf-input`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `type:` | `.text` `.email` `.password` `.number` `.search` `.tel` `.url` `.date` `.time` `.datetime` `.color` | What the field holds |
| `placeholder:` | `String` | Hint shown while the field is empty |
| `value:` | `Any` | The control's value, when its state is not bound |
| `min:` | `Any` | Lowest value |
| `max:` | `Any` | Highest value |
| `step:` | `Number` | Granularity |
| `required:` | `Bool` | Whether the form refuses to submit without it |
| `disabled:` | `Bool` | Whether the control is inert |
| `size:` | `.sm` `.md` `.lg` | Its size |
| `width:` | `.full` | Width |
| `rounded:` | `Bool` | Rounded corners |

**Flags:** `.text` `.email` `.password` `.number` `.search` `.tel` `.url` `.date` `.time` `.datetime` `.color` `.required` `.disabled` `.sm` `.md` `.lg` `.full` `.rounded`

Attribute families: `global`, `aria`, `data`, `input`.

### Select

Drop-down of `Select.Option` children.

```
Select(bind: …, value: …, label: …) { … }
```

```wf
state plan = "free"
Select(bind: plan, label: "Plan") {
    Select.Option("Free", value: "free")
    Select.Option("Team", value: "team")
}
```

Renders with the class `wf-select`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `value:` | `Any` | The control's value, when its state is not bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `disabled:` | `Bool` | Whether the control is inert |
| `multiple:` | `Bool` | Allow several choices |

**Flags:** `.disabled` `.multiple`

**Parts:**

- `Select.Option(label, value: …)` — One choice: the visible label, and the value sent with the form.
  - `label:` `String` — The visible label
  - `value:` `Any` — The value `bind` takes when this one is chosen; the label when absent

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `input`.

### Checkbox

Checkbox bound to a boolean state.

```
Checkbox(bind: …, checked: …, label: …)
```

```wf
state agree = false
Checkbox(bind: agree, label: "I agree to the terms")
```

Renders with the class `wf-checkbox`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `checked:` | `Bool` | Whether it is checked, when its state is not bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.checked` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Radio

One choice of a group; every radio bound to the same state is one group.

```
Radio(bind: …, value: …, checked: …)
```

```wf
state size = "m"
Radio(bind: size, value: "s", label: "Small")
Radio(bind: size, value: "m", label: "Medium")
```

Renders with the class `wf-radio`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `value:` | `Any` | The value `bind` takes when this one is chosen |
| `checked:` | `Bool` | Whether it is checked, when its state is not bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.checked` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Switch

On/off toggle bound to a boolean state.

```
Switch(bind: …, checked: …, label: …)
```

```wf
state notify = true
Switch(bind: notify, label: "Email me about replies")
```

Renders with the class `wf-switch`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `checked:` | `Bool` | Whether it is checked, when its state is not bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.checked` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Slider

Range input. `aria-*` and its other attributes reach the input; with `aria-valuetext` the raw number is not shown beside the track.

```
Slider(bind: …, min: …, max: …)
```

```wf
state volume = 50
Slider(bind: volume, min: 0, max: 100, step: 5, label: "Volume")
```

Renders with the class `wf-slider`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `min:` | `Number` | Lowest value |
| `max:` | `Number` | Highest value |
| `step:` | `Number` | Granularity |
| `label:` | `String` | Visible label; also the control's accessible name |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### DatePicker

A date field: the reader picks a day, and the bound state holds a `Date`.

```
DatePicker(bind: …, label: …, min: …)
```

```wf
state start: Date? = null
DatePicker(bind: start, label: "Start date", min: "2026-01-01")
```

Renders with the class `wf-datepicker`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `min:` | `String` | Earliest date, `YYYY-MM-DD` |
| `max:` | `String` | Latest date, `YYYY-MM-DD` |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### FileUpload

File input; handle the chosen files in `on change`.

```
FileUpload(accept: …, label: …, multiple: …) { … }
```

```wf
FileUpload(accept: "image/*", label: "Choose a picture") { on change(e) { log(e.target.files[0].name) } }
```

Renders with the class `wf-file-upload`.

| Prop | Type | Meaning |
|---|---|---|
| `accept:` | `String` | Accepted types, e.g. `image/*` or `.pdf,.doc` |
| `label:` | `String` | Visible label; also the control's accessible name |
| `multiple:` | `Bool` | Accept several files |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.multiple` `.disabled`

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `input`.

### Form

Groups controls; `on submit` runs when it is submitted, and the page never navigates away.

```
Form(bind: …, show: …) { … }
```

```wf
state name = ""
Form(bind: form) {
    on submit { log(name) }
    Input(bind: name, label: "Name").required
    Button("Save", type: .submit, disabled: !form.valid).primary
}
```

Renders with the class `wf-form`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A handle on the form: `form.valid`, `form.errors`, `form.values` by field name, `form.reset()`, `form.apply(serverErrors)` |
| `show:` | `.onBlur` `.onSubmit` `.live` | When a `validate` block's message is shown |

**Flags:** `.onBlur` `.onSubmit` `.live`

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `form`.

### Textarea

Several lines of text. With `label`, `hint` or `error` it is a labelled field; with `maxLength` it counts what is left.

```
Textarea(bind: …, label: …, hint: …)
```

```wf
state note = ""
Textarea(bind: note, label: "Notes", rows: 4, maxLength: 200)
```

Renders with the class `wf-input wf-textarea`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `placeholder:` | `String` | Hint shown while the field is empty |
| `rows:` | `Number` | How many lines it shows |
| `maxLength:` | `Number` | The most characters it takes |
| `required:` | `Bool` | Whether the form refuses to submit without it |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.required` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Field

A label, a hint and an error around a control of your own: the first control in its block is named by the label and described by the hint and the error, and marked invalid while there is one.

```
Field(label: …, hint: …, error: …) { … }
```

```wf
state plan = "basic"
Field(label: "Plan", hint: "You can change it later.") {
    Select(bind: plan) { Select.Option("Basic", value: "basic")  Select.Option("Pro", value: "pro") }
}
```

Renders with the class `wf-field wf-field--custom`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |

**Parts:**

- `Field.Label(text) { … }` — Its label. Drawn for it.
  - `text:` `String` — The label
- `Field.Hint(text) { … }` — Its hint. Drawn for it.
  - `text:` `String` — The hint
- `Field.Error(text) { … }` — Its error, announced as it appears. Drawn for it.
  - `text:` `String` — The error

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Fieldset

Controls that belong together — an address, a choice of plan — under a legend that names them.

```
Fieldset(legend: …) { … }
```

```wf
state street = ""
state city = ""
Fieldset(legend: "Address") {
    Input(bind: street, label: "Street")
    Input(bind: city, label: "City")
}
```

Renders with the class `wf-fieldset`.

| Prop | Type | Meaning |
|---|---|---|
| `legend:` (required) | `String` | What the group is |

**Parts:**

- `Fieldset.Legend(text) { … }` — Its legend. Drawn for it.
  - `text:` `String` — The legend

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### InputGroup

An input drawn as one box with what goes beside it: an icon, `InputGroup.Addon("https://")`, a button, or `InputGroup.Clear`, which empties the input and gives it the focus back.

```
InputGroup { … }
```

```wf
state q = ""
InputGroup {
    Icon("search")
    Input(bind: q, aria-label: "Search")
    InputGroup.Clear
}
state site = ""
InputGroup { InputGroup.Addon("https://")  Input(bind: site, aria-label: "Site") }
```

Renders with the class `wf-input-group`.

**Parts:**

- `InputGroup.Addon(text) { … }` — Text beside the input: `https://`, `.com`, `kg`.
  - `text:` `String` — The text
- `InputGroup.Clear` — A button that empties the input and puts the focus back in it; hidden while it is empty.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### RadioGroup

One choice of several, as native radio buttons under a legend: the arrow keys move between them, and `.cards` draws each as a card with its hint.

```
RadioGroup(bind: …, label: …, cards: …) { … }
```

```wf
state plan = "free"
RadioGroup(bind: plan, label: "Plan").cards {
    RadioGroup.Option("Free", value: "free", hint: "For trying it out")
    RadioGroup.Option("Team", value: "team", hint: "€8 a seat")
}
```

Renders with the class `wf-radio-group`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The chosen value |
| `label:` (required) | `String` | What is being chosen: the group's legend |
| `cards:` | `Bool` | Each option a card, its hint beneath its words |

**Flags:** `.cards`

**Parts:**

- `RadioGroup.Option(label, value: …, hint: …) { … }` — One choice: its words, its value, and a hint said with it.
  - `label:` `String` — Its words
  - `value:` `Any` — What `bind:` holds when it is chosen
  - `hint:` `String` — A line beneath it, read with it
- `RadioGroup.Legend(text) { … }` — Its legend. Drawn for it.
  - `text:` `String` — The legend
- `RadioGroup.Hint(text) { … }` — An option's hint. Drawn for it.
  - `text:` `String` — The hint

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### NumberInput

A number with − and + beside it: the arrow keys step it, PageUp and PageDown by ten steps, and it stays between `min:` and `max:`.

```
NumberInput(bind: …, min: …, max: …)
```

```wf
state qty = 1
NumberInput(bind: qty, min: 1, max: 99, label: "Quantity")
```

Renders with the class `wf-number-input`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The `Number` it holds |
| `min:` | `Number` | The lowest it goes |
| `max:` | `Number` | The highest it goes |
| `step:` | `Number` | How far a press moves it (unset, 1) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |

**Parts:**

- `NumberInput.Decrease(disabled: …) { … }` — Its − button. Drawn for it.
  - `disabled:` `Bool` — Whether the control is inert
  - flags: `.disabled`
- `NumberInput.Increase(disabled: …) { … }` — Its + button. Drawn for it.
  - `disabled:` `Bool` — Whether the control is inert
  - flags: `.disabled`

Attribute families: `global`, `aria`, `data`.

### Calendar

A month as a grid of days, chosen with a press or the keyboard — the arrows by day and week, PageUp and PageDown by month, Home and End to the week's ends — one day, a range, or several. `bind:` holds a `Date`, a `{ start, end }` with `.range`, a list with `.multiple`.

```
Calendar(bind: …, mode: …, min: …)
```

```wf
state day: Date? = null
Calendar(bind: day, min: @2026-01-01, disabled: d => d.weekday() >= 6, label: "Delivery day")
```

Renders with the class `wf-calendar`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The chosen day, range or days |
| `mode:` | `.single` `.range` `.multiple` | How many days it holds |
| `min:` | `Any` | The first day that can be chosen |
| `max:` | `Any` | The last day that can be chosen |
| `disabled:` | `Any` | Days that cannot be chosen: a list of them, or `d => d.weekday() >= 6` |
| `label:` | `String` | Its name for a screen reader; unset, `Calendar` |

**Flags:** `.single` `.range` `.multiple`

Attribute families: `global`, `aria`, `data`.

### Listbox

A list of options always shown, one chosen (selection follows the arrow keys) or, `.multiple`, several: Space chooses, Shift with an arrow takes the next one in too, Ctrl or ⌘ with A takes them all; a letter goes to the next option that starts with it.

```
Listbox(bind: …, label: …, multiple: …) { … }
```

```wf
state fruit = "apple"
Listbox(bind: fruit, label: "Fruit") {
    Listbox.Option("Apple", value: "apple")
    Listbox.Option("Banana", value: "banana")
    Listbox.Option("Cherry", value: "cherry")
}
```

Renders with the class `wf-listbox`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The chosen value, or a list of them with `.multiple` |
| `label:` (required) | `String` | What is being chosen, for a screen reader |
| `multiple:` | `Bool` | Several at once: `bind:` is a list |

**Flags:** `.multiple`

**Parts:**

- `Listbox.Option(label, value: …, icon: …, aria-disabled: …) { … }` — One option: its words, and the value `bind:` holds when it is chosen.
  - `label:` `String` — Its words
  - `value:` `Any` — What `bind:` holds when it is chosen
  - `icon:` `String` — An icon before the words
  - `aria-disabled:` `String` — `"true"`: shown, passed over, never chosen

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Combobox

A text field that offers what matches as the reader types: from `options:`, or `search:` asked of a server. The arrow keys go through the offers, Enter takes one, Escape closes them; `.multiple` keeps several, each a chip Backspace takes back. `option(o) { … }` draws an offer; `empty { … }` shows when nothing matches.

```
Combobox(bind: …, options: …, search: …)
```

```wf
state city = ""
state cities = [{ id: "par", name: "Paris", country: "France" }, { id: "por", name: "Porto", country: "Portugal" }]
Combobox(bind: city, options: cities, text: c => c.name, key: c => c.id, label: "City") {
    option(c) { Text(c.name)  Text(c.country).muted }
    empty { Text("No city by that name") }
}
```

Renders with the class `wf-combobox`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The chosen option's key (or the option itself without `key:`), or a list with `.multiple` |
| `options:` | `Any` | Every option, filtered as the reader types |
| `search:` | `Any` | Options asked for as the reader types: `q => Geo.cities(q: q)` |
| `text:` (required) | `Any` | An option's words: `p => p.name` |
| `key:` | `Any` | What tells two options apart: `p => p.id` |
| `debounce:` | `Any` | How long the typing rests before `search:` is asked (unset, 200ms) |
| `multiple:` | `Bool` | Several at once, each a chip |
| `label:` (required) | `String` | Its name for a screen reader |
| `placeholder:` | `String` | Shown while it is empty |

**Flags:** `.multiple`

Attribute families: `global`, `aria`, `data`.

## Feedback

### Alert

Inline notice; a `danger` or `warning` one is announced as an alert.

```
Alert(message, tone: …) { … }
```

```wf
Alert("Your changes were saved.").success
Alert("The server is not answering.").danger
```

Renders with the class `wf-alert`.

| Prop | Type | Meaning |
|---|---|---|
| `message` (positional) | `String` | The message |
| `tone:` | `.success` `.danger` `.warning` `.info` | Kind of notice |

**Flags:** `.success` `.danger` `.warning` `.info`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Toast

Temporary notification.

```
Toast(message, tone: …)
```

```wf
state saved = false
if saved { Toast("Saved").success }
```

Renders with the class `wf-toast`.

| Prop | Type | Meaning |
|---|---|---|
| `message` (positional) | `String` | The message |
| `tone:` | `.success` `.danger` `.warning` `.info` | Kind of notice |

**Flags:** `.success` `.danger` `.warning` `.info`

Attribute families: `global`, `aria`, `data`.

### Modal

Modal `<dialog>`, opened by a state flag; the browser supplies focus trapping and Escape. `on close` runs when it closes, however it was closed, and the focus goes back to what had it.

```
Modal(visible: …, title: …, closable: …) { … }
```

```wf
state open = false
Button("Delete") { on click { open = true } }
Modal(visible: open, title: "Delete deploys.wf?").closable {
    Modal.Title { Markdown("Delete `deploys.wf`?") }
    Text("This cannot be undone.")
    Modal.Footer { Button("Cancel") { on click { open = false } } }
}
```

Renders with the class `wf-modal`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `closable:` | `Bool` | A close button in the heading row, which closes it as Escape does |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button |
| `initialFocus:` | `Any` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

**Flags:** `.closable` `.any` `.escape` `.none`

**Events of its own:** `on close`

**Parts:**

- `Modal.Title { … }` — What the modal's heading draws, in place of its `title:`'s words, which stay its name: `Modal.Title { Text { "Rename "  Code(name) } }`.
- `Modal.Footer { … }` — The modal's action row.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Popover

A layer at a control: drawn above everything, placed against its `anchor:`, closed by `Escape` or a press outside, which write `false` back to `visible:` and fire `on close`.

```
Popover(visible: …, anchor: …, at: …) { … }
```

```wf
state open = false
Button("Branches", ref: branches) { on click { open = !open } }
Popover(visible: open, anchor: branches, placement: .bottomStart, label: "Branches").arrow {
    on close(reason) { log(reason) }
    Text("main")
}
```

Renders with the class `wf-popover`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `anchor:` | `Any` | What it is placed against: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }` |
| `at:` | `Any` | A point to place it at, `{ x: e.clientX, y: e.clientY }`: where the pointer asked for a context menu |
| `placement:` | `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room. Unset, `.bottomStart` |
| `offset:` | `Number` | How far from the anchor, in px |
| `label:` | `String` | What it is, for a screen reader: its accessible name |
| `arrow:` | `Bool` | A point toward the anchor |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button |
| `initialFocus:` | `Any` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

**Flags:** `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` `.arrow` `.any` `.escape` `.none`

**Events of its own:** `on close`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Dialog

Confirmation `<dialog>`, opened by a state flag. `on close` runs when it closes, however it was closed, and the focus goes back to what had it.

```
Dialog(visible: …, title: …, closable: …) { … }
```

```wf
state asking = false
Dialog(visible: asking, title: "Leave without saving notes.md?") {
    Dialog.Title { Markdown("Leave without saving `notes.md`?") }
    Text("Your draft will be lost.")
}
```

Renders with the class `wf-dialog`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `closable:` | `Bool` | A close button in the heading row, which closes it as Escape does |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button |
| `initialFocus:` | `Any` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

**Flags:** `.closable` `.any` `.escape` `.none`

**Events of its own:** `on close`

**Parts:**

- `Dialog.Title { … }` — What the dialog's heading draws, in place of its `title:`'s words, which stay its name.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### AlertDialog

A `<dialog>` that asks before something that cannot be undone: `role="alertdialog"`, its body read out with its title, and closed by Escape or its own buttons, not a press outside. For a question asked from a handler, `await ask(…)` draws one.

```
AlertDialog(visible: …, title: …, dismiss: …) { … }
```

```wf
state sure = false
Button("Discard") { on click { sure = true } }
AlertDialog(visible: sure, title: "Discard your changes?") {
    Text("Your edits will be lost.")
    AlertDialog.Footer {
        Button("Keep editing") { on click { sure = false } }
        Button("Discard").danger { on click { sure = false } }
    }
}
```

Renders with the class `wf-dialog wf-alert-dialog`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button |
| `initialFocus:` | `Any` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

**Flags:** `.any` `.escape` `.none`

**Events of its own:** `on close`

**Parts:**

- `AlertDialog.Title { … }` — What the alert dialog's heading draws, in place of its `title:`'s words, which stay its name.
- `AlertDialog.Footer { … }` — Its answers: the button that loses nothing first.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Sheet

A modal `<dialog>` that comes in from an edge: the bottom (a phone's filters, a share list) or the start or end of the line. Dragged by its handle or heading toward its edge it closes; a bottom sheet with `snap:` points settles at the nearest.

```
Sheet(visible: …, title: …, closable: …) { … }
```

```wf
state filters = false
state sort = "new"
Button("Filters") { on click { filters = true } }
Sheet(visible: filters, title: "Filters", snap: [0.5, 1]).closable {
    Radio(bind: sort, value: "new", label: "Newest")
    Radio(bind: sort, value: "price", label: "Price")
    Sheet.Footer { Button("Show results").primary { on click { filters = false } } }
}
state details = false
Sheet(visible: details, side: .end, title: "Details") { Text("Opened 3 days ago") }
```

Renders with the class `wf-sheet`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `closable:` | `Bool` | A close button in the heading row, which closes it as Escape does |
| `side:` | `.bottom` `.start` `.end` | The edge it comes in from |
| `snap:` | `Any` | A bottom sheet's heights, as fractions of the window's, `[0.5, 1]`: it opens at the first |
| `label:` | `String` | Its accessible name, when it has no `title:` |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button |
| `initialFocus:` | `Any` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

**Flags:** `.closable` `.bottom` `.start` `.end` `.any` `.escape` `.none`

**Events of its own:** `on close`

**Parts:**

- `Sheet.Title { … }` — What the sheet's heading draws, in place of its `title:`'s words, which stay its name.
- `Sheet.Footer { … }` — The sheet's action row, kept in view below what scrolls.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Spinner

Loading indicator, announced as a status.

```
Spinner(size: …)
```

```wf
Spinner
Spinner.lg
```

Renders with the class `wf-spinner`.

| Prop | Type | Meaning |
|---|---|---|
| `size:` | `.sm` `.md` `.lg` | Its size |

**Flags:** `.sm` `.md` `.lg`

Attribute families: `global`, `aria`, `data`.

### Progress

Progress bar; a `value` that reads state moves with it.

```
Progress(value: …, max: …)
```

```wf
Progress(value: 40, max: 100)
```

Renders with the class `wf-progress`.

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | Current value |
| `max:` | `Number` | Value at 100% |

Attribute families: `global`, `aria`, `data`.

### Skeleton

Placeholder shape shown while content loads.

```
Skeleton(height: …, width: …, size: …)
```

```wf
Skeleton(height: "20px", width: "200px")
Skeleton(size: "48px").circle
```

Renders with the class `wf-skeleton`.

| Prop | Type | Meaning |
|---|---|---|
| `height:` | `String` | CSS height |
| `width:` | `String` | CSS width |
| `size:` | `String` | Diameter of a `circle` |
| `circle:` | `Bool` | A round placeholder |

**Flags:** `.circle`

Attribute families: `global`, `aria`, `data`.

## Content

### Callout

A note set apart from the text around it — `.note`, `.tip`, `.important`, `.warning`, `.caution` — with an icon and a title, as GitHub's alerts are. Its text is the positional, or its block.

```
Callout(text, kind: …, title: …) { … }
```

```wf
Callout("Run `wf check` before you push.").tip
Callout("Deleting a project deletes its builds too.", title: "Before you delete").caution
```

Renders with the class `wf-callout`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | What it says |
| `kind:` | `.note` `.tip` `.important` `.warning` `.caution` | What kind of note it is |
| `title:` | `String` | Its title; unset, the kind's word (`Tip`) |

**Flags:** `.note` `.tip` `.important` `.warning` `.caution`

**Parts:**

- `Callout.Title { … }` — Its title row: the icon and the words. Drawn for it; written by hand to draw something else there.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Kbd

A key or a combination, written as this system writes it: `Kbd("mod+k")` is `⌘K` on a Mac and `Ctrl+K` elsewhere.

```
Kbd(keys)
```

```wf
Text { Text("Press ")  Kbd("mod+k")  Text(" to search") }
```

Renders with the class `wf-kbd`.

| Prop | Type | Meaning |
|---|---|---|
| `keys` (positional) | `String` | The combination, as `on key` names it: `mod+k`, `shift+Enter` |

Attribute families: `global`, `aria`, `data`.

### EmptyState

What a list or a page shows when it has nothing yet: an icon, a title, a sentence and, in its block, what to do about it.

```
EmptyState(icon: …, title: …, text: …) { … }
```

```wf
EmptyState(icon: "folder", title: "No projects yet", text: "Create one to get started.") {
    Button("New project").primary
}
```

Renders with the class `wf-empty-state`.

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String` | An icon's name, drawn large above the title |
| `title:` (required) | `String` | What is missing: `No projects yet` |
| `text:` | `String` | A sentence on what to do about it |

**Parts:**

- `EmptyState.Title(text)` — Its title.
  - `text:` `String` — The title
- `EmptyState.Text(text)` — Its sentence.
  - `text:` `String` — The sentence
- `EmptyState.Actions { … }` — What to do: the block it was given.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Stat

A figure with its label — revenue, sign-ups — and how it moved: `Stat("Revenue", value: total, format: .currency, change: 0.124)`. `count:` counts to a new value.

```
Stat(label, value: …, format: …, change: …) { … }
```

```wf
Stat("Revenue", value: 48210, format: .currency, change: 0.124)
Stat("Churn", value: 0.031, format: .percent, change: -0.02)
```

Renders with the class `wf-stat`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | What the figure is |
| `value:` (required) | `Any` | The figure |
| `format:` | `.number` `.integer` `.currency` `.percent` `.compact` | How the figure is written, as `format` writes it |
| `change:` | `Number` | How it moved, as a fraction: `0.124` is up 12.4% |

**Flags:** `.number` `.integer` `.currency` `.percent` `.compact`

**Parts:**

- `Stat.Label(text)` — Its label.
  - `text:` `String` — The label
- `Stat.Value(text)` — Its figure.
  - `text:` `Any` — The figure
- `Stat.Change(text)` — How it moved, with its direction.
  - `text:` `String` — The change

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### CodeBlock

A block of code with a bar above it: a file name, a copy button (`.copy`), and line numbers (`.lines`). `language:` colours it, as on `Code`.

```
CodeBlock(code, language: …, title: …, copy: …)
```

```wf
CodeBlock("page Home(path: \"/\") {\n    Text(\"Hello\")\n}", language: "wf", title: "Home.wf").copy.lines
```

Renders with the class `wf-code-block`.

| Prop | Type | Meaning |
|---|---|---|
| `code` (positional) | `String` | The code |
| `language:` | `String` | Colours the code as this language: `wf`, `json`, `bash` or `css` |
| `title:` | `String` | The file name or the language, in the bar |
| `copy:` | `Bool` | A button in the bar that copies the code |
| `lines:` | `Bool` | Line numbers beside it |

**Flags:** `.copy` `.lines`

**Parts:**

- `CodeBlock.Bar { … }` — The bar above the code.
- `CodeBlock.Lines(text)` — The line numbers.
  - `text:` `String` — The numbers, a line each

Attribute families: `global`, `aria`, `data`.

### Chip

A compact label — a filter, a tag, a person — that can carry an icon, be pressed (`selected:`, with `on click`), and be removed (`.removable`, with `on remove`). `Tag` is the one that does none of it.

```
Chip(label, icon: …, selected: …, removable: …)
```

```wf
state open = false
Chip("Open only", selected: open) { on click { open = !open } }
Chip("Rust", icon: "code").removable { on remove { log("removed") } }
```

Renders with the class `wf-chip`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Its words |
| `icon:` | `String` | An icon before the words |
| `selected:` | `Bool` | Pressed: a filter that is on. Makes it a toggle button |
| `removable:` | `Bool` | A button that removes it, named `Remove {label}`, firing `on remove` |

**Flags:** `.selected` `.removable`

**Events of its own:** `on remove`

**Parts:**

- `Chip.Action(text) { … }` — What pressing a chip presses: its icon and words, a toggle button when it can be `selected:`.
  - `text:` `String` — Its words
- `Chip.Label(text) { … }` — Its icon and words, when pressing it does nothing.
  - `text:` `String` — Its words
- `Chip.Remove { … }` — Its remove button.

Attribute families: `global`, `aria`, `data`.

### VisuallyHidden

Words for a screen reader only: `Link(…) { Text("Docs")  VisuallyHidden("(opens in a new tab)") }`.

```
VisuallyHidden(text) { … }
```

```wf
Link(to: "/docs") { Text("Docs")  VisuallyHidden("(the reference)") }
```

Renders with the class `wf-visually-hidden`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The words |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Accordion

Sections a reader opens one by one, each a native `<details>`: `Accordion.Item("Shipping") { … }`. `.single` keeps one open at a time.

```
Accordion(single: …) { … }
```

```wf
Accordion.single {
    Accordion.Item("Shipping") { Text("Two to four days.") }
    Accordion.Item("Returns", open: true) { Text("Thirty days.") }
}
```

Renders with the class `wf-accordion`.

| Prop | Type | Meaning |
|---|---|---|
| `single:` | `Bool` | One open at a time: opening one closes the others |

**Flags:** `.single`

**Parts:**

- `Accordion.Item(title, open: …, name: …) { … }` — One section: its title is what a reader presses, its block what opens.
  - `title:` `String` — What a reader presses
  - `open:` `Bool` — Open when it is drawn
  - `name:` `String` — Items of one name open one at a time: what `.single` writes
  - flags: `.open`
- `Accordion.Summary(text) { … }` — What a reader presses to open an item: its title. Drawn for it.
  - `text:` `String` — The title
- `Accordion.Panel { … }` — What opens: the item's block. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Actions

### CopyButton

A button that copies `value:` to the clipboard, says `Copied` for a moment, and tells a screen reader so.

```
CopyButton(label, value: …, copied: …)
```

```wf
CopyButton(value: "https://example.com/invite/42")
CopyButton("Copy link", value: "https://example.com", copied: "Link copied")
```

Renders with the class `wf-btn wf-copy-button`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Its words; unset, `Copy` |
| `value:` (required) | `Any` | What it copies |
| `copied:` | `String` | What it says once it has copied; unset, `Copied` |

Attribute families: `global`, `aria`, `data`.

### ThemeToggle

A button that switches the page between light and dark, over `setTheme`: a moon in a light page, a sun in a dark one, named for the theme it switches to. The reader's choice is kept.

```
ThemeToggle
```

```wf
Row(gap: .sm) { ThemeToggle  LocaleSwitch }
```

Renders with the class `wf-icon-btn wf-theme-toggle`.

Attribute families: `global`, `aria`, `data`.

### LocaleSwitch

One button for each of `i18n.locales`, each written in its own language (`English`, `العربية`), the current one pressed; a press is `setLocale`.

```
LocaleSwitch(label: …)
```

```wf
LocaleSwitch(label: "Language")
```

Renders with the class `wf-locale-switch`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The group's name for a screen reader; unset, `Language` |

Attribute families: `global`, `aria`, `data`.

### Toggle

A button that stays pressed while its state holds — bold in an editor, a filter that is on: `aria-pressed`, a press flipping `bind:`.

```
Toggle(label, bind: …, icon: …)
```

```wf
state bold = false
Toggle("Bold", bind: bold, icon: "type")
```

Renders with the class `wf-btn wf-toggle`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Its words |
| `bind:` (required) | `State` | The `Bool` state it shows and flips |
| `icon:` | `String` | An icon before the words |

Attribute families: `global`, `aria`, `data`.

### ToggleGroup

Buttons of which one is chosen — a view switcher, a segmented control — or, `.multiple`, several: one stop for Tab, the arrow keys inside it, each `aria-pressed`. `bind:` holds the chosen `value:` (a list with `.multiple`).

```
ToggleGroup(bind: …, label: …, multiple: …) { … }
```

```wf
state view = "grid"
ToggleGroup(bind: view, label: "View") {
    ToggleGroup.Item("Grid", value: "grid", icon: "grid")
    ToggleGroup.Item("List", value: "list", icon: "list")
}
state styles: [String] = []
ToggleGroup(bind: styles, label: "Text style").multiple {
    ToggleGroup.Item("Bold", value: "b")
    ToggleGroup.Item("Italic", value: "i")
}
```

Renders with the class `wf-toggle-group`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The chosen value, or a list of them with `.multiple` |
| `label:` (required) | `String` | What the group chooses, for a screen reader |
| `multiple:` | `Bool` | Several at once: `bind:` is a list |

**Flags:** `.multiple`

**Parts:**

- `ToggleGroup.Item(label, value: …, icon: …, disabled: …)` — One choice: its words, and the value `bind:` holds when it is chosen.
  - `label:` `String` — Its words
  - `value:` `Any` — What `bind:` holds when it is chosen
  - `icon:` `String` — An icon before the words
  - `disabled:` `Bool` — Whether the control is inert
  - flags: `.disabled`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Button

A button: its label or block is what it shows, `on click` what it does.

```
Button(label, tone: …, size: …, shape: …) { … }
```

```wf
Button("Save").primary { on click { log("saved") } }
Button("Delete").danger.outlined.sm
Button("Send", type: .submit).primary.full
```

Renders with the class `wf-btn`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The label |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Colour |
| `size:` | `.sm` `.md` `.lg` | Its size |
| `shape:` | `.rounded` `.pill` | Corner shape |
| `width:` | `.full` | Width |
| `outlined:` | `Bool` | A border and no fill |
| `type:` | `.button` `.submit` `.reset` | Its role inside a form |
| `to:` | `Path` | The route to go to when it is pressed |
| `disabled:` | `Bool` | Whether the control is inert |
| `loading:` | `Bool` | Busy while it holds — `loading: Checkout.pay.pending`: a spinner in place of its label, at the same width, and a press does nothing |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.rounded` `.pill` `.full` `.outlined` `.button` `.submit` `.reset` `.disabled` `.loading`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### IconButton

Icon-only button. `label` is its accessible name, never visible text.

```
IconButton(icon: …, label: …, size: …) { … }
```

```wf
IconButton(icon: "close", label: "Close")
IconButton(icon: "edit", label: "Edit the title").primary.sm
```

Renders with the class `wf-icon-btn`.

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String` | One of the built-in icon names |
| `label:` | `String` | Accessible name (`aria-label` and `title`) |
| `size:` | `.sm` `.md` `.lg` | Its size |
| `tone:` | `.primary` `.danger` | Colour |
| `type:` | `.button` `.submit` `.reset` | Its role inside a form |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.sm` `.md` `.lg` `.primary` `.danger` `.button` `.submit` `.reset` `.disabled`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ButtonGroup

Buttons joined into one row.

```
ButtonGroup { … }
```

```wf
ButtonGroup {
    Button("Day")
    Button("Week")
    Button("Month")
}
```

Renders with the class `wf-btn-group`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Dropdown

Button that opens a menu of `Dropdown.Item`s.

```
Dropdown(label) { … }
```

```wf
Dropdown(label: "Actions") {
    Dropdown.Item { Text("Duplicate") }
    Dropdown.Divider
    Dropdown.Item { Text("Archive") }
}
```

Renders with the class `wf-dropdown`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Label of the button that opens it |

**Parts:**

- `Dropdown.Item { … }` — One entry of the menu.
- `Dropdown.Divider` — A rule between groups of entries.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Media

### Image

Image; `alt` is required for accessibility (rule A01). Given an `image` the program declares, it becomes a `<picture>` with every width the build wrote.

```
Image(source, src: …, sizes: …, placeholder: …)
```

```wf
Image(src: "/team.jpg", alt: "The team at the launch", width: 1200, height: 630)
Image(src: "/flourish.svg", alt: "")
```

Renders with the class `wf-image`.

| Prop | Type | Meaning |
|---|---|---|
| `source` (positional) | `Any` | An `image` the program declares, or a URL |
| `src:` | `String` | Image URL |
| `sizes:` | `String` | How wide it will be shown, so the browser picks a width: `(max-width: 768px) 100vw, 1200px` |
| `placeholder:` | `.blur` `.color` `.none` | What fills the box while the image loads |
| `alt:` | `String` | Text alternative; empty for a purely decorative image |
| `width:` | `Number` | Intrinsic width |
| `height:` | `Number` | Intrinsic height |
| `loading:` | `.lazy` `.eager` | Unset, the first image on a page is eager at high priority and the rest lazy |
| `shape:` | `.rounded` `.circle` | Shape |

**Flags:** `.blur` `.color` `.none` `.lazy` `.eager` `.rounded` `.circle`

Attribute families: `global`, `aria`, `data`.

### Video

Video player. Without `captions` it draws an `A09` warning: a video nobody can hear is a video nobody can follow.

```
Video(src: …, controls: …, autoplay: …)
```

```wf
Video(src: "/demo.mp4", captions: "/demo.en.vtt", poster: "/demo.jpg").controls
```

Renders with the class `wf-video`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Video URL |
| `controls:` | `Bool` | Show the browser's playback controls |
| `autoplay:` | `Bool` | Start playing when shown |
| `poster:` | `String` | The frame shown before it plays |
| `muted:` | `Bool` | Start with the sound off |
| `loop:` | `Bool` | Start again when it ends |
| `playsinline:` | `Bool` | Play where it sits, rather than full screen |
| `captions:` | `String` | A WebVTT file of captions, which becomes a `<track>` |
| `width:` | `Number` | Intrinsic width |
| `height:` | `Number` | Intrinsic height |

**Flags:** `.controls` `.autoplay` `.muted` `.loop` `.playsinline`

Attribute families: `global`, `aria`, `data`, `media`.

### Audio

Audio player. `transcript` links what was said, which is what a reader who cannot hear it needs.

```
Audio(src: …, controls: …, autoplay: …)
```

```wf
Audio(src: "/episode-1.mp3", transcript: "/episode-1.txt").controls
```

Renders with the class `wf-audio`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Audio URL |
| `controls:` | `Bool` | Show the browser's playback controls |
| `autoplay:` | `Bool` | Start playing when shown |
| `loop:` | `Bool` | Start again when it ends |
| `transcript:` | `String` | A link to what was said, shown beneath the player |

**Flags:** `.controls` `.autoplay` `.loop`

Attribute families: `global`, `aria`, `data`.

### Frame

Another page, shown inside this one: an `iframe` — the page at `src:`, or the HTML `srcdoc:` gives it, sandboxed as `sandbox:` says. `on message(m)` hears what that page posts to this one (`m.data`), and nothing any other page posts. Without `title:` it draws an `A17` warning: a frame is announced by its title.

```
Frame(src: …, srcdoc: …, title: …)
```

```wf
Frame(src: "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ", title: "The launch talk", allow: "fullscreen")
state heard = ""
Frame(src: "/preview/", title: "Preview", sandbox: "allow-scripts") {
    on message(m) { heard = m.data }
}
state version = 1
Frame(srcdoc: "<p>Version {version}</p>", title: "Example", sandbox: "")
```

Renders with the class `wf-frame`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The address of the page it shows |
| `srcdoc:` | `String` | The HTML it draws in place of an address, sandboxed as `sandbox:` says; a value that reads state draws the new HTML as it changes |
| `title:` | `String` | What it shows, in a few words: its accessible name |
| `sandbox:` | `String` | What the framed page may do: `"allow-scripts allow-forms"`; empty, it may do nothing |
| `allow:` | `String` | The features it may use: `"fullscreen; clipboard-write"` |
| `loading:` | `.lazy` `.eager` | Unset, at once |

**Flags:** `.lazy` `.eager`

**Events of its own:** `on message`, `on load`

Attribute families: `global`, `aria`, `data`.

### Icon

One of the built-in SVG icons, rendered inline.

```
Icon(name, size: …, tone: …, muted: …)
```

```wf
Icon("search")
Icon("check").lg.success
```

Renders with the class `wf-icon`.

| Prop | Type | Meaning |
|---|---|---|
| `name` (positional) | `String` | The icon name |
| `size:` | `.sm` `.md` `.lg` | Its size |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Colour |
| `muted:` | `Bool` | Dimmed, for secondary text |

**Flags:** `.sm` `.md` `.lg` `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.muted`

Attribute families: `global`, `aria`, `data`.

### Carousel

Slide track with named controls and dots; optional autoplay.

```
Carousel(label: …, autoplay: …, interval: …) { … }
```

```wf
Carousel(label: "Recent work", autoplay: true, interval: 5000) {
    Carousel.Slide { Image(src: "/one.jpg", alt: "A bridge at dusk") }
    Carousel.Slide { Image(src: "/two.jpg", alt: "The harbour") }
}
```

Renders with the class `wf-carousel`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Accessible name of the region |
| `autoplay:` | `Bool` | Advance automatically |
| `interval:` | `Number` | Milliseconds between slides |

**Flags:** `.autoplay`

**Parts:**

- `Carousel.Slide { … }` — One slide of a carousel.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Typography

### Text

A paragraph of text. `{…}` in the string reads state and stays live.

```
Text(content, tone: …, size: …, align: …)
```

```wf
Text("Hello, world")
Text("Last updated today").muted.sm
Text("Important").bold.danger
```

Renders with the class `wf-text`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `Any` | The text |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Colour |
| `size:` | `.sm` `.md` `.lg` | Its size |
| `align:` | `.left` `.center` `.right` | Text alignment |
| `transform:` | `.uppercase` `.lowercase` | Letter case |
| `kind:` | `.heading` `.subtitle` | Typographic role |
| `bold:` | `Bool` | Bold |
| `italic:` | `Bool` | Italic |
| `underline:` | `Bool` | Underlined |
| `muted:` | `Bool` | Dimmed, for secondary text |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.left` `.center` `.right` `.uppercase` `.lowercase` `.heading` `.subtitle` `.bold` `.italic` `.underline` `.muted`

Attribute families: `global`, `aria`, `data`.

### Heading

Heading; `level` picks the tag (`h2` when none is given).

```
Heading(content, level: …, align: …, muted: …)
```

```wf
Heading("Pricing").h1
Heading("Frequently asked").h2
```

Renders with the class `wf-heading`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `Any` | The heading text |
| `level:` | `.h1` `.h2` `.h3` `.h4` `.h5` `.h6` | Heading level: the tag, and its place in the outline |
| `align:` | `.left` `.center` `.right` | Text alignment |
| `muted:` | `Bool` | Dimmed, for secondary text |

**Flags:** `.h1` `.h2` `.h3` `.h4` `.h5` `.h6` `.left` `.center` `.right` `.muted`

Attribute families: `global`, `aria`, `data`.

### Code

Inline code, or a code block with `block`; `language:` colours it.

```
Code(content, block: …, language: …)
```

```wf
Text("Run ")
Code("wf serve")
Code("page Home(path: \"/\") { Text(\"Hi\") }", language: "wf").block
```

Renders with the class `wf-code`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `Any` | The code |
| `block:` | `Bool` | A multi-line block |
| `language:` | `String` | Colours the code as this language: `wf`, `json`, `bash` or `css` |

**Flags:** `.block`

Attribute families: `global`, `aria`, `data`.

### Markdown

Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run is drawn as its words. `.inert` is for text somebody else wrote.

```
Markdown(text, inert: …)
```

```wf
state docs = "Returns &lt;T&gt;, as [the guide](https://webfluent.monzeromer.dev/docs) says."
Markdown("Some **bold** text and a [link](https://webfluent.monzeromer.dev/docs).")
Markdown(docs).inert
```

Renders with the class `wf-markdown`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The Markdown text |
| `inert:` | `Bool` | Text somebody else wrote, such as a language server's documentation: its entities are decoded, a link is drawn as its words and a picture is left out |

**Flags:** `.inert`

Attribute families: `global`, `aria`, `data`.

### Unsafe

The one door for markup a page did not write. It has a single part, `Unsafe.Html`.

```
Unsafe
```

```wf
state body = "<p>From the CMS</p>"
Unsafe.Html(sanitize(body))
```

**Parts:**

- `Unsafe.Html(markup)` — Markup, put in as markup. It is **not** sanitised — the name says so — and `sanitize(…)` is one call away for anything the project did not write itself.
  - `markup:` `String` — The HTML to insert

Attribute families: `global`, `aria`, `data`.

### Blockquote

Quotation: the text, or its block as the quoted content.

```
Blockquote(content) { … }
```

```wf
Blockquote { Text("Make it work, make it right, make it fast.") }
```

Renders with the class `wf-blockquote`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `Any` | The quoted text |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## PDF

### TableOfContents

The document's headings with the page each lands on, each a link to it.

```
TableOfContents(levels: …, title: …)
```

```wf
TableOfContents(levels: 2)
```

Renders with the class `wf-toc`.

| Prop | Type | Meaning |
|---|---|---|
| `levels:` | `Number` | How deep it lists: 1 is the `h1`s only (default 3) |
| `title:` | `String` | A heading above the list |

Attribute families: `global`, `aria`, `data`.

### Document

Root of a PDF document: the paper, the margins and what the file says about itself.

```
Document(page_size: …, size: …, landscape: …) { … }
```

```wf
Document(page_size: "A4") {
    Section { Heading("Report").h1  Paragraph("Revenue grew.") }
}
```

Renders with the class `wf-document`.

| Prop | Type | Meaning |
|---|---|---|
| `page_size:` | `.A4` `.A3` `.A5` `.Letter` `.Legal` | Paper size (also `size:`) |
| `size:` | `String` | Paper size: `A4`, `Letter`, `Tabloid`, or two lengths, `210mm 297mm` |
| `landscape:` | `Bool` | Turn the paper on its side |
| `margin:` | `Any` | The page margins: one length, or top right bottom left (`72`, `20mm 25mm`) |
| `title:` | `String` | The file's title, shown by a reader in place of its name |
| `author:` | `String` | The file's author; several separated by commas |
| `subject:` | `String` | What the document is about |
| `keywords:` | `String` | Comma-separated keywords |
| `lang:` | `String` | The document's language, for screen readers and hyphenation |

**Flags:** `.A4` `.A3` `.A5` `.Letter` `.Legal` `.landscape`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Section

Groups content with spacing; on the web, a `<section>`.

```
Section { … }
```

```wf
Section {
    Heading("Terms").h2
    Text("Payment within 30 days.")
}
```

Renders with the class `wf-section`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Paragraph

Block of text with paragraph spacing.

```
Paragraph(content, tone: …, size: …, align: …)
```

```wf
Paragraph("A block of prose, with the spacing a paragraph has.")
```

Renders with the class `wf-text`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `Any` | The text |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Colour |
| `size:` | `.sm` `.md` `.lg` | Its size |
| `align:` | `.left` `.center` `.right` | Text alignment |
| `transform:` | `.uppercase` `.lowercase` | Letter case |
| `kind:` | `.heading` `.subtitle` | Typographic role |
| `bold:` | `Bool` | Bold |
| `italic:` | `Bool` | Italic |
| `underline:` | `Bool` | Underlined |
| `muted:` | `Bool` | Dimmed, for secondary text |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.left` `.center` `.right` `.uppercase` `.lowercase` `.heading` `.subtitle` `.bold` `.italic` `.underline` `.muted`

Attribute families: `global`, `aria`, `data`.

### Header

Repeated at the top of every PDF page (or those `on:` names), where `page` and `pages` are the page's number and the count; a `<header>` on the web.

```
Header(on: …) { … }
```

```wf
Header { Text("Company Inc.").muted.sm }
```

Renders with the class `wf-header`.

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Footer

Repeated at the bottom of every PDF page (or those `on:` names), where `page` and `pages` are the page's number and the count; a `<footer>` on the web.

```
Footer(on: …) { … }
```

```wf
Footer { Text("© Company Inc.").muted.sm }
```

Renders with the class `wf-footer`.

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Background

Drawn behind each PDF page's content, the size of the page: a cover, a band, a full-bleed picture.

```
Background(on: …) { … }
```

```wf
Background(on: .first) { Stack { style { height: 100%; background: linear-gradient(160deg, #1d4ed8, #0f172a) } } }
```

Renders with the class `wf-background`.

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Watermark

Large, faint, rotated text behind each PDF page's content.

```
Watermark(text, on: …)
```

```wf
Watermark("DRAFT")
```

Renders with the class `wf-watermark`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The text |
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Attribute families: `global`, `aria`, `data`.

### PageBreak

Forces a new PDF page.

```
PageBreak
```

```wf
Text("End of chapter one.")
PageBreak
Heading("Chapter two").h2
```

Renders with the class `wf-page-break`.

Attribute families: `global`, `aria`, `data`.

## Slides

### Presentation

Root of a slide deck; its children must be slides.

```
Presentation { … }
```

```wf
Presentation {
    TitleSlide("Q1 review", subtitle: "March 2026")
    Slide { Heading("Highlights").h1 }
}
```

Renders with the class `wf-presentation`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Slide

One slide, one PDF page; content is top-aligned and clipped at the bottom margin.

```
Slide { … }
```

```wf
Presentation {
    Slide {
        Heading("Revenue").h1
        Text("Up 18% on last quarter.")
    }
}
```

Renders with the class `wf-slide`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### TitleSlide

Cover slide: title and subtitle, vertically centred.

```
TitleSlide(title, subtitle: …)
```

```wf
Presentation { TitleSlide("Q1 review", subtitle: "Acme, March 2026") }
```

Renders with the class `wf-title-slide`.

| Prop | Type | Meaning |
|---|---|---|
| `title` (positional) | `String` | The title |
| `subtitle:` | `String` | The subtitle |

Attribute families: `global`, `aria`, `data`.

### SectionSlide

Full-bleed coloured band with a centred label.

```
SectionSlide(label, tone: …)
```

```wf
Presentation { SectionSlide("Numbers").primary }
```

Renders with the class `wf-section-slide`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The label |
| `tone:` | `.primary` `.success` `.danger` `.warning` `.info` | Colour of the band |

**Flags:** `.primary` `.success` `.danger` `.warning` `.info`

Attribute families: `global`, `aria`, `data`.

### TwoColumn

Two equal columns; takes exactly two `Container` children.

```
TwoColumn { … }
```

```wf
Presentation {
    TwoColumn {
        Container { Heading("Wins").h3 }
        Container { Heading("Risks").h3 }
    }
}
```

Renders with the class `wf-two-column`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ImageSlide

Image slide with an optional caption.

```
ImageSlide(src: …, caption: …)
```

```wf
Presentation { ImageSlide(src: "chart.png", caption: "Revenue by month") }
```

Renders with the class `wf-image-slide`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Image path (required) |
| `caption:` | `String` | Caption under the image |

Attribute families: `global`, `aria`, `data`.

## Routing

### Router

Where the current page renders. May sit at any depth inside `app`; pages declare their own paths.

```
Router(transition: …, duration: …) { … }
```

```wf
app {
    Navbar { Navbar.Brand { Link("Acme", to: "/") } }
    Router(transition: .fade, duration: "200ms")
}
```

Renders with the class `wf-router`.

| Prop | Type | Meaning |
|---|---|---|
| `transition:` | `.none` `.fade` `.slide` | How a route change moves between pages |
| `duration:` | `String` | How long each half of the transition plays, e.g. `200ms` |

**Flags:** `.none` `.fade` `.slide`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## On every element

| Prop | Type | Meaning |
|---|---|---|
| `animate:` | `.fadeIn` `.fadeOut` `.slideUp` `.slideDown` `.slideLeft` `.slideRight` `.scaleIn` `.scaleOut` `.bounce` `.shake` `.pulse` `.spin` `.expand` `.collapse` | Animation played when the element appears |
| `exit:` | `.fadeIn` `.fadeOut` `.slideUp` `.slideDown` `.slideLeft` `.slideRight` `.scaleIn` `.scaleOut` `.bounce` `.shake` `.pulse` `.spin` `.expand` `.collapse` | Animation played when the element leaves |
| `speed:` | `.fast` `.normal` `.slow` | How fast its animation runs |
| `duration:` | `String` | Animation length, e.g. `300ms` |
| `delay:` | `String` | Wait before the animation starts |
| `easing:` | `.standard` `.spring` `.ease` `.easeIn` `.easeOut` `.easeInOut` `.linear` `.bouncy` `.smooth` | How the animation is paced — a named easing, or any CSS timing function |
| `stagger:` | `String` | Delay added per item of a list, e.g. `50ms` |
| `on:` | `.mount` `.enterView` | When its animation plays |
| `count:` | `String` | How long the number it shows takes to count to a new value, e.g. `600ms` |
| `shared:` | `String` | A name this element keeps across a route change, so the browser carries it from one page to the next |
| `class:` | `String` | Classes added beside the engine's, from the project's own stylesheets |
| `ref:` | `State` | A handle on the element, usable as the element: `ref: nameInput`, then `nameInput.focus()` |
| `drag:` | `Any` | What it carries when it is picked up — by a mouse moved a few pixels, or a finger held still — and dropped on something with `on receive(d)` |
| `dragGroup:` | `String` | Which drops it belongs to: a drag goes only to targets of its own group, so tabs and tree rows stay apart |
| `dropValue:` | `Any` | What this target is, as a drag sees it: `d.over` in the drag's `on dragmove(d)` and in `on receive(d)` |
| `dropAxis:` | `.y` `.x` | Which way its list runs, for where a drop lands |
| `dropInside:` | `Bool` | A drop on its middle lands inside it — a folder, a container — as well as before or after |
| `roving:` | `.x` `.y` `.both` | One Tab stop for the group, and the arrow keys, Home, End and the first letters of a name to move inside it: a toolbar, a row of choices, a list |
| `trapFocus:` | `Bool` | While true, focus moves into it and Tab and Shift+Tab stay inside; when it turns false or the element leaves, focus goes back where it was |
| `mount:` | `Any` | Run with the element each time it is made — `mount: (n) => initTilt(n)` — and what it returns is the handle `update:` and `cleanup:` get |
| `update:` | `Any` | Run with the handle again whenever the state it reads changes: `update: (h) => h.setMax(max)` |
| `cleanup:` | `Any` | Run with the handle when the element leaves — a route change, an `if` branch, a list item: `cleanup: (h) => h.destroy()` |

The cases of `animate:` and `speed:` are also flags on every element: `.fadeIn` … `.spin`, `.fast` `.normal` `.slow`; so are the `Bool` props above, `.dropInside` and `.trapFocus`.


Events every element accepts in an `on … { }` handler:

- `on click` — The element was clicked or activated
- `on input` — The value changed, on every keystroke
- `on change` — The value was committed
- `on submit` — The form was submitted
- `on focus` — The element received focus
- `on blur` — The element lost focus
- `on keydown` — A key went down; `e.key` names it
- `on longpress` — Held still for half a second; `p.x`, `p.y` say where. The click it ends with is not a click
- `on swipe` — A finger or a pen swept across it; `s.direction` is "left", "right", "up" or "down"
- `on receive` — Something with `drag:` was dropped on it; `d.value` is what, `d.position` "before", "after" or "inside"
- `on dragmove` — On what is dragged, while it moves: `d.over` is the target's `dropValue` or null, `d.position` where it would land
- `on visible` — It came into view: scrolled to, or drawn where the reader is looking. `v.ratio` is how much of it shows. A "load more" row's cue
- `on key` — One key, named: `on key("ctrl+k") { }`
- `on keyup` — A key came up
- `on keypress` — A character key was pressed
- `on mouseenter` — The pointer entered the element
- `on mouseleave` — The pointer left the element
- `on dblclick` — The element was double-clicked
- `on auxclick` — A button other than the main one was clicked; `e.button` is which — 1 is the middle, which closes a tab
- `on focusin` — Focus moved onto the element or something inside it
- `on focusout` — Focus left the element or something inside it
- `on contextmenu` — The context menu was asked for; `e.preventDefault()` keeps the browser's
- `on mousedown` — A mouse button went down over the element
- `on mouseup` — A mouse button came up over the element
- `on mousemove` — The mouse moved over the element
- `on mouseover` — The pointer moved onto the element or one of its children
- `on mouseout` — The pointer moved off the element or one of its children
- `on pointerdown` — A pointer — mouse, pen or touch — went down
- `on pointerup` — A pointer came up
- `on pointermove` — A pointer moved
- `on pointerenter` — A pointer entered the element
- `on pointerleave` — A pointer left the element
- `on pointercancel` — The browser took the pointer back, for a scroll or a gesture
- `on touchstart` — A finger touched the element
- `on touchend` — A finger left the element
- `on touchmove` — A finger moved across the element
- `on wheel` — The wheel or trackpad scrolled over the element
- `on scroll` — The element's content scrolled
- `on focusin` — The element or something inside it received focus
- `on focusout` — The element or something inside it lost focus
- `on reset` — The form was reset
- `on invalid` — A control failed the browser's own validation
- `on select` — Text in a field was selected
- `on copy` — Something was copied from the element
- `on cut` — Something was cut from the element
- `on paste` — Something was pasted into the element; `e.clipboardData` holds it
- `on dragstart` — The element started being dragged
- `on drag` — The element is being dragged
- `on dragend` — A drag of the element ended
- `on dragenter` — Something dragged entered the element
- `on dragover` — Something dragged is over the element; `e.preventDefault()` allows a drop
- `on dragleave` — Something dragged left the element
- `on drop` — Something was dropped on the element; `e.dataTransfer` holds it
- `on load` — An image, video or frame finished loading
- `on error` — An image, video or script failed to load
- `on play` — A video or audio started playing
- `on pause` — A video or audio paused
- `on ended` — A video or audio reached its end
- `on timeupdate` — A playing video or audio moved on
- `on animationend` — A CSS animation on the element ended
- `on transitionend` — A CSS transition on the element ended
- `on toggle` — A details element opened or closed

## Icons

The names `Icon(…)`, `IconButton(icon:)` and `Sidebar.Item(icon:)` draw; any other name shows as the word, and the compiler warns.

`close` `menu` `search` `home` `user` `settings` `check` `plus` `minus` `edit` `trash` `star` `heart` `mail` `bell` `download` `upload` `eye` `link` `calendar` `filter` `chevron-down` `chevron-right` `chevron-left` `info` `warning` `arrow-left` `arrow-right` `logout` `copy` `sun` `moon` `folder` `folder-plus` `folder-off` `file` `file-plus` `refresh` `alert-circle` `clock` `dot` `diff` `book` `git-pull-request` `pin` `git-branch` `more-horizontal` `window-minimize` `window-maximize` `outline` `help-circle` `collapse-all` `locate` `chevron-up` `globe` `external-link` `panel-down` `panel-up` `code` `monitor` `git-commit` `play` `layers` `insert` `grid` `list` `square` `pill` `image` `sparkle` `lock` `type` `pointer` `reach` `frame` `line` `pipette` `comment` `hand` `zoom` `ruler` `eye-off` `unlock`

## Next

[CLI](https://webfluent.monzeromer.dev/reference/cli.md).
