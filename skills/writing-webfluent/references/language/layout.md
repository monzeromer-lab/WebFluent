### Layout

| Component | Usage |
|-----------|-------|
| `AppShell` | `AppShell { AppShell.Header { … }  Sidebar { … }  Router  AppShell.Footer { … } }` — an app's frame: the header across the top (it sticks), a `Sidebar` down the start (its drawer on a narrow screen), the page beside it, the footer under them |
| `Container` | `Container { ... }` — centered max-width wrapper |
| `Host` | `Host(tag: "canvas", mount: (n) => …, update: (h) => …, cleanup: (h) => …)` — a node handed to somebody else's code, with a lifetime |
| `Element` | `Element("stripe-pricing-table", publishableKey: key) { on ready { … } }` — any custom element, by its tag; named arguments are attributes, handlers its events |
| `Portal` | `Portal(into: editorHost(row.id), data-popup: "row") { … }` — its block drawn into a node somebody else's code owns: an element a script returns, a `ref:` handle, or `null` to keep it aside; `into:` is read again when what it reads changes, and the block moves there with its nodes and state; it goes with the page, branch or item it is written in, and what is inside belongs there (handlers, `emit`, lifetimes). The box is `display: contents`; `id:`, `data-*` and `aria-*` are its own; the pre-rendered page has only a marker. Without `into:` it is a `C01` |
| `Row` | `Row(gap: .md, align: .center, justify: .between) { ... }` — horizontal flex. `gap`: xs sm md lg xl · `align`: start center end stretch baseline · `justify`: start center end between around evenly |
| `Column` | `Column(span: 6) { ... }` — 12-column grid child |
| `Grid` | `Grid(columns: 3, gap: .md) { ... }` — CSS grid; takes the same `gap`/`align`/`justify` as `Row` |
| `Stack` | `Stack(gap: .md) { ... }` — vertical flex; takes the same `gap`/`align`/`justify` as `Row` |
| `Spacer` | `Spacer` or `Spacer.sm` `Spacer.xl` — vertical space (the size is a flag) |
| `Divider` | `Divider()` — horizontal line |
| `VirtualList` | `VirtualList(rowHeight: 32, reveal: selected, label: "Files") { for f in files by f.path { … } }` — a scrolling box that draws only the rows in view of its one keyed `for`; without `rowHeight:` rows are measured (`estimate:` for the rest); `reveal:` scrolls the row with that key into view; give the box a height its rows do not set (`style { height: 400px }`, a `max-height:`), and one drawn before it is laid out paints its first frame with the rows in view and the overscan only, its `reveal:` row among them; anything else in its body is a `V12` and draws everything |
| `Splitter` | `Splitter(bind: side, min: 160, max: 480, reset: 240, label: "Sidebar width")` — a handle that sizes the region beside it: dragged between `min:` and `max:`, double-click or Enter back to `reset:`, arrows ±16 px (Shift ±64), Home/End; `.horizontal` between regions one above the other, `.reverse` on the start side of what it sizes (a pane on the right, a panel below) |
| `Masonry` | `Masonry(columns: 3) { for p in posts by p.id { Card { … } } }` — its children in columns, each under the last in its column at its own height; `columns:` takes one per breakpoint too |
| `AspectRatio` | `AspectRatio(ratio: 16 / 9) { Frame(src: "/demo/", title: "Demo") }` — a box that keeps its shape as its width changes, its child filling it; a literal ratio is a rule of the sheet, one that reads state follows it |
| `ScrollArea` | `ScrollArea(height: "300px", label: "Log") { … }` — a box that scrolls within `height:`, a thin scrollbar, its edges fading where there is more (`data-before`, `data-after`), a Tab stop named by `label:` so a keyboard can scroll it; `.horizontal` |
| `Center` | `Center { Spinner }` — its children in its middle, across and down |
| `Sticky` | `Sticky(offset: "64px") { Toolbar() }` — sticks to the top of its scrolling parent, `offset:` below it (`.bottom` to the bottom), and is marked `data-stuck` while it is |
| `PanelGroup` | `persist sizes: [Number] = [25, 75]` · `PanelGroup(bind: sizes, label: "Resize the sidebar") { PanelGroup.Panel(min: 15) { … }  PanelGroup.Panel { … } }` — panels side by side (`.vertical`, one above another), a handle (`role="separator"`) between each two: dragged, or the arrow keys on it (Shift for larger steps), Home and End; each held to its `min:` in percent; `bind:` holds the sizes in percent, and a `persist` keeps them across visits; `PanelGroup.Panel(collapsible: true)` folds away on Enter or a double press on the handle after it, and comes back the same way |
| `ClientOnly` | `ClientOnly { Text("It is {now:.time}") }` — its block drawn in the browser alone: the static paint, a template and a PDF leave it out; a `placeholder { … }` block is what they show instead |

