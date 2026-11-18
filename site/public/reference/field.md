# Field

> A label, a hint and an error around a control of your own: the first control in its block is named by the label and described by the hint and the error, and marked invalid while there is one.

The page: https://webfluent.monzeromer.dev/reference/field

A label, a hint and an error around a control of your own: the first control in its block is named by the label and described by the hint and the error, and marked invalid while there is one.

```wf
Field(label: String, hint: String, error: String) { … }
```

## Example

```wf
state color = "#0f766e"
derived colorError = if color == "#ffffff" { "White disappears on the page." } else { "" }
Field(label: "Brand colour", hint: "Used for buttons and links.", error: colorError) {
    ColorPicker(bind: color, label: "Brand colour", swatches: ["#0f766e", "#1d4ed8", "#b91c1c", "#ffffff"])
}
Text("Chosen: {color}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The label above the control; it names the first control in the block (`aria-labelledby`), and a press on it focuses that control |
| `hint:` | `String` | Help text under the control, linked to the first control in the block by `aria-describedby` |
| `error:` | `String` | Error text under the control: `null` or empty for none; while there is one it shows, is announced, and marks the control `aria-invalid` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Drawn for it

What `Field` draws itself, by its class; not written.

- `Field.Label` (`.`): Its label.
- `Field.Hint` (`.`): Its hint.
- `Field.Error` (`.`): Its error, announced as it appears.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
