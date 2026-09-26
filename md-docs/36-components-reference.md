# 36. Components reference

<!--
route: reference
group: reference
blurb: Every built-in with its props, cases, flags, events, slots and parts — generated from the compiler's own registry.
description: The components reference: every built-in element, its props, flags, events, slots, parts, attribute families and an example.
-->

Generated from the compiler's registry (`wf registry --json`) by `scripts/components-reference.py`, with an example of each from `scripts/component_examples.py`; edit those, not this file.

Every component is written `Name(positional, prop: value).flag { block }`. A **prop** is passed by name; the one **positional** prop, where there is one, comes first and unnamed. A **flag** is written after the parentheses with a dot: a `Bool` prop, or a case of one of the element's enum props (`Button("x").primary.lg`). An **enum prop** takes a case written `.case` (`Row(gap: .md)`). Every element also takes the universal props and events listed at the end, and the HTML attribute families named on it (`aria-*`, `data-*`, and the global attributes such as `id`, `class`, `title`, `role`, `tabindex`).

## Contents

- **Layout**: [`Container`](#container), [`Row`](#row), [`Column`](#column), [`Grid`](#grid), [`Stack`](#stack), [`Spacer`](#spacer), [`Divider`](#divider), [`Host`](#host)
- **Navigation**: [`Navbar`](#navbar), [`Sidebar`](#sidebar), [`Link`](#link), [`Tabs`](#tabs), [`Breadcrumb`](#breadcrumb), [`Menu`](#menu)
- **Data display**: [`Card`](#card), [`Table`](#table), [`List`](#list), [`Badge`](#badge), [`Tag`](#tag), [`Avatar`](#avatar), [`Tooltip`](#tooltip)
- **Form**: [`Input`](#input), [`Select`](#select), [`Checkbox`](#checkbox), [`Radio`](#radio), [`Switch`](#switch), [`Slider`](#slider), [`DatePicker`](#datepicker), [`FileUpload`](#fileupload), [`Form`](#form), [`Textarea`](#textarea)
- **Feedback**: [`Alert`](#alert), [`Toast`](#toast), [`Modal`](#modal), [`Dialog`](#dialog), [`Spinner`](#spinner), [`Progress`](#progress), [`Skeleton`](#skeleton)
- **Actions**: [`Button`](#button), [`IconButton`](#iconbutton), [`ButtonGroup`](#buttongroup), [`Dropdown`](#dropdown)
- **Media**: [`Image`](#image), [`Video`](#video), [`Audio`](#audio), [`Icon`](#icon), [`Carousel`](#carousel)
- **Typography**: [`Text`](#text), [`Heading`](#heading), [`Code`](#code), [`Markdown`](#markdown), [`Unsafe`](#unsafe), [`Blockquote`](#blockquote)
- **PDF**: [`Document`](#document), [`Section`](#section), [`Paragraph`](#paragraph), [`Header`](#header), [`Footer`](#footer), [`PageBreak`](#pagebreak)
- **Slides**: [`Presentation`](#presentation), [`Slide`](#slide), [`TitleSlide`](#titleslide), [`SectionSlide`](#sectionslide), [`TwoColumn`](#twocolumn), [`ImageSlide`](#imageslide)
- **Routing**: [`Router`](#router)

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

### Host

An element handed to somebody else's code, with a lifetime. `mount:` runs once with the node, `update:` again whenever the state it reads changes, and `cleanup:` when the page, branch or list item it belongs to leaves.

```
Host(mount: …, update: …, cleanup: …)
```

```wf
Host(tag: "canvas", mount: (node) => node.getContext("2d"), cleanup: (ctx) => log("gone"))
```

Renders with the class `wf-host`.

| Prop | Type | Meaning |
|---|---|---|
| `mount:` | `Any` | `(node) => …` — run once with the element, giving back whatever the library hands over |
| `update:` | `Any` | `(handle) => …` — run again whenever the state it reads changes |
| `cleanup:` | `Any` | `(handle) => …` — run when what owns it leaves, so nothing is left behind |
| `tag:` | `String` | The element to make, `div` by default |

Attribute families: `global`, `aria`, `data`.

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

Tab strip switching between its pages.

```
Tabs { … }
```

```wf
Tabs {
    Tabs.Page("Profile") { Text("Your details") }
    Tabs.Page("Billing") { Text("Your plan") }
}
```

Renders with the class `wf-tabs`.

**Parts:**

- `Tabs.Page(label) { … }` — One tab and its panel.
  - `label:` `String` — The tab's label

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

Menu opened by a trigger button.

```
Menu(trigger: …, label: …) { … }
```

```wf
Menu(trigger: "Options") {
    Menu.Item { Text("Rename") }
    Menu.Divider
    Menu.Item { Text("Delete") }
}
```

Renders with the class `wf-menu`.

| Prop | Type | Meaning |
|---|---|---|
| `trigger:` | `String` | Label of the button that opens the menu |
| `label:` | `String` | Same as `trigger` |

**Parts:**

- `Menu.Item { … }` — One entry of the menu.
- `Menu.Divider` — A rule between groups of entries.

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
- `Table.Body { … }` — Body section.
- `Table.Row { … }` — A row.
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

Shows `text` when its child is hovered or focused.

```
Tooltip(text) { … }
```

```wf
Tooltip("Copies the link") { IconButton(icon: "copy", label: "Copy link") }
```

Renders with the class `wf-tooltip`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The tooltip text |

Takes a block of children.

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

Range input. `aria-*` arguments reach the input; with `aria-valuetext` the raw number is not shown beside the track.

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

Date input.

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

Modal `<dialog>`, opened by a state flag; the browser supplies focus trapping and Escape.

```
Modal(visible: …, title: …) { … }
```

```wf
state open = false
Button("Delete") { on click { open = true } }
Modal(visible: open, title: "Delete this?") {
    Text("This cannot be undone.")
    Modal.Footer { Button("Cancel") { on click { open = false } } }
}
```

Renders with the class `wf-modal`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |

**Parts:**

- `Modal.Footer { … }` — The modal's action row.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Dialog

Confirmation `<dialog>`, opened by a state flag.

```
Dialog(visible: …, title: …) { … }
```

```wf
state asking = false
Dialog(visible: asking, title: "Leave without saving?") { Text("Your draft will be lost.") }
```

Renders with the class `wf-dialog`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |

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

## Actions

### Button

Button. Its block holds what it shows; `on click` what it does.

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
| `to:` | `Path` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `disabled:` | `Bool` | Whether the control is inert |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.rounded` `.pill` `.full` `.outlined` `.button` `.submit` `.reset` `.disabled`

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

- `Carousel.Slide { … }` — One slide.

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

Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run.

```
Markdown(text)
```

```wf
Markdown("Some **bold** text and a [link](/docs).")
```

Renders with the class `wf-markdown`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The Markdown text |

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

### Document

Root of a PDF document.

```
Document(page_size: …) { … }
```

```wf
Document(page_size: "A4") {
    Section { Heading("Report").h1  Paragraph("Revenue grew.") }
}
```

Renders with the class `wf-document`.

| Prop | Type | Meaning |
|---|---|---|
| `page_size:` | `.A4` `.A3` `.A5` `.Letter` `.Legal` | Paper size |

**Flags:** `.A4` `.A3` `.A5` `.Letter` `.Legal`

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

Repeated at the top of every PDF page; a `<header>` on the web.

```
Header { … }
```

```wf
Header { Text("Company Inc.").muted.sm }
```

Renders with the class `wf-header`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Footer

Repeated at the bottom of every PDF page; a `<footer>` on the web.

```
Footer { … }
```

```wf
Footer { Text("© Company Inc.").muted.sm }
```

Renders with the class `wf-footer`.

Takes a block of children.

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

The cases of `animate:` and `speed:` are also flags on every element: `.fadeIn` … `.spin`, `.fast` `.normal` `.slow`.


Events every element accepts in an `on … { }` handler:

- `on click` — The element was clicked or activated
- `on input` — The value changed, on every keystroke
- `on change` — The value was committed
- `on submit` — The form was submitted
- `on focus` — The element received focus
- `on blur` — The element lost focus
- `on keydown` — A key went down; `e.key` names it
- `on key` — One key, named: `on key("ctrl+k") { }`
- `on keyup` — A key came up
- `on keypress` — A character key was pressed
- `on mouseenter` — The pointer entered the element
- `on mouseleave` — The pointer left the element
- `on dblclick` — The element was double-clicked
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

`close` `menu` `search` `home` `user` `settings` `check` `plus` `minus` `edit` `trash` `star` `heart` `mail` `bell` `download` `upload` `eye` `link` `calendar` `filter` `chevron-down` `chevron-right` `chevron-left` `info` `warning` `arrow-left` `arrow-right` `logout` `copy` `sun` `moon`

## Next

[CLI](37-cli.md).
