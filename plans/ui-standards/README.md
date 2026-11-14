# The built-ins against enterprise UI standards

Researched 2026-11-14. The standards are those in *Advanced UI Component
Standards: The Enterprise Guide*, eight sections drawn from MUI and the
Angular CDK:

1. native elements first;
2. roving tabindex, arrow keys that follow RTL, multi-selection;
3. focus trapping, initial focus, restoration;
4. light dismiss;
5. touch targets, bottom sheets, passive listeners;
6. a z-index scale and the top layer;
7. double submits, loading states, exit animations;
8. live regions and ARIA relationships.

Every claim below was read in the source. The bugs marked **(verified)** were
checked a second time by hand.

## Where the engine stands

What is already good, and better than most libraries:

- Modal and Dialog are a real `<dialog>` opened with `showModal()`.
- Popover, Menu, Dropdown and Tooltip use the Popover API (`popover="manual"`),
  so they are in the top layer.
- One layer stack decides who hears Escape (`layer.js:58`, `106-134`). A dialog
  joins that stack, and a tooltip keeps Escape only while its control has
  focus.
- Light dismiss works for layers: a press outside closes them from the top
  down (`layer.js:139-151`).
- Focus comes back to the trigger when a layer or menu closes.
- `trapFocus:` is a universal prop that wraps Tab both ways, handles nested
  traps and restores focus.
- Tabs have complete tab/panel ARIA and a roving tabindex, with Home and End.
- Menus have arrow keys, Home and End, wrapping, submenus, `menuitemcheckbox`,
  and the trigger's `aria-haspopup`, `aria-controls` and `aria-expanded`.
- Fields link their hint, error and count through `aria-describedby`, set
  `aria-invalid`, and move focus to the first invalid field on submit.
- Exits wait for their animations before removal (WAAPI `finished`, or
  `animationend` with a timeout), and reduced motion is honoured throughout.
- Select and Radio are native.
- The CSS mostly uses logical properties (`margin-inline-start`, and so on).

### Scorecard

| # | Standard | Verdict | The gap |
|---|---|---|---|
| 1 | Native first | Mostly met | Breadcrumb has no `ol`/`li` and does not set `aria-current` itself. ButtonGroup has no `role="group"`. Radios have no `radiogroup`/`fieldset`, so the group has no name. |
| 2 | Roving tabindex | Tabs only | No shared primitive. Carousel dots, Sidebar, List, Table, ButtonGroup and VirtualList have no arrow keys. There is no Toolbar. |
| 2 | RTL arrows | **Not met** | Nothing in the runtime reads text direction. Tabs, Menu submenus (fixed `right-start`) and Splitter (ArrowRight always grows) are fixed to physical left and right. |
| 2 | Multi-selection | **Not met** | No `aria-multiselectable` and no Space, Shift+Arrow or Ctrl+A anywhere. |
| 2 | Typeahead | Not met | Neither menus nor tabs jump to a letter. |
| 3 | Focus trap | Mostly met | The mobile Sidebar drawer has no trap. |
| 3 | Initial focus | Partly met | A Popover without `[autofocus]` calls `focus()` on a div that cannot take it, so focus stays on the trigger. The drawer never moves focus. There is no `initialFocus:`. |
| 3 | Restoration | Mostly met | Modal and Dialog leave it to the browser. The drawer restores focus only after Escape or a scrim click. |
| 4 | Light dismiss | Partly met | **Clicking a Modal's backdrop does nothing**, although `overlay.js:7` and `overlay.rs:149` say it closes it. The runtime has `dismiss: false`, but the language cannot pass it, so no modal can refuse to close. |
| 4 | Topmost Escape | Met | The drawer listens on the whole document, outside the stack. |
| 5 | 44×44 targets | **Not met** | The only floor is 24px, on some controls. Button is about 34px, `.sm` about 24px, IconButton 36 (28 at `.sm`), Splitter about 10px wide. There is no `pointer: coarse` rule. |
| 5 | Bottom sheet, swipe to dismiss | Not met | `on swipe` exists, but there is no sheet, no snap points and no drag-to-close. |
| 5 | Passive listeners | Mostly met | The recognisers use pointer events. One `touchmove` listener with `{ passive: false }` is added to every `drag:` element, even when nothing is being dragged (`gesture.js:322`). |
| 6 | z-index scale | **Not met** | Hard-coded values: 100, 199, 200, 201, 9999, 10000 and 2147483647. There is no `--z-*` token. |
| 6 | Top layer | Mostly met | Toast and the Sidebar drawer are `position: fixed`. **A toast shown while a modal is open is drawn under it, and the modal makes it inert.** |
| 7 | Double submit | **Not met** | Nothing disables a button while its action runs, and nothing stops the action running twice. `pending` is a boolean, so the first call to finish clears it while a second is still running. `form.pending` means "an async rule is still checking", not "the submit is running". |
| 7 | Debounce, throttle | Not met | Neither exists. The recipe needs an effect, a timer and a second state. |
| 7 | Loading states | Partly met | Spinner, Skeleton and Progress exist. Button has no busy state, no `aria-busy`, and resources set no ARIA. |
| 7 | Exit animations | Partly met | `if`, `for` and `show` play exits. Modal, Dialog, Popover and Menu vanish at once. Toast uses a fixed 300 ms timer rather than the animation's end. Leaving nodes are not made inert. |
| 8 | Live announcer | Partly met | `WF.announce` exists, but only politely, the language cannot call it, and it creates its region lazily, against its own comment. |
| 8 | describedby, controls | Mostly met | There is no `aria-errormessage`. Errors are announced twice (`role="alert"` and `announce`). A Popover's anchor gets no `aria-expanded`/`aria-controls`. A submenu has no `aria-controls`. A Modal without `title:` has no name. |

## Bugs to fix whatever is decided

In order of harm:

1. **Switch cannot be reached (verified).** `.wf-switch input { display: none; }`
   (`components.rs:326`, `structural.rs:302`) hides the very input that carries
   `role="switch"` and `aria-checked`. Keyboard and screen reader users cannot
   use any Switch. It should be visually hidden instead.
2. **The off-screen Sidebar is still in the Tab order.** On a narrow screen it
   is only `translateX(-100%)`, with no `inert` or `visibility: hidden`.
3. **Modal backdrop.** Make a backdrop click close it (`closedby="any"`, with a
   fallback), or correct the comments that say it does.
4. **Toasts under a modal.** They are invisible, and probably not announced.
5. **Lazy live regions (verified).** Toast and announce both create their
   region on first use, so the first message can be lost. Create them at boot.
6. **`show` hides content that came back (verified).**
   `show.js:43` hides after the exit with no check that the condition is still
   false.
7. **Async actions run twice and `pending` clears early (verified).**
   `flow.rs:93`, `132`.
8. **RTL.** The Carousel track moves the wrong way (`carousel.js:32`). The
   Switch thumb uses `translateX(20px)`. Toasts slide in from the physical
   right. Splitter, Tabs and submenu keys are fixed to physical directions.
9. **Arrow keys swallow Alt+Arrow.** Tabs and menus call `preventDefault`
   without checking modifiers, which breaks back and forward navigation.
10. **Smaller ones:**
    - The Splitter's hit area is about 10px.
    - A field error is announced twice.
    - A `Toast` element calls `WF.toast` each time it renders.
    - `overlay.js:124` has a stray doc comment, and `popup()` looks unused.

## Evidence from our own site

The docs site writes its own tablists, with `role: "tab"` on plain buttons, in
`CodeTabs`, `SearchPalette`, `OutputTabs`, `Example` and the recipe for the
active tab in the address. It also writes its own listbox of search results.
None of these has arrow keys or `aria-activedescendant`. The built-in Tabs
cannot be used for them because the selected tab is an internal signal: it
cannot be bound, cannot follow `hash`, and a tab cannot be disabled. The first
user of the language works around the same gaps the standards name.

## Language: what to add

These are proposals in the language's own style. Each one is used by the
built-ins first, so authors get the same machinery the engine uses.

- **`roving:`**, a universal prop. It is one runtime key manager, like the
  CDK's `ListKeyManager`:
  - it makes the active child `tabindex=0` and the rest `-1`;
  - it takes the axis (`.x`, `.y` or `.grid`), wrap, Home/End and typeahead;
  - it reads direction from the element, so ArrowLeft and ArrowRight swap in
    RTL;
  - it ignores keys with modifiers.

  Tabs, Menu, Carousel dots, a new Toolbar, Listbox, Tree and ToggleGroup are
  rebuilt on it, and an author's own component gets it with
  `Row(roving: .x) { … }`.
- **A selection model.** `bind:` to a list makes a widget multi-select:
  Space toggles, Shift+Arrow selects a range, Ctrl/Cmd+A selects all, and
  `aria-multiselectable` and `aria-selected` are set. Listbox, Tree and Table
  rows share it.
- **Logical key names.** `on key("start")` and `on key("end")` are
  ArrowLeft/ArrowRight according to direction. `dir` is readable as a browser
  value, like `theme`.
- **Overlay options**, on Modal, Dialog, Popover and the drawer:
  - `dismiss:` with `.any`, `.escape` or `.none` (maps to `closedby`);
  - `initialFocus:` taking a `ref`;
  - the anchor's `aria-expanded` and `aria-controls` written automatically.
- **Handler timing.** `on input(debounce: 300.ms) { … }` and
  `on scroll(throttle: 100.ms) { … }` use the existing `Duration` type, so the
  debounce recipe becomes one line.
- **Double submit by default.**
  - A `Button` or `Form` whose handler calls an async action is disabled and
    `aria-busy` while it runs; the compiler already knows the action is async.
  - `pending` becomes a count.
  - A second call while one is running is dropped, unless the action says
    `action save() concurrent { … }`.
  - `Button(loading: save.pending)` draws a spinner and keeps the button's
    width.
- **`announce(text)` and `announce(text, .assertive)`** as language
  functions, backed by regions made at boot.
- **Tokens.**
  - A z scale: `z-dropdown`, `z-sticky`, `z-drawer`, `z-modal`, `z-toast`,
    `z-tooltip`.
  - `target-min`: 24px by default and 44px under `pointer: coarse`, applied to
    every control. A theme can change it.
- **Checks.**
  - `wf verify` already runs Chrome, so it can measure every interactive box
    and flag anything under 24×24.
  - Lints:
    - `A18`: a Modal with no name;
    - `A19`: a `role: "tab"` or `role: "option"` written by hand, suggesting
      the built-in.

## Components: what to add

The catalogue is drawn from MUI (core, lab and X) and shadcn/ui, set against
the 60-odd built-ins we have.

The **Source** column says where each one comes from:

- **M**: MUI
- **S**: shadcn
- **W**: we have already written it by hand, in `site/` or `examples/`. That is
  the strongest evidence of demand.

The **Tier** column says when to build it:

- **1**: next, either because we already write it ourselves or because the
  standards need it;
- **2**: after that;
- **3**: when there is room.

### Inputs and forms

| Component | Source | Tier | What it is |
|---|---|---|---|
| `Combobox` | M S | 1 | A typed filter over options from a list or a `resource`, with `.multiple`, `.creatable`, groups, and loading and empty states (`aria-activedescendant`). It also covers MultiSelect and Autocomplete. |
| `Listbox` | S | 1 | Single or multiple choice on the selection model, when a native `<select>` is not enough (rich options). |
| `RadioGroup` | M S | 1 | A named group with a `legend` and arrow keys, plus a `.cards` look where each option is a card. |
| `CheckboxGroup` | M | 2 | `bind:` to a list, with "select all" in its mixed state. |
| `Toggle` | S | 1 | A button that stays pressed (`aria-pressed`), such as bold in an editor. |
| `ToggleGroup` / `SegmentedControl` | M S | 1 | Single or multiple choice with roving focus: view switchers, filters. Our hand-made tablists become this. |
| `InputGroup` | M S | 1 | An input with an icon, a prefix or suffix, a clear button or an action button: `InputGroup { Icon("search")  Input(bind: q) }`. |
| `NumberInput` | M | 1 | A `spinbutton` with − and + buttons, Up and Down, PageUp and PageDown, min, max and step, formatted with `format`. |
| `InputOTP` / `PinInput` | S | 2 | One box per digit, with paste across all of them and `autocomplete="one-time-code"`. |
| `TagInput` | M | 2 | Chips typed into a field, removed with Backspace. |
| `Rating` | M W | 2 | Stars as a radio group, with half steps and read-only. |
| `Slider`, two thumbs | M S | 2 | `Slider(bind: range)`, where the state holds `[lo, hi]`. |
| `Calendar` | M S | 1 | An inline month grid with grid keyboard, single, range or multiple, and disabled dates. It is the base for the pickers. |
| `DateRangePicker` | M | 2 | Two months and presets ("Last 7 days"). |
| `TimePicker` / `DateTimePicker` | M | 2 | Typed to the language's `Time` and `DateTime`. |
| `ColorPicker` | — | 3 | Swatches and a picker, bound to the language's `Color` type. |
| `FileUpload.dropzone` | M | 2 | A drop area with previews (`File.preview()`), progress from an `api` upload, and removal. |
| `TransferList` | M | 3 | Two lists with move buttons. |
| `Field` / `Fieldset` | S | 1 | A label, a description, the error and the control in one, for custom controls; `Fieldset` with a legend groups them. |
| `Textarea.autosize` | M | 2 | Grows with its text, using `field-sizing: content` with a fallback. |
| `Input.password.reveal` | — | 2 | Show and hide the password, announced to screen readers. |
| `Editable` | — | 3 | Text that turns into an input on click: Enter saves, Escape cancels. |

### Navigation

| Component | Source | Tier | What it is |
|---|---|---|---|
| `Pagination` | M S | 1 | `nav` with `aria-current` and ellipses; works with `paginate: .page` and `query`. |
| `Stepper` | M | 1 | Linear or free steps, each with a state (done, current or error), plus a `Wizard` form over it. |
| `CommandPalette` | S W | 1 | A Combobox inside a Dialog, opened with `mod+k`, with groups, shortcuts and recent items. It replaces the site's SearchPalette. |
| `Tree` | M W | 1 | Roving focus, typeahead, arrow keys that open and close, lazy children, `drag:` and multi-select. It replaces the site's FileTree. |
| `OnThisPage` / `ScrollSpy` | W | 1 | The page's headings, with the one in view marked, on the web as well as in PDF. |
| `PrevNext` | W | 2 | Links to the previous and next pages of an ordered list. |
| `NavigationMenu` | S | 2 | Navbar links that open panels (a mega menu) as a disclosure, with arrow keys. |
| `Menubar` | S | 2 | File, Edit and View menus as in a desktop app, built on the menu module. It fits the Tauri apps. |
| `ContextMenu` | S | 2 | A wrapper: a right-click or long press anywhere inside it opens its `Menu`, so `at:` no longer has to be written by hand. |
| `BottomNavigation` | M | 2 | A phone's tab bar, with targets of at least 44px and the safe-area inset. |
| `Fab` / `SpeedDial` | M | 3 | A floating main action that opens a few more. |
| `BackToTop` | M | 3 | Appears after scrolling down and moves focus to the top. |
| `ThemeToggle` | W | 1 | Light, dark and system, over `setTheme` and `theme`. |
| `LocaleSwitch` | W | 1 | One entry per `i18n.locales`, each written in its own language, over `setLocale`. |

### Overlays

| Component | Source | Tier | What it is |
|---|---|---|---|
| `Sheet` / `Drawer` | M S W | 1 | A `<dialog>` from the bottom, the start or the end. It has swipe to close, snap points and a handle, and the Sidebar's mobile drawer moves onto it. |
| `AlertDialog` and `confirm()` | S | 1 | A dialog that closes only by its buttons. `if await confirm("Delete?", danger: true) { … }` is the language function over it. |
| `HoverCard` | S | 2 | A preview card on hover and focus, using the layer and its delay. |
| `Popconfirm` | — | 3 | An AlertDialog drawn as a popover beside its button. |
| `LoadingOverlay` / `Backdrop` | M | 2 | Covers a region (not the page) while it is busy, with `aria-busy` on that region. |
| `Lightbox` | — | 2 | Images opened full screen, with arrow keys, swipe and zoom, over `Image` and the gallery. |
| `Tour` | W | 2 | Steps that point at `ref`s, with a cut-out spotlight, using the layer and focus. |
| `ConsentBanner` | W | 2 | A cookie choice with "decline" as the default. Choosing loads the `meta.scripts` entries marked `"consent": true`. |
| `Toast`, upgraded | M S | 1 | An action button, pause on hover and focus, close, position, a limit, the top layer, and `toast.promise(save(), loading:, done:, failed:)` (Sonner). |

### Showing data

| Component | Source | Tier | What it is |
|---|---|---|---|
| `Accordion` / `Collapsible` | M S | 1 | Native `<details>`, exclusive by `name`, opening with `.expand`. |
| `DataTable` | M S W | 1 | Columns declared once, then sorting (`aria-sort`), a filter, pagination, row selection, a sticky header, resizable columns, and an empty and a loading state, drawn by `VirtualList` when long. It replaces PropsTable and EventsTable. |
| `Callout` | W | 1 | Note, tip, warning and danger, each with an icon and a title. It replaces the site's Callout, Note and Aside. |
| `CodeBlock` | W | 1 | `Code.block` with a copy button, a file name, line numbers, highlighted lines and tabs. It replaces CodeBlock and CodeTabs. |
| `Stat` | W | 1 | A label, a value with `count:`, a change with its direction, and an optional sparkline. It replaces Kpi and KpiCard. |
| `EmptyState` | S W | 1 | An icon, a title, a sentence and an action. |
| `Kbd` | S | 1 | A key, written for this system by `keyLabel` (`⌘K` or `Ctrl+K`). |
| `Chip` (`Tag` upgraded) | M W | 1 | Icon, avatar, removable (with its own label), clickable and selectable. |
| `DescriptionList` | — | 2 | Key and value pairs (`dl`), in a column or a grid. |
| `Timeline` | M W | 2 | Events in order, with a dot, a time (`ago`) and content. It replaces VersionList. |
| `AvatarGroup` | M | 2 | Overlapping avatars with "+3". |
| `Item` | S | 2 | A row with media, title, description and actions, for lists, settings and search results. |
| `Gallery` / `Masonry` | M | 2 | A grid of images (`ImageList`), and a masonry layout. |
| `CircularProgress` / `Meter` | M | 2 | A ring, which can be indeterminate, and a native `<meter>` for a level against thresholds. |
| `Sparkline` | M | 2 | A tiny line or bar chart, inline, as SVG made at build time. |
| `RelativeTime` | — | 2 | `ago(when)` that keeps itself current (`now`), with the exact time on hover. |
| `Highlight` | — | 3 | Marks the matches of a search in text (`<mark>`). |

### Layout

| Component | Source | Tier | What it is |
|---|---|---|---|
| `AppShell` | W | 1 | A header, a sidebar and a main area. The sidebar becomes a Sheet on a narrow screen, with the skip link and the landmarks. It replaces DocsShell, HomeShell and ShowcaseShell. |
| `AspectRatio` | S | 2 | Keeps a box's ratio for media and embeds. |
| `ScrollArea` | S | 2 | A styled scrollbar that can be reached by keyboard, with fades at the edges. |
| `Center` | — | 2 | Centres its child in both directions. |
| `Sticky` | — | 2 | Sticks within its parent, and is marked once it is stuck. |
| `PanelGroup` | S | 2 | Several resizable panels over `Splitter`, with their sizes kept through `persist`. |

### Helpers that draw little or nothing

| Component | Tier | What it is |
|---|---|---|
| `VisuallyHidden` | 1 | Text for screen readers only, the `wf-visually-hidden` the runtime already has. |
| `CopyButton` | 1 | Copies a value, says "Copied" and announces it. |
| `InView` / `on visible` | 1 | An event when an element scrolls into view, for "load more" and lazy sections. Animation already does this inside for `on: .enterView`. |
| `ClientOnly` | 3 | Drawn only in the browser, never in the static paint (a clock, the reader's own data). |

### Blocks, which are not built-ins

shadcn's "blocks" are whole sections: a sign-in form, pricing, a dashboard,
settings pages. In WebFluent they belong as `wf generate block <name>`, which
writes ordinary `.wf` source into `src/` for the project to own and change.
They are not built-ins.

### What it adds up to

- **Tier 1:** 31 entries. A few are upgrades of ones we have: Toast, Chip
  (`Tag`) and CodeBlock (`Code.block`).
- **Tier 2:** 33.
- **Tier 3:** 8.

Our 60-odd built-ins would grow to about 130, close to MUI's set and beyond
shadcn's. Most of Tier 1 stands on two foundations from this plan: the
`focus` module (roving focus, typeahead, selection, direction) and the single
overlay path. The registry should be the one source before this many are
added.

## What has to change in how built-ins are built

Adding one built-in today touches about ten places:

- a lexer token (`token.rs` has 104 built-in `TokenType`s);
- the parser's name map;
- the registry;
- the tag and class map in `project/builtin.rs`;
- the dispatch in `element.rs`;
- a widget emitter;
- the CSS twice, in `components.rs` and again in `structural.rs` (the Switch
  bug sits in both);
- `prune.rs`;
- the static paint in `ssg.rs`;
- the PDF list (`pdf_validation.rs`) and the output linter.

Fifteen new components would mean 150 edits that have to agree with each
other. Before adding them:

1. **Make the registry the one source.** Each `ComponentSig` carries its tag,
   class, CSS feature, runtime modules, whether it is allowed in a PDF and how
   it paints statically.
   - The lexer reads a capitalised name as an identifier and the registry
     resolves it.
   - `builtin.rs`, `prune.rs` and the PDF list are generated from the table.
2. **Write each component's CSS once.** Mark the look apart from the
   structure, and generate the structural sheet from the full one.
3. **Add one `focus` runtime module**: roving focus, typeahead, selection and
   direction. Rebuild `WF.tabs`, `WF.menu` and the Carousel on it, and delete
   their copies. Pruning keeps the cost to projects that use it.
4. **Treat overlays alike.** One `overlay` path for Modal, Dialog, Sheet and
   Popover: top layer, `dismiss`, initial focus, restoration, and an exit
   played before `close()`/`hidePopover()` (or `@starting-style` with
   `allow-discrete`).
5. **Do the full work for each new widget.** It needs a static paint and a
   hydration takeover, a template-engine render, a PDF rule (E109 or a static
   drawing), a `wf docs` entry, and editor support through the registry JSON.
6. **Tests.**
   - Keyboard tests through `wf test` (`press "ArrowDown"` already exists).
   - Add `press "Shift+ArrowDown"` and `expect focus "…"`.
   - Run each widget in `dir="rtl"`.

## The plan, in stages

Each stage ends in a state that can be released: the full suite passes,
clippy and formatting are clean, the docs site builds and `wf verify` passes on
it, and the release notes say what changed. Nothing starts until the stage
before it is done. The one exception is stage 5, which can run beside stages 2
to 4 once stage 1 is done.

### What "done" means for every new or changed built-in

A component is done when it has all of the following:

- **Registry:** a registry entry: props, cases, events, slots and parts.
- **Live page:** an emitter and the runtime it needs, with that runtime
  pruned away for projects that don't use it.
- **Look:** CSS written once, its look apart from its structure, using
  tokens only.
- **Static and hydration:** a static paint, and the hydration takeover of
  that paint.
- **Template engine:** a render in the template engine.
- **PDF:** a PDF rule: either a static drawing or an `E109`.
- **Tests:**
  - snapshot tests;
  - a keyboard test through `wf test`;
  - the same test again with `dir="rtl"`;
  - axe-style checks through `wf verify`.
- **Docs and editors:**
  - its reference entry (generated from the registry);
  - its guide section and `wf docs`;
  - AGENTS.md and the copy under `skills/` (kept identical);
  - the editors, through the registry JSON.

### Stage 0: fix the bugs

No new features. The ten bugs listed above:

1. Switch can't be reached.
2. The off-screen Sidebar is in the Tab order.
3. The Modal backdrop does nothing.
4. Toasts are hidden under a modal.
5. Live regions are created lazily.
6. `show` hides content that came back.
7. Async actions run twice and `pending` clears early.
8. RTL transforms and keys.
9. Alt+Arrow is swallowed.
10. The small ones.

**Done when** each bug has a test that fails before its fix. This is a patch
release.

**Status: done, 2026-11-14.** The tests are `tests/js/standards.test.mjs`
(6 tests, all failing on the 6.1.0 runtime) and `tests/browser/standards.mjs`
(18 checks in Chrome, 14 failing on the 6.1.0 binary). The notes are under
v6.1.1 in `RELEASE_NOTES.md`. Store actions had the same `pending` bug as
page actions, and layers placed `.bottomStart` had the same RTL bug as
submenus; both are fixed.

Two items were left as they are:

- A `Toast` element shows each time it is drawn. That is what placing it
  means: it is drawn when its `if` turns true.
- `WF.popup()` stays. No built-in calls it, but a test does, and so may a
  project's own script.

### Stage 1: the registry as the one source

No new components, and no change to anything a project's build writes.

- **The registry is the one source.** Each `ComponentSig` carries its tag,
  class, CSS feature, runtime modules, PDF rule and static paint.
  - The lexer reads built-in names as identifiers.
  - `builtin.rs`, `prune.rs`, the PDF list and the output linter's table are
    derived from the registry.
- **The project's own component wins.** A project's component named like a
  built-in is drawn instead of the built-in, and `V11` becomes a note. Without
  this, every component added in stages 3 to 9 would replace a project's own
  component of the same name.
- **CSS once.** `components.rs` and `structural.rs` become one sheet with
  each rule marked as look or structure.
- **Tokens:**
  - the z scale: `z-dropdown`, `z-sticky`, `z-drawer`, `z-modal`, `z-toast`,
    `z-tooltip`;
  - `target-min`: 24px, and 44px under `pointer: coarse`.

  Every hard-coded z-index goes, and every control gets a minimum size.

**Done when:**
- the golden builds are byte-identical, except for the new tokens and sizes;
- adding a test component takes one registry entry, one emitter and one CSS
  block.

**Status: done, 2026-11-14.** What adding a built-in takes now:

1. a registry entry, with its tag and class (`.html(…)`), the stylesheet
   sections it draws on (`.css(…)`) and what paper does with it
   (`.on_paper()` or `.not_on_paper(…)`; a test fails on an entry that says
   neither);
2. its name in the parser's list, `ALL_COMPONENT_NAMES` in `syntax`. The
   layers keep the parser below the registry, so it cannot ask it; a test
   holds the two lists equal;
3. an emitter (the static paint and the template engine draw its tag and
   class without one);
4. its CSS, once, in `themes/sheet.rs`, under a section titled as its
   `.css(…)` says.

`builtin_to_html`, the stylesheet pruning and the PDF list read the
registry. The lexer's 104 component tokens were left alone: they belong to
the WebFluent 2 parser `wf migrate` uses, whose grammar is frozen; the
parser every build uses already read names as identifiers. The two sheets
are one file, `sheet.rs`, line by line: 841 lines of CSS in place of 1,270,
each shared line written once and each difference marked `/*@full*/` or
`/*@structural*/`. Both sheets come out byte for byte as they were; the
build output of every project is unchanged but for the new tokens.

### Stage 2: keyboard and focus

- **One `focus` runtime module:**
  - roving tabindex;
  - typeahead;
  - Home and End;
  - wrapping;
  - modifier keys left alone;
  - direction read from the element.
- **In the language:**
  - `roving:` as a universal prop;
  - `on key("start")` and `on key("end")`;
  - `dir` as a browser value.
- **Rebuilt on it:**
  - Tabs, which can now be bound (`bind:`) and have disabled tabs;
  - Menu, with typeahead and `menuitemradio`;
  - Carousel dots;
  - Splitter, which now follows RTL.

  Their copies of the keyboard code are deleted.
- **New `wf test` steps:** `press "Shift+ArrowDown"` and `expect focus "…"`.
  Each keyboard test runs again in RTL.
- **On the site:** CodeTabs, OutputTabs and the search kinds switch to the
  built-in Tabs.

**Done when** no built-in carries its own arrow-key code, and the site has no
hand-written `role: "tab"`.

**Status: done, 2026-11-14.** `WF.roving` (`modules/focus.js`) is the one
key manager; `WF.tabs`, menus and the carousel's dots are built on it, and
their own arrow-key code is gone. The Splitter keeps its own keys, which
size it rather than move between items, and follows the page's direction
since stage 0. Two things differ from the plan:

- The direction-following keys are `inline-start` and `inline-end`, not
  `start` and `end`: `on key("end")` already names the End key.
- The site's hand-made tab strips (`CodeTabs`, `OutputTabs`, `Example`, the
  search's kinds) keep their own markup and gain `roving: .x`. They share
  one panel between tabs, show tabs conditionally, or hold other controls
  in their bar, which built-in `Tabs` does not; the recipe for a tab in the
  address now uses `Tabs(value:)` and `on change`. A segmented control for
  the search's kinds is stage 6's `ToggleGroup`.

### Stage 3: overlays

- **One overlay path** for Modal, Dialog, Popover and the new Sheet:
  - top layer;
  - `dismiss:` (`.any`, `.escape`, `.none`);
  - `initialFocus:`;
  - focus restoration;
  - an exit played before closing;
  - `aria-expanded` and `aria-controls` on the anchor.
- **New and upgraded components:**
  - **Sheet**: bottom, start and end, with swipe to close, snap points and a
    handle;
  - **the Sidebar's mobile drawer**, which moves onto Sheet;
  - **AlertDialog**, with `await ask(…)`;
  - **Toast**, upgraded: the top layer, an action, pause, close, a limit, and
    `toast.promise`.
- `announce(text)` and `announce(text, .assertive)`.
- **Lint:** `A18`, a Modal with no name.

**Done when** standards 3, 4 and 6 hold for every overlay, checked by tests.

**Status: done, 2026-11-14.** `WF.dialog` serves Modal, Dialog,
AlertDialog and Sheet, and `WF.layer` serves Popover, with the same
`dismiss:` and `initialFocus:`; the checks are `tests/browser/overlays.mjs`
(30 in Chrome), `tests/js/overlays.test.mjs` and `tests/overlays.rs`. Four
things differ from the plan:

- The exit is the stylesheet's (`transition` with `allow-discrete`), not a
  script's: the browser keeps a closing dialog or popover in the top layer
  until it has played, so Escape, a press outside and `visible:` all play
  it with no code in their way. A browser without `allow-discrete` closes
  at once.
- The Sidebar's drawer did not move onto Sheet. Its panel is the same
  element a wide page shows in place, which a `<dialog>` cannot be; the
  drawer instead joins the layers (Escape in turn, a press outside), holds
  the focus with `trapFocus`, and gives it back to its toggle.
- The question is `await ask(…)`, not `await confirm(…)`: `confirm` is the
  browser's own function.
- `A08` already named a modal with no title, so it now covers AlertDialog
  and Sheet too, and `A18` is a Popover with no name.

`toast`, `ask` and `announce` are the language's only where a program has
no name of its own so called: `state toast` stays a program's, as it was.

### Stage 4: interactions that hold up

- `pending` becomes a count.
- **Double submits:**
  - a second call while one is running is dropped, unless the action says
    `concurrent`;
  - a Button or Form whose handler awaits is disabled and marked
    `aria-busy` while it runs.
- `Button(loading:)`.
- `on input(debounce: …)` and `on scroll(throttle: …)`.
- `on visible` and `InView`.
- Leaving nodes are made `inert`.
- `wf verify` measures interactive boxes and flags anything under 24×24.

**Done when** standard 7 holds, and the debounce recipe is one line.

**Status: done, 2026-11-14.** The checks are `tests/browser/busy.mjs` (21
in Chrome), `tests/js/busy.test.mjs` and `tests/interactions.rs`; the
debounce recipe is `on input(debounce: 300.ms) { q = typed }`. Five things
differ from the plan:

- `pending` stays a `Bool`. Stage 0 already kept a count of the calls under
  it, so it is true until the last ends; a number would have broken every
  `disabled: save.pending`.
- A second call is not dropped: one with the *same arguments* as a call
  still running is that call (the same promise, so an `await` of it gets
  the answer). Dropping every second call would have dropped the next row's
  toggle while the first was saving.
- A busy button is `aria-disabled`, not `disabled`: a disabled button
  loses the focus, which a keyboard user would then have to find again. A
  press while it is busy does nothing all the same.
- There is no `InView` component. `on visible` works on any element, so a
  wrapper added nothing.
- `wf verify` warns of a target under 24×24 rather than failing: WCAG lets
  a small target pass by the space around it, which a box's size cannot
  tell.

### Stage 5: Tier 1, the components that need no new foundation

This stage can run beside stages 2 to 4. It adds:

- Callout, Kbd, EmptyState, Stat;
- CodeBlock, an upgrade of `Code.block`;
- Chip, an upgrade of `Tag`;
- VisuallyHidden, CopyButton, ThemeToggle, LocaleSwitch;
- Accordion, Field / Fieldset, InputGroup, OnThisPage.

**Done when** the site's own Callout, Note, Aside, CodeBlock, Kpi, Chip,
EmptyState, ThemeToggle, LangSwitch and OnThisPage are replaced by the
built-ins.

**Status: built, 2026-11-14; the site's own are not replaced yet.** All
fourteen are in the registry and drawn alike on the live page, in the
static paint, by the template engine and in a PDF. They are not each an
emitter: `check::sema::expand` rewrites each into parts the registry gives
a tag and a class (`Callout.Title` is a `<p class="wf-callout__title">`)
and built-ins that already exist, before anything draws them, and what
they do is a runtime helper (`WF.copyButton`, `WF.themeToggle`, …) called
from a `mount:`. The checks are `tests/browser/content.mjs` (22 in
Chrome, with `wf test`) and `tests/content.rs`. What differs:

- `CodeBlock` has no tabs (`files:`) and no highlighted lines yet; `Stat`
  no sparkline.
- `Accordion` opens with the stylesheet's `::details-content` transition,
  not `.expand`.
- `LocaleSwitch` and `OnThisPage` are drawn once the page runs: the
  locales and the page's headings are the page's to read.
- The site keeps its own `Callout`, `CodeBlock`, `OnThisPage`,
  `ThemeToggle` and the rest. They carry the site's look, its analytics
  attributes, the Playground button and the copy-as-Markdown rail, and a
  project's own component wins over a built-in of its name. Moving the
  site onto the built-ins is its own piece of work.

### Stage 6: Tier 1, on the focus module

Toggle, ToggleGroup / SegmentedControl, RadioGroup, Pagination, Stepper /
Wizard, NumberInput and Calendar.

**Status: done, 2026-11-14.** Six are rewritten during lowering, as
stage 5's are, their `bind:` written back by handlers the rewrite adds;
`Calendar` has its own emitter and runtime (`WF.calendar`). The checks are
`tests/browser/choosing.mjs` (29 in Chrome, the arrow keys again right to
left) and `tests/choosing.rs`. What differs is under Leftovers.

### Stage 7: Tier 1, the larger components

1. AppShell, which replaces DocsShell and the other shells.
2. Combobox, Listbox and CommandPalette, which replaces the site's
   SearchPalette.
3. The selection model (`bind:` to a list makes a widget multi-select), then
   Tree, which replaces FileTree, and DataTable, which replaces PropsTable and
   EventsTable.

**Done when** the whole of Tier 1 is in, and the site and examples use the
built-ins wherever they used to build their own.

**Status: built, 2026-11-14; the site and examples are not moved over.**
`Listbox` and `CommandPalette` are rewritten during lowering as stage 5's
are; `Tree`, `DataTable` and `Combobox` have emitters that hand the runtime
their data and compile the blocks written for a row (`row(f)`, `cell(r)`,
`option(o)`, `empty`) into functions, and a built-in's entry now lists the
named blocks it takes (`.slots(…)`). `AppShell` is a frame of parts and a
grid. The checks are `tests/browser/collections.mjs` (36 in Chrome, a tree
again right to left) and `tests/collections.rs`. What differs is under
Leftovers.

### Stage 8: Tier 2, in four groups

Each group is released on its own:

1. **Forms:** InputOTP, TagInput, Rating, a two-thumb Slider, the date and
   time pickers, the FileUpload drop zone, a self-sizing Textarea, password
   reveal, CheckboxGroup.
2. **Navigation:** NavigationMenu, Menubar, ContextMenu, BottomNavigation,
   PrevNext.
3. **Overlays:** HoverCard, LoadingOverlay, Lightbox, Tour, ConsentBanner.
4. **Display and layout:** DescriptionList, Timeline, AvatarGroup, Item,
   Gallery / Masonry, CircularProgress / Meter, Sparkline, RelativeTime,
   AspectRatio, ScrollArea, Center, Sticky, PanelGroup.

### Stage 9: Tier 3 and blocks

- **Tier 3:** ColorPicker, TransferList, Editable, Fab / SpeedDial,
  BackToTop, Popconfirm, Highlight, ClientOnly.
- **`wf generate block <name>`:** a sign-in form, pricing, a dashboard and
  settings pages, written into `src/` as ordinary source.

## Leftovers

What each stage left undone, said at its end and kept here, to be taken
up together once stage 9 is done. A line comes off when its work lands.

### From stage 2

- Keyboard tests do not run a second time with `dir="rtl"`, as the
  definition of done asks; the right-to-left cases are separate checks.
- The site's hand-made tab strips (`CodeTabs`, `OutputTabs`, `Example`, the
  search's kinds) keep their own markup with `roving: .x` rather than
  built-in `Tabs`; the search's kinds want stage 6's `ToggleGroup`.

### From stage 3

- The exit of a modal, a dialog, a sheet or a popover is the stylesheet's
  (`allow-discrete`): a browser without it closes at once.
- The Sidebar's drawer is not a `Sheet` (its panel is the page's own on a
  wide screen); it joins the layers and holds the focus instead.

### From stage 4

- No `InView` component (`on visible` covers it).
- `wf verify` warns of a target under 24×24 and never fails on one; the
  site has 1,714 of them (the footer links at 21px, the cookie button at
  18px).
- `E901`, the read-back of the JavaScript the build writes, did not catch
  `--0.02` (a decrement of a literal), which a browser refuses.

### From stage 5

- `CodeBlock` has no tabs (`files:`) and no highlighted lines.
- `Stat` has no sparkline.
- `Accordion` opens with the stylesheet's `::details-content` transition,
  not `.expand`.
- `LocaleSwitch` and `OnThisPage` are drawn once the page runs, not in the
  static paint.
- The guide's example component `Chip` became `Pill`; two examples
  (`medium-field-notes`'s `ThemeToggle`, `hard-pocket`'s `EmptyState`)
  keep components named like a built-in, which draws a `V11` note.
- The site keeps its own `Callout`, `Aside`, `CodeBlock`, `CodeTabs`,
  `KpiCard`, `ThemeToggle`, `LangSwitch` and `OnThisPage` rather than the
  built-ins, so stage 5's "done when" is not met.

### From stage 6

- There is no `SegmentedControl` or `Wizard` by those names:
  `ToggleGroup` is the segmented control, and `Stepper` has no form of its
  own over it.
- A `Stepper.Step` must be a direct child of its `Stepper`, not drawn by
  a `for` or an `if`.
- `NumberInput` does not write its value through `format`.
- `Calendar` is drawn once the page runs, not in the static paint, and
  has no month or year picker beyond the previous and next buttons.
- A live `Radio` has no `value` attribute (its binding holds the value),
  where its static paint has one: a form posted without script sends `on`.
- The site's search kinds are still hand-made buttons, not `ToggleGroup`.

### From stage 7

- The site and the examples keep their own shells, search palette, file
  tree and props tables (`DocsShell`, `SearchPalette`, `FileTree`,
  `PropsTable`, `EventsTable`): stage 7's "done when" is not met.
- There is no shared `WF.selection`: `Listbox`, `Tree` and `DataTable` each
  choose in their own way (a listbox by its options' own presses).
- `DataTable` has no filter, no pagination, no resizable columns, and is
  not drawn by `VirtualList` when long.
- `Tree`, `DataTable` and `Combobox` are drawn once the page runs, not in
  the static paint; a `row`, `cell` or `option` block's parameter is `Any`
  to the checker.
- `Combobox`'s offers are not drawn by `VirtualList`; its `search:` has no
  `.pending` of its own beyond `aria-busy`.
- `AppShell.Sidebar` is not a part: the frame takes the project's own
  `Sidebar`, which is a drawer on a narrow screen rather than a `Sheet`.
