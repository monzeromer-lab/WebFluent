# PanelGroup

> Panels side by side (or .vertical, one above another) with a handle between each two that resizes them: dragged, or the arrow keys on it. bind: holds their sizes in percent — a persist keeps them across visits — and on resize(e) hears them as e.detail.

The page: https://webfluent.monzeromer.dev/reference/panelgroup

Panels side by side (or .vertical, one above another) with a handle between each two that resizes them: dragged, or the arrow keys on it. bind: holds their sizes in percent — a persist keeps them across visits — and on resize(e) hears them as e.detail.

```wf
PanelGroup(bind: State, vertical: Bool, label: String) { … }
```

## Example

```wf
state sizes: [Number] = [30, 70]
Text("Sizes: {sizes[0] ?? 0}% and {sizes[1] ?? 0}%").muted.sm
PanelGroup(bind: sizes, label: "Resize the sidebar") {
    style { height: 200px; border: 1px solid $color-border; border-radius: $radius-md }
    PanelGroup.Panel(min: 15) {
        Stack(gap: .xs) {
            style { padding: 0.75rem }
            Text("Inbox").bold
            Text("Archive")
            Text("Spam")
        }
    }
    PanelGroup.Panel(min: 40) {
        Text("Drag the handle, or use the arrow keys on it.") { style { padding: 0.75rem } }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | A list of Numbers, each panel's size in percent, `[30, 70]`, written as they move; a list of another length is ignored |
| `vertical:` | `Bool` | Panels one above another; the handles move up and down. Written as a flag, `.vertical`, or `vertical: true`. |
| `label:` | `String` | Each handle's accessible name (`aria-label`): what it resizes |

## Flags

`.vertical`

## Events

- `resize`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `PanelGroup`.

### PanelGroup.Panel

One panel.

```wf
PanelGroup.Panel(min: Number, collapsible: Bool) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `min:` | `Number` | The least it may be, in percent of the group |
| `collapsible:` | `Bool` | Enter or a double press on the handle after it folds it away, its share to its neighbour; again brings it back. Written as a flag, `.collapsible`, or `collapsible: true`. |

Flags: `.collapsible`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
