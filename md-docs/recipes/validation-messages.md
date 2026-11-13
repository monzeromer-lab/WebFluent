---
kind: "recipe"
route: "/recipes/validation-messages"
group: "data"
description: "Check a form as the reader fills it in: a validate block beside each state, and each control shows its own message."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#show-validation-messages-on-a-form"]
---
# Show validation messages on a form

A form should say what is wrong beside the field that is wrong, before the
reader submits it. To check a form as the reader fills it in, put a
`validate` block beside each state:

```wf
page Signup(path: "/", title: "Sign up", description: "Make an account.") {
    state email = ""
    state password = ""
    state confirm = ""
    state sent = false
    validate email { required  email "Enter a valid email" }
    validate password { required  minLength(8) "At least 8 characters" }
    validate confirm { matches(password) "The passwords differ" }
    Heading("Sign up").h1
    Form(bind: form) {
        on submit { sent = true  form.reset() }
        Input(bind: email, name: "email", label: "Email").email
        Input(bind: password, name: "password", label: "Password").password
        Input(bind: confirm, name: "confirm", label: "Confirm password").password
        Button("Create account", type: .submit, disabled: !form.valid).primary
    }
    if sent { Alert("Welcome aboard.").success }
}
```

## How it works

- Each [`validate`](../learn/14-forms.md#validate-a-field) block names a
  state and lists its rules. A rule may take a message after it; without
  one, the rule's own is used.
- `matches(password)` holds `confirm` to whatever `password` is now.
- The control bound to a state shows that state's message. Each control
  shows its own message when the reader leaves it, with the accessible
  wiring an `error:` gets.
- `Form(bind: form)` hands the page a handle: `form.valid` disables the
  button until every rule passes, and `form.reset()` clears the form after
  a submit.
- A submit that fails moves focus to the first problem and does not run
  the `on submit` handler.

[Forms](../learn/14-forms.md) lists every rule.
