# InputOTP

> A one-time code typed a digit to a box: the focus moves on as each is typed and back with Backspace, and a code pasted into any box fills them all; the browser offers the code it was sent (autocomplete="one-time-code").

The page: https://webfluent.monzeromer.dev/reference/inputotp

A one-time code typed a digit to a box: the focus moves on as each is typed and back with Backspace, and a code pasted into any box fills them all; the browser offers the code it was sent (autocomplete="one-time-code").

```wf
InputOTP(bind: State, length: Number, label: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The code, as text |
| `length:` | `Number` | How many digits (unset, 6) |
| `label:` | `String  (required)` | What the code is, for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state code = ""
InputOTP(bind: code, length: 6, label: "Code from your app")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
