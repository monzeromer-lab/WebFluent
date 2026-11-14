# Textarea

> Several lines of text. With label, hint or error it is a labelled field; with maxLength it counts what is left.

The page: https://webfluent.monzeromer.dev/reference/textarea

Several lines of text. With label, hint or error it is a labelled field; with maxLength it counts what is left.

```wf
Textarea(bind: State, label: String, hint: String, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `placeholder:` | `String` | Hint shown while the field is empty |
| `rows:` | `Number` | How many lines it shows |
| `maxLength:` | `Number` | The most characters it takes |
| `required:` | `Bool` | Whether the form refuses to submit without it |
| `autosize:` | `Bool` | Grows with its text, from `rows:` up |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.required` `.autosize` `.disabled`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on input`: The value changed, on every keystroke
- `on change`: The value was committed
- `on keydown`: A key went down; `e.key` names it
- `on focus`: The element received focus
- `on blur`: The element lost focus

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state note = ""
Textarea(bind: note, label: "Notes", rows: 4, maxLength: 200)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A04](https://webfluent.monzeromer.dev/errors/A04.md): A control with no label

Since WebFluent 6.2.0.
