# Modal.Footer

> The modal's action row.

The page: https://webfluent.monzeromer.dev/reference/modal-footer

The modal's action row.

```wf
Modal.Footer { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state open = false
Button("Delete") { on click { open = true } }
Modal(visible: open, title: "Delete deploys.wf?").closable {
    Modal.Title { Markdown("Delete `deploys.wf`?") }
    Text("This cannot be undone.")
    Modal.Footer { Button("Cancel") { on click { open = false } } }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
