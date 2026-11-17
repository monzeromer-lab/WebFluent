# ToggleGroup

> Buttons of which one is chosen — a view switcher, a segmented control — or, .multiple, several: one stop for Tab, the arrow keys inside it, each aria-pressed. bind: holds the chosen value: (a list with .multiple).

The page: https://webfluent.monzeromer.dev/reference/togglegroup

Buttons of which one is chosen — a view switcher, a segmented control — or, .multiple, several: one stop for Tab, the arrow keys inside it, each aria-pressed. bind: holds the chosen value: (a list with .multiple).

```wf
ToggleGroup(bind: State, label: String, multiple: Bool) { … }
```

## Example

```wf
state view = "grid"
ToggleGroup(bind: view, label: "View") {
    ToggleGroup.Item("Grid", value: "grid", icon: "grid")
    ToggleGroup.Item("List", value: "list", icon: "list")
}
Text("Showing the {view} view").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The chosen value, or a list of them with `.multiple` |
| `label:` | `String  (required)` | What the group chooses, for a screen reader |
| `multiple:` | `Bool` | Several at once: `bind:` is a list |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `ToggleGroup`.

### ToggleGroup.Item

One choice: its words, and the value bind: holds when it is chosen.

```wf
ToggleGroup.Item(_ label: String, value: String or Number or Bool, icon: String, disabled: Bool)
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `value:` | `String or Number or Bool  (required)` | What `bind:` holds when it is chosen |
| `icon:` | `String` | An icon before the words |
| `disabled:` | `Bool` | Whether the control is inert |

Flags: `.disabled`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
