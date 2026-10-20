# 9. Events

<!--
route: guide/events
group: basics
blurb: Handlers on elements, keyboard shortcuts, the events components fire, and the overlays a page opens.
description: DOM event handlers, the event object, keyboard shortcuts, component events, modals, dialogs, toasts and menus.
-->

## Handlers

A handler is written inside an element's block: `on <event> { … }`, or
`on <event>(name) { … }` when the body reads the event object.

```wf
page Clicks(path: "/") {
    state n = 0
    state last = ""
    Button("+1") { on click { n = n + 1 } }
    Input(bind: last, placeholder: "type") {
        on input { log(last) }
        on keydown(e) { if e.key == "Enter" { log("submitted {last}") } }
        on focus { log("in") }
        on blur { log("out") }
    }
    Text("{n}")
}
```

A handler may name any DOM event: the common ones are `click`, `input`,
`change`, `submit`, `focus`, `blur`, `keydown`, `keyup`, `mouseenter` and
`mouseleave`, and the rest — `dblclick`, `contextmenu`, `pointerdown`,
`pointermove`, `touchstart`, `wheel`, `scroll`, `paste`, `drop`,
`dragover`, `animationend`, `ended` and the others — work the same way. The
[components reference](37-components-reference.md#on-every-element) lists
every one. A component's own declared events (chapter 11) join them on its
calls. A handler's body is an imperative block, so it may assign state,
call actions, `await`, and `navigate`.

### The event object

Name a parameter to read the event: `on keydown(e) { … }`. Without one, the
event is still in scope as `event`. It is the browser's own event object, so
`e.key`, `e.target.value`, `e.clientX`, `e.preventDefault()` and
`e.stopPropagation()` are all there.

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

A handler on a component call — `UserCard("x") { on click { } }` — attaches
to the component's root element, so a styled button component is clickable
wherever it is used.

## Keys

`on key("…")` answers to one key, with modifiers:

```wf
page Keys(path: "/") {
    state open = false
    state q = ""
    Input(bind: q, ref: box, placeholder: "Search") {
        on key("Escape") { q = "" }            // on the element: fires while it has focus
        on key("Enter", e) { log(e.target.value) }
    }
    on key("ctrl+k") { box.focus() }           // on the page: fires anywhere on the page
    on key("cmd+k") { box.focus() }
    on key("?") { open = !open }
    if open { Alert("Shortcuts: ctrl+k to search, ? for this.").info }
}
```

Spelling: modifiers `ctrl`, `shift`, `alt`, `meta` (or `cmd`), joined with
`+`, then the key as the browser names it — `k`, `Enter`, `Escape` (or `esc`),
`Tab`, `ArrowDown` (or `down`), `space`, `F2`. On an element the handler
listens on that element; at the top of a page or component it listens on the
document for as long as the page shows. Every modifier must match exactly, so
`ctrl+k` does not fire on `ctrl+shift+k`.

### Shortcuts for an app

An app's shortcuts are written with **`mod`**, which is ⌘ on a Mac and Ctrl
everywhere else, and declared on `app`, where they are heard on every page
for as long as the app runs. A **chord** is its steps with a space between:
the second step must come within a second and a half of the first, and alone
it does not fire a shortcut of its own.

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

- **`keyIs(e, "mod+s")`** is whether a key event is that combination — a
  chord's too — so one table of commands can drive the shortcuts, the menus
  and a palette.
- **`keyLabel("mod+shift+p")`** is how this system writes it: `⇧⌘P` on a
  Mac, `Ctrl+Shift+P` elsewhere. A `Menu.Item`'s and a `Tooltip`'s
  `shortcut:` are written the same way, and the item tells assistive
  technology with `aria-keyshortcuts`.
- Outside any element, a page, a component or `app` may handle `keydown`
  and `keyup`, and the window's `resize`, `scroll`, `hashchange`,
  `popstate`, `online`, `offline`, `message`, `storage` and
  `visibilitychange`: each listens for as long as it shows.
- A letter is also the key in its place: on a Mac `⌥⌘P` sends `π`, and
  `mod+alt+p` still matches it.

## Emitting events from components

A component declares the events it fires and fires them with `emit`; the
caller handles them like DOM events. [Chapter 11](11-components.md#events)
covers it:

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

page Use(path: "/") {
    state total = 0
    Counter(10) { on change(v) { total = v } }
    Text("Now {total}")
}
```

## Overlays: modals, dialogs, toasts and menus

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

- **`Modal`** and **`Dialog`** are real `<dialog>` elements, opened while their
  `visible:` state is true. The browser traps focus, makes the rest of the
  page inert and closes on Escape or a click on the backdrop — and when it
  closes that way, the state is set back to `false`, so it never disagrees
  with what is on screen. Focus returns to what opened it. `title:` is the
  accessible name (`A08` without one). `Modal.Footer` is the row of actions.
- **`Toast`** shows the moment it is rendered and leaves after three seconds;
  it is announced politely to screen readers. Put it under a condition that
  turns true.
- **`Menu`** and **`Dropdown`** open a list from a button. Arrow keys move
  between items, Home and End jump to the ends, Enter and Space choose,
  Escape closes with focus back on the button, and a click outside closes it.
- **`Tooltip`** shows its text when its child is hovered or focused, and is
  linked to it with `aria-describedby`.

### Layers: popovers, menus and tooltips above everything

A `Popover`, a `Menu`'s list and a `Tooltip` are drawn in the browser's top
layer, above everything else on the page, so a box with `overflow: hidden`
or a stacking context of its own cannot clip or cover them. Each is placed
against its anchor: it flips to the other side when there is no room, slides
along its side to stay on screen, and follows the anchor while the page
scrolls. The open layers are one stack, newest on top: Escape closes the
top one only, and a press outside closes every layer it is outside of.

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
    }
    Text(said)
}
```

- **`Popover`** is open while its `visible:` state is true, placed against
  `anchor:` — an element's `ref:` — on `placement:` (`.bottom`,
  `.bottomStart`, `.bottomEnd`, `.top…`, `.left…`, `.right…`; unset,
  `.bottomStart`), `offset:` pixels away. When Escape or a press outside
  closes it, `visible:` is set back to `false` and `on close(reason)` hears
  why: `"escape"` or `"outside"`. `.arrow` draws a point toward the anchor;
  `label:` is its accessible name.
- **`Menu`** with `trigger:` is a button and its list. Without one it is a
  list that state opens: `visible:` at an `anchor:`, or `at:` a point
  `{ x, y }` — where the pointer asked for a context menu. Choosing an item
  closes it (`on close` hears `"choose"`).
- **`Menu.Item`** takes `shortcut:` — the keys that do the same, drawn at
  its end and given to assistive technology as `aria-keyshortcuts`;
  `checked:`, which makes it a check item that follows its state; and
  `disabled:`, which keeps it in sight but out of reach — the arrow keys
  skip it and a press does nothing.
- **`Menu.Sub(label:)`** is an entry with a menu of its own, opened beside
  it by a hover, ArrowRight or Enter. ArrowLeft and Escape go back to the
  entry; choosing in it closes the whole menu, and it closes with its menu.
- **`Tooltip`** waits `delay:` milliseconds of hover (unset, 500) and shows
  at once for keyboard focus; it never takes focus, and goes on leaving, on
  a press and on Escape. `shortcut:` adds a chip with the keys;
  `placement:` is where it goes (unset, `.top`).

## Listening outside the page's elements

A handler on an element hears that element. For the window or the document
— a resize, a scroll of the page, a message from another window — use an
`effect` with a `cleanup`, which removes the listener when the page leaves.
An action named without parentheses is the function itself, so it can be
handed to the browser:

```wf
page Scrolled(path: "/", title: "Scrolled", description: "How far down the reader is.") {
    state y = 0
    action track() { y = window.scrollY }
    effect {
        window.addEventListener("scroll", track)
        cleanup { window.removeEventListener("scroll", track) }
    }
    Heading("Scrolled").h1
    Text("{y}px down")
}
```

For the viewport's size, the network and the URL, the language already keeps
values current — `viewport`, `network`, `query`, `hash` — so no listener is
needed ([State](08-state-and-reactivity.md#the-browser-as-values)).

## Next

[Control flow](10-control-flow.md).
