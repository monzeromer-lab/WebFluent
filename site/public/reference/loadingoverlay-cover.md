# LoadingOverlay.Cover

> What covers it while it is busy. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/loadingoverlay-cover

What covers it while it is busy. Drawn for it.

```wf
LoadingOverlay.Cover { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state saving = false
Button("Save") { on click { saving = !saving } }
LoadingOverlay(visible: saving, label: "Saving") {
    Card { Text("Your profile") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
