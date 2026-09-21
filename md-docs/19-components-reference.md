# 19. Built-in components — reference

Generated from the compiler's registry (`wf registry --json`) by `scripts/components-reference.py`; edit the registry, not this file.

Every component is written `Name(positional, prop: value).flag { block }`. A **prop** is passed by name; the one **positional** prop, where there is one, comes first and unnamed. A **flag** is written after the parentheses with a dot: a `Bool` prop, or a case of one of the element's enum props (`Button("x").primary.lg`). An **enum prop** takes a case written `.case` (`Row(gap: .md)`). Every element also takes the universal props and events listed at the end, and the HTML attribute families named on it (`aria-*`, `data-*`, and the global attributes such as `id`, `class`, `title`, `role`, `tabindex`).

## Contents

- **Layout**: [`Container`](#container), [`Row`](#row), [`Column`](#column), [`Grid`](#grid), [`Stack`](#stack), [`Spacer`](#spacer), [`Divider`](#divider)
- **Navigation**: [`Navbar`](#navbar), [`Sidebar`](#sidebar), [`Link`](#link), [`Tabs`](#tabs), [`Breadcrumb`](#breadcrumb), [`Menu`](#menu)
- **Data display**: [`Card`](#card), [`Table`](#table), [`List`](#list), [`Badge`](#badge), [`Tag`](#tag), [`Avatar`](#avatar), [`Tooltip`](#tooltip)
- **Form**: [`Input`](#input), [`Select`](#select), [`Checkbox`](#checkbox), [`Radio`](#radio), [`Switch`](#switch), [`Slider`](#slider), [`DatePicker`](#datepicker), [`FileUpload`](#fileupload), [`Form`](#form)
- **Feedback**: [`Alert`](#alert), [`Toast`](#toast), [`Modal`](#modal), [`Dialog`](#dialog), [`Spinner`](#spinner), [`Progress`](#progress), [`Skeleton`](#skeleton)
- **Actions**: [`Button`](#button), [`IconButton`](#iconbutton), [`ButtonGroup`](#buttongroup), [`Dropdown`](#dropdown)
- **Media**: [`Image`](#image), [`Video`](#video), [`Icon`](#icon), [`Carousel`](#carousel)
- **Typography**: [`Text`](#text), [`Heading`](#heading), [`Code`](#code), [`Markdown`](#markdown), [`Blockquote`](#blockquote)
- **PDF**: [`Document`](#document), [`Section`](#section), [`Paragraph`](#paragraph), [`Header`](#header), [`Footer`](#footer), [`PageBreak`](#pagebreak)
- **Slides**: [`Presentation`](#presentation), [`Slide`](#slide), [`TitleSlide`](#titleslide), [`SectionSlide`](#sectionslide), [`TwoColumn`](#twocolumn), [`ImageSlide`](#imageslide)
- **Routing**: [`Router`](#router)

## Layout

### Container

Centred wrapper with a maximum width and horizontal padding.

```
Container(fluid: …) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `fluid:` | `Bool` | Full width, no maximum |

**Flags:** `.fluid`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Row

Horizontal flex layout.

```
Row(gap: …, align: …, justify: …) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between children |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Cross-axis alignment |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | Main-axis distribution |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Column

A child of the 12-column grid, spanning `span` columns.

```
Column(span: …) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `span:` | `Number` | How many of the 12 grid columns to span |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Grid

CSS grid with a fixed number of columns.

```
Grid(columns: …, gap: …, align: …) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | Number of equal columns |
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between children |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Cross-axis alignment |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | Main-axis distribution |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Stack

Vertical flex layout.

```
Stack(gap: …, align: …, justify: …) { … }
```

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

Attribute families: `global`, `aria`, `data`.

## Navigation

### Navbar

Top navigation bar with brand, links and action areas.

```
Navbar { … }
```

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

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The tag text |

Attribute families: `global`, `aria`, `data`.

### Avatar

A picture of a person, or their initials.

```
Avatar(src: …, alt: …, initials: …)
```

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
Form(bind: …) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A handle on the form: `form.valid`, `form.values` by field name, `form.reset()` |

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `form`.

## Feedback

### Alert

Inline notice; a `danger` or `warning` one is announced as an alert.

```
Alert(message, tone: …) { … }
```

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

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Dropdown

Button that opens a menu of `Dropdown.Item`s.

```
Dropdown(label) { … }
```

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

Image; `alt` is required for accessibility (rule A01).

```
Image(src: …, alt: …, width: …)
```

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Image URL |
| `alt:` | `String` | Text alternative; empty for a purely decorative image |
| `width:` | `Number` | Intrinsic width |
| `height:` | `Number` | Intrinsic height |
| `loading:` | `.lazy` `.eager` | Unset, the first image on a page is eager at high priority and the rest lazy |
| `shape:` | `.rounded` `.circle` | Shape |

**Flags:** `.lazy` `.eager` `.rounded` `.circle`

Attribute families: `global`, `aria`, `data`.

### Video

Video player.

```
Video(src: …, controls: …, autoplay: …)
```

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Video URL |
| `controls:` | `Bool` | Show the browser's playback controls |
| `autoplay:` | `Bool` | Start playing when shown |
| `width:` | `Number` | Intrinsic width |
| `height:` | `Number` | Intrinsic height |

**Flags:** `.controls` `.autoplay`

Attribute families: `global`, `aria`, `data`, `media`.

### Icon

One of the built-in SVG icons, rendered inline.

```
Icon(name, size: …, tone: …, muted: …)
```

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

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The Markdown text |

Attribute families: `global`, `aria`, `data`.

### Blockquote

Quotation: the text, or its block as the quoted content.

```
Blockquote(content) { … }
```

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

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Paragraph

Block of text with paragraph spacing.

```
Paragraph(content, tone: …, size: …, align: …)
```

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

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Footer

Repeated at the bottom of every PDF page; a `<footer>` on the web.

```
Footer { … }
```

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### PageBreak

Forces a new PDF page.

```
PageBreak
```

Attribute families: `global`, `aria`, `data`.

## Slides

### Presentation

Root of a slide deck; its children must be slides.

```
Presentation { … }
```

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Slide

One slide, one PDF page; content is top-aligned and clipped at the bottom margin.

```
Slide { … }
```

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### TitleSlide

Cover slide: title and subtitle, vertically centred.

```
TitleSlide(title, subtitle: …)
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

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ImageSlide

Image slide with an optional caption.

```
ImageSlide(src: …, caption: …)
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
| `animate:` | `.fadeIn` `.fadeOut` `.slideUp` `.slideDown` `.slideLeft` `.slideRight` `.scaleIn` `.scaleOut` `.bounce` `.shake` `.pulse` `.spin` | Animation played when the element appears |
| `exit:` | `.fadeIn` `.fadeOut` `.slideUp` `.slideDown` `.slideLeft` `.slideRight` `.scaleIn` `.scaleOut` `.bounce` `.shake` `.pulse` `.spin` | Animation played when the element leaves |
| `speed:` | `.fast` `.normal` `.slow` | How fast its animation runs |
| `duration:` | `String` | Animation length, e.g. `300ms` |
| `delay:` | `String` | Wait before the animation starts |
| `easing:` | `String` | Timing function of the animation, e.g. `ease-out` |
| `stagger:` | `String` | Delay added per item of a list, e.g. `50ms` |
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

## Icons

The names `Icon(…)`, `IconButton(icon:)` and `Sidebar.Item(icon:)` draw; any other name shows as the word, and the compiler warns.

`close` `menu` `search` `home` `user` `settings` `check` `plus` `minus` `edit` `trash` `star` `heart` `mail` `bell` `download` `upload` `eye` `link` `calendar` `filter` `chevron-down` `chevron-right` `chevron-left` `info` `warning` `arrow-left` `arrow-right` `logout` `copy` `sun` `moon`
