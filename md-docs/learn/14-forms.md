---
chapter: 14
kind: "tutorial"
route: "/learn/14-forms"
from: ["/docs/guide/forms"]
group: "build"
blurb: "Bind controls to state, validate what the reader types, upload files, and show what the server said."
description: "Bind every control to state, use the Form handle, write validate rules, upload files, send a form to an API, and show server errors and pending state."
learned: ["Bind any control to a state, a store member or a list item with `bind:`", "Write `validate` rules and let the bound control show their messages", "Read the whole form through `Form(bind: form)`: `form.valid`, `form.errors`, `form.apply`", "Upload a file and follow its progress", "Send a form to an `api` and show what the server answered"]
---
# 14. Forms and validation

This chapter is about forms in WebFluent: controls that read and write
state, rules that say what each value must be, and a submit that sends the
form to a server and shows what came back.

You will be able to:

- bind any control to a state, in both directions;
- validate a field with a `validate` block, without writing the error
  messages' markup;
- know whether the whole form is valid, and submit it;
- upload a file and show its progress;
- show the errors a server sends back, and a busy state while a request runs.

WebFluent writes the accessible parts for you: a real `<label>` for every
label, `aria-describedby` for hints and errors, `aria-invalid`, errors
announced as they appear, and focus moved to the first problem.

## Before you start

You should know how to declare `state` and an `action`
([State and reactivity](07-state-and-reactivity.md)) and how to handle an
event ([Events](08-events.md)). Sending a form to a server uses an `api`,
which [Data and APIs](15-data.md) covers in full.

## Bind a control to state

To keep a control and a state in step, give the control `bind:` and the
state's name. Typing in the control writes the state, and changing the
state updates the control.

This sign-up page binds one of each kind of control:

```wf
page Signup(path: "/signup", title: "Sign up", description: "Every control, bound to state.") {
    state name = ""
    state email = ""
    state plan = "free"
    state agree = false
    state seats = 1
    state when: Date? = null

    Heading("Sign up").h1
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
    Text("{name} <{email}> wants {seats} {plan} seats")
    if let start = when { Text("Starting {start}") }
}
```

> [!NOTE]
> **Coming from React?** A controlled input needs `value={q}` and an
> `onChange` that calls the setter. `bind: q` is both directions at once, and
> the control holds the state's type, so a `Slider` writes a `Number`, not
> the string `e.target.value` would give you. See
> [Coming from React](../coming-from/react.md).

The last lines repeat what you type and pick. The `Select` and the two
`Radio`s share `plan`, so choosing in one moves the others.

For several lines of text, use a `Textarea`. With `maxLength:` it shows how
many characters are left, and tells a screen reader politely (without
interrupting):

```wf
Textarea(bind: note, label: "Notes", rows: 4, maxLength: 200)
```

### What each control holds

The checker knows the type each control writes, and refuses a `bind:` to a
state of another type (`F02`):

| Control | Holds |
|---|---|
| `Checkbox`, `Switch` | `Bool` |
| `Slider` | `Number` |
| `Input.number` | `Number`, and `null` when the reader empties it |
| `DatePicker` | [`Date?`](12-types.md#the-types-the-language-brings-with-it) — `null` until a date is picked |
| `Select` | the option's `value:`, as written |
| `Input`, `Textarea`, `Radio` | `String` |

A value comes back as the type it went in as. `Select.Option("Two",
value: 2)` binds the number `2`, not the string `"2"`.

Every control needs an accessible name: a `label:`, a `placeholder:` (on an
`Input`), or an `aria-label:`. The accessibility lints (`A03`, `A04`) warn
when one has none.

### Bind to a store or a list item

`bind:` takes anything that can be written:

- a state;
- a store's member, such as `bind: Cart.note`;
- a field of a loop's item.

This list lets the reader rename and tick off each todo in place:

```wf
type Todo { id: String, title: String, done: Bool = false }

page Todos(path: "/todos", title: "Todos", description: "Edit each todo in place.") {
    state todos: [Todo] = [Todo(id: "1", title: "Water the plants"), Todo(id: "2", title: "Post the letter")]

    Heading("Todos").h1
    for todo in todos by todo.id {
        Input(bind: todo.title, label: "Title")
        Checkbox(bind: todo.done, label: "Done")
    }
    Text("{todos.filter(todo => !todo.done).length} left")
}
```

The count at the bottom drops as you tick items. The item is written and
the list is told it changed, so everything that reads `todos` follows:

- In a keyed loop (`for … by`), the list hears of the change as the reader
  types.
- In a loop with no `by`, which redraws every item when its list changes,
  the list hears of it when the reader leaves the field, so the row is not
  redrawn under the cursor.

You cannot bind to a constant, a derived value, a prop or a literal: that is
`F01`.

## Add a label, a hint or your own error

Every control takes `label:`, `hint:` and `error:`. With any of them, the
control is wrapped in a field:

- the label is a real `<label for>`;
- the hint and the error are linked to the control with `aria-describedby`;
- the error is a string, empty when there is none. When it appears, it is
  announced, and it sets `aria-invalid` on the control.

This page shows an error while the handle is too short:

```wf
page Handle(path: "/handle", title: "Handle", description: "Pick a handle.") {
    state handle = ""
    derived problem = if handle.length > 0 && handle.length < 3 { "At least three characters" } else { "" }

    Heading("Pick a handle").h1
    Input(bind: handle, label: "Handle", hint: "Letters, numbers and dashes", error: problem)
}
```

For a rule about the value itself, a `validate` block is usually better: it
works out the error for you ([Validate a field](#validate-a-field)). Use
`error:` for a message that comes from somewhere else.

## Group controls in a form

To submit several controls together, put them in a `Form`. Its `on submit`
handler runs when the form is submitted, and the page never navigates away.
`Form(bind: form)` gives you a handle, named `form` here, that knows the
state of the whole form.

This contact form sends its fields, disables **Send** until every field is
valid, and thanks the reader:

```wf
page Contact(path: "/contact", title: "Contact", description: "Send us a message.") {
    state name = ""
    state email = ""
    state message = ""
    state sent = false

    action send() {
        let r = await fetch("/api/contact", { method: "POST", body: { name: name, email: email, message: message } })
        sent = true
        form.reset()
    }

    Heading("Contact us").h1
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

> [!NOTE]
> **Coming from React?** A submit handler there starts with
> `event.preventDefault()`. A `Form`'s `on submit` never reloads or leaves
> the page, so the handler holds only what the submit does. See
> [Coming from React](../coming-from/react.md).

### The form handle

| Member | What it is |
|---|---|
| `form.valid` | `true` when every control passes its own checks and every `validate` rule holds. It stays current as the reader types. |
| `form.errors` | The message of every field that has one, by name |
| `form.touched` | Which fields the reader has left |
| `form.pending` | Whether an `async` rule is still waiting for its answer |
| `form.values` | A map of the controls' values, by their `name:` attribute |
| `form.reset()` | Clears the form, and every message it was showing |
| `form.submit()` | Submits the form from code |
| `form.apply(errors)` | Shows what the server said, on the fields it named ([Show what the server said](#show-what-the-server-said)) |
| `form.element` | The `<form>` element itself |

### Choose which buttons submit

A `Button` with `type: .submit` (or the `.submit` flag) submits the form,
and so does Enter in a field. The browser's own constraint checks run first
(`.required`, `.email` and the rest), so an invalid form never reaches
`on submit`.

Every other button in a form — **Add a row**, **Show password** — is
`type="button"`, and does only what its `on click` says.

## Validate a field

To say what a value must be, put a `validate` block beside the state it
guards. Each rule is one line. The control bound to that state shows the
rules' messages itself: you write no `error:` and no `derived` value.

This page requires an email, a password of at least eight characters with a
number in it, and a matching confirmation:

```wf
page Join(path: "/join", title: "Join", description: "Make an account.") {
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

Leave the password field after typing `abc`, and it shows
`Use at least 8 characters`. **Join** stays disabled until every rule holds.

A `validate` block must guard a state that a control binds. A block for a
state no control binds, or for a derived value, can never pass: that is
`F03`.

### Validation rules

| Rule | Holds when |
|---|---|
| `required` | Something was given: a non-blank string, a non-empty list, a ticked checkbox |
| `email`, `url` | The value looks like an email address, or a URL |
| `minLength(n)`, `maxLength(n)` | The text is at least, or at most, that long |
| `min(v)`, `max(v)` | The number, date or time is within the limit |
| `pattern(/…/)` | The text matches the regular expression |
| `matches(other)` | The value equals another value, and follows it as it changes |
| `oneOf([…])` | The value is one of those listed |
| `custom "…" { expr }` | The expression is true |
| `async "…" { await … }` | The answer is true, once it arrives |

How the rules behave:

- **Messages.** Each rule takes an optional message after it: a string, or
  `t("key")`, which is read in the reader's language. Without one, the rule
  uses its own message. A project with translations replaces those by
  naming the keys `form.required`, `form.email`, `form.minLength` and so on
  ([Translate a site](../recipes/translate-a-site.md),
  [Show validation messages](../recipes/validation-messages.md)).
- **Arguments that read state.** An argument such as `max(Ledger.today)` is
  read again each time the rule is checked.
- **Types.** A rule is checked against the type of what it guards:
  `minLength` on a `Number` is `T01`.
- **Empty values.** Every rule but `required` passes an empty value, so a
  blank optional field shows one message rather than two.
- **Async rules.** An `async` rule is asked only once the other rules pass,
  and once per value. A server is not asked about every keystroke of an
  address that is not valid yet. While it waits, `form.pending` is `true`.

This rule asks the server whether an address is free. The rule passes when the check's answer is `true`, so ask the question whose answer you want to be yes:

```wf
api Backend(base: "/api") {
    get emailFree(email: String) -> Bool
}

page Register(path: "/register", title: "Register", description: "Check an address is free.") {
    state email = ""

    validate email {
        required
        email
        async "That address is taken" { await Backend.emailFree(email: email) }
    }

    Heading("Register").h1
    Form(bind: form) {
        on submit { log("registering") }
        Input(bind: email, label: "Email").email
        Button("Register", type: .submit, disabled: !form.valid || form.pending).primary
    }
}
```

### Choose when a message shows

`Form(show:)` sets when the messages appear:

| `show:` | A field's message appears |
|---|---|
| `.onBlur` (default) | after the reader leaves the field |
| `.onSubmit` | only after a submit |
| `.live` | as the reader types |

The rules run either way, so `form.valid` is always true to the values,
whatever the reader has seen.

A submit that fails shows every message, moves focus to the first field
that has one, announces it, and does not run `on submit`.

### Preview every message at once

To see a form as a failed submit would leave it, without filling it in, set
`data-wf-validation="shown"` on the page's root element:

```js
document.documentElement.setAttribute("data-wf-validation", "shown")
```

Every field then shows what its rules say about its value now. Remove the
attribute and the messages go again. An editor's preview sets this
attribute to show a form as *Invalid*.

### Let a refined type do the checking

A *refined type* is a type with a condition, such as `Number(1..=30)`. The
condition is already a rule, so you only write the rules the type does not
say:

```wf
state nights: Number(1..=30) = 2
validate nights { required }
```

- A range's open end is excluded: `Number(0..10)` does not take `10`.
- A date's `after:` and `before:` are checked as the reader types, like
  `min` and `max`.

See [Types](12-types.md#a-condition-on-a-type) for every condition a type
can carry.

### Read the rules as data

`wf rules --json` lists, for a tool to read:

- every rule, what it takes, its own message and the translation key that
  replaces it;
- the patterns people write, by name: `a number`, `a letter` (in any
  script), `no spaces` …;
- the `show:` choices;
- every state the project holds to a rule, in order, including the rules its
  type implies.

The Rust library has the same data as `webfluent::validation` (with the `internals` feature, outside semver), which can
also write a state's `validate` block back one rule at a time.

### Show what the server said

When the server rejects a value — an address already taken, a code that
expired — pass its messages, by field name, to `form.apply`:

```wf
api Backend(base: "/api") {
    post signUp(body: Map) -> Map
        errors { 422 -> Map }
}

action join() {
    try {
        await Backend.signUp(body: { email: email, password: password })
    } catch e {
        form.apply(e.body.errors)     // { email: "That address is taken" }
    }
}
```

Each message is shown on the field whose `name:` it matches, and moves focus
to the first one. A message stays until the reader changes that field's
value. A field may also be given a list of messages; the first is shown.
[Send a form to a server](#send-a-form-to-a-server) shows the whole page.

## Upload a file

To let the reader pick a file, place a `FileUpload`. The files arrive in its
`change` event. Each is a `File`, with:

- `.name`, `.size` (in bytes) and `.type`;
- `.preview()`, an object URL that shows an image before it is uploaded. It
  is revoked when the page leaves.

This page shows the picture the reader picks, then uploads it with a
progress bar:

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

- An endpoint with a `File` parameter sends a multipart form.
- `Backend.avatar.progress` follows the upload from 0 to 1.
- `FileUpload(…).multiple` lets the reader pick several files.

> [!IMPORTANT]
> `accept:` and any size limit you show are for the reader's convenience,
> not checks. The server decides what it accepts
> ([Security](../about/security.md#uploads-are-checked-on-the-server)).

## Send a form to a server

To send a form, describe the service once with `api`, call it from the
submit handler, and hand what a `422` said back to the form.

```wf
api Backend(base: "/api") {
    post signUp(body: Map) -> Map
        errors { 422 -> Map }
}

page Welcome(path: "/welcome", title: "Join", description: "Make an account.") {
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

`errors { 422 -> Map }` types the body of a `422` response, so
`e.body.errors` is what the server sent. If the server answers
`{ "errors": { "email": "That address is taken" } }`, the message appears
under **Email**.

[Data and APIs](15-data.md) covers the service in full: retries, timeouts
and typed errors.

## Show that a request is running

An action that `await`s has a `.pending` value, `true` while a call of it
runs. Use it to show a spinner, or to keep a button busy with `loading:`.
On a store's action, read it as `Store.action.pending`:

```wf
store Api {
    state saved = 0
    action save(payload: Map) {
        let r = await fetch("/api/save", { method: "POST", body: payload })
        saved = saved + 1
    }
}

page Notes(path: "/notes", title: "Notes", description: "Save a note.") {
    use Api
    state text = ""

    Heading("Notes").h1
    Input(bind: text, label: "Note")
    Button("Save").primary { on click { Api.save({ text: text }) } }
    Button("Save a copy", loading: Api.save.pending) { on click { Api.save({ text: text, copy: true }) } }
    Text("Saved {Api.saved} times")
}
```

- **A button or a form waits for its work.** A `Button`'s `on click` or a
  `Form`'s `on submit` that calls an action that awaits is busy until that
  call ends: `aria-busy`, a press that does nothing, and, past a moment, a
  spinner in place of its label at the same width. The focus stays on it.
  Nothing has to be written for it.
- **A double click submits once.** A call of an action with the same
  arguments as one still running *is* that call: the work is not done
  twice, and both get the same answer. A call with other arguments — the
  next row's save — runs beside it. An action whose every call should run,
  a poll, says so: `action poll() concurrent { … }`.
- **`loading:`** keeps any button busy while a value holds — another
  button's work, a store's state.

Reading `.pending` on an action that awaits nothing is `T17`.

## Choosing: toggles, groups, steps and days

Some choices are not a text field. These built-ins draw them, each with the
keyboard a reader expects:

```wf
page Order(path: "/", title: "Order", description: "Choose how it is sent.") {
    state view = "grid"
    state plan = "free"
    state qty = 1
    state step = 0
    state day: Date? = null
    Heading("Order").h1
    ToggleGroup(bind: view, label: "View") {
        ToggleGroup.Item("Grid", value: "grid", icon: "grid")
        ToggleGroup.Item("List", value: "list", icon: "list")
    }
    RadioGroup(bind: plan, label: "Plan").cards {
        RadioGroup.Option("Free", value: "free", hint: "For trying it out")
        RadioGroup.Option("Team", value: "team", hint: "€8 a seat")
    }
    NumberInput(bind: qty, min: 1, max: 9, label: "Quantity")
    Stepper(bind: step) {
        Stepper.Step("Address") { Text("Where it goes.") }
        Stepper.Step("Delivery") {
            Calendar(bind: day, disabled: d => d.weekday() >= 6, label: "Delivery day")
        }
    }
}
```

- **`Toggle`** is one button that stays pressed; **`ToggleGroup`** is a row
  of them, one chosen (or several, `.multiple`). Tab stops once on the
  group, and the arrow keys move inside it.
- **`RadioGroup`** is native radio buttons under a legend; `.cards` draws
  each as a card with its hint.
- **`NumberInput`** steps with − and +, the arrow keys, and PageUp and
  PageDown, and stays between `min:` and `max:`.
- **`Pagination(bind: page, pages: n)`** draws the pages of a list, with
  gaps for the ones it leaves out.
- **`Stepper`** shows one step at a time and each step's state.
- **`Calendar`** is a month a keyboard can work: the arrows by day and
  week (the other way round right to left), PageUp and PageDown by month.
  `.range` holds `{ start, end }`, `.multiple` a list.

## Checklist for a good form

- Every control has a `label:`. A placeholder is not a label.
- Rules live in `validate` blocks, not in hand-written `derived` errors.
- The submit button is `type: .submit`, so Enter submits.
- The submit button is disabled while the request runs:
  `disabled: save.pending`.
- The page says what happened afterwards, in an `Alert` or a `Toast`.
- Server errors go back to their fields with `form.apply`.
- The server checks everything again. The page's rules are for the reader.

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| `F01` on a `bind:` | The target cannot be written: a constant, a derived value, a prop or a literal. Bind a `state` instead. |
| `F02` on a `bind:` | The control cannot hold the state's type, such as a text `Input` on a `Number`. Use `.number`, or change the state's type. |
| `F03` on a `validate` | No control binds that state, or it is a derived value, so its rules can never pass. |
| `T01` on a rule | The rule does not fit the type, such as `minLength` on a `Number`. Use `min`/`max`. |
| `on submit` never runs | The form is invalid, so the submit stopped. Check `form.errors`, or set `data-wf-validation="shown"` to see every message. |
| A button submits the form by accident | Only `type: .submit` submits. Check that the button is not the submit button. |
| A server message never shows | `form.apply` matches each key to a field's `name:`. Give the control a `name:` equal to the key. |
| A message shows before the reader has typed | The form uses `show: .live`. Use `.onBlur` or `.onSubmit`. |

## Common mistakes

**A text field bound to a number.** An `Input` with no type flag holds a
`String`, so it cannot write a `Number` state:

```wf expect=F02
page P(path: "/", title: "Seats", description: "Pick a number of seats.") {
    state seats = 1
    Heading("Seats").h1
    Input(bind: seats, label: "Seats")
    Text("{seats} seats")
}
```

Add `.number`, which holds a `Number`, or use a `Slider`
([F02](../errors/F02.md)).

**A `validate` block for a state no control binds.** The rules check the
state, and nothing the reader types reaches it:

```wf expect=F03
page P(path: "/", title: "Join", description: "Make an account.") {
    state email = ""
    validate email { required  email }
    Heading("Join").h1
    Input(label: "Email").email
    Text("You typed {email}")
}
```

Bind the control to the state the rules guard: `Input(bind: email,
label: "Email")` ([F03](../errors/F03.md)).

**A `bind:` to a derived value.** A control writes back to what it binds,
and a `derived` value is worked out, never assigned:

```wf expect=F01
page P(path: "/", title: "Handle", description: "Pick a handle.") {
    state first = ""
    derived handle = first.toLowerCase()
    Heading("Handle").h1
    Input(bind: first, label: "First name")
    Input(bind: handle, label: "Handle")
}
```

Bind a `state` instead, and show the derived value with `Text(handle)`
([F01](../errors/F01.md)).

## Next

[Fetching data](15-data.md).
