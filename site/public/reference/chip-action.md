# Chip.Action

> What pressing a chip presses: its icon and words, a toggle button when it can be selected:.

The page: https://webfluent.monzeromer.dev/reference/chip-action

What pressing a chip presses: its icon and words, a toggle button when it can be selected:.

```wf
Chip.Action(_ text: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | Its words |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state open = false
Chip("Open only", selected: open) { on click { open = !open } }
Chip("Rust", icon: "code").removable { on remove { log("removed") } }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
