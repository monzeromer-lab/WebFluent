### Actions

| Component | Usage |
|-----------|-------|
| `Button` | `Button("Label").primary.lg` — click handler: `Button("Save") { on click { doSave() } }`; `loading: Checkout.pay.pending` keeps it busy while a value holds (a spinner at its width, a press does nothing), as a click whose action is still running does by itself |
| `IconButton` | `IconButton(icon: "close", label: "Close")` or `IconButton(icon: "edit", label: "Edit").primary { on click { editItem() } }` — flags: `.sm`, `.lg`, `.primary`, `.danger` |
| `ButtonGroup` | `ButtonGroup { Button("A") Button("B") }` |
| `Dropdown` | Deprecated since 8.0 (`V13`): `Menu(trigger: "Actions") { Menu.Item { ... } }` draws and behaves the same |
| `Toggle` | `Toggle("Bold", bind: bold, icon: "type")` — a button that stays pressed while `bind:` holds (`aria-pressed`); a press flips it |
| `ToggleGroup` | `ToggleGroup(bind: view, label: "View") { ToggleGroup.Item("Grid", value: "grid", icon: "grid") … }` — one choice (a segmented control, a view switcher) or, `.multiple`, several (`bind:` a list); one Tab stop, the arrow keys inside, each `aria-pressed` |
| `CopyButton` | `CopyButton("Copy link", value: url, copied: "Link copied")` — copies `value:`, says `copied:` (unset, `Copied`) for a moment and announces it |
| `ThemeToggle` | `ThemeToggle` — light and dark over `setTheme`: a moon in a light page, a sun in a dark one (chosen by the stylesheet, so the static paint is right), named for the theme it switches to |
| `LocaleSwitch` | `LocaleSwitch(label: "Language")` — a button per `i18n.locales`, each in its own language (`English`, `العربية`), the current one pressed; in the static paint too, the page's own pressed |
| `Fab` | `Fab(icon: "plus", label: "New note") { on click { … } }`, or a speed dial: `Fab(icon: "plus", label: "Create") { Fab.Action("Note", icon: "file") { on click { … } } … }` — a round button at the bottom of the screen for the page's main action; with actions, it opens them (`aria-expanded`), the arrow keys go between them, Escape, a press outside or a choice closes them |
| `BackToTop` | `BackToTop(after: 600)` — a button that appears once the page has scrolled past `after:` px (unset, 400) and takes the reader, and the focus, back to the top |

Button tones: `.primary`, `.secondary`, `.success`, `.danger`, `.warning`, `.info`
Button flags: `.sm`, `.lg`, `.full`, `.rounded`, `.pill`, `.outlined`; `type: .submit` inside a form (any other button is `type="button"`, so it never submits one)

