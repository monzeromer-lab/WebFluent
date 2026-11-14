# PanelGroup.Panel

> One panel.

The page: https://webfluent.monzeromer.dev/reference/panelgroup-panel

One panel.

```wf
PanelGroup.Panel(min: Number) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `min:` | `Number` | The least it may be, in percent (unset, 10) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
