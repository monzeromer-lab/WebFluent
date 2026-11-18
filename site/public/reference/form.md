# Form

> Groups controls; on submit runs when it is submitted, and the page never navigates away.

The page: https://webfluent.monzeromer.dev/reference/form

Groups controls; on submit runs when it is submitted, and the page never navigates away.

```wf
Form(bind: State, show: Enum) { … }
```

## Example

```wf
state email = ""
state password = ""
state sent = ""
validate email {
    required "Enter your email"
    email "That is not an email address"
}
validate password {
    required
    minLength(8) "Use at least 8 characters"
}
Form(bind: form, show: .onBlur) {
    on submit { sent = "Signed in as {email}" }
    Input(bind: email, label: "Email").email
    Input(bind: password, label: "Password").password
    Button("Sign in", type: .submit, disabled: !form.valid).primary
}
Text(if sent == "" { "Not signed in yet." } else { sent }).muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A name for a handle on the form, not a state: `form.valid`, `form.errors`, `form.touched`, `form.pending`, `form.values` by field name, `form.reset()`, `form.submit()`, `form.apply(serverErrors)` |
| `show:` | `one of its cases` | When a `validate` block's message is shown on its field. Written as a flag, `.onBlur`, or by name, `show: .onBlur`.

- `.onBlur` (the default): Once the reader has left the field (the default)
- `.onSubmit`: Only after a submit
- `.live`: As they type |

## Flags

`.onBlur` `.onSubmit` `.live`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on submit`: The form was submitted
- `on reset`: The form was reset

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
