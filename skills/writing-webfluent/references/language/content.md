### Content

| Component | Usage |
|-----------|-------|
| `Callout` | `Callout("Run `wf check` first.", title: "Before you push").tip` — a note set apart, its text Markdown, `role="note"`: `.note` (default), `.tip`, `.important`, `.warning`, `.caution`, each with its icon and word (or `title:`); its block follows its text; `Callout.Title { … }` written by hand draws the title row instead |
| `Kbd` | `Kbd("mod+k")` — a `<kbd>`, written as this system writes it (`⌘K`, `Ctrl+K`; the static paint and a PDF write `Ctrl+K`) |
| `EmptyState` | `EmptyState(icon: "folder", title: "No projects yet", text: "Create one.") { Button("New project").primary }` — what a list shows with nothing in it; the block is its actions |
| `Stat` | `Stat("Revenue", value: total, format: .currency, change: 0.124, count: "600ms")` — a figure, its label, and which way it moved (`▲ 12.4%`, read "Up 12.4%"); `format:` is `.number`, `.integer`, `.currency`, `.percent`, `.compact`; `trend: [31, 40, 28]` draws its recent values as a sparkline under it |
| `CodeBlock` | `CodeBlock(code, language: "wf", title: "App.wf").copy.lines` — a bar with the file name and a copy button, line numbers beside the code; `highlight: [3, 4]` marks lines, and `files: [{ name: "App.wf", code: app, language: "wf" }, …]` is a tab a file |
| `Chip` | `Chip("Rust", icon: "code", selected: on).removable { on click { on = !on }  on remove { drop("Rust") } }` — `selected:` (or an `on click`) makes it a toggle button with `aria-pressed`; `.removable` adds a button named `Remove Rust` that fires `on remove`. `Tag` is the one that does neither |
| `Accordion` | `Accordion.single { Accordion.Item("Shipping") { … }  Accordion.Item("Returns", open: true) { … } }` — native `<details>`, its title the `<summary>`; `.single` keeps one open at a time (a shared `name`) |
| `VisuallyHidden` | `Link(to: url) { Text("Docs")  VisuallyHidden("(opens in a new tab)") }` — words for a screen reader only |

These are drawn as other elements — parts with a tag and a class of their
own, and built-ins that already exist — the same on the live page, in the
static paint, by the template engine and in a PDF.

