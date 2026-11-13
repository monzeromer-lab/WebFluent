---
chapter: 8
kind: "tutorial"
route: "/learn/08-events"
from: ["/docs/guide/events"]
group: "language"
blurb: "Handlers on elements, keyboard shortcuts, the events components fire, the overlays a page opens, and drag and drop."
description: "DOM event handlers, the event object, keyboard shortcuts, component events, modals, menus, popovers, gestures and window events."
learned: ["Handle a DOM event with `on click { }`, and read the event with `on keydown(e) { }`", "Answer a key or a chord with `on key(\"mod+k\")`, on an element or for the whole app", "Fire your component's own events with `event` and `emit`", "Open modals, menus, popovers and tooltips from state, and know how they stack", "Drag, drop, long-press, swipe and listen to the window"]
---
# 8. Events

This chapter covers events in WebFluent. An event handler is the code that
runs when something happens: a click, a key, a drop, a component saying it
changed. You write it as `on <event> { … }`
inside the block of the element it listens to, and its body can change state,
call actions and navigate.

In this chapter:

- handlers and the event object
- keys, and shortcuts for a whole app
- the events your own components fire
- overlays: modals, toasts, menus, popovers and tooltips, and how they stack
- gestures: drag and drop, long-press, swipe and the splitter
- listening to the window and the document

## Handlers

A handler is written inside an element's block: `on <event> { … }`, or
`on <event>(name) { … }` when the body reads the event object. This page
counts clicks and logs what happens in a text field:

```wf
page Clicks(path: "/", title: "Clicks", description: "Handlers on a button and a field.") {
    state n = 0
    state last = ""
    Heading("Clicks").h1
    Button("+1") { on click { n = n + 1 } }
    Input(bind: last, placeholder: "Type here") {
        on input { log(last) }
        on keydown(e) { if e.key == "Enter" { log("submitted {last}") } }
        on focus { log("in") }
        on blur { log("out") }
    }
    Text("{n}")
}
```

A handler may name any DOM event. The common ones are `click`, `input`,
`change`, `submit`, `focus`, `blur`, `keydown`, `keyup`, `mouseenter` and
`mouseleave`. The rest — `dblclick`, `contextmenu`, `pointerdown`,
`pointermove`, `touchstart`, `wheel`, `scroll`, `paste`, `drop`, `dragover`,
`animationend`, `ended` and the others — work the same way. The
[components reference](../reference/components.md#on-every-element) lists
every one. On a call of your own component, the events it declares
([Components](10-components.md#events)) join them.

A handler's body is an imperative block, so it may assign state, call
actions, `await` and `navigate`.

> [!NOTE]
> **Coming from React or Svelte?** A handler is not a prop: there is no
> `onClick={…}` and no `on:click={…}`. Write `on click { … }` inside the
> element's block. `onclick:` as an argument is an inline handler, and the
> compiler refuses it (`E106`). See
> [Coming from React](../coming-from/react.md#the-map).

A handler on a component call — `UserCard("Sam") { on click { … } }` —
attaches to the component's root element, so a styled button component is
clickable wherever it is used.

### The event object

To read the event, name a parameter: `on keydown(e) { … }`. It is the
browser's own event object, so `e.key`, `e.target.value`, `e.clientX`,
`e.preventDefault()` and `e.stopPropagation()` are all there.

Without a parameter, the event is still in scope as `event` — unless the
program has an `event` of its own (a loop's item, a state, an action), which
wins. Name the parameter to read the event there.

A DOM event sends one value, the event, so a handler names one name at most:
`on click(e, x)` is an error (`T22`). Read what you want from the event,
`e.clientX`. Several names are for a component's own events, which send what
they declare ([below](#emitting-events-from-components)).

This drop zone uses the event object twice — to allow the drop, and to read
the dropped files:

```wf
page Drop(path: "/", title: "Drop", description: "A place to drop files.") {
    state names: [String] = []
    Heading("Drop files here").h1
    Card {
        on dragover(e) { e.preventDefault() }
        on drop(e) {
            e.preventDefault()
            names = Array.from(e.dataTransfer.files).map(f => f.name)
        }
        Text(if names.length == 0 { "Nothing yet" } else { names.join(", ") })
    }
}
```

## Keys

`on key("…")` answers to one key, with modifiers. On an element it fires while
that element has focus; at the top of a page or component it fires anywhere on
the page:

```wf
page Keys(path: "/", title: "Keys", description: "Keys on a field and on the page.") {
    state open = false
    state q = ""
    Heading("Keys").h1
    Input(bind: q, ref: box, placeholder: "Search") {
        on key("Escape") { q = "" }            // on the element: while it has focus
        on key("Enter", e) { log(e.target.value) }
    }
    on key("ctrl+k") { box.focus() }           // on the page: anywhere on the page
    on key("cmd+k") { box.focus() }
    on key("?") { open = !open }
    if open { Alert("Shortcuts: ctrl+k to search, ? for this.").info }
}
```

How a combination is spelled:

- Modifiers are `ctrl`, `shift`, `alt` and `meta` (or `cmd`), joined with `+`.
- Then comes the key as the browser names it: `k`, `Enter`, `Escape` (or
  `esc`), `Tab`, `ArrowDown` (or `down`), `space`, `F2`.
- `on key("Enter", e)` names the event too.

How it listens:

- On an element, the handler listens on that element.
- At the top of a page or component, it listens on the document for as long as
  the page shows.
- Every modifier must match exactly, so `ctrl+k` does not fire on
  `ctrl+shift+k`.

A combination no keyboard sends — an unknown modifier, no key — is an error
(`E117`).

### Shortcuts for an app

To give an app shortcuts that work on every page, write them with `mod` and
declare them on `app`:

- **`mod`** is ⌘ on a Mac and Ctrl everywhere else.
- On **`app`**, a shortcut is heard on every page for as long as the app runs.
- A **chord** is its steps with a space between: `mod+k mod+s`. The second
  step must come within a second and a half of the first, and on its own it
  does not fire a shortcut of its own.

This app drives its shortcuts and its **View** menu from one table of
commands:

```wf
store Commands {
    state said = ""
    state table: [Command] = [
        { id: "palette", keys: "mod+shift+p", title: "Commands" },
        { id: "sidebar", keys: "mod+b", title: "Toggle sidebar" },
    ]
    action run(e) {
        for c in table {
            if keyIs(e, c.keys) {
                e.preventDefault()
                said = c.id
            }
        }
    }
}

type Command { id: String, keys: String, title: String }

app {
    on key("mod+s") { Commands.said = "saved" }
    on key("mod+k mod+s") { Commands.said = "shortcut help" }
    on keydown(e) { Commands.run(e) }
    Menu(trigger: "View") {
        for c in Commands.table {
            Menu.Item(shortcut: c.keys) { Text(c.title) }
        }
    }
    Text("Commands: {keyLabel(\"mod+shift+p\")}")
    Router
}
```

The helpers and rules it relies on:

- **`keyIs(e, "mod+s")`** is whether a key event is that combination — a
  chord's too — so one table of commands can drive the shortcuts, the menus
  and a palette.
- **`keyLabel("mod+shift+p")`** is how this system writes it: `⇧⌘P` on a Mac,
  `Ctrl+Shift+P` elsewhere. A `Menu.Item`'s and a `Tooltip`'s `shortcut:` are
  written the same way, and the item — or the tooltip's control — tells
  assistive technology with `aria-keyshortcuts`.
- **Events outside any element.** A page, a component or `app` may handle
  `keydown` and `keyup`, and the window's `resize`, `scroll`, `hashchange`,
  `popstate`, `online`, `offline`, `message`, `storage` and
  `visibilitychange`, and the window losing and gaining focus (`blur`,
  `focus` — an editor saves on leaving, for example). Each listens for as
  long as its owner shows.
- **A letter is also the key in its place.** On a Mac `⌥⌘P` sends `π`, and
  `mod+alt+p` still matches it.

## Emitting events from components

A component declares the events it fires with `event`, and fires them with
`emit`. The caller handles them like DOM events, with a name for each value
the event sends — `on change(v)`, `on menu(key, x, y)`.
[Components](10-components.md#events) covers the details. This counter tells
its page every time it changes:

```wf
component Counter(_ start: Number) {
    event change(value: Number)
    state n = start
    Row(gap: .sm) {
        Button("−") { on click { n = n - 1  emit change(n) } }
        Text("{n}")
        Button("+") { on click { n = n + 1  emit change(n) } }
    }
}

page Tally(path: "/", title: "Tally", description: "A component that reports its count.") {
    state total = 0
    Heading("Tally").h1
    Counter(10) { on change(v) { total = v } }
    Text("Now {total}")
}
```

## Overlays: modals, dialogs, toasts and menus

An *overlay* is something a page opens over itself: a modal that asks a
question, a toast that confirms, a menu of actions, a tooltip. Most are
opened by a state, so the page decides when they show. This page has one of
each:

```wf
page Overlays(path: "/", title: "Overlays", description: "What a page opens.") {
    state confirm = false
    state deleted = false
    Heading("Overlays").h1
    Button("Delete").danger { on click { confirm = true } }
    Modal(visible: confirm, title: "Delete this?") {
        Text("This cannot be undone.")
        Modal.Footer {
            Button("Cancel") { on click { confirm = false } }
            Button("Delete").danger { on click { confirm = false  deleted = true } }
        }
    }
    if deleted { Toast("Deleted").success }
    Menu(trigger: "More") {
        Menu.Item { Text("Rename") }
        Menu.Divider
        Menu.Item { Text("Archive") }
    }
    Tooltip("Copies the link") { IconButton(icon: "copy", label: "Copy link") }
}
```

- **`Modal`** and **`Dialog`** are real `<dialog>` elements, open while their
  `visible:` state is true.
  - The browser traps focus, makes the rest of the page inert and closes the
    dialog on Escape. A click on the backdrop leaves it open.
  - When it closes that way, the state is set back to `false`, so it never
    disagrees with what is on screen. Focus returns to what opened it.
  - `on close` runs when it closes, however it was closed.
  - `title:` is its heading, which names it (`A08` without one).
    `Modal.Footer` is the row of actions.
  - `Modal.Title { … }` is what the heading draws when words alone will not
    do — `Modal.Title { Markdown("Rename `{name}`") }` — while `title:` stays
    its name.
  - `.closable` puts a close button in the heading row, which closes it as
    Escape does.
- **`Toast`** shows the moment it is rendered and leaves after three seconds.
  It is announced politely to screen readers. Put it under a condition that
  turns true.
- **`Menu`** and **`Dropdown`** open a list from a button. Arrow keys move
  between items, Home and End jump to the ends, Enter and Space choose, Escape
  closes with focus back on the button, and a click outside closes it.
- **`Tooltip`** shows its text when its child is hovered or focused, and is
  linked to it with `aria-describedby`.

### Layers: popovers, menus and tooltips above everything

A `Popover`, a `Menu`'s list and a `Tooltip` are drawn in the browser's *top
layer*, above everything else on the page. A box with `overflow: hidden`, or
one with a stacking context of its own, cannot clip or cover them.

Each layer is placed against its *anchor*, the element it belongs to:

- It flips to the other side when there is no room, and slides along its side
  to stay on screen.
- It follows the anchor while the page scrolls. When the anchor scrolls out of
  view the layer goes with it, still open, and comes back with it.

The open layers form one stack, newest on top. A press outside closes every
layer it is outside of.

Escape closes the newest layer that can be dismissed — a popover, a menu, an
open `Modal` or `Dialog` — and stops there: nothing behind it hears that key.
For example:

- A modal opened from a popover is above it. The first Escape closes the
  modal, which its `on close` hears, and the next closes the popover.
- A modal covers what is beneath it, so a press in it, or on its backdrop,
  leaves the popover under it open.
- A tooltip keeps Escape only while focus is on its control: it hides, and the
  key goes no further. Otherwise the same Escape hides it on its way past and
  goes on, to the layer below or to the element with focus. So a tooltip under
  a pointer resting on a toolbar button does not take the Escape that closes
  an editor's completion list.

This editor has a popover at a button, a menu bar with every kind of item, a
tooltip and a context menu at the pointer:

```wf
page Editor(path: "/", title: "Editor", description: "Layers at a control and at the pointer.") {
    state branches = false
    state wrap = false
    state asked = false
    state point = { x: 0, y: 0 }
    state said = ""
    Heading("Editor").h1
    Row {
        Button("main", ref: branch) { on click { branches = !branches } }
        Menu(trigger: "View") {
            Menu.Item(shortcut: "Ctrl+B") {
                on click { said = "sidebar" }
                Text("Toggle sidebar")
            }
            Menu.Item(checked: wrap) {
                on click { wrap = !wrap }
                Text("Word wrap")
            }
            Menu.Item(disabled: true) { Text("Zen mode") }
            Menu.Divider
            Menu.Sub(label: "Appearance") {
                Menu.Item { Text("Light") }
                Menu.Item { Text("Dark") }
            }
        }
        Tooltip("Save", shortcut: "Ctrl+S", delay: 400) { Button("Save") }
    }
    Popover(visible: branches, anchor: branch, placement: .bottomStart, label: "Branches").arrow {
        on close(reason) { said = "closed by {reason}" }
        Text("main")
        Text("feature/layers")
    }
    Card {
        on contextmenu(e) {
            e.preventDefault()
            point = { x: e.clientX, y: e.clientY }
            asked = true
        }
        Text("Right-click for the context menu")
    }
    Menu(visible: asked, at: point) {
        Menu.Item(shortcut: "F12") { Text("Go to definition") }
        Menu.Item { Text("Rename") }
        Menu.Divider
        Menu.Item.danger { Text("Delete") }
    }
    Text(said)
}
```

What each part does:

- **`Popover`** is open while its `visible:` state is true. It is placed
  against `anchor:` — an element's `ref:` — on `placement:` (`.bottom`,
  `.bottomStart`, `.bottomEnd`, `.top…`, `.left…`, `.right…`; unset,
  `.bottomStart`), `offset:` pixels away. When Escape or a press outside
  closes it, `visible:` is set back to `false` and `on close(reason)` hears
  why: `"escape"` or `"outside"`. `.arrow` draws a point toward the anchor;
  `label:` is its accessible name.
- **`Menu`** with `trigger:` is a button and its list. Without one it is a
  list that state opens — open from the first paint if its state starts
  `true` — at an `anchor:`, or `at:` a point `{ x, y }`, where the pointer
  asked for a context menu. `label:` names the list for a screen reader; it
  is never a button's text. Choosing an item closes the menu, and `on close`
  hears `"choose"`.
- **`Menu.Item`** takes:
  - `shortcut:` — the keys that do the same, drawn at its end and given to
    assistive technology as `aria-keyshortcuts`;
  - `checked:` — makes it a check item that follows its state;
  - `disabled:` — keeps it in sight but out of reach: the arrow keys skip it
    and a press does nothing;
  - `.danger` — for what cannot be taken back (Delete, Discard), drawn in the
    danger colour.
- **`Menu.Sub(label:)`** is an entry with a menu of its own, opened beside it
  by a hover, ArrowRight or Enter. ArrowLeft and Escape go back to the entry.
  Choosing in it closes the whole menu, and it closes with its menu.
- **`Tooltip`** waits `delay:` milliseconds of hover (unset, 500) and shows at
  once for keyboard focus. It never takes focus, and goes on leaving, on a
  press and on Escape. Once Escape has hidden it, it stays hidden until the
  pointer comes back to it or keyboard focus reaches its control.
  `shortcut:` adds a chip with the keys and gives the control it names its
  `aria-keyshortcuts`, in ARIA's spelling (`Control+S`). `placement:` is where
  it goes (unset, `.top`).

A layer takes what any element takes ([Elements](06-elements.md)), on the
element it draws: `id:`, `role:`, `tabindex:`, `title:`, `aria-*`, `data-*`,
`ref:`, and its `on` handlers. Where they land:

- A `Popover` is a `dialog` unless its `role:` says what else it is — a
  `listbox` of choices, for example.
- A `Menu` with a `trigger:` takes them on the box around its button and
  list. One that state opens takes them on its list, whose `role:` then
  replaces `menu`.
- A `Tooltip` takes them on the box around its control.
- A `Modal` or a `Dialog` takes them on its `<dialog>`, whose `title:` is its
  heading rather than an attribute.

A `Popover`'s or a `Menu`'s `on close` is the layer's own, and hears why it
closed.

## Gestures: drag, long-press, swipe and the splitter

The runtime turns what the pointer does over time into one event that says
what it means. A mouse, a pen and a finger run the same code. A finger picks a
drag up with a long press, so a plain swipe still scrolls the page.

This file browser reorders tabs, moves tree rows into a folder, and sizes its
sidebar with a splitter:

```wf
page Files(path: "/", title: "Files", description: "Reorder, move and resize.") {
    state side = 240
    state tabs = ["App.wf", "Home.wf"]
    state rows = ["src", "public", "README.md"]
    state landing = ""

    action moveTab(what: String, near: String, position: String) {
        let rest = tabs.filter((x) => x != what)
        let at = rest.indexOf(near)
        let i = if position == "after" { at + 1 } else { at }
        tabs = [...rest.slice(0, i), what, ...rest.slice(i)]
    }

    Heading("Files").h1
    Row {
        Stack {
            style { width: {side}px }
            for r in rows {
                Card(drag: r, dragGroup: "tree", dropValue: r, dropInside: r == "src") {
                    on receive(d) { log("{d.value} goes {d.position} {r}") }
                    on dragmove(d) { landing = if d.over == null { "" } else { "Goes {d.position} {d.over}" } }
                    on longpress(p) { log("menu for {r}") }
                    Text(r)
                }
            }
            Text(landing)
        }
        Splitter(bind: side, min: 160, max: 480, reset: 240, label: "Sidebar width")
        Row {
            for tab in tabs {
                Card(drag: tab, dragGroup: "tabs", dropValue: tab, dropAxis: .x) {
                    on receive(d) { moveTab(d.value, tab, d.position) }
                    Text(tab)
                }
            }
        }
    }
}
```

- **`drag: value`** picks an element up: a mouse moved a few pixels, or a
  finger held still for half a second. A ghost follows the pointer, a box the
  pointer passes near the edge of scrolls, and Escape puts the element back.
  The click a drag ends with is not a click.
- **`on receive(d)`** takes a drop. `d.value` is what was dropped, and
  `d.position` is where:
  - `"before"` or `"after"` along `dropAxis:` — `.y`, down the page, is the
    default; `.x`, across it, is for a tab strip;
  - `"inside"`, on the middle of a target with `dropInside: true` (a folder, a
    container).
  - The target shows a line where the drop would go, or an outline for
    inside.
- **`on dragmove(d)`** on the dragged element hears, as it moves, what it is
  over — the target's `dropValue:` as `d.over`, null over nothing — and where
  it would land. That is enough to say "Goes after Badge, inside Row".
- **`dragGroup:`** on a drag and its targets keeps groups apart, such as tabs
  and tree rows.
- **`on longpress(p)`** is half a second held still (`p.x`, `p.y`); the click
  it ends with is not a click. **`on swipe(s)`** is a finger or a pen swept
  across: `s.direction` is `"left"`, `"right"`, `"up"` or `"down"`.
- **`Splitter(bind:, min:, max:, reset:)`** sizes the region beside it:
  - It follows the pointer between its bounds; a double-click or Enter resets
    it; the arrow keys move it 16 px (64 with Shift).
  - `.horizontal` sits between regions one above the other; `.reverse` sits on
    the start side of what it sizes — a pane on the right, a panel at the
    bottom.
  - A drag or a key starts from the region's size as it is drawn. So the bound
    value may mean "the default share" — `0`, with the region's width a
    percentage until it is dragged — and `reset: 0` goes back to it: a reset
    is taken as written, not held to `min:`.
  - It is a `separator` in the tab order unless its `role:` or `tabindex:`
    says otherwise.

On a call of your own component, all of these — the drag, the drop, the
gestures — apply to the one element it draws, as a DOM event's handler or an
`aria-label` written on the call does:

```wf
component TreeRow(_ name: String) {
    Row { Icon("file")  Text(name) }
}

page Files(path: "/files", title: "Files", description: "Tree rows you can drag.") {
    state rows = ["src", "README.md"]
    Heading("Files").h1
    Stack {
        for r in rows {
            TreeRow(r, drag: r, dragGroup: "tree", dropValue: r, dropInside: r == "src") {
                on receive(d) { log("{d.value} goes {d.position} {r}") }
                on contextmenu(e) { log("menu for {r}") }
            }
        }
    }
}
```

A component that declares a prop or an event of that name takes it itself. A
component whose body is an `if`, a `for` or several elements has no one
element for them, and the build says so (`C07`): wrap what it draws in a
`Stack` or a `Row`.

> [!IMPORTANT]
> A drag has no keyboard of its own. Give what it does a menu item or a
> command too (`Move left`, `Move to…`), as the platform's accessibility rules
> ask of every pointer action.

## Listening outside the page's elements

A handler on an element hears that element. For the window, write the handler
at the top of the page, a component or `app`, outside any element — `resize`,
`scroll`, `hashchange`, `popstate`, `online`, `offline`, `message`, `storage`,
`visibilitychange`, `blur` and `focus` are heard there
([Shortcuts for an app](#shortcuts-for-an-app)). It listens for as long as its
owner shows:

```wf
page Scrolled(path: "/", title: "Scrolled", description: "How far down the reader is.") {
    state y = 0
    on scroll { y = window.scrollY }
    Heading("Scrolled").h1
    Text("{y}px down")
}
```

For any other target or event, use an `effect` with a `cleanup`, which removes
the listener when the page leaves. An action named without parentheses is the
function itself, so you can hand it to the browser:

```wf
page Selection(path: "/", title: "Selection", description: "What the reader has selected.") {
    state picked = ""
    action track() { picked = document.getSelection().toString() }
    effect {
        document.addEventListener("selectionchange", track)
        cleanup { document.removeEventListener("selectionchange", track) }
    }
    Heading("Selection").h1
    Text("Selected: {picked}")
}
```

For the viewport's size, the network and the URL, the language already keeps
values current — `viewport`, `network`, `query`, `hash` — so no listener is
needed ([State](07-state-and-reactivity.md#the-browser-as-values)).

## Summary

| You write | It does | Section |
|---|---|---|
| `on click { … }` | runs when the element fires the event | [Handlers](#handlers) |
| `on keydown(e) { … }` | names the event object | [The event object](#the-event-object) |
| `on key("Escape") { … }` | answers to one key or combination | [Keys](#keys) |
| `on key("mod+k mod+s") { … }` on `app` | an app-wide shortcut, here a chord | [Shortcuts for an app](#shortcuts-for-an-app) |
| `keyIs(e, "mod+s")`, `keyLabel("mod+s")` | test a key event; write a combination for this system | [Shortcuts for an app](#shortcuts-for-an-app) |
| `event change(v: Number)`, `emit change(n)` | a component's own event | [Emitting events from components](#emitting-events-from-components) |
| `Modal(visible: open, title: "…")` | a dialog opened by state | [Overlays](#overlays-modals-dialogs-toasts-and-menus) |
| `Popover(visible:, anchor:)`, `Menu(visible:, at:)` | layers placed at a control or a point | [Layers](#layers-popovers-menus-and-tooltips-above-everything) |
| `drag: v`, `on receive(d)`, `on dragmove(d)` | drag and drop | [Gestures](#gestures-drag-long-press-swipe-and-the-splitter) |
| `on longpress(p)`, `on swipe(s)` | a held press; a sweep | [Gestures](#gestures-drag-long-press-swipe-and-the-splitter) |
| `Splitter(bind: side, min:, max:)` | a handle that sizes a region | [Gestures](#gestures-drag-long-press-swipe-and-the-splitter) |
| `on scroll { … }` outside any element | a window event | [Listening outside the page's elements](#listening-outside-the-page-s-elements) |

The [components reference](../reference/components.md) lists every event and
prop of each built-in.

## Common mistakes

**Naming more values than a DOM event sends.** A DOM event sends one value,
the event, so a second name has nothing to hold:

```wf expect=T22
page Spot(path: "/", title: "Spot", description: "Where the pointer clicked.") {
    state at = ""
    Heading("Spot").h1
    Button("Click me") { on click(e, x) { at = "{x}" } }
    Text(at)
}
```

Name the event alone and read from it: `on click(e) { at = "{e.clientX}" }`
([T22](../errors/T22.md)).

**A misspelled event.** A handler for an event the element never fires never
runs, so the compiler warns:

```wf expect=V09
page Clicks(path: "/", title: "Clicks", description: "A misspelled event.") {
    state n = 0
    Heading("Clicks").h1
    Button("+1") { on clik { n = n + 1 } }
    Text("{n}")
}
```

Spell the event as the browser does, `on click`
([V09](../errors/V09.md)).

**A key combination no keyboard sends.** Modifiers have fixed names, and a
word that is not one stops the build:

```wf expect=E117
page Search(path: "/", title: "Search", description: "A shortcut no keyboard sends.") {
    state q = ""
    Heading("Search").h1
    Input(bind: q, ref: box, placeholder: "Search")
    on key("command+k") { box.focus() }
}
```

Write `cmd+k`, or `mod+k` for ⌘ on a Mac and Ctrl elsewhere
([E117](../errors/E117.md)).

## Next

[Control flow](09-control-flow.md).
