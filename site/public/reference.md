# Components reference

> The components reference: every built-in element, its props, flags, events, slots, parts, attribute families and an example.

The page: https://webfluent.monzeromer.dev/reference

Generated from the compiler's registry (`wf registry --json`) by `scripts/docsite/components.py`, with an example of each from `scripts/docsite/examples.py`; edit those, not this file.

Every component is written `Name(positional, prop: value).flag { block }`. A **prop** is passed by name; the one **positional** prop, where there is one, comes first and unnamed. A **flag** is written after the parentheses with a dot: a `Bool` prop, or a case of one of the element's enum props (`Button("x").primary.lg`). An **enum prop** takes a case written `.case` (`Row(gap: .md)`). Every element also takes the universal props and events listed at the end, and the HTML attribute families named on it (`aria-*`, `data-*`, and the global attributes such as `id`, `class`, `title`, `role`, `tabindex`).

## Contents

- **Layout**: [`Container`](https://webfluent.monzeromer.dev/reference/container), [`Row`](https://webfluent.monzeromer.dev/reference/row), [`Column`](https://webfluent.monzeromer.dev/reference/column), [`Grid`](https://webfluent.monzeromer.dev/reference/grid), [`Stack`](https://webfluent.monzeromer.dev/reference/stack), [`Spacer`](https://webfluent.monzeromer.dev/reference/spacer), [`VirtualList`](https://webfluent.monzeromer.dev/reference/virtuallist), [`Splitter`](https://webfluent.monzeromer.dev/reference/splitter), [`Divider`](https://webfluent.monzeromer.dev/reference/divider), [`Masonry`](https://webfluent.monzeromer.dev/reference/masonry), [`AspectRatio`](https://webfluent.monzeromer.dev/reference/aspectratio), [`ScrollArea`](https://webfluent.monzeromer.dev/reference/scrollarea), [`Center`](https://webfluent.monzeromer.dev/reference/center), [`Sticky`](https://webfluent.monzeromer.dev/reference/sticky), [`PanelGroup`](https://webfluent.monzeromer.dev/reference/panelgroup), [`ClientOnly`](https://webfluent.monzeromer.dev/reference/clientonly), [`AppShell`](https://webfluent.monzeromer.dev/reference/appshell), [`Host`](https://webfluent.monzeromer.dev/reference/host), [`Portal`](https://webfluent.monzeromer.dev/reference/portal), [`Element`](https://webfluent.monzeromer.dev/reference/element)
- **Navigation**: [`Navbar`](https://webfluent.monzeromer.dev/reference/navbar), [`Sidebar`](https://webfluent.monzeromer.dev/reference/sidebar), [`Link`](https://webfluent.monzeromer.dev/reference/link), [`Tabs`](https://webfluent.monzeromer.dev/reference/tabs), [`Breadcrumb`](https://webfluent.monzeromer.dev/reference/breadcrumb), [`Menu`](https://webfluent.monzeromer.dev/reference/menu), [`NavigationMenu`](https://webfluent.monzeromer.dev/reference/navigationmenu), [`Menubar`](https://webfluent.monzeromer.dev/reference/menubar), [`ContextMenu`](https://webfluent.monzeromer.dev/reference/contextmenu), [`BottomNavigation`](https://webfluent.monzeromer.dev/reference/bottomnavigation), [`PrevNext`](https://webfluent.monzeromer.dev/reference/prevnext), [`OnThisPage`](https://webfluent.monzeromer.dev/reference/onthispage), [`Pagination`](https://webfluent.monzeromer.dev/reference/pagination), [`Stepper`](https://webfluent.monzeromer.dev/reference/stepper), [`CommandPalette`](https://webfluent.monzeromer.dev/reference/commandpalette)
- **Feedback**: [`HoverCard`](https://webfluent.monzeromer.dev/reference/hovercard), [`LoadingOverlay`](https://webfluent.monzeromer.dev/reference/loadingoverlay), [`Tour`](https://webfluent.monzeromer.dev/reference/tour), [`ConsentBanner`](https://webfluent.monzeromer.dev/reference/consentbanner), [`ConsentSettings`](https://webfluent.monzeromer.dev/reference/consentsettings), [`CircularProgress`](https://webfluent.monzeromer.dev/reference/circularprogress), [`Meter`](https://webfluent.monzeromer.dev/reference/meter), [`Popconfirm`](https://webfluent.monzeromer.dev/reference/popconfirm), [`Alert`](https://webfluent.monzeromer.dev/reference/alert), [`Toast`](https://webfluent.monzeromer.dev/reference/toast), [`Modal`](https://webfluent.monzeromer.dev/reference/modal), [`Popover`](https://webfluent.monzeromer.dev/reference/popover), [`Dialog`](https://webfluent.monzeromer.dev/reference/dialog), [`AlertDialog`](https://webfluent.monzeromer.dev/reference/alertdialog), [`Sheet`](https://webfluent.monzeromer.dev/reference/sheet), [`Spinner`](https://webfluent.monzeromer.dev/reference/spinner), [`Progress`](https://webfluent.monzeromer.dev/reference/progress), [`Skeleton`](https://webfluent.monzeromer.dev/reference/skeleton)
- **Media**: [`Lightbox`](https://webfluent.monzeromer.dev/reference/lightbox), [`Gallery`](https://webfluent.monzeromer.dev/reference/gallery), [`Image`](https://webfluent.monzeromer.dev/reference/image), [`Video`](https://webfluent.monzeromer.dev/reference/video), [`Audio`](https://webfluent.monzeromer.dev/reference/audio), [`Frame`](https://webfluent.monzeromer.dev/reference/frame), [`Icon`](https://webfluent.monzeromer.dev/reference/icon), [`Carousel`](https://webfluent.monzeromer.dev/reference/carousel)
- **Data display**: [`DescriptionList`](https://webfluent.monzeromer.dev/reference/descriptionlist), [`Timeline`](https://webfluent.monzeromer.dev/reference/timeline), [`AvatarGroup`](https://webfluent.monzeromer.dev/reference/avatargroup), [`Item`](https://webfluent.monzeromer.dev/reference/item), [`Sparkline`](https://webfluent.monzeromer.dev/reference/sparkline), [`RelativeTime`](https://webfluent.monzeromer.dev/reference/relativetime), [`Card`](https://webfluent.monzeromer.dev/reference/card), [`Table`](https://webfluent.monzeromer.dev/reference/table), [`List`](https://webfluent.monzeromer.dev/reference/list), [`Badge`](https://webfluent.monzeromer.dev/reference/badge), [`Tag`](https://webfluent.monzeromer.dev/reference/tag), [`Avatar`](https://webfluent.monzeromer.dev/reference/avatar), [`Tooltip`](https://webfluent.monzeromer.dev/reference/tooltip), [`Tree`](https://webfluent.monzeromer.dev/reference/tree), [`DataTable`](https://webfluent.monzeromer.dev/reference/datatable), [`Chart`](https://webfluent.monzeromer.dev/reference/chart), [`QrCode`](https://webfluent.monzeromer.dev/reference/qrcode)
- **Form**: [`ColorPicker`](https://webfluent.monzeromer.dev/reference/colorpicker), [`TransferList`](https://webfluent.monzeromer.dev/reference/transferlist), [`Editable`](https://webfluent.monzeromer.dev/reference/editable), [`Input`](https://webfluent.monzeromer.dev/reference/input), [`Select`](https://webfluent.monzeromer.dev/reference/select), [`Checkbox`](https://webfluent.monzeromer.dev/reference/checkbox), [`Radio`](https://webfluent.monzeromer.dev/reference/radio), [`Switch`](https://webfluent.monzeromer.dev/reference/switch), [`Slider`](https://webfluent.monzeromer.dev/reference/slider), [`DatePicker`](https://webfluent.monzeromer.dev/reference/datepicker), [`FileUpload`](https://webfluent.monzeromer.dev/reference/fileupload), [`Form`](https://webfluent.monzeromer.dev/reference/form), [`Textarea`](https://webfluent.monzeromer.dev/reference/textarea), [`Field`](https://webfluent.monzeromer.dev/reference/field), [`Fieldset`](https://webfluent.monzeromer.dev/reference/fieldset), [`InputGroup`](https://webfluent.monzeromer.dev/reference/inputgroup), [`RadioGroup`](https://webfluent.monzeromer.dev/reference/radiogroup), [`NumberInput`](https://webfluent.monzeromer.dev/reference/numberinput), [`Calendar`](https://webfluent.monzeromer.dev/reference/calendar), [`Listbox`](https://webfluent.monzeromer.dev/reference/listbox), [`Combobox`](https://webfluent.monzeromer.dev/reference/combobox), [`CheckboxGroup`](https://webfluent.monzeromer.dev/reference/checkboxgroup), [`InputOTP`](https://webfluent.monzeromer.dev/reference/inputotp), [`TagInput`](https://webfluent.monzeromer.dev/reference/taginput), [`Rating`](https://webfluent.monzeromer.dev/reference/rating), [`DateRangePicker`](https://webfluent.monzeromer.dev/reference/daterangepicker)
- **Actions**: [`Fab`](https://webfluent.monzeromer.dev/reference/fab), [`BackToTop`](https://webfluent.monzeromer.dev/reference/backtotop), [`CopyButton`](https://webfluent.monzeromer.dev/reference/copybutton), [`ThemeToggle`](https://webfluent.monzeromer.dev/reference/themetoggle), [`LocaleSwitch`](https://webfluent.monzeromer.dev/reference/localeswitch), [`Toggle`](https://webfluent.monzeromer.dev/reference/toggle), [`ToggleGroup`](https://webfluent.monzeromer.dev/reference/togglegroup), [`Button`](https://webfluent.monzeromer.dev/reference/button), [`IconButton`](https://webfluent.monzeromer.dev/reference/iconbutton), [`ButtonGroup`](https://webfluent.monzeromer.dev/reference/buttongroup), [`Dropdown`](https://webfluent.monzeromer.dev/reference/dropdown)
- **Typography**: [`Highlight`](https://webfluent.monzeromer.dev/reference/highlight), [`Text`](https://webfluent.monzeromer.dev/reference/text), [`Heading`](https://webfluent.monzeromer.dev/reference/heading), [`Code`](https://webfluent.monzeromer.dev/reference/code), [`Markdown`](https://webfluent.monzeromer.dev/reference/markdown), [`Unsafe`](https://webfluent.monzeromer.dev/reference/unsafe), [`Blockquote`](https://webfluent.monzeromer.dev/reference/blockquote)
- **Content**: [`Callout`](https://webfluent.monzeromer.dev/reference/callout), [`Kbd`](https://webfluent.monzeromer.dev/reference/kbd), [`EmptyState`](https://webfluent.monzeromer.dev/reference/emptystate), [`Stat`](https://webfluent.monzeromer.dev/reference/stat), [`CodeBlock`](https://webfluent.monzeromer.dev/reference/codeblock), [`Chip`](https://webfluent.monzeromer.dev/reference/chip), [`VisuallyHidden`](https://webfluent.monzeromer.dev/reference/visuallyhidden), [`Accordion`](https://webfluent.monzeromer.dev/reference/accordion)
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
Container {
    style { background: $color-surface; border: 1px dashed $color-border; padding: 1rem }
    Heading("Release notes").h3
    Text("Version 7.0 makes the compiler strict: what 6.x accepted and 7.0 refuses is an error with its fix.").muted
    Row(gap: .sm) {
        Badge("Compiler").primary
        Badge("Breaking").warning
    }
}
```

Renders with the class `wf-container`.

| Prop | Type | Meaning |
|---|---|---|
| `fluid:` | `Bool` | Drops the 1200px maximum width, so it spans its parent; the side padding stays |

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
    style { padding: 0.75rem 1rem; border: 1px solid $color-border; border-radius: $radius-md }
    Row(gap: .sm, align: .center) {
        Avatar(initials: "AL").primary
        Stack(gap: .xs) {
            Text("Ada Lovelace").bold
            Text("Owner").muted.sm
        }
    }
    Badge("Active").success
    Button("Manage").sm
}
```

Renders with the class `wf-row`.

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between its children, from the theme's spacing scale (`wf-gap--*`, `gap:`); a flag (`.lg`) or a value per breakpoint (`{ base: .sm, lg: .lg }`) |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Where its children sit across the row (vertically) (`align-items`); a flag for `.stretch`/`.baseline`, or a value per breakpoint |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | How its children share the free space along the row (`justify-content`); a flag for `.between`/`.around`/`.evenly`, or a value per breakpoint |
| `direction:` | `.row` `.column` | Which way its children run (`flex-direction`): a case, `direction: .column`, or a value per breakpoint, `{ base: .column, md: .row }` |
| `stacks:` | `Bool` | Below 768px it turns into a column (`flex-direction: column`); without it the row stays a wrapping row at every width |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.row` `.column` `.stacks`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Column

A child of the 12-column grid, spanning `span` columns.

```
Column(span: …, stacks: …) { … }
```

```wf
Row(gap: .md) {
    Column(span: { base: 12, md: 8 }) {
        Card {
            Card.Body {
                Heading("Article").h3
                Text("The main column takes eight of the twelve on a wide screen.")
            }
        }
    }
    Column(span: { base: 12, md: 4 }) {
        Card.outlined {
            Card.Body {
                Heading("Related").h3
                Link("Layout", to: "/learn")
            }
        }
    }
}
```

Renders with the class `wf-col`.

| Prop | Type | Meaning |
|---|---|---|
| `span:` | `Number` | How many of the row's 12 columns it takes, a whole number 1–12 (`T01` otherwise), less its share of the gap; or a value per breakpoint |
| `stacks:` | `Bool` | Full width below 768px; a `span` of 3 or 4 is half the width below 1024px |

**Flags:** `.stacks`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Grid

CSS grid with a fixed number of columns.

```
Grid(columns: …, gap: …, align: …) { … }
```

```wf
Grid(columns: { base: 1, sm: 2, md: 3 }, gap: .md) {
    Card.elevated {
        Card.Body {
            Text("Visitors").muted.sm
            Heading("12,480").h3
            Badge("+8%").success
        }
    }
    Card.elevated {
        Card.Body {
            Text("Sign-ups").muted.sm
            Heading("342").h3
            Badge("+3%").success
        }
    }
    Card.elevated {
        Card.Body {
            Text("Churn").muted.sm
            Heading("1.2%").h3
            Badge("-0.4%").danger
        }
    }
}
```

Renders with the class `wf-grid`.

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many equal columns (`repeat(n, 1fr)`), a whole 1–12 as a plain number (`T01` otherwise); any count in a value per breakpoint, `{ base: 1, md: 3 }`; it follows state |
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between its children, from the theme's spacing scale (`wf-gap--*`, `gap:`); a flag (`.lg`) or a value per breakpoint (`{ base: .sm, lg: .lg }`) |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Where its children sit in their cells, down (`align-items`) (`align-items`); a flag for `.stretch`/`.baseline`, or a value per breakpoint |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | `justify-content` on the grid's tracks; with its equal `1fr` columns the tracks fill the width, so it shows no effect; a flag for `.between`/`.around`/`.evenly` |
| `stacks:` | `Bool` | Two columns below 1024px and one below 768px; without it the columns stay as written at every width |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.stacks`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Stack

Vertical flex layout.

```
Stack(gap: …, align: …, justify: …) { … }
```

```wf
Stack(gap: .md) {
    style { max-width: 22rem; padding: 1rem; border: 1px solid $color-border; border-radius: $radius-md }
    Heading("Sign in").h3
    Text("Use the address you signed up with.").muted
    Button("Continue with email").primary.full
    Button("Continue with a passkey").full
}
```

Renders with the class `wf-stack`.

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `.xs` `.sm` `.md` `.lg` `.xl` | Space between its children, from the theme's spacing scale (`wf-gap--*`, `gap:`); a flag (`.lg`) or a value per breakpoint (`{ base: .sm, lg: .lg }`) |
| `align:` | `.start` `.center` `.end` `.stretch` `.baseline` | Where its children sit across it (horizontally): `.stretch` makes each child full width (`align-items`); a flag for `.stretch`/`.baseline`, or a value per breakpoint |
| `justify:` | `.start` `.center` `.end` `.between` `.around` `.evenly` | How its children share the free space down the column, seen only when it is taller than its children (`justify-content`); a flag for `.between`/`.around`/`.evenly`, or a value per breakpoint |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Spacer

Vertical space; `md` when no size is given.

```
Spacer(size: …)
```

```wf
Heading("Before you deploy").h3
Text("Run the checks, then the tests.")
Spacer.lg
Button("Deploy").primary
Spacer.sm
Text("Last deployed 3 hours ago.").muted.sm
```

Renders with the class `wf-spacer`.

| Prop | Type | Meaning |
|---|---|---|
| `size:` | `.xs` `.sm` `.md` `.lg` `.xl` | How tall the empty block is, from the theme's spacing scale; written as a flag, `Spacer.lg` |

**Flags:** `.xs` `.sm` `.md` `.lg` `.xl`

Attribute families: `global`, `aria`, `data`.

### VirtualList

A scrolling box that draws only the rows in view of the one keyed `for` inside it — for a tree, search results or a diff of thousands of rows.

```
VirtualList(rowHeight: …, estimate: …, reveal: …) { … }
```

```wf
state rows = (0..500).map(i => "Commit #{i + 1}")
state picked = "Commit #250"
Text("Picked: {picked}").muted
VirtualList(rowHeight: 36, reveal: picked, label: "Commits") {
    style { height: 260px; border: 1px solid $color-border; border-radius: $radius-md }
    for r in rows by r {
        Row(justify: .between, align: .center) {
            style { height: 36px; padding: 0 0.75rem }
            Text(r)
            Button("Pick").sm { on click { picked = r } }
        }
    }
}
```

Renders with the class `wf-virtual-list`.

| Prop | Type | Meaning |
|---|---|---|
| `rowHeight:` | `Number` | Every row's height, in px: the fast path. Unset, each row is measured once drawn |
| `estimate:` | `Number` | The height a row not yet drawn stands for while there is no `rowHeight`, in px |
| `reveal:` | `String or Number` | A row's key, as its `for … by` gives it: when it changes, that row is scrolled into view — the selection, the current match |
| `label:` | `String` | The scrolling box's accessible name (`aria-label`): what the list is, for a screen reader |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Splitter

A handle that sets the size of a region beside it: dragged, between `min:` and `max:`; double-clicked or Enter, back to `reset:`; the arrow keys move it.

```
Splitter(bind: …, min: …, max: …)
```

```wf
state side = 220
Text("Sidebar: {side}px").muted.sm
Row {
    style { height: 180px; border: 1px solid $color-border; border-radius: $radius-md; overflow: hidden }
    Stack(gap: .xs) {
        style { width: {side}px; padding: 0.75rem; background: $color-surface }
        Text("Files").bold
        Text("App.wf")
        Text("Home.wf")
    }
    Splitter(bind: side, min: 140, max: 400, reset: 220, label: "Sidebar width")
    Stack {
        style { flex: 1; padding: 0.75rem }
        Text("Drag the handle, or focus it and use the arrow keys. Double-click puts it back.")
    }
}
```

Renders with the class `wf-splitter`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A Number of px it reads and writes: the size of the region beside it, which your own `style { width: {side}px }` applies; without it the handle does nothing |
| `min:` | `Number` | The smallest size it writes, in px; Home goes there |
| `max:` | `Number` | The largest size it writes, in px; End goes there |
| `reset:` | `Number` | The size a double-click or Enter writes, taken as written (not held to `min:`): `0` can mean the region's default share. Unset, neither does anything |
| `label:` | `String` | The handle's accessible name (`aria-label`): what it resizes, for a screen reader |
| `horizontal:` | `Bool` | A bar between two regions one above the other: it moves up and down, and the arrow keys are Up and Down |
| `reverse:` | `Bool` | It sizes the region after it, not before — a pane on the right, a panel below — so a drag toward the start grows it |

**Flags:** `.horizontal` `.reverse`

Attribute families: `global`, `aria`, `data`.

### Divider

Horizontal rule.

```
Divider
```

```wf
Card {
    Card.Body {
        Heading("Billing").h3
        Text("Pro plan, billed yearly.")
        Divider
        Row(justify: .between) {
            Text("Next invoice").muted
            Text("$240.00").bold
        }
    }
}
```

Renders with the class `wf-divider`.

Attribute families: `global`, `aria`, `data`.

### Masonry

Its children in `columns:` columns, each placed under the last in its column, keeping its own height.

```
Masonry(columns: …) { … }
```

```wf
Masonry(columns: { base: 1, sm: 2, md: 3 }) {
    Card { Card.Body { Heading("Shipping").h3  Text("Free over $50.") } }
    Card {
        Card.Body {
            Heading("Returns").h3
            Text("Thirty days, no questions. Keep the box and the receipt, and send it back from any post office.")
        }
    }
    Card { Card.Body { Heading("Support").h3  Text("Every day, 8–8.") } }
    Card { Image(src: "/og.png", alt: "The WebFluent card") }
    Card { Card.Body { Heading("Gift cards").h3  Text("Sent by email, any amount.") } }
    Card { Card.Body { Heading("Warranty").h3  Text("Two years on everything we make.") } }
}
```

Renders with the class `wf-masonry`.

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many columns at most (CSS `columns`), each at least 12rem wide, so a narrow box has fewer; or one per breakpoint, `{ base: 1, md: 3 }` |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### AspectRatio

A box that keeps its shape as its width changes — `ratio: 16 / 9` — its child filling it: a picture, a video, a frame.

```
AspectRatio(ratio: …) { … }
```

```wf
Grid(columns: 2, gap: .md) {
    AspectRatio(ratio: 16 / 9) {
        Image(src: "/og.png", alt: "The WebFluent card, in 16:9")
    }
    AspectRatio(ratio: 1) {
        Image(src: "/icon-512.png", alt: "The WebFluent icon, square")
    }
}
```

Renders with the class `wf-aspect-ratio`.

| Prop | Type | Meaning |
|---|---|---|
| `ratio:` (required) | `Number` | Its width over its height, `16 / 9` (CSS `aspect-ratio`): a literal is a rule of the sheet, one that reads state follows it |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ScrollArea

A box that scrolls within `height:`, with a thin scrollbar, its edges fading where there is more, and a Tab stop so a keyboard can scroll it.

```
ScrollArea(height: …, label: …, horizontal: …) { … }
```

```wf
ScrollArea(height: "200px", label: "Build log") {
    style { border: 1px solid $color-border; border-radius: $radius-md; padding: 0.5rem 0.75rem }
    for line in (1..=40).map(i => "[{i}] compiled src/pages/Page{i}.wf") by line {
        Text(line).sm
    }
}
```

Renders with the class `wf-scroll-area`.

| Prop | Type | Meaning |
|---|---|---|
| `height:` | `String` | A CSS length, `300px`: its `max-height`, past which it scrolls. Unset, it grows with what is inside unless its parent holds it |
| `label:` (required) | `String` | The region's accessible name (`aria-label`); it is a Tab stop so a keyboard can scroll it |
| `horizontal:` | `Bool` | It scrolls across, not down (`overflow-x: auto`, `overflow-y: hidden`), its fades at the start and end |

**Flags:** `.horizontal`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Center

Its children in the middle of it, across and down.

```
Center { … }
```

```wf
Center {
    style { height: 200px; background: $color-surface; border-radius: $radius-md }
    Stack(gap: .sm, align: .center) {
        Spinner
        Text("Loading your projects…").muted
    }
}
```

Renders with the class `wf-center`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Sticky

A box that sticks to the top of its scrolling parent (`offset:` below it) as the page scrolls past, marked `data-stuck` while it is stuck.

```
Sticky(offset: …, bottom: …) { … }
```

```wf
ScrollArea(height: "240px", label: "Changelog") {
    style { border: 1px solid $color-border; border-radius: $radius-md }
    Sticky {
        style { background: $color-background; padding: 0.5rem 0.75rem; border-bottom: 1px solid $color-border }
        Row(justify: .between, align: .center) {
            Text("Changelog").bold
            Badge("7.0").primary
        }
    }
    for v in (0..20).map(i => 19 - i).map(n => "7.0.{n}: fixes and small changes") by v {
        Text(v) { style { padding: 0.5rem 0.75rem } }
    }
}
```

Renders with the class `wf-sticky`.

| Prop | Type | Meaning |
|---|---|---|
| `offset:` | `String` | A CSS length, `64px`: how far from the top it sticks (from the bottom with `.bottom`) |
| `bottom:` | `Bool` | It sticks to the bottom of its scrolling parent instead of the top; `.bottom` or `bottom: true` |

**Flags:** `.bottom`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### PanelGroup

Panels side by side (or `.vertical`, one above another) with a handle between each two that resizes them: dragged, or the arrow keys on it. `bind:` holds their sizes in percent — a `persist` keeps them across visits — and `on resize(e)` hears them as `e.detail`.

```
PanelGroup(bind: …, vertical: …, label: …) { … }
```

```wf
state sizes: [Number] = [30, 70]
Text("Sizes: {sizes[0] ?? 0}% and {sizes[1] ?? 0}%").muted.sm
PanelGroup(bind: sizes, label: "Resize the sidebar") {
    style { height: 200px; border: 1px solid $color-border; border-radius: $radius-md }
    PanelGroup.Panel(min: 15) {
        Stack(gap: .xs) {
            style { padding: 0.75rem }
            Text("Inbox").bold
            Text("Archive")
            Text("Spam")
        }
    }
    PanelGroup.Panel(min: 40) {
        Text("Drag the handle, or use the arrow keys on it.") { style { padding: 0.75rem } }
    }
}
```

Renders with the class `wf-panel-group`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A list of Numbers, each panel's size in percent, `[30, 70]`, written as they move; a list of another length is ignored |
| `vertical:` | `Bool` | Panels one above another; the handles move up and down |
| `label:` | `String` | Each handle's accessible name (`aria-label`): what it resizes |

**Flags:** `.vertical`

**Events of its own:** `on resize`

**Parts:**

- `PanelGroup.Panel(min: …, collapsible: …) { … }` — One panel.
  - `min:` `Number` — The least it may be, in percent of the group
  - `collapsible:` `Bool` — Enter or a double press on the handle after it folds it away, its share to its neighbour; again brings it back
  - flags: `.collapsible`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ClientOnly

Its block drawn in the browser alone, never in the static paint, a template or a PDF: a clock, the reader's own data. A `placeholder { … }` block is what those show in its place.

```
ClientOnly { … }
```

```wf
Card {
    Card.Body {
        Text("Your local time").muted.sm
        ClientOnly {
            placeholder { Skeleton(height: "28px", width: "120px") }
            Heading("{now(every: 1.seconds):.time}").h3
        }
    }
}
```

Renders with the class `wf-client-only`.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### AppShell

An app's frame: `AppShell.Header` across the top, a `Sidebar` down the start (a drawer on a narrow screen), the `Router`'s page beside it, and an `AppShell.Footer`.

```
AppShell { … }
```

```wf
AppShell {
    style { min-height: 320px; border: 1px solid $color-border }
    AppShell.Header {
        Row(justify: .between, align: .center) {
            style { flex: 1 }
            Text("Ledger").bold
            Button("Sign out").sm
        }
    }
    Sidebar {
        style { height: auto }
        Sidebar.Item(to: "/learn", icon: "book") { Text("Learn") }
        Sidebar.Item(to: "/reference", icon: "list") { Text("Reference") }
    }
    Stack(gap: .sm) {
        style { padding: 1rem }
        Heading("Dashboard").h3
        Text("The page the router draws sits here.")
    }
    AppShell.Footer { Text("© 2026 Ledger").muted.sm }
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
state level = 40
Slider(bind: level, min: 0, max: 100, label: "Level")
Host(tag: "canvas", aria-label: "A bar at {level}%",
     mount: (node) => node.getBoundingClientRect(),
     update: (box) => console.log("redraw at {level}% across {box.width}px"),
     cleanup: (box) => console.log("canvas gone")) {
    style { width: 100%; height: 120px; border: 1px dashed $color-border }
}
```

Renders with the class `wf-host`.

| Prop | Type | Meaning |
|---|---|---|
| `tag:` | `String` | The element to make: `div`, `span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul` or `table`; any other is a `div` (`V10`) |

Attribute families: `global`, `aria`, `data`.

### Portal

Its block, drawn into a node somebody else's code owns: an element a script hands over, or a `ref:` handle. When `into:` names another node the block moves there with its nodes and state; `null` keeps it aside until a node comes back; it goes when the page, branch or list item it is written in leaves. What is inside belongs to where it is written: its handlers, `emit` and lifetimes.

```
Portal(into: …) { … }
```

```wf
state count = 0
Stack(gap: .sm, ref: panel) {
    style { padding: 1rem; border: 1px dashed $color-border; border-radius: $radius-md }
    Text("A node somebody else's code owns:").muted.sm
}
Portal(into: panel) {
    Row(gap: .sm, align: .center) {
        Text("Clicked {count} times")
        Button("Click").primary.sm { on click { count = count + 1 } }
    }
}
```

Renders with the class `wf-portal`.

| Prop | Type | Meaning |
|---|---|---|
| `into:` (required) | `Element?` | Where it draws: an element, a `ref:` handle, or `null` to keep it aside; read again whenever what it reads changes |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Element

A custom element a script defines, placed by its tag: `Element("stripe-pricing-table", publishableKey: key) { on ready { … } }`. Every named argument is an attribute, every `on` handler an event it fires, and the block its children.

```
Element(tag) { … }
```

```wf
state loaded = false
Element("stripe-pricing-table", pricingTableId: "prctbl_1Pq", publishableKey: "pk_test_51Hx") {
    on ready { loaded = true }
    if !loaded { Text("Loading the prices…").muted }
}
```

| Prop | Type | Meaning |
|---|---|---|
| `tag` (positional) | `String` | The custom element's tag: lower case, with a hyphen, as every custom element's is (`E116` otherwise) |

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
    Navbar.Brand { Link("WebFluent", to: "/") }
    Navbar.Links {
        Link("Learn", to: "/learn", active: .prefix)
        Link("Reference", to: "/reference", active: .prefix)
        Link("Recipes", to: "/recipes")
    }
    Navbar.Actions {
        ThemeToggle
        Button("Try it").primary.sm { on click { navigate("/playground") } }
    }
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
Row {
    style { height: 360px; border: 1px solid $color-border }
    Sidebar {
        style { height: 100% }
        Sidebar.Header { Text("Docs").bold }
        Sidebar.Item(to: "/", icon: "home") { Text("Home") }
        Sidebar.Item(to: "/learn", icon: "book", active: .prefix) { Text("Learn") }
        Sidebar.Item(to: "/reference", icon: "list", active: .prefix) { Text("Reference") }
        Sidebar.Item(to: "/recipes", icon: "layers") { Text("Recipes") }
        Sidebar.Divider
        Sidebar.Item(to: "/playground", icon: "play") { Text("Playground") }
    }
    Stack(gap: .sm) {
        style { flex: 1; padding: 24px }
        Heading("Reference").h3
        Text("Every built-in, with its props and an example.").muted
    }
}
```

Renders with the class `wf-sidebar`.

**Parts:**

- `Sidebar.Header { … }` — The sidebar's heading area.
- `Sidebar.Item(to: …, icon: …, active: …) { … }` — One navigation entry.
  - `to:` `Path` — The route it links to, navigated in place; while it matches the current route the item carries `.active` and `aria-current="page"`. Unset, the item is not a link
  - `icon:` `String` — The name of a built-in icon, drawn before its words
  - `active:` `.prefix` — How it decides it is the current page: unset, the route must equal `to`; `.prefix` also matches routes beneath it
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
Stack(gap: .sm) {
    Link("Start the guide", to: "/learn")
    Link("Every component", to: "/reference", active: .prefix)
    Link(to: "/recipes") {
        Icon("book")
        Text("Recipes for common screens")
    }
    Link("The source on GitHub", to: "https://github.com/webfluent/webfluent", target: "_blank")
}
```

Renders with the class `wf-link`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The link's words; a block written after it may be its content instead |
| `to:` | `Path` | The route or address it goes to, navigated in place; while it matches the current route the link carries `.active` and `aria-current="page"` |
| `active:` | `.prefix` | How it decides it is the current page: unset, the route must equal `to`; `.prefix` also matches routes beneath it |

**Flags:** `.prefix`

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `anchor`.

### Tabs

Tab strip switching between its pages: the arrow keys move between the tabs (the other way round in a right-to-left page), Home and End go to the ends.

```
Tabs(bind: …, value: …, orientation: …) { … }
```

```wf
state tab = "profile"
Tabs(bind: tab, label: "Account") {
    Tabs.Page("Profile", value: "profile") {
        Text("Your name, photo and the email we write to.")
    }
    Tabs.Page("Billing", value: "billing") {
        Text("Pro plan, renews on 1 March.")
    }
    Tabs.Page("Danger zone", value: "danger", disabled: true) {
        Text("Only an owner can delete the account.")
    }
}
Text("Open: {tab}").muted.sm
```

Renders with the class `wf-tabs`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The open tab, read and written: a page's `value:`, or its place from 0 when the pages give none; a value no page has opens the first |
| `value:` | `String or Number` | The open tab, read only: a press does not change it, only `on change(v)` hears the one picked — for a page that sets it itself |
| `orientation:` | `.horizontal` `.vertical` | Which way the tabs run, and which arrow keys move between them; a flag, `.vertical` |
| `activation:` | `.automatic` `.manual` | Whether moving the focus opens a tab; a flag, `.manual` |
| `label:` | `String` | The tab strip's accessible name (`aria-label` on its `tablist`) |

**Flags:** `.horizontal` `.vertical` `.automatic` `.manual`

**Events of its own:** `on change`

**Parts:**

- `Tabs.Page(label, value: …, disabled: …) { … }` — One tab and its panel.
  - `label:` `String` — The words on its tab button
  - `value:` `String or Number` — What the Tabs' `bind:` holds while this tab is open
  - `disabled:` `Bool` — Shown, but cannot be opened: the button is disabled and the arrow keys pass over it; `.disabled`, or `disabled: value` to follow state
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
    Breadcrumb.Item(to: "/reference") { Text("Reference") }
    Breadcrumb.Item { Text("Breadcrumb") }
}
```

Renders with the class `wf-breadcrumb`.

**Parts:**

- `Breadcrumb.Item(to: …) { … }` — One step of the trail.
  - `to:` `Path` — The route the step links to, followed through the router as a `Link` is. Without it the step is the current page, `aria-current="page"`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Menu

A menu above everything: opened by its `trigger:` button, or by `visible:` at an `anchor:` or a point `at:` — a context menu. Escape, a press outside or a choice close it, write `false` back to `visible:` and fire `on close`.

```
Menu(trigger: …, label: …, visible: …) { … }
```

```wf
state said = "Nothing chosen yet"
state pinned = false
Menu(trigger: "File actions").bottomStart {
    Menu.Item(shortcut: "F2") {
        on click { said = "Rename" }
        Text("Rename")
    }
    Menu.Item(shortcut: "mod+d") {
        on click { said = "Duplicate" }
        Text("Duplicate")
    }
    Menu.Item(checked: pinned) {
        on click { pinned = !pinned }
        Text("Pinned")
    }
    Menu.Sub(label: "Move to") {
        Menu.Item { on click { said = "Moved to Archive" }  Text("Archive") }
        Menu.Item { on click { said = "Moved to Drafts" }  Text("Drafts") }
    }
    Menu.Divider
    Menu.Item(shortcut: "mod+backspace").danger {
        on click { said = "Delete" }
        Text("Delete")
    }
}
Text(said).muted.sm
```

Renders with the class `wf-menu`.

| Prop | Type | Meaning |
|---|---|---|
| `trigger:` | `String` | The words on the button that opens it |
| `label:` | `String` | The menu list's accessible name (`aria-label`); it is not the button's words, which `trigger:` gives |
| `visible:` | `State` | A Bool state that opens and closes it: the button sets it, and Escape, a press outside or a choice write `false`. With no `trigger:`, state alone opens it |
| `anchor:` | `Element or Point` | What it is placed against: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }` |
| `at:` | `Point` | A point to place it at, `{ x: e.clientX, y: e.clientY }`: where the pointer asked for a context menu |
| `placement:` | `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room, and slides to stay on screen |
| `offset:` | `Number` | The gap between it and its anchor, in px |

**Flags:** `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd`

**Events of its own:** `on close`

**Parts:**

- `Menu.Item(shortcut: …, checked: …, disabled: …, danger: …) { … }` — One entry of the menu; `on click` is what choosing it does.
  - `shortcut:` `String` — The keys that do the same, `mod+b`, shown at its end as this system writes them (`⌘B`, `Ctrl+B`); in a `Menubar` the keys also run it
  - `checked:` `Bool` — A check mark that follows the value (`menuitemcheckbox`, `aria-checked`): the item turns something on and off
  - `disabled:` `Bool` — Shown dimmed (`aria-disabled`), skipped by the arrow keys, and cannot be chosen
  - `danger:` `Bool` — What it does cannot be taken back — Delete, Discard: drawn in the danger colour
  - flags: `.checked` `.disabled` `.danger`
- `Menu.Sub(label: …) { … }` — An entry that opens a menu of its own beside it.
  - `label:` `String` — The words of the entry that opens its menu beside it
- `Menu.Divider` — A rule between groups of entries.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### NavigationMenu

A site's main navigation whose entries open panels (a mega menu) as disclosures: one open at a time, the arrow keys along the row, Down into a panel, Escape, a press outside or the focus leaving closes it.

```
NavigationMenu(label: …) { … }
```

```wf
NavigationMenu(label: "Main") {
    NavigationMenu.Item("Learn") {
        Stack(gap: .sm) {
            Link("The guide", to: "/learn")
            Link("Data and requests", to: "/learn/15-data")
            Link("Recipes", to: "/recipes")
        }
    }
    NavigationMenu.Item("Reference") {
        Stack(gap: .sm) {
            Link("Components", to: "/reference")
            Link("Diagnostics", to: "/errors")
        }
    }
    NavigationMenu.Link("Playground", to: "/playground")
    NavigationMenu.Link("Showcase", to: "/showcase")
}
```

Renders with the class `wf-nav-menu`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The navigation's accessible name (`aria-label` on its `<nav>`) |

**Parts:**

- `NavigationMenu.Item(label) { … }` — An entry that opens a panel: its words are its button, its block the panel.
  - `label:` `String` — The words on the button that opens its panel; the block is the panel
- `NavigationMenu.Link(label, to: …, active: …)` — An entry that is a link, beside the ones that open panels.
  - `label:` `String` — The link's words
  - `to:` `Path` — The route it links to, navigated in place; while it matches the current route it carries `.active` and `aria-current="page"`
  - `active:` `.prefix` — How it decides it is the current page: unset, the route must equal `to`; `.prefix` also matches routes beneath it
  - flags: `.prefix`
- `NavigationMenu.List { … }` — Its entries. Drawn for it.
- `NavigationMenu.Trigger(label) { … }` — An entry's button. Drawn for it.
  - `label:` `String` — The button's words; written for you from a `NavigationMenu.Item`'s label
- `NavigationMenu.Panel { … }` — An entry's panel. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Menubar

File, Edit and View, as in a desktop app: one Tab stop, the arrow keys along it; Down or Enter opens a menu, and with one open Left and Right open the one beside it.

```
Menubar(label: …) { … }
```

```wf
state said = "Pick a command"
Menubar(label: "Editor") {
    Menubar.Menu("File") {
        Menu.Item(shortcut: "mod+n") { on click { said = "New file" }  Text("New file") }
        Menu.Item(shortcut: "mod+o") { on click { said = "Open…" }  Text("Open…") }
        Menu.Divider
        Menu.Item(shortcut: "mod+s") { on click { said = "Saved" }  Text("Save") }
    }
    Menubar.Menu("Edit") {
        Menu.Item(shortcut: "mod+z") { on click { said = "Undo" }  Text("Undo") }
        Menu.Item(shortcut: "mod+shift+z") { on click { said = "Redo" }  Text("Redo") }
    }
    Menubar.Menu("View") {
        Menu.Sub(label: "Zoom") {
            Menu.Item(shortcut: "mod+=") { on click { said = "Zoom in" }  Text("Zoom in") }
            Menu.Item(shortcut: "mod+-") { on click { said = "Zoom out" }  Text("Zoom out") }
        }
    }
}
Text(said).muted.sm
```

Renders with the class `wf-menubar`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The bar's accessible name (`aria-label` on its `menubar`) |

**Parts:**

- `Menubar.Menu(label) { … }` — One menu of the bar: its button's words, and its `Menu.Item`s, `Menu.Sub`s and `Menu.Divider`s.
  - `label:` `String` — The words on its button, and the name of the menu it opens

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ContextMenu

What it wraps, and a menu a right-click, a long press, the menu key or Shift+F10 inside it opens where it happened.

```
ContextMenu { … }
```

```wf
state said = "Right-click the file, or focus it and press Shift+F10"
ContextMenu {
    Card.outlined {
        Row(gap: .sm, align: .center) {
            Icon("file")
            Text("quarterly-report.pdf")
        }
    }
    ContextMenu.Items(label: "File actions") {
        Menu.Item(shortcut: "mod+o") { on click { said = "Opened" }  Text("Open") }
        Menu.Item(shortcut: "F2") { on click { said = "Renamed" }  Text("Rename") }
        Menu.Divider
        Menu.Item.danger { on click { said = "Deleted" }  Text("Delete") }
    }
}
Text(said).muted.sm
```

Renders with the class `wf-context-menu`.

**Parts:**

- `ContextMenu.Items(label: …) { … }` — The menu: `Menu.Item`s, `Menu.Sub`s and `Menu.Divider`s.
  - `label:` `String` — The menu's accessible name (`aria-label`)

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### BottomNavigation

A phone's tab bar along the bottom of the screen: each entry an icon over its words, at least 44px to press, clear of the home indicator (the safe-area inset); the current route's is marked.

```
BottomNavigation(label: …) { … }
```

```wf
Stack(gap: .sm) {
    style { width: 360px; height: 240px; padding: 16px; contain: layout; overflow: hidden; border: 1px solid $color-border; border-radius: 24px }
    Heading("Inbox").h3
    Text("Three new replies since this morning.").muted
    BottomNavigation(label: "Sections") {
        BottomNavigation.Item("Home", to: "/", icon: "home")
        BottomNavigation.Item("Learn", to: "/learn", icon: "book").prefix
        BottomNavigation.Item("Reference", to: "/reference", icon: "list").prefix
        BottomNavigation.Item("Play", to: "/playground", icon: "play")
    }
}
```

Renders with the class `wf-bottom-nav`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The bar's accessible name (`aria-label` on its `<nav>`) |

**Parts:**

- `BottomNavigation.Item(label, to: …, active: …, icon: …)` — One entry: its words, where it goes, and its icon.
  - `label:` `String` — Its words, under the icon
  - `to:` `Path` — The route it links to, navigated in place; while it matches the current route it carries `.active` and `aria-current="page"`
  - `active:` `.prefix` — How it decides it is the current page: unset, the route must equal `to`; `.prefix` also matches routes beneath it
  - `icon:` `String` — The name of a built-in icon, drawn over its words
  - flags: `.prefix`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### PrevNext

The pages before and after this one in an ordered set — a guide's chapters: each a link that says which way it goes and the page's title.

```
PrevNext(label: …) { … }
```

```wf
PrevNext {
    PrevNext.Prev("Forms", to: "/learn")
    PrevNext.Next("Data and requests", to: "/learn/15-data")
}
```

Renders with the class `wf-prev-next`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Its accessible name (`aria-label` on its `<nav>`) |

**Parts:**

- `PrevNext.Prev(title, to: …, label: …)` — The page before: its title, and where it is.
  - `title:` `String` — The previous page's title, shown bold under the word
  - `to:` `Path` — The route of the previous page, linked with `rel="prev"` and navigated in place
  - `label:` `String` — The small word above the title
- `PrevNext.Next(title, to: …, label: …)` — The page after: its title, and where it is.
  - `title:` `String` — The next page's title, shown bold under the word
  - `to:` `Path` — The route of the next page, linked with `rel="next"` and navigated in place
  - `label:` `String` — The small word above the title

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### OnThisPage

The page's headings as links, the one being read marked as the reader scrolls: a table of contents beside a long page.

```
OnThisPage(levels: …, label: …)
```

```wf
Row(gap: .lg, align: .start) {
    Stack {
        Heading("Deploying").h1
        Heading("Build the site").h2
        Text("Run wf build; the output is in build/.")
        Heading("Choose a host").h2
        Heading("Static hosts").h3
        Text("Any host that serves files.")
        Heading("Set the base path").h2
        Text("Under a folder, name it in the config.")
    }
    OnThisPage(levels: 2, label: "On this page")
}
```

Renders with the class `wf-on-this-page`.

| Prop | Type | Meaning |
|---|---|---|
| `levels:` | `Number` | How many heading levels it lists, from `h2`: `1` is `h2` alone, `2` adds `h3`, up to 5 |
| `label:` | `String` | Its title, and its accessible name (`aria-label` on its `<nav>`) |

**Parts:**

- `OnThisPage.Title(text)` — Its title. Drawn for it.
  - `text:` `String` — The title's words; written for you from the `OnThisPage`'s `label:`

Attribute families: `global`, `aria`, `data`.

### Pagination

Pages of a list: Previous, the first and last pages, those around the current one with a gap for the rest, and Next. The current page is `aria-current="page"`; `bind:` holds it, from 1.

```
Pagination(bind: …, pages: …, siblings: …)
```

```wf
state page = 4
Pagination(bind: page, pages: 12, siblings: 1, label: "Results")
Text("Showing page {page} of 12").muted.sm
```

Renders with the class `wf-pagination`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | A Number state, the current page counted from 1: the buttons write it |
| `pages:` (required) | `Number` | How many pages there are; the last button and Next stop there |
| `siblings:` | `Number` | How many page buttons it shows on each side of the current one; the first and last always show, `…` stands for the rest |
| `label:` | `String` | Its accessible name (`aria-label` on its `<nav>`) |

**Parts:**

- `Pagination.Prev(disabled: …) { … }` — The previous page. Drawn for it.
  - `disabled:` `Bool` — Written by `Pagination` itself: true on the first page, when there is nowhere to go back to
  - flags: `.disabled`
- `Pagination.Next(disabled: …) { … }` — The next page. Drawn for it.
  - `disabled:` `Bool` — Written by `Pagination` itself: true on the last page
  - flags: `.disabled`
- `Pagination.Page(text) { … }` — A page. Drawn for it.
  - `text:` `String` — The page number it shows; written by `Pagination`
- `Pagination.Gap(text) { … }` — The pages left out. Drawn for it.
  - `text:` `String` — The `…` standing for the pages not shown; written by `Pagination`

Attribute families: `global`, `aria`, `data`.

### Stepper

Steps taken in order — an account, then billing, then a summary — with each step's state (done, current, an error) and the current step's block shown. `bind:` holds the current step, from 0; `.linear` keeps a step out of reach until the ones before it are passed.

```
Stepper(bind: …, linear: …) { … }
```

```wf
state step = 0
state billed = false
Stepper(bind: step) {
    Stepper.Step("Account", done: step > 0) {
        Text("Your name and email.")
        Button("Next").primary { on click { step = 1 } }
    }
    Stepper.Step("Billing", done: billed) {
        Text("A card, or an invoice each month.")
        Button("Add a card").primary { on click { billed = true  step = 2 } }
    }
    Stepper.Step("Confirm") {
        Text("Check it over, then start.")
        Button("Start the trial").success
    }
}
Text("Step {step + 1} of 3").muted.sm
```

Renders with the class `wf-stepper`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | A Number state, the current step counted from 0: pressing a step's button writes it, and only that step's block shows |
| `linear:` | `Bool` | A step after the current one cannot be pressed: its button is disabled |

**Flags:** `.linear`

**Parts:**

- `Stepper.Step(title, done: …, error: …) { … }` — One step: its title in the list, its block shown while it is the current one.
  - `title:` `String` — Its title, on its button in the list and as its panel's name
  - `done:` `Bool` — Passed: drawn with a check while it is not the current step; `.done`, or `done: value` to follow state
  - `error:` `Bool` — Something in it needs fixing: drawn as an error, before current or done; `.error`, or `error: value` to follow state
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
state said = "Nothing run yet"
Button("Search commands…") { on click { palette = true } }
Text(said).muted.sm
CommandPalette(open: palette, label: "Commands", placeholder: "Type a command or a page") {
    CommandPalette.Group("Pages") {
        CommandPalette.Item("Reference", icon: "list") { on select { navigate("/reference") } }
        CommandPalette.Item("Recipes", icon: "book") { on select { navigate("/recipes") } }
    }
    CommandPalette.Group("Actions") {
        CommandPalette.Item("New file", icon: "file-plus", shortcut: "mod+n") { on select { said = "New file" } }
        CommandPalette.Item("Copy link", icon: "copy", shortcut: "mod+shift+c") { on select { said = "Link copied" } }
        CommandPalette.Item("Switch theme", icon: "moon") { on select { said = "Theme switched" } }
    }
}
```

Renders with the class `wf-command-palette`.

| Prop | Type | Meaning |
|---|---|---|
| `open:` (required) | `State` | A Bool state that shows it (a modal), written `false` when it closes — Escape, a press outside, a choice |
| `label:` | `String` | Its title, and the name of its field and its list |
| `placeholder:` | `String` | Shown in the empty field |

**Parts:**

- `CommandPalette.Group(title) { … }` — Items under a heading.
  - `title:` `String` — Its heading, and the group's accessible name
- `CommandPalette.Item(label, icon: …, shortcut: …) { … }` — One thing to do or go to; `on select` is what it does.
  - `label:` `String` — Its words, which the typing matches
  - `icon:` `String` — The name of a built-in icon, drawn before its words
  - `shortcut:` `String` — Its keys, shown at its end as this system writes them (`⌘H`); only shown — the key is not bound
- `CommandPalette.List { … }` — What matches. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Feedback

### HoverCard

A preview card at a link or a button: shown after `delay:` ms of hover, at once on focus; it stays while the pointer is on it, and goes when the pointer leaves both, on Escape or on a press outside.

```
HoverCard(delay: …) { … }
```

```wf
Text("Reviewed by") {
    HoverCard(delay: 300) {
        Link("@ada", to: "/showcase")
        HoverCard.Content {
            Row(gap: .sm, align: .center) {
                Avatar(initials: "AL").primary
                Stack(gap: .xs) {
                    Text("Ada Lovelace").bold
                    Text("Wrote the first program.").muted.sm
                }
            }
        }
    }
}
```

Renders with the class `wf-hover-card`.

| Prop | Type | Meaning |
|---|---|---|
| `delay:` | `Number` | How long the pointer rests on the control before the card shows, in ms; focus shows it at once, and it goes 300 ms after the pointer leaves both |

**Parts:**

- `HoverCard.Content { … }` — What the card shows. The rest of the block is what it is at.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### LoadingOverlay

A region covered while it is busy: `aria-busy` on it, its content out of reach, and a spinner with `label:` over it — the region, not the page.

```
LoadingOverlay(visible: …, label: …) { … }
```

```wf
state saving = false
state name = "Ada Lovelace"
LoadingOverlay(visible: saving, label: "Saving your profile") {
    Card {
        Input(bind: name, label: "Name")
        Button("Save").primary { on click { saving = true } }
    }
}
Button("Finish saving") { on click { saving = false } }
```

Renders with the class `wf-loading-overlay`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` (required) | `Bool` | While it holds, the region is busy: `aria-busy`, its content `inert`, a spinner and `label:` drawn over it; any `Bool` expression, read and never written |
| `label:` | `String` | What it is busy with, shown under the spinner and read by a screen reader |

**Flags:** `.visible`

**Parts:**

- `LoadingOverlay.Content { … }` — The region. Drawn for it.
- `LoadingOverlay.Cover { … }` — What covers it while it is busy. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Tour

A tour of the page while `open:` holds: each step in a card at its `target:` (a `ref:`), lit through a cut-out in the dimmed page, with Back, Next and Done; Escape ends it and writes `false` back. `on finish` hears Done, `on close` any end.

```
Tour(open: …, back: …, next: …) { … }
```

```wf
state touring = false
state toured = "Not toured yet"
Row(gap: .sm) {
    Button("New project", ref: newBtn).primary
    Button("Import", ref: importBtn)
    Button("Take the tour") { on click { touring = true } }
}
Tour(open: touring) {
    on finish { toured = "Tour finished" }
    Tour.Step(target: newBtn, title: "Start here") { Text("Make a project from a template.") }
    Tour.Step(target: importBtn, title: "Bring your work") { Text("Import a folder you already have.") }
}
Text(toured).muted
```

Renders with the class `wf-tour`.

| Prop | Type | Meaning |
|---|---|---|
| `open:` (required) | `State` | Whether the tour is showing: a `Bool` state, set back to `false` when it ends (Done, Escape or the close button); it starts at the first step each time it opens |
| `back:` | `String` | The back button's words; hidden on the first step |
| `next:` | `String` | The next button's words, on every step but the last |
| `done:` | `String` | The last step's button's words; pressing it ends the tour and fires `on finish` |
| `close:` | `String` | The close button's accessible name (it shows ×) |
| `of:` | `String` | The word in the step count, `2 of 5` |

**Events of its own:** `on close`, `on finish`

**Parts:**

- `Tour.Step(target: …, title: …) { … }` — One step: what it points at, its title, and its block.
  - `target:` `Element` — The element the step points at and lights up, by its `ref:` (even one written further down); unset, the card sits mid-window with nothing lit
  - `title:` `String` — The step's heading, drawn as an `h2` above its block, and the card's accessible name

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ConsentBanner

The reader's say over the scripts that track them: shown until they choose, Decline as easy as Accept, and the choice kept. Accepting loads the `meta.scripts` entries marked `"load": "consent"`, then and on every later visit; nothing loads before.

```
ConsentBanner(text, accept: …, decline: …, label: …) { … }
```

```wf
ConsentBanner("We would like to count visits to improve the site. Nothing is shared.", accept: "Allow", decline: "No thanks") {
    Link("How we count", to: "/learn")
}
```

Renders with the class `wf-consent`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | What is asked, drawn above the block and the two buttons |
| `accept:` | `String` | The yes button's words; pressing it loads the `"load": "consent"` scripts and remembers the answer |
| `decline:` | `String` | The no button's words, drawn first and as Accept is; the answer is remembered and nothing loads |
| `label:` | `String` | The banner's accessible name (`aria-label`) |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ConsentSettings

A button that asks the reader again — a footer's "Cookie settings": it forgets their answer and shows the `ConsentBanner`. A script already loaded stays until the page is loaded again.

```
ConsentSettings(label)
```

```wf
Row(gap: .md, align: .center) {
    Text("© 2026 Ledger").muted.sm
    Link("Privacy", to: "/learn")
    ConsentSettings("Cookie settings")
}
ConsentBanner("We would like to count visits.", accept: "Allow", decline: "No thanks")
```

Renders with the class `wf-consent-settings`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The button's words; pressing it forgets the reader's answer and shows the page's `ConsentBanner` again |

Attribute families: `global`, `aria`, `data`.

### CircularProgress

A ring that fills to `value:` of `max:`, announced as a progress bar; with no `value:`, it turns, for work of no known length. Its block is drawn in the middle.

```
CircularProgress(value: …, max: …, label: …) { … }
```

```wf
state done = 40
Row(gap: .lg, align: .center) {
    CircularProgress(value: done, max: 100, label: "Upload") { Text("{done}%").sm }
    Stack(gap: .xs) {
        Text("Uploading photos.zip").bold
        Button("Add 20%").sm { on click { done = if done >= 80 { 100 } else { done + 20 } } }
    }
}
```

Renders with the class `wf-circular-progress`.

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | How far along, out of `max:`, clamped to the ring; unset or `null`, the ring turns without a value |
| `max:` | `Number` | The value at which the ring is full; `0` or not a number counts as 100 |
| `label:` | `String` | What is progressing: the progress bar's accessible name (`aria-label`) |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Meter

A level against its range (`<meter>`): a disk's use, a password's strength. Past `high:` or under `low:` it says so in its colour; `optimum:` says which end is good.

```
Meter(value: …, min: …, max: …)
```

```wf
state used = 0.72
Stack(gap: .sm) {
    Text("Disk: {used:.percent} used")
    Meter(value: used, max: 1, high: 0.9, optimum: 0, label: "Disk")
    Slider(bind: used, min: 0, max: 1, step: 0.01, label: "Used")
}
```

Renders with the class `wf-meter`.

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | The level, between `min:` and `max:` (the native `<meter>`'s `value`) |
| `min:` | `Number` | The bottom of the range |
| `max:` | `Number` | The top of the range |
| `low:` | `Number` | Below this the level counts as low, and its colour says so |
| `high:` | `Number` | Above this the level counts as high, and its colour says so |
| `optimum:` | `Number` | The best level: which part of the range (low, middle, high) is drawn as good |
| `label:` | `String` | What it measures: its accessible name (`aria-label`) |

Attribute families: `global`, `aria`, `data`.

### Popconfirm

A question beside the button in its block before what it does: the button opens it, `on confirm` hears yes, `on cancel` no or Escape; the focus starts on the answer that loses nothing.

```
Popconfirm(title: …, yes: …, no: …) { … }
```

```wf
state files = ["notes.md", "draft.md", "todo.md"]
Stack(gap: .sm) {
    for f in files by f {
        Row(gap: .sm, align: .center, justify: .between) {
            Text(f)
            Popconfirm(title: "Delete {f}?", yes: "Delete", no: "Keep").danger {
                on confirm { files = files.filter(x => x != f) }
                Button("Delete").sm.outlined
            }
        }
    }
}
```

Renders with the class `wf-popconfirm`.

| Prop | Type | Meaning |
|---|---|---|
| `title:` (required) | `String` | The question, in a layer above the first button of its block when that button is pressed |
| `yes:` | `String` | The yes button's words; pressing it fires `on confirm` |
| `no:` | `String` | The no button's words, which has the focus when it opens; pressing it or Escape fires `on cancel` |
| `danger:` | `Bool` | What yes does cannot be taken back: the yes button is drawn in the danger colour, not the primary, and the question in bold |

**Flags:** `.danger`

**Events of its own:** `on confirm`, `on cancel`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Alert

Inline notice; a `danger` or `warning` one is announced as an alert.

```
Alert(message, tone: …) { … }
```

```wf
Stack(gap: .sm) {
    Alert("Your changes were saved.").success
    Alert("Your trial ends in 3 days.").warning
    Alert("The server is not answering. We will try again in a minute.").danger
    Alert("A new version of the editor is available.").info
}
```

Renders with the class `wf-alert`.

| Prop | Type | Meaning |
|---|---|---|
| `message` (positional) | `String` | The message; its block, when it has one, follows it |
| `tone:` | `.success` `.danger` `.warning` `.info` | The kind of notice, which sets its colours; `.danger` and `.warning` are announced at once (`role="alert"`), the rest politely (`role="status"`) |

**Flags:** `.success` `.danger` `.warning` `.info`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Toast

Temporary notification.

```
Toast(message, tone: …)
```

```wf
state archived = 0
Row(gap: .sm) {
    Button("Archive conversation") {
        on click {
            archived = archived + 1
            toast("Conversation archived", action: "Undo", onAction: () => toast("Restored"))
        }
    }
    Button("Save draft").primary { on click { toast("Draft saved", tone: .success) } }
}
Text("{archived} archived").muted
```

Renders with the class `wf-toast`.

| Prop | Type | Meaning |
|---|---|---|
| `message` (positional) | `String` | The message, shown above everything for 3 s once the element is drawn, and said to a screen reader; the element itself draws nothing in place |
| `tone:` | `.success` `.danger` `.warning` `.info` | The kind of notice, which sets its colours; `.danger` is announced at once, the rest politely |

**Flags:** `.success` `.danger` `.warning` `.info`

Attribute families: `global`, `aria`, `data`.

### Modal

Modal `<dialog>`, opened by a state flag; the browser supplies focus trapping and Escape. `on close` runs when it closes, however it was closed, and the focus goes back to what had it.

```
Modal(visible: …, title: …, closable: …) { … }
```

```wf
state open = false
state name = "Q3 report"
Button("Rename") { on click { open = true } }
Modal(visible: open, title: "Rename the report").closable {
    on close { open = false }
    Input(bind: name, label: "Name")
    Modal.Footer {
        Button("Cancel") { on click { open = false } }
        Button("Save").primary { on click { open = false } }
    }
}
Text("Report: {name}").muted
```

Renders with the class `wf-modal`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape, a press on the backdrop or its close button shuts it; a condition opens it but is not written |
| `title:` | `String` | The heading at the top, and the accessible name; a value that reads state follows it; a `Modal.Title` block draws the heading instead |
| `closable:` | `Bool` | A close button (×) in the heading row, which closes it as Escape does; with no `title:` the row holds the button alone |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button and `visible:` |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control; when it closes the focus goes back where it was |

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
state branch = "main"
Button("Branch: {branch}", ref: branchBtn) { on click { open = !open } }
Popover(visible: open, anchor: branchBtn, placement: .bottomStart, label: "Branches").arrow {
    on close(reason) { open = false }
    Stack(gap: .xs) {
        Button("main") { on click { branch = "main"  open = false } }
        Button("release-7.0") { on click { branch = "release-7.0"  open = false } }
        Button("fix-login") { on click { branch = "fix-login"  open = false } }
    }
}
```

Renders with the class `wf-popover`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape or a press outside closes it (as `dismiss:` allows); a condition opens it but is not written |
| `anchor:` | `Element or Point` | What it is placed against and follows as the page scrolls: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }`; its control gets `aria-controls` and `aria-expanded` |
| `at:` | `Point` | A point to place it at, `{ x: e.clientX, y: e.clientY }` — where the pointer asked for a context menu; the same option as `anchor:`, so give one or the other |
| `placement:` | `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room, and slides along its side to stay on screen |
| `offset:` | `Number` | The gap between it and its anchor, in px |
| `label:` | `String` | Its accessible name; one drawn as a dialog (no other `role:`) without it is `A18` |
| `arrow:` | `Bool` | A small point on the side toward its anchor, following the placement it lands on |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides `visible:`; `on close` hears `"escape"` or `"outside"` |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, its first control, else the popover itself |

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
Button("Close the editor") { on click { asking = true } }
Dialog(visible: asking, title: "Leave without saving?").closable {
    Text("Your draft of notes.md has changes that are not saved.")
    Row(gap: .sm, justify: .end) {
        Button("Stay") { on click { asking = false } }
        Button("Leave").danger { on click { asking = false } }
    }
}
```

Renders with the class `wf-dialog`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape, a press on the backdrop or its close button shuts it; a condition opens it but is not written |
| `title:` | `String` | The heading at the top, and the accessible name; a value that reads state follows it; a `Dialog.Title` block draws the heading instead |
| `closable:` | `Bool` | A close button (×) in the heading row, which closes it as Escape does; with no `title:` the row holds the button alone |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button and `visible:` |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control; when it closes the focus goes back where it was |

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
state said = "Draft: 3 unsaved edits"
Button("Discard changes").danger.outlined { on click { sure = true } }
AlertDialog(visible: sure, title: "Discard your changes?") {
    Text("Your 3 edits to the pricing page will be lost.")
    AlertDialog.Footer {
        Button("Keep editing") { on click { sure = false } }
        Button("Discard").danger { on click { sure = false  said = "Draft discarded" } }
    }
}
Text(said).muted
```

Renders with the class `wf-dialog wf-alert-dialog`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape, a press on the backdrop or its close button shuts it; a condition opens it but is not written |
| `title:` | `String` | The heading at the top, and the accessible name; a value that reads state follows it; a `AlertDialog.Title` block draws the heading instead |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button and `visible:`; an alert dialog is answered, so a press beside it does nothing unless it says `.any` |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control; when it closes the focus goes back where it was |

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
state inStock = true
Button("Filters") { on click { filters = true } }
Sheet(visible: filters, title: "Filters", snap: [0.5, 1]).closable {
    Radio(bind: sort, value: "new", label: "Newest first")
    Radio(bind: sort, value: "price", label: "Lowest price")
    Switch(bind: inStock, label: "In stock only")
    Sheet.Footer { Button("Show results").primary.full { on click { filters = false } } }
}
Text("Sorted by {sort}").muted
```

Renders with the class `wf-sheet`.

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape, a press on the backdrop or its close button shuts it; a condition opens it but is not written |
| `title:` | `String` | The heading at the top, and the accessible name; a value that reads state follows it; a `Sheet.Title` block draws the heading instead |
| `closable:` | `Bool` | A close button (×) in the heading row, which closes it as Escape does; with no `title:` the row holds the button alone |
| `side:` | `.bottom` `.start` `.end` | The edge it slides in from, and is dragged back toward to close |
| `snap:` | `[Number]` | A bottom sheet's heights, as fractions of the window's (`[0.5, 1]`): it opens at the first written, a drag settles at the nearest; values outside 0–1 are dropped |
| `label:` | `String` | Its accessible name when it has no `title:`; ignored when it has one |
| `dismiss:` | `.any` `.escape` `.none` | What closes it besides the page's own button and `visible:` |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control; when it closes the focus goes back where it was |

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
Row(gap: .lg, align: .center) {
    Spinner.sm
    Spinner
    Spinner.lg
}
```

Renders with the class `wf-spinner`.

| Prop | Type | Meaning |
|---|---|---|
| `size:` | `.sm` `.md` `.lg` | Its diameter |

**Flags:** `.sm` `.md` `.lg`

Attribute families: `global`, `aria`, `data`.

### Progress

Progress bar; a `value` that reads state moves with it.

```
Progress(value: …, max: …)
```

```wf
state sent = 3
Stack(gap: .sm) {
    Text("Sending invitations: {sent} of 8")
    Progress(value: sent, max: 8)
    Button("Send one more") { on click { sent = if sent < 8 { sent + 1 } else { 0 } } }
}
```

Renders with the class `wf-progress`.

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | How far along, out of `max:`; unset, the bar shows activity with no amount (indeterminate); a value that reads state follows it |
| `max:` | `Number` | The value at which the bar is full (the native `<progress>`'s `max`); with it unset `value:` is a fraction of 1 |

Attribute families: `global`, `aria`, `data`.

### Skeleton

Placeholder shape shown while content loads.

```
Skeleton(height: …, width: …, size: …)
```

```wf
Row(gap: .md, align: .center) {
    Skeleton(size: "48px").circle
    Stack(gap: .sm) {
        Skeleton(height: "16px", width: "180px")
        Skeleton(height: "12px", width: "260px")
        Skeleton(height: "12px", width: "220px")
    }
}
```

Renders with the class `wf-skeleton`.

| Prop | Type | Meaning |
|---|---|---|
| `height:` | `String` | Its height, a CSS length (`"20px"`, `"1.5rem"`); a value that reads state follows it |
| `width:` | `String` | Its width, a CSS length (`"200px"`, `"60%"`) |
| `size:` | `String` | The diameter of a `.circle`, a CSS length, used for any of `width:` and `height:` not given; ignored without `.circle` |
| `circle:` | `Bool` | Round instead of a rounded rectangle, for an avatar's place |

**Flags:** `.circle`

Attribute families: `global`, `aria`, `data`.

## Media

### Lightbox

Every picture in its block opens, on a press or Enter, full screen: the arrow keys and a swipe go between them, a double press or `+` zooms, Escape closes and the focus goes back. A picture's `data-full:` is the file it opens, else its largest size.

```
Lightbox(label: …, previous: …, next: …) { … }
```

```wf
Lightbox(label: "Launch week", previous: "Previous photo", next: "Next photo", close: "Close", zoom: "Zoom", of: "of") {
    Gallery(columns: 3, ratio: 4 / 3) {
        Image(src: "/og.png", alt: "The launch card, as it was shared")
        Image(src: "/icon-512.png", alt: "The app icon on a white tile")
        Image(src: "/og.png", alt: "The launch card, cropped to its title") {
            style { object-position: 20% 50% }
        }
    }
}
```

Renders with the class `wf-lightbox`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | What the pictures are: the accessible name of the full-screen dialog they open in |
| `previous:` | `String` | The previous button's accessible name, in your words |
| `next:` | `String` | The next button's accessible name, in your words |
| `close:` | `String` | The close button's accessible name, in your words |
| `zoom:` | `String` | The zoom button's accessible name, in your words |
| `of:` | `String` | The word between the two numbers of the counter, `2 of 5` |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Gallery

Pictures in a grid of `columns:` (unset, as many as fit), each cropped to one shape — `ratio:`, unset square. In a `Lightbox`, each opens full screen.

```
Gallery(columns: …, ratio: …) { … }
```

```wf
Gallery(columns: { base: 2, md: 4 }, ratio: 1) {
    Image(src: "/og.png", alt: "The sharing card")
    Image(src: "/icon-512.png", alt: "The app icon")
    Image(src: "/og.png", alt: "The sharing card, its left edge") {
        style { object-position: 0% 50% }
    }
    Image(src: "/og.png", alt: "The sharing card, its right edge") {
        style { object-position: 100% 50% }
    }
}
```

Renders with the class `wf-gallery`.

| Prop | Type | Meaning |
|---|---|---|
| `columns:` | `Number` | How many pictures across, or one per breakpoint (`{ base: 2, md: 4 }`); unset, as many columns at least 10rem wide as fit |
| `ratio:` | `Number` | Each picture's shape, width over height (`4 / 3`, `16 / 9`); each is cropped to it (`object-fit: cover`) |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Image

Image; `alt` is required for accessibility (rule A01). Given an `image` the program declares, it becomes a `<picture>` with every width the build wrote.

```
Image(source, src: …, sizes: …, placeholder: …)
```

```wf
Stack(gap: .md) {
    Image(src: "/og.png", alt: "WebFluent's sharing card: the name over a dark field",
          sizes: "(max-width: 768px) 100vw, 640px", placeholder: .blur).rounded
    Row(gap: .md, align: .center) {
        Image(src: "/icon-512.png", alt: "The app icon", width: 64, height: 64).circle
        Text("Pictures from public/ are resized to WebP at every width a page asks for.").muted.sm
    }
}
```

Renders with the class `wf-image`.

| Prop | Type | Meaning |
|---|---|---|
| `source` (positional) | `Picture or String` | An `image` the program declares (a `<picture>` at every width the build wrote), or a URL |
| `src:` | `String` | The picture's address. A literal path of `public/` to a PNG, JPEG, GIF or WebP is processed by the build like an `image`; an SVG, a URL or a `data:image/…` is used as written |
| `sizes:` | `String` | How wide it will be shown, so the browser picks one of the widths the build wrote: `(max-width: 768px) 100vw, 1200px`; for a processed picture |
| `placeholder:` | `.blur` `.color` `.none` | What fills the box of a processed picture while it loads, cleared when it lands |
| `alt:` | `String` | Text alternative; `""` for a purely decorative image (`A01` when left out) |
| `width:` | `Number` | The `<img>`'s `width` in px, so its box is reserved before it loads; ignored on a processed picture, which carries the file's real size |
| `height:` | `Number` | The `<img>`'s `height` in px, so its box is reserved before it loads; ignored on a processed picture, which carries the file's real size |
| `loading:` | `.lazy` `.eager` | When the browser fetches it |
| `shape:` | `.rounded` `.circle` | Its corners |

**Flags:** `.blur` `.color` `.none` `.lazy` `.eager` `.rounded` `.circle`

Attribute families: `global`, `aria`, `data`.

### Video

Video player. Without `captions` it draws an `A09` warning: a video nobody can hear is a video nobody can follow.

```
Video(src: …, controls: …, autoplay: …)
```

```wf
Video(src: "/media/launch.mp4", poster: "/og.png", captions: "/media/launch.en.vtt",
      width: 1280, height: 720).controls.playsinline
```

Renders with the class `wf-video`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The video's address |
| `controls:` | `Bool` | The browser's playback controls; without them, `A09` |
| `autoplay:` | `Bool` | Start playing when shown; browsers allow it only for a `.muted` video |
| `poster:` | `String` | The picture shown before it plays |
| `muted:` | `Bool` | Start with the sound off |
| `loop:` | `Bool` | Start again when it ends |
| `playsinline:` | `Bool` | Play where it sits on a phone, rather than full screen |
| `captions:` | `String` | A WebVTT file of captions, which becomes a default `<track kind="captions">`; without it, `A09` |
| `width:` | `Number` | The player's `width` in px |
| `height:` | `Number` | The player's `height` in px |

**Flags:** `.controls` `.autoplay` `.muted` `.loop` `.playsinline`

Attribute families: `global`, `aria`, `data`, `media`.

### Audio

Audio player. `transcript` links what was said, which is what a reader who cannot hear it needs.

```
Audio(src: …, controls: …, autoplay: …)
```

```wf
Stack(gap: .sm) {
    Heading("Episode 12: Shipping 7.0").h3
    Text("Forty minutes on types, results and what changed.").muted
    Audio(src: "/media/episode-12.mp3", transcript: "/media/episode-12.txt").controls
}
```

Renders with the class `wf-audio`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The audio's address |
| `controls:` | `Bool` | The browser's playback controls; without them the player shows nothing |
| `autoplay:` | `Bool` | Start playing when shown; most browsers refuse until the reader has interacted with the page |
| `loop:` | `Bool` | Start again when it ends |
| `transcript:` | `String` | The address of what was said, linked as `Read the transcript` beneath the player; without it, `A09` |

**Flags:** `.controls` `.autoplay` `.loop`

Attribute families: `global`, `aria`, `data`.

### Frame

Another page, shown inside this one: an `iframe` — the page at `src:`, or the HTML `srcdoc:` gives it, sandboxed as `sandbox:` says. `on message(m)` hears what that page posts to this one (`m.data`), and nothing any other page posts. Without `title:` it draws an `A17` warning: a frame is announced by its title.

```
Frame(src: …, srcdoc: …, title: …)
```

```wf
state version = 2
state heard = "nothing yet"
Stack(gap: .sm) {
    Row(gap: .sm) {
        Button("Version 1") { on click { version = 1 } }
        Button("Version 2") { on click { version = 2 } }
    }
    Frame(srcdoc: "<p style='font-family:sans-serif'>Release notes, version {version}</p>",
          title: "Release notes preview", sandbox: "") {
        style { width: 100%; height: 120px; border: 1px solid $color-border; border-radius: 8px }
        on load { heard = "version {version} loaded" }
    }
    Text("Frame says: {heard}").muted.sm
}
```

Renders with the class `wf-frame`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The address of the page it shows |
| `srcdoc:` | `String` | The HTML it draws in place of an address, sandboxed as `sandbox:` says; a value that reads state draws the new HTML as it changes |
| `title:` | `String` | What it shows, in a few words: its accessible name (`A17` without it) |
| `sandbox:` | `String` | What the framed page may do: `"allow-scripts allow-forms"`; `""`, nothing. Unset, no sandbox, except `srcdoc:` from a value, which gets `""` |
| `allow:` | `String` | The browser features it may use, a permissions policy: `"fullscreen; clipboard-write"` |
| `loading:` | `.lazy` `.eager` | When the browser fetches the framed page |

**Flags:** `.lazy` `.eager`

**Events of its own:** `on message`, `on load`

Attribute families: `global`, `aria`, `data`.

### Icon

One of the built-in SVG icons, rendered inline.

```
Icon(name, size: …, tone: …, muted: …)
```

```wf
Row(gap: .lg, align: .center) {
    Row(gap: .xs, align: .center) { Icon("check").success  Text("Deployed") }
    Row(gap: .xs, align: .center) { Icon("warning").warning  Text("2 warnings") }
    Row(gap: .xs, align: .center) { Icon("alert-circle").danger  Text("1 failed") }
    Row(gap: .xs, align: .center) { Icon("clock").muted  Text("Queued") }
}
```

Renders with the class `wf-icon`.

| Prop | Type | Meaning |
|---|---|---|
| `name` (positional) | `String` | One of the 83 built-in icon names (`home`, `search`, `chevron-down` …), drawn inline as SVG; another is `V08` and draws nothing |
| `size:` | `.sm` `.md` `.lg` | How large it is, against the text around it |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Its colour, a tone of the theme |
| `muted:` | `Bool` | In `color-text-muted`, for something secondary |

**Flags:** `.sm` `.md` `.lg` `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.muted`

Attribute families: `global`, `aria`, `data`.

### Carousel

Slide track with named controls and dots; optional autoplay.

```
Carousel(label: …, autoplay: …, interval: …) { … }
```

```wf
Carousel(label: "Featured work", autoplay: true, interval: 6000) {
    Carousel.Slide {
        Image(src: "/og.png", alt: "The sharing card")
    }
    Carousel.Slide {
        Image(src: "/og.png", alt: "The sharing card, its title in close-up") {
            style { object-fit: cover; object-position: 15% 50%; aspect-ratio: 1200 / 630 }
        }
    }
    Carousel.Slide {
        Center {
            style { aspect-ratio: 1200 / 630; background: $color-surface }
            Image(src: "/icon-512.png", alt: "The app icon", width: 160, height: 160)
        }
    }
}
```

Renders with the class `wf-carousel`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The region's accessible name |
| `autoplay:` | `Bool` | Advance by itself every `interval:`, with a pause button; it waits while hovered, focused or in a hidden tab, and never runs under reduced motion; `.autoplay`, or a value that reads state |
| `interval:` | `Number` | Milliseconds between slides while it plays, 1 to 2147483647; under 1, 5000 |

**Flags:** `.autoplay`

**Parts:**

- `Carousel.Slide { … }` — One slide of a carousel.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Data display

### DescriptionList

Terms and what they are (`<dl>`): a column of pairs, or `.grid`, each term beside its value.

```
DescriptionList(grid: …) { … }
```

```wf
DescriptionList.grid {
    DescriptionList.Item("Service", value: "checkout-api")
    DescriptionList.Item("Region", value: "eu-west-1")
    DescriptionList.Item("Replicas", value: 3)
    DescriptionList.Item("Status") { Badge("Live").success }
    DescriptionList.Item("Last deploy") { RelativeTime(@2026-11-14T16:05Z) }
}
```

Renders with the class `wf-description-list`.

| Prop | Type | Meaning |
|---|---|---|
| `grid:` | `Bool` | Each term beside its value, in two columns (the terms as wide as the widest); unset, each term above its value |

**Flags:** `.grid`

**Parts:**

- `DescriptionList.Item(term, value: …) { … }` — One pair: its term, and `value:` or its block.
  - `term:` `String` — What it is: the term (`<dt>`)
  - `value:` `String or Number or Bool` — Its value, shown as text in the `<dd>`; unset, the item's block is the value
- `DescriptionList.Term(text) { … }` — A term. Drawn for it.
  - `text:` `String` — The term, as text
- `DescriptionList.Details(text) { … }` — A value. Drawn for it.
  - `text:` `String` — The value, as text

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Timeline

Events in order (`<ol>`), each a dot on a line, its title, its time and its block.

```
Timeline { … }
```

```wf
Timeline {
    Timeline.Item("7.0 released", at: @2026-11-10) {
        Text("Typed requests, results and error enums.")
    }
    Timeline.Item("6.1 released", at: @2026-09-02) {
        Text("Charts and QR codes on a web page.")
    }
    Timeline.Item("Public beta", at: @2026-06-15) {
        Text("The first hundred projects moved over.").muted
    }
}
```

Renders with the class `wf-timeline`.

**Parts:**

- `Timeline.Item(title, at: …) { … }` — One event: its title, `at:` its time, and its block.
  - `title:` `String` — What happened, in bold above the date
  - `at:` `Date or DateTime or String` — When: a `Date`, a `DateTime` or an ISO string, written as a medium date (`Jan 2, 2026`) in a `<time datetime>`
- `Timeline.Time(text, datetime: …) { … }` — An event's time (`<time>`). Drawn for it.
  - `text:` `String` — The time, as a reader reads it
  - `datetime:` `String` — The time as a machine reads it: an ISO 8601 date or date-time, the `<time>`'s `datetime` attribute

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### AvatarGroup

Avatars overlapping in a row: past `max:` the rest are one `+3`.

```
AvatarGroup(max: …, label: …) { … }
```

```wf
AvatarGroup(max: 4, label: "Reviewers") {
    Avatar(src: "/icon-512.png", alt: "Ada Lovelace")
    Avatar(initials: "GH").primary
    Avatar(initials: "AT")
    Avatar(initials: "ED")
    Avatar(initials: "KJ")
    Avatar(initials: "MH")
}
```

Renders with the class `wf-avatar-group`.

| Prop | Type | Meaning |
|---|---|---|
| `max:` | `Number` | How many avatars show; the rest are hidden and counted in one `+N` avatar (read "N more"). `0` shows them all |
| `label:` | `String` | Who they are: the accessible name of the group (`role="group"`) |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Item

A row of a list, a settings page or a search's results: `Item.Media` at its start, its `title:` and `description:`, then its block, and `Item.Actions` at its end.

```
Item(title: …, description: …) { … }
```

```wf
state push = true
state digest = false
Stack(gap: .sm) {
    Item(title: "Push notifications", description: "Sent to your phone when a deploy fails") {
        Item.Media { Icon("bell") }
        Item.Actions { Switch(bind: push, aria-label: "Push notifications") }
    }
    Item(title: "Weekly digest", description: "A summary every Monday morning") {
        Item.Media { Icon("mail") }
        Item.Actions { Switch(bind: digest, aria-label: "Weekly digest") }
    }
}
```

Renders with the class `wf-item`.

| Prop | Type | Meaning |
|---|---|---|
| `title:` | `String` | What it is: a bold line before the rest of its block |
| `description:` | `String` | A line under the title, small and muted |

**Parts:**

- `Item.Media { … }` — What goes at its start: an icon, an avatar, a picture.
- `Item.Content { … }` — Its title, description and block. Drawn for it.
- `Item.Actions { … }` — What goes at its end: a switch, a button, a menu.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Sparkline

A tiny chart in a line of text — a line, or `.bar` — of a list of numbers, drawn as an SVG when the page is built.

```
Sparkline(data, bar: …, width: …, height: …)
```

```wf
Stack(gap: .sm) {
    Row(gap: .sm, align: .center) {
        Text("Sign-ups this week")
        Sparkline([12, 18, 9, 24, 31, 27, 40], label: "Sign-ups this week")
    }
    Row(gap: .sm, align: .center) {
        Text("Errors by day")
        Sparkline([4, 2, 7, 1, 0, 3, 2], label: "Errors by day").bar
    }
}
```

Renders with the class `wf-sparkline`.

| Prop | Type | Meaning |
|---|---|---|
| `data` (positional) | `[Number]` | The numbers, a list, drawn as an SVG when the page is built, so they must be known then (`E109` otherwise); scaled from the least to the most |
| `bar:` | `Bool` | Bars from zero rather than a line |
| `width:` | `Number` | Its width in px; 0 or less is the default |
| `height:` | `Number` | Its height in px; 0 or less is the default |
| `color:` | `String` | The line's or the bars' colour, a CSS colour (`#0F766E`, `var(--color-primary)`); anything else is the default, the text's colour (`currentColor`) |
| `label:` | `String` | What it shows, as the picture's accessible name (`role="img"`) |

**Flags:** `.bar`

Attribute families: `global`, `aria`, `data`.

### RelativeTime

A moment as `3 minutes ago`, kept current, with the exact time on hover (`<time>`).

```
RelativeTime(when, datetime: …)
```

```wf
Stack(gap: .xs) {
    Row(gap: .xs) { Text("Deployed")  RelativeTime(@2026-11-01T09:30Z) }
    Row(gap: .xs) { Text("Created")  RelativeTime(@2026-01-15) }
}
```

Renders with the class `wf-relative-time`.

| Prop | Type | Meaning |
|---|---|---|
| `when` (positional) | `Date or DateTime or String` | The moment: a `Date`, a `DateTime` or an ISO string, shown as `3 minutes ago` and kept current; the exact time is its `title` |
| `datetime:` | `String` | The moment as a machine reads it, the `<time>`'s `datetime` attribute; written from `when` unless given |

Attribute families: `global`, `aria`, `data`.

### Card

Card surface with optional header, body and footer sections.

```
Card(surface: …) { … }
```

```wf
Card.elevated {
    Card.Header {
        Row(justify: .between, align: .center) {
            Heading("Pro").h3
            Badge("Popular").primary.pill
        }
    }
    Card.Body {
        Text("Everything in Free, and more.").muted
        List {
            List.Item { Text("Unlimited projects") }
            List.Item { Text("Priority support") }
        }
    }
    Card.Footer { Button("Choose Pro").primary.full }
}
```

Renders with the class `wf-card`.

| Prop | Type | Meaning |
|---|---|---|
| `surface:` | `.flat` `.elevated` `.outlined` | How it stands off the page |

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
state regions = [{ name: "fra1", latency: 42, status: "Healthy" }, { name: "iad1", latency: 88, status: "Healthy" }, { name: "sin1", latency: 190, status: "Degraded" }]
Table(caption: "Regions") {
    Table.Head {
        Table.Row { Table.Cell("Region")  Table.Cell("Latency")  Table.Cell("Status") }
    }
    Table.Body {
        for r in regions by r.name {
            Table.Row {
                Table.Cell(r.name).header
                Table.Cell("{r.latency} ms")
                Table.Cell { Badge(r.status, tone: if r.status == "Healthy" { .success } else { .warning }) }
            }
        }
    }
}
```

Renders with the class `wf-table`.

| Prop | Type | Meaning |
|---|---|---|
| `caption:` | `String` | The table's accessible name, drawn as a `<caption>` that is visually hidden |

**Parts:**

- `Table.Head { … }` — Header section; its cells become `<th scope="col">`.
- `Table.Body { … }` — The body of a table: its rows of data.
- `Table.Row { … }` — A row of cells.
- `Table.Cell(content, header: …) { … }` — A cell. `header` makes it a `<th>` anywhere, as `Table.Head` does for its cells.
  - `content:` `String or Number or Bool` — The cell's text: a string, number or boolean; unset, its block
  - `header:` `Bool` — Drawn as a header cell, `<th scope="col">`, outside a `Table.Head` too (a cell a component draws)
  - flags: `.header`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### List

List of its children; `ordered` for a numbered list.

```
List(ordered: …) { … }
```

```wf
Stack(gap: .md) {
    List {
        List.Item { Text("Unlimited projects") }
        List.Item { Text("Custom domains") }
        List.Item { Text("Priority support").bold }
    }
    List.ordered {
        List.Item { Code("wf init shop") }
        List.Item { Text("Write your first page") }
        List.Item { Code("wf serve") }
    }
}
```

Renders with the class `wf-list`.

| Prop | Type | Meaning |
|---|---|---|
| `ordered:` | `Bool` | Numbered: an `<ol>` whose items are counted `1.`, `2.` …; unset, a `<ul>` with no markers |

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
Row(gap: .sm, align: .center) {
    Heading("Billing").h3
    Badge("Beta").info
    Badge("3 overdue").danger.pill
}
```

Renders with the class `wf-badge`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The badge's text; unset, its block |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | The colour it is filled with, a tone of the theme, its text in the tone's `color-on-*` |
| `pill:` | `Bool` | Fully rounded ends; the full theme already rounds a badge, so it matters with `theme.builtin: "structural"` |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.pill`

Attribute families: `global`, `aria`, `data`.

### Tag

Compact chip.

```
Tag(label)
```

```wf
Stack(gap: .sm) {
    Heading("Slow roasting, explained").h3
    Row(gap: .xs) {
        Tag("Cooking")
        Tag("Technique")
        Tag("Long read")
    }
}
```

Renders with the class `wf-tag`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The tag's text; unset, its block |

Attribute families: `global`, `aria`, `data`.

### Avatar

A picture of a person, or their initials.

```
Avatar(src: …, alt: …, initials: …)
```

```wf
Row(gap: .md, align: .center) {
    Avatar(src: "/icon-512.png", alt: "Ada Lovelace").lg
    Avatar(initials: "GH").primary
    Avatar(initials: "AT").sm
}
```

Renders with the class `wf-avatar`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The picture's address, cropped to the circle; a `data:image/…` address is taken. With it, `initials:` is not shown |
| `alt:` | `String` | What the picture shows, its `alt`; empty for a picture the name beside it already says |
| `initials:` | `String` | The letters shown when there is no `src:` |
| `size:` | `.sm` `.md` `.lg` | How large the circle is |
| `tone:` | `.primary` | The colour behind the initials |

**Flags:** `.sm` `.md` `.lg` `.primary`

Attribute families: `global`, `aria`, `data`.

### Tooltip

Shows `text` above everything when its child is hovered (after `delay:`) or focused from the keyboard.

```
Tooltip(text, shortcut: …, placement: …, delay: …) { … }
```

```wf
Row(gap: .sm) {
    Tooltip("Copy the link") { IconButton(icon: "copy", label: "Copy link") }
    Tooltip("Edit the page", shortcut: "mod+e") { IconButton(icon: "edit", label: "Edit") }
    Tooltip("Delete it for good", placement: .bottom, delay: 200) {
        IconButton(icon: "trash", label: "Delete").danger
    }
}
```

Renders with the class `wf-tooltip`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The tooltip's words, shown above everything at the control in its block and linked to it by `aria-describedby` |
| `shortcut:` | `String` | The keys that do the same, `mod+s`, shown as a chip as this system writes them and set as the control's `aria-keyshortcuts` |
| `placement:` | `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` | Where it goes against its control; it flips to the other side when there is no room |
| `delay:` | `Number` | How long a hover waits before it shows, in ms; keyboard focus shows it at once |

**Flags:** `.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Tree

Nested items — files and folders, an outline — as a tree a keyboard can work: Up and Down through what shows, Right to open and go in, Left to close and go out (the other way round right to left), Home and End, a letter. `row(item) { … }` draws each; `bind:` holds the chosen item's key, or a list of keys with `.multiple`.

```
Tree(items: …, children: …, key: …)
```

```wf
state file = "src/pages/Home.wf"
Tree(items: [
    { path: "src", name: "src", children: [
        { path: "src/App.wf", name: "App.wf" },
        { path: "src/pages", name: "pages", children: [
            { path: "src/pages/Home.wf", name: "Home.wf" },
            { path: "src/pages/About.wf", name: "About.wf" },
        ] },
    ] },
    { path: "webfluent.app.json", name: "webfluent.app.json" },
], children: f => f.children ?? [], key: f => f.path, bind: file, open: ["src", "src/pages"], label: "Files") {
    row(f) {
        Icon(if f.children != null { "folder" } else { "file" })
        Text(f.name)
    }
}
Text("Open: {file}").muted
```

Renders with the class `wf-tree`.

| Prop | Type | Meaning |
|---|---|---|
| `items:` (required) | `[Item]` | The items at the top level |
| `children:` (required) | `(Item) -> [Item]?` | An item's children: `f => f.children`; null or an empty list for a leaf |
| `key:` (required) | `(Item) -> String or (Item) -> Number` | What tells two items apart: `f => f.path`; what `bind:` and `open:` hold |
| `bind:` | `State` | The state holding the chosen item's key, written on Enter, Space or a press; with `.multiple`, a list of keys |
| `multiple:` | `Bool` | Several chosen at once: Space, Shift with an arrow, Ctrl or ⌘ with A; `bind:` holds a list |
| `open:` | `[String] or [Number]` | The keys of the items open when it is first drawn |
| `label:` (required) | `String` | What the tree is, its accessible name |

**Flags:** `.multiple`

Attribute families: `global`, `aria`, `data`.

### DataTable

Rows of records with their columns declared once: a column sorts when it is `.sortable` (its header a button, `aria-sort` saying which way), `cell(r) { … }` draws a cell, `empty { … }` shows when there are no rows, and `bind:` chooses rows (a list with `.multiple`, a box for each and one for all).

```
DataTable(rows: …, key: …, bind: …) { … }
```

```wf
state deploys = [{ id: "d1", service: "checkout", region: "fra1", minutes: 4, status: "Live" }, { id: "d2", service: "search", region: "iad1", minutes: 11, status: "Failed" }, { id: "d3", service: "accounts", region: "sin1", minutes: 7, status: "Live" }]
state chosen: [String] = []
DataTable(rows: deploys, key: d => d.id, bind: chosen, label: "Deploys").multiple {
    DataTable.Column("Service", value: d => d.service).sortable
    DataTable.Column("Region", value: d => d.region)
    DataTable.Column("Minutes", value: d => d.minutes).sortable
    DataTable.Column("Status") {
        cell(d) { Badge(d.status, tone: if d.status == "Live" { .success } else { .danger }) }
    }
}
Text("{chosen.length} chosen").muted
```

Renders with the class `wf-data-table`.

| Prop | Type | Meaning |
|---|---|---|
| `rows:` (required) | `[Item]` | The records, a row each |
| `key:` (required) | `(Item) -> String or (Item) -> Number` | What tells two rows apart: `r => r.id`; what `bind:` holds, and what keeps a row's nodes as it moves |
| `bind:` | `State` | The state holding the chosen row's key (a radio a row), or with `.multiple` a list of keys (a box a row and one for all); without it, no column to choose |
| `multiple:` | `Bool` | Several rows at once, with a box in the header that takes them all |
| `label:` (required) | `String` | What the table is: its caption, visually hidden, read by a screen reader |
| `filter:` | `String` | Only the rows where a column's `value:` holds this text, its case ignored; a column with only `cell` is not searched. A change goes back to page 1 |
| `pageSize:` | `Number` | Rows a page, behind Previous and Next with `1–20 of 54`; 0 shows every row. Whether there are pages is decided when it is drawn |

**Flags:** `.multiple`

**Parts:**

- `DataTable.Column(title, value: …, sortable: …)` — One column: its title, its `value:` (what it shows and sorts by), and a `cell(r) { … }` that draws it instead.
  - `title:` `String` — Its header
  - `value:` `(Item) -> String or (Item) -> Number or (Item) -> Bool or (Item) -> Money` — What the column holds for a row: `r => r.service`; shown as text unless `cell(r)` draws it, and what sorting and `filter:` read
  - `sortable:` `Bool` — Its header is a button that sorts the rows by its `value:`, ascending then descending (`aria-sort`); needs `value:`
  - flags: `.sortable`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Chart

A chart drawn as vector graphics when the page is built: bars, lines, areas, a pie or a donut over a list of records.

```
Chart(kind: …, data: …, x: …)
```

```wf
Chart(kind: .bar, data: [
    { month: "Jan", web: 41, store: 22 },
    { month: "Feb", web: 48, store: 25 },
    { month: "Mar", web: 56, store: 24 },
    { month: "Apr", web: 61, store: 30 },
], x: "month", y: ["web", "store"], unit: "k").legend
```

Renders with the class `wf-chart`.

| Prop | Type | Meaning |
|---|---|---|
| `kind:` | `.bar` `.line` `.area` `.pie` `.donut` | What it draws (also written `type:`) |
| `data:` | `[Json]` | The rows: a list of records (`{ month: "Jan", web: 41 }`) or of numbers, known when the page is built (`E109` on a page otherwise) |
| `x:` | `String` | The key of each row's label along the bottom (a pie's legend) |
| `y:` | `String or [String]` | The key of each row's value, or a list of keys for several series; a pie or donut uses the first |
| `colors:` | `[String]` | The series' colours in order (a pie's: one a slice), repeated when there are more; one that is not a CSS colour is replaced by the default |
| `width:` | `Number` | The drawing's width in its own units (its 11-unit text scales with it); on a page the SVG fills its box's width, so with `height:` it sets the shape |
| `height:` | `Number` | The drawing's height in its own units; with `width:` it sets the shape |
| `ink:` | `String` | The colour of its axis labels, value labels and legend text, a CSS colour |
| `grid:` | `String` | The colour of its horizontal grid lines, a CSS colour |
| `legend:` | `Bool` | Name the series above a bar, line or area chart (only with two or more), or list each slice with its value beside a pie; `legend: false` leaves it out |
| `stacked:` | `Bool` | Stack a bar chart's series in one bar a row; other kinds ignore it |
| `labels:` | `Bool` | Write each bar's value above it (bars over 12 units tall) or each slice's percentage on it (slices over about 17°); lines and areas ignore it |
| `unit:` | `String` | Written after each value on the axis and in a pie's legend: `ms`, `%` |

**Flags:** `.bar` `.line` `.area` `.pie` `.donut` `.legend` `.stacked` `.labels`

Attribute families: `global`, `aria`, `data`.

### QrCode

A QR code for a URL or any text, drawn as vector graphics when the page is built.

```
QrCode(value, color: …, background: …)
```

```wf
Card.outlined {
    Row(gap: .md, align: .center) {
        QrCode("https://example.com/tickets/QX-4821")
        Stack(gap: .xs) {
            Text("Ticket QX-4821").bold
            Text("Show this at the door.").muted
        }
    }
}
```

Renders with the class `wf-qr`.

| Prop | Type | Meaning |
|---|---|---|
| `value` (positional) | `String` | What it encodes, a URL or any text, known when the page is built (`E109` on a page otherwise); error correction level M, a 4-module quiet zone |
| `color:` | `String` | The colour of the dark modules, a CSS colour; anything else is the default |
| `background:` | `String` | The colour behind them, a CSS colour; anything else is the default |

Attribute families: `global`, `aria`, `data`.

## Form

### ColorPicker

A colour, chosen from `swatches:` (each a button, the chosen one pressed), with the browser's own picker, or typed as hex; `bind:` holds it as `#rrggbb`.

```
ColorPicker(bind: …, label: …, swatches: …)
```

```wf
state accent = "#0f766e"
ColorPicker(bind: accent, label: "Accent colour", swatches: ["#0f766e", "#1d4ed8", "#7c3aed", "#b91c1c", "#c2410c"])
Button("Preview").primary { style { background: {accent} } }
Text("Chosen: {accent}").muted
```

Renders with the class `wf-color-picker`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The `String` state it reads and writes: always a lowercase `#rrggbb`; a value that is not a hex colour shows as `#000000` |
| `label:` (required) | `String` | What the colour is for: the group's name, and the start of its picker's and hex field's names |
| `swatches:` | `[String]` | The colours offered as buttons before the picker: `#rrggbb` or `#rgb`; anything else is left out, and with none the row is hidden |

Attribute families: `global`, `aria`, `data`.

### TransferList

Two lists side by side — what can be chosen and what is — with buttons that move the selected across, or all of them; `bind:` holds the keys of the chosen.

```
TransferList(bind: …, options: …, text: …)
```

```wf
state picked: [String] = ["ada"]
TransferList(bind: picked, options: [
    { id: "ada", name: "Ada Lovelace" },
    { id: "grace", name: "Grace Hopper" },
    { id: "alan", name: "Alan Turing" },
    { id: "katherine", name: "Katherine Johnson" },
], text: p => p.name, key: p => p.id, label: "Reviewers", from: "Everyone")
Text("Reviewers: {picked.join(", ")}").muted
```

Renders with the class `wf-transfer-list`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | A list state holding the keys of the chosen options, kept in the order of `options:`; anything else reads as nothing chosen |
| `options:` (required) | `[Item]` | Everything that can be chosen, in the order both lists show it |
| `text:` | `(Item) -> String` | What an option says: `o => o.name` |
| `key:` | `(Item) -> String or (Item) -> Number` | What identifies an option, and what `bind:` holds for it: `o => o.id` |
| `label:` (required) | `String` | What is being chosen: the name of the whole control |
| `from:` | `String` | The heading over the list of what is not chosen |
| `to:` | `String` | The heading over the list of what is chosen |

Attribute families: `global`, `aria`, `data`.

### Editable

Text that turns into a field when pressed: Enter or leaving keeps what was typed, Escape puts it back; `bind:` holds the text.

```
Editable(bind: …, label: …, placeholder: …)
```

```wf
state project = "Quarterly report"
Editable(bind: project, label: "Project name", placeholder: "Name the project")
Text("Saved as “{project}”").muted
```

Renders with the class `wf-editable`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The `String` state it shows and writes; Enter or leaving the field writes what was typed, Escape leaves it as it was |
| `label:` (required) | `String` | What the text is: the field's name, and the button's (`Name: Ada, edit`) for a screen reader |
| `placeholder:` | `String` | Shown in place of the text while it is empty, and in the field while editing |

Attribute families: `global`, `aria`, `data`.

### Input

Text field. With `label`, `hint` or `error` it is a labelled field.

```
Input(reveal: …, bind: …, label: …)
```

```wf
state name = "Ada Lovelace"
state email = ""
Stack(gap: .md) {
    Input(bind: name, label: "Full name", placeholder: "Ada Lovelace").text
    Input(bind: email, label: "Work email", hint: "We send the invoice here.", placeholder: "ada@example.com").email.required
    Text(if email == "" { "Hello, {name}" } else { "Hello, {name}: we write to {email}" }).muted
}
```

Renders with the class `wf-input`.

| Prop | Type | Meaning |
|---|---|---|
| `reveal:` | `Bool` | With `.password`: a Show/Hide button after the field that shows the password as text and hides it again, pressed (`aria-pressed`) while it shows |
| `bind:` | `State` | The state it reads and writes as you type: a `String`, a `Number` with `.number` (`null` when emptied), a `Date` with `.date`, a `Time` with `.time`, a `DateTime` with `.datetime` |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `type:` | `.text` `.email` `.password` `.number` `.search` `.tel` `.url` `.date` `.time` `.datetime` `.color` | What the field holds, and so the keyboard and picker a browser offers; it also sets what `bind:` holds |
| `placeholder:` | `String` | Hint shown while the field is empty |
| `value:` | `String or Number` | The text it starts with, when no state is bound; `bind:` takes its place |
| `min:` | `Number or String` | The lowest value a `.number`, `.date` or `.time` field takes: a number, or a date or time as text (`"2026-01-01"`) |
| `max:` | `Number or String` | The highest value a `.number`, `.date` or `.time` field takes: a number, or a date or time as text |
| `step:` | `Number` | How far the arrow keys and the spinner move a `.number` value (seconds for `.time`) |
| `required:` | `Bool` | Whether the browser refuses to submit the form while it is empty |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |
| `size:` | `.sm` `.md` `.lg` | Its padding and text size |
| `width:` | `.full` | Its width |
| `rounded:` | `Bool` | Fully rounded ends, a pill shape |

**Flags:** `.reveal` `.text` `.email` `.password` `.number` `.search` `.tel` `.url` `.date` `.time` `.datetime` `.color` `.required` `.disabled` `.sm` `.md` `.lg` `.full` `.rounded`

Attribute families: `global`, `aria`, `data`, `input`.

### Select

Drop-down of `Select.Option` children.

```
Select(bind: …, value: …, label: …) { … }
```

```wf
state region = "eu-west"
Select(bind: region, label: "Region", hint: "Where your data is stored.") {
    Select.Option("Europe (Ireland)", value: "eu-west")
    Select.Option("US East (Virginia)", value: "us-east")
    Select.Option("Asia Pacific (Tokyo)", value: "ap-northeast")
}
Text("Chosen: {region}").muted
```

Renders with the class `wf-select`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The state it reads and writes: the chosen option's `value:` as it was written (a `String`, `Number` or `Bool`), or its label when it has none |
| `value:` | `String or Number` | The option chosen at first, by its value, when no state is bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |
| `multiple:` | `Bool` | A list box that takes several choices; a bound `[String]` state holds every one chosen |

**Flags:** `.disabled` `.multiple`

**Parts:**

- `Select.Option(label, value: …)` — One choice: the visible label, and the value sent with the form.
  - `label:` `String` — The words the option shows; also its value when it has no `value:`
  - `value:` `String or Number or Bool` — What `bind:` holds when this one is chosen, kept as written (`String`, `Number` or `Bool`)

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `input`.

### Checkbox

Checkbox bound to a boolean state.

```
Checkbox(bind: …, checked: …, label: …)
```

```wf
state terms = false
state newsletter = true
Stack(gap: .sm) {
    Checkbox(bind: terms, label: "I accept the terms of service")
    Checkbox(bind: newsletter, label: "Send me the monthly newsletter")
    Text(if terms { "Ready to sign up." } else { "Accept the terms to go on." }).muted
}
```

Renders with the class `wf-checkbox`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `Bool` state it reads and writes: true while ticked |
| `checked:` | `Bool` | Whether it is ticked, when no state is bound: shown, not written back — handle `on change` to keep it (`F04` otherwise) |
| `label:` | `String` | The words after the box, which a press on also ticks; also its accessible name |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |

**Flags:** `.checked` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Radio

One choice of a group; every radio bound to the same state is one group.

```
Radio(bind: …, value: …, checked: …)
```

```wf
state shipping = "standard"
Stack(gap: .sm) {
    Radio(bind: shipping, value: "standard", label: "Standard — 5 to 7 days, free")
    Radio(bind: shipping, value: "express", label: "Express — 2 days, $9")
    Radio(bind: shipping, value: "overnight", label: "Overnight — $24")
    Text("Chosen: {shipping}").muted
}
```

Renders with the class `wf-radio`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The state shared by a group of radios: it holds the chosen radio's `value:`, and radios bound to one state are one group for the arrow keys |
| `value:` | `String or Number or Bool` | What `bind:` holds when this one is chosen, and what a form sends for it; a bound radio without one is never checked |
| `checked:` | `Bool` | Whether it is chosen, when no state is bound: shown, not written back |
| `label:` | `String` | The words after the button, which a press on also chooses; also its accessible name |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |

**Flags:** `.checked` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Switch

On/off toggle bound to a boolean state.

```
Switch(bind: …, checked: …, label: …)
```

```wf
state replies = true
state digest = false
Stack(gap: .sm) {
    Switch(bind: replies, label: "Email me when someone replies")
    Switch(bind: digest, label: "Send a weekly digest")
    Text("Replies {if replies { "on" } else { "off" }}, digest {if digest { "on" } else { "off" }}").muted
}
```

Renders with the class `wf-switch`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `Bool` state it reads and writes: true while on |
| `checked:` | `Bool` | Whether it is on, when no state is bound: shown, not written back — handle `on change` to keep it |
| `label:` | `String` | The words after the switch, which a press on also flips; also its accessible name |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |

**Flags:** `.checked` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Slider

Range input. `aria-*` and its other attributes reach the input; with `aria-valuetext` the raw number is not shown beside the track.

```
Slider(bind: …, min: …, max: …)
```

```wf
state volume = 60
Slider(bind: volume, min: 0, max: 100, step: 5, label: "Volume")
Text("Volume: {volume}%").muted
```

Renders with the class `wf-slider`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `Number` state it reads and writes as it is dragged; with `.range` a `[Number]` holding `[low, high]`. Its value is shown beside the track |
| `min:` | `Number` | The value at the start of the track |
| `max:` | `Number` | The value at the end of the track |
| `step:` | `Number` | How far one move of the thumb, or one arrow key, changes the value |
| `range:` | `Bool` | Two thumbs, a low and a high that cannot pass each other: `bind:` holds `[low, high]`, read as `[min, max]` until it holds two numbers |
| `label:` | `String` | The label drawn above the track and the input's accessible name; with `.range` no label is drawn, and it names the group and each thumb (`Price, lowest`) |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |

**Flags:** `.range` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### DatePicker

A date field: the reader picks a day, and the bound state holds a `Date`.

```
DatePicker(bind: …, label: …, min: …)
```

```wf
state start: Date? = @2026-11-17
DatePicker(bind: start, label: "Start date", min: "2026-01-01", max: "2026-12-31")
if let d = start {
    Text("Starts {d:.date(long)}").muted
} else {
    Text("No date chosen").muted
}
```

Renders with the class `wf-datepicker`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `Date?` state it reads and writes: the day chosen, `null` when the field is emptied |
| `label:` | `String` | The label drawn above the field; also its accessible name |
| `min:` | `String` | The earliest day the picker offers, `YYYY-MM-DD` |
| `max:` | `String` | The latest day the picker offers, `YYYY-MM-DD` |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |

**Flags:** `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### FileUpload

File input; `bind:` holds what was chosen, and `on change` hears it.

```
FileUpload(accept: …, label: …, multiple: …) { … }
```

```wf
state photo: File? = null
FileUpload(bind: photo, accept: "image/*", label: "Profile picture")
if let p = photo {
    Text("Chosen: {p.name}").muted
} else {
    Text("No picture chosen").muted
}
```

Renders with the class `wf-file-upload`.

| Prop | Type | Meaning |
|---|---|---|
| `accept:` | `String` | The kinds of file it offers: MIME types or extensions, comma-separated (`image/*`, `.pdf,.doc`) |
| `label:` | `String` | The label drawn above the file input and its accessible name; with `.dropzone`, the words of its Choose button |
| `multiple:` | `Bool` | Several files at once: `bind:` holds a `[File]`, and with `.dropzone` each choice adds to the list |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |
| `bind:` | `State` | The state the choice is written to: a `File?` (`null` when none), or a `[File]` with `.multiple` |
| `dropzone:` | `Bool` | An area files are dropped on or chosen into, each listed with its name, a preview if it is an image, and a button to take it out |

**Flags:** `.multiple` `.disabled` `.dropzone`

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `input`.

### Form

Groups controls; `on submit` runs when it is submitted, and the page never navigates away.

```
Form(bind: …, show: …) { … }
```

```wf
state email = ""
state password = ""
state sent = ""
validate email {
    required "Enter your email"
    email "That is not an email address"
}
validate password {
    required
    minLength(8) "Use at least 8 characters"
}
Form(bind: form, show: .onBlur) {
    on submit { sent = "Signed in as {email}" }
    Input(bind: email, label: "Email").email
    Input(bind: password, label: "Password").password
    Button("Sign in", type: .submit, disabled: !form.valid).primary
}
Text(if sent == "" { "Not signed in yet." } else { sent }).muted
```

Renders with the class `wf-form`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A name for a handle on the form, not a state: `form.valid`, `form.errors`, `form.touched`, `form.pending`, `form.values` by field name, `form.reset()`, `form.submit()`, `form.apply(serverErrors)` |
| `show:` | `.onBlur` `.onSubmit` `.live` | When a `validate` block's message is shown on its field |

**Flags:** `.onBlur` `.onSubmit` `.live`

Takes a block of children.

Attribute families: `global`, `aria`, `data`, `form`.

### Textarea

Several lines of text. With `label`, `hint` or `error` it is a labelled field; with `maxLength` it counts what is left.

```
Textarea(bind: …, label: …, hint: …)
```

```wf
state feedback = ""
Textarea(bind: feedback, label: "What could we do better?", hint: "Be as specific as you like.", placeholder: "The export was slow when…", rows: 4, maxLength: 280)
Text("{feedback.length} of 280 characters").muted
```

Renders with the class `wf-input wf-textarea`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `String` state it reads and writes as you type |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `placeholder:` | `String` | Hint shown while the field is empty |
| `rows:` | `Number` | How many lines tall it is drawn; never less than 6rem |
| `maxLength:` | `Number` | The most characters it takes; a count beneath it (`12 / 200`) says how many are used, read politely |
| `required:` | `Bool` | Whether the browser refuses to submit the form while it is empty |
| `autosize:` | `Bool` | Grows taller with its text, from `rows:` up, rather than scrolling |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it |

**Flags:** `.required` `.autosize` `.disabled`

Attribute families: `global`, `aria`, `data`, `input`.

### Field

A label, a hint and an error around a control of your own: the first control in its block is named by the label and described by the hint and the error, and marked invalid while there is one.

```
Field(label: …, hint: …, error: …) { … }
```

```wf
state color = "#0f766e"
derived colorError = if color == "#ffffff" { "White disappears on the page." } else { "" }
Field(label: "Brand colour", hint: "Used for buttons and links.", error: colorError) {
    ColorPicker(bind: color, label: "Brand colour", swatches: ["#0f766e", "#1d4ed8", "#b91c1c", "#ffffff"])
}
Text("Chosen: {color}").muted
```

Renders with the class `wf-field wf-field--custom`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The label above the control; it names the first control in the block (`aria-labelledby`), and a press on it focuses that control |
| `hint:` | `String` | Help text under the control, linked to the first control in the block by `aria-describedby` |
| `error:` | `String` | Error text under the control: `null` or empty for none; while there is one it shows, is announced, and marks the control `aria-invalid` |

**Parts:**

- `Field.Label(text) { … }` — Its label. Drawn for it.
  - `text:` `String` — The label's words; the first control in the `Field` is named by it
- `Field.Hint(text) { … }` — Its hint. Drawn for it.
  - `text:` `String` — The hint's words; the first control in the `Field` is described by it
- `Field.Error(text) { … }` — Its error, announced as it appears. Drawn for it.
  - `text:` `String` — The error's words; the first control in the `Field` is described by it, and it is announced (`role="alert"`)

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
state postcode = ""
Fieldset(legend: "Shipping address") {
    Input(bind: street, label: "Street")
    Row(gap: .md) {
        Input(bind: city, label: "City")
        Input(bind: postcode, label: "Postcode")
    }
}
Text("{street}, {postcode} {city}").muted
```

Renders with the class `wf-fieldset`.

| Prop | Type | Meaning |
|---|---|---|
| `legend:` (required) | `String` | What the group is: drawn as its `<legend>`, which names the group for a screen reader |

**Parts:**

- `Fieldset.Legend(text) { … }` — Its legend. Drawn for it.
  - `text:` `String` — The legend's words: the group's name

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### InputGroup

An input drawn as one box with what goes beside it: an icon, `InputGroup.Addon("https://")`, a button, or `InputGroup.Clear`, which empties the input and gives it the focus back.

```
InputGroup { … }
```

```wf
state search = ""
state domain = "acme"
Stack(gap: .md) {
    InputGroup {
        Icon("search")
        Input(bind: search, placeholder: "Search the docs", aria-label: "Search the docs")
        InputGroup.Clear
    }
    InputGroup {
        InputGroup.Addon("https://")
        Input(bind: domain, aria-label: "Subdomain")
        InputGroup.Addon(".example.com")
    }
    Text("Searching for “{search}” on https://{domain}.example.com").muted
}
```

Renders with the class `wf-input-group`.

**Parts:**

- `InputGroup.Addon(text) { … }` — Text beside the input: `https://`, `.com`, `kg`.
  - `text:` `String` — The words drawn beside the input, inside the group's box: `https://`, `.com`, `kg`
- `InputGroup.Clear` — A button that empties the input and puts the focus back in it; hidden while it is empty.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### RadioGroup

One choice of several, as native radio buttons under a legend: the arrow keys move between them, and `.cards` draws each as a card with its hint.

```
RadioGroup(bind: …, label: …, cards: …) { … }
```

```wf
state plan = "team"
RadioGroup(bind: plan, label: "Plan").cards {
    RadioGroup.Option("Free", value: "free", hint: "For trying it out")
    RadioGroup.Option("Team", value: "team", hint: "Up to 20 people, $12 each")
    RadioGroup.Option("Scale", value: "scale", hint: "Unlimited people, single sign-on")
}
Text("Chosen: {plan}").muted
```

Renders with the class `wf-radio-group`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The state it reads and writes: the chosen option's `value:` (a `String`, `Number` or `Bool`) |
| `label:` (required) | `String` | What is being chosen: drawn as the group's legend, which names it for a screen reader |
| `cards:` | `Bool` | Each option drawn as a card, its hint beneath its words |

**Flags:** `.cards`

**Parts:**

- `RadioGroup.Option(label, value: …, hint: …) { … }` — One choice: its words, its value, and a hint said with it.
  - `label:` `String` — The option's words, beside its radio
  - `value:` `String or Number or Bool` — What `bind:` holds when it is chosen
  - `hint:` `String` — A line beneath its words, read with the option (`aria-description`)
- `RadioGroup.Legend(text) { … }` — Its legend. Drawn for it.
  - `text:` `String` — The legend's words: the group's name
- `RadioGroup.Hint(text) { … }` — An option's hint. Drawn for it.
  - `text:` `String` — The words of an option's hint, drawn beneath it

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### NumberInput

A number with − and + beside it: the arrow keys step it, PageUp and PageDown by ten steps, and it stays between `min:` and `max:`.

```
NumberInput(bind: …, min: …, max: …)
```

```wf
state tickets = 2
NumberInput(bind: tickets, min: 1, max: 10, step: 1, label: "Tickets", hint: "Ten at most per order.")
Text("Total: ${tickets * 45}").muted
```

Renders with the class `wf-number-input`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The `Number` state it reads and writes; typing writes `null` when the field is emptied, and a number past `min:` or `max:` is brought back within them when it changes |
| `min:` | `Number` | The lowest value: − is disabled there, and the buttons and PageDown never go below it (a typed value is not held to it) |
| `max:` | `Number` | The highest value: + is disabled there, and the buttons and PageUp never go above it (a typed value is not held to it) |
| `step:` | `Number` | How far −, + and the arrow keys move it; PageUp and PageDown move ten steps |
| `label:` | `String` | Visible label; also the input's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |

**Parts:**

- `NumberInput.Decrease(disabled: …) { … }` — Its − button. Drawn for it.
  - `disabled:` `Bool` — Whether the − button is inert; `NumberInput` sets it while the value is at `min:`
  - flags: `.disabled`
- `NumberInput.Increase(disabled: …) { … }` — Its + button. Drawn for it.
  - `disabled:` `Bool` — Whether the + button is inert; `NumberInput` sets it while the value is at `max:`
  - flags: `.disabled`

Attribute families: `global`, `aria`, `data`.

### Calendar

A month as a grid of days, chosen with a press or the keyboard — the arrows by day and week, PageUp and PageDown by month, Home and End to the week's ends — one day, a range, or several. `bind:` holds a `Date`, a `{ start, end }` with `.range`, a list with `.multiple`.

```
Calendar(bind: …, mode: …, min: …)
```

```wf
state delivery: Date? = @2026-11-17
Calendar(bind: delivery, min: @2026-11-17, disabled: d => d.weekday() >= 6, label: "Delivery day").single
if let d = delivery {
    Text("Delivered on {d:.date(long)}").muted
}
```

Renders with the class `wf-calendar`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The state it reads and writes, by `mode:`: a `Date?`, a `{ start, end }` of `Date`s (`end` null until the second day), or a `[Date]` kept in order |
| `mode:` | `.single` `.range` `.multiple` | How many days it holds |
| `min:` | `Date` | The first day that can be chosen; days before it are shown but cannot be chosen |
| `max:` | `Date` | The last day that can be chosen; days after it are shown but cannot be chosen |
| `disabled:` | `[Date] or (Date) -> Bool` | Days that cannot be chosen: a list of them, or a test of a day, `d => d.weekday() >= 6` |
| `label:` | `String` | Its name for a screen reader, read once when it is drawn |

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
    Listbox.Option("Mango", value: "mango")
}
Text("Chosen: {fruit}").muted
```

Renders with the class `wf-listbox`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The state it reads and writes: the chosen option's `value:`, or with `.multiple` a list of them in the order they were chosen |
| `label:` (required) | `String` | What is being chosen: the list's accessible name (`aria-label`) |
| `multiple:` | `Bool` | Several at once: `bind:` is a list; Space toggles, Shift with an arrow takes the next one in, Ctrl/⌘+A all |

**Flags:** `.multiple`

**Parts:**

- `Listbox.Option(label, value: …, icon: …, aria-disabled: …) { … }` — One option: its words, and the value `bind:` holds when it is chosen.
  - `label:` `String` — The option's words; a letter typed goes to the next option they start with
  - `value:` `String or Number or Bool` — What `bind:` holds when it is chosen (or, with `.multiple`, what the list holds while it is chosen)
  - `icon:` `String` — The name of a built-in icon drawn before its words
  - `aria-disabled:` `String` — `"true"`: shown, passed over by the arrow keys, never chosen

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Combobox

A text field that offers what matches as the reader types: from `options:`, or `search:` asked of a server. The arrow keys go through the offers, Enter takes one, Escape closes them; `.multiple` keeps several, each a chip Backspace takes back. `option(o) { … }` draws an offer; `empty { … }` shows when nothing matches.

```
Combobox(bind: …, options: …, search: …)
```

```wf
state city = "lis"
Combobox(bind: city, options: [
    { id: "ams", name: "Amsterdam" },
    { id: "ber", name: "Berlin" },
    { id: "lis", name: "Lisbon" },
    { id: "mad", name: "Madrid" },
    { id: "par", name: "Paris" },
], text: c => c.name, key: c => c.id, label: "City", placeholder: "Type a city")
Text("Chosen: {city}").muted
```

Renders with the class `wf-combobox`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The state it reads and writes: the chosen option's key (the option itself without `key:`), or with `.multiple` a list of keys in the order chosen |
| `options:` | `[Item]` | Every option, filtered as the reader types to those whose words contain what is typed, case ignored; use this or `search:` |
| `search:` | `(String) -> [Item] or (String) -> [Item]!` | Options asked for as the reader types, each time the typing rests for `debounce:`: a function of the text giving a list, or an endpoint's call (`q => Geo.cities(q: q)`), whose `.ok` list is shown and whose failure shows none |
| `text:` (required) | `(Item) -> String` | An option's words, shown in the list and in the field once chosen: `p => p.name` |
| `key:` | `(Item) -> String or (Item) -> Number` | What tells two options apart, and what `bind:` holds: `p => p.id` |
| `debounce:` | `Number or Duration` | How long the typing must rest before `search:` is asked: a `Duration` (`250.ms`) or milliseconds |
| `multiple:` | `Bool` | Several at once, each a chip with a button to take it out; Backspace in the empty field takes the last back |
| `label:` (required) | `String` | Its name for a screen reader: the field's and the list's (`aria-label`) |
| `placeholder:` | `String` | Shown in the field while it is empty |

**Flags:** `.multiple`

Attribute families: `global`, `aria`, `data`.

### CheckboxGroup

Check boxes under a legend whose `bind:` is the list of the values ticked, with `all:` a box above them for every one at once — ticked when all are, mixed when some are.

```
CheckboxGroup(bind: …, label: …, all: …) { … }
```

```wf
state channels: [String] = ["email"]
CheckboxGroup(bind: channels, label: "Notify me by", all: "Every channel") {
    CheckboxGroup.Option("Email", value: "email")
    CheckboxGroup.Option("Text message", value: "sms")
    CheckboxGroup.Option("Push notification", value: "push")
}
Text("Chosen: {channels.join(", ")}").muted
```

Renders with the class `wf-checkbox-group`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The list state it reads and writes: the `value:` of every option ticked, in the order they were ticked |
| `label:` (required) | `String` | What is being chosen: drawn as the group's legend, which names it for a screen reader |
| `all:` | `String` | The words of a first box that ticks every option or none, mixed while some are ticked; only when the options are written out, not drawn by a `for` or an `if` |

**Parts:**

- `CheckboxGroup.Option(label, value: …)` — One box: its words and its value.
  - `label:` `String` — The option's words, beside its box; also its value when it has no `value:`
  - `value:` `String or Number or Bool` — What the list holds while it is ticked
- `CheckboxGroup.Legend(text) { … }` — Its legend. Drawn for it.
  - `text:` `String` — The legend's words: the group's name

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### InputOTP

A one-time code typed a digit to a box: the focus moves on as each is typed and back with Backspace, and a code pasted into any box fills them all; the browser offers the code it was sent (`autocomplete="one-time-code"`).

```
InputOTP(bind: …, length: …, label: …)
```

```wf
state code = ""
derived left = 6 - code.length
InputOTP(bind: code, length: 6, label: "Code from your authenticator app")
Text(if code.length == 6 { "Checking {code}…" } else { "{left} digit(s) to go" }).muted
```

Renders with the class `wf-otp`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The `String` state it reads and writes: the digits typed so far, in order; anything but a digit is dropped |
| `length:` | `Number` | How many boxes, one digit each |
| `label:` (required) | `String` | What the code is: the group's name for a screen reader; each box is named `Digit 1 of 6` |

Attribute families: `global`, `aria`, `data`.

### TagInput

Words typed into a field, each a chip once Enter or a comma ends it; Backspace in the empty field takes the last back, and each chip has a button to take it out.

```
TagInput(bind: …, label: …, placeholder: …)
```

```wf
state tags: [String] = ["design", "frontend"]
TagInput(bind: tags, label: "Tags", placeholder: "Type a tag, then Enter")
Text("{tags.length} tag(s): {tags.join(", ")}").muted
```

Renders with the class `wf-tag-input`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The `[String]` state it reads and writes: the words, in the order added; a word already there is not added again |
| `label:` (required) | `String` | What the words are: the field's name for a screen reader |
| `placeholder:` | `String` | Shown while the field is empty |

Attribute families: `global`, `aria`, `data`.

### Rating

Stars, as radio buttons a screen reader hears as a choice of one to `max:`: the arrow keys move between them; `.half` takes halves; `.readonly` only shows.

```
Rating(bind: …, max: …, label: …)
```

```wf
state stars = 4
Rating(bind: stars, max: 5, label: "Your rating")
Text("You gave it {stars} of 5").muted
```

Renders with the class `wf-rating`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The `Number` state it reads and writes: how many stars are chosen (halves with `.half`), 0 for none |
| `max:` | `Number` | How many stars it draws, and the highest rating |
| `label:` (required) | `String` | What is being rated: the group's name, read once; with `.readonly` it says `Quality: 4 out of 5` |
| `half:` | `Bool` | Half stars too: the rating moves in steps of 0.5 rather than 1 |
| `readonly:` | `Bool` | Shown, not chosen: drawn as an image (`role="img"`) that says the rating, with no stops for Tab |

**Flags:** `.half` `.readonly`

Attribute families: `global`, `aria`, `data`.

### DateRangePicker

A first and a last day, chosen in two months side by side that a button opens, or by a preset: `presets: [.last7, .thisMonth]`. `bind:` holds `{ start, end }`.

```
DateRangePicker(bind: …, label: …, min: …)
```

```wf
state period = { start: @2026-11-01, end: @2026-11-17 }
DateRangePicker(bind: period, label: "Report period", presets: [.today, .last7, .last30, .thisMonth, .lastMonth])
Text("{period.start:.date(medium)} to {period.end:.date(medium)}").muted
```

Renders with the class `wf-date-range`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The state it reads and writes: a `{ start, end }` of `Date`s, `end` null until the second day; the panel closes once both are chosen |
| `label:` (required) | `String` | What the range is: the button's words (`Stay: 3 Mar 2026 – 9 Mar 2026` once chosen) and the panel's name |
| `min:` | `Date` | The first day that can be chosen in either month |
| `max:` | `Date` | The last day that can be chosen in either month |
| `presets:` | `[RangePreset]` | Buttons above the months, each a range a press takes: `.today`, `.last7`, `.last30`, `.thisMonth`, `.lastMonth`, in the order listed |

Attribute families: `global`, `aria`, `data`.

## Actions

### Fab

A round button floating at the bottom of the screen for the page's main action; with `Fab.Action`s in its block it opens them instead (a speed dial).

```
Fab(icon: …, label: …) { … }
```

```wf
state said = "Nothing made yet"
Fab(icon: "plus", label: "Create") {
    Fab.Action("Note", icon: "file") { on click { said = "A new note" } }
    Fab.Action("Folder", icon: "folder") { on click { said = "A new folder" } }
    Fab.Action("Upload", icon: "upload") { on click { said = "An upload" } }
}
Text(said).muted
```

Renders with the class `wf-fab`.

| Prop | Type | Meaning |
|---|---|---|
| `icon:` (required) | `String` | The icon on the round button, one of the built-in icon names; required with `label:`, or the call draws as a plain element |
| `label:` (required) | `String` | What the button does: its accessible name and tooltip (`aria-label`, `title`); never shown |

**Parts:**

- `Fab.Action(label, icon: …) { … }` — One more action the button opens: its words and its icon; `on click` is what it does.
  - `label:` `String` — The action's words, shown beside its icon while the dial is open
  - `icon:` `String` — An icon before the words, one of the built-in icon names
- `Fab.Trigger { … }` — Its button. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### BackToTop

A button that appears once the page has scrolled past `after:` px, and takes the reader — and the focus — back to the top.

```
BackToTop(label: …, after: …)
```

```wf
Stack(gap: .md) {
    for n in 1..=12 {
        Text("Chapter {n}: what the page says, long enough to scroll past.")
    }
}
BackToTop(after: 600, label: "Back to the top")
```

Renders with the class `wf-back-to-top`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Its accessible name and tooltip (`aria-label`, `title`); the button shows only an upward chevron |
| `after:` | `Number` | How far the window must scroll down before it appears, in px; `0` counts as 400 |

Attribute families: `global`, `aria`, `data`.

### CopyButton

A button that copies `value:` to the clipboard, says `Copied` for a moment, and tells a screen reader so.

```
CopyButton(label, value: …, copied: …)
```

```wf
Row(gap: .sm, align: .center) {
    Code("https://ledger.example.com/invite/42")
    CopyButton("Copy link", value: "https://ledger.example.com/invite/42", copied: "Link copied")
}
```

Renders with the class `wf-btn wf-copy-button`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Its words |
| `value:` (required) | `String or Number` | What it copies, read when it is pressed; a value that reads state copies what it holds then |
| `copied:` | `String` | What it says for 1.6 s once it has copied, also announced to a screen reader |

Attribute families: `global`, `aria`, `data`.

### ThemeToggle

A button that switches the page between light and dark, over `setTheme`: a moon in a light page, a sun in a dark one, named for the theme it switches to. The reader's choice is kept.

```
ThemeToggle
```

```wf
Row(gap: .sm, align: .center, justify: .between) {
    Text("Appearance").bold
    ThemeToggle
}
Text("The theme is {theme}.").muted
```

Renders with the class `wf-icon-btn wf-theme-toggle`.

Attribute families: `global`, `aria`, `data`.

### LocaleSwitch

One button for each of `i18n.locales`, each written in its own language (`English`, `العربية`), the current one pressed; a press is `setLocale`.

```
LocaleSwitch(label: …)
```

```wf
Row(gap: .sm, align: .center, justify: .between) {
    Text("Language").bold
    LocaleSwitch(label: "Language")
}
```

Renders with the class `wf-locale-switch`.

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The group's accessible name; the group draws one button per `i18n.locales` entry, and nothing with fewer than two |

Attribute families: `global`, `aria`, `data`.

### Toggle

A button that stays pressed while its state holds — bold in an editor, a filter that is on: `aria-pressed`, a press flipping `bind:`.

```
Toggle(label, bind: …, icon: …)
```

```wf
state bold = false
state italic = true
Row(gap: .sm) {
    Toggle("Bold", bind: bold, icon: "type")
    Toggle("Italic", bind: italic)
}
Text("The quick brown fox") {
    style {
        font-weight: {if bold { "700" } else { "400" }}
        font-style: {if italic { "italic" } else { "normal" }}
    }
}
```

Renders with the class `wf-btn wf-toggle`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Its words |
| `bind:` (required) | `State` | The `Bool` state it shows (`aria-pressed`) and flips on each press |
| `icon:` | `String` | An icon before the words, one of the built-in icon names |

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
Text("Showing the {view} view").muted
```

Renders with the class `wf-toggle-group`.

| Prop | Type | Meaning |
|---|---|---|
| `bind:` (required) | `State` | The chosen item's `value:` (pressing the chosen one again keeps it), or with `.multiple` a list of them, each press adding or taking one |
| `label:` (required) | `String` | What the group chooses: its accessible name (`aria-label`) |
| `multiple:` | `Bool` | Several at once: `bind:` holds a list |

**Flags:** `.multiple`

**Parts:**

- `ToggleGroup.Item(label, value: …, icon: …, disabled: …)` — One choice: its words, and the value `bind:` holds when it is chosen.
  - `label:` `String` — Its words
  - `value:` `String or Number or Bool` — What `bind:` holds (or its list contains) while it is chosen; it is drawn pressed then
  - `icon:` `String` — An icon before the words, one of the built-in icon names
  - `disabled:` `Bool` — Whether the item is inert: it cannot be pressed and the arrow keys pass over it
  - flags: `.disabled`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Button

A button: its label or block is what it shows, `on click` what it does.

```
Button(label, tone: …, size: …, shape: …) { … }
```

```wf
state saved = 0
Row(gap: .sm, justify: .end) {
    Button("Cancel").outlined
    Button("Save draft") { on click { saved = saved + 1 } }
    Button("Publish").primary { on click { toast("Published", tone: .success) } }
}
Text("Saved {saved} times").muted
```

Renders with the class `wf-btn`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Its words; a block may hold more (an icon, `Text`) after them |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | The colour it is filled with, from the theme's tones |
| `size:` | `.sm` `.md` `.lg` | Its padding and text size |
| `shape:` | `.rounded` `.pill` | Its corners; unset, the theme's `radius-md` |
| `width:` | `.full` | How wide it is |
| `outlined:` | `Bool` | A border and no fill; with `.primary` or `.danger` its words and border take that colour |
| `type:` | `.button` `.submit` `.reset` | What it does inside a form; any button not `.submit` or `.reset` never submits one |
| `to:` | `Path` | A route to go to when it is pressed, through the router as a `Link`'s `to:`; it stays a `<button>` |
| `disabled:` | `Bool` | Whether it is inert: it cannot be pressed or focused |
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
state stars = 0
Row(gap: .sm, align: .center) {
    IconButton(icon: "star", label: "Star this project").primary { on click { stars = stars + 1 } }
    IconButton(icon: "edit", label: "Edit")
    IconButton(icon: "copy", label: "Copy link")
    IconButton(icon: "trash", label: "Delete").danger
    Text("{stars} stars").muted
}
```

Renders with the class `wf-icon-btn`.

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String` | The icon it shows, one of the built-in icon names; a value that reads state follows it |
| `label:` | `String` | Its accessible name (`aria-label`) and tooltip (`title`, unless one is written), never visible text; without it the tooltip is the icon's name and it has no name (`A02`) |
| `size:` | `.sm` `.md` `.lg` | Its square size |
| `tone:` | `.primary` `.danger` | The icon's colour; the button stays transparent |
| `type:` | `.button` `.submit` `.reset` | What it does inside a form; any button not `.submit` or `.reset` never submits one |
| `disabled:` | `Bool` | Whether it is inert: it cannot be pressed or focused |

**Flags:** `.sm` `.md` `.lg` `.primary` `.danger` `.button` `.submit` `.reset` `.disabled`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### ButtonGroup

Buttons joined into one row.

```
ButtonGroup { … }
```

```wf
state range = "Week"
ButtonGroup {
    Button("Day", outlined: range != "Day") { on click { range = "Day" } }
    Button("Week", outlined: range != "Week") { on click { range = "Week" } }
    Button("Month", outlined: range != "Month") { on click { range = "Month" } }
}
Text("Showing one {range}").muted
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
state said = "Nothing chosen"
Dropdown("Actions") {
    Dropdown.Item { Button("Duplicate") { on click { said = "Duplicated" } } }
    Dropdown.Item { Button("Rename") { on click { said = "Renamed" } } }
    Dropdown.Divider
    Dropdown.Item { Button("Archive").danger { on click { said = "Archived" } } }
}
Text(said).muted
```

Renders with the class `wf-dropdown`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The words of the button that opens its menu; the first argument, `Dropdown("Actions")`, or by name |

**Parts:**

- `Dropdown.Item { … }` — One entry of the menu.
- `Dropdown.Divider` — A rule between groups of entries.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Typography

### Highlight

Text with every match of `query:` marked (`<mark>`), as a search's results show what was found; the case is ignored.

```
Highlight(text, query: …)
```

```wf
state q = "state"
Stack(gap: .sm) {
    Input(bind: q, label: "Search the guide").search
    Highlight("Pages, components and stores hold state; a derived value reads it.", query: q)
    Highlight("A state change repaints only what reads it.", query: q)
}
```

Renders with the class `wf-highlight`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The text, every match of `query:` in it wrapped in `<mark>` |
| `query:` | `String` | What to mark: every match, its case ignored, after trimming; empty marks nothing |

Attribute families: `global`, `aria`, `data`.

### Text

A paragraph of text. `{…}` in the string reads state and stays live.

```
Text(content, tone: …, size: …, align: …)
```

```wf
Stack(gap: .xs) {
    Text("Billing").uppercase.sm.muted
    Text("Your plan renews on 1 December.").lg
    Text("Payment failed: the card was declined.").danger.bold
    Text("Updated 3 minutes ago").muted.sm.italic
}
```

Renders with the class `wf-text`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `String or Number or Bool` | The text, a `<p>`: a string, a number or a boolean; a splice in it follows state |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Its colour, a tone of the theme |
| `size:` | `.sm` `.md` `.lg` | Its font size |
| `align:` | `.left` `.center` `.right` | Where its lines sit |
| `transform:` | `.uppercase` `.lowercase` | The case its letters are drawn in; the text itself is unchanged |
| `kind:` | `.heading` `.subtitle` | A typographic role, still a `<p>` |
| `bold:` | `Bool` | Bold (`font-weight-bold`) |
| `italic:` | `Bool` | Italic |
| `underline:` | `Bool` | Underlined |
| `muted:` | `Bool` | In `color-text-muted`, for secondary text |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.left` `.center` `.right` `.uppercase` `.lowercase` `.heading` `.subtitle` `.bold` `.italic` `.underline` `.muted`

Attribute families: `global`, `aria`, `data`.

### Heading

Heading; `level` picks the tag (`h2` when none is given).

```
Heading(content, level: …, align: …, muted: …)
```

```wf
Stack(gap: .xs) {
    Heading("Pricing").h1
    Heading("Plans for every team").h2.muted
    Heading("What every plan has").h3
}
```

Renders with the class `wf-heading`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `String or Number or Bool` | The heading text: a string, a number or a boolean |
| `level:` | `.h1` `.h2` `.h3` `.h4` `.h5` `.h6` | The tag (`<h1>` … `<h6>`), its place in the outline, and its size |
| `align:` | `.left` `.center` `.right` | Where its lines sit |
| `muted:` | `Bool` | In `color-text-muted` |

**Flags:** `.h1` `.h2` `.h3` `.h4` `.h5` `.h6` `.left` `.center` `.right` `.muted`

Attribute families: `global`, `aria`, `data`.

### Code

Inline code, or a code block with `block`; `language:` colours it.

```
Code(content, block: …, language: …)
```

```wf
Stack(gap: .sm) {
    Text { Text("Start the dev server with ")  Code("wf serve")  Text(", then open the page.") }
    Code("""
        page Home(path: "/", title: "Home") {
            state count = 0
            Button("+1") { on click { count = count + 1 } }
        }
        """, language: "wf").block
}
```

Renders with the class `wf-code`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `String or Number or Bool` | The code, as text in a monospace font |
| `block:` | `Bool` | A block of several lines that keeps its line breaks and scrolls sideways, on the terminal colours, rather than a span in a line |
| `language:` | `String` | Colours the code as this language: `wf` (or `wfx`), `json`, `bash` (`sh`, `shell`) or `css`; any other is shown plain |

**Flags:** `.block`

Attribute families: `global`, `aria`, `data`.

### Markdown

Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run is drawn as its words. `.inert` is for text somebody else wrote.

```
Markdown(text, inert: …)
```

```wf
Markdown("""
    ## Getting started

    Install `wf`, then:

    - write a page in **src/pages**
    - run `wf serve` and open it
    - read [the guide](https://webfluent.monzeromer.dev/learn.md) when you are stuck

    ```wf
    page Home(path: "/", title: "Home") { Heading("Hello").h1 }
    ```
    """)
```

Renders with the class `wf-markdown`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The Markdown, drawn as HTML at build time and live; it is escaped first, so HTML in it is shown, not run |
| `inert:` | `Bool` | Text somebody else wrote, such as a language server's documentation: its entities are decoded, a link is drawn as its words and a picture is left out |

**Flags:** `.inert`

Attribute families: `global`, `aria`, `data`.

### Unsafe

The one door for markup a page did not write. It has a single part, `Unsafe.Html`.

```
Unsafe
```

```wf
Unsafe.Html(sanitize("<p>From the <strong>CMS</strong>: <a href='/learn'>read the guide</a>.</p><script>alert(1)</script>"))
```

**Parts:**

- `Unsafe.Html(markup)` — Markup, put in as markup. It is **not** sanitised — the name says so — and `sanitize(…)` is one call away for anything the project did not write itself.
  - `markup:` `String` — The HTML to insert, as it is: not sanitised (wrap it in `sanitize(…)` unless the project wrote it); every use is a `V03`

Attribute families: `global`, `aria`, `data`.

### Blockquote

Quotation: the text, or its block as the quoted content.

```
Blockquote(content) { … }
```

```wf
Blockquote {
    Text("We moved the whole docs site over in a weekend, and the compiler caught every broken link.").italic
    Text("Ada Lovelace, platform team").muted.sm
}
```

Renders with the class `wf-blockquote`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `String or Number or Bool` | The quoted text; unset, its block |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## Content

### Callout

A note set apart from the text around it — `.note`, `.tip`, `.important`, `.warning`, `.caution` — with an icon and a title, as GitHub's alerts are. Its text is the positional, in Markdown (`code`, **bold**, a link), or its block.

```
Callout(text, kind: …, title: …) { … }
```

```wf
Callout("Run `wf check` before you push: every finding has a **code**, and [a page](https://webfluent.monzeromer.dev/errors.md) that says how to fix it.", title: "Before you push").tip
```

Renders with the class `wf-callout`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | What it says, as Markdown: a `code` span, **bold** and a link show as such; markup in it is shown, not run. Its block follows |
| `kind:` | `.note` `.tip` `.important` `.warning` `.caution` | What kind of note it is: its colour, its icon and its title's word |
| `title:` | `String` | The words of its title row, after the kind's icon; a `Callout.Title` block written by hand draws the row instead |

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
Stack(gap: .xs) {
    Row(gap: .xs, align: .center) { Text("Press")  Kbd("mod+k")  Text("to search the docs.") }
    Row(gap: .xs, align: .center) { Text("Send with")  Kbd("shift+Enter")  Text("and close with")  Kbd("Escape") }
}
```

Renders with the class `wf-kbd`.

| Prop | Type | Meaning |
|---|---|---|
| `keys` (positional) | `String` | The combination, as `on key` names it (`mod+k`, `shift+Enter`), written as this system writes it: `⌘K` on a Mac, `Ctrl+K` elsewhere and in the static paint |

Attribute families: `global`, `aria`, `data`.

### EmptyState

What a list or a page shows when it has nothing yet: an icon, a title, a sentence and, in its block, what to do about it.

```
EmptyState(icon: …, title: …, text: …) { … }
```

```wf
state count = 0
EmptyState(icon: "folder", title: "No projects yet", text: "A project holds your pages, components and stores.") {
    Button("New project").primary { on click { count = count + 1 } }
    Link("Read the guide", to: "/learn")
}
if count > 0 { Text("{count} project(s) asked for").muted }
```

Renders with the class `wf-empty-state`.

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String` | An icon's name, drawn large and muted above the title, hidden from screen readers |
| `title:` (required) | `String` | What is missing: `No projects yet`, in bold |
| `text:` | `String` | A sentence on what to do about it, muted, under the title |

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
Grid(columns: { base: 1, md: 3 }, gap: .md) {
    Stat("Revenue", value: 48210, format: .currency, change: 0.124, trend: [31, 40, 28, 44, 52, 48])
    Stat("Active users", value: 18400, format: .compact, change: 0.031, trend: [15, 16, 16, 17, 18, 18])
    Stat("Churn", value: 0.021, format: .percent, change: -0.004)
}
```

Renders with the class `wf-stat`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | What the figure is, small and muted above it |
| `value:` (required) | `Number or Money or String` | The figure, large; `format:` says how it is written |
| `format:` | `.number` `.integer` `.currency` `.percent` `.compact` | How the figure is written, as `format(value, .style)` writes it in the page's locale |
| `change:` | `Number` | How it moved, as a fraction (`0.124` is `▲ 12.4%`, green; a negative one `▼`, red), read "Up" or "Down" by a screen reader |
| `trend:` | `[Number]` | Its recent values, drawn as a 120×28 sparkline under it when the page is built (`E109` when not known then): `[31, 40, 28, 51]` |

**Flags:** `.number` `.integer` `.currency` `.percent` `.compact`

**Parts:**

- `Stat.Label(text)` — Its label.
  - `text:` `String` — The label
- `Stat.Value(text)` — Its figure.
  - `text:` `String or Number` — The figure
- `Stat.Change(text)` — How it moved, with its direction.
  - `text:` `String` — The change

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### CodeBlock

A block of code with a bar above it: a file name, a copy button (`.copy`), and line numbers (`.lines`). `language:` colours it, as on `Code`; `highlight:` marks lines; `files:` makes it a tab for each file.

```
CodeBlock(code, highlight: …, files: …, language: …)
```

```wf
CodeBlock("""
    page Home(path: "/", title: "Home") \{
        state count = 0
        Heading("Welcome").h1
        Button("+1").primary \{ on click \{ count = count + 1 \} \}
    \}
    """, language: "wf", title: "Home.wf", highlight: [2, 4]).copy.lines
```

Renders with the class `wf-code-block`.

| Prop | Type | Meaning |
|---|---|---|
| `code` (positional) | `String` | The code, shown as written in a block that scrolls sideways |
| `highlight:` | `[Number]` | The lines to mark, counted from 1: `[3, 4]`; each gets a tint and a bar in the primary colour |
| `files:` | `[CodeFile]` | A tab for each file, each its own code block: `[{ name: "App.wf", code: app, language: "wf" }, …]`; `.copy` and `.lines` apply to each |
| `language:` | `String` | Colours the code as this language: `wf` (or `wfx`), `json`, `bash` (`sh`, `shell`) or `css`; any other is shown plain |
| `title:` | `String` | The file name or the language, in the bar above the code; with `files:`, the tab list's accessible name |
| `copy:` | `Bool` | A button in the bar that copies the code and says `Copied` |
| `lines:` | `Bool` | Line numbers in a column beside the code, hidden from screen readers |

**Flags:** `.copy` `.lines`

**Parts:**

- `CodeBlock.Bar { … }` — The bar above the code.
- `CodeBlock.Lines(text)` — The line numbers.
  - `text:` `String` — The numbers, one a line
- `CodeBlock.Marks { … }` — The marks over the lines `highlight:` names. Drawn for it.
- `CodeBlock.Mark` — One line's mark. Drawn for it.

Attribute families: `global`, `aria`, `data`.

### Chip

A compact label — a filter, a tag, a person — that can carry an icon, be pressed (`selected:`, with `on click`), and be removed (`.removable`, with `on remove`). `Tag` is the one that does none of it.

```
Chip(label, icon: …, selected: …, removable: …)
```

```wf
state langs = ["Rust", "TypeScript", "Go", "Python"]
state openOnly = false
Stack(gap: .sm, align: .start) {
    Row(gap: .xs) {
        for l in langs by l {
            Chip(l, icon: "code").removable { on remove { langs = langs.filter(x => x != l) } }
        }
    }
    Chip("Open issues only", selected: openOnly) { on click { openOnly = !openOnly } }
    Text("{langs.length} languages, open only: {openOnly}").muted
}
```

Renders with the class `wf-chip`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | Its words |
| `icon:` | `String` | An icon's name, drawn before the words |
| `selected:` | `Bool` | Pressed (`aria-pressed`): a filter that is on, filled in the primary colour. Given at all, true or false, it makes the chip a toggle button |
| `removable:` | `Bool` | A button after the words, named `Remove {label}`, that fires `on remove` |

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
Row(gap: .md, align: .center) {
    Link(to: "/reference") {
        Text("Read more")
        VisuallyHidden("about the reference")
    }
    Row {
        Text("★★★★☆", aria-hidden: "true")
        VisuallyHidden("Rated 4 out of 5")
    }
}
```

Renders with the class `wf-visually-hidden`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The words a screen reader reads and the page does not show; unset, its block |

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Accordion

Sections a reader opens one by one, each a native `<details>`: `Accordion.Item("Shipping") { … }`. `.single` keeps one open at a time.

```
Accordion(single: …) { … }
```

```wf
Accordion.single {
    Accordion.Item("How long does shipping take?") {
        Text("Two to four working days in the EU, a week elsewhere.")
    }
    Accordion.Item("Can I return an order?", open: true) {
        Text("Within thirty days, unused and in its box.")
        Link("Start a return", to: "/learn")
    }
    Accordion.Item("Do you ship abroad?") {
        Text("To 40 countries; the checkout lists them.")
    }
}
```

Renders with the class `wf-accordion`.

| Prop | Type | Meaning |
|---|---|---|
| `single:` | `Bool` | One open at a time: opening one closes the others (every item given one `name`) |

**Flags:** `.single`

**Parts:**

- `Accordion.Item(title, open: …, name: …) { … }` — One section: its title is what a reader presses, its block what opens.
  - `title:` `String` — What a reader presses: the `<summary>`
  - `open:` `Bool` — Open when it is drawn; the reader opens and closes it after
  - `name:` `String` — Items of one name open one at a time, as `<details name>` does: what `.single` writes for you
  - flags: `.open`
- `Accordion.Summary(text) { … }` — What a reader presses to open an item: its title. Drawn for it.
  - `text:` `String` — The title
- `Accordion.Panel { … }` — What opens: the item's block. Drawn for it.

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

## PDF

### TableOfContents

The document's headings with the page each lands on, each a link to it.

```
TableOfContents(levels: …, title: …)
```

```wf
Document(title: "Employee handbook") {
    Heading("Employee handbook").h1
    TableOfContents(levels: 2, title: "Contents")
    PageBreak
    Heading("Your first week").h2
    Heading("Equipment").h3
    Heading("Time off").h2
}
```

Renders with the class `wf-toc`.

| Prop | Type | Meaning |
|---|---|---|
| `levels:` | `Number` | The deepest heading level listed: `1` is the `h1`s only, `2` adds the `h2`s |
| `title:` | `String` | A heading (`<h2>`) above the list |

Attribute families: `global`, `aria`, `data`.

### Document

Root of a PDF document: the paper, the margins and what the file says about itself.

```
Document(page_size: …, size: …, landscape: …) { … }
```

```wf
Document(size: "A4", margin: 56, title: "Invoice INV-2041", author: "Northwind Ltd",
         subject: "Invoice", keywords: "invoice, november", lang: "en") {
    Header { Text("Northwind Ltd · Invoice INV-2041").muted.sm.right }
    Footer { Text("Northwind Ltd · 12 Harbour Street · Bristol").muted.sm.center }
    Heading("Invoice INV-2041").h1
    Text("Issued 17 November 2026 · Due in 30 days").muted
    Table(caption: "Items") {
        Table.Head { Table.Row { Table.Cell("Item")  Table.Cell("Amount") } }
        Table.Body {
            Table.Row { Table.Cell("Design review")  Table.Cell("€1,200.00") }
            Table.Row { Table.Cell("Hosting, November")  Table.Cell("€90.00") }
        }
    }
    Text("Total: €1,290.00").bold.right
}
```

Renders with the class `wf-document`.

| Prop | Type | Meaning |
|---|---|---|
| `page_size:` | `.A4` `.A3` `.A5` `.Letter` `.Legal` | The paper, as a case (also written `size:`, which takes more); unset, the config's `page_size` under `build.pdf` |
| `size:` | `String` | The paper: `A3`–`A6`, `B5`, `Letter`, `Legal`, `Tabloid`, `Executive`, or a width and a height, `210mm 297mm` (a bare number is pt); unset, the config's `page_size` under `build.pdf` |
| `landscape:` | `Bool` | Turn the paper on its side, its longer edge across; `landscape: false` keeps it upright |
| `margin:` | `Number or String` | The page margins: one length, or two to four as CSS writes them (`72`, `20mm 25mm`); a bare number is pt; unset, the config's `build.pdf.margins` |
| `title:` | `String` | The file's title, shown by a reader in place of its name |
| `author:` | `String` | The file's author; several separated by commas |
| `subject:` | `String` | What the document is about: the file's description |
| `keywords:` | `String` | Comma-separated keywords |
| `lang:` | `String` | The document's language (`en`, `ar`): the file's language, and `ar`, `he`, `fa` or `ur` lays it out right to left |

**Flags:** `.A4` `.A3` `.A5` `.Letter` `.Legal` `.landscape`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Section

Groups content with spacing; on the web, a `<section>`.

```
Section { … }
```

```wf
Document(title: "Service agreement") {
    Section {
        Heading("1. Payment").h2
        Paragraph("Invoices are due within 30 days of issue.")
    }
    Section {
        style { break-inside: avoid }
        Heading("2. Signatures").h2
        Paragraph("Signed for Northwind Ltd: ______________________")
    }
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
Document(title: "Q3 report") {
    Heading("Summary").h1
    Paragraph("Revenue grew 15% on the quarter, led by the new self-serve plan.")
    Paragraph("Churn held at 2.1%; support response times halved.").muted
    Paragraph("Figures are unaudited.").sm.italic
}
```

Renders with the class `wf-text`.

| Prop | Type | Meaning |
|---|---|---|
| `content` (positional) | `String or Number or Bool` | The text, a `<p>`: a string, a number or a boolean |
| `tone:` | `.primary` `.secondary` `.success` `.danger` `.warning` `.info` | Its colour, a tone of the theme |
| `size:` | `.sm` `.md` `.lg` | Its font size |
| `align:` | `.left` `.center` `.right` | Where its lines sit |
| `transform:` | `.uppercase` `.lowercase` | The case its letters are drawn in |
| `kind:` | `.heading` `.subtitle` | A typographic role |
| `bold:` | `Bool` | Bold |
| `italic:` | `Bool` | Italic |
| `underline:` | `Bool` | Underlined |
| `muted:` | `Bool` | In `color-text-muted` |

**Flags:** `.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.left` `.center` `.right` `.uppercase` `.lowercase` `.heading` `.subtitle` `.bold` `.italic` `.underline` `.muted`

Attribute families: `global`, `aria`, `data`.

### Header

Repeated at the top of every PDF page (or those `on:` names), where `page` and `pages` are the page's number and the count; a `<header>` on the web.

```
Header(on: …) { … }
```

```wf
Document(title: "Annual report 2026") {
    Header(on: .rest) {
        Row(justify: .between) {
            Text("Northwind Ltd").bold.sm
            Text("Annual report 2026").muted.sm
        }
    }
    Heading("Annual report 2026").h1
    PageBreak
    Heading("The year").h2
}
```

Renders with the class `wf-header`.

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on, in the top margin |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Footer

Repeated at the bottom of every PDF page (or those `on:` names), where `page` and `pages` are the page's number and the count; a `<footer>` on the web.

```
Footer(on: …) { … }
```

```wf
Document(title: "Invoice INV-2041") {
    Footer {
        Row(justify: .between) {
            Text("Northwind Ltd · VAT GB123456789").muted.sm
            Text("Page {page} of {pages}").muted.sm
        }
    }
    Heading("Invoice INV-2041").h1
}
```

Renders with the class `wf-footer`.

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on, in the bottom margin |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Background

Drawn behind each PDF page's content, the size of the page: a cover, a band, a full-bleed picture.

```
Background(on: …) { … }
```

```wf
Document(title: "Q3 report") {
    Background(on: .first) {
        Stack { style { height: 100%; background: linear-gradient(160deg, #0B1220, #1E3A8A) } }
    }
    Stack {
        style { color: #ffffff; padding-top: 240px }
        Heading("Q3 report").h1
        Text("Northwind Ltd · October 2026")
    }
}
```

Renders with the class `wf-background`.

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on, behind the content, edge to edge |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Takes a block of children.

Attribute families: `global`, `aria`, `data`.

### Watermark

Large, faint, rotated text behind each PDF page's content.

```
Watermark(text, on: …)
```

```wf
Document(title: "Contract draft") {
    Watermark("DRAFT")
    Heading("Service agreement").h1
    Paragraph("This draft is for review and is not binding.")
}
```

Renders with the class `wf-watermark`.

| Prop | Type | Meaning |
|---|---|---|
| `text` (positional) | `String` | The text: 72pt, bold, 8% black, rotated behind the content (`.wf-watermark` restyles it) |
| `on:` | `.all` `.first` `.rest` `.odd` `.even` | Which pages it is drawn on |

**Flags:** `.all` `.first` `.rest` `.odd` `.even`

Attribute families: `global`, `aria`, `data`.

### PageBreak

Forces a new PDF page.

```
PageBreak
```

```wf
Document(title: "Report") {
    Heading("Summary").h1
    Paragraph("The quarter in one page.")
    PageBreak
    Heading("Appendix: the figures").h2
    Paragraph("Every number, month by month.")
}
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
    TitleSlide("Q3 review", subtitle: "Northwind Ltd · October 2026")
    SectionSlide("The numbers").primary
    Slide {
        Heading("Highlights").h1
        List {
            List.Item { Text("Revenue up 15%") }
            List.Item { Text("Three new enterprise customers") }
        }
    }
    TwoColumn {
        Container { Heading("Wins").h2  Text("Self-serve plan launched") }
        Container { Heading("Risks").h2  Text("Hiring behind plan") }
    }
    ImageSlide(src: "/og.png", caption: "The launch card")
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
        style { background: linear-gradient(135deg, #1E3A8A, #0F172A); color: #ffffff }
        Heading("Revenue").h1
        Text("Up 15% on last quarter, to €2.4M.").lg
        Text("Source: finance, unaudited").sm
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
Presentation {
    TitleSlide("Shipping WebFluent 7", subtitle: "Platform team · 17 November 2026")
}
```

Renders with the class `wf-title-slide`.

| Prop | Type | Meaning |
|---|---|---|
| `title` (positional) | `String` | The title, an `<h1>` centred on the slide |
| `subtitle:` | `String` | A line under the title |

Attribute families: `global`, `aria`, `data`.

### SectionSlide

Full-bleed coloured band with a centred label.

```
SectionSlide(label, tone: …)
```

```wf
Presentation {
    SectionSlide("Part one: the numbers").primary
}
```

Renders with the class `wf-section-slide`.

| Prop | Type | Meaning |
|---|---|---|
| `label` (positional) | `String` | The label, an `<h2>` centred on the band |
| `tone:` | `.primary` `.success` `.danger` `.warning` `.info` | The colour of the band, which fills the slide; the label is white (black on `warning`) |

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
        Container {
            Heading("Before").h2
            Text("A build took four minutes.")
        }
        Container {
            Heading("After").h2
            Text("It takes forty seconds.")
        }
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
Presentation {
    ImageSlide(src: "/og.png", caption: "The card every shared link shows")
}
```

Renders with the class `wf-image-slide`.

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The picture's path, centred on the slide; a slide with none draws an empty image |
| `caption:` | `String` | A line under the picture, which is also its `alt` |

Attribute families: `global`, `aria`, `data`.

## Routing

### Router

Where the current page renders. May sit at any depth inside `app`; pages declare their own paths.

```
Router(transition: …, duration: …) { … }
```

```wf
Stack {
    Navbar {
        Navbar.Brand { Link("Ledger", to: "/") }
        Navbar.Links {
            Link("Learn", to: "/learn")
            Link("Reference", to: "/reference")
        }
    }
    Router(transition: .fade, duration: "200ms")
    Text("© 2026 Ledger").muted.sm
}
```

Renders with the class `wf-router`.

| Prop | Type | Meaning |
|---|---|---|
| `transition:` | `.none` `.fade` `.slide` | How a route change moves between pages, through the browser's View Transitions where it has them; none for a reader who asks for less motion |
| `duration:` | `String` | How long each half of the transition plays, a CSS time, `200ms` |

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
| `drag:` | `Json` | What it carries when it is picked up — by a mouse moved a few pixels, or a finger held still — and dropped on something with `on receive(d)` |
| `dragGroup:` | `String` | Which drops it belongs to: a drag goes only to targets of its own group, so tabs and tree rows stay apart |
| `dropValue:` | `Json` | What this target is, as a drag sees it: `d.over` in the drag's `on dragmove(d)` and in `on receive(d)` |
| `dropAxis:` | `.y` `.x` | Which way its list runs, for where a drop lands |
| `dropInside:` | `Bool` | A drop on its middle lands inside it — a folder, a container — as well as before or after |
| `roving:` | `.x` `.y` `.both` | One Tab stop for the group, and the arrow keys, Home, End and the first letters of a name to move inside it: a toolbar, a row of choices, a list |
| `trapFocus:` | `Bool` | While true, focus moves into it and Tab and Shift+Tab stay inside; when it turns false or the element leaves, focus goes back where it was |
| `mount:` | `(Element) -> Handle` | Run with the element each time it is made — `mount: (n) => initTilt(n)` — and what it returns is the handle `update:` and `cleanup:` get |
| `update:` | `(Handle) -> ()` | Run with the handle again whenever the state it reads changes: `update: (h) => h.setMax(max)` |
| `cleanup:` | `(Handle) -> ()` | Run with the handle when the element leaves — a route change, an `if` branch, a list item: `cleanup: (h) => h.destroy()` |

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
