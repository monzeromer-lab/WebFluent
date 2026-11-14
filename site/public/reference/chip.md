# Chip

> A compact label — a filter, a tag, a person — that can carry an icon, be pressed (selected:, with on click), and be removed (.removable, with on remove). Tag is the one that does none of it.

The page: https://webfluent.monzeromer.dev/reference/chip

A compact label — a filter, a tag, a person — that can carry an icon, be pressed (selected:, with on click), and be removed (.removable, with on remove). Tag is the one that does none of it.

```wf
Chip(_ label: String, icon: String, selected: Bool, removable: Bool)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `icon:` | `String` | An icon before the words |
| `selected:` | `Bool` | Pressed: a filter that is on. Makes it a toggle button |
| `removable:` | `Bool` | A button that removes it, named `Remove {label}`, firing `on remove` |

## Flags

`.selected` `.removable`

## Events

- `remove`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Parts

- `Chip.Chip.Action`
- `Chip.Chip.Label`
- `Chip.Chip.Remove`

## Example

```wf
state open = false
Chip("Open only", selected: open) { on click { open = !open } }
Chip("Rust", icon: "code").removable { on remove { log("removed") } }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
