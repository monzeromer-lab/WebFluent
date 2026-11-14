# What the code looks like, stage by stage

These are sketches for the stages in [README.md](README.md). Each stage shows
two things:

- what an author writes in `.wf`;
- what changes inside the compiler and the runtime.

The internal sketches start from today's code: `ComponentSig` in
`src/registry/mod.rs`, the Tabs emitter in `src/web/codegen/js/widgets/nav.rs`,
and `WF.tabs` in `overlay.js`. They are shapes, not final code. Every name in
them is open to change, and "Open questions" at the end lists the choices that
are still undecided.

---

## Stage 0: bug fixes

Authors write nothing new; their pages simply start working. The fixes are
small.

**Switch: the input stays in the accessibility tree.** In
`themes/components.rs` and `structural.rs`:

```css
/* before */
.wf-switch input { display: none; }

/* after: invisible, but still focusable and still announced */
.wf-switch { position: relative; }
.wf-switch input { position: absolute; inset: 0; margin: 0; opacity: 0; cursor: pointer; }
.wf-switch input:focus-visible + .wf-switch__track { outline: 2px solid var(--color-primary); outline-offset: 2px; }
.wf-switch input:checked + .wf-switch__track .wf-switch__thumb { transform: translateX(20px); }
.wf-switch:dir(rtl) input:checked + .wf-switch__track .wf-switch__thumb { transform: translateX(-20px); }
```

**An async action keeps a count of the calls running.** In
`codegen/js/dom/flow.rs`:

```js
// before
_save_pending.set(true);  try { … } finally { _save_pending.set(false); }

// after: two clicks are two calls, and `pending` holds until both end
_save_pending.set(_save_pending() + 1);
try { … } finally { _save_pending.set(_save_pending() - 1); }
```

`save.pending` still reads as a Bool: `_save_pending() > 0`.

**`show` checks the condition again once its exit has played.** In
`show.js`:

```js
const plays = leave([...shown.children], animConfig);
const hide = () => { if (!read()) shown.style.display = "none"; };   // shown again meanwhile? keep it
if (plays.length) Promise.all(plays).then(hide); else hide();
```

**Arrow keys leave the browser's own shortcuts alone.** In `overlay.js` and
`layer.js`:

```js
nav.addEventListener("keydown", (e) => {
  if (e.altKey || e.ctrlKey || e.metaKey) return;   // Alt+ArrowLeft is "back"
  …
});
```

**A click on a Modal's backdrop closes it.** In `overlay.js`:

```js
el.setAttribute("closedby", "any");                  // browsers that know it do the rest
el.addEventListener("click", (e) => {                // the others: a click on the
  if (e.target !== el) return;                       // dialog box itself, outside its
  const r = el.getBoundingClientRect();              // content, is the backdrop
  const inside = e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom;
  if (!inside) el._wfShut();
});
```

**The off-screen Sidebar leaves the Tab order.** In `drawer.js`:

```js
const narrow = matchMedia("(max-width: 768px)");
effect(() => { panel.inert = narrow.matches && !open(); });
narrow.addEventListener("change", () => { panel.inert = narrow.matches && !open(); });
```

**Live regions exist before the first message.** In `toast.js` and
`announce.js`, both regions are made at boot, through the module's set-up hook
in `runtime/mod.rs`, instead of on first use.

**Each fix comes with a test that fails before it**, for example:

```wf
test "a switch can be reached and toggled from the keyboard" {
    state on = false
    Switch(bind: on, label: "Notifications")
    Text(if on { "on" } else { "off" })
    press "Tab"
    press " "
    expect "on"
}
```

---

## Stage 1: the registry as the single source

Authors notice nothing. Inside the compiler, each component carries
everything that today is spread over ten places.

The registry entry grows. In `registry/mod.rs`:

```rust
pub struct ComponentSig {
    pub name: &'static str,
    pub owner: Option<&'static str>,
    pub group: &'static str,
    pub summary: &'static str,
    // … props, events, parts, attrs, children as today …

    /// The element it draws, and its class. Was `project/builtin.rs`.
    pub tag: &'static str,
    pub class: &'static str,
    /// The CSS feature that pruning keeps for it. Was `themes/prune.rs`.
    pub css: &'static str,
    /// The runtime modules it needs. Was found by scanning the bundle.
    pub runtime: &'static [&'static str],
    /// What a PDF or a deck does with it. Was `pdf_validation.rs`.
    pub paper: Paper,
}

pub enum Paper { Draws, Refused, Static }   // Static: drawn as its first frame
```

An entry, in `registry/builtins.rs`:

```rust
ComponentSig {
    name: "Tabs",
    group: "Navigation",
    summary: "Tab strip switching between its pages.",
    tag: "div",
    class: "wf-tabs",
    css: "Tabs",
    runtime: &["focus"],
    paper: Paper::Refused,
    props: &[BIND_VALUE, ORIENTATION, ACTIVATION, LABEL],
    parts: &["Page"],
    ..BASE
}
```

**The lexer stops knowing component names.** It drops its 104 `TokenType`s
for built-ins. A capitalised word is an identifier, and the parser asks the
registry:

```rust
// before: TokenType::Carousel => "Carousel", repeated across 104 arms
// after
Token::Ident(name) if name.starts_with(char::is_uppercase) => match registry::component(name) {
    Some(sig) => ComponentRef::BuiltIn(sig.name),
    None => ComponentRef::User(name.into()),
},
```

**Each component's CSS is written once**, with structure and look apart. The
structural sheet becomes "the full sheet without the look" instead of a second
copy:

```rust
style!("Switch",
    structure: ".wf-switch { display: inline-flex; align-items: center; gap: var(--spacing-sm); position: relative; }",
    look:      ".wf-switch__track { background: var(--color-border); border-radius: 999px; }",
);
```

**New tokens**, from the baseline theme:

```css
:root {
  --z-dropdown: 1000; --z-sticky: 1100; --z-drawer: 1200;
  --z-modal: 1300;    --z-toast: 1400;  --z-tooltip: 1500;
  --target-min: 24px;
}
@media (pointer: coarse) { :root { --target-min: 44px; } }

.wf-btn, .wf-icon-btn, .wf-tabs__tab, .wf-menu__item, .wf-sidebar__item, .wf-carousel__dot {
  min-block-size: var(--target-min);
  min-inline-size: var(--target-min);
}
.wf-toast-container { z-index: var(--z-toast); }   /* was 9999 */
```

An author's theme can change them like any other token:

```wf
theme Brand {
    color-primary: #3B82F6
    target-min: 32px
}
```

---

## Stage 2: keyboard and focus

**Tabs can be bound.** The selected tab can live in the address, and a tab
can be disabled:

```wf
page Account(path: "/account", title: "Account") {
    state tab = "profile"
    Tabs(bind: tab, label: "Account") {
        Tabs.Page("Profile", value: "profile") { ProfileForm }
        Tabs.Page("Billing", value: "billing", disabled: !Session.isOwner) { Billing }
    }
}
```

Tabs can also be vertical, with a tab activated only on Enter:

```wf
Tabs(bind: section, orientation: .vertical, activation: .manual) { … }
```

**`roving:` gives an author's own component the same keyboard model:**

```wf
component Swatches(colors: [Color]) {
    state picked = 0
    Row(roving: .x, role: "radiogroup", aria-label: "Colour") {
        for c, i in colors by c {
            Button(role: "radio", aria-checked: i == picked, aria-label: c) {
                style { background: {c} }
                on click { picked = i }
            }
        }
    }
}
```

Its options are `roving: .x` (or `.y`, or `.grid(columns: 4)`), with
`wrap: false` and `typeahead: true`. The arrow keys swap by themselves in an
RTL page.

**Keys named by direction:**

```wf
on key("start") { Gallery.previous() }   // ArrowLeft in English, ArrowRight in Arabic
on key("end")   { Gallery.next() }
```

**A new runtime module,** `modules/focus.js`. It takes the place of the key
code in `WF.tabs`, `WF.menu`, the Carousel and the Splitter:

```js
function roving(box, opts) {
  const usable = (n) => !n.hidden && !n.inert && n.getAttribute("aria-disabled") !== "true";
  const items = () => [...box.querySelectorAll(opts.items || ":scope > *")].filter(usable);
  const rtl = () => getComputedStyle(box).direction === "rtl";
  let typed = "", typedAt = 0;

  const move = (list, i, focus = true) => {
    list.forEach((n, j) => n.tabIndex = j === i ? 0 : -1);
    if (focus) list[i].focus();
    if (opts.onMove) opts.onMove(i, list[i]);          // Tabs: select on move (automatic)
  };

  box.addEventListener("keydown", (e) => {
    if (e.altKey || e.ctrlKey || e.metaKey) return;
    const list = items();
    const at = list.indexOf(document.activeElement);
    if (at < 0) return;
    const across = opts.axis !== "y";
    const next = across ? (rtl() ? "ArrowLeft" : "ArrowRight") : "ArrowDown";
    const prev = across ? (rtl() ? "ArrowRight" : "ArrowLeft") : "ArrowUp";
    const last = list.length - 1;
    let to = null;
    if (e.key === next) to = at < last ? at + 1 : opts.wrap === false ? at : 0;
    else if (e.key === prev) to = at > 0 ? at - 1 : opts.wrap === false ? at : last;
    else if (e.key === "Home") to = 0;
    else if (e.key === "End") to = last;
    else if (opts.typeahead && e.key.length === 1) {
      typed = (performance.now() - typedAt < 500 ? typed : "") + e.key.toLowerCase();
      typedAt = performance.now();
      const hit = list.findIndex((n, j) => j !== at && n.textContent.trim().toLowerCase().startsWith(typed));
      if (hit >= 0) to = hit;
    }
    if (to === null) return;
    e.preventDefault();
    move(list, to);
  });

  // The active item, or the first one, is the single Tab stop.
  const sync = () => {
    const list = items();
    const i = Math.max(0, opts.active ? opts.active() : 0);
    if (list.length) move(list, Math.min(i, list.length - 1), false);
  };
  sync();
  return { sync, move };
}
```

**The Tabs emitter shrinks.** In `widgets/nav.rs`, the selected tab is the
bound state instead of a hidden `WF.signal(0)`:

```rust
let active = match bind_target(ui) {
    Some(t) => t.read.clone(),                        // Tabs(bind: tab)
    None => { let v = self.fresh_var(); self.emit_line(&format!("const {v} = WF.signal({first});")); format!("{v}()") }
};
// … buttons as today, keyed by `value:` …
self.emit_line(&format!(
    "WF.roving({nav}, {{ axis: \"{axis}\", items: \"[role=tab]\", active: () => {index_of_active}, onMove: {on_move} }});"
));
```

**`wf test` can check focus, and run a test right to left:**

```wf
test "tabs follow the arrow keys" {
    Tabs { Tabs.Page("One") { Text("first") }  Tabs.Page("Two") { Text("second") } }
    press "Tab"
    press "ArrowRight"
    expect focus "Two"
    expect "second"
}

test "and the other way in Arabic"(dir: "rtl") {
    Tabs { Tabs.Page("One") { Text("first") }  Tabs.Page("Two") { Text("second") } }
    press "Tab"
    press "ArrowLeft"
    expect focus "Two"
}
```

---

## Stage 3: overlays

**Modal options.** A modal can refuse to close on a backdrop click, and can
start in a particular field:

```wf
state editing = false
Modal(visible: editing, title: "Edit profile", dismiss: .escape, initialFocus: nameField) {
    Input(bind: name, label: "Name", ref: nameField)
    Modal.Footer { Button("Save").primary { on click { save() } } }
}
```

`dismiss:` takes `.any` (the default for Popover, and now for Modal),
`.escape` or `.none`.

**A bottom sheet that can be dragged:**

```wf
Sheet(visible: filters, side: .bottom, snap: [0.5, 1], title: "Filters") {
    RadioGroup(bind: sort, label: "Sort by") { … }
    Sheet.Footer { Button("Show {count} results").primary.full { on click { filters = false } } }
}
```

`side:` takes `.bottom`, `.start` or `.end`. Dragging down past 30% of the
height closes it, and dragging up snaps to the next point.

**A question that waits for its answer.** It is named `ask` because `confirm`
is the browser's own function:

```wf
Button("Delete project").danger {
    on click {
        if await ask("Delete “{project.name}”?", detail: "This cannot be undone.", yes: "Delete", danger: true) {
            Projects.remove(project.id)
        }
    }
}
```

**Toasts as calls**, with an action and a promise:

```wf
action archive(id: String) {
    Mail.archive(id)
    toast("Conversation archived", action: "Undo", onAction: () => Mail.restore(id))
}

action save() {
    toast.promise(Profile.save(), loading: "Saving…", done: "Saved", failed: e => e.message)
}
```

**What a screen reader hears**, when nothing on screen moves:

```wf
effect { announce("{results.length} results") }
announce("Connection lost", .assertive)
```

**One overlay path in the runtime.** `modules/overlay.js` serves Modal,
Dialog, Sheet and Popover:

```js
function overlay(el, { read, write, kind, dismiss = "any", initial, exit }) {
  let trigger = null;
  effect(() => {
    if (read() && !isOpen(el)) {
      trigger = document.activeElement;
      kind === "popover" ? el.showPopover() : el.showModal();      // the top layer, always
      stack(el, { dismiss, close: shut });                         // Escape: the topmost only
      (initial?.() || firstStop(el) || el).focus();
    } else if (!read() && isOpen(el)) shut();
  });
  async function shut() {
    if (exit) await exit(el);                                      // the exit plays before close
    kind === "popover" ? el.hidePopover() : el.close();
    unstack(el);
    if (trigger?.isConnected) trigger.focus();                     // focus goes back
    if (read()) write(false);
  }
  if (dismiss === "any") el.addEventListener("click", backdropClick(shut));
  if (dismiss === "none") el.addEventListener("cancel", (e) => e.preventDefault());
}
```

The anchor gets its ARIA by itself. In `widgets/overlay.rs`:

```rust
// Popover(anchor: btn): the button says what it opens, and whether it is open
self.emit_line(&format!(
    "WF.controls({anchor}, {pop}, () => {open});"   // aria-controls, aria-expanded, aria-haspopup
));
```

---

## Stage 4: interactions that hold up

**No double submits, with nothing to write.** A button whose handler calls an
async action is disabled and marked busy until the action ends:

```wf
Button("Save").primary { on click { Profile.save() } }
```

It compiles to:

```js
WF.el("button", {
  disabled: () => Profile.save_pending() > 0,
  "aria-busy": () => Profile.save_pending() > 0 ? "true" : null,
  "on:click": () => Profile.save(),
}, "Save");
```

The action drops a second call while one is running:

```js
async save() {
  if (_save_pending() > 0) return;
  _save_pending.set(_save_pending() + 1);
  try { … } finally { _save_pending.set(_save_pending() - 1); }
}
```

Polling and other actions that are meant to overlap say so:

```wf
action poll() concurrent { … }
```

**A spinner in the button, at the button's width:**

```wf
Button("Pay {format(total)}", loading: Checkout.pay.pending).primary.full
```

**Debounce and throttle** turn the recipe's effect, timer and second state
into one line:

```wf
state q = ""
Input(bind: typed, label: "Search") { on input(debounce: 300.ms) { q = typed } }

Stack {
    on scroll(e, throttle: 100.ms) { Reading.progress(e.target.scrollTop) }
}
```

**"Load more" when the end comes into view:**

```wf
for row in feed.items by row.id { Post(row) }
if feed.hasMore {
    Spinner { on visible { feed.loadMore() } }
}
```

---

## Stage 5: components that need no new foundation

```wf
Callout("Run `wf check` before you push.", title: "Tip").tip   // .note .tip .warning .danger

Text { Text("Press ")  Kbd("mod+k")  Text(" to search") }      // ⌘K on a Mac, Ctrl+K elsewhere

EmptyState(icon: "folder", title: "No projects yet", text: "Create one to get started.") {
    Button("New project").primary { on click { creating = true } }
}

Stat("Revenue", value: revenue, format: .currency, change: 0.124, count: "600ms")

CodeBlock(files: [CodeFile(name: "App.wf", code: APP), CodeFile(name: "styles.css", code: CSS)]).copy.lines

Accordion(single: true) {                                      // `single`: native <details name>
    Accordion.Item("Shipping") { Text("Two to four days.") }
    Accordion.Item("Returns", open: true) { Text("Thirty days.") }
}

Field(label: "Plan", hint: "You can change it later.", error: planError) {
    PlanPicker(bind: plan)                                     // the field links its label and its error to this
}

InputGroup {
    Icon("search")
    Input(bind: q, aria-label: "Search")
    InputGroup.Clear                                           // empties q, then puts focus back
}

Chip("Rust", icon: "code").removable { on remove { Tags.drop("Rust") } }

Row(gap: .sm) { ThemeToggle  LocaleSwitch }
OnThisPage(levels: 2)
CopyButton(value: shareUrl, label: "Copy link")
Link(to: url, target: "_blank") { Text("Docs")  VisuallyHidden("(opens in a new tab)") }
```

---

## Stage 6: components on the focus module

```wf
ToggleGroup(bind: view, label: "View") {
    ToggleGroup.Item("Grid", value: "grid", icon: "grid")
    ToggleGroup.Item("List", value: "list", icon: "list")
}
ToggleGroup(bind: styles, label: "Text style").multiple {      // bound to a list: several at once
    ToggleGroup.Item("Bold", value: "b", icon: "type")
    ToggleGroup.Item("Italic", value: "i")
}

RadioGroup(bind: plan, label: "Plan").cards {
    RadioGroup.Option("Free", value: "free", hint: "For trying it out")
    RadioGroup.Option("Team", value: "team", hint: "€8 a seat")
}

Pagination(bind: page, pages: totalPages, siblings: 1)        // follows `query.page` with bind: too

Stepper(bind: step) {
    Stepper.Step("Account") { AccountForm }
    Stepper.Step("Billing", done: billingDone) { BillingForm }
    Stepper.Step("Confirm") { Summary }
}

NumberInput(bind: qty, min: 1, max: 99, step: 1, label: "Quantity")

Calendar(bind: stay, mode: .range, min: today, disabled: d => d.weekday() >= 6)
```

---

## Stage 7: the large components

```wf
app {
    AppShell {
        AppShell.Header { Brand  Spacer  ThemeToggle }
        AppShell.Sidebar { Sidebar.Item(to: "/", icon: "home") { Text("Home") } }  // a Sheet on a phone
        Router
    }
}

// Options from a list: typed, filtered, chosen
Combobox(bind: assignees, options: Team.people, text: p => p.name, key: p => p.id,
         label: "Assignees").multiple

// Options from the server, asked for as the reader types
Combobox(bind: city, search: q => Geo.cities(q: q), debounce: 250.ms, label: "City") {
    option(c) { Text(c.name)  Text(c.country).muted }
    empty { Text("No city by that name") }
}

CommandPalette(open: palette) {
    on key("mod+k") { palette = !palette }
    CommandPalette.Group("Pages") {
        for p in Docs.pages by p.path {
            CommandPalette.Item(p.title, icon: "file") { on select { navigate(p.path) } }
        }
    }
    CommandPalette.Group("Actions") {
        CommandPalette.Item("Toggle theme", shortcut: "mod+shift+l") { on select { setTheme(if theme == "dark" { "light" } else { "dark" }) } }
    }
}

// Bound to a list, so several can be chosen: Space, Shift+Arrow, Ctrl/Cmd+A
Tree(items: files, children: f => f.children, key: f => f.path, bind: selected).multiple {
    row(f) { Icon(if f.children { "folder" } else { "file" })  Text(f.name) }
}

DataTable(rows: Deploys.items, key: d => d.id, bind: chosen, label: "Deploys") {
    DataTable.Column("Service", value: d => d.service).sortable
    DataTable.Column("Status") { cell(d) { Badge(d.status) } }
    DataTable.Column("When", value: d => d.at).sortable { cell(d) { RelativeTime(d.at) } }
    empty { EmptyState(icon: "list", title: "No deploys yet") }
}
```

The multi-select model is one runtime piece, `WF.selection`. Listbox, Tree,
DataTable rows and ToggleGroup `.multiple` all share it:

```js
function selection(box, { items, read, write, multiple }) {
  let anchor = 0;
  box.addEventListener("keydown", (e) => {
    const list = items(), at = list.indexOf(document.activeElement);
    if (e.key === " ") { e.preventDefault(); toggle(list[at]); anchor = at; }
    else if (multiple && (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") { e.preventDefault(); write(list.map(keyOf)); }
  });
  box.addEventListener("wf:move", (e) => {                 // from roving(): Shift+Arrow selects a range
    if (multiple && e.detail.shift) write(range(items(), anchor, e.detail.to).map(keyOf));
  });
  box.setAttribute("aria-multiselectable", multiple ? "true" : "false");
  effect(() => {
    const chosen = new Set(multiple ? read() : [read()]);
    for (const n of items()) n.setAttribute("aria-selected", chosen.has(keyOf(n)) ? "true" : "false");
  });
}
```

---

## Stages 8 and 9: examples

```wf
InputOTP(bind: code, length: 6, label: "Code from your app")
Rating(bind: stars, max: 5, label: "Your rating").half
Slider(bind: price, min: 0, max: 500, label: "Price")          // price: [Number] = [50, 200], two thumbs
DateRangePicker(bind: stay, label: "Dates", presets: [.last7, .thisMonth])

Menubar {
    Menubar.Menu("File") { Menu.Item(shortcut: "mod+n") { Text("New") } }
    Menubar.Menu("Edit") { Menu.Item(shortcut: "mod+z") { Text("Undo") } }
}
ContextMenu {
    FileRow(f)
    ContextMenu.Items { Menu.Item { Text("Rename") }  Menu.Item { Text("Delete") } }
}
HoverCard { Link("@ada", to: "/u/ada")  HoverCard.Content { UserCard(ada) } }
Tour(steps: [TourStep(target: newBtn, text: "Start here"), TourStep(target: search, text: "Find anything")], open: touring)
Timeline { for r in releases by r.version { Timeline.Item(r.version, at: r.date) { Markdown(r.notes) } } }
```

```bash
wf generate block sign-in       # writes src/pages/SignIn.wf, an ordinary page the project owns
```

---

## Open questions

1. **Handler options.** `on input(debounce: 300.ms)` mixes a named option
   into the list of names a handler binds. That is a grammar change, so
   tree-sitter and the formatter change too. The other way is a separate
   word, `on input debounce 300.ms { }`. I recommend the named option: it
   reads like every other call in the language.
2. **`ask(…)`, not `confirm(…)`.** `confirm` is the browser's global, and the
   language keeps the browser's names. `ask` returns `Bool`.
3. **Automatic busy buttons.** Disabling a button whose handler awaits is a
   change in behaviour for existing projects. It would be announced in the
   release notes, with `concurrent` on the action as the opt-out.
4. **`dir` is already the language's.** Under i18n it is the reader's
   direction. `roving:` reads the element's computed direction instead, so a
   `dir: "ltr"` code block inside an Arabic page still moves the right way.
5. **A project's own component must win over a new built-in.** Today a
   project component named like a built-in is never drawn: the built-in wins,
   and `V11` warns. Our own site has `Callout`, `Sheet`, `Rating`, `Chip`,
   `CodeBlock`, `EmptyState` and `Stat`-like components. Every new built-in
   would therefore silently replace components that projects already have. So
   stage 1 flips the rule, the way a declared `type` already wins over a
   built-in type:
   - the project's declaration wins;
   - `V11` becomes a note saying the built-in is shadowed.

   Adding a built-in then never changes an existing project.
