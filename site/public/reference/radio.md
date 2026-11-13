# Radio

> One choice of a group; every radio bound to the same state is one group.

The page: https://webfluent.monzeromer.dev/reference/radio

One choice of a group; every radio bound to the same state is one group.

```wf
Radio(bind: State, value: Any, checked: Bool, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `value:` | `Any` | The value `bind` takes when this one is chosen |
| `checked:` | `Bool` | Whether it is checked, when its state is not bound |
| `label:` | `String` | Visible label; also the control's accessible name |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.checked` `.disabled` `.dropInside` `.trapFocus`

## Example

```wf
state size = "m"
Radio(bind: size, value: "s", label: "Small")
Radio(bind: size, value: "m", label: "Medium")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A04](https://webfluent.monzeromer.dev/errors/A04.md): A control with no label

Since WebFluent 5.0.
