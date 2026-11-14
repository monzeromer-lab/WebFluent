# Select

> Drop-down of Select.Option children.

The page: https://webfluent.monzeromer.dev/reference/select

Drop-down of Select.Option children.

```wf
Select(bind: State, value: Any, label: String, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `value:` | `Any` | The control's value, when its state is not bound |
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

- `Select.Select.Option`

## Example

```wf
state plan = "free"
Select(bind: plan, label: "Plan") {
    Select.Option("Free", value: "free")
    Select.Option("Team", value: "team")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [F05](https://webfluent.monzeromer.dev/errors/F05.md): A select whose value is none of its options

Before WebFluent 5.0.
