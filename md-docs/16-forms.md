# 16. Forms and validation

<!--
route: guide/forms
group: building
blurb: Controls bound to state, forms with a handle, validation rules that show their own messages, files, and what the server said.
description: Two-way binding on every control, Form handles, validate rules, file uploads, sending a form to an API, server errors and pending state.
-->

A form in WebFluent is controls bound to state, a `Form` that knows whether
they are valid, and `validate` blocks that say what each value must be. The
accessible plumbing — labels, `aria-describedby`, `aria-invalid`, announced
errors, focus on the first problem — is written for you.

## Controls and `bind:`

`bind: stateName` on a control keeps the state and the control in step, both
ways:

```wf
page Signup(path: "/") {
    state name = ""
    state email = ""
    state plan = "free"
    state agree = false
    state seats = 1
    state when: Date? = null

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

`Textarea(bind: note, label: "Notes", rows: 4, maxLength: 200)` is several
lines of text; with `maxLength` it shows how much is left, politely.

The checker knows what each control holds: `Checkbox` and `Switch` bind a
`Bool`, `Slider` a `Number`, `DatePicker` a [`Date?`](13-types.md#the-types-the-language-brings-with-it)
(nothing, until one is picked), the rest a `String` (an `Input.number` holds a `Number`). A control needs a
`label:` (or a `placeholder:` for an `Input`, or an `aria-label:`) — the
accessibility lint says so otherwise.

## Labels, hints and errors you write yourself

Every control takes `label:`, `hint:` and `error:`. With any of them the
control is wrapped in a field: the label is a real `<label for>`, the hint
and the error are linked with `aria-describedby`, and an error — a string,
empty when there is none — is announced as it appears and sets
`aria-invalid`.

```wf
page Handle(path: "/", title: "Handle", description: "Pick a handle.") {
    state handle = ""
    derived problem = if handle.length > 0 && handle.length < 3 { "At least three characters" } else { "" }
    Heading("Pick a handle").h1
    Input(bind: handle, label: "Handle", hint: "Letters, numbers and dashes", error: problem)
}
```

Most of the time a `validate` block (below) is simpler: it computes the
error for you. `error:` is for a message that comes from somewhere else.

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
| `form.valid` | `true` when every control passes its own checks and every `validate` rule holds — kept current as the reader types |
| `form.errors` | The message of every field that has one, by name |
| `form.touched` | Which fields the reader has left |
| `form.pending` | Whether an `async` rule is still asking |
| `form.values` | A map of the controls' values by their `name:` attribute |
| `form.reset()` | Clears the form, and everything it was showing |
| `form.submit()` | Submits it in code |
| `form.apply(errors)` | Shows what the server said, on the fields it named |
| `form.element` | The `<form>` itself |

A `Button` with `type: .submit` (or `.submit`) submits; the browser's own
constraint validation runs first, so an invalid form never reaches
`on submit`.

## What a value must be: `validate`

A `validate` block sits beside the state it guards. Every rule is one
line, and the control bound to that state shows what they say — there is
no `error:` to write and no `derived` to compute:

```wf
page Signup(path: "/join", title: "Join", description: "Make an account.") {
    state email = ""
    state password = ""
    state confirm = ""

    validate email {
        required
        email
    }
    validate password {
        required
        minLength(8) "Use at least 8 characters"
        pattern(/[0-9]/) "Include a number"
    }
    validate confirm {
        matches(password) "The two passwords differ"
    }

    Heading("Join").h1
    Form(bind: form) {
        on submit { log("signing up") }
        Input(bind: email, label: "Email").email
        Input(bind: password, label: "Password").password
        Input(bind: confirm, label: "Confirm").password
        Button("Join", type: .submit, disabled: !form.valid).primary
    }
}
```

### The rules

| Rule | Holds when |
|---|---|
| `required` | Something was given — a non-blank string, a non-empty list, a ticked checkbox |
| `email`, `url` | It looks like one |
| `minLength(n)`, `maxLength(n)` | The text is that long |
| `min(v)`, `max(v)` | The number, date or time is within it |
| `pattern(/…/)` | The text matches |
| `matches(other)` | It equals another value, which it follows |
| `oneOf([…])` | It is one of them |
| `custom "…" { expr }` | The expression is true |
| `async "…" { await … }` | The answer, once it arrives |

Each takes an optional message after it. Without one, the rule's own is
used — and a project with translations replaces it by naming
`form.required`, `form.email`, `form.minLength` and so on.

Every rule but `required` passes an empty value, so a blank optional field
shows one message rather than two. An `async` rule is only asked once the
rest have passed, and once per value — so a server is not asked about every
keystroke of an address that is not one yet.

### When a message shows

`Form(show:)` says: `.onBlur` (the default — after the reader leaves the
field), `.onSubmit`, or `.live` as they type. The fault is known either
way, so `form.valid` is the truth whatever the reader has seen.

A submit that fails shows every message, moves focus to the first field
that has one and announces it, and does not run `on submit`.

### A refined type validates itself

```wf
state nights: Number(1..=30) = 2
validate nights { required }
```

The range on the type is already a rule; only `required` needed saying.
See [Types](13-types.md#a-condition-on-a-type).

### What the server said

```wf
action signUp() {
    try {
        await Backend.signUp(email, password)
    } catch e {
        form.apply(e.body.errors)     // { email: "That address is taken" }
    }
}
```

Each message is shown on the field it names, and stands until the reader
changes that value.

## Files

`FileUpload` is a file input; the files arrive in its `change` event. Each is
a `File` — `.name`, `.size`, `.type`, and `.preview()`, an object URL for
showing an image before it is uploaded, revoked when the page leaves.

```wf
api Backend(base: "/api") {
    post avatar(id: String, file: File) at "users/:id/avatar"
}

page Avatar(path: "/avatar", title: "Your picture", description: "Change your picture.") {
    state picked: File? = null
    state done = false

    action upload() {
        if let f = picked {
            await Backend.avatar(id: "me", file: f)
            done = true
        }
    }

    Heading("Your picture").h1
    FileUpload(accept: "image/*", label: "Choose a picture") {
        on change(e) { picked = e.target.files[0] }
    }
    if let f = picked {
        Image(src: f.preview(), alt: "The picture you chose")
        Text("{f.name}, {format(f.size / 1024, .integer)} kB").muted.sm
        Button("Upload", disabled: upload.pending).primary { on click { upload() } }
        if upload.pending { Progress(value: Backend.avatar.progress * 100, max: 100) }
    }
    if done { Alert("Uploaded.").success }
}
```

An endpoint with a `File` parameter sends a multipart form, and
`Backend.avatar.progress` follows the upload from 0 to 1. `accept:` and the
size are conveniences for the reader, not checks: the server decides what
it accepts ([Security](23-security.md#uploads)). `FileUpload(…).multiple`
takes several.

## Sending it to a server

Describe the service once with `api`, call it from the submit handler, and
hand what a `422` said back to the form:

```wf
api Backend(base: "/api") {
    post signUp(body: Map) -> Map
        errors { 422 -> Map }
}

page Join(path: "/join", title: "Join", description: "Make an account.") {
    state email = ""
    state password = ""
    state joined = false

    validate email { required  email }
    validate password { required  minLength(8) }

    action join() {
        try {
            await Backend.signUp(body: { email: email, password: password })
            joined = true
        } catch e {
            form.apply(e.body.errors)
        }
    }

    Heading("Join").h1
    if joined { Alert("Welcome aboard.").success }
    Form(bind: form) {
        on submit { join() }
        Input(bind: email, name: "email", label: "Email").email
        Input(bind: password, name: "password", label: "Password").password
        Button("Join", type: .submit, disabled: !form.valid || join.pending).primary
    }
}
```

`form.apply({ email: "That address is taken" })` shows each message on the
field whose `name:` it matches, until the reader changes that value.
[Data and APIs](17-data.md) covers the service in full: retries, timeouts,
typed errors.

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

## A checklist for a good form

- Every control has a `label:` — a placeholder is not a label.
- Rules live in `validate` blocks, not in hand-written `derived` errors.
- The submit button is `type: .submit`, so Enter submits.
- Disable it while the request is out: `disabled: save.pending`.
- Say what happened afterwards, in an `Alert` or a `Toast`.
- The server checks everything again; the page's rules are for the reader.

## Next

[Data](17-data.md).
