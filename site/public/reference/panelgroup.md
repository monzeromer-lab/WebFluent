# PanelGroup

> Panels side by side (or .vertical, one above another) with a handle between each two that resizes them: dragged, or the arrow keys on it. bind: holds their sizes in percent — a persist keeps them across visits — and on resize(e) hears them as e.detail.

The page: https://webfluent.monzeromer.dev/reference/panelgroup

Panels side by side (or .vertical, one above another) with a handle between each two that resizes them: dragged, or the arrow keys on it. bind: holds their sizes in percent — a persist keeps them across visits — and on resize(e) hears them as e.detail.

```wf
PanelGroup(bind: State, vertical: Bool, label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The panels' sizes, in percent: `[30, 70]` |
| `vertical:` | `Bool` | One above another |
| `label:` | `String` | What the handles resize, for a screen reader |

## Flags

`.vertical`

## Events

- `resize`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `PanelGroup.PanelGroup.Panel`

## Example

```wf
state sizes: [Number] = [30, 70]
PanelGroup(bind: sizes, label: "Resize the panels") {
    style { height: 160px }
    PanelGroup.Panel(min: 20) { Text("Files") }
    PanelGroup.Panel { Text("Editor") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
