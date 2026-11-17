# Select

> Drop-down of Select.Option children.

The page: https://webfluent.monzeromer.dev/reference/select

Drop-down of Select.Option children.

```wf
Select(bind: State, value: String or Number, label: String, …) { … }
```

## Example

```wf
state region = "eu-west"
Select(bind: region, label: "Region", hint: "Where your data is stored.") {
    Select.Option("Europe (Ireland)", value: "eu-west")
    Select.Option("US East (Virginia)", value: "us-east")
    Select.Option("Asia Pacific (Tokyo)", value: "ap-northeast")
}
Text("Chosen: {region}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `value:` | `String or Number` | The control's value, when its state is not bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `hint:` | `String` | Help text under the control, linked to it by `aria-describedby` |
| `error:` | `String` | Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid` |
| `disabled:` | `Bool` | Whether the control is inert |
| `multiple:` | `Bool` | Allow several choices |

## Flags

`.disabled` `.multiple`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on change`: The value was committed
- `on focus`: The element received focus
- `on blur`: The element lost focus

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Select`.

### Select.Option

One choice: the visible label, and the value sent with the form.

```wf
Select.Option(_ label: String, value: String or Number or Bool)
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The visible label |
| `value:` | `String or Number or Bool` | The value `bind` takes when this one is chosen; the label when absent |

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [F05](https://webfluent.monzeromer.dev/errors/F05.md): A select whose value is none of its options

Before WebFluent 5.0.
