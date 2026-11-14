# Form

> Groups controls; on submit runs when it is submitted, and the page never navigates away.

The page: https://webfluent.monzeromer.dev/reference/form

Groups controls; on submit runs when it is submitted, and the page never navigates away.

```wf
Form(bind: State, show: Enum) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A handle on the form: `form.valid`, `form.errors`, `form.values` by field name, `form.reset()`, `form.apply(serverErrors)` |
| `show:` | `.onBlur .onSubmit .live` | When a `validate` block's message is shown |

## Flags

`.onBlur` `.onSubmit` `.live`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on submit`: The form was submitted
- `on reset`: The form was reset

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state name = ""
Form(bind: form) {
    on submit { log(name) }
    Input(bind: name, label: "Name").required
    Button("Save", type: .submit, disabled: !form.valid).primary
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
