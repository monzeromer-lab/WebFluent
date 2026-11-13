---
kind: "recipe"
route: "/recipes/validation-messages"
group: "data"
description: "To check a form as the reader fills it in, put a `validate` block beside each state:"
from: ["/docs/guide/recipes#show-validation-messages-on-a-form"]
---
# Show validation messages on a form

To check a form as the reader fills it in, put a `validate` block beside
each state:

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

Each control shows its own message when the reader leaves it, and a submit
that fails moves focus to the first problem. [Forms](../learn/14-forms.md) lists
every rule.
