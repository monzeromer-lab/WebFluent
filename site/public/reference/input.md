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
| `reveal:` | `Bool` | With `.password`: a Show/Hide button after the field that shows the password as text and hides it again, pressed (`aria-pressed`) while it shows. Written as a flag, `.reveal`, or `reveal: true`. |
| `bind:` | `State` | The state it reads and writes as you type: a `String`, a `Number` with `.number` (`null` when emptied), a `Date` with `.date`, a `Time` with `.time`, a `DateTime` with `.datetime` |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `type:` | `one of its cases` | What the field holds, and so the keyboard and picker a browser offers; it also sets what `bind:` holds. Written as a flag, `.text`, or by name, `type: .text`.

- `.text` (the default): Plain text
- `.email`: An email address
- `.password`: A password, its characters hidden
- `.number`: A number: `bind:` holds a `Number`
- `.search`: A search query
- `.tel`: A phone number
- `.url`: A URL
- `.date`: A day, from the browser's date picker: `bind:` holds a `Date`
- `.time`: A time of day: `bind:` holds a `Time`
- `.datetime`: A day and a time (`datetime-local`): `bind:` holds a `DateTime`
- `.color`: A colour, from the browser's picker, as `#rrggbb` |
| `placeholder:` | `String` | Hint shown while the field is empty |
| `value:` | `String or Number` | The text it starts with, when no state is bound; `bind:` takes its place |
| `min:` | `Number or String` | The lowest value a `.number`, `.date` or `.time` field takes: a number, or a date or time as text (`"2026-01-01"`) |
| `max:` | `Number or String` | The highest value a `.number`, `.date` or `.time` field takes: a number, or a date or time as text |
| `step:` | `Number` | How far the arrow keys and the spinner move a `.number` value (seconds for `.time`) |
| `required:` | `Bool` | Whether the browser refuses to submit the form while it is empty. Written as a flag, `.required`, or `required: true`. |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it. Written as a flag, `.disabled`, or `disabled: true`. |
| `size:` | `one of its cases` | Its padding and text size. Written as a flag, `.sm`, or by name, `size: .sm`.

- `.sm`: Less padding, smaller text
- `.md` (the default): The default size
- `.lg`: More padding, larger text |
| `width:` | `one of its cases` | Its width. Written as a flag, `.full`, or by name, `width: .full`.

- `.full`: The whole width of its container (an input already is, outside a row) |
| `rounded:` | `Bool` | Fully rounded ends, a pill shape. Written as a flag, `.rounded`, or `rounded: true`. |

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
