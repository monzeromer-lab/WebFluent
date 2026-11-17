# Input

> Text field. With label, hint or error it is a labelled field.

The page: https://webfluent.monzeromer.dev/reference/input

Text field. With label, hint or error it is a labelled field.

```wf
Input(reveal: Bool, bind: State, label: String, …)
```

## Example

```wf
state name = "Ada Lovelace"
state email = ""
Stack(gap: .md) {
    Input(bind: name, label: "Full name", placeholder: "Ada Lovelace").text
    Input(bind: email, label: "Work email", hint: "We send the invoice here.", placeholder: "ada@example.com").email.required
    Text(if email == "" { "Hello, {name}" } else { "Hello, {name}: we write to {email}" }).muted
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `reveal:` | `Bool` | A password's button to show it and hide it again, said to a screen reader |
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `type:` | `.text .email .password .number .search .tel .url .date .time .datetime .color` | What the field holds |
| `placeholder:` | `String` | Hint shown while the field is empty |
| `value:` | `String or Number` | The control's value, when its state is not bound |
| `min:` | `Number or String` | Lowest value |
| `max:` | `Number or String` | Highest value |
| `step:` | `Number` | Granularity |
| `required:` | `Bool` | Whether the form refuses to submit without it |
| `disabled:` | `Bool` | Whether the control is inert |
| `size:` | `.sm .md .lg` | Its size |
| `width:` | `.full` | Width |
| `rounded:` | `Bool` | Rounded corners |

## Flags

`.reveal` `.text` `.email` `.password` `.number` `.search` `.tel` `.url` `.date` `.time` `.datetime` `.color` `.required` `.disabled` `.sm` `.md` `.lg` `.full` `.rounded`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on input`: The value changed, on every keystroke
- `on change`: The value was committed
- `on keydown`: A key went down; `e.key` names it
- `on focus`: The element received focus
- `on blur`: The element lost focus

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [F01](https://webfluent.monzeromer.dev/errors/F01.md): A bind: to something that cannot be written
- [F02](https://webfluent.monzeromer.dev/errors/F02.md): A bind: to a state the control cannot hold
- [F03](https://webfluent.monzeromer.dev/errors/F03.md): A validate block no control can satisfy
- [A03](https://webfluent.monzeromer.dev/errors/A03.md): An input with no label

Before WebFluent 5.0.
