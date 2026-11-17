# Chip

> A compact label — a filter, a tag, a person — that can carry an icon, be pressed (selected:, with on click), and be removed (.removable, with on remove). Tag is the one that does none of it.

The page: https://webfluent.monzeromer.dev/reference/chip

A compact label — a filter, a tag, a person — that can carry an icon, be pressed (selected:, with on click), and be removed (.removable, with on remove). Tag is the one that does none of it.

```wf
Chip(_ label: String, icon: String, selected: Bool, removable: Bool)
```

## Example

```wf
state langs = ["Rust", "TypeScript", "Go", "Python"]
state openOnly = false
Stack(gap: .sm, align: .start) {
    Row(gap: .xs) {
        for l in langs by l {
            Chip(l, icon: "code").removable { on remove { langs = langs.filter(x => x != l) } }
        }
    }
    Chip("Open issues only", selected: openOnly) { on click { openOnly = !openOnly } }
    Text("{langs.length} languages, open only: {openOnly}").muted
}
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

Written under its name, inside `Chip`.

### Chip.Action

What pressing a chip presses: its icon and words, a toggle button when it can be selected:.

```wf
Chip.Action(_ text: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | Its words |

### Chip.Label

Its icon and words, when pressing it does nothing.

```wf
Chip.Label(_ text: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | Its words |

### Chip.Remove

Its remove button.

```wf
Chip.Remove { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
