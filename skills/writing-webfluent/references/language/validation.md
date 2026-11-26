### Validation

A `validate` block declares the rules a state must meet; the control bound to it shows the messages.

A `validate` block sits beside the state it guards. The control bound to
that state then shows what the rules say — no `error:` to write, and the
accessible plumbing (`role="alert"`, `aria-invalid`, `aria-describedby`)
is the same one `error:` has always used.

```wf
type SignUp { email: String, password: String }

api Backend(base: "/api") {
    get emailTaken(email: String) at "emailTaken" -> Bool!
    post signUp(body: SignUp) at "signUp" -> !
}

page Signup(path: "/join", title: "Join", description: "Make an account.") {
    state email = ""
    state password = ""
    state confirm = ""
    state note = ""

    validate email {
        required
        email
        async "That address is taken" {
            match await Backend.emailTaken(email: email) { .ok(taken) { !taken } else { false } }
        }
    }
    validate password {
        required
        minLength(8) "Use at least 8 characters"
        pattern(/[0-9]/) "Include a number"
    }
    validate confirm { matches(password) "The two passwords differ" }

    Heading("Join").h1
    Form(bind: form, show: .onBlur) {        // .onBlur (default) · .onSubmit · .live
        on submit { Backend.signUp(body: SignUp(email: email, password: password)) }
        Input(bind: email, label: "Email").email
        Input(bind: password, label: "Password").password
        Input(bind: confirm, label: "Confirm").password
        Textarea(bind: note, label: "Anything else?", rows: 4, maxLength: 200)
        Button("Join", type: .submit, disabled: !form.valid || form.pending).primary
    }
}
```

The rules: `required`, `email`, `url`, `minLength(n)`, `maxLength(n)`,
`min(v)`, `max(v)`, `pattern(/…/)`, `matches(other)`, `oneOf([…])`,
`custom "…" { expr }`, `async "…" { await … }`. Each takes an optional
message — a string, or `t("key")` read in the reader's language; without
one the rule's own is used, which a project's translations replace by
naming `form.required`, `form.email` and so on. An argument that reads
state — `max(Ledger.today)` — is read each time the rule is checked.
A rule is checked against what it guards — `minLength` on a `Number` is
`T01` — and every rule but `required` passes an empty value, so a blank
optional field is one message, not two.

**A refined type validates itself.** `state nights: Number(1..=30)` needs
no rule written for the range; the condition on the type is already one.

A form holds the rules of the states its own controls bind, and no
others: with one `Form` a step, a rule on a later step's field does not
hold back the first. `form.valid`, `form.errors` and the submit are that
form's fields alone. A submit that fails shows every message of its
fields, moves focus to the first field that has one and announces it, and
does not run the handler.

<!-- core -->
