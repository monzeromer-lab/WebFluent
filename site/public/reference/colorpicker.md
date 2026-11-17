# ColorPicker

> A colour, chosen from swatches: (each a button, the chosen one pressed), with the browser's own picker, or typed as hex; bind: holds it as #rrggbb.

The page: https://webfluent.monzeromer.dev/reference/colorpicker

A colour, chosen from swatches: (each a button, the chosen one pressed), with the browser's own picker, or typed as hex; bind: holds it as #rrggbb.

```wf
ColorPicker(bind: State, label: String, swatches: [String])
```

## Example

```wf
state accent = "#0f766e"
ColorPicker(bind: accent, label: "Accent colour", swatches: ["#0f766e", "#1d4ed8", "#7c3aed", "#b91c1c", "#c2410c"])
Button("Preview").primary { style { background: {accent} } }
Text("Chosen: {accent}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The colour, `#rrggbb` |
| `label:` | `String  (required)` | What the colour is for |
| `swatches:` | `[String]` | The colours offered first: a list of `#rrggbb` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
