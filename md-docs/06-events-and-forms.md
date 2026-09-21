# 6. Events and forms

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

The DOM events every element accepts: `click`, `input`, `change`, `submit`,
`focus`, `blur`, `keydown`, `keyup`, `keypress`, `mouseenter`, `mouseleave`;
a component's own declared events (chapter 8) join them on its calls. A
handler's body is an imperative block, so it may assign state, call actions,
`await`, and `navigate`.

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
`Tab`, `ArrowDown` (or `down`), `space`. On an element the handler listens on
that element; at the top of a page or component it listens on the document
for as long as the page shows. Every modifier must match exactly, so
`ctrl+k` does not fire on `ctrl+shift+k`.

## Two-way binding

`bind: stateName` on a control keeps the state and the control in step, both
ways:

```wf
page Signup(path: "/") {
    state name = ""
    state email = ""
    state plan = "free"
    state agree = false
    state seats = 1
    state when = ""

    Input(bind: name, label: "Name", placeholder: "Ada Lovelace").required
    Input(bind: email, label: "Email").email.required
    Select(bind: plan, label: "Plan") {
        Select.Option("Free", value: "free")
        Select.Option("Team", value: "team")
    }
    Radio(bind: plan, value: "free", label: "Free")
    Radio(bind: plan, value: "team", label: "Team")
    Checkbox(bind: agree, label: "I agree to the terms")
    Switch(bind: agree, label: "Same thing, as a switch")
    Slider(bind: seats, min: 1, max: 50, label: "Seats")
    DatePicker(bind: when, label: "Start date")
    Text("{name} <{email}> wants {seats} {plan} seats from {when}")
}
```

The checker knows what each control holds: `Checkbox` and `Switch` bind a
`Bool`, `Slider` a `Number`, the rest a `String` (an `Input.number` holds a
`Number`). A control needs a `label:` (or a `placeholder:` for an `Input`, or
an `aria-label:`) — the accessibility lint says so otherwise.

## Forms

A `Form` groups controls; `on submit` runs when it is submitted, and the
page never navigates away. `Form(bind: name)` hands you a handle on the form:

```wf
page Contact(path: "/") {
    state name = ""
    state email = ""
    state message = ""
    state sent = false

    action send() {
        let r = await fetch("/api/contact", { method: "POST", body: { name: name, email: email, message: message } })
        sent = true
        form.reset()
    }

    Form(bind: form) {
        on submit { send() }
        Input(bind: name, name: "name", label: "Name").required
        Input(bind: email, name: "email", label: "Email").email.required
        Input(bind: message, name: "message", label: "Message").required
        Button("Send", type: .submit, disabled: !form.valid || send.pending).primary
        Text("Fields: {form.values.name} {form.values.email}").muted.sm
    }
    if sent { Alert("Thanks — we will be in touch.").success }
}
```

The handle gives:

| | |
|---|---|
| `form.valid` | `true` when every control passes its own checks (`required`, `type: email`, `min`/`max`, `pattern`) — kept current as the reader types |
| `form.values` | A map of the controls' values by their `name:` attribute |
| `form.reset()` | Clears the form |
| `form.submit()` | Submits it in code |
| `form.element` | The `<form>` itself |

A `Button` with `type: .submit` (or `.submit`) submits; the browser's own
constraint validation runs first, so an invalid form never reaches
`on submit`.

## Pending actions

An action that `await`s exposes `name.pending`, `true` while a call of it runs
— for a disabled button or a spinner:

```wf
store Api {
    state saved = 0
    action save(payload: Map) {
        let r = await fetch("/api/save", { method: "POST", body: payload })
        saved = saved + 1
    }
}

page Save(path: "/") {
    use Api
    state text = ""
    Input(bind: text, label: "Note")
    Button("Save", disabled: Api.save.pending).primary { on click { Api.save({ text: text }) } }
    if Api.save.pending { Spinner.sm }
    Text("Saved {Api.saved} times")
}
```

## Emitting events from components

A component declares the events it fires and fires them with `emit`; the
caller handles them like DOM events. [Chapter 8](08-components.md#events)
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

## Switching the theme

`setTheme("dark" | "light" | "system")` records the reader's choice, applies
it, and keeps it across visits; `theme` reads it. [Chapter 12](12-styling.md#dark-mode)
shows the theme declaration it switches to.

```wf
page ThemePicker(path: "/") {
    Row(gap: .sm) {
        Button("Light", outlined: theme != "light") { on click { setTheme("light") } }
        Button("Dark", outlined: theme != "dark") { on click { setTheme("dark") } }
        Button("System", outlined: theme != "system") { on click { setTheme("system") } }
    }
}
```

## Overlays: modals, dialogs, toasts

```wf
page Overlays(path: "/") {
    state confirm = false
    state deleted = false
    Button("Delete").danger { on click { confirm = true } }
    Modal(visible: confirm, title: "Delete this?") {
        Text("This cannot be undone.")
        Modal.Footer {
            Button("Cancel") { on click { confirm = false } }
            Button("Delete").danger { on click { confirm = false  deleted = true } }
        }
    }
    if deleted { Toast("Deleted").success }
}
```

A `Modal` or `Dialog` opens while its `visible:` state is true, traps focus,
closes on Escape, and returns focus where it was. A `Toast` shows the moment
it is rendered, so it lives under a condition that turns true.

## Next

[Control flow](07-control-flow.md).
